//! Boss fights change the stage as the boss weakens:
//!
//! - DARKNESS: the lights go out. Only your ship, your shots and the boss's glow light the
//!   way; enemy shots stay visible so the fight remains fair.
//! - RISING GROUND: rock peaks rise from the ground and split your lane into valleys.
//! - METEOR SHOWER: burning rocks rain from above. Shoot them or dodge them.
//!
//! The first change comes at two thirds of the boss's health; from level 40 on (and in the
//! boss rush) a second, different one comes at one third.

use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scene {
    Darkness,
    Ground,
    Meteors,
}

#[derive(Default)]
pub struct Scenery {
    pub scene: Option<Scene>,
    t: f32,
    marks: u8,
    meteor_cd: f32,
    peaks: [f32; 2],
    rise: f32,
    /// True once the current boss has been fought in the dark.
    pub dark_fight: bool,
}

const CELL: f32 = 12.0;
const PEAK_HALF: f32 = 28.0;

impl Game {
    pub(super) fn update_scenery(&mut self, dt: f32, au: &mut Audio) {
        let Some(b) = &self.boss else {
            if self.sc.scene.is_some() || self.sc.marks > 0 {
                self.sc = Scenery::default();
            }
            return;
        };
        if self.enc.void || b.max_hp <= 0.0 {
            return;
        }
        let ratio = b.hp / b.max_hp;
        let second = self.diff() >= 3.0 || self.cfg.mode == Mode::BossRush || b.final_boss;
        let want = if ratio < 0.33 && second {
            2
        } else if ratio < 0.66 {
            1
        } else {
            0
        };
        if want > self.sc.marks && b.hp > 0.0 {
            self.sc.marks = want;
            self.change_scene(au);
        }
        let Some(scene) = self.sc.scene else { return };
        self.sc.t += dt;
        match scene {
            Scene::Darkness => self.sc.dark_fight = true,
            Scene::Ground => self.sc.rise = (self.sc.rise + dt / 2.5).min(1.0),
            Scene::Meteors => {
                self.sc.meteor_cd -= dt;
                if self.sc.meteor_cd <= 0.0 {
                    self.sc.meteor_cd = gen_range(0.55, 1.1) / (1.0 + self.diff() * 0.06);
                    let big = gen_range(0, 3) == 0;
                    self.rocks.push(Rock {
                        pos: vec2(gen_range(20.0, W - 20.0), -30.0),
                        vel: vec2(gen_range(-50.0, 50.0), gen_range(150.0, 230.0)),
                        hp: if big { 5 } else { 2 },
                        big,
                        solid: false,
                        rot: 0.0,
                        vr: gen_range(-3.0, 3.0),
                        variant: gen_range(0, 2),
                        flash: 0.0,
                    });
                }
            }
        }
    }

    fn change_scene(&mut self, au: &mut Audio) {
        let options: Vec<Scene> = [Scene::Darkness, Scene::Ground, Scene::Meteors]
            .into_iter()
            .filter(|s| Some(*s) != self.sc.scene)
            .collect();
        let s = options[gen_range(0, options.len())];
        self.sc.scene = Some(s);
        self.sc.t = 0.0;
        self.sc.rise = 0.0;
        self.sc.meteor_cd = 1.2;
        let a = gen_range(W * 0.22, W * 0.38);
        let b = gen_range(W * 0.62, W * 0.78);
        self.sc.peaks = [a, b];
        let (title, sub) = match s {
            Scene::Darkness => ("THE LIGHTS GO OUT", "Your shots light the way."),
            Scene::Ground => ("THE GROUND RISES", "Rock peaks split your lane."),
            Scene::Meteors => ("METEOR SHOWER", "Shoot them or dodge them!"),
        };
        self.banner = Some((title.into(), sub.into(), 2.5));
        self.fx.shake = self.fx.shake.max(0.5);
        self.events.push(Event::Rumble(None, 0.6, 400));
        au.play(Sfx::Warning);
    }

    /// Development helper: starts a boss fight already in the given scene.
    pub(super) fn debug_scene(&mut self, k: usize) {
        let s = [Scene::Darkness, Scene::Ground, Scene::Meteors][k.min(2)];
        self.sc.scene = Some(s);
        self.sc.marks = 2;
        self.sc.t = 0.0;
        self.sc.meteor_cd = 1.0;
        self.sc.peaks = [W * 0.3, W * 0.7];
    }

    /// Keeps the ship out of the rock peaks.
    pub(super) fn scene_clamp(&mut self) {
        if self.sc.scene != Some(Scene::Ground) || self.sc.rise < 0.5 {
            return;
        }
        let hr = self.hit_r() + 6.0;
        let x = &mut self.player.pos.x;
        for px in self.sc.peaks {
            if (*x - px).abs() < PEAK_HALF + hr {
                *x = if *x < px {
                    px - PEAK_HALF - hr
                } else {
                    px + PEAK_HALF + hr
                };
            }
        }
    }

    /// Drawn after the enemies and before the enemy shots.
    pub(super) fn draw_scenery(&self, off: Vec2) {
        match self.sc.scene {
            Some(Scene::Ground) => self.draw_ground(off),
            Some(Scene::Darkness) => self.draw_darkness(off),
            _ => {}
        }
    }

    fn draw_ground(&self, off: Vec2) {
        let r = self.sc.rise;
        let base = H - 26.0 * r + off.y;
        let rock = Color::from_rgba(70, 48, 40, 255);
        let edge = Color::from_rgba(150, 95, 60, 255);
        let lava = Color::new(1.0, 0.45, 0.15, 0.55 + (self.time * 4.0).sin() * 0.2);
        draw_rectangle(off.x, base, W, H - base + 40.0, rock);
        draw_line(off.x, base, W + off.x, base, 3.0, edge);
        for px in self.sc.peaks {
            let top = H - (H - PLAYER_Y + 70.0) * r + off.y;
            let x = px + off.x;
            let l = vec2(x - PEAK_HALF - 10.0, H + off.y);
            let rr = vec2(x + PEAK_HALF + 10.0, H + off.y);
            let t = vec2(x, top);
            draw_triangle(l, rr, t, rock);
            draw_triangle(
                vec2(x - PEAK_HALF, H + off.y),
                vec2(x + PEAK_HALF, H + off.y),
                vec2(x - 4.0, top + 14.0),
                Color::from_rgba(92, 62, 50, 255),
            );
            draw_line(l.x, l.y, t.x, t.y, 2.5, edge);
            draw_line(rr.x, rr.y, t.x, t.y, 2.5, edge);
            draw_line(x - 6.0, top + 30.0, x + 3.0, H + off.y, 2.0, lava);
        }
    }

    fn draw_darkness(&self, off: Vec2) {
        let fade = (self.sc.t / 1.2).min(1.0) * 0.92;
        let mut lights: Vec<(Vec2, f32)> = Vec::with_capacity(64);
        let mut ship = |p: &Player| {
            if p.alive() {
                lights.push((p.pos, 135.0));
            }
        };
        ship(&self.player);
        if let Some(p2) = &self.p2 {
            ship(p2);
        }
        if let Some(b) = &self.boss {
            lights.push((b.pos, 80.0));
        }
        for b in self.bullets.iter().take(48) {
            lights.push((b.pos, 46.0));
        }
        for r in &self.arcade.rails {
            for k in 0..6 {
                lights.push((vec2(r.x, r.y * k as f32 / 6.0), 60.0));
            }
        }
        let cols = (W / CELL).ceil() as usize;
        let rows = (H / CELL).ceil() as usize;
        for j in 0..rows {
            let cy = (j as f32 + 0.5) * CELL;
            for i in 0..cols {
                let cx = (i as f32 + 0.5) * CELL;
                let c = vec2(cx, cy);
                let mut lit = 0.0f32;
                for (p, r) in &lights {
                    let d = c.distance(*p);
                    if d < *r {
                        lit = lit.max(1.0 - (d / r).powi(2));
                    }
                }
                let a = fade * (1.0 - lit);
                if a > 0.02 {
                    draw_rectangle(
                        i as f32 * CELL + off.x,
                        j as f32 * CELL + off.y,
                        CELL,
                        CELL,
                        Color::new(0.01, 0.0, 0.04, a),
                    );
                }
            }
        }
    }
}
