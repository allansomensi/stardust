//! Chefes: Rainha, Couraçado, Serpente, Tecelã, Olho, Gêmeas, Cometa e Devorador.

use crate::art::Art;
use crate::audio::Sfx;
use crate::game::{EBullet, Kind};
use crate::sprites::Sprites;
use crate::{H, W};
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BossKind {
    Queen,
    Warship,
    Serpent,
    Weaver,
    Eye,
    Twins,
    Devourer,
    Comet,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PartKind {
    Claw,
    Turret,
    Cannon,
    Segment,
    Leg,
    Sister,
    EyeOrb,
    Shard,
}

struct Part {
    pos: Vec2,
    off: Vec2,
    hp: f32,
    kind: PartKind,
    cd: f32,
    flash: f32,
}

struct Beam {
    x: f32,
    y: f32,
    warn: f32,
    on: f32,
}

#[derive(Default)]
pub struct BossOut {
    pub bullets: Vec<EBullet>,
    pub spawns: Vec<(Kind, Vec2, Vec2)>,
    pub sfx: Vec<Sfx>,
    pub explosions: Vec<(Vec2, bool)>,
}

#[derive(Clone, Copy)]
pub enum HitTarget {
    Body,
    Part(usize),
}

pub struct DamageRes {
    pub part_destroyed: bool,
    pub part_pos: Vec2,
    pub killed: bool,
    pub blocked: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Pattern {
    Radial,
    Fan,
    Minions,
    Spiral,
    Curtain,
    Laser,
    Volley,
    Rush,
    Web,
    Net,
    Beams,
    Gaze,
    Pull,
    Shower,
}

struct Active {
    pattern: Pattern,
    elapsed: f32,
    sub_cd: f32,
    count: u32,
}

pub struct Boss {
    pub kind: BossKind,
    pub pos: Vec2,
    pub hp: f32,
    pub max_hp: f32,
    t: f32,
    move_t: f32,
    enter_t: f32,
    parts: Vec<Part>,
    flash: f32,
    dying: f32,
    explode_cd: f32,
    pattern_cd: f32,
    pattern_idx: usize,
    active: Option<Active>,
    spiral: f32,
    beam: Option<Beam>,
    lvl: f32,
    pub final_boss: bool,
    pub variant: bool,
    aim: Vec2,
    trail: Vec<Vec2>,
    // olho
    open: bool,
    lid_t: f32,
    beam_angles: Vec<f32>,
    beam_warn: f32,
    beam_on: f32,
    gaze: Option<(Vec2, f32)>,
    rush: f32,
    pub pull: f32,
    parts_total: usize,
}

const ENTER_TIME: f32 = 2.6;

impl Boss {
    pub fn new(kind: BossKind, lvl: f32, final_boss: bool, variant: bool) -> Boss {
        let mult = if final_boss {
            1.6
        } else if variant {
            1.3
        } else {
            1.0
        };
        let part = |off: Vec2, hp: f32, kind: PartKind, cd: f32| Part {
            pos: Vec2::ZERO,
            off,
            hp,
            kind,
            cd,
            flash: 0.0,
        };
        let (hp, parts) = match kind {
            BossKind::Queen => (
                (160.0 + 80.0 * lvl) * mult,
                vec![
                    part(vec2(-60.0, 24.0), 16.0 + 16.0 * lvl, PartKind::Claw, 1.0),
                    part(vec2(60.0, 24.0), 16.0 + 16.0 * lvl, PartKind::Claw, 1.6),
                ],
            ),
            BossKind::Warship => {
                let mut p = vec![
                    part(vec2(-44.0, 4.0), 14.0 + 16.0 * lvl, PartKind::Turret, 1.2),
                    part(vec2(44.0, 4.0), 14.0 + 16.0 * lvl, PartKind::Turret, 1.8),
                    part(vec2(0.0, 36.0), 20.0 + 25.0 * lvl, PartKind::Cannon, 99.0),
                ];
                if variant {
                    p.push(part(
                        vec2(-80.0, -6.0),
                        14.0 + 14.0 * lvl,
                        PartKind::Turret,
                        0.9,
                    ));
                    p.push(part(
                        vec2(80.0, -6.0),
                        14.0 + 14.0 * lvl,
                        PartKind::Turret,
                        1.5,
                    ));
                }
                ((180.0 + 80.0 * lvl) * mult, p)
            }
            BossKind::Serpent => {
                let n = if variant { 14 } else { 10 };
                (
                    (120.0 + 70.0 * lvl) * mult,
                    (0..n)
                        .map(|i| {
                            part(
                                Vec2::ZERO,
                                8.0 + 6.0 * lvl,
                                PartKind::Segment,
                                1.0 + i as f32 * 0.3,
                            )
                        })
                        .collect(),
                )
            }
            BossKind::Weaver => (
                (170.0 + 80.0 * lvl) * mult,
                [
                    (-62.0, -24.0),
                    (62.0, -24.0),
                    (-74.0, 12.0),
                    (74.0, 12.0),
                    (-58.0, 46.0),
                    (58.0, 46.0),
                ]
                .iter()
                .enumerate()
                .map(|(i, (x, y))| {
                    part(
                        vec2(*x, *y),
                        10.0 + 10.0 * lvl,
                        PartKind::Leg,
                        1.0 + i as f32 * 0.4,
                    )
                })
                .collect(),
            ),
            BossKind::Eye => ((200.0 + 90.0 * lvl) * mult, vec![]),
            BossKind::Twins => (
                (150.0 + 70.0 * lvl) * mult,
                vec![part(
                    Vec2::ZERO,
                    (120.0 + 56.0 * lvl) * mult,
                    PartKind::Sister,
                    1.2,
                )],
            ),
            BossKind::Comet => (
                (190.0 + 85.0 * lvl) * mult,
                (0..4)
                    .map(|i| {
                        part(
                            Vec2::ZERO,
                            14.0 + 12.0 * lvl,
                            PartKind::Shard,
                            1.4 + i as f32 * 0.35,
                        )
                    })
                    .collect(),
            ),
            BossKind::Devourer => (
                (650.0 + 220.0 * lvl) * mult,
                (0..6)
                    .map(|i| {
                        part(
                            Vec2::ZERO,
                            24.0 + 14.0 * lvl,
                            PartKind::EyeOrb,
                            1.0 + i as f32 * 0.35,
                        )
                    })
                    .collect(),
            ),
        };
        let parts_total = parts.len();
        Boss {
            kind,
            pos: vec2(W / 2.0, -130.0),
            hp,
            max_hp: hp,
            t: 0.0,
            move_t: 0.0,
            enter_t: 0.0,
            parts,
            flash: 0.0,
            dying: -1.0,
            explode_cd: 0.0,
            pattern_cd: 1.0,
            pattern_idx: 0,
            active: None,
            spiral: 0.0,
            beam: None,
            lvl,
            final_boss,
            variant,
            aim: vec2(W / 2.0, H),
            trail: vec![],
            open: true,
            lid_t: 0.0,
            beam_angles: vec![],
            beam_warn: 0.0,
            beam_on: 0.0,
            gaze: None,
            rush: 0.0,
            pull: 0.0,
            parts_total,
        }
    }

    /// Índice no bestiário (a partir de LORE_BOSS_START).
    pub fn lore_id(&self) -> usize {
        match (self.kind, self.final_boss) {
            (BossKind::Queen, true) => 5,
            (BossKind::Queen, false) => 0,
            (BossKind::Warship, _) => 1,
            (BossKind::Serpent, _) => 2,
            (BossKind::Weaver, _) => 3,
            (BossKind::Eye, _) => 4,
            (BossKind::Twins, _) => 6,
            (BossKind::Devourer, _) => 7,
            (BossKind::Comet, _) => 8,
        }
    }

    /// Índice absoluto no bestiário.
    pub fn lore_index(&self) -> usize {
        match self.lore_id() {
            6 => crate::progress::LORE_TWINS,
            7 => crate::progress::LORE_DEVOURER,
            8 => crate::progress::LORE_COMET,
            n => crate::progress::LORE_BOSS_START + n,
        }
    }

    pub fn name(&self) -> &'static str {
        match (self.kind, self.final_boss, self.variant) {
            (BossKind::Queen, true, _) => "RAINHA SUPREMA DO ENXAME",
            (BossKind::Queen, false, _) => "RAINHA DA COLMEIA",
            (BossKind::Warship, _, true) => "COURAÇADO ÔMEGA",
            (BossKind::Warship, _, false) => "COURAÇADO ZÊNITE",
            (BossKind::Serpent, _, true) => "SERPENTE ANCESTRAL",
            (BossKind::Serpent, _, false) => "SERPENTE COMETA",
            (BossKind::Weaver, _, _) => "TECELÃ ABISSAL",
            (BossKind::Eye, _, true) => "OLHO ANCESTRAL",
            (BossKind::Eye, _, false) => "OLHO DO VAZIO",
            (BossKind::Twins, _, _) => "AS GÊMEAS",
            (BossKind::Devourer, _, _) => "O DEVORADOR",
            (BossKind::Comet, _, true) => "COMETA WILD ANCESTRAL",
            (BossKind::Comet, _, false) => "COMETA WILD",
        }
    }

    pub fn entered(&self) -> bool {
        self.enter_t >= ENTER_TIME
    }

    pub fn targetable(&self) -> bool {
        self.entered() && self.hp > 0.0
    }

    pub fn all_parts_destroyed(&self) -> bool {
        self.parts_total > 0 && self.parts.iter().all(|p| p.hp <= 0.0)
    }

    fn phase(&self) -> u32 {
        let r = self.hp / self.max_hp;
        let base = if r > 0.6 {
            0
        } else if r > 0.3 {
            1
        } else {
            2
        };
        if self.kind == BossKind::Twins && !self.part_alive(PartKind::Sister) {
            (base + 1).min(2)
        } else {
            base
        }
    }

    fn part_alive(&self, k: PartKind) -> bool {
        self.parts.iter().any(|p| p.kind == k && p.hp > 0.0)
    }

    fn speed(&self) -> f32 {
        0.9 + 0.1 * self.lvl
    }

    fn pattern_list(&self, ph: u32) -> Vec<Pattern> {
        use Pattern::*;
        match self.kind {
            BossKind::Queen => {
                let mut l = vec![Radial, Fan, Minions];
                l.push(if ph >= 1 || self.final_boss {
                    Spiral
                } else {
                    Radial
                });
                if self.final_boss {
                    l.extend([Curtain, Web, Beams]);
                }
                l.push(Fan);
                l
            }
            BossKind::Warship => {
                let laser = if self.part_alive(PartKind::Cannon) {
                    Laser
                } else {
                    Curtain
                };
                let mut l = vec![Fan, laser, Curtain, if ph >= 1 { Radial } else { Fan }];
                if self.variant {
                    l.push(laser);
                    l.push(Spiral);
                }
                l
            }
            BossKind::Serpent => {
                let mut l = vec![Volley, Fan, Volley, if ph >= 1 { Radial } else { Fan }];
                if ph >= 2 || self.variant {
                    l.push(Rush);
                }
                l
            }
            BossKind::Weaver => vec![Web, Net, Minions, if ph >= 1 { Radial } else { Web }, Net],
            BossKind::Eye => {
                let mut l = vec![Beams, Radial, Gaze];
                if ph >= 1 {
                    l.push(Spiral);
                }
                l
            }
            BossKind::Twins => vec![
                Fan,
                Radial,
                Fan,
                if ph >= 1 { Spiral } else { Curtain },
                Minions,
            ],
            BossKind::Comet => {
                let mut l = vec![Shower, Radial, Volley, Shower];
                l.push(if ph >= 1 { Spiral } else { Fan });
                if ph >= 2 || self.variant {
                    l.push(Rush);
                    l.push(Shower);
                }
                l
            }
            BossKind::Devourer => match ph {
                0 => vec![Radial, Gaze, Spiral, Pull],
                1 => vec![Beams, Radial, Web, Pull, Curtain, Gaze],
                _ => vec![Spiral, Beams, Pull, Net, Radial, Minions],
            },
        }
    }

    pub fn update(&mut self, dt: f32, player: Vec2, bmul: f32, out: &mut BossOut) {
        self.t += dt;
        self.aim = player;
        self.pull = (self.pull - dt).max(0.0);
        self.flash = (self.flash - dt).max(0.0);
        for p in &mut self.parts {
            p.flash = (p.flash - dt).max(0.0);
        }
        if self.hp <= 0.0 {
            if self.dying < 0.0 {
                self.dying = 0.0;
            }
            self.dying += dt;
            self.beam = None;
            self.beam_angles.clear();
            self.gaze = None;
            self.explode_cd -= dt;
            if self.explode_cd <= 0.0 {
                self.explode_cd = 0.11;
                let o = vec2(gen_range(-50.0, 50.0), gen_range(-30.0, 30.0));
                out.explosions.push((self.pos + o, gen_range(0, 4) == 0));
                out.sfx.push(Sfx::Boom);
            }
            self.pos.y += 12.0 * dt;
            self.place_parts();
            return;
        }
        if self.enter_t < ENTER_TIME {
            self.enter_t += dt;
            let k = (self.enter_t / ENTER_TIME).min(1.0);
            let e = 1.0 - (1.0 - k).powi(3);
            self.pos = vec2(W / 2.0, -130.0 + (150.0 + 130.0) * e);
            if matches!(self.kind, BossKind::Serpent | BossKind::Comet) {
                self.trail.insert(0, self.pos);
                self.trail.truncate(600);
            }
            self.place_parts();
            return;
        }
        let ph = self.phase();
        let bs = bmul * self.speed();

        // movimento
        let frozen = self.beam.is_some() || self.beam_on > 0.0 || self.beam_warn > 0.0;
        if !frozen {
            self.move_t += dt * (1.0 + ph as f32 * 0.25) * if self.rush > 0.0 { 2.4 } else { 1.0 };
        }
        self.rush = (self.rush - dt).max(0.0);
        match self.kind {
            BossKind::Serpent => {
                let sp = if self.variant { 1.2 } else { 1.0 };
                let t = self.move_t * sp;
                self.pos = vec2(
                    W / 2.0 + (t * 0.8).sin() * W * 0.36,
                    230.0 + (t * 1.6).sin() * 130.0,
                );
                self.trail.insert(0, self.pos);
                self.trail.truncate(600);
            }
            BossKind::Eye | BossKind::Weaver => {
                self.pos.x = W / 2.0 + (self.move_t * 0.35).sin() * W * 0.2;
                self.pos.y = 150.0 + (self.t * 0.7).sin() * 12.0;
            }
            BossKind::Twins => {
                self.pos.x = W / 2.0 - 110.0 + (self.move_t * 0.7).sin() * 70.0;
                self.pos.y = 160.0 + (self.t * 1.1).sin() * 30.0;
            }
            BossKind::Comet => {
                let t = self.move_t * if self.variant { 1.15 } else { 1.0 };
                self.pos = vec2(
                    W / 2.0 + (t * 0.55).sin() * W * 0.34,
                    150.0 + (t * 1.1).sin() * 45.0,
                );
                self.trail.insert(0, self.pos);
                self.trail.truncate(90);
            }
            BossKind::Devourer => {
                self.pos.x = W / 2.0 + (self.move_t * 0.4).sin() * W * 0.16;
                self.pos.y = 175.0 + (self.t * 0.5).sin() * 15.0;
            }
            _ => {
                self.pos.x = W / 2.0 + (self.move_t * 0.5).sin() * W * 0.27;
                self.pos.y = 150.0 + (self.t * 0.9).sin() * 16.0;
            }
        }
        self.place_parts();

        // pálpebra do olho
        if self.kind == BossKind::Eye {
            self.lid_t += dt;
            let (open_t, closed_t) = if ph >= 2 { (2.6, 1.4) } else { (3.6, 2.0) };
            if self.open && self.lid_t > open_t {
                self.open = false;
                self.lid_t = 0.0;
                out.sfx.push(Sfx::Clang);
            } else if !self.open && self.lid_t > closed_t {
                self.open = true;
                self.lid_t = 0.0;
            }
        }

        // partes atiram sozinhas
        let spd = self.speed();
        for p in &mut self.parts {
            if p.hp <= 0.0 || p.kind == PartKind::Cannon {
                continue;
            }
            p.cd -= dt;
            if p.cd <= 0.0 {
                let base = match p.kind {
                    PartKind::Segment => 3.6,
                    PartKind::Leg => 2.6,
                    _ => 1.8,
                };
                p.cd = (base - ph as f32 * 0.35).max(0.8) / spd;
                let from = p.pos;
                let dir = (player - from).normalize_or_zero();
                let n = if p.kind == PartKind::Turret { 2 } else { 1 };
                for k in 0..n {
                    let a = dir.y.atan2(dir.x) + (k as f32 - (n - 1) as f32 / 2.0) * 0.12;
                    out.bullets.push(EBullet::new(
                        from,
                        vec2(a.cos(), a.sin()) * 200.0 * bs,
                        4.0,
                        0,
                    ));
                }
                out.sfx.push(Sfx::EnemyShot);
            }
        }

        // laser do couraçado
        if let Some(b) = &mut self.beam {
            if let Some(c) = self.parts.iter().find(|p| p.kind == PartKind::Cannon) {
                b.x = c.pos.x;
                b.y = c.pos.y + 10.0;
            }
            if b.warn > 0.0 {
                b.warn -= dt;
                if b.warn <= 0.0 {
                    out.sfx.push(Sfx::Bomb);
                }
            } else {
                b.on -= dt;
            }
            if b.on <= 0.0 || !self.part_alive(PartKind::Cannon) {
                self.beam = None;
            }
        }

        // raios giratórios do olho
        if self.beam_warn > 0.0 {
            self.beam_warn -= dt;
            if self.beam_warn <= 0.0 {
                out.sfx.push(Sfx::Thunder);
            }
        } else if self.beam_on > 0.0 {
            self.beam_on -= dt;
            let dir = if self.variant { -1.0 } else { 1.0 };
            for a in &mut self.beam_angles {
                *a += dt * 0.55 * dir * (1.0 + ph as f32 * 0.2);
            }
            if self.beam_on <= 0.0 {
                self.beam_angles.clear();
            }
        }

        // olhar (rajada mirada)
        if let Some((target, t)) = &mut self.gaze {
            *t -= dt;
            if *t <= 0.0 {
                let dir = (*target - self.pos).normalize_or_zero();
                for k in 0..10 {
                    out.bullets.push(EBullet::new(
                        self.pos + dir * (20.0 + k as f32 * 14.0),
                        dir * 330.0 * bs,
                        4.0,
                        4,
                    ));
                }
                out.sfx.push(Sfx::Laser);
                self.gaze = None;
            }
        }

        // padrões
        let list = self.pattern_list(ph);
        let pos = self.pos;
        match &mut self.active {
            None => {
                self.pattern_cd -= dt;
                if self.pattern_cd <= 0.0 {
                    let pattern = list[self.pattern_idx % list.len()];
                    self.pattern_idx += 1;
                    self.active = Some(Active {
                        pattern,
                        elapsed: 0.0,
                        sub_cd: 0.0,
                        count: 0,
                    });
                }
            }
            Some(a) => {
                a.elapsed += dt;
                a.sub_cd -= dt;
                let fire = a.sub_cd <= 0.0;
                let mut done = false;
                match a.pattern {
                    Pattern::Radial => {
                        if fire {
                            a.sub_cd = 0.5;
                            a.count += 1;
                            let n = 12 + 4 * ph;
                            self.spiral += 0.21;
                            for i in 0..n {
                                let ang = self.spiral + i as f32 / n as f32 * TAU;
                                out.bullets.push(EBullet::new(
                                    pos,
                                    vec2(ang.cos(), ang.sin()) * 150.0 * bs,
                                    4.5,
                                    0,
                                ));
                            }
                            out.sfx.push(Sfx::EnemyShot);
                            done = a.count >= 2 + ph.min(1);
                        }
                    }
                    Pattern::Fan => {
                        if fire {
                            a.sub_cd = 0.38;
                            a.count += 1;
                            let n = 5 + 2 * ph;
                            let dir = (player - pos).normalize_or_zero();
                            let base = dir.y.atan2(dir.x);
                            for i in 0..n {
                                let ang = base + (i as f32 - (n - 1) as f32 / 2.0) * 0.16;
                                out.bullets.push(EBullet::new(
                                    pos + vec2(0.0, 20.0),
                                    vec2(ang.cos(), ang.sin()) * 205.0 * bs,
                                    4.0,
                                    1,
                                ));
                            }
                            out.sfx.push(Sfx::EnemyShot);
                            done = a.count >= 3;
                        }
                    }
                    Pattern::Minions => {
                        let kind = if self.kind == BossKind::Weaver {
                            Kind::Tecela
                        } else {
                            Kind::Kamikaze
                        };
                        for s in [-1.0, 1.0] {
                            out.spawns.push((
                                kind,
                                pos + vec2(s * 40.0, 30.0),
                                vec2(s * 90.0, 140.0),
                            ));
                        }
                        if ph >= 1 {
                            out.spawns.push((
                                Kind::Zangao,
                                pos + vec2(0.0, 40.0),
                                vec2(0.0, 160.0),
                            ));
                        }
                        out.sfx.push(Sfx::Powerup);
                        done = true;
                    }
                    Pattern::Spiral => {
                        if fire {
                            a.sub_cd = 0.065;
                            let arms = 2 + ph.min(1);
                            self.spiral += 0.3;
                            for k in 0..arms {
                                let ang = self.spiral + k as f32 / arms as f32 * TAU;
                                out.bullets.push(EBullet::new(
                                    pos,
                                    vec2(ang.cos(), ang.sin()) * 140.0 * bs,
                                    4.0,
                                    2,
                                ));
                            }
                        }
                        done = a.elapsed > 2.6;
                    }
                    Pattern::Curtain => {
                        if fire {
                            a.sub_cd = 0.6;
                            a.count += 1;
                            let n = 15;
                            let gap = gen_range(2, n - 4);
                            for i in 0..n {
                                if (gap..gap + 3).contains(&i) {
                                    continue;
                                }
                                let x = 16.0 + i as f32 * (W - 32.0) / (n - 1) as f32;
                                out.bullets.push(EBullet::new(
                                    vec2(x, pos.y + 30.0),
                                    vec2(0.0, 150.0 * bs),
                                    4.5,
                                    1,
                                ));
                            }
                            out.sfx.push(Sfx::EnemyShot);
                            done = a.count >= 3;
                        }
                    }
                    Pattern::Laser => {
                        if a.count == 0 {
                            a.count = 1;
                            self.beam = Some(Beam {
                                x: pos.x,
                                y: pos.y + 46.0,
                                warn: 1.0,
                                on: 1.2,
                            });
                            out.sfx.push(Sfx::Warning);
                        }
                        done = self.beam.is_none();
                    }
                    Pattern::Volley => {
                        if fire {
                            a.sub_cd = 0.09;
                            let alive: Vec<Vec2> = self
                                .parts
                                .iter()
                                .filter(|p| p.hp > 0.0)
                                .map(|p| p.pos)
                                .collect();
                            let i = a.count as usize;
                            if i < alive.len() {
                                let from = alive[i];
                                let dir = (player - from).normalize_or_zero();
                                out.bullets
                                    .push(EBullet::new(from, dir * 190.0 * bs, 4.0, 1));
                                out.sfx.push(Sfx::EnemyShot);
                            }
                            a.count += 1;
                            done = a.count as usize >= alive.len().max(1) + 2;
                        }
                    }
                    Pattern::Rush => {
                        if a.count == 0 {
                            a.count = 1;
                            self.rush = 2.5;
                            out.sfx.push(Sfx::Warning);
                        }
                        done = self.rush <= 0.0;
                    }
                    Pattern::Web => {
                        if fire {
                            a.sub_cd = 0.45;
                            a.count += 1;
                            let dir = (player - pos).normalize_or_zero();
                            let base = dir.y.atan2(dir.x);
                            for k in -1..=1 {
                                let ang = base + k as f32 * 0.35;
                                out.bullets.push(EBullet::new(
                                    pos + vec2(0.0, 30.0),
                                    vec2(ang.cos(), ang.sin()) * 120.0 * bs,
                                    9.0,
                                    3,
                                ));
                            }
                            out.sfx.push(Sfx::Web);
                            done = a.count >= 3;
                        }
                    }
                    Pattern::Net => {
                        if fire {
                            a.sub_cd = 0.9;
                            a.count += 1;
                            let n = 11;
                            let gap = gen_range(1, n - 3);
                            for i in 0..n {
                                if (gap..gap + 2).contains(&i) {
                                    continue;
                                }
                                let x = 24.0 + i as f32 * (W - 48.0) / (n - 1) as f32;
                                out.bullets.push(EBullet::new(
                                    vec2(x, pos.y + 40.0),
                                    vec2(0.0, 105.0 * bs),
                                    8.0,
                                    3,
                                ));
                            }
                            out.sfx.push(Sfx::Web);
                            done = a.count >= 2;
                        }
                    }
                    Pattern::Beams => {
                        if a.count == 0 {
                            a.count = 1;
                            let n = if self.variant || ph >= 2 || self.final_boss {
                                4
                            } else {
                                2
                            };
                            let start = gen_range(0.0, PI);
                            self.beam_angles =
                                (0..n).map(|i| start + i as f32 / n as f32 * TAU).collect();
                            self.beam_warn = 1.0;
                            self.beam_on = 4.0;
                            out.sfx.push(Sfx::Warning);
                        }
                        done = self.beam_on <= 0.0;
                    }
                    Pattern::Pull => {
                        if a.count == 0 {
                            a.count = 1;
                            self.pull = 3.0;
                            out.sfx.push(Sfx::Devour);
                        }
                        if fire {
                            a.sub_cd = 0.12;
                            self.spiral -= 0.35;
                            for k in 0..3 {
                                let ang = self.spiral + k as f32 / 3.0 * TAU;
                                out.bullets.push(EBullet::new(
                                    pos,
                                    vec2(ang.cos(), ang.sin()) * 120.0 * bs,
                                    4.5,
                                    4,
                                ));
                            }
                        }
                        done = self.pull <= 0.0;
                    }
                    Pattern::Shower => {
                        if fire {
                            a.sub_cd = 0.06 - ph as f32 * 0.01;
                            let back = self.trail.get(18).copied().unwrap_or(pos - vec2(0.0, 20.0));
                            let from = back + vec2(gen_range(-26.0, 26.0), gen_range(-10.0, 10.0));
                            out.bullets.push(EBullet::new(
                                from,
                                vec2(gen_range(-45.0, 45.0), gen_range(95.0, 150.0)) * bs,
                                3.5,
                                2,
                            ));
                            if a.count % 6 == 0 {
                                out.sfx.push(Sfx::EnemyShot);
                            }
                            a.count += 1;
                        }
                        done = a.elapsed > 2.4;
                    }
                    Pattern::Gaze => {
                        if fire {
                            a.sub_cd = 1.4;
                            a.count += 1;
                            self.gaze = Some((player, 0.9));
                            out.sfx.push(Sfx::Beep);
                            done = a.count >= 3;
                        }
                    }
                }
                if done {
                    self.active = None;
                    let base = if self.final_boss { 1.1 } else { 1.4 };
                    self.pattern_cd = (base - ph as f32 * 0.3).max(0.5);
                }
            }
        }
    }

    fn place_parts(&mut self) {
        let pos = self.pos;
        let spacing = 14usize;
        for (i, p) in self.parts.iter_mut().enumerate() {
            p.pos = match p.kind {
                PartKind::Segment => {
                    let idx = (i + 1) * spacing;
                    self.trail
                        .get(idx)
                        .copied()
                        .or_else(|| self.trail.last().copied())
                        .unwrap_or(pos)
                }
                PartKind::Leg => pos + p.off + vec2(0.0, (self.t * 3.0 + p.off.x).sin() * 4.0),
                PartKind::Sister => vec2(W - pos.x, pos.y + (self.t * 1.3).sin() * 24.0),
                PartKind::EyeOrb => {
                    let a = self.t * 0.4 + i as f32 / 6.0 * TAU;
                    pos + vec2(a.cos(), a.sin()) * 76.0
                }
                PartKind::Shard => {
                    let a = self.t * 1.1 + i as f32 / 4.0 * TAU;
                    pos + vec2(a.cos(), a.sin() * 0.7)
                        * (62.0 + (self.t * 2.0 + i as f32).sin() * 8.0)
                }
                _ => pos + p.off,
            };
        }
    }

    pub fn hit_test(&self, p: Vec2) -> Option<HitTarget> {
        for (i, part) in self.parts.iter().enumerate() {
            let r = match part.kind {
                PartKind::Segment | PartKind::EyeOrb | PartKind::Shard => 14.0,
                PartKind::Sister => 30.0,
                _ => 17.0,
            };
            if part.hp > 0.0 && p.distance(part.pos) < r {
                return Some(HitTarget::Part(i));
            }
        }
        let d = p - self.pos;
        let (rx, ry) = match self.kind {
            BossKind::Queen => (42.0, 34.0),
            BossKind::Warship => (50.0, 26.0),
            BossKind::Serpent => (22.0, 22.0),
            BossKind::Weaver => (42.0, 38.0),
            BossKind::Eye => (40.0, 30.0),
            BossKind::Twins => (34.0, 28.0),
            BossKind::Devourer => (58.0, 52.0),
            BossKind::Comet => (36.0, 32.0),
        };
        if (d.x / rx).powi(2) + (d.y / ry).powi(2) <= 1.0 {
            return Some(HitTarget::Body);
        }
        None
    }

    pub fn damage(&mut self, t: HitTarget, dmg: f32) -> DamageRes {
        let mut res = DamageRes {
            part_destroyed: false,
            part_pos: self.pos,
            killed: false,
            blocked: false,
        };
        let was_alive = self.hp > 0.0;
        match t {
            HitTarget::Body => {
                if self.kind == BossKind::Eye && !self.open {
                    res.blocked = true;
                    return res;
                }
                self.hp -= dmg;
                self.flash = 0.05;
            }
            HitTarget::Part(i) => {
                let p = &mut self.parts[i];
                p.hp -= dmg;
                p.flash = 0.05;
                if p.hp <= 0.0 {
                    res.part_destroyed = true;
                    res.part_pos = p.pos;
                    let bonus = if p.kind == PartKind::Segment {
                        0.05
                    } else {
                        0.08
                    };
                    self.hp -= self.max_hp * bonus;
                }
            }
        }
        if was_alive && self.hp <= 0.0 {
            self.hp = 0.0;
            res.killed = true;
        }
        res
    }

    pub fn bomb_damage(&mut self, d: f32) {
        if !self.targetable() {
            return;
        }
        self.hp = (self.hp - d).max(0.0);
        self.flash = 0.2;
        for p in &mut self.parts {
            if p.hp > 0.0 {
                p.hp = (p.hp - d * 0.5).max(0.01);
            }
        }
    }

    pub fn touches_player(&self, p: Vec2, r: f32) -> bool {
        if !self.targetable() {
            return false;
        }
        if let Some(b) = &self.beam
            && b.warn <= 0.0
            && b.on > 0.0
            && (p.x - b.x).abs() < 14.0 + r
            && p.y > b.y
        {
            return true;
        }
        if self.beam_warn <= 0.0 && self.beam_on > 0.0 {
            for a in &self.beam_angles {
                let dir = vec2(a.cos(), a.sin());
                let v = p - self.pos;
                let along = v.dot(dir);
                let across = (v.x * dir.y - v.y * dir.x).abs();
                if along > 30.0 && across < 9.0 + r {
                    return true;
                }
            }
        }
        for part in &self.parts {
            if part.hp > 0.0 && part.kind == PartKind::Segment && part.pos.distance(p) < 12.0 + r {
                return true;
            }
        }
        let d = p - self.pos;
        let (rx, ry) = match self.kind {
            BossKind::Queen => (36.0, 28.0),
            BossKind::Warship => (46.0, 22.0),
            BossKind::Serpent => (18.0, 18.0),
            BossKind::Weaver => (36.0, 32.0),
            BossKind::Eye => (36.0, 26.0),
            BossKind::Twins => (30.0, 24.0),
            BossKind::Devourer => (50.0, 44.0),
            BossKind::Comet => (30.0, 26.0),
        };
        (d.x / rx).powi(2) + (d.y / ry).powi(2) <= 1.0
    }

    pub fn draw(&self, sp: &Sprites, art: &Art, time: f32, off: Vec2) {
        let mut pos = self.pos + off;
        if self.dying >= 0.0 {
            pos += vec2(gen_range(-3.0, 3.0), gen_range(-3.0, 3.0));
        }
        let ph = self.phase();
        let rage = if ph == 2 && self.hp > 0.0 {
            (time * 8.0).sin() * 0.5 + 0.5
        } else {
            0.0
        };
        let tint = Color::new(1.0, 1.0 - rage * 0.35, 1.0 - rage * 0.35, 1.0);
        let flash = self.flash > 0.0;
        match self.kind {
            BossKind::Comet => self.draw_comet(pos, off, time, tint, flash),
            BossKind::Queen => {
                let f = ((time * 5.0) as usize) % 2;
                draw_circle(
                    pos.x,
                    pos.y,
                    70.0,
                    Color::new(0.8, 0.2, 0.8, 0.08 + rage * 0.08),
                );
                sp.queen[f].draw(pos, 4.0, 0.0, tint, flash);
                for p in &self.parts {
                    if p.hp > 0.0 {
                        let bob = (time * 4.0 + p.off.x).sin() * 3.0;
                        sp.claw
                            .draw(p.pos + off + vec2(0.0, bob), 3.0, 0.0, tint, p.flash > 0.0);
                    }
                }
                if self.final_boss {
                    let c = Color::new(1.0, 0.85, 0.3, 0.5 + (time * 6.0).sin() * 0.3);
                    draw_circle_lines(pos.x, pos.y - 30.0, 14.0, 2.0, c);
                }
            }
            BossKind::Warship => {
                let t2 = if self.variant {
                    Color::new(tint.r, tint.g * 0.8, tint.b * 0.75, 1.0)
                } else {
                    tint
                };
                sp.warship.draw(pos, 4.0, 0.0, t2, flash);
                for p in &self.parts {
                    let pp = p.pos + off;
                    if p.hp <= 0.0 {
                        draw_circle(pp.x, pp.y, 8.0, Color::from_rgba(30, 20, 20, 255));
                        continue;
                    }
                    match p.kind {
                        PartKind::Turret => {
                            let d = self.aim - pp;
                            let rot = d.y.atan2(d.x) - PI / 2.0;
                            sp.turret.draw(pp, 3.0, rot, WHITE, p.flash > 0.0);
                        }
                        _ => {
                            let charging =
                                self.beam.as_ref().map(|b| b.warn > 0.0).unwrap_or(false);
                            if charging {
                                let r = 10.0 + (time * 30.0).sin().abs() * 6.0;
                                draw_circle(pp.x, pp.y + 10.0, r, Color::new(1.0, 0.3, 0.4, 0.5));
                            }
                            sp.cannon.draw(pp, 3.0, 0.0, WHITE, p.flash > 0.0);
                        }
                    }
                }
            }
            BossKind::Serpent => {
                for (i, p) in self.parts.iter().enumerate().rev() {
                    if p.hp <= 0.0 {
                        continue;
                    }
                    let glow = ((time * 6.0) - i as f32 * 0.5).sin() * 0.5 + 0.5;
                    draw_circle(
                        p.pos.x + off.x,
                        p.pos.y + off.y,
                        16.0,
                        Color::new(1.0, 0.4, 0.1, 0.12 + glow * 0.1),
                    );
                    let tint2 = if self.variant {
                        Color::new(0.8, 0.9, 1.0, 1.0)
                    } else {
                        tint
                    };
                    art.serpent_seg
                        .draw(p.pos + off, 3.0, 0.0, tint2, p.flash > 0.0);
                }
                let next = self.trail.get(6).copied().unwrap_or(self.pos);
                let d = self.pos - next;
                let rot = if d.length() > 0.5 {
                    d.y.atan2(d.x) - PI / 2.0
                } else {
                    0.0
                };
                draw_circle(pos.x, pos.y, 30.0, Color::new(1.0, 0.5, 0.1, 0.15));
                art.serpent_head.draw(pos, 3.5, rot, tint, flash);
            }
            BossKind::Weaver => {
                for p in &self.parts {
                    if p.hp <= 0.0 {
                        continue;
                    }
                    let d = p.pos - self.pos;
                    let rot = d.y.atan2(d.x) - PI / 4.0;
                    draw_line(
                        pos.x,
                        pos.y,
                        p.pos.x + off.x,
                        p.pos.y + off.y,
                        3.0,
                        Color::from_rgba(70, 30, 100, 255),
                    );
                    art.leg.draw(p.pos + off, 3.0, rot, tint, p.flash > 0.0);
                }
                draw_circle(
                    pos.x,
                    pos.y,
                    64.0,
                    Color::new(0.6, 0.3, 0.9, 0.08 + rage * 0.08),
                );
                art.spider.draw(pos, 4.0, 0.0, tint, flash);
            }
            BossKind::Twins => {
                let f = ((time * 6.0) as usize) % 2;
                draw_circle(
                    pos.x,
                    pos.y,
                    52.0,
                    Color::new(0.3, 0.9, 0.8, 0.1 + rage * 0.1),
                );
                sp.queen[f].draw(
                    pos,
                    3.2,
                    0.0,
                    Color::new(0.55 * tint.r, 1.0 * tint.g, 0.9 * tint.b, 1.0),
                    flash,
                );
                for p in &self.parts {
                    if p.hp > 0.0 {
                        let pp = p.pos + off;
                        draw_circle(pp.x, pp.y, 52.0, Color::new(1.0, 0.4, 0.7, 0.1));
                        sp.queen[1 - f].draw(
                            pp,
                            3.2,
                            0.0,
                            Color::new(1.0, 0.6, 0.85, 1.0),
                            p.flash > 0.0,
                        );
                        // laço entre as irmãs
                        let k = (time * 3.0).sin() * 0.5 + 0.5;
                        draw_line(
                            pos.x,
                            pos.y,
                            pp.x,
                            pp.y,
                            1.5,
                            Color::new(1.0, 0.8, 1.0, 0.15 + k * 0.15),
                        );
                    }
                }
            }
            BossKind::Devourer => {
                let pulse = (time * 2.0).sin() * 0.5 + 0.5;
                for k in 0..4 {
                    let r = 120.0 - k as f32 * 18.0 + pulse * 8.0;
                    draw_circle(
                        pos.x,
                        pos.y,
                        r,
                        Color::new(0.4, 0.0, 0.25, 0.06 + k as f32 * 0.03),
                    );
                }
                // tentáculos
                for i in 0..10 {
                    let base = i as f32 / 10.0 * TAU + time * 0.2;
                    let mut prev = pos + vec2(base.cos(), base.sin()) * 56.0;
                    for seg in 1..8 {
                        let r = 56.0 + seg as f32 * 16.0;
                        let wob = (time * 2.0 + seg as f32 * 0.7 + i as f32).sin() * 0.25;
                        let p = pos + vec2((base + wob).cos(), (base + wob).sin()) * r;
                        draw_line(
                            prev.x,
                            prev.y,
                            p.x,
                            p.y,
                            7.0 - seg as f32 * 0.8,
                            Color::new(0.12, 0.02, 0.12, 0.9),
                        );
                        prev = p;
                    }
                }
                draw_circle(pos.x, pos.y, 62.0, Color::new(0.03, 0.0, 0.05, 1.0));
                draw_circle_lines(
                    pos.x,
                    pos.y,
                    62.0,
                    3.0,
                    Color::new(0.7, 0.1, 0.4, 0.6 + pulse * 0.4),
                );
                // boca
                let open = ((time * 1.7).sin() * 0.5 + 0.5) * 10.0
                    + if self.pull > 0.0 { 14.0 } else { 0.0 };
                let my = pos.y + 26.0;
                draw_ellipse(
                    pos.x,
                    my,
                    34.0,
                    6.0 + open,
                    0.0,
                    Color::new(0.35, 0.0, 0.08, 1.0),
                );
                for t in 0..9 {
                    let x = pos.x - 30.0 + t as f32 * 7.5;
                    draw_triangle(
                        vec2(x - 3.0, my - 6.0 - open),
                        vec2(x + 3.0, my - 6.0 - open),
                        vec2(x, my + 2.0 - open * 0.3),
                        WHITE,
                    );
                    draw_triangle(
                        vec2(x - 3.0, my + 6.0 + open),
                        vec2(x + 3.0, my + 6.0 + open),
                        vec2(x, my - 2.0 + open * 0.3),
                        WHITE,
                    );
                }
                // olho central
                let d = (self.aim - self.pos).normalize_or_zero() * 8.0;
                draw_ellipse(
                    pos.x,
                    pos.y - 14.0,
                    26.0,
                    14.0,
                    0.0,
                    Color::new(0.95, 0.9, 0.85, 1.0),
                );
                draw_circle(
                    pos.x + d.x,
                    pos.y - 14.0 + d.y * 0.5,
                    10.0,
                    Color::new(0.8, 0.05, 0.15, 1.0),
                );
                draw_circle(pos.x + d.x, pos.y - 14.0 + d.y * 0.5, 4.0, BLACK);
                for p in &self.parts {
                    if p.hp <= 0.0 {
                        continue;
                    }
                    let pp = p.pos + off;
                    let dd = (self.aim - p.pos).normalize_or_zero() * 3.0;
                    draw_circle(pp.x, pp.y, 13.0, Color::new(0.2, 0.0, 0.1, 1.0));
                    draw_circle(
                        pp.x,
                        pp.y,
                        10.0,
                        if p.flash > 0.0 {
                            RED
                        } else {
                            Color::new(0.95, 0.9, 0.85, 1.0)
                        },
                    );
                    draw_circle(
                        pp.x + dd.x,
                        pp.y + dd.y,
                        4.5,
                        Color::new(0.8, 0.05, 0.15, 1.0),
                    );
                }
                if flash {
                    draw_circle(pos.x, pos.y, 62.0, Color::new(1.0, 1.0, 1.0, 0.35));
                }
            }
            BossKind::Eye => {
                let glow = if self.open { 0.12 } else { 0.05 };
                let c = if self.variant {
                    Color::new(1.0, 0.7, 0.2, glow)
                } else {
                    Color::new(0.4, 0.8, 1.0, glow)
                };
                draw_circle(pos.x, pos.y, 74.0, c);
                if self.open {
                    art.eye_open.draw(pos, 4.0, 0.0, tint, flash);
                    let d = (self.aim - self.pos).normalize_or_zero() * 10.0;
                    draw_circle(
                        pos.x + d.x,
                        pos.y + d.y,
                        11.0,
                        Color::from_rgba(20, 10, 40, 255),
                    );
                    let iris = if self.variant {
                        Color::from_rgba(255, 170, 40, 255)
                    } else {
                        Color::from_rgba(255, 50, 90, 255)
                    };
                    draw_circle(pos.x + d.x, pos.y + d.y, 5.0, iris);
                } else {
                    art.eye_closed.draw(pos, 4.0, 0.0, tint, flash);
                }
            }
        }
    }

    fn draw_comet(&self, pos: Vec2, off: Vec2, time: f32, tint: Color, flash: bool) {
        // cauda de poeira
        let n = self.trail.len();
        for (i, p) in self.trail.iter().enumerate().rev().step_by(2) {
            let k = i as f32 / n.max(1) as f32;
            let q = *p + off + vec2(0.0, -k * 40.0);
            let r = 34.0 * (1.0 - k) + 6.0;
            let wob = (time * 3.0 + i as f32 * 0.3).sin() * 4.0 * k;
            draw_circle(
                q.x + wob,
                q.y,
                r,
                Color::new(0.5, 0.85, 1.0, 0.05 + 0.08 * (1.0 - k)),
            );
            if i % 6 == 0 {
                draw_circle(
                    q.x - wob * 2.0,
                    q.y + 4.0,
                    2.0,
                    Color::new(1.0, 1.0, 1.0, 0.6 * (1.0 - k)),
                );
            }
        }
        // coma
        let pulse = (time * 2.5).sin() * 4.0;
        draw_circle(pos.x, pos.y, 62.0 + pulse, Color::new(0.4, 0.8, 1.0, 0.08));
        draw_circle(
            pos.x,
            pos.y,
            46.0 + pulse * 0.5,
            Color::new(0.6, 0.9, 1.0, 0.12),
        );
        // núcleo
        let base = if flash {
            WHITE
        } else {
            Color::new(0.42 * tint.r, 0.5 * tint.g, 0.62 * tint.b, 1.0)
        };
        draw_poly(pos.x, pos.y, 9, 32.0, time * 12.0, base);
        draw_poly(
            pos.x - 4.0,
            pos.y - 4.0,
            9,
            24.0,
            time * 12.0,
            Color::new(0.62, 0.72, 0.85, 1.0),
        );
        for (dx, dy, r) in [
            (-10.0, -6.0, 6.0),
            (9.0, 5.0, 5.0),
            (-3.0, 12.0, 4.0),
            (12.0, -11.0, 3.0),
        ] {
            draw_circle(pos.x + dx, pos.y + dy, r, Color::new(0.35, 0.42, 0.55, 1.0));
        }
        draw_poly(
            pos.x + 6.0,
            pos.y - 12.0,
            4,
            6.0,
            45.0,
            Color::new(0.9, 0.97, 1.0, 0.9),
        );
        if self.variant || self.phase() >= 2 {
            draw_poly_lines(
                pos.x,
                pos.y,
                9,
                34.0,
                time * 12.0,
                2.0,
                Color::new(1.0, 0.5, 0.4, 0.8),
            );
        }
        // fragmentos
        for p in &self.parts {
            if p.hp <= 0.0 {
                continue;
            }
            let q = p.pos + off;
            let c = if p.flash > 0.0 {
                WHITE
            } else {
                Color::from_rgba(190, 235, 255, 255)
            };
            draw_circle(q.x, q.y, 15.0, Color::new(0.6, 0.9, 1.0, 0.15));
            draw_poly(q.x, q.y, 4, 11.0, time * 90.0 + p.cd * 40.0, c);
            draw_poly_lines(
                q.x,
                q.y,
                4,
                11.0,
                time * 90.0 + p.cd * 40.0,
                1.5,
                Color::from_rgba(80, 140, 200, 255),
            );
        }
    }

    pub fn draw_beam(&self, time: f32) {
        if let Some(b) = &self.beam {
            if b.warn > 0.0 {
                if (time * 20.0) as i32 % 2 == 0 {
                    draw_line(b.x, b.y, b.x, H, 2.0, Color::new(1.0, 0.2, 0.3, 0.8));
                }
                draw_rectangle(
                    b.x - 16.0,
                    b.y,
                    32.0,
                    H - b.y,
                    Color::new(1.0, 0.2, 0.3, 0.08),
                );
            } else {
                let flick = 0.85 + (time * 50.0).sin() * 0.15;
                draw_rectangle(
                    b.x - 22.0,
                    b.y,
                    44.0,
                    H - b.y,
                    Color::new(1.0, 0.25, 0.4, 0.25 * flick),
                );
                draw_rectangle(
                    b.x - 14.0,
                    b.y,
                    28.0,
                    H - b.y,
                    Color::new(1.0, 0.4, 0.55, 0.7 * flick),
                );
                draw_rectangle(
                    b.x - 6.0,
                    b.y,
                    12.0,
                    H - b.y,
                    Color::new(1.0, 0.95, 0.95, flick),
                );
            }
        }
        if !self.beam_angles.is_empty() {
            for a in &self.beam_angles {
                let dir = vec2(a.cos(), a.sin());
                let from = self.pos + dir * 30.0;
                let to = self.pos + dir * 900.0;
                if self.beam_warn > 0.0 {
                    if (time * 16.0) as i32 % 2 == 0 {
                        draw_line(
                            from.x,
                            from.y,
                            to.x,
                            to.y,
                            2.0,
                            Color::new(0.6, 0.9, 1.0, 0.8),
                        );
                    }
                } else {
                    let flick = 0.8 + (time * 40.0).sin() * 0.2;
                    draw_line(
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        22.0,
                        Color::new(0.4, 0.8, 1.0, 0.3 * flick),
                    );
                    draw_line(
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        12.0,
                        Color::new(0.6, 0.9, 1.0, 0.8 * flick),
                    );
                    draw_line(
                        from.x,
                        from.y,
                        to.x,
                        to.y,
                        4.0,
                        Color::new(1.0, 1.0, 1.0, flick),
                    );
                }
            }
        }
        if let Some((target, t)) = &self.gaze {
            let dir = (*target - self.pos).normalize_or_zero();
            let to = self.pos + dir * 900.0;
            let a = (1.0 - t / 0.9).clamp(0.2, 1.0);
            if (time * 18.0) as i32 % 2 == 0 {
                draw_line(
                    self.pos.x,
                    self.pos.y,
                    to.x,
                    to.y,
                    1.5,
                    Color::new(1.0, 0.3, 0.5, a),
                );
            }
        }
    }
}
