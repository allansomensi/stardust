//! Sistemas da versão 4 (Stardust): maldições que caem dos inimigos, Relíquias secretas,
//! armas novas, Fendas de minigame, a Estrela Cadente que abre Salas Secretas,
//! as habilidades das naves novas e os rastros personalizados.

use super::minigame::{Grade, MG_KINDS, MgKind, Minigame};
use super::*;
use crate::icons;
use crate::ships::RELICS;
use std::f32::consts::{PI, TAU};

// ---------------------------------------------------------------------------
// Maldições
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Curse {
    Caveira,
    Ferrugem,
    Ancora,
    Espelho,
    Pane,
}

impl Curse {
    pub fn name(self) -> &'static str {
        match self {
            Curse::Caveira => "CAVEIRA",
            Curse::Ferrugem => "FERRUGEM",
            Curse::Ancora => "ÂNCORA",
            Curse::Espelho => "ESPELHO",
            Curse::Pane => "PANE",
        }
    }
    fn effect(self) -> &'static str {
        match self {
            Curse::Caveira => "Você perdeu a arma!",
            Curse::Ferrugem => "A arma perdeu 2 níveis.",
            Curse::Ancora => "Nave pesada por 8 segundos.",
            Curse::Espelho => "Controles invertidos por 6 segundos.",
            Curse::Pane => "Armas travadas por 3 segundos.",
        }
    }
    fn random() -> Curse {
        match gen_range(0, 100) {
            0..=34 => Curse::Caveira,
            35..=54 => Curse::Ferrugem,
            55..=71 => Curse::Ancora,
            72..=86 => Curse::Espelho,
            _ => Curse::Pane,
        }
    }
}

// ---------------------------------------------------------------------------
// Sala secreta
// ---------------------------------------------------------------------------

pub struct Geode {
    pos: Vec2,
    hp: i32,
    flash: f32,
}

pub struct Room {
    t: f32,
    chest: Vec2,
    chest_hp: f32,
    chest_max: f32,
    geodes: Vec<Geode>,
    ring_cd: f32,
    ring_rot: f32,
    opened: Option<f32>,
    stash_enemies: Vec<Enemy>,
    stash_ebullets: Vec<EBullet>,
    stash_rocks: Vec<Rock>,
    theme: usize,
}

struct Star {
    pos: Vec2,
    vel: Vec2,
    hp: i32,
    flash: f32,
}

// ---------------------------------------------------------------------------
// Estado
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct V4State {
    pub anchor_t: f32,
    pub mirror_t: f32,
    pub jam_t: f32,
    pub curses_taken: u32,
    pub curses_shot: u32,
    curses_shot_sector: u32,
    pub relics_found: Vec<usize>,
    relics_given: Vec<Weapon>,
    pub minigame: Option<Minigame>,
    last_mg: Option<MgKind>,
    pub minigames_won: u32,
    pub minigames_played: u32,
    pub perfects: u32,
    rifts_sector: u32,
    pub room: Option<Room>,
    room_enter: f32,
    star: Option<Star>,
    star_planned: Option<u32>,
    pub rooms_found: u32,
    nova_t: f32,
    pub aerogel_t: f32,
    pub loot_t: f32,
    pub reflect_t: f32,
    pub captured: u32,
    trail: Vec<(Vec2, f32)>,
    trail_cd: f32,
    pub sector_nohit: bool,
    pub blessing: Option<Blessing>,
    pub blessings_taken: u32,
    shield_each: bool,
}

impl V4State {
    pub fn new() -> V4State {
        V4State::default()
    }
}

// ---------------------------------------------------------------------------
// Armas novas e Relíquias
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
pub(super) fn fire_v4(
    w: Weapon,
    lv: i32,
    dmg: f32,
    o: Vec2,
    ss: f32,
    timer: &mut f32,
    rate: f32,
    mods: &Mods,
    out: &mut Vec<PBullet>,
    au: &mut Audio,
) -> f32 {
    let mk = |pos: Vec2, ang_deg: f32, speed: f32, dmg: f32, kind: BKind| {
        let a = ang_deg.to_radians();
        PBullet::new(pos, vec2(a.sin(), -a.cos()) * speed * ss, dmg, kind, w)
    };
    let lvi = (lv - 1).clamp(0, 4) as usize;
    match w {
        Weapon::Agulha => {
            let xs: &[f32] = match lvi {
                0 => &[0.0],
                1 => &[-4.0, 4.0],
                2 => &[-7.0, 0.0, 7.0],
                3 => &[-9.0, -3.0, 3.0, 9.0],
                _ => &[-12.0, -6.0, 0.0, 6.0, 12.0],
            };
            for x in xs {
                let mut b = mk(
                    o + vec2(*x, x.abs() * 0.3),
                    x * 0.15,
                    1150.0,
                    0.55 * dmg,
                    BKind::Needle,
                );
                b.pierce = if lv >= 5 { 3 } else { 2 };
                out.push(b);
            }
            au.play_vol(Sfx::Laser, 0.18);
            0.075
        }
        Weapon::Bumerangue => {
            let angs: &[f32] = if lv >= 4 { &[-12.0, 12.0] } else { &[0.0] };
            for a in angs {
                let mut b = mk(
                    o,
                    *a,
                    600.0,
                    (1.3 + lv as f32 * 0.25) * dmg,
                    BKind::Boomerang,
                );
                b.pierce = 255;
                b.life = 3.2;
                out.push(b);
            }
            if lv >= 3 {
                out.push(mk(o, 0.0, 640.0, 0.8 * dmg, BKind::Shot));
            }
            au.play_vol(Sfx::Disc, 0.35);
            0.5 - lv as f32 * 0.03
        }
        Weapon::Prisma => {
            let mut b = mk(o, 0.0, 720.0, 1.4 * dmg, BKind::Prism);
            b.phase = if lv >= 4 { 5.0 } else { 3.0 };
            b.base_x = 0.8 + lv as f32 * 0.12;
            out.push(b);
            if lv >= 2 {
                for x in [-10.0, 10.0] {
                    out.push(mk(o + vec2(x, 4.0), 0.0, 700.0, 0.7 * dmg, BKind::Laser));
                }
            }
            au.play_vol(Sfx::Laser, 0.32);
            0.22
        }
        Weapon::Mina => {
            out.push(mk(o, 0.0, 640.0, 0.9 * dmg, BKind::Shot));
            if *timer <= 0.0 {
                *timer = (0.75 - lv as f32 * 0.07) / rate;
                let n = if lv >= 4 { 2 } else { 1 };
                for k in 0..n {
                    let side = if n == 1 {
                        0.0
                    } else if k == 0 {
                        -1.0
                    } else {
                        1.0
                    };
                    let mut b = PBullet::new(
                        o + vec2(side * 14.0, 16.0),
                        vec2(side * 60.0, -150.0),
                        (2.5 + lv as f32 * 0.6) * dmg,
                        BKind::Mine,
                        w,
                    );
                    b.life = 4.5;
                    b.phase = (48.0 + lv as f32 * 7.0) * mods.blast_mul;
                    out.push(b);
                }
                au.play_vol(Sfx::Missile, 0.25);
            }
            0.18
        }
        Weapon::Supernova => {
            for a in [-7.0, 0.0, 7.0] {
                let mut b = mk(o, a, 760.0, 2.2 * dmg, BKind::Nova);
                b.pierce = 2;
                out.push(b);
            }
            if *timer <= 0.0 {
                *timer = 0.85 / rate;
                for k in 0..18 {
                    let a = k as f32 / 18.0 * 360.0;
                    let mut b = mk(o + vec2(0.0, 16.0), a, 520.0, 1.6 * dmg, BKind::Star);
                    b.phase = k as f32;
                    b.life = 1.2;
                    out.push(b);
                }
                au.play_vol(Sfx::Sparkle, 0.35);
            } else {
                au.play_vol(Sfx::Blaster, 0.3);
            }
            0.12
        }
        Weapon::Poeira => {
            for k in 0..4 {
                let side = if k % 2 == 0 { -1.0 } else { 1.0 };
                let mut b = PBullet::new(
                    o + vec2(side * gen_range(2.0, 12.0), 2.0),
                    vec2(side * gen_range(60.0, 260.0), -gen_range(260.0, 420.0)),
                    1.05 * dmg,
                    BKind::Dust,
                    w,
                );
                b.life = 2.0;
                b.pierce = 1;
                b.phase = gen_range(0.0, TAU);
                out.push(b);
            }
            au.play_vol(Sfx::Crystal, 0.12);
            0.085
        }
        Weapon::Lanca => {
            let mut b = mk(o + vec2(0.0, -6.0), 0.0, 1500.0, 1.35 * dmg, BKind::Lance);
            b.pierce = 255;
            out.push(b);
            if *timer <= 0.0 {
                *timer = 0.2;
                au.play_vol(Sfx::Flame, 0.3);
            }
            0.035
        }
        Weapon::Singularidade => {
            for x in [-7.0, 7.0] {
                let mut b = mk(o + vec2(x, 0.0), 0.0, 980.0, 1.3 * dmg, BKind::Laser);
                b.pierce = 3;
                out.push(b);
            }
            if *timer <= 0.0 {
                *timer = 0.7 / rate;
                let mut b = mk(o, 0.0, 330.0, 0.0, BKind::Hole);
                b.life = 2.4 * mods.hole_mul;
                b.pierce = 255;
                b.phase = 96.0;
                out.push(b);
                au.play_vol(Sfx::Portal, 0.3);
            }
            0.1
        }
        _ => 0.14,
    }
}

pub(super) fn move_bullet(
    b: &mut PBullet,
    dt: f32,
    targets: &[Vec2],
    boss_pos: Option<Vec2>,
    pp: Vec2,
) {
    match b.kind {
        BKind::Boomerang => {
            if b.t < 0.42 {
                b.vel *= 1.0 - (dt * 2.4).min(1.0);
                b.pos += b.vel * dt;
            } else {
                if b.base_x == b.pos.x || b.t - dt < 0.42 {
                    b.hit = [0; 6];
                }
                let to = pp - b.pos;
                let speed = (b.vel.length() + 1400.0 * dt).min(760.0);
                b.vel = to.normalize_or_zero() * speed;
                b.pos += b.vel * dt;
                if to.length() < 20.0 {
                    b.life = 0.0;
                }
            }
        }
        BKind::Mine => {
            b.vel.x *= 1.0 - (dt * 3.0).min(1.0);
            b.vel.y += (-55.0 - b.vel.y) * (dt * 2.5).min(1.0);
            b.pos += b.vel * dt;
            if targets.iter().any(|t| t.distance(b.pos) < 34.0)
                || boss_pos.map(|p| p.distance(b.pos) < 60.0).unwrap_or(false)
            {
                b.life = 0.0;
            }
        }
        BKind::Dust => {
            let mut best: Option<Vec2> = boss_pos;
            let mut bd = boss_pos.map(|p| p.distance(b.pos)).unwrap_or(f32::MAX);
            for t in targets {
                let d = t.distance(b.pos);
                if d < bd {
                    bd = d;
                    best = Some(*t);
                }
            }
            let desired = best
                .map(|t| (t - b.pos).normalize_or_zero())
                .unwrap_or(vec2(0.0, -1.0));
            let cur = b.vel.normalize_or_zero();
            let wob = vec2(-cur.y, cur.x) * (b.t * 18.0 + b.phase).sin() * 0.4;
            b.vel = (cur + (desired * 9.0 + wob * 3.0) * dt).normalize_or_zero() * 640.0;
            b.pos += b.vel * dt;
        }
        _ => b.pos += b.vel * dt,
    }
}

fn star_shape(c: Vec2, r_out: f32, r_in: f32, points: usize, rot: f32, col: Color) {
    let n = points * 2;
    for i in 0..n {
        let a0 = rot + i as f32 / n as f32 * TAU - PI / 2.0;
        let a1 = rot + (i + 1) as f32 / n as f32 * TAU - PI / 2.0;
        let r0 = if i % 2 == 0 { r_out } else { r_in };
        let r1 = if i % 2 == 0 { r_in } else { r_out };
        draw_triangle(
            c,
            c + vec2(a0.cos(), a0.sin()) * r0,
            c + vec2(a1.cos(), a1.sin()) * r1,
            col,
        );
    }
}

pub(super) fn draw_bullet(b: &PBullet, p: Vec2, time: f32) {
    match b.kind {
        BKind::Needle => {
            let d = b.vel.normalize_or_zero();
            draw_line(
                p.x,
                p.y,
                p.x - d.x * 16.0,
                p.y - d.y * 16.0,
                3.0,
                Color::new(0.8, 1.0, 0.5, 0.35),
            );
            draw_line(
                p.x,
                p.y,
                p.x - d.x * 13.0,
                p.y - d.y * 13.0,
                1.4,
                Color::from_rgba(230, 255, 170, 255),
            );
            draw_circle(p.x, p.y, 1.3, WHITE);
        }
        BKind::Boomerang => {
            let a = b.t * 22.0;
            let c = Color::from_rgba(255, 170, 90, 255);
            let d1 = vec2(a.cos(), a.sin()) * 9.0;
            let d2 = vec2((a + 2.0).cos(), (a + 2.0).sin()) * 9.0;
            draw_circle(p.x, p.y, 11.0, Color::new(1.0, 0.6, 0.3, 0.15));
            draw_line(p.x, p.y, p.x + d1.x, p.y + d1.y, 4.0, c);
            draw_line(p.x, p.y, p.x + d2.x, p.y + d2.y, 4.0, c);
            draw_circle(p.x, p.y, 2.5, WHITE);
        }
        BKind::Prism => {
            draw_circle(p.x, p.y, 9.0, Color::new(0.5, 1.0, 1.0, 0.2));
            draw_poly(
                p.x,
                p.y,
                3,
                7.0,
                time * 300.0,
                Color::from_rgba(170, 255, 255, 255),
            );
            draw_poly_lines(p.x, p.y, 3, 7.0, time * 300.0, 1.5, WHITE);
        }
        BKind::Mine => {
            let blink = (time * 8.0 + b.t * 3.0).sin() > 0.0;
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU + b.t;
                let q = p + vec2(a.cos(), a.sin()) * 9.0;
                draw_line(p.x, p.y, q.x, q.y, 1.5, Color::from_rgba(200, 80, 80, 255));
            }
            draw_circle(p.x, p.y, 6.0, Color::from_rgba(120, 30, 40, 255));
            draw_circle(
                p.x,
                p.y,
                2.5,
                if blink {
                    RED
                } else {
                    Color::from_rgba(255, 200, 200, 255)
                },
            );
            if b.life < 1.0 {
                draw_circle_lines(p.x, p.y, b.phase * 0.5, 1.0, Color::new(1.0, 0.3, 0.3, 0.4));
            }
        }
        BKind::Nova => {
            draw_circle(p.x, p.y, 11.0, Color::new(1.0, 0.8, 0.3, 0.2));
            star_shape(
                p,
                9.0,
                3.5,
                6,
                b.t * 8.0,
                Color::from_rgba(255, 170, 60, 255),
            );
            star_shape(
                p,
                6.0,
                2.5,
                4,
                -b.t * 6.0,
                Color::from_rgba(255, 245, 190, 255),
            );
            draw_circle(p.x, p.y, 2.0, WHITE);
        }
        BKind::Dust => {
            let tw = (time * 30.0 + b.phase).sin() * 0.5 + 0.5;
            let c = crate::ui::hsl(0.8 + (b.phase * 0.1).fract() * 0.25, 0.9, 0.75);
            draw_circle(p.x, p.y, 5.0, Color::new(c.r, c.g, c.b, 0.25));
            star_shape(p, 4.0 + tw * 2.0, 1.4, 4, 0.0, c);
            draw_circle(p.x, p.y, 1.2, WHITE);
        }
        BKind::Lance => {
            let flick = (time * 60.0 + b.pos.y * 0.1).sin() * 1.5;
            let h = 46.0;
            draw_rectangle(
                p.x - 13.0 - flick,
                p.y - h,
                26.0 + flick * 2.0,
                h * 2.0,
                Color::new(1.0, 0.7, 0.2, 0.1),
            );
            draw_rectangle(
                p.x - 7.0,
                p.y - h,
                14.0,
                h * 2.0,
                Color::new(1.0, 0.8, 0.3, 0.45),
            );
            draw_rectangle(
                p.x - 3.0,
                p.y - h,
                6.0,
                h * 2.0,
                Color::from_rgba(255, 250, 220, 255),
            );
        }
        _ => {}
    }
}

/// Moldura de cápsula: ícone da arma, nome e o anel que mostra quando ela vai trocar.
pub(super) fn draw_capsule(
    p: Vec2,
    w: Weapon,
    next: Option<Weapon>,
    progress: f32,
    time: f32,
    blink: bool,
) {
    let c = w.color();
    let pulse = (time * 5.0).sin() * 1.5;
    draw_circle(p.x, p.y, 23.0 + pulse, Color::new(c.r, c.g, c.b, 0.16));
    icons::weapon_badge(w, p, 15.0, if blink { 0.55 } else { 1.0 });
    if next.is_some() {
        let n = 24;
        let lit = ((1.0 - progress) * n as f32) as usize;
        for k in 0..n {
            if k >= lit {
                continue;
            }
            let a = -PI / 2.0 + k as f32 / n as f32 * TAU;
            let q = p + vec2(a.cos(), a.sin()) * 19.5;
            draw_circle(q.x, q.y, 1.3, Color::new(1.0, 1.0, 1.0, 0.75));
        }
    }
    let name = w.name();
    let tw = crate::ui::width(name, 11.0);
    draw_rectangle(
        p.x - tw / 2.0 - 4.0,
        p.y + 22.0,
        tw + 8.0,
        14.0,
        Color::new(0.0, 0.0, 0.05, 0.7),
    );
    crate::ui::text_c(name, p.x, p.y + 33.0, 11.0, c);
}

fn draw_skull(p: Vec2, s: f32, col: Color) {
    draw_circle(p.x, p.y - 1.0 * s, 8.0 * s, col);
    draw_rectangle(p.x - 5.0 * s, p.y + 3.0 * s, 10.0 * s, 6.0 * s, col);
    let dark = Color::from_rgba(25, 5, 15, 255);
    draw_circle(p.x - 3.2 * s, p.y - 1.5 * s, 2.4 * s, dark);
    draw_circle(p.x + 3.2 * s, p.y - 1.5 * s, 2.4 * s, dark);
    draw_triangle(
        p + vec2(0.0, 1.5) * s,
        p + vec2(-1.3, 4.0) * s,
        p + vec2(1.3, 4.0) * s,
        dark,
    );
    for k in 0..3 {
        let x = p.x - 3.0 * s + k as f32 * 3.0 * s;
        draw_line(x, p.y + 6.0 * s, x, p.y + 9.0 * s, 1.0, dark);
    }
}

fn draw_curse(p: Vec2, c: Curse, hp: i32, time: f32) {
    let pulse = (time * 7.0).sin() * 0.5 + 0.5;
    draw_circle(
        p.x,
        p.y,
        21.0 + pulse * 3.0,
        Color::new(0.8, 0.0, 0.2, 0.18 + pulse * 0.1),
    );
    draw_circle(p.x, p.y, 15.0, Color::from_rgba(40, 0, 20, 240));
    draw_circle_lines(p.x, p.y, 15.0, 2.0, Color::from_rgba(255, 60, 90, 255));
    for k in 0..hp.max(0) {
        let a = -PI / 2.0 + k as f32 * 0.5 - 0.5;
        let q = p + vec2(a.cos(), a.sin()) * 19.0;
        draw_circle(q.x, q.y, 2.0, Color::from_rgba(255, 90, 110, 255));
    }
    let skull_col = Color::from_rgba(240, 230, 220, 255);
    match c {
        Curse::Caveira => draw_skull(p, 1.0, skull_col),
        Curse::Ferrugem => {
            draw_skull(
                p + vec2(-2.0, -1.0),
                0.8,
                Color::from_rgba(215, 135, 70, 255),
            );
            crate::ui::text_c("-2", p.x + 7.0, p.y + 11.0, 11.0, WHITE);
        }
        Curse::Ancora => {
            let col = Color::from_rgba(180, 190, 210, 255);
            draw_line(p.x, p.y - 9.0, p.x, p.y + 8.0, 2.5, col);
            draw_line(p.x - 5.0, p.y - 5.0, p.x + 5.0, p.y - 5.0, 2.5, col);
            draw_circle_lines(p.x, p.y - 10.0, 2.5, 1.5, col);
            draw_line(p.x - 8.0, p.y + 3.0, p.x, p.y + 9.0, 2.5, col);
            draw_line(p.x + 8.0, p.y + 3.0, p.x, p.y + 9.0, 2.5, col);
        }
        Curse::Espelho => {
            let col = Color::from_rgba(200, 160, 255, 255);
            draw_triangle(
                p + vec2(-10.0, 0.0),
                p + vec2(-3.0, -6.0),
                p + vec2(-3.0, 6.0),
                col,
            );
            draw_triangle(
                p + vec2(10.0, 0.0),
                p + vec2(3.0, -6.0),
                p + vec2(3.0, 6.0),
                col,
            );
            draw_line(p.x, p.y - 9.0, p.x, p.y + 9.0, 1.5, WHITE);
        }
        Curse::Pane => {
            let col = Color::from_rgba(255, 220, 80, 255);
            draw_line(p.x - 6.0, p.y - 8.0, p.x + 6.0, p.y + 8.0, 3.0, col);
            draw_line(p.x + 6.0, p.y - 8.0, p.x - 6.0, p.y + 8.0, 3.0, col);
        }
    }
    let name = c.name();
    let tw = crate::ui::width(name, 11.0);
    draw_rectangle(
        p.x - tw / 2.0 - 4.0,
        p.y + 22.0,
        tw + 8.0,
        14.0,
        Color::new(0.15, 0.0, 0.05, 0.75),
    );
    crate::ui::text_c(
        name,
        p.x,
        p.y + 33.0,
        11.0,
        Color::from_rgba(255, 90, 110, 255),
    );
}

fn draw_relic_drop(p: Vec2, w: Weapon, time: f32) {
    for k in 0..12 {
        let a = time * 0.8 + k as f32 / 12.0 * TAU;
        let len = 30.0 + (time * 4.0 + k as f32).sin() * 6.0;
        let q = p + vec2(a.cos(), a.sin()) * len;
        draw_line(p.x, p.y, q.x, q.y, 2.0, Color::new(1.0, 0.85, 0.3, 0.35));
    }
    draw_circle(p.x, p.y, 24.0, Color::new(1.0, 0.85, 0.3, 0.2));
    icons::weapon_badge(w, p, 17.0, 1.0);
    let label = "RELÍQUIA";
    let tw = crate::ui::width(label, 11.0);
    draw_rectangle(
        p.x - tw / 2.0 - 4.0,
        p.y + 24.0,
        tw + 8.0,
        14.0,
        Color::new(0.1, 0.06, 0.0, 0.8),
    );
    crate::ui::text_c(label, p.x, p.y + 35.0, 11.0, GOLD);
}

fn draw_rift(p: Vec2, time: f32) {
    for k in 0..4 {
        let r = 8.0 + k as f32 * 4.5 + (time * 3.0 + k as f32).sin() * 1.5;
        let c = crate::ui::hsl(0.55 + k as f32 * 0.08 + time * 0.1, 0.9, 0.65);
        draw_poly_lines(
            p.x,
            p.y,
            6,
            r,
            time * 90.0 * if k % 2 == 0 { 1.0 } else { -1.0 },
            2.0,
            Color::new(c.r, c.g, c.b, 0.8 - k as f32 * 0.15),
        );
    }
    draw_circle(p.x, p.y, 7.0, Color::from_rgba(20, 10, 40, 255));
    crate::ui::text_c("?", p.x, p.y + 6.0, 16.0, WHITE);
    let label = "FENDA";
    let tw = crate::ui::width(label, 11.0);
    draw_rectangle(
        p.x - tw / 2.0 - 4.0,
        p.y + 22.0,
        tw + 8.0,
        14.0,
        Color::new(0.05, 0.0, 0.12, 0.8),
    );
    crate::ui::text_c(label, p.x, p.y + 33.0, 11.0, CYAN);
}

/// Movimento dos drops novos (chamado de update_drops).
pub(super) fn drop_motion(kind: &DropKind, vel: &mut Vec2, age: f32, pos: Vec2, pp: Vec2, dt: f32) {
    match kind {
        DropKind::Curse(..) => {
            vel.y = (vel.y + 160.0 * dt).min(105.0);
            let toward = (pp.x - pos.x).signum() * 28.0;
            vel.x += (toward - vel.x) * (dt * 1.5).min(1.0);
        }
        DropKind::Relic(_) => {
            vel.y = (vel.y + 80.0 * dt).min(50.0);
            vel.x = (age * 1.6).sin() * 30.0;
        }
        DropKind::Rift => {
            vel.y = (vel.y + 100.0 * dt).min(70.0);
            vel.x = (age * 2.2).cos() * 35.0;
        }
        DropKind::Letter(_) => {
            vel.y = (vel.y + 100.0 * dt).min(80.0);
            vel.x = (age * 2.6).sin() * 28.0;
        }
        _ => {}
    }
}

pub fn trail_color(style: usize, time: f32, age: f32) -> Color {
    let k = age / 0.5;
    match style {
        1 => crate::ui::hsl(time * 0.6 + age * 2.0, 0.9, 0.6),
        2 => Color::new(1.0, 0.5 - k * 0.4, 0.1, 1.0),
        3 => Color::new(1.0, 0.85, 0.5, 1.0),
        4 => Color::new(0.6, 0.3, 1.0, 1.0),
        5 => Color::new(0.6, 0.9, 1.0, 1.0),
        6 => Color::new(1.0, 0.85, 0.2, 1.0),
        _ => Color::new(0.4, 1.0, 0.6, 1.0),
    }
}

pub(super) fn draw_drop(kind: &DropKind, p: Vec2, time: f32) {
    match kind {
        DropKind::Curse(c, hp) => draw_curse(p, *c, *hp, time),
        DropKind::Relic(w) => draw_relic_drop(p, *w, time),
        DropKind::Rift => draw_rift(p, time),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Jogo
// ---------------------------------------------------------------------------

impl Game {
    pub(super) fn v4_begin_sector(&mut self) {
        self.v4_sector_start();
        self.y.rifts_sector = 0;
        self.y.curses_shot_sector = 0;
        self.y.sector_nohit = true;
        self.y.star_planned = None;
        let campaign = matches!(
            self.cfg.mode,
            Mode::Campanha | Mode::Diario | Mode::Expedicao
        );
        if campaign && !self.secret_stage && !self.x.void && gen_range(0.0, 1.0) < 0.4 {
            // a estrela cadente aparece em uma das ondas normais do setor
            self.y.star_planned = Some(gen_range(1, 4));
        }
    }

    pub(super) fn v4_wave_start(&mut self) {
        if let Some(n) = self.y.star_planned
            && self.sub() == n
            && self.wave_kind == WaveKind::Normal
        {
            self.y.star_planned = None;
            let left = gen_range(0, 2) == 0;
            let x = if left { -20.0 } else { W + 20.0 };
            let dir = if left { 1.0 } else { -1.0 };
            self.y.star = Some(Star {
                pos: vec2(x, gen_range(90.0, 220.0)),
                vel: vec2(dir * gen_range(120.0, 160.0), gen_range(10.0, 40.0)),
                hp: 3,
                flash: 0.0,
            });
        }
    }

    /// Chamado em cada inimigo abatido.
    pub(super) fn v4_on_kill(&mut self, e: &Enemy) {
        if self.classic() || self.secret_stage || self.y.room.is_some() || e.harmless {
            return;
        }
        let tier = self.stage().tier;
        let busy = self
            .drops
            .iter()
            .filter(|d| matches!(d.kind, DropKind::Curse(..)))
            .count();
        let curse_chance = if tier < 2 {
            0.0
        } else if e.elite {
            0.06
        } else {
            0.014 + tier as f32 * 0.002
        } * self.mods.curse_mul;
        if busy == 0 && gen_range(0.0, 1.0) < curse_chance {
            self.spawn_drop(e.pos, DropKind::Curse(Curse::random(), 3));
            self.fx.ring(e.pos, 120.0, 0.3, RED, 2.0);
        }
        let rift_chance = if e.kind == Kind::Cargueiro {
            0.25
        } else if e.elite {
            0.1
        } else {
            0.012
        } * self.mods.rift_mul;
        let rift_on_screen = self.drops.iter().any(|d| matches!(d.kind, DropKind::Rift));
        if !rift_on_screen && self.y.rifts_sector < 2 && gen_range(0.0, 1.0) < rift_chance {
            self.y.rifts_sector += 1;
            self.spawn_drop(e.pos, DropKind::Rift);
        }
    }

    pub(super) fn v4_on_hit(&mut self) {
        self.y.sector_nohit = false;
    }

    /// Perder a arma (caveira) ou morrer com uma Relíquia na mão.
    fn lose_weapon(&mut self) {
        let p = &mut self.player;
        p.weapon = Weapon::Rajada;
        p.level = (p.level - 1).max(1);
    }

    pub(super) fn v4_on_death(&mut self) {
        if self.player.weapon.is_relic() {
            let w = self.player.weapon;
            self.lose_weapon();
            self.fx.text(
                self.player.pos - vec2(0.0, 40.0),
                format!("{} PERDIDA", w.name()),
                GOLD,
                18.0,
            );
        }
    }

    pub(super) fn take_curse(&mut self, c: Curse, au: &mut Audio) {
        let pp = self.player.pos;
        if self.mods.amulet && gen_range(0, 2) == 0 {
            au.play(Sfx::Shield);
            self.fx.ring(pp, 260.0, 0.4, GOLD, 4.0);
            self.fx
                .text(pp - vec2(0.0, 40.0), "O AMULETO PROTEGEU!", GOLD, 18.0);
            return;
        }
        self.y.curses_taken += 1;
        self.events.push(Event::CurseTaken);
        au.play(Sfx::Curse);
        self.fx.ring(pp, 300.0, 0.5, RED, 5.0);
        self.fx
            .burst(pp, Color::from_rgba(120, 20, 50, 255), 30, 200.0);
        self.fx.shake = self.fx.shake.max(0.4);
        match c {
            Curse::Caveira => {
                let lost = self.player.weapon;
                self.lose_weapon();
                self.banner = Some((
                    "CAVEIRA!".into(),
                    format!("{} foi perdida.", lost.name()),
                    2.2,
                ));
            }
            Curse::Ferrugem => {
                if !self.player.weapon.is_relic() {
                    self.player.level = (self.player.level - 2).max(1);
                }
                self.banner = Some(("FERRUGEM!".into(), c.effect().into(), 2.0));
            }
            Curse::Ancora => {
                self.y.anchor_t = 8.0;
                self.banner = Some(("ÂNCORA!".into(), c.effect().into(), 2.0));
            }
            Curse::Espelho => {
                self.y.mirror_t = 6.0;
                self.banner = Some(("ESPELHO!".into(), c.effect().into(), 2.0));
            }
            Curse::Pane => {
                self.y.jam_t = 3.0;
                self.banner = Some(("PANE!".into(), c.effect().into(), 2.0));
            }
        }
    }

    fn grant_relic(&mut self, w: Weapon, pos: Vec2) {
        if self.y.relics_given.contains(&w) {
            return;
        }
        self.y.relics_given.push(w);
        self.spawn_drop(pos, DropKind::Relic(w));
        self.banner = Some((
            "UMA RELÍQUIA SURGIU!".into(),
            "Pegue antes que ela se perca.".into(),
            2.5,
        ));
    }

    pub(super) fn take_relic(&mut self, w: Weapon, au: &mut Audio) {
        let pp = self.player.pos;
        v3::pick_new_weapon(&mut self.player, w);
        self.player.level = 5;
        au.play(Sfx::Relic);
        self.fx.flash = 0.6;
        self.fx.flash_color = Color::new(1.0, 0.9, 0.5, 1.0);
        self.fx.ring(pp, 400.0, 0.6, GOLD, 6.0);
        self.fx.burst(pp, GOLD, 60, 320.0);
        let i = RELICS.iter().position(|r| *r == w).unwrap_or(0);
        if !self.y.relics_found.contains(&i) {
            self.y.relics_found.push(i);
        }
        self.events.push(Event::Relic(i));
        self.banner = Some((w.name().into(), w.desc().into(), 3.5));
    }

    pub(super) fn start_minigame(&mut self, au: &mut Audio) {
        let mut kind = MG_KINDS[gen_range(0, MG_KINDS.len())];
        if Some(kind) == self.y.last_mg {
            kind = MG_KINDS[(kind as usize + 1) % MG_KINDS.len()];
        }
        self.y.last_mg = Some(kind);
        self.y.minigames_played += 1;
        au.play(Sfx::Portal);
        self.y.minigame = Some(Minigame::new(kind));
    }

    /// Roda o minigame em vez do mundo. Devolve true enquanto ele estiver ativo.
    pub(super) fn v4_minigame(&mut self, dt: f32, inp: &Input, au: &mut Audio) -> bool {
        let Some(mg) = &mut self.y.minigame else {
            return false;
        };
        mg.update(dt, inp, au);
        if !mg.done {
            return true;
        }
        let grade = mg.grade;
        self.y.minigame = None;
        let pp = self.player.pos;
        self.events.push(Event::Minigame(grade as u8));
        match grade {
            Grade::Derrota => {
                for _ in 0..1 {
                    self.spawn_drop(pp - vec2(0.0, 60.0), DropKind::Crystal(5));
                }
            }
            Grade::Vitoria | Grade::Perfeito => {
                let perfect = grade == Grade::Perfeito;
                self.y.minigames_won += 1;
                if perfect {
                    self.y.perfects += 1;
                }
                let n = if perfect { 12 } else { 6 };
                for _ in 0..n {
                    self.spawn_drop(
                        pp - vec2(gen_range(-60.0, 60.0), 90.0),
                        DropKind::Crystal(5),
                    );
                }
                let p = &mut self.player;
                if !p.weapon.is_relic() && p.level < 5 {
                    p.level += 1;
                    self.fx.text(
                        pp - vec2(0.0, 40.0),
                        format!("{} NÍVEL {}", p.weapon.name(), p.level),
                        p.weapon.color(),
                        20.0,
                    );
                }
                self.spawn_drop(pp - vec2(0.0, 120.0), DropKind::Power(PowerKind::random()));
                if perfect {
                    self.spawn_drop(pp - vec2(30.0, 140.0), DropKind::Card(cards::roll(1)));
                }
                au.play(Sfx::Fanfare);
                if self.y.minigames_won == 3 {
                    self.grant_relic(Weapon::Supernova, vec2(W / 2.0, 80.0));
                }
            }
        }
        self.player.invuln = self.player.invuln.max(1.5);
        false
    }

    // -----------------------------------------------------------------------
    // Sala Secreta
    // -----------------------------------------------------------------------

    fn enter_room(&mut self, au: &mut Audio) {
        let theme = self.bg.theme;
        let stash_enemies = std::mem::take(&mut self.enemies);
        let stash_ebullets = std::mem::take(&mut self.ebullets);
        let stash_rocks = std::mem::take(&mut self.rocks);
        let tier = self.stage().tier as f32;
        let hp = 70.0 + tier * 14.0;
        let mut geodes = vec![];
        for k in 0..8 {
            let a = k as f32 / 8.0 * TAU;
            geodes.push(Geode {
                pos: vec2(W / 2.0 + a.cos() * 150.0, 250.0 + a.sin() * 110.0),
                hp: 6,
                flash: 0.0,
            });
        }
        self.y.room = Some(Room {
            t: 0.0,
            chest: vec2(W / 2.0, 250.0),
            chest_hp: hp,
            chest_max: hp,
            geodes,
            ring_cd: 2.5,
            ring_rot: 0.0,
            opened: None,
            stash_enemies,
            stash_ebullets,
            stash_rocks,
            theme,
        });
        self.y.rooms_found += 1;
        self.events.push(Event::Room);
        self.bg.set_theme(5);
        au.play(Sfx::Portal);
        au.music(Some(Music::Shop));
        self.fx.flash = 0.9;
        self.fx.flash_color = WHITE;
        self.banner = Some((
            "SALA SECRETA".into(),
            "Abra o relicário antes que a fenda se feche!".into(),
            3.0,
        ));
    }

    fn leave_room(&mut self, au: &mut Audio) {
        if let Some(r) = self.y.room.take() {
            self.enemies = r.stash_enemies;
            self.ebullets = r.stash_ebullets;
            self.rocks = r.stash_rocks;
            self.bg.set_theme(r.theme);
            self.fx.flash = 0.7;
            self.fx.flash_color = WHITE;
            self.player.invuln = self.player.invuln.max(2.0);
            au.play(Sfx::Portal);
            au.music(Some(Music::Stage));
        }
    }

    pub(super) fn room_active(&self) -> bool {
        self.y.room.is_some() || self.y.room_enter > 0.0
    }

    fn update_room(&mut self, dt: f32, au: &mut Audio) {
        let Some(r) = &mut self.y.room else { return };
        r.t += dt;
        for g in &mut r.geodes {
            g.flash = (g.flash - dt).max(0.0);
        }
        let mut leave = false;
        if let Some(t) = &mut r.opened {
            *t += dt;
            if *t > 3.5 {
                leave = true;
            }
        } else {
            r.ring_cd -= dt;
            r.ring_rot += dt * 0.7;
            if r.ring_cd <= 0.0 {
                r.ring_cd = (1.7 - r.t * 0.02).max(1.0);
                let n = 14;
                for k in 0..n {
                    let a = r.ring_rot + k as f32 / n as f32 * TAU;
                    self.ebullets.push(EBullet::new(
                        r.chest,
                        vec2(a.cos(), a.sin()) * 120.0 * self.cfg.diff.bullet_speed(),
                        5.0,
                        4,
                    ));
                }
                au.play_vol(Sfx::EnemyShot, 0.4);
            }
            if r.t > 28.0 {
                self.banner = Some((
                    "A FENDA SE FECHOU".into(),
                    "O relicário ficou para trás.".into(),
                    2.5,
                ));
                leave = true;
            }
        }
        if leave {
            self.leave_room(au);
        }
    }

    fn open_chest(&mut self, au: &mut Audio) {
        let Some(r) = &mut self.y.room else { return };
        r.opened = Some(0.0);
        let pos = r.chest;
        self.ebullets.clear();
        au.play(Sfx::BigBoom);
        self.fx.explosion(pos, GOLD, true);
        self.fx.flash = 0.6;
        self.fx.flash_color = GOLD;
        for _ in 0..16 {
            self.drops.push(Drop {
                pos,
                vel: vec2(gen_range(-200.0, 200.0), gen_range(-260.0, -60.0)),
                kind: DropKind::Crystal(5),
                age: 0.0,
            });
        }
        if !self.y.relics_given.contains(&Weapon::Poeira) {
            self.grant_relic(Weapon::Poeira, pos);
        } else {
            self.spawn_drop(pos, DropKind::Card(cards::roll(2)));
            self.spawn_drop(pos, DropKind::Power(PowerKind::Vida));
        }
    }

    // -----------------------------------------------------------------------
    // Atualização
    // -----------------------------------------------------------------------

    pub(super) fn v4_update(&mut self, dt: f32, au: &mut Audio) {
        for t in [
            &mut self.y.anchor_t,
            &mut self.y.mirror_t,
            &mut self.y.jam_t,
            &mut self.y.aerogel_t,
            &mut self.y.loot_t,
            &mut self.y.reflect_t,
        ] {
            *t = (*t - dt).max(0.0);
        }
        self.y.nova_t -= dt;

        // prismas se dividem
        let mut splits = vec![];
        for b in &mut self.bullets {
            if b.kind == BKind::Prism && b.t > 0.17 && b.life > 0.0 {
                b.life = 0.0;
                splits.push((
                    b.pos,
                    b.phase as usize,
                    b.base_x,
                    b.weapon.unwrap_or(Weapon::Prisma),
                ));
            }
        }
        for (pos, n, dmg, w) in splits {
            for k in 0..n {
                let a = (-14.0 + 28.0 * k as f32 / (n - 1).max(1) as f32).to_radians();
                let mut b =
                    PBullet::new(pos, vec2(a.sin(), -a.cos()) * 900.0, dmg, BKind::Laser, w);
                b.pierce = 1;
                self.bullets.push(b);
            }
            self.fx.sparks(
                pos,
                vec2(0.0, -1.0),
                Color::from_rgba(170, 255, 255, 255),
                4,
            );
        }

        // Singularidade: engolir tiros com buracos negros
        if self.absorbed >= 150 && !self.classic() {
            self.grant_relic(Weapon::Singularidade, vec2(W / 2.0, 70.0));
        }

        // estrela cadente
        let mut open = false;
        if let Some(s) = &mut self.y.star {
            s.pos += s.vel * dt;
            s.flash = (s.flash - dt).max(0.0);
            if gen_range(0, 2) == 0 {
                self.fx.particles.push(crate::fx::Particle {
                    pos: s.pos,
                    vel: -s.vel * 0.2 + vec2(gen_range(-20.0, 20.0), gen_range(-20.0, 20.0)),
                    life: 0.6,
                    max: 0.6,
                    color: Color::new(1.0, 0.9, 0.5, 0.9),
                    size: 2.0,
                    drag: 2.0,
                });
            }
            for b in &mut self.bullets {
                if b.life > 0.0 && b.kind != BKind::Hole && b.pos.distance(s.pos) < 16.0 {
                    b.life = 0.0;
                    s.hp -= 1;
                    s.flash = 0.1;
                    au.play_vol(Sfx::Crystal, 0.8);
                }
            }
            if s.hp <= 0 {
                open = true;
            }
            if s.pos.x < -40.0 || s.pos.x > W + 40.0 || s.pos.y > H * 0.7 {
                self.y.star = None;
            }
        }
        if open {
            let pos = self.y.star.take().map(|s| s.pos).unwrap_or_default();
            self.fx.burst(pos, GOLD, 50, 260.0);
            self.fx.ring(pos, 500.0, 0.6, WHITE, 5.0);
            au.play(Sfx::Relic);
            if self.boss.is_some() || self.wave_kind != WaveKind::Normal || self.y.room.is_some() {
                for _ in 0..6 {
                    self.spawn_drop(pos, DropKind::Crystal(5));
                }
            } else {
                self.y.room_enter = 1.2;
                self.banner = Some((
                    "A ESTRELA SE ABRIU".into(),
                    "Algo puxa a nave para dentro...".into(),
                    1.2,
                ));
            }
        }
        if self.y.room_enter > 0.0 {
            self.y.room_enter -= dt;
            self.bg.speed_mul = 1.0 + (1.2 - self.y.room_enter) * 4.0;
            if self.y.room_enter <= 0.0 {
                self.bg.speed_mul = 1.0;
                self.enter_room(au);
            }
        }
        self.update_room(dt, au);

        // habilidades novas
        if self.y.aerogel_t > 0.0 && self.player.alive() {
            let pp = self.player.pos;
            let mut caught = 0;
            self.ebullets.retain(|b| {
                let hit = b.pos.distance(pp) < 95.0;
                if hit {
                    caught += 1;
                }
                !hit
            });
            if caught > 0 {
                self.y.captured += caught;
                self.crystals += caught as f32 * 0.5;
                self.fx.sparks(pp, vec2(0.0, -1.0), CYAN, 2);
            }
        }
        if self.y.reflect_t > 0.0 && self.player.alive() {
            let pp = self.player.pos;
            let mut back = vec![];
            self.ebullets.retain(|b| {
                let hit = b.pos.distance(pp) < 70.0;
                if hit {
                    back.push(b.pos);
                }
                !hit
            });
            let dmg = self.ship().dmg * 1.5;
            for pos in back {
                let mut b =
                    PBullet::new(pos, vec2(0.0, -700.0), dmg, BKind::Shot, self.player.weapon);
                b.weapon = None;
                self.bullets.push(b);
            }
        }
        if self.y.loot_t > 0.0 && self.player.alive() {
            let pp = self.player.pos;
            for d in &mut self.drops {
                if !matches!(d.kind, DropKind::Curse(..)) {
                    let to = (pp - d.pos).normalize_or_zero() * 600.0;
                    d.vel += (to - d.vel) * (dt * 6.0).min(1.0);
                }
            }
        }

        // rastro
        self.y.trail_cd -= dt;
        if self.y.trail_cd <= 0.0 && self.player.alive() && self.cfg.trail > 0 {
            self.y.trail_cd = 0.016;
            self.y
                .trail
                .push((self.player.pos + vec2(gen_range(-3.0, 3.0), 20.0), 0.0));
        }
        for t in &mut self.y.trail {
            t.1 += dt;
            t.0.y += 160.0 * dt;
        }
        self.y.trail.retain(|t| t.1 < 0.5);
    }

    /// Tiros contra maldições, a estrela e os objetos da sala secreta.
    pub(super) fn v4_collisions(&mut self, au: &mut Audio) {
        let mut destroyed = vec![];
        for (i, d) in self.drops.iter_mut().enumerate() {
            let DropKind::Curse(_, hp) = &mut d.kind else {
                continue;
            };
            for b in &mut self.bullets {
                if b.life <= 0.0 || b.kind == BKind::Hole || *hp <= 0 {
                    continue;
                }
                if b.pos.distance(d.pos) < 17.0 {
                    if b.pierce != 255 {
                        b.life = 0.0;
                    }
                    *hp -= 1;
                    au.play_vol(Sfx::Clang, 0.4);
                    if *hp <= 0 {
                        destroyed.push(i);
                    }
                    if b.pierce == 255 {
                        break;
                    }
                }
            }
        }
        for &i in destroyed.iter().rev() {
            let d = self.drops.swap_remove(i);
            self.y.curses_shot += 1;
            self.y.curses_shot_sector += 1;
            self.events.push(Event::CurseShot);
            au.play(Sfx::Pop);
            self.fx
                .burst(d.pos, Color::from_rgba(255, 90, 120, 255), 24, 200.0);
            self.fx
                .text(d.pos - vec2(0.0, 20.0), "EXORCIZADA!", PINK, 16.0);
            for _ in 0..3 {
                self.spawn_drop(d.pos, DropKind::Crystal(1));
            }
            if self.y.curses_shot_sector >= 3 {
                self.grant_relic(Weapon::Lanca, d.pos);
            }
        }

        // sala secreta
        let mut open = false;
        if let Some(r) = &mut self.y.room
            && r.opened.is_none()
        {
            let mut crystals = vec![];
            for b in &mut self.bullets {
                if b.life <= 0.0 || b.kind == BKind::Hole {
                    continue;
                }
                for g in &mut r.geodes {
                    if g.hp > 0 && b.pos.distance(g.pos) < 16.0 {
                        g.hp -= 1;
                        g.flash = 0.08;
                        b.life = 0.0;
                        if g.hp <= 0 {
                            crystals.push(g.pos);
                        }
                        break;
                    }
                }
                if b.life > 0.0 && b.pos.distance(r.chest) < 30.0 {
                    r.chest_hp -= b.dmg;
                    b.life = 0.0;
                    if r.chest_hp <= 0.0 {
                        open = true;
                    }
                }
            }
            for p in crystals {
                au.play(Sfx::Crystal);
                self.fx.burst(p, CYAN, 20, 180.0);
                for _ in 0..4 {
                    self.drops.push(Drop {
                        pos: p,
                        vel: vec2(gen_range(-120.0, 120.0), gen_range(-200.0, -40.0)),
                        kind: DropKind::Crystal(5),
                        age: 0.0,
                    });
                }
            }
        }
        if open {
            self.open_chest(au);
        }
    }

    // -----------------------------------------------------------------------
    // Habilidades das naves novas
    // -----------------------------------------------------------------------

    pub(super) fn v4_ability(&mut self, ab: Ability, au: &mut Audio) {
        let pp = self.player.pos;
        match ab {
            Ability::Aerogel => {
                self.y.aerogel_t = 4.0;
                au.play(Sfx::Shield);
                self.fx.ring(pp, 200.0, 0.4, CYAN, 4.0);
            }
            Ability::Saque => {
                self.y.loot_t = 5.0;
                au.play(Sfx::Steal);
                self.fx.ring(pp, 400.0, 0.5, GOLD, 4.0);
            }
            Ability::Reflexo => {
                self.y.reflect_t = 3.0;
                au.play(Sfx::Ability);
                self.fx
                    .ring(pp, 260.0, 0.4, Color::from_rgba(255, 200, 255, 255), 4.0);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Desenho
    // -----------------------------------------------------------------------

    pub(super) fn draw_trail(&self, off: Vec2) {
        let style = self.cfg.trail;
        for (p, age) in &self.y.trail {
            let k = age / 0.5;
            let a = (1.0 - k) * 0.8;
            let c = trail_color(style, self.time, *age);
            let r = match style {
                3 => 1.5,
                _ => 4.0 * (1.0 - k) + 1.0,
            };
            let q = *p + off;
            if style == 3 {
                star_shape(
                    q,
                    3.0 * (1.0 - k) + 1.0,
                    1.0,
                    4,
                    0.0,
                    Color::new(c.r, c.g, c.b, a),
                );
            } else {
                draw_circle(q.x, q.y, r, Color::new(c.r, c.g, c.b, a));
            }
        }
    }

    pub(super) fn v4_draw(&self, off: Vec2) {
        if let Some(s) = &self.y.star {
            let p = s.pos + off;
            let tw = (self.time * 12.0).sin() * 0.5 + 0.5;
            draw_circle(p.x, p.y, 18.0 + tw * 4.0, Color::new(1.0, 0.9, 0.5, 0.15));
            star_shape(
                p,
                11.0 + tw * 3.0,
                3.0,
                4,
                self.time * 2.0,
                if s.flash > 0.0 {
                    WHITE
                } else {
                    Color::from_rgba(255, 230, 140, 255)
                },
            );
            draw_circle(p.x, p.y, 2.5, WHITE);
        }
        if let Some(r) = &self.y.room {
            for g in &r.geodes {
                if g.hp <= 0 {
                    continue;
                }
                let p = g.pos + off;
                let c = if g.flash > 0.0 { WHITE } else { CYAN };
                draw_circle(p.x, p.y, 16.0, Color::new(0.4, 0.9, 1.0, 0.15));
                draw_poly(
                    p.x,
                    p.y,
                    6,
                    12.0,
                    self.time * 30.0,
                    Color::from_rgba(40, 90, 140, 255),
                );
                draw_poly(p.x, p.y, 4, 8.0, 0.0, c);
                draw_poly(p.x - 2.0, p.y - 3.0, 4, 3.0, 0.0, WHITE);
            }
            let p = r.chest + off;
            if r.opened.is_none() {
                let k = r.chest_hp / r.chest_max;
                for i in 0..3 {
                    let rr = 38.0 + i as f32 * 10.0 + (self.time * 3.0 + i as f32).sin() * 3.0;
                    draw_poly_lines(
                        p.x,
                        p.y,
                        8,
                        rr,
                        self.time * 20.0 * (i as f32 + 1.0),
                        1.5,
                        Color::new(1.0, 0.85, 0.3, 0.4),
                    );
                }
                draw_rectangle(
                    p.x - 26.0,
                    p.y - 18.0,
                    52.0,
                    36.0,
                    Color::from_rgba(120, 70, 20, 255),
                );
                draw_rectangle(
                    p.x - 26.0,
                    p.y - 18.0,
                    52.0,
                    12.0,
                    Color::from_rgba(170, 110, 30, 255),
                );
                draw_rectangle_lines(p.x - 26.0, p.y - 18.0, 52.0, 36.0, 3.0, GOLD);
                draw_rectangle(p.x - 5.0, p.y - 6.0, 10.0, 12.0, GOLD);
                draw_rectangle(
                    p.x - 40.0,
                    p.y + 30.0,
                    80.0,
                    6.0,
                    Color::new(0.2, 0.1, 0.0, 0.8),
                );
                draw_rectangle(p.x - 40.0, p.y + 30.0, 80.0 * k.max(0.0), 6.0, GOLD);
                let left = (28.0 - r.t).max(0.0);
                crate::ui::text_c(
                    &format!("{:.0}", left.ceil()),
                    W / 2.0,
                    120.0,
                    30.0,
                    if left < 6.0 { RED } else { GOLD },
                );
            }
        }
    }

    pub(super) fn v4_draw_player_fx(&self, p: Vec2) {
        if self.y.aerogel_t > 0.0 {
            let a = (self.y.aerogel_t * 3.0).min(1.0);
            draw_circle(p.x, p.y, 95.0, Color::new(0.5, 0.9, 1.0, 0.08 * a));
            draw_circle_lines(p.x, p.y, 95.0, 2.0, Color::new(0.6, 0.95, 1.0, 0.5 * a));
        }
        if self.y.reflect_t > 0.0 {
            let a = (self.y.reflect_t * 3.0).min(1.0);
            draw_poly_lines(
                p.x,
                p.y,
                6,
                70.0,
                self.time * 120.0,
                2.5,
                Color::new(1.0, 0.8, 1.0, 0.7 * a),
            );
        }
        if self.y.anchor_t > 0.0 {
            draw_circle_lines(p.x, p.y + 4.0, 22.0, 2.0, Color::new(0.6, 0.6, 0.7, 0.6));
        }
        if self.y.jam_t > 0.0 && (self.time * 10.0) as i32 % 2 == 0 {
            crate::ui::text_c("PANE", p.x, p.y - 30.0, 13.0, GOLD);
        }
        if self.y.mirror_t > 0.0 {
            crate::ui::text_c(
                "<>",
                p.x,
                p.y - 30.0,
                13.0,
                Color::from_rgba(200, 160, 255, 255),
            );
        }
    }

    pub(super) fn v4_draw_top(&self, sp: &Sprites, art: &Art) {
        if let Some(m) = &self.y.minigame {
            m.draw(self.time, &art.ships[self.cfg.ship][self.cfg.skin]);
        }
        self.draw_blessing(sp);
        if self.y.room_enter > 0.0 {
            let k = 1.0 - self.y.room_enter / 1.2;
            draw_rectangle(0.0, 0.0, W, H, Color::new(1.0, 0.95, 0.8, k * 0.8));
        }
    }

    /// Linhas extras para os painéis laterais.
    pub fn curse_status(&self) -> Vec<(String, Color)> {
        let mut v = vec![];
        if self.y.anchor_t > 0.0 {
            v.push((format!("ÂNCORA {:.0}s", self.y.anchor_t.ceil()), RED));
        }
        if self.y.mirror_t > 0.0 {
            v.push((format!("ESPELHO {:.0}s", self.y.mirror_t.ceil()), RED));
        }
        if self.y.jam_t > 0.0 {
            v.push((format!("PANE {:.0}s", self.y.jam_t.ceil()), RED));
        }
        v
    }
}

// ---------------------------------------------------------------------------
// Ganchos de teste (variáveis de ambiente STARDUST_*)
// ---------------------------------------------------------------------------

impl Game {
    pub fn debug_showcase(&mut self) {
        let caps = [Weapon::Laser, Weapon::Agulha, Weapon::Mina, Weapon::Prisma];
        for (i, w) in caps.iter().enumerate() {
            self.drops.push(Drop {
                pos: vec2(70.0 + i as f32 * 110.0, 150.0),
                vel: Vec2::ZERO,
                kind: DropKind::Capsule {
                    weapons: [*w, Weapon::Rajada, Weapon::Onda, *w],
                    n: 3,
                    idx: 0,
                    t: i as f32 * 0.4,
                },
                age: 0.0,
            });
        }
        let curses = [Curse::Caveira, Curse::Ferrugem, Curse::Ancora];
        for (i, c) in curses.iter().enumerate() {
            self.drops.push(Drop {
                pos: vec2(110.0 + i as f32 * 130.0, 260.0),
                vel: Vec2::ZERO,
                kind: DropKind::Curse(*c, 3),
                age: 0.0,
            });
        }
        self.drops.push(Drop {
            pos: vec2(150.0, 360.0),
            vel: Vec2::ZERO,
            kind: DropKind::Rift,
            age: 0.0,
        });
        self.drops.push(Drop {
            pos: vec2(330.0, 360.0),
            vel: Vec2::ZERO,
            kind: DropKind::Relic(Weapon::Supernova),
            age: 0.0,
        });
    }

    pub fn debug_minigame(&mut self, k: usize) {
        self.y.minigame = Some(Minigame::new(MG_KINDS[k % MG_KINDS.len()]));
    }

    pub fn debug_bless(&mut self) {
        self.offer_blessing();
        if let Some(b) = &mut self.y.blessing {
            b.t = -30.0;
        }
    }

    pub fn debug_v5(&mut self) {
        for i in 0..3 {
            self.drops.push(Drop {
                pos: vec2(120.0 + i as f32 * 120.0, 200.0),
                vel: Vec2::ZERO,
                kind: DropKind::Letter(i * 2),
                age: 0.0,
            });
        }
        self.z.letters = [true, false, true, true, false, false, false, true];
        self.debug_ufo_soon();
    }

    pub fn debug_room(&mut self) {
        self.y.room_enter = 0.05;
    }

    pub fn debug_star(&mut self) {
        self.y.star = Some(Star {
            pos: vec2(-10.0, 160.0),
            vel: vec2(130.0, 20.0),
            hp: 3,
            flash: 0.0,
        });
    }
}

// ---------------------------------------------------------------------------
// Bênçãos da Expedição: ao fim de cada setor, escolha 1 de 3.
// ---------------------------------------------------------------------------

pub const BLESSINGS: [(&str, &str); 12] = [
    ("VIDA EXTRA", "+1 nave agora."),
    ("GATILHO LEVE", "Cadência +12% pelo resto da expedição."),
    ("PROPULSORES", "Velocidade +10% pelo resto da expedição."),
    ("ARSENAL", "Sua arma sobe 2 níveis agora."),
    ("PAIOL", "+2 bombas agora."),
    ("ESCUDO ETERNO", "Um escudo novo no começo de cada setor."),
    ("GANÂNCIA", "Cristais +30% pelo resto da expedição."),
    ("AMULETO", "Metade das maldições não te pega mais."),
    ("FENDAS", "Fendas de minigame caem 2,5x mais."),
    ("DRONE", "+1 drone de apoio (máximo 3)."),
    ("FOCO", "A barra de habilidade enche 35% mais rápido."),
    ("TREVO", "Cartas caem 50% mais."),
];

pub struct Blessing {
    pub options: Vec<usize>,
    pub sel: usize,
    t: f32,
}

impl Game {
    pub(super) fn offer_blessing(&mut self) {
        let mut pool: Vec<usize> = (0..BLESSINGS.len()).collect();
        if self.mods.amulet {
            pool.retain(|&i| i != 7);
        }
        if self.drones_total() >= 3 {
            pool.retain(|&i| i != 9);
        }
        if self.y.shield_each {
            pool.retain(|&i| i != 5);
        }
        let mut options = vec![];
        while options.len() < 3 && !pool.is_empty() {
            let i = gen_range(0, pool.len());
            options.push(pool.swap_remove(i));
        }
        self.y.blessing = Some(Blessing {
            options,
            sel: 1,
            t: 0.0,
        });
    }

    fn apply_blessing(&mut self, i: usize, au: &mut Audio) {
        let pp = self.player.pos;
        match i {
            0 => self.player.lives = (self.player.lives + 1).min(9),
            1 => self.mods.fire_mul *= 1.12,
            2 => self.mods.speed_mul *= 1.1,
            3 => {
                if !self.player.weapon.is_relic() {
                    self.player.level = (self.player.level + 2).min(5);
                }
            }
            4 => self.player.bombs = (self.player.bombs + 2).min(9),
            5 => {
                self.y.shield_each = true;
                self.player.shield = true;
            }
            6 => self.mods.crystal_mul *= 1.3,
            7 => self.mods.amulet = true,
            8 => self.mods.rift_mul *= 2.5,
            9 => self.drones_bought += 1,
            10 => self.mods.meter_mul *= 1.35,
            _ => self.mods.card_mul *= 1.5,
        }
        self.y.blessings_taken += 1;
        au.play(Sfx::Relic);
        self.fx.ring(pp, 300.0, 0.5, GOLD, 4.0);
    }

    /// Roda a escolha de bênção por cima da loja. Devolve true enquanto ela estiver aberta.
    pub(super) fn v4_blessing(&mut self, dt: f32, inp: &Input, au: &mut Audio) -> bool {
        let Some(b) = &mut self.y.blessing else {
            return false;
        };
        b.t += dt;
        let n = b.options.len();
        if inp.left_p && n > 0 {
            b.sel = (b.sel + n - 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.right_p && n > 0 {
            b.sel = (b.sel + 1) % n;
            au.play(Sfx::MenuMove);
        }
        if b.t > 0.6 && inp.confirm && n > 0 {
            let pick = b.options[b.sel];
            self.y.blessing = None;
            self.apply_blessing(pick, au);
            return false;
        }
        true
    }

    pub(super) fn v4_sector_start(&mut self) {
        if self.y.shield_each {
            self.player.shield = true;
        }
    }

    pub(super) fn draw_blessing(&self, sp: &Sprites) {
        let Some(b) = &self.y.blessing else { return };
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.02, 0.01, 0.06, 0.9));
        crate::ui::text_outline_fit(
            "ESCOLHA UMA BÊNÇÃO",
            W / 2.0,
            150.0,
            28.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
            W - 30.0,
        );
        crate::ui::text_c(
            &format!("Expedição · setor {} vencido", self.sector()),
            W / 2.0,
            178.0,
            14.0,
            DIM,
        );
        let n = b.options.len();
        let cw = 140.0;
        let gap = 12.0;
        let total = n as f32 * cw + (n as f32 - 1.0) * gap;
        let x0 = W / 2.0 - total / 2.0;
        for (k, &i) in b.options.iter().enumerate() {
            let x = x0 + k as f32 * (cw + gap);
            let y = 240.0;
            let sel = k == b.sel;
            let lift = if sel {
                -8.0 - (self.time * 4.0).sin() * 3.0
            } else {
                0.0
            };
            let border = if sel {
                GOLD
            } else {
                Color::new(0.5, 0.45, 0.3, 1.0)
            };
            draw_rectangle(x, y + lift, cw, 230.0, Color::new(0.1, 0.07, 0.16, 0.97));
            draw_rectangle_lines(x, y + lift, cw, 230.0, if sel { 3.0 } else { 1.5 }, border);
            let c = vec2(x + cw / 2.0, y + lift + 60.0);
            star_shape(
                c,
                34.0,
                14.0,
                8,
                self.time * if sel { 0.8 } else { 0.2 },
                Color::new(1.0, 0.85, 0.3, if sel { 0.35 } else { 0.15 }),
            );
            use crate::sprites::Icon;
            let icon = match i {
                0 => Icon::Heart,
                1 => Icon::Bolt,
                2 => Icon::Engine,
                3 => Icon::Star,
                4 => Icon::Bomb,
                5 => Icon::Shield,
                6 => Icon::Crystal,
                7 => Icon::Disc,
                8 => Icon::Target,
                9 => Icon::Drone,
                10 => Icon::Clock,
                _ => Icon::Card,
            };
            sp.icon(icon).draw(
                c,
                4.0,
                0.0,
                if sel {
                    WHITE
                } else {
                    Color::new(0.7, 0.7, 0.8, 1.0)
                },
                false,
            );
            let (name, desc) = BLESSINGS[i];
            crate::ui::text_outline_fit(
                name,
                c.x,
                y + lift + 124.0,
                15.0,
                if sel { WHITE } else { DIM },
                BLACK,
                cw - 10.0,
            );
            for (j, l) in crate::ui::wrap(desc, 12.0, cw - 16.0)
                .iter()
                .take(4)
                .enumerate()
            {
                crate::ui::text_c(
                    l,
                    c.x,
                    y + lift + 150.0 + j as f32 * 15.0,
                    12.0,
                    if sel {
                        Color::new(0.9, 0.9, 1.0, 1.0)
                    } else {
                        DIM
                    },
                );
            }
        }
        crate::ui::text_c(
            "ESQ/DIR escolhe · ENTER confirma",
            W / 2.0,
            520.0,
            14.0,
            DIM,
        );
    }
}
