//! Minigames: a Rift drops from an enemy, the pilot touches it and the game pauses for a
//! quick 10 to 15 second challenge. The result becomes a reward in the run.

use crate::audio::{Audio, Sfx};
use crate::ui::{self, CYAN, DIM, GOLD, GREEN, PINK, RED};
use crate::{H, Input, W};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MgKind {
    CrystalRain,
    Vault,
    Sequence,
    Targets,
    Asteroids,
}

pub const MG_KINDS: [MgKind; 5] = [
    MgKind::CrystalRain,
    MgKind::Vault,
    MgKind::Sequence,
    MgKind::Targets,
    MgKind::Asteroids,
];

impl MgKind {
    pub fn name(self) -> &'static str {
        match self {
            MgKind::CrystalRain => "CRYSTAL RAIN",
            MgKind::Vault => "STAR VAULT",
            MgKind::Sequence => "STAR SEQUENCE",
            MgKind::Targets => "TARGET PRACTICE",
            MgKind::Asteroids => "ASTEROID FIELD",
        }
    }
    fn how(self) -> [&'static str; 2] {
        match self {
            MgKind::CrystalRain => ["Catch the falling crystals.", "Dodge the rocks!"],
            MgKind::Vault => [
                "Press {FIRE} when the needle",
                "is on the golden band. 3 locks.",
            ],
            MgKind::Sequence => [
                "Memorize the stars that light up",
                "and repeat them with the ARROWS.",
            ],
            MgKind::Targets => [
                "Hit the targets with {FIRE}.",
                "Golden ones are worth more.",
            ],
            MgKind::Asteroids => ["Survive for 12 seconds.", "Every crash costs you."],
        }
    }
    fn color(self) -> Color {
        match self {
            MgKind::CrystalRain => CYAN,
            MgKind::Vault => GOLD,
            MgKind::Sequence => PINK,
            MgKind::Targets => GREEN,
            MgKind::Asteroids => Color::from_rgba(255, 150, 80, 255),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Grade {
    Defeat,
    Victory,
    Perfect,
}

impl Grade {
    pub fn label(self) -> &'static str {
        match self {
            Grade::Defeat => "NOT THIS TIME",
            Grade::Victory => "VICTORY!",
            Grade::Perfect => "PERFECT!",
        }
    }
}

struct Obj {
    pos: Vec2,
    vel: Vec2,
    kind: u8,
    r: f32,
    alive: bool,
}

pub struct Minigame {
    pub kind: MgKind,
    stage: u8, // 0 intro, 1 playing, 2 result
    t: f32,
    play_t: f32,
    ship: Vec2,
    objs: Vec<Obj>,
    shots: Vec<Vec2>,
    shot_cd: f32,
    spawn_t: f32,
    score: i32,
    total: i32,
    hits: i32,
    // vault
    needle: f32,
    needle_dir: f32,
    zone: (f32, f32),
    round: u8,
    flash: f32,
    flash_ok: bool,
    // sequence
    seq: Vec<u8>,
    shown: f32,
    last_shown: i32,
    input_i: usize,
    showing: bool,
    lit: Option<(u8, f32)>,
    pub grade: Grade,
    pub done: bool,
}

impl Minigame {
    pub fn new(kind: MgKind) -> Minigame {
        let mut m = Minigame {
            kind,
            stage: 0,
            t: 0.0,
            play_t: 0.0,
            ship: vec2(W / 2.0, H - 110.0),
            objs: vec![],
            shots: vec![],
            shot_cd: 0.0,
            spawn_t: 0.0,
            score: 0,
            total: 0,
            hits: 0,
            needle: 0.0,
            needle_dir: 1.0,
            zone: (0.0, 0.0),
            round: 0,
            flash: 0.0,
            flash_ok: false,
            seq: vec![],
            shown: 0.0,
            last_shown: -1,
            input_i: 0,
            showing: true,
            lit: None,
            grade: Grade::Defeat,
            done: false,
        };
        m.setup();
        m
    }

    fn setup(&mut self) {
        match self.kind {
            MgKind::Vault => self.new_zone(),
            MgKind::Sequence => {
                self.seq = (0..4).map(|_| gen_range(0, 4) as u8).collect();
            }
            MgKind::Targets => {
                for row in 0..3 {
                    for k in 0..4 {
                        let gold = gen_range(0, 6) == 0;
                        let dir = if row % 2 == 0 { 1.0 } else { -1.0 };
                        self.objs.push(Obj {
                            pos: vec2(
                                60.0 + k as f32 * 110.0 + row as f32 * 30.0,
                                170.0 + row as f32 * 90.0,
                            ),
                            vel: vec2(dir * (70.0 + row as f32 * 35.0), 0.0),
                            kind: if gold { 1 } else { 0 },
                            r: if gold { 13.0 } else { 18.0 },
                            alive: true,
                        });
                    }
                }
                self.total = self.objs.len() as i32;
            }
            _ => {}
        }
    }

    fn new_zone(&mut self) {
        let width = [0.2, 0.14, 0.09][self.round.min(2) as usize];
        let start = gen_range(0.15, 0.85 - width);
        self.zone = (start, start + width);
        self.needle = 0.0;
        self.needle_dir = 1.0;
    }

    fn duration(&self) -> f32 {
        match self.kind {
            MgKind::CrystalRain => 10.0,
            MgKind::Targets => 10.0,
            MgKind::Asteroids => 12.0,
            _ => 30.0,
        }
    }

    fn finish(&mut self) {
        self.grade = match self.kind {
            MgKind::CrystalRain => {
                if self.score >= 22 && self.hits == 0 {
                    Grade::Perfect
                } else if self.score >= 12 {
                    Grade::Victory
                } else {
                    Grade::Defeat
                }
            }
            MgKind::Vault => match self.score {
                3 => Grade::Perfect,
                2 => Grade::Victory,
                _ => Grade::Defeat,
            },
            MgKind::Sequence => match self.score {
                3 => Grade::Perfect,
                2 => Grade::Victory,
                _ => Grade::Defeat,
            },
            MgKind::Targets => {
                if self.score >= self.total {
                    Grade::Perfect
                } else if self.score >= self.total * 2 / 3 {
                    Grade::Victory
                } else {
                    Grade::Defeat
                }
            }
            MgKind::Asteroids => match self.hits {
                0 => Grade::Perfect,
                1 | 2 => Grade::Victory,
                _ => Grade::Defeat,
            },
        };
        self.stage = 2;
        self.t = 0.0;
    }

    fn move_ship(&mut self, dt: f32, inp: &Input, vertical: bool) {
        let mut d = Vec2::ZERO;
        if inp.left {
            d.x -= 1.0;
        }
        if inp.right {
            d.x += 1.0;
        }
        if vertical {
            if inp.up {
                d.y -= 1.0;
            }
            if inp.down {
                d.y += 1.0;
            }
        }
        self.ship += d.normalize_or_zero() * 330.0 * dt;
        self.ship.x = self.ship.x.clamp(24.0, W - 24.0);
        self.ship.y = self.ship.y.clamp(H * 0.45, H - 60.0);
    }

    /// Advances the minigame. Sets `done` once the result has been shown.
    pub fn update(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
        self.t += dt;
        self.flash = (self.flash - dt).max(0.0);
        if let Some((_, t)) = &mut self.lit {
            *t -= dt;
            if *t <= 0.0 {
                self.lit = None;
            }
        }
        match self.stage {
            0 => {
                if self.t > 2.2 || (self.t > 0.6 && inp.confirm) {
                    self.stage = 1;
                    self.t = 0.0;
                    au.play(Sfx::Beep);
                }
                return;
            }
            2 => {
                if self.t > 2.6 || (self.t > 0.8 && inp.confirm) {
                    self.done = true;
                }
                return;
            }
            _ => {}
        }
        self.play_t += dt;
        let fire = inp.confirm || (inp.fire && self.shot_cd <= 0.0 && self.kind == MgKind::Targets);
        match self.kind {
            MgKind::CrystalRain => {
                self.move_ship(dt, inp, false);
                self.spawn_t -= dt;
                if self.spawn_t <= 0.0 {
                    self.spawn_t = 0.22;
                    let rock = gen_range(0, 5) == 0;
                    self.objs.push(Obj {
                        pos: vec2(gen_range(20.0, W - 20.0), -10.0),
                        vel: vec2(0.0, gen_range(170.0, 260.0) + self.play_t * 12.0),
                        kind: rock as u8,
                        r: if rock { 15.0 } else { 9.0 },
                        alive: true,
                    });
                }
                let ship = self.ship;
                for o in &mut self.objs {
                    o.pos += o.vel * dt;
                    if o.alive && o.pos.distance(ship) < o.r + 14.0 {
                        o.alive = false;
                        if o.kind == 1 {
                            self.hits += 1;
                            self.score = (self.score - 3).max(0);
                            au.play(Sfx::Clang);
                        } else {
                            self.score += 1;
                            au.play_vol(Sfx::Crystal, 0.5);
                        }
                    }
                }
                self.objs.retain(|o| o.alive && o.pos.y < H + 20.0);
                if self.play_t >= self.duration() {
                    self.finish();
                }
            }
            MgKind::Vault => {
                let speed = [0.9, 1.25, 1.65][self.round.min(2) as usize];
                self.needle += self.needle_dir * speed * dt;
                if self.needle > 1.0 {
                    self.needle = 1.0;
                    self.needle_dir = -1.0;
                }
                if self.needle < 0.0 {
                    self.needle = 0.0;
                    self.needle_dir = 1.0;
                }
                if inp.confirm && self.flash <= 0.0 {
                    let ok = self.needle >= self.zone.0 && self.needle <= self.zone.1;
                    self.flash = 0.5;
                    self.flash_ok = ok;
                    if ok {
                        self.score += 1;
                        au.play(Sfx::Powerup);
                    } else {
                        au.play(Sfx::Deny);
                    }
                    self.round += 1;
                    if self.round >= 3 {
                        self.finish();
                    } else {
                        self.new_zone();
                    }
                }
            }
            MgKind::Sequence => {
                if self.showing {
                    self.shown += dt;
                    let step = 0.55;
                    let i = (self.shown / step) as usize;
                    if self.shown >= 0.0 && i < self.seq.len() {
                        if i as i32 != self.last_shown {
                            self.last_shown = i as i32;
                            self.lit = Some((self.seq[i], 0.38));
                            au.play_vol(Sfx::Beep, 0.6);
                        }
                    } else if self.shown > self.seq.len() as f32 * step + 0.2 {
                        self.showing = false;
                        self.input_i = 0;
                    }
                } else {
                    let pressed = if inp.up_p {
                        Some(0)
                    } else if inp.right_p {
                        Some(1)
                    } else if inp.down_p {
                        Some(2)
                    } else if inp.left_p {
                        Some(3)
                    } else {
                        None
                    };
                    if let Some(k) = pressed {
                        self.lit = Some((k, 0.2));
                        if self.seq[self.input_i] == k {
                            au.play_vol(Sfx::MenuMove, 0.8);
                            self.input_i += 1;
                            if self.input_i >= self.seq.len() {
                                self.score += 1;
                                au.play(Sfx::Powerup);
                                if self.score >= 3 {
                                    self.finish();
                                } else {
                                    self.seq.push(gen_range(0, 4) as u8);
                                    self.seq.push(gen_range(0, 4) as u8);
                                    self.showing = true;
                                    self.shown = -0.6;
                                    self.last_shown = -1;
                                }
                            }
                        } else {
                            au.play(Sfx::Deny);
                            self.finish();
                        }
                    }
                }
            }
            MgKind::Targets => {
                self.move_ship(dt, inp, false);
                self.shot_cd -= dt;
                if fire && self.shot_cd <= 0.0 {
                    self.shot_cd = 0.22;
                    self.shots.push(self.ship - vec2(0.0, 16.0));
                    au.play_vol(Sfx::Blaster, 0.4);
                }
                for s in &mut self.shots {
                    s.y -= 760.0 * dt;
                }
                for o in &mut self.objs {
                    o.pos += o.vel * dt;
                    if o.pos.x < 20.0 || o.pos.x > W - 20.0 {
                        o.vel.x = -o.vel.x;
                        o.pos.x = o.pos.x.clamp(20.0, W - 20.0);
                    }
                }
                let mut gained = 0;
                for s in &mut self.shots {
                    for o in &mut self.objs {
                        if o.alive && s.distance(o.pos) < o.r + 3.0 {
                            o.alive = false;
                            s.y = -100.0;
                            gained += 1;
                            au.play_vol(Sfx::Pop, 0.6);
                            break;
                        }
                    }
                }
                self.score += gained;
                self.shots.retain(|s| s.y > -20.0);
                if self.play_t >= self.duration() || self.objs.iter().all(|o| !o.alive) {
                    self.finish();
                }
            }
            MgKind::Asteroids => {
                self.move_ship(dt, inp, false);
                self.spawn_t -= dt;
                if self.spawn_t <= 0.0 {
                    self.spawn_t = (0.36 - self.play_t * 0.018).max(0.13);
                    let r = gen_range(10.0, 24.0);
                    self.objs.push(Obj {
                        pos: vec2(gen_range(10.0, W - 10.0), -30.0),
                        vel: vec2(gen_range(-50.0, 50.0), gen_range(200.0, 330.0)),
                        kind: 0,
                        r,
                        alive: true,
                    });
                }
                let ship = self.ship;
                for o in &mut self.objs {
                    o.pos += o.vel * dt;
                    if o.alive && o.pos.distance(ship) < o.r + 7.0 && self.flash <= 0.0 {
                        o.alive = false;
                        self.hits += 1;
                        self.flash = 0.8;
                        au.play(Sfx::Clang);
                    }
                }
                self.objs.retain(|o| o.alive && o.pos.y < H + 40.0);
                if self.play_t >= self.duration() {
                    self.score = 1;
                    self.finish();
                }
            }
        }
    }

    fn draw_ship(&self, time: f32, ship: &crate::sprites::Spr) {
        let s = self.ship;
        let blink =
            self.kind == MgKind::Asteroids && self.flash > 0.0 && (time * 20.0) as i32 % 2 == 0;
        if blink {
            return;
        }
        let flame = 6.0 + (time * 40.0).sin().abs() * 6.0;
        draw_triangle(
            s + vec2(-4.0, 18.0),
            s + vec2(4.0, 18.0),
            s + vec2(0.0, 18.0 + flame),
            ORANGE,
        );
        ship.draw(s, 3.0, 0.0, WHITE, false);
    }

    pub fn draw(&self, time: f32, ship: &crate::sprites::Spr) {
        let c = self.kind.color();
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.02, 0.01, 0.06, 0.94));
        for i in 0..40 {
            let x = ((i as f32 * 97.3).sin() * 0.5 + 0.5) * W;
            let y = ((i as f32 * 41.7 + time * 30.0 * (1.0 + (i % 3) as f32)) % H + H) % H;
            draw_circle(x, y, 1.0 + (i % 2) as f32, Color::new(1.0, 1.0, 1.0, 0.3));
        }
        draw_rectangle_lines(
            8.0,
            8.0,
            W - 16.0,
            H - 16.0,
            2.0,
            Color::new(c.r, c.g, c.b, 0.6),
        );
        ui::text_outline_fit(
            self.kind.name(),
            W / 2.0,
            52.0,
            26.0,
            c,
            Color::from_rgba(20, 5, 30, 255),
            W - 40.0,
        );

        match self.stage {
            0 => {
                ui::text_c("STAR RIFT", W / 2.0, 250.0, 16.0, DIM);
                let how = self.kind.how();
                ui::hint_c(how[0], W / 2.0, 320.0, 18.0, WHITE);
                ui::hint_c(how[1], W / 2.0, 346.0, 18.0, WHITE);
                let n = 3 - (self.t / 0.75) as i32;
                ui::text_outline(&format!("{}", n.max(1)), W / 2.0, 450.0, 60.0, c, BLACK);
                return;
            }
            2 => {
                let gc = match self.grade {
                    Grade::Defeat => RED,
                    Grade::Victory => GREEN,
                    Grade::Perfect => GOLD,
                };
                ui::text_outline_fit(
                    self.grade.label(),
                    W / 2.0,
                    H / 2.0,
                    40.0,
                    gc,
                    BLACK,
                    W - 30.0,
                );
                let sub = match self.grade {
                    Grade::Defeat => "A consolation prize: 5 crystals.",
                    Grade::Victory => "Reward: crystals, a weapon level and a power-up.",
                    Grade::Perfect => "Double reward and a card!",
                };
                ui::text_c(sub, W / 2.0, H / 2.0 + 40.0, 15.0, WHITE);
                return;
            }
            _ => {}
        }

        // timer
        if matches!(
            self.kind,
            MgKind::CrystalRain | MgKind::Targets | MgKind::Asteroids
        ) {
            let left = (self.duration() - self.play_t).max(0.0) / self.duration();
            draw_rectangle(30.0, 80.0, W - 60.0, 6.0, Color::new(0.2, 0.2, 0.3, 1.0));
            draw_rectangle(30.0, 80.0, (W - 60.0) * left, 6.0, c);
        }
        match self.kind {
            MgKind::CrystalRain => {
                for o in &self.objs {
                    if o.kind == 1 {
                        draw_poly(
                            o.pos.x,
                            o.pos.y,
                            7,
                            o.r,
                            time * 90.0,
                            Color::from_rgba(110, 90, 80, 255),
                        );
                        draw_poly_lines(
                            o.pos.x,
                            o.pos.y,
                            7,
                            o.r,
                            time * 90.0,
                            2.0,
                            Color::from_rgba(60, 45, 40, 255),
                        );
                    } else {
                        draw_poly(o.pos.x, o.pos.y, 4, o.r, 0.0, CYAN);
                        draw_poly(o.pos.x, o.pos.y - 2.0, 4, o.r * 0.45, 0.0, WHITE);
                    }
                }
                self.draw_ship(time, ship);
                ui::text_c(
                    &format!("CRYSTALS: {}", self.score),
                    W / 2.0,
                    120.0,
                    22.0,
                    CYAN,
                );
                ui::text_c(
                    "12 to win · 22 without crashing = perfect",
                    W / 2.0,
                    142.0,
                    12.0,
                    DIM,
                );
            }
            MgKind::Vault => {
                let (x0, x1, y) = (50.0, W - 50.0, H / 2.0);
                let bw = x1 - x0;
                draw_rectangle(x0, y - 18.0, bw, 36.0, Color::new(0.12, 0.1, 0.2, 1.0));
                draw_rectangle(
                    x0 + bw * self.zone.0,
                    y - 18.0,
                    bw * (self.zone.1 - self.zone.0),
                    36.0,
                    GOLD,
                );
                draw_rectangle_lines(x0, y - 18.0, bw, 36.0, 2.0, WHITE);
                let nx = x0 + bw * self.needle;
                draw_triangle(
                    vec2(nx, y - 26.0),
                    vec2(nx - 8.0, y - 40.0),
                    vec2(nx + 8.0, y - 40.0),
                    WHITE,
                );
                draw_line(nx, y - 26.0, nx, y + 26.0, 3.0, WHITE);
                for i in 0..3 {
                    let lx = W / 2.0 - 50.0 + i as f32 * 50.0;
                    let col = if (i as u8) < self.round {
                        if i < self.score { GOLD } else { RED }
                    } else {
                        Color::new(0.3, 0.3, 0.4, 1.0)
                    };
                    draw_circle(lx, y + 90.0, 14.0, col);
                    draw_circle_lines(lx, y + 90.0, 14.0, 2.0, WHITE);
                }
                if self.flash > 0.0 {
                    ui::text_c(
                        if self.flash_ok {
                            "LOCK OPENED!"
                        } else {
                            "MISSED!"
                        },
                        W / 2.0,
                        y - 70.0,
                        24.0,
                        if self.flash_ok { GOLD } else { RED },
                    );
                }
                ui::hint_c("{FIRE} to lock", W / 2.0, y + 150.0, 14.0, DIM);
            }
            MgKind::Sequence => {
                let center = vec2(W / 2.0, H / 2.0 + 20.0);
                let dirs = [
                    vec2(0.0, -1.0),
                    vec2(1.0, 0.0),
                    vec2(0.0, 1.0),
                    vec2(-1.0, 0.0),
                ];
                let cols = [GOLD, PINK, CYAN, GREEN];
                for k in 0..4 {
                    let p = center + dirs[k] * 110.0;
                    let lit = matches!(self.lit, Some((j, _)) if j as usize == k);
                    let col = cols[k];
                    let a = if lit { 1.0 } else { 0.25 };
                    if lit {
                        draw_circle(p.x, p.y, 54.0, Color::new(col.r, col.g, col.b, 0.25));
                    }
                    draw_star(
                        p,
                        40.0,
                        16.0,
                        Color::new(col.r, col.g, col.b, a),
                        time * 20.0,
                    );
                }
                let label = if self.showing {
                    "WATCH..."
                } else {
                    "YOUR TURN!"
                };
                ui::text_c(
                    label,
                    W / 2.0,
                    150.0,
                    22.0,
                    if self.showing { DIM } else { WHITE },
                );
                ui::text_c(
                    &format!("ROUND {} of 3 · {} stars", self.score + 1, self.seq.len()),
                    W / 2.0,
                    176.0,
                    14.0,
                    DIM,
                );
                if !self.showing {
                    ui::text_c(
                        &format!("{} / {}", self.input_i, self.seq.len()),
                        W / 2.0,
                        H - 90.0,
                        18.0,
                        WHITE,
                    );
                }
            }
            MgKind::Targets => {
                for o in &self.objs {
                    if !o.alive {
                        continue;
                    }
                    let col = if o.kind == 1 { GOLD } else { RED };
                    draw_circle(o.pos.x, o.pos.y, o.r, WHITE);
                    draw_circle(o.pos.x, o.pos.y, o.r * 0.7, col);
                    draw_circle(o.pos.x, o.pos.y, o.r * 0.4, WHITE);
                    draw_circle(o.pos.x, o.pos.y, o.r * 0.15, col);
                }
                for s in &self.shots {
                    draw_line(s.x, s.y, s.x, s.y + 12.0, 3.0, GOLD);
                }
                self.draw_ship(time, ship);
                ui::text_c(
                    &format!("TARGETS: {} / {}", self.score, self.total),
                    W / 2.0,
                    120.0,
                    22.0,
                    GREEN,
                );
            }
            MgKind::Asteroids => {
                for o in &self.objs {
                    draw_poly(
                        o.pos.x,
                        o.pos.y,
                        7,
                        o.r,
                        time * 60.0 + o.r * 10.0,
                        Color::from_rgba(130, 100, 85, 255),
                    );
                    draw_poly_lines(
                        o.pos.x,
                        o.pos.y,
                        7,
                        o.r,
                        time * 60.0 + o.r * 10.0,
                        2.0,
                        Color::from_rgba(70, 50, 45, 255),
                    );
                }
                self.draw_ship(time, ship);
                let hearts = 3 - self.hits.min(3);
                let col = if hearts > 1 { GREEN } else { RED };
                ui::text_r("SHIELD", W / 2.0 - 10.0, 120.0, 18.0, col);
                for i in 0..3 {
                    let x = W / 2.0 + 10.0 + i as f32 * 22.0;
                    if i < hearts {
                        draw_circle(x, 114.0, 7.0, col);
                    }
                    draw_circle_lines(x, 114.0, 7.0, 1.5, WHITE);
                }
            }
        }
    }
}

fn draw_star(c: Vec2, r_out: f32, r_in: f32, col: Color, rot: f32) {
    let n = 10;
    for i in 0..n {
        let a0 = (rot + i as f32 * 36.0 - 90.0).to_radians();
        let a1 = (rot + (i + 1) as f32 * 36.0 - 90.0).to_radians();
        let r0 = if i % 2 == 0 { r_out } else { r_in };
        let r1 = if i % 2 == 0 { r_in } else { r_out };
        draw_triangle(
            c,
            c + vec2(a0.cos(), a0.sin()) * r0,
            c + vec2(a1.cos(), a1.sin()) * r1,
            col,
        );
    }
    let _ = TAU;
}
