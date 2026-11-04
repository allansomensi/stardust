//! Gamepad support through gilrs (XInput / Windows.Gaming.Input on Windows, evdev on Linux).
//!
//! Layout (Xbox names): stick or D-pad move, A fires, X drops a bomb, Y or RB uses the ship
//! ability, LB is precision mode, Start pauses. In menus, A confirms and B goes back.

use gilrs::{Axis, Button, GamepadId, Gilrs};

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
    pub fire: bool,
    pub fire_p: bool,
    pub bomb_p: bool,
    pub ability_p: bool,
    pub focus: bool,
    pub confirm_p: bool,
    pub back_p: bool,
    pub start_p: bool,
    pub up_p: bool,
    pub down_p: bool,
    pub left_p: bool,
    pub right_p: bool,
    pub any_p: bool,
}

#[derive(Default, Clone, Copy)]
struct Held {
    buttons: [bool; 8],
    dirs: [f32; 4],
}

pub struct Pads {
    gilrs: Option<Gilrs>,
    order: Vec<GamepadId>,
    held: [Held; 2],
    pub state: [PadState; 2],
}

impl Pads {
    pub fn new() -> Pads {
        Pads {
            // A missing driver or permission problem must never stop the game.
            gilrs: Gilrs::new().ok(),
            order: vec![],
            held: [Held::default(); 2],
            state: [PadState::default(); 2],
        }
    }

    /// Number of gamepads currently connected.
    pub fn count(&self) -> usize {
        self.state.iter().filter(|s| s.connected).count()
    }

    pub fn update(&mut self, dt: f32) {
        let Some(g) = &mut self.gilrs else { return };
        while g.next_event().is_some() {}
        // Keep a stable order: the first pad connected is player 1.
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
            let buttons = [
                p.is_pressed(Button::South) || p.is_pressed(Button::RightTrigger2),
                p.is_pressed(Button::East),
                p.is_pressed(Button::West),
                p.is_pressed(Button::North) || p.is_pressed(Button::RightTrigger),
                p.is_pressed(Button::LeftTrigger) || p.is_pressed(Button::LeftTrigger2),
                p.is_pressed(Button::Start),
                p.is_pressed(Button::Select),
                p.is_pressed(Button::South),
            ];
            let prev = self.held[slot];
            let edge = |i: usize| buttons[i] && !prev.buttons[i];
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
            let s = PadState {
                connected: true,
                left,
                right,
                up,
                down,
                fire: buttons[0],
                fire_p: edge(0),
                bomb_p: edge(2) || edge(1),
                ability_p: edge(3),
                focus: buttons[4],
                confirm_p: edge(7) || edge(5),
                back_p: edge(1) || edge(6),
                start_p: edge(5),
                up_p,
                down_p,
                left_p,
                right_p,
                any_p: (0..8).any(edge) || up_p || down_p || left_p || right_p,
            };
            self.state[slot] = s;
            self.held[slot] = Held { buttons, dirs };
        }
    }
}
