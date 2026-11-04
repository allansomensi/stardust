//! The Swarm's elite guard, found in the later levels of the campaign:
//!
//! - ARMORED: plated all over. Shots from the front only chip the armor; hits from behind
//!   (bouncing, homing or returning shots), explosions and the railgun go straight through.
//! - HEALER: pulses every few seconds, mending the enemies and armor around it.
//! - MIRROR: while its mirror is open, it sends your shots back at you.
//! - LEADER: crowned. When it falls, the formation scatters into new places and fights
//!   in a rage for a few seconds.

use super::*;

#[derive(Default)]
pub struct Vanguard {
    /// Seconds of rage left after a Leader dies.
    pub rage_t: f32,
    heal_t: f32,
    /// Healer pulses being drawn (position, age).
    pulses: Vec<(Vec2, f32)>,
}

const HEAL_EVERY: f32 = 4.0;
const HEAL_RADIUS: f32 = 95.0;

impl Kind {
    /// Armor points an enemy of this kind starts with at difficulty `diff`.
    pub(super) fn base_armor(self, diff: f32) -> i32 {
        match self {
            Kind::Armored => 5 + (diff * 0.6) as i32,
            Kind::Leader => 2 + (diff * 0.3) as i32,
            _ => 0,
        }
    }
}

/// True when an armored enemy's plates stop a shot coming from `from`.
pub(super) fn armor_blocks(e: &Enemy, from: Vec2) -> bool {
    matches!(e.kind, Kind::Armored | Kind::Leader) && e.shield > 0 && from.y > e.pos.y - 2.0
}

/// The mirror opens and closes on its own rhythm.
pub(super) fn mirror_open(e: &Enemy, time: f32) -> bool {
    e.kind == Kind::Mirror && ((time * 0.45 + e.seed * 0.37).fract()) < 0.45
}

impl Game {
    /// Which elite (if any) stands in this slot of a formation at the current tier.
    pub(super) fn vanguard_kind(&self, row: usize, col: usize) -> Option<Kind> {
        let tier = self.stage().tier;
        if self.classic() {
            return None;
        }
        match (row, col) {
            (0, 4) if tier >= 9 => Some(Kind::Leader),
            (1, 3) | (1, 4) if tier >= 7 && (col == 3 || tier >= 11) => Some(Kind::Healer),
            (2, 0) | (2, 7) if tier >= 8 => Some(Kind::Mirror),
            (3, 1) | (3, 6) if tier >= 6 => Some(Kind::Armored),
            (4, 2) | (4, 5) if tier >= 10 => Some(Kind::Armored),
            _ => None,
        }
    }

    pub(super) fn update_vanguard(&mut self, dt: f32) {
        self.vg.rage_t = (self.vg.rage_t - dt).max(0.0);
        for p in &mut self.vg.pulses {
            p.1 += dt;
        }
        self.vg.pulses.retain(|p| p.1 < 0.7);
        self.vg.heal_t += dt;
        if self.vg.heal_t < HEAL_EVERY {
            return;
        }
        self.vg.heal_t = 0.0;
        let diff = self.diff();
        let armor_bonus = self.armor();
        let healers: Vec<Vec2> = self
            .enemies
            .iter()
            .filter(|e| e.kind == Kind::Healer && e.active() && !e.harmless && e.frozen <= 0.0)
            .map(|e| e.pos)
            .collect();
        for h in healers {
            self.vg.pulses.push((h, 0.0));
            for e in &mut self.enemies {
                if !e.active() || e.harmless || e.kind == Kind::Healer {
                    continue;
                }
                if e.pos.distance(h) < HEAL_RADIUS {
                    let max_hp = e.kind.hp() + armor_bonus;
                    if e.hp < max_hp {
                        e.hp += 1;
                    }
                    let max_armor = e.kind.base_armor(diff);
                    if e.shield < max_armor {
                        e.shield = (e.shield + 2).min(max_armor);
                    }
                }
            }
        }
    }

    /// A shot touched a Mirror with its mirror open: it comes back as an enemy shot.
    pub(super) fn reflect_shot(ebullets: &mut Vec<EBullet>, pos: Vec2, vel: Vec2, mul: f32) {
        let v = vec2(vel.x * 0.3, vel.y.abs().clamp(180.0, 320.0)) * mul;
        ebullets.push(EBullet::new(pos + vec2(0.0, 12.0), v, 5.0, 4));
    }

    pub(super) fn vanguard_on_kill(&mut self, e: &Enemy) {
        if e.kind != Kind::Leader || e.harmless {
            return;
        }
        // The formation panics: everyone swaps places and fights harder for a while.
        let idx: Vec<usize> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, o)| {
                !o.dead && o.hp > 0 && !o.harmless && matches!(o.state, EState::Formation)
            })
            .map(|(i, _)| i)
            .collect();
        let mut slots: Vec<(usize, usize)> = idx
            .iter()
            .map(|&i| (self.enemies[i].col, self.enemies[i].row))
            .collect();
        for i in (1..slots.len()).rev() {
            let j = gen_range(0, i + 1);
            slots.swap(i, j);
        }
        for (k, &i) in idx.iter().enumerate() {
            let o = &mut self.enemies[i];
            (o.col, o.row) = slots[k];
            let p = o.pos;
            o.state = EState::Entering {
                t: 0.0,
                path: Path::one(p, p + vec2(0.0, -50.0), p + vec2(0.0, -50.0), Vec2::ZERO),
                dur: 0.9,
            };
        }
        self.vg.rage_t = 7.0;
        self.dive_timer = self.dive_timer.min(0.6);
        self.fx
            .ring(e.pos, 260.0, 0.5, Color::from_rgba(255, 210, 80, 255), 4.0);
        self.fx.text(e.pos, "THE SWARM REGROUPS!", GOLD, 18.0);
        for _ in 0..6 {
            self.spawn_drop(e.pos, DropKind::Crystal(3));
        }
    }

    /// Rage multiplies the formation's fire and dive rates.
    pub(super) fn rage_mul(&self) -> f32 {
        if self.vg.rage_t > 0.0 { 1.7 } else { 1.0 }
    }

    pub(super) fn draw_vanguard(&self, e: &Enemy, off: Vec2) {
        let p = e.pos + off;
        match e.kind {
            Kind::Armored | Kind::Leader if e.shield > 0 => {
                let a = (e.shield as f32 / 6.0).clamp(0.35, 1.0);
                let c = Color::new(0.75, 0.8, 0.9, a);
                for k in 0..5 {
                    let ang = (200.0 + k as f32 * 35.0).to_radians() + e.rot;
                    let q = p + vec2(ang.cos(), -ang.sin()) * 17.0;
                    draw_poly(q.x, q.y, 4, 4.5, 45.0 + k as f32 * 10.0, c);
                    draw_poly_lines(
                        q.x,
                        q.y,
                        4,
                        4.5,
                        45.0 + k as f32 * 10.0,
                        1.0,
                        Color::new(0.1, 0.1, 0.2, a),
                    );
                }
            }
            Kind::Mirror if mirror_open(e, self.time) => {
                let s = 0.6 + (self.time * 12.0).sin() * 0.2;
                draw_poly_lines(
                    p.x,
                    p.y,
                    6,
                    20.0,
                    self.time * 40.0,
                    2.0,
                    Color::new(0.7, 1.0, 1.0, s),
                );
                draw_poly(
                    p.x,
                    p.y,
                    6,
                    20.0,
                    self.time * 40.0,
                    Color::new(0.6, 0.95, 1.0, 0.15),
                );
            }
            _ => {}
        }
        if e.kind == Kind::Leader {
            let glow = 0.25 + (self.time * 5.0).sin().abs() * 0.2;
            draw_circle(p.x, p.y - 4.0, 22.0, Color::new(1.0, 0.85, 0.3, glow * 0.5));
        }
    }

    pub(super) fn draw_vanguard_fx(&self, off: Vec2) {
        for (pos, age) in &self.vg.pulses {
            let k = age / 0.7;
            let p = *pos + off;
            draw_circle_lines(
                p.x,
                p.y,
                HEAL_RADIUS * k,
                3.0,
                Color::new(0.5, 1.0, 0.5, (1.0 - k) * 0.8),
            );
            if k < 0.3 {
                draw_text_plus(p, 1.0 - k / 0.3);
            }
        }
    }
}

/// A small green cross above a pulsing Healer.
fn draw_text_plus(p: Vec2, a: f32) {
    let c = Color::new(0.6, 1.0, 0.6, a);
    draw_rectangle(p.x - 2.0, p.y - 30.0, 4.0, 12.0, c);
    draw_rectangle(p.x - 6.0, p.y - 26.0, 12.0, 4.0, c);
}
