//! Gamepad support through gilrs (XInput / Windows.Gaming.Input on Windows, evdev on Linux).
//!
//! The game buttons (fire, bomb, ability, precision) are remappable (see `controls`). The
//! menu buttons are fixed: A or Start confirms, B or Select goes back, Start pauses.
//! Vibration uses force feedback when the gamepad and the driver support it.

use crate::controls::{PAD_BUTTONS, PadButton};
use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Replay, Ticks};
use gilrs::{Axis, Button, GamepadId, Gilrs};
use std::time::{Duration, Instant};

const DEAD_ZONE: f32 = 0.4;
const REPEAT_DELAY: f32 = 0.35;
const REPEAT_RATE: f32 = 0.075;

/// Snapshot of one gamepad for the current frame.
#[derive(Default, Clone, Copy)]
pub struct PadState {
    pub connected: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    /// Game buttons held, indexed like `controls::PAD_BUTTONS`.
    pub held: [bool; 8],
    /// Game buttons pressed this frame.
    pub pressed: [bool; 8],
    pub confirm_p: bool,
    pub back_p: bool,
    pub start_p: bool,
    pub up_p: bool,
    pub down_p: bool,
    pub left_p: bool,
    pub right_p: bool,
    pub any_p: bool,
}

impl PadState {
    /// The first game button pressed this frame, if any (used to rebind).
    pub fn first_pressed(&self) -> Option<PadButton> {
        PAD_BUTTONS
            .iter()
            .copied()
            .find(|b| self.pressed[b.index()])
    }
}

#[derive(Default, Clone, Copy)]
struct Held {
    buttons: [bool; 8],
    menu: [bool; 3],
    dirs: [f32; 4],
}

pub struct Pads {
    gilrs: Option<Gilrs>,
    order: Vec<GamepadId>,
    held: [Held; 2],
    pub state: [PadState; 2],
    /// Vibration effects still playing (they stop when dropped).
    effects: Vec<(Effect, Instant)>,
}

fn button_of(b: PadButton) -> [Button; 2] {
    match b {
        PadButton::South => [Button::South, Button::South],
        PadButton::East => [Button::East, Button::East],
        PadButton::West => [Button::West, Button::West],
        PadButton::North => [Button::North, Button::North],
        PadButton::LB => [Button::LeftTrigger, Button::LeftTrigger],
        PadButton::RB => [Button::RightTrigger, Button::RightTrigger],
        PadButton::LT => [Button::LeftTrigger2, Button::LeftTrigger2],
        PadButton::RT => [Button::RightTrigger2, Button::RightTrigger2],
    }
}

impl Pads {
    pub fn new() -> Pads {
        Pads {
            // A missing driver or permission problem must never stop the game.
            gilrs: Gilrs::new().ok(),
            order: vec![],
            held: [Held::default(); 2],
            state: [PadState::default(); 2],
            effects: vec![],
        }
    }

    /// Number of gamepads currently connected.
    pub fn count(&self) -> usize {
        self.state.iter().filter(|s| s.connected).count()
    }

    pub fn update(&mut self, dt: f32) {
        let now = Instant::now();
        self.effects.retain(|(_, until)| *until > now);
        let Some(g) = &mut self.gilrs else { return };
        while g.next_event().is_some() {}
        // Keep a stable order: the first pad connected is the first slot.
        let ids: Vec<GamepadId> = g
            .gamepads()
            .filter(|(_, p)| p.is_connected())
            .map(|(id, _)| id)
            .collect();
        self.order.retain(|id| ids.contains(id));
        for id in ids {
            if !self.order.contains(&id) {
                self.order.push(id);
            }
        }
        for slot in 0..2 {
            let Some(&id) = self.order.get(slot) else {
                self.state[slot] = PadState::default();
                self.held[slot] = Held::default();
                continue;
            };
            let p = g.gamepad(id);
            let lx = p.value(Axis::LeftStickX);
            let ly = p.value(Axis::LeftStickY);
            let left = p.is_pressed(Button::DPadLeft) || lx < -DEAD_ZONE;
            let right = p.is_pressed(Button::DPadRight) || lx > DEAD_ZONE;
            let up = p.is_pressed(Button::DPadUp) || ly > DEAD_ZONE;
            let down = p.is_pressed(Button::DPadDown) || ly < -DEAD_ZONE;
            let mut buttons = [false; 8];
            for (i, b) in PAD_BUTTONS.iter().enumerate() {
                buttons[i] = button_of(*b).iter().any(|x| p.is_pressed(*x));
            }
            let menu = [
                p.is_pressed(Button::Start),
                p.is_pressed(Button::Select),
                p.is_pressed(Button::South),
            ];
            let prev = self.held[slot];
            let mut pressed = [false; 8];
            for i in 0..8 {
                pressed[i] = buttons[i] && !prev.buttons[i];
            }
            let menu_edge = |i: usize| menu[i] && !prev.menu[i];
            let east_p = pressed[PadButton::East.index()];
            let mut dirs = prev.dirs;
            let mut rep = |i: usize, on: bool| -> bool {
                if !on {
                    dirs[i] = 0.0;
                    return false;
                }
                let before = dirs[i];
                dirs[i] += dt;
                if before == 0.0 {
                    return true;
                }
                if dirs[i] > REPEAT_DELAY {
                    let a = ((before - REPEAT_DELAY) / REPEAT_RATE).floor();
                    let b = ((dirs[i] - REPEAT_DELAY) / REPEAT_RATE).floor();
                    return before <= REPEAT_DELAY || b > a;
                }
                false
            };
            let up_p = rep(0, up);
            let down_p = rep(1, down);
            let left_p = rep(2, left);
            let right_p = rep(3, right);
            self.state[slot] = PadState {
                connected: true,
                left,
                right,
                up,
                down,
                held: buttons,
                pressed,
                confirm_p: menu_edge(2) || menu_edge(0),
                back_p: east_p || menu_edge(1),
                start_p: menu_edge(0),
                up_p,
                down_p,
                left_p,
                right_p,
                any_p: pressed.iter().any(|b| *b)
                    || (0..3).any(menu_edge)
                    || up_p
                    || down_p
                    || left_p
                    || right_p,
            };
            self.held[slot] = Held {
                buttons,
                menu,
                dirs,
            };
        }
    }

    /// Vibrates the gamepad in `slot` (0 or 1). `strength` goes from 0 to 1.
    pub fn rumble(&mut self, slot: usize, strength: f32, ms: u32) {
        let Some(g) = &mut self.gilrs else { return };
        let Some(&id) = self.order.get(slot) else {
            return;
        };
        if !g.gamepad(id).is_ff_supported() {
            return;
        }
        let mag = (strength.clamp(0.0, 1.0) * u16::MAX as f32) as u16;
        let play = Ticks::from_ms(ms);
        let effect = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong { magnitude: mag },
                scheduling: Replay {
                    play_for: play,
                    ..Default::default()
                },
                ..Default::default()
            })
            .add_effect(BaseEffect {
                kind: BaseEffectType::Weak { magnitude: mag },
                scheduling: Replay {
                    play_for: play,
                    ..Default::default()
                },
                ..Default::default()
            })
            .gamepads(&[id])
            .finish(g);
        if let Ok(e) = effect
            && e.play().is_ok()
        {
            let until = Instant::now() + Duration::from_millis(ms as u64 + 50);
            self.effects.push((e, until));
            if self.effects.len() > 8 {
                self.effects.remove(0);
            }
        }
    }
}
