//! Level designs. Every normal level of the campaign has its own formation shape, its own
//! way of moving, its own entry paths, a featured enemy, a diving style and, later on,
//! synchronized salvos. Designs are built from the level number, so level 37 is always the
//! same level 37 and no two levels of the campaign are alike.

use super::*;
use std::f32::consts::{PI, TAU};

pub const SHAPES: [&str; 16] = [
    "BLOCK",
    "CHEVRON",
    "ARROWHEAD",
    "DIAMOND",
    "RING",
    "TWIN BLOCKS",
    "CROSS",
    "WAVE",
    "CHECKERBOARD",
    "PYRAMID",
    "COLUMNS",
    "HOURGLASS",
    "SPIRAL",
    "WINGS",
    "CRESCENT",
    "STAR",
];

pub const MOTIONS: [&str; 7] = [
    "SWAYING", "DRIFTING", "BOBBING", "TWISTING", "ORBITING", "PULSING", "MARCHING",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Dive {
    #[default]
    Solo,
    Pairs,
    Squads,
    Swarm,
    Rush,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Design {
    pub shape: usize,
    pub motion: usize,
    /// First entry template and the step between groups.
    entry_base: u32,
    entry_step: u32,
    /// 0 split halves, 1 alternate sides per group, 2 all from the left, 3 all from the right.
    sides: u8,
    /// Seconds between groups and flight time of the entry.
    gap: f32,
    flight: f32,
    pub feature: Option<Kind>,
    feature_group: usize,
    pub dive: Dive,
    pub salvo: bool,
}

#[derive(Default)]
pub struct LevelState {
    pub design: Design,
    /// Formation slots (offsets from the formation center), indexed by row * COLS + col.
    pub form: Vec<Vec2>,
    salvo_t: f32,
    salvo_warned: bool,
    salvo_group: u32,
    dive_count: u32,
    /// Boss fight twist of the current sector and its timer.
    boss_trait: u8,
    boss_t: f32,
}

/// Enemies that can be featured, by the tier where they first appear.
const FEATURES: [(Kind, u32); 13] = [
    (Kind::Wasp, 1),
    (Kind::Beetle, 1),
    (Kind::Gunner, 2),
    (Kind::Splitter, 3),
    (Kind::Jumper, 3),
    (Kind::Kamikaze, 4),
    (Kind::Shieldbearer, 4),
    (Kind::Weaver, 5),
    (Kind::Minelayer, 6),
    (Kind::Armored, 6),
    (Kind::Chameleon, 7),
    (Kind::Healer, 8),
    (Kind::Mirror, 9),
];

/// The design of normal level number `n` (0 = the very first one).
pub fn design(n: u32, tier: u32) -> Design {
    if n == 0 {
        // The first level stays a gentle, classic block.
        return Design {
            shape: 0,
            motion: 0,
            entry_base: 0,
            entry_step: 1,
            sides: 0,
            gap: 1.9,
            flight: 2.3,
            feature: None,
            feature_group: 0,
            dive: Dive::Solo,
            salvo: false,
        };
    }
    // a = shape, b = how many times the shapes have cycled: (a, b) is unique for n < 112,
    // and so is the pair (shape, motion) below.
    let a = n % 16;
    let b = n / 16;
    let motion = ((a + b * 2) % 7) as usize;
    let pool: Vec<Kind> = FEATURES
        .iter()
        .filter(|(_, t)| *t <= tier.max(1))
        .map(|(k, _)| *k)
        .collect();
    let feature = if n % 4 == 3 {
        None
    } else {
        Some(pool[((n * 7 + b) as usize) % pool.len()])
    };
    Design {
        shape: a as usize,
        motion,
        entry_base: (n * 5 + b) % 8,
        entry_step: 1 + (n % 3),
        sides: ((n / 2 + b) % 4) as u8,
        gap: 1.5 + ((n * 3) % 5) as f32 * 0.15,
        flight: 2.0 + ((n * 7) % 4) as f32 * 0.2,
        feature,
        feature_group: ((n + b) % 5) as usize,
        dive: [
            Dive::Solo,
            Dive::Pairs,
            Dive::Squads,
            Dive::Swarm,
            Dive::Rush,
        ][((a + b * 3) % 5) as usize],
        salvo: tier >= 3 && n % 3 != 0,
    }
}

/// Formation slot offsets of a shape, in pixels from the formation center.
pub fn shape_points(shape: usize, rows: usize) -> Vec<Vec2> {
    let gx = 44.0;
    let gy = 38.0;
    let grid = |skip_corners: bool| -> Vec<Vec2> {
        let mut v = vec![];
        for r in 0..rows {
            for c in 0..8 {
                if skip_corners && r == 0 && !(2..6).contains(&c) {
                    continue;
                }
                v.push(vec2((c as f32 - 3.5) * gx, (r as f32 - 2.0) * gy));
            }
        }
        v
    };
    let rows_of = |widths: &[usize], dx: f32, dy: f32| -> Vec<Vec2> {
        let mut v = vec![];
        let h = widths.len() as f32;
        for (r, &w) in widths.iter().enumerate() {
            for c in 0..w {
                v.push(vec2(
                    (c as f32 - (w as f32 - 1.0) / 2.0) * dx,
                    (r as f32 - (h - 1.0) / 2.0) * dy,
                ));
            }
        }
        v
    };
    let n = rows * 8;
    let mut pts = match shape {
        0 => grid(true),
        1 => grid(false)
            .into_iter()
            .map(|p| p + vec2(0.0, -(p.x.abs() / gx) * 13.0 + 20.0))
            .collect(),
        2 => grid(false)
            .into_iter()
            .map(|p| p + vec2(0.0, (p.x.abs() / gx) * 13.0 - 20.0))
            .collect(),
        3 => rows_of(&[2, 4, 6, 8, 8, 6, 4, 2], 42.0, 24.0),
        4 => {
            let mut v = vec![];
            for i in 0..24 {
                let a = i as f32 / 24.0 * TAU;
                v.push(vec2(a.cos() * 175.0, a.sin() * 105.0));
            }
            for i in 0..16 {
                let a = i as f32 / 16.0 * TAU + 0.2;
                v.push(vec2(a.cos() * 90.0, a.sin() * 52.0));
            }
            v
        }
        5 => {
            let mut v = vec![];
            for r in 0..5 {
                for c in 0..8 {
                    let side = if c < 4 { -1.0 } else { 1.0 };
                    let cc = (c % 4) as f32;
                    v.push(vec2(side * (70.0 + cc * 38.0), (r as f32 - 2.0) * 34.0));
                }
            }
            v
        }
        6 => {
            let mut v = vec![];
            for i in 0..20 {
                let t = i as f32 / 19.0 * 2.0 - 1.0;
                v.push(vec2(t * 185.0, t * 115.0));
                // the second line skips the crossing point
                if !(9..=10).contains(&i) {
                    v.push(vec2(t * 185.0, -t * 115.0));
                }
            }
            v.push(vec2(0.0, -48.0));
            v.push(vec2(0.0, 48.0));
            v
        }
        7 => grid(false)
            .into_iter()
            .map(|p| p + vec2(0.0, ((p.x / gx) * 0.9 + p.y / gy).sin() * 22.0))
            .collect(),
        8 => {
            let mut v = vec![];
            for r in 0..10 {
                for c in 0..8 {
                    if (r + c) % 2 == 0 {
                        v.push(vec2((c as f32 - 3.5) * 46.0, (r as f32 - 4.5) * 24.0));
                    }
                }
            }
            v
        }
        9 => rows_of(&[2, 4, 6, 8, 10, 10], 40.0, 34.0),
        10 => {
            let mut v = vec![];
            for r in 0..8 {
                for c in 0..5 {
                    v.push(vec2((c as f32 - 2.0) * 88.0, (r as f32 - 3.5) * 27.0));
                }
            }
            v
        }
        11 => rows_of(&[8, 6, 4, 2, 2, 4, 6, 8], 42.0, 26.0),
        12 => {
            // an even spiral: the same distance between neighbours all the way out
            let mut v = vec![];
            let mut th = 0.0f32;
            while v.len() < 40 {
                let r = 26.0 + 10.5 * th;
                v.push(vec2(th.cos() * r * 1.2, th.sin() * r * 0.72));
                th += 30.0 / r;
            }
            v
        }
        13 => {
            let mut v = vec![];
            for side in [-1.0f32, 1.0] {
                for r in 0..4 {
                    for c in 0..(5 - r) {
                        v.push(vec2(
                            side * (60.0 + c as f32 * 34.0 + r as f32 * 17.0),
                            (r as f32 - 1.5) * 34.0 - c as f32 * 9.0,
                        ));
                    }
                }
                v.push(vec2(side * 40.0, 60.0));
                v.push(vec2(side * 40.0, -60.0));
                v.push(vec2(side * 20.0, 90.0));
                v.push(vec2(side * 20.0, -90.0));
                v.push(vec2(side * 210.0, 70.0));
                v.push(vec2(side * 210.0, -70.0));
            }
            v
        }
        14 => {
            let mut v = vec![];
            for band in 0..3 {
                let r = 200.0 - band as f32 * 36.0;
                let k = [16, 13, 11][band];
                for i in 0..k {
                    let a = (200.0 + 140.0 * i as f32 / (k - 1) as f32).to_radians();
                    v.push(vec2(a.cos() * r, a.sin() * r * 0.75 + 150.0));
                }
            }
            v
        }
        _ => {
            // a five-pointed star outline
            let mut corners = vec![];
            for i in 0..10 {
                let a = -PI / 2.0 + i as f32 * PI / 5.0;
                let r = if i % 2 == 0 { 140.0 } else { 58.0 };
                corners.push(vec2(a.cos() * r * 1.2, a.sin() * r * 0.85 + 10.0));
            }
            let mut v = vec![];
            for i in 0..10 {
                let p = corners[i];
                let q = corners[(i + 1) % 10];
                v.push(p);
                v.push(p.lerp(q, 0.5));
            }
            // a small ring in the heart of the star
            for i in 0..8 {
                let a = i as f32 / 8.0 * TAU + 0.3;
                v.push(vec2(a.cos() * 40.0, a.sin() * 30.0 + 10.0));
            }
            v
        }
    };
    pts.truncate(n);
    pts
}

/// Entry flight paths: four classics plus four newer ones.
pub fn entry_path(template: u32, s: f32) -> Path {
    let mx = |x: f32| if s < 0.0 { x } else { W - x };
    let v = |x: f32, y: f32| vec2(mx(x), y);
    match template % 8 {
        0 => Path::one(
            v(-40.0, H * 0.62),
            v(W * 0.8, H * 0.86),
            v(W * 0.15, H * 0.12),
            Vec2::ZERO,
        ),
        1 => Path::one(v(180.0, -40.0), v(10.0, 360.0), v(420.0, 273.0), Vec2::ZERO),
        2 => Path::two(
            [
                v(-30.0, 60.0),
                v(W * 0.55, 40.0),
                v(W * 0.75, H * 0.55),
                v(W * 0.45, H * 0.55),
            ],
            [
                v(W * 0.45, H * 0.55),
                v(W * 0.15, H * 0.55),
                v(W * 0.2, H * 0.25),
                Vec2::ZERO,
            ],
        ),
        3 => Path::one(
            v(W * 0.05, -40.0),
            v(W * 1.1, H * 0.3),
            v(-W * 0.1, H * 0.45),
            Vec2::ZERO,
        ),
        4 => Path::two(
            // a loop-the-loop from the top center
            [
                v(W * 0.5, -40.0),
                v(W * 0.5, H * 0.45),
                v(W * 0.05, H * 0.45),
                v(W * 0.15, H * 0.25),
            ],
            [
                v(W * 0.15, H * 0.25),
                v(W * 0.25, H * 0.05),
                v(W * 0.6, H * 0.15),
                Vec2::ZERO,
            ],
        ),
        5 => Path::one(
            // a low sweep from the side, rising into place
            v(-40.0, H * 0.72),
            v(W * 0.5, H * 0.7),
            v(W * 0.9, H * 0.1),
            Vec2::ZERO,
        ),
        6 => Path::two(
            // a zigzag dive from the top
            [
                v(W * 0.3, -40.0),
                v(W * 0.9, H * 0.15),
                v(W * 0.1, H * 0.3),
                v(W * 0.7, H * 0.42),
            ],
            [
                v(W * 0.7, H * 0.42),
                v(W * 0.9, H * 0.5),
                v(W * 0.4, H * 0.5),
                Vec2::ZERO,
            ],
        ),
        _ => Path::two(
            // a figure eight
            [
                v(-40.0, H * 0.15),
                v(W * 0.8, H * 0.15),
                v(W * 0.8, H * 0.5),
                v(W * 0.5, H * 0.35),
            ],
            [
                v(W * 0.5, H * 0.35),
                v(W * 0.2, H * 0.2),
                v(W * 0.2, H * 0.55),
                Vec2::ZERO,
            ],
        ),
    }
}

impl Game {
    /// Every boss fight of the campaign gets a twist, so the twenty fights all differ:
    /// 0 none, 1 kamikaze escorts, 2 extra barrages, 3 the stage changes right away.
    pub(super) fn boss_trait_begin(&mut self) {
        let s = self.sector() + self.cycle * 5;
        self.lv.boss_trait = if s <= 1 {
            0
        } else {
            ((s * 3 + s / 4) % 4) as u8
        };
        self.lv.boss_t = 4.0;
        if self.lv.boss_trait == 3 && !self.enc.void {
            self.debug_scene(((s / 2) % 3) as usize);
        }
    }

    pub(super) fn update_boss_trait(&mut self, dt: f32) {
        let Some(b) = &self.boss else { return };
        if !b.targetable() || !self.player.alive() {
            return;
        }
        let bpos = b.pos;
        self.lv.boss_t -= dt;
        if self.lv.boss_t > 0.0 {
            return;
        }
        match self.lv.boss_trait {
            1 => {
                self.lv.boss_t = (7.0 - self.diff() * 0.3).max(3.5);
                if self.enemies.len() < 8 {
                    for side in [-1.0f32, 1.0] {
                        self.spawn(
                            Kind::Kamikaze,
                            0,
                            0,
                            EState::Free {
                                vel: vec2(side * 110.0, 140.0),
                            },
                            99,
                            false,
                        );
                        if let Some(e) = self.enemies.last_mut() {
                            e.pos = bpos + vec2(side * 50.0, 20.0);
                        }
                    }
                }
            }
            2 => {
                self.lv.boss_t = (5.5 - self.diff() * 0.2).max(2.5);
                let speed = (170.0 + self.diff() * 10.0) * self.bullet_mul();
                let aim = (self.player.pos - bpos).normalize_or_zero();
                for k in -3..=3 {
                    let a = k as f32 * 0.16;
                    let d = vec2(
                        aim.x * a.cos() - aim.y * a.sin(),
                        aim.x * a.sin() + aim.y * a.cos(),
                    );
                    self.ebullets.push(EBullet::new(bpos, d * speed, 5.0, 4));
                }
            }
            _ => self.lv.boss_t = 99.0,
        }
    }

    /// Number of the current normal level (0-based), used to pick its design.
    fn normal_index(&self) -> u32 {
        match self.cfg.mode {
            Mode::Campaign | Mode::Daily | Mode::Expedition => {
                let sector = self.sector() - 1;
                let sub = (self.sub() - 1).min(2);
                let n = sector * 3 + sub;
                // Later cycles and the endless Expedition reshuffle the designs.
                n + self.cycle * 17
            }
            _ => self.wave.saturating_sub(1),
        }
    }

    /// Picks the design and the formation of the coming normal level.
    pub(super) fn level_begin(&mut self) {
        let n = self.normal_index();
        let d = design(n % 112, self.stage().tier);
        let first = self.wave == 1 && self.cycle == 0;
        self.lv.form = shape_points(d.shape, if first { 4 } else { 5 });
        self.lv.design = d;
        self.lv.salvo_t = 6.0;
        self.lv.salvo_warned = false;
        self.lv.dive_count = 0;
        if n > 0 && !self.classic() {
            let title = match self.cfg.mode {
                Mode::Campaign if self.cycle == 0 => format!("LEVEL {}", self.level()),
                _ => format!("WAVE {}", self.wave),
            };
            self.banner = Some((
                title,
                format!("{} · {}", SHAPES[d.shape], MOTIONS[d.motion]),
                2.2,
            ));
        }
    }

    /// Where the formation slot (col, row) is right now.
    pub(super) fn level_slot(&self, col: usize, row: usize) -> Option<Vec2> {
        let p = *self.lv.form.get(row * COLS + col)?;
        let t = self.time;
        let calm = self.stage().tier >= 2;
        let mut center = vec2(W / 2.0, 176.0);
        let mut off = p;
        match self.lv.design.motion {
            0 => center.x += (t * 0.6).sin() * 30.0,
            1 => center.x += (t * 0.35).sin() * 60.0,
            2 => {
                center.x += (t * 0.5).sin() * 18.0;
                center.y += (t * 1.1).sin() * 16.0;
            }
            3 => {
                let a = (t * 0.45).sin() * 0.28;
                off = vec2(
                    off.x * a.cos() - off.y * a.sin(),
                    off.x * a.sin() + off.y * a.cos(),
                );
            }
            4 => center += vec2((t * 0.7).cos() * 32.0, (t * 0.7).sin() * 18.0),
            5 => off *= 1.0 + 0.12 * (t * 1.6).sin(),
            _ => {
                // marching: steps sideways like an old arcade invasion
                let phase = (t * 0.25).rem_euclid(4.0);
                let x = if phase < 2.0 {
                    phase - 1.0
                } else {
                    3.0 - phase
                };
                center.x += (x * 4.0).round() / 4.0 * 55.0;
            }
        }
        if calm && self.lv.design.motion != 5 {
            off *= 1.0 + 0.05 * (t * 1.4).sin();
        }
        // Keep the whole formation on screen.
        let pos = center + off;
        Some(vec2(
            pos.x.clamp(18.0, W - 18.0),
            pos.y.clamp(40.0, H * 0.55),
        ))
    }

    /// Builds the enemies of a normal level from its design.
    pub(super) fn build_level(&mut self) {
        let d = self.lv.design;
        let fast = self.cfg.mode == Mode::Survival;
        let n = self.lv.form.len();
        let groups = n.div_ceil(COLS);
        // Groups enter in an order that changes with the design.
        let order: Vec<usize> = match d.entry_step % 3 {
            0 => (0..groups).collect(),
            1 => (0..groups).rev().collect(),
            _ => (0..groups).map(|g| (g * 3 + 1) % groups).collect(),
        };
        for (gi, &g) in order.iter().enumerate() {
            let template = d.entry_base + gi as u32 * d.entry_step;
            let ids: Vec<usize> = (g * COLS..((g + 1) * COLS).min(n)).collect();
            let half = ids.len() / 2;
            for (ci, &idx) in ids.iter().enumerate() {
                let col = idx % COLS;
                let row = idx / COLS;
                let left = ci < half;
                let s = match d.sides {
                    0 => {
                        if left {
                            -1.0
                        } else {
                            1.0
                        }
                    }
                    1 => {
                        if gi % 2 == 0 {
                            -1.0
                        } else {
                            1.0
                        }
                    }
                    2 => -1.0,
                    _ => 1.0,
                };
                let k = if d.sides == 0 {
                    if left { half - 1 - ci } else { ci - half }
                } else {
                    ci
                };
                let gap = if fast { 1.1 } else { d.gap };
                let delay = 0.6 + gi as f32 * gap + k as f32 * 0.15;
                let kind = match d.feature {
                    Some(f)
                        if g == d.feature_group % groups.max(1)
                            && self.vanguard_kind(row, col).is_none() =>
                    {
                        f
                    }
                    _ => self.row_kind(row, col),
                };
                self.spawn(
                    kind,
                    col,
                    row,
                    EState::Waiting {
                        delay,
                        path: entry_path(template, s),
                        dur: d.flight,
                    },
                    gi as u32,
                    false,
                );
            }
        }
    }

    /// How many enemies join each dive, by the level's diving style.
    pub(super) fn dive_partners(&mut self, first: usize) -> Vec<usize> {
        self.lv.dive_count += 1;
        let style = self.lv.design.dive;
        let extra = match style {
            Dive::Solo | Dive::Swarm => 0,
            Dive::Pairs => 1,
            Dive::Squads => 2,
            Dive::Rush => {
                if self.lv.dive_count.is_multiple_of(5) {
                    7
                } else {
                    0
                }
            }
        };
        let (row, col) = (self.enemies[first].row, self.enemies[first].col);
        let mut near: Vec<(usize, usize)> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(i, e)| {
                *i != first
                    && matches!(e.state, EState::Formation)
                    && !e.dead
                    && e.kind != Kind::Carrier
                    && (style == Dive::Rush || e.row == row)
            })
            .map(|(i, e)| {
                (
                    i,
                    (e.col as i32 - col as i32).unsigned_abs() as usize
                        + (e.row as i32 - row as i32).unsigned_abs() as usize * 3,
                )
            })
            .collect();
        near.sort_by_key(|(_, d)| *d);
        near.into_iter().take(extra).map(|(i, _)| i).collect()
    }

    /// Dive timer multiplier for the level's style (Swarm dives much more often).
    pub(super) fn dive_rate(&self) -> f32 {
        match self.lv.design.dive {
            Dive::Swarm => 1.6,
            Dive::Squads => 0.8,
            Dive::Rush => 0.9,
            _ => 1.0,
        }
    }

    /// Synchronized salvos: every few seconds a whole group flashes, then fires at you.
    pub(super) fn update_salvo(&mut self, dt: f32) {
        if !self.lv.design.salvo || self.wave_kind != WaveKind::Normal || self.wave_age < 4.0 {
            return;
        }
        if !self.player.alive() {
            return;
        }
        self.lv.salvo_t -= dt;
        if self.lv.salvo_t < 0.7 && !self.lv.salvo_warned {
            self.lv.salvo_warned = true;
            let groups: Vec<u32> = self
                .enemies
                .iter()
                .filter(|e| matches!(e.state, EState::Formation) && !e.harmless)
                .map(|e| e.group)
                .collect();
            if groups.is_empty() {
                self.lv.salvo_t = 5.0;
                self.lv.salvo_warned = false;
                return;
            }
            self.lv.salvo_group = groups[gen_range(0, groups.len())];
            for e in &self.enemies {
                if e.group == self.lv.salvo_group && matches!(e.state, EState::Formation) {
                    self.fx
                        .ring(e.pos, 60.0, 0.6, Color::new(1.0, 0.4, 0.3, 0.9), 2.0);
                }
            }
        }
        if self.lv.salvo_t <= 0.0 {
            let target = self.player.pos;
            let speed = (190.0 + self.diff() * 12.0) * self.bullet_mul();
            let g = self.lv.salvo_group;
            let shots: Vec<Vec2> = self
                .enemies
                .iter()
                .filter(|e| e.group == g && matches!(e.state, EState::Formation))
                .map(|e| e.pos)
                .collect();
            for p in shots {
                let v = (target - p).normalize_or_zero() * speed;
                self.ebullets
                    .push(EBullet::new(p + vec2(0.0, 10.0), v, 4.0, 2));
            }
            self.lv.salvo_t = (7.0 - self.diff() * 0.25).max(3.5);
            self.lv.salvo_warned = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_campaign_level_is_unique() {
        let mut seen = std::collections::HashSet::new();
        for n in 0..60 {
            let d = design(n, 9);
            assert!(seen.insert((d.shape, d.motion)), "level design {n} repeats");
        }
    }

    #[test]
    fn formations_fit_and_do_not_overlap() {
        for shape in 0..SHAPES.len() {
            for rows in [4, 5] {
                let pts = shape_points(shape, rows);
                assert!(
                    pts.len() >= 28,
                    "shape {shape} has only {} slots",
                    pts.len()
                );
                for (i, a) in pts.iter().enumerate() {
                    assert!(a.x.abs() <= 225.0, "shape {shape} slot {i} too wide: {a}");
                    assert!(
                        a.y > -140.0 && a.y < 175.0,
                        "shape {shape} slot {i} too tall: {a}"
                    );
                    for b in &pts[i + 1..] {
                        assert!(
                            a.distance(*b) > 17.0,
                            "shape {shape}: slots too close {a} {b}"
                        );
                    }
                }
            }
        }
    }
}
