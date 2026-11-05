//! Heavy weapons sold in the shop and found in capsules: Fireball, Plasma, Railgun and Meteor.
//!
//! - FIREBALL: a big burning orb that explodes on impact and scatters embers.
//! - PLASMA: slow orbs that grow as they travel and burn through up to six enemies.
//! - RAILGUN: an instant beam that hits the whole column in front of the ship.
//! - METEOR: a piercing rock that shatters into shards at the end of its flight.

use super::*;
use std::f32::consts::TAU;

/// A railgun beam left on screen for a moment after the shot (visual only).
#[derive(Clone, Copy)]
pub struct RailFx {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub life: f32,
}

pub(super) const RAIL_LIFE: f32 = 0.22;

/// Builds the shots of one trigger pull. Returns the weapon cooldown.
pub(super) fn fire(
    w: Weapon,
    lv: i32,
    dmg: f32,
    o: Vec2,
    ss: f32,
    evo: bool,
    out: &mut Vec<PBullet>,
    au: &mut Audio,
) -> f32 {
    let mk = |pos: Vec2, ang_deg: f32, speed: f32, dmg: f32, kind: BKind| {
        let a = ang_deg.to_radians();
        PBullet::new(pos, vec2(a.sin(), -a.cos()) * speed * ss, dmg, kind, w)
    };
    let lvf = lv as f32;
    match w {
        Weapon::Fireball => {
            let angles: &[f32] = if lv >= 5 { &[-7.0, 7.0] } else { &[0.0] };
            for a in angles {
                let mut b = mk(o, *a, 420.0, (1.5 + lvf * 0.3) * dmg, BKind::Fireball);
                b.life = 0.9;
                // blast radius and blast damage (an evolved fireball is a mini sun)
                b.phase = (38.0 + lvf * 6.0) * if evo { 1.6 } else { 1.0 };
                b.base_x = (2.2 + lvf * 0.55) * dmg * if evo { 1.5 } else { 1.0 };
                b.evo = evo;
                out.push(b);
            }
            au.play_vol(Sfx::Fireball, 0.6);
            0.46
        }
        Weapon::Plasma => {
            let angles: &[f32] = match lv {
                1 | 2 => &[0.0],
                3 | 4 => &[-5.0, 5.0],
                _ if evo => &[-16.0, -9.0, 0.0, 9.0, 16.0],
                _ => &[-9.0, 0.0, 9.0],
            };
            for (k, a) in angles.iter().enumerate() {
                let mut b = mk(o, *a, 430.0, (1.5 + lvf * 0.25) * dmg, BKind::Plasma);
                b.evo = evo;
                b.pierce = 255;
                b.life = 2.0;
                b.phase = k as f32 * 1.7;
                out.push(b);
            }
            au.play_vol(Sfx::Plasma, 0.55);
            0.27
        }
        Weapon::Railgun => {
            let main = (4.5 + lvf * 1.4) * dmg;
            // evolved: twin beams side by side
            let xs: &[f32] = if evo { &[-9.0, 9.0] } else { &[0.0] };
            for x in xs {
                let mut b = mk(o + vec2(*x, 0.0), 0.0, 0.0, main, BKind::Rail);
                b.phase = 5.0 + lvf * 2.0;
                b.life = 1.0;
                out.push(b);
            }
            if lv >= 4 {
                for x in [-16.0, 16.0] {
                    let mut s = mk(o + vec2(x, 6.0), 0.0, 0.0, main * 0.35, BKind::Rail);
                    s.phase = 3.0;
                    s.life = 1.0;
                    out.push(s);
                }
            }
            au.play_vol(Sfx::Railgun, 0.6);
            0.58
        }
        Weapon::Meteor => {
            let n = if lv >= 4 { 2 } else { 1 };
            for k in 0..n {
                let a = (k as f32 - (n - 1) as f32 / 2.0) * 12.0;
                let mut b = mk(o, a, 520.0, (2.4 + lvf * 0.5) * dmg, BKind::Meteor);
                b.pierce = if evo { 5 } else { 2 };
                b.life = 0.62;
                b.evo = evo;
                // shard count and shard damage
                b.phase = ((4 + lv) * if evo { 2 } else { 1 }) as f32;
                b.base_x = 0.9 * dmg;
                out.push(b);
            }
            au.play_vol(Sfx::Meteor, 0.6);
            0.4
        }
        _ => 0.2,
    }
}

pub(super) fn move_bullet(b: &mut PBullet, dt: f32, fx: &mut Fx) {
    match b.kind {
        BKind::Fireball => {
            b.pos += b.vel * dt;
            if gen_range(0, 2) == 0 {
                fx.particles.push(crate::fx::Particle {
                    pos: b.pos + vec2(gen_range(-5.0, 5.0), gen_range(0.0, 6.0)),
                    vel: vec2(gen_range(-30.0, 30.0), gen_range(40.0, 110.0)),
                    life: 0.35,
                    max: 0.35,
                    color: Color::new(1.0, gen_range(0.35, 0.75), 0.1, 0.9),
                    size: gen_range(2.0, 4.0),
                    drag: 2.0,
                });
            }
        }
        BKind::Plasma => {
            b.vel *= 1.0 + dt * 0.35;
            b.pos += b.vel * dt;
            b.pos.x += (b.t * 9.0 + b.phase).sin() * 26.0 * dt;
        }
        BKind::Meteor => {
            b.pos += b.vel * dt;
        }
        _ => {}
    }
}

/// Plasma orbs grow while they fly.
fn plasma_r(b: &PBullet) -> f32 {
    6.0 + (b.t * 9.0).min(8.0) + if b.evo { 4.0 } else { 0.0 }
}

pub(super) fn draw_bullet(b: &PBullet, p: Vec2, time: f32) {
    match b.kind {
        BKind::Fireball => {
            let pulse = (1.0 + (time * 30.0).sin() * 0.08) * if b.evo { 1.6 } else { 1.0 };
            if b.evo {
                // the mini sun's corona
                for k in 0..8 {
                    let ang = time * 3.0 + k as f32 / 8.0 * std::f32::consts::TAU;
                    let q = p + vec2(ang.cos(), ang.sin()) * 18.0 * pulse;
                    draw_line(p.x, p.y, q.x, q.y, 2.0, Color::new(1.0, 0.8, 0.3, 0.6));
                }
            }
            draw_circle(
                p.x,
                p.y + 6.0,
                13.0 * pulse,
                Color::new(1.0, 0.25, 0.05, 0.25),
            );
            draw_circle(p.x, p.y, 10.0 * pulse, Color::new(1.0, 0.45, 0.1, 0.85));
            draw_circle(p.x, p.y - 1.0, 6.5, Color::new(1.0, 0.8, 0.3, 1.0));
            draw_circle(p.x, p.y - 2.0, 3.0, Color::new(1.0, 1.0, 0.85, 1.0));
        }
        BKind::Plasma => {
            let r = plasma_r(b);
            draw_circle(p.x, p.y, r * 1.6, Color::new(0.8, 0.3, 1.0, 0.16));
            draw_circle(p.x, p.y, r, Color::new(0.6, 0.4, 1.0, 0.55));
            draw_circle(p.x, p.y, r * 0.55, Color::new(0.85, 0.95, 1.0, 1.0));
            // crackling arcs
            for k in 0..3 {
                let a = time * 11.0 + k as f32 * 2.1 + b.phase;
                let q = p + vec2(a.cos(), a.sin()) * r * 1.3;
                let m = p + vec2((a + 0.6).cos(), (a + 0.6).sin()) * r * 0.7;
                draw_line(p.x, p.y, m.x, m.y, 1.2, Color::new(0.7, 0.9, 1.0, 0.8));
                draw_line(m.x, m.y, q.x, q.y, 1.2, Color::new(0.9, 0.6, 1.0, 0.8));
            }
        }
        BKind::Meteor => {
            let rot = b.t * 9.0;
            draw_circle(p.x, p.y + 8.0, 7.0, Color::new(1.0, 0.5, 0.2, 0.3));
            draw_poly(
                p.x,
                p.y,
                6,
                8.0,
                rot.to_degrees(),
                Color::from_rgba(120, 90, 80, 255),
            );
            draw_poly_lines(
                p.x,
                p.y,
                6,
                8.0,
                rot.to_degrees(),
                1.5,
                Color::from_rgba(255, 150, 70, 255),
            );
            draw_circle(p.x - 2.0, p.y - 2.0, 2.0, Color::from_rgba(70, 50, 45, 255));
        }
        _ => {}
    }
}

pub(super) fn draw_rail(r: &RailFx, off: Vec2) {
    let a = (r.life / RAIL_LIFE).clamp(0.0, 1.0);
    let x = r.x + off.x;
    let y = r.y + off.y;
    let w = r.width * (0.4 + a * 0.8);
    draw_rectangle(
        x - w * 1.8,
        0.0,
        w * 3.6,
        y,
        Color::new(0.3, 0.7, 1.0, 0.18 * a),
    );
    draw_rectangle(x - w / 2.0, 0.0, w, y, Color::new(0.5, 0.9, 1.0, 0.8 * a));
    draw_rectangle(x - w / 5.0, 0.0, w / 2.5, y, Color::new(1.0, 1.0, 1.0, a));
    draw_circle(x, y, w * 1.4, Color::new(0.7, 0.95, 1.0, a));
}

impl Game {
    /// Railgun beams hit everything in their column at once, then vanish.
    pub(super) fn rail_hits(&mut self, au: &mut Audio) {
        let mut rails: Vec<(Vec2, f32, f32, Option<Weapon>)> = vec![];
        for b in &mut self.bullets {
            if b.kind == BKind::Rail && b.life > 0.0 {
                rails.push((b.pos, b.phase, b.dmg, b.weapon));
                b.life = 0.0;
            }
        }
        for (pos, width, dmg, weapon) in rails {
            self.arcade.rails.push(RailFx {
                x: pos.x,
                y: pos.y,
                width,
                life: RAIL_LIFE,
            });
            self.fx.shake = self.fx.shake.max(0.06);
            let hit = dmg.floor() as i32 + (gen_range(0.0, 1.0) < dmg.fract()) as i32;
            for e in &mut self.enemies {
                if !e.active() || e.harmless || e.hp <= 0 || e.pos.y > pos.y || e.pos.y < -10.0 {
                    continue;
                }
                if (e.pos.x - pos.x).abs() < e.kind.radius() + width * 0.5 {
                    // Shields do not stop a railgun.
                    e.shield = 0;
                    e.hp -= if e.frozen > 0.0 { hit * 2 } else { hit };
                    e.flash = 0.1;
                    e.damaged = true;
                    e.last_weapon = weapon;
                    self.fx.sparks(
                        e.pos,
                        vec2(0.0, 1.0),
                        Color::from_rgba(170, 230, 255, 255),
                        6,
                    );
                }
            }
            if let Some(boss) = &mut self.boss
                && boss.targetable()
            {
                let mut y = pos.y;
                while y > 0.0 {
                    if let Some(t) = boss.hit_test(vec2(pos.x, y)) {
                        let res = boss.damage(t, dmg * 0.8);
                        self.fx.sparks(
                            vec2(pos.x, y),
                            vec2(0.0, 1.0),
                            Color::from_rgba(170, 230, 255, 255),
                            8,
                        );
                        if res.part_destroyed {
                            self.enc.parts += 1;
                            au.play(Sfx::Boom);
                        }
                        break;
                    }
                    y -= 12.0;
                }
            }
            for rock in &mut self.rocks {
                if !rock.solid && rock.pos.y < pos.y && (rock.pos.x - pos.x).abs() < 20.0 + width {
                    rock.hp -= hit;
                }
            }
        }
    }

    /// Fireballs explode and meteors shatter when they hit something or run out of time.
    pub(super) fn arsenal_expire(&mut self) {
        let mut shards: Vec<PBullet> = vec![];
        for b in &mut self.bullets {
            if b.life > 0.0 {
                continue;
            }
            match b.kind {
                BKind::Fireball => {
                    self.blasts.push((b.pos, b.phase, b.base_x, b.weapon));
                    b.kind = BKind::Shot;
                }
                BKind::Meteor => {
                    let n = b.phase as usize;
                    for k in 0..n {
                        let a = -150.0 + 300.0 * k as f32 / (n.max(2) - 1) as f32;
                        let a = (a + gen_range(-8.0, 8.0)).to_radians();
                        let mut s = PBullet::new(
                            b.pos,
                            vec2(a.sin(), -a.cos().abs() * 0.6 - 0.4) * gen_range(380.0, 520.0),
                            b.base_x,
                            BKind::Pellet,
                            Weapon::Meteor,
                        );
                        s.life = 0.45;
                        shards.push(s);
                    }
                    self.fx
                        .burst(b.pos, Color::from_rgba(255, 150, 70, 255), 14, 160.0);
                    b.kind = BKind::Shot;
                }
                _ => {}
            }
        }
        self.bullets.extend(shards);
    }

    /// Embers left by a fireball blast.
    pub(super) fn fireball_embers(&mut self, pos: Vec2, dmg: f32, n: usize) {
        for k in 0..n {
            let a = k as f32 / n as f32 * TAU + gen_range(-0.2, 0.2);
            let mut e = PBullet::new(
                pos,
                vec2(a.cos(), a.sin()) * gen_range(160.0, 260.0),
                dmg * 0.25,
                BKind::Flame,
                Weapon::Fireball,
            );
            e.life = 0.35;
            self.bullets.push(e);
        }
    }

    pub(super) fn update_rails(&mut self, dt: f32) {
        for r in &mut self.arcade.rails {
            r.life -= dt;
        }
        self.arcade.rails.retain(|r| r.life > 0.0);
    }
}
