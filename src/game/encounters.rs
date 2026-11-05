//! Encounter systems: extra weapons, Fever, contracts, sector events, Mimics,
//! Thieves, elites, freighters, flight logs, the Void and the calf.

use super::*;
use crate::progress::LOGS;
use std::f32::consts::TAU;

// ---------------------------------------------------------------------------
// Sector events
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SEvent {
    Eclipse,
    Meteors,
    GoldRush,
    Ambush,
    Gravity,
    Convoy,
}

pub const SEVENTS: [SEvent; 6] = [
    SEvent::Eclipse,
    SEvent::Meteors,
    SEvent::GoldRush,
    SEvent::Ambush,
    SEvent::Gravity,
    SEvent::Convoy,
];

impl SEvent {
    pub fn name(self) -> &'static str {
        match self {
            SEvent::Eclipse => "ECLIPSE",
            SEvent::Meteors => "METEOR SHOWER",
            SEvent::GoldRush => "GOLD RUSH",
            SEvent::Ambush => "AMBUSH",
            SEvent::Gravity => "GRAVITY TIDE",
            SEvent::Convoy => "CARGO CONVOY",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            SEvent::Eclipse => "The light fades. You can only see near your ship.",
            SEvent::Meteors => "Rocks rain down all over the sector.",
            SEvent::GoldRush => "Crystals are worth double this sector.",
            SEvent::Ambush => "Part of the Swarm comes from below!",
            SEvent::Gravity => "Enemy shots curve toward you.",
            SEvent::Convoy => "Freighters full of crystals cross the screen.",
        }
    }
    pub fn index(self) -> usize {
        SEVENTS.iter().position(|e| *e == self).unwrap_or(0)
    }
}

// ---------------------------------------------------------------------------
// Contracts
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CKind {
    Kill(Kind),
    Combo,
    Grazes,
    Capsules,
    Crystals,
    Dives,
    Parts,
    NoBomb,
    Elites,
    Frozen,
    Fever,
}

pub struct Contract {
    pub kind: CKind,
    pub target: u32,
    pub progress: u32,
    pub done: bool,
    pub xp: u32,
}

impl Contract {
    pub fn text(&self) -> String {
        match self.kind {
            CKind::Kill(k) => format!(
                "Shoot down {} {}",
                self.target,
                crate::progress::BESTIARY[crate::progress::lore_of_kind(k.index())].name
            ),
            CKind::Combo => format!("Make a {} combo", self.target),
            CKind::Grazes => format!("Make {} grazes", self.target),
            CKind::Capsules => format!("Grab {} capsules", self.target),
            CKind::Crystals => format!("Collect {} crystals", self.target),
            CKind::Dives => format!("Shoot down {} while diving", self.target),
            CKind::Parts => format!("Destroy {} boss parts", self.target),
            CKind::NoBomb => "Finish a sector without bombs".to_string(),
            CKind::Elites => format!("Shoot down {} elites", self.target),
            CKind::Frozen => format!("Freeze {} enemies", self.target),
            CKind::Fever => "Enter Fever".to_string(),
        }
    }
}

fn roll_contracts(mode: Mode, rank: usize, achs: &[bool]) -> Vec<Contract> {
    let classic = mode == Mode::Classic;
    let mut pool: Vec<(CKind, u32, u32)> = vec![
        (CKind::Kill(Kind::Wasp), 30, 300),
        (CKind::Kill(Kind::Beetle), 12, 350),
        (CKind::Kill(Kind::Drone), 40, 300),
        (CKind::Combo, 40, 400),
        (CKind::Dives, 20, 350),
        (CKind::NoBomb, 1, 400),
    ];
    if !classic {
        pool.extend([
            (CKind::Grazes, 60, 350),
            (CKind::Capsules, 6, 300),
            (CKind::Crystals, 150, 300),
            (CKind::Fever, 1, 500),
        ]);
        if matches!(
            mode,
            Mode::Campaign | Mode::Daily | Mode::BossRush | Mode::Expedition
        ) {
            pool.push((CKind::Parts, 4, 450));
        }
        if matches!(mode, Mode::Campaign | Mode::Survival | Mode::Expedition) {
            pool.push((CKind::Elites, 4, 450));
        }
        if Weapon::Cryo.unlocked(rank, achs) {
            pool.push((CKind::Frozen, 25, 350));
        }
    }
    if mode == Mode::BossRush {
        pool.retain(|(k, _, _)| {
            matches!(
                k,
                CKind::Parts
                    | CKind::Grazes
                    | CKind::Capsules
                    | CKind::Crystals
                    | CKind::Fever
                    | CKind::NoBomb
            )
        });
    }
    let mut out = vec![];
    while out.len() < 3 && !pool.is_empty() {
        let (kind, target, xp) = pool.swap_remove(gen_range(0, pool.len()));
        out.push(Contract {
            kind,
            target,
            progress: 0,
            done: false,
            xp,
        });
    }
    out
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Loot {
    Capsule,
    Card(usize),
    Power(PowerKind),
}

#[derive(Default)]
pub struct Encounters {
    pub fever_t: f32,
    fever_lock: bool,
    pub fevers: u32,
    pub contracts: Vec<Contract>,
    pub contract_xp: u32,
    pub contracts_done: u32,
    pub event: Option<SEvent>,
    event_timer: f32,
    pub events_seen: Vec<usize>,
    pub capsules: u32,
    pub crystals: u32,
    pub parts: u32,
    pub dives: u32,
    pub elites: u32,
    pub bombs_sector: u32,
    pub void: bool,
    pub void_done: bool,
    thief_timer: f32,
    loot: Vec<(u32, Loot)>,
    pub mimics: u32,
    pub cows: u32,
    pub nohit_t: f32,
    calf: Vec2,
    calf_moo: f32,
    pub logs_found: Vec<usize>,
    pub saws: Vec<Vec2>,
}

impl Encounters {
    pub fn new(cfg: &RunConfig) -> Encounters {
        Encounters {
            contracts: roll_contracts(cfg.mode, cfg.rank, &cfg.achs),
            thief_timer: 3.0,
            calf_moo: 15.0,
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Extra weapons
// ---------------------------------------------------------------------------

/// Fires the extra weapons and returns the delay until the next shot.
#[allow(clippy::too_many_arguments)]
pub(super) fn fire_extra(
    w: Weapon,
    lv: i32,
    dmg: f32,
    o: Vec2,
    ss: f32,
    hole_cd: &mut f32,
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
        Weapon::Ricochet => {
            let n = [2, 2, 3, 4, 5][lvi];
            for k in 0..n {
                let a = if n == 1 {
                    0.0
                } else {
                    -32.0 + 64.0 * k as f32 / (n - 1) as f32
                };
                let mut b = mk(o, a, 620.0, dmg, BKind::Bounce);
                b.pierce = 1;
                b.phase = 2.0 + (lv / 2) as f32 + mods.bounce_extra;
                b.life = 2.6;
                out.push(b);
            }
            au.play_vol(Sfx::Laser, 0.3);
            0.17
        }
        Weapon::Grenade => {
            let n = if lv >= 4 { 2 } else { 1 };
            for k in 0..n {
                let a = (k as f32 - (n - 1) as f32 / 2.0) * 14.0;
                let mut b = mk(o, a, 470.0, (2.0 + lv as f32 * 0.5) * dmg, BKind::Grenade);
                b.life = 0.55 + lv as f32 * 0.03;
                b.phase = (46.0 + lv as f32 * 8.0) * mods.blast_mul;
                out.push(b);
            }
            au.play_vol(Sfx::Missile, 0.3);
            0.5
        }
        Weapon::Cryo => {
            let n = [1, 2, 2, 3, 3][lvi];
            for k in 0..n {
                let a = if n == 1 {
                    0.0
                } else {
                    -8.0 + 16.0 * k as f32 / (n - 1) as f32
                };
                let mut b = mk(o, a, 700.0, dmg, BKind::Ice);
                b.pierce = if lv >= 5 { 1 } else { 0 };
                out.push(b);
            }
            au.play_vol(Sfx::Ice, 0.3);
            0.13
        }
        Weapon::Orbit => {
            out.push(mk(o, 0.0, 640.0, dmg, BKind::Shot));
            0.2
        }
        Weapon::BlackHole => {
            out.push(mk(o + vec2(-5.0, 0.0), 0.0, 640.0, 0.7 * dmg, BKind::Shot));
            out.push(mk(o + vec2(5.0, 0.0), 0.0, 640.0, 0.7 * dmg, BKind::Shot));
            if *hole_cd <= 0.0 {
                *hole_cd = (1.6 - lv as f32 * 0.15) / rate;
                let mut b = mk(o, 0.0, 300.0, 0.0, BKind::Hole);
                b.life = (2.6 + lv as f32 * 0.4) * mods.hole_mul;
                b.pierce = 255;
                b.phase = 70.0 + lv as f32 * 12.0;
                out.push(b);
                au.play_vol(Sfx::Portal, 0.35);
            }
            0.18
        }
        Weapon::Hive => {
            let n = ((2 + lv) as f32 * mods.bee_mul) as usize;
            for k in 0..n {
                let side = if k % 2 == 0 { -1.0 } else { 1.0 };
                let mut b = PBullet::new(
                    o + vec2(side * 8.0, 4.0),
                    vec2(side * gen_range(80.0, 240.0), -gen_range(140.0, 300.0)),
                    0.8 * dmg,
                    BKind::Bee,
                    w,
                );
                b.life = 2.4;
                b.phase = gen_range(0.0, TAU);
                out.push(b);
            }
            au.play_vol(Sfx::Buzz, 0.5);
            0.24
        }
        Weapon::Cow => {
            let mut b = mk(o, 0.0, 380.0, (3.0 + lv as f32) * dmg, BKind::Cow);
            b.life = 1.3;
            b.phase = (60.0 + lv as f32 * 6.0) * mods.blast_mul;
            out.push(b);
            au.play_vol(Sfx::Moo, 0.25);
            0.6
        }
        other => super::rifts::fire_advanced(other, lv, dmg, o, ss, hole_cd, rate, mods, out, au),
    }
}

pub(super) fn move_bullet(b: &mut PBullet, dt: f32, targets: &[Vec2], boss_pos: Option<Vec2>) {
    match b.kind {
        BKind::Bounce => {
            b.pos += b.vel * dt;
            if b.pos.x < 6.0 || b.pos.x > W - 6.0 {
                if b.phase >= 1.0 {
                    b.vel.x = -b.vel.x;
                    b.phase -= 1.0;
                    b.pos.x = b.pos.x.clamp(6.0, W - 6.0);
                    b.hit = [0; 6];
                } else {
                    b.life = 0.0;
                }
            }
        }
        BKind::Grenade => {
            b.vel *= 1.0 - (dt * 1.2).min(1.0);
            b.pos += b.vel * dt;
        }
        BKind::Bee => {
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
            let wob = vec2(-cur.y, cur.x) * (b.t * 14.0 + b.phase).sin() * 0.6;
            b.vel = (cur + (desired * 5.0 + wob * 4.0) * dt).normalize_or_zero() * 400.0;
            b.pos += b.vel * dt;
        }
        BKind::Hole => {
            if b.t < 0.6 {
                b.pos += b.vel * dt;
                b.vel *= 1.0 - (dt * 2.5).min(1.0);
            }
        }
        _ => b.pos += b.vel * dt,
    }
}

/// Picking up a new weapon: it replaces the current one and keeps the power level.
pub(super) fn pick_new_weapon(p: &mut Player, w: Weapon) {
    p.weapon = w;
}

pub(super) fn draw_bullet(b: &PBullet, p: Vec2, sp: &Sprites, time: f32) {
    match b.kind {
        BKind::Bounce => {
            draw_poly(
                p.x,
                p.y,
                4,
                6.0,
                time * 400.0,
                Color::new(0.4, 1.0, 0.9, 0.35),
            );
            draw_poly(
                p.x,
                p.y,
                4,
                3.5,
                time * 400.0,
                Color::from_rgba(200, 255, 245, 255),
            );
        }
        BKind::Grenade => {
            draw_circle(p.x, p.y, 6.0, Color::from_rgba(90, 110, 50, 255));
            draw_circle_lines(p.x, p.y, 6.0, 1.5, Color::from_rgba(30, 40, 20, 255));
            if (time * 12.0) as i32 % 2 == 0 {
                draw_circle(p.x + 2.0, p.y - 2.0, 1.8, RED);
            }
        }
        BKind::Ice => {
            let dir = b.vel.normalize_or_zero();
            draw_line(
                p.x,
                p.y,
                p.x - dir.x * 12.0,
                p.y - dir.y * 12.0,
                5.0,
                Color::new(0.6, 0.9, 1.0, 0.4),
            );
            draw_poly(p.x, p.y, 4, 4.0, 45.0, Color::from_rgba(220, 245, 255, 255));
        }
        BKind::Bee => {
            draw_circle(p.x, p.y, 3.5, Color::from_rgba(255, 210, 40, 255));
            draw_line(p.x - 3.0, p.y, p.x + 3.0, p.y, 1.5, BLACK);
            let flap = (time * 60.0).sin() * 2.0;
            draw_circle(
                p.x - 3.0,
                p.y - 3.0 - flap,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.7),
            );
            draw_circle(
                p.x + 3.0,
                p.y - 3.0 - flap,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.7),
            );
        }
        BKind::Hole => {
            if b.t < 0.6 {
                draw_circle(p.x, p.y, 7.0, BLACK);
                draw_circle_lines(p.x, p.y, 8.0, 2.0, Color::from_rgba(170, 100, 255, 255));
            } else {
                let r = b.phase;
                let fade = (b.life * 2.0).min(1.0);
                draw_circle(p.x, p.y, r, Color::new(0.35, 0.1, 0.6, 0.08 * fade));
                for k in 0..4 {
                    let rr = r * (0.3 + 0.18 * k as f32) + (time * 3.0 + k as f32).sin() * 3.0;
                    draw_circle_lines(
                        p.x,
                        p.y,
                        rr,
                        2.0,
                        Color::new(0.7, 0.4, 1.0, (0.5 - k as f32 * 0.1) * fade),
                    );
                }
                for k in 0..6 {
                    let a = time * 4.0 + k as f32 / 6.0 * TAU;
                    let q = p + vec2(a.cos(), a.sin()) * r * 0.55;
                    draw_line(
                        q.x,
                        q.y,
                        p.x,
                        p.y,
                        1.0,
                        Color::new(0.8, 0.6, 1.0, 0.25 * fade),
                    );
                }
                draw_circle(p.x, p.y, 14.0, BLACK);
                draw_circle_lines(p.x, p.y, 15.0, 2.0, Color::new(1.0, 0.8, 1.0, 0.8 * fade));
            }
        }
        BKind::Cow => {
            sp.enemies[Kind::Cow.index()][((time * 8.0) as usize) % 2].draw(
                p,
                1.6,
                b.t * 9.0,
                WHITE,
                false,
            );
        }
        _ => {}
    }
}

pub(super) fn draw_log_drop(p: Vec2, time: f32) {
    let pulse = (time * 5.0).sin() * 0.5 + 0.5;
    draw_circle(p.x, p.y, 18.0 + pulse * 4.0, Color::new(0.3, 1.0, 0.8, 0.2));
    draw_rectangle(
        p.x - 9.0,
        p.y - 11.0,
        18.0,
        22.0,
        Color::from_rgba(30, 60, 70, 255),
    );
    draw_rectangle_lines(
        p.x - 9.0,
        p.y - 11.0,
        18.0,
        22.0,
        2.0,
        Color::from_rgba(120, 255, 220, 255),
    );
    for k in 0..3 {
        draw_line(
            p.x - 5.0,
            p.y - 5.0 + k as f32 * 5.0,
            p.x + 5.0,
            p.y - 5.0 + k as f32 * 5.0,
            1.5,
            Color::from_rgba(120, 255, 220, 255),
        );
    }
}

impl Game {
    // -----------------------------------------------------------------------
    // Hooks called by the main game
    // -----------------------------------------------------------------------

    pub(super) fn expire_bullets(&mut self) {
        self.arsenal_expire();
        for b in &mut self.bullets {
            if b.life <= 0.0 && matches!(b.kind, BKind::Grenade | BKind::Cow | BKind::Mine) {
                self.blasts.push((b.pos, b.phase, b.dmg, b.weapon));
                b.kind = BKind::Shot;
            }
        }
    }

    pub(super) fn drop_log(&mut self, i: usize, pos: Vec2) {
        let found =
            self.cfg.found_logs.get(i).copied().unwrap_or(true) || self.enc.logs_found.contains(&i);
        let queued = self
            .drops
            .iter()
            .any(|d| matches!(d.kind, DropKind::Log(j) if j == i));
        if !found && !queued {
            self.spawn_drop(pos, DropKind::Log(i));
        }
    }

    pub(super) fn collect_log(&mut self, i: usize, au: &mut Audio) {
        if !self.enc.logs_found.contains(&i) {
            self.enc.logs_found.push(i);
            self.events.push(Event::Log(i));
        }
        au.play(Sfx::Log);
        self.banner = Some((LOGS[i].0.to_string(), LOGS[i].1.to_string(), 4.5));
    }

    pub(super) fn event_crystal_mul(&self) -> f32 {
        if self.enc.event == Some(SEvent::GoldRush) {
            2.0
        } else {
            1.0
        }
    }

    pub(super) fn enc_begin_sector(&mut self) {
        self.enc.event = None;
        self.enc.bombs_sector = 0;
        let campaign = matches!(
            self.cfg.mode,
            Mode::Campaign | Mode::Daily | Mode::Expedition
        );
        if campaign
            && self.sector() >= 2
            && !self.secret_stage
            && !self.enc.void
            && gen_range(0.0, 1.0) < 0.4
        {
            let e = SEVENTS[gen_range(0, SEVENTS.len())];
            self.enc.event = Some(e);
            self.enc.event_timer = 3.0;
            if !self.enc.events_seen.contains(&e.index()) {
                self.enc.events_seen.push(e.index());
            }
            self.events.push(Event::SectorEvent(e.index()));
        }
    }

    pub(super) fn enc_sector_clear(&mut self) {
        if self.enc.bombs_sector == 0 {
            for c in &mut self.enc.contracts {
                if c.kind == CKind::NoBomb {
                    c.progress = 1;
                }
            }
        }
        if self.enc.event == Some(SEvent::Eclipse) {
            self.ach(ach::ECLIPSE);
        }
    }

    /// Called right after a normal wave is built.
    pub(super) fn enc_wave_start(&mut self) {
        let tier = self.stage().tier;
        let classic = self.classic();
        // elites
        if tier >= 3 && !classic {
            for e in &mut self.enemies {
                if !e.bonus && e.group < 90 && gen_range(0.0, 1.0) < 0.06 {
                    e.elite = true;
                    e.hp += 2;
                }
            }
        }
        // ambush: half of the groups come in from below
        if self.enc.event == Some(SEvent::Ambush) {
            for e in &mut self.enemies {
                if e.group % 2 == 1
                    && e.group < 90
                    && let EState::Waiting { path, .. } = &mut e.state
                {
                    for seg in path.seg.iter_mut() {
                        for p in seg.iter_mut() {
                            p.y = H - p.y;
                        }
                    }
                    e.pos = path.eval(0.0, None);
                }
            }
        }
        // mimic disguised as a capsule
        if tier >= 2 && !classic && gen_range(0.0, 1.0) < 0.14 {
            self.spawn(
                Kind::Mimic,
                0,
                0,
                EState::Free { vel: Vec2::ZERO },
                96,
                false,
            );
            if let Some(m) = self.enemies.last_mut() {
                m.pos = vec2(gen_range(60.0, W - 60.0), -24.0);
                m.harmless = true;
            }
        }
    }

    pub(super) fn on_kill(&mut self, e: &Enemy, au: &mut Audio) {
        for c in &mut self.enc.contracts {
            if c.kind == CKind::Kill(e.kind) {
                c.progress += 1;
            }
        }
        if e.diving() {
            self.enc.dives += 1;
        }
        if self.enc.fever_t > 0.0 && !e.bonus {
            self.spawn_drop(e.pos, DropKind::Crystal(1));
        }
        if e.elite {
            self.enc.elites += 1;
            self.events.push(Event::Elite);
            self.add_score(e.kind.points() * 2);
            for _ in 0..3 {
                self.spawn_drop(e.pos, DropKind::Crystal(1));
            }
            if self.mods.elite_drop {
                let cap = self.make_capsule();
                self.spawn_drop(e.pos, cap);
                if gen_range(0, 20) == 0 {
                    self.spawn_drop(e.pos, DropKind::Card(cards::roll(1)));
                }
            }
        }
        match e.kind {
            Kind::Mimic => {
                self.enc.mimics += 1;
                self.events.push(Event::Mimic);
                let cap = self.make_capsule();
                self.spawn_drop(e.pos, cap);
                for _ in 0..2 {
                    self.spawn_drop(e.pos, DropKind::Crystal(5));
                }
            }
            Kind::Thief => {
                if let Some(i) = self.enc.loot.iter().position(|(id, _)| *id == e.id) {
                    let (_, l) = self.enc.loot.swap_remove(i);
                    self.ach(ach::THIEF_CATCHER);
                    au.play(Sfx::Fanfare);
                    self.fx
                        .text(e.pos - vec2(0.0, 24.0), "RECOVERED TWICE OVER!", GOLD, 20.0);
                    for _ in 0..2 {
                        let kind = match l {
                            Loot::Capsule => self.make_capsule(),
                            Loot::Card(c) => DropKind::Card(c),
                            Loot::Power(k) => DropKind::Power(k),
                        };
                        self.spawn_drop(e.pos, kind);
                    }
                }
                for _ in 0..3 {
                    self.spawn_drop(e.pos, DropKind::Crystal(5));
                }
            }
            Kind::Freighter => {
                au.play(Sfx::Buy);
                for _ in 0..6 {
                    self.spawn_drop(e.pos, DropKind::Crystal(5));
                }
            }
            Kind::Cow => {
                self.enc.cows += 1;
                self.events.push(Event::Cow);
                self.drop_log(9, e.pos);
            }
            _ => {}
        }
    }

    pub(super) fn on_boss_killed(&mut self, b: &Boss) {
        let campaign = matches!(self.cfg.mode, Mode::Campaign | Mode::Daily);
        let log = match (b.kind, b.final_boss, b.variant) {
            (BossKind::Queen, true, _) => 7,
            (BossKind::Queen, false, _) => 0,
            (BossKind::Warship, _, false) => 1,
            (BossKind::Warship, _, true) => 5,
            (BossKind::Serpent, _, _) => 2,
            (BossKind::Weaver, _, _) => 3,
            (BossKind::Eye, _, _) => 4,
            (BossKind::Twins, _, _) => 6,
            (BossKind::Devourer, _, _) => 11,
            (BossKind::Comet, _, _) => usize::MAX,
        };
        if campaign || b.kind == BossKind::Devourer {
            self.drop_log(log, b.pos + vec2(30.0, 0.0));
        }
        if b.kind == BossKind::Devourer {
            self.enc.void_done = true;
            self.spawn_drop(b.pos - vec2(30.0, 0.0), DropKind::Card(cards::id::DEVOURER));
        }
    }

    // -----------------------------------------------------------------------
    // The Void (secret sector after the last one)
    // -----------------------------------------------------------------------

    pub(super) fn devour_ready(&self) -> bool {
        self.cfg.mode == Mode::Campaign
            && self.cycle == 0
            && self.cfg.diff as u8 >= Difficulty::Hard as u8
            && self.continues_used == 0
            && !self.enc.void
            && !self.enc.void_done
            && self.sector() == SECTORS
    }

    pub(super) fn enter_void(&mut self, au: &mut Audio) -> Phase {
        self.enc.void = true;
        self.wave = 41;
        self.ebullets.clear();
        self.collect_all_drops(au);
        au.music(None);
        au.play(Sfx::Devour);
        Phase::Portal { t: 0.0 }
    }

    // -----------------------------------------------------------------------
    // Update
    // -----------------------------------------------------------------------

    pub(super) fn enc_update(&mut self, dt: f32, au: &mut Audio) {
        self.process_blasts(au);
        self.update_saws(dt, au);
        self.update_holes(dt);
        self.update_mimics(dt, au);
        self.update_thieves(dt, au);
        self.update_events(dt);
        self.update_fever(dt, au);
        self.update_contracts(au);
        self.update_calf(dt, au);
        let pp = self.player.pos;
        // reactive shield
        self.enc.nohit_t += dt;
        if self.mods.regen_shield
            && !self.player.shield
            && self.enc.nohit_t > 25.0
            && self.player.alive()
        {
            self.player.shield = true;
            self.enc.nohit_t = 0.0;
            au.play(Sfx::Shield);
            self.fx
                .text(pp - vec2(0.0, 40.0), "SHIELD RESTORED", CYAN, 18.0);
        }
        // Devourer's gravity
        if let Some(b) = &self.boss
            && b.pull > 0.0
            && self.player.alive()
        {
            let d = (b.pos - pp).normalize_or_zero();
            let p = &mut self.player;
            p.pos += d * 95.0 * dt;
            p.pos.x = p.pos.x.clamp(18.0, W - 18.0);
            p.pos.y = p.pos.y.clamp(H * 0.55, H - 36.0);
        }
        // double arsenal: two different weapons at level 5 in the same run
        if self.mastery_run.iter().filter(|m| **m).count() >= 2 {
            self.ach(ach::DOUBLE_ARSENAL);
        }
    }

    fn process_blasts(&mut self, au: &mut Audio) {
        let blasts: Vec<_> = self.blasts.drain(..).collect();
        for (pos, r, dmg, weapon) in blasts {
            let cow = weapon == Some(Weapon::Cow);
            let fire = weapon == Some(Weapon::Fireball);
            if fire {
                self.fireball_embers(pos, dmg, if r > 80.0 { 16 } else { 8 });
                self.fx
                    .burst(pos, Color::from_rgba(255, 140, 40, 255), 18, 200.0);
            }
            au.play_vol(if cow { Sfx::Moo } else { Sfx::Boom }, 0.6);
            self.fx.explosion(
                pos,
                if cow {
                    WHITE
                } else if fire {
                    Color::from_rgba(255, 120, 30, 255)
                } else {
                    Color::from_rgba(200, 220, 90, 255)
                },
                r > 55.0,
            );
            self.fx
                .ring(pos, r * 3.0, 0.35, Color::new(1.0, 0.9, 0.5, 0.9), 3.0);
            if cow {
                self.fx.text(pos - vec2(0.0, 20.0), "MUU!", WHITE, 22.0);
            }
            self.fx.shake = self.fx.shake.max(0.15);
            let hit = dmg.ceil() as i32;
            for e in &mut self.enemies {
                if e.active() && !e.harmless && e.pos.distance(pos) < r + e.kind.radius() {
                    e.hp -= if e.frozen > 0.0 { hit * 2 } else { hit };
                    e.shield = 0;
                    e.flash = 0.1;
                    e.damaged = true;
                    e.last_weapon = weapon;
                }
                if e.kind == Kind::Mimic && e.pos.distance(pos) < r {
                    e.damaged = true;
                }
            }
            if let Some(b) = &mut self.boss
                && b.targetable()
            {
                let mut probes = vec![pos];
                for k in 0..8 {
                    let a = k as f32 / 8.0 * TAU;
                    probes.push(pos + vec2(a.cos(), a.sin()) * r * 0.6);
                }
                if let Some(t) = probes.iter().find_map(|p| b.hit_test(*p)) {
                    let res = b.damage(t, dmg * 1.5);
                    if res.part_destroyed {
                        self.enc.parts += 1;
                    }
                }
            }
            for rock in &mut self.rocks {
                if !rock.solid && rock.pos.distance(pos) < r + 20.0 {
                    rock.hp -= hit;
                }
            }
        }
    }

    fn update_saws(&mut self, dt: f32, au: &mut Audio) {
        self.enc.saws.clear();
        if self.player.weapon != Weapon::Orbit || !self.player.alive() {
            return;
        }
        let lv = self.effective_level();
        let n = (lv + 1) as usize;
        let pp = self.player.pos;
        for i in 0..n {
            let a = self.time * 4.0 + i as f32 / n as f32 * TAU;
            self.enc.saws.push(pp + vec2(a.cos(), a.sin()) * 48.0);
        }
        let saws = self.enc.saws.clone();
        let dmg = self.ship().dmg.ceil() as i32;
        let mut hits = 0;
        for e in &mut self.enemies {
            if !e.active() || e.harmless || e.saw_cd > 0.0 {
                continue;
            }
            if saws
                .iter()
                .any(|s| s.distance(e.pos) < e.kind.radius() + 11.0)
            {
                e.hp -= dmg;
                e.saw_cd = 0.22;
                e.flash = 0.06;
                e.damaged = true;
                e.last_weapon = Some(Weapon::Orbit);
                hits += 1;
            }
        }
        if hits > 0 {
            au.play_vol(Sfx::Clang, 0.3);
        }
        let before = self.ebullets.len();
        self.ebullets
            .retain(|b| !saws.iter().any(|s| s.distance(b.pos) < 11.0 + b.r));
        if self.ebullets.len() < before {
            self.fx.sparks(pp, vec2(0.0, -1.0), WHITE, 2);
        }
        if let Some(b) = &mut self.boss
            && b.targetable()
        {
            for s in &saws {
                if let Some(t) = b.hit_test(*s) {
                    b.damage(t, 6.0 * dt * dmg as f32);
                }
            }
        }
    }

    fn update_holes(&mut self, dt: f32) {
        let holes: Vec<(Vec2, f32)> = self
            .bullets
            .iter()
            .filter(|b| b.kind == BKind::Hole && b.t >= 0.6)
            .map(|b| (b.pos, b.phase))
            .collect();
        if holes.is_empty() {
            return;
        }
        let mut absorbed = 0;
        for b in &mut self.ebullets {
            for (p, r) in &holes {
                let d = b.pos.distance(*p);
                if d < r * 1.7 {
                    let pull = (*p - b.pos).normalize_or_zero();
                    b.pos += pull * 140.0 * dt;
                }
            }
        }
        self.ebullets.retain(|b| {
            let eaten = holes.iter().any(|(p, r)| b.pos.distance(*p) < r * 0.35);
            if eaten {
                absorbed += 1;
            }
            !eaten
        });
        self.absorbed += absorbed;
        for e in &mut self.enemies {
            if !e.active() || e.harmless {
                continue;
            }
            for (p, r) in &holes {
                if e.pos.distance(*p) < r * 0.75 && gen_range(0.0, 1.0) < 3.0 * dt {
                    e.hp -= 1;
                    e.flash = 0.05;
                    e.damaged = true;
                    e.last_weapon = Some(Weapon::BlackHole);
                }
            }
        }
        if self.mods.hole_magnet {
            for d in &mut self.drops {
                if let DropKind::Crystal(_) = d.kind {
                    for (p, r) in &holes {
                        if d.pos.distance(*p) < r * 2.0 {
                            d.pos += (*p - d.pos).normalize_or_zero() * 200.0 * dt;
                        }
                    }
                }
            }
        }
        if let Some(b) = &mut self.boss
            && b.targetable()
        {
            for (p, r) in &holes {
                if b.pos.distance(*p) < r + 50.0 {
                    b.damage(crate::boss::HitTarget::Body, 5.0 * dt);
                }
            }
        }
    }

    fn update_mimics(&mut self, dt: f32, au: &mut Audio) {
        let pp = self.player.pos;
        let alive = self.player.alive();
        let time = self.time;
        let mut bursts = vec![];
        for e in &mut self.enemies {
            if e.kind != Kind::Mimic || e.dead {
                continue;
            }
            if !e.revealed {
                e.pos.y += 75.0 * dt;
                e.pos.x += (time * 2.0 + e.seed).sin() * 30.0 * dt;
                if (alive && pp.distance(e.pos) < 130.0) || e.damaged {
                    e.revealed = true;
                    e.harmless = false;
                    e.spawn_cd = 1.2;
                    e.state = EState::Free {
                        vel: (pp - e.pos).normalize_or_zero() * 250.0,
                    };
                    bursts.push(e.pos);
                }
                if e.pos.y > H + 30.0 {
                    e.dead = true;
                    e.hp = e.hp.max(1);
                }
                continue;
            }
            e.spawn_cd -= dt;
            if let EState::Free { vel } = &mut e.state {
                if e.spawn_cd <= 0.0 {
                    *vel = vec2(vel.x * 0.5, -280.0);
                }
                e.pos += *vel * dt;
            }
            if e.pos.y < -50.0 || e.pos.y > H + 50.0 {
                e.dead = true;
                e.hp = e.hp.max(1);
            }
        }
        let bspeed = 200.0 * self.bullet_mul();
        for p in bursts {
            au.play(Sfx::Steal);
            self.fx.text(p - vec2(0.0, 26.0), "MIMIC!", RED, 24.0);
            self.fx.burst(p, RED, 20, 160.0);
            for k in 0..10 {
                let a = k as f32 / 10.0 * TAU;
                self.ebullets
                    .push(EBullet::new(p, vec2(a.cos(), a.sin()) * bspeed, 4.0, 0));
            }
        }
    }

    fn update_thieves(&mut self, dt: f32, au: &mut Audio) {
        let tier = self.stage().tier;
        self.enc.thief_timer -= dt;
        if self.enc.thief_timer <= 0.0 {
            self.enc.thief_timer = 2.0;
            let loot_on_screen = self.drops.iter().any(|d| {
                matches!(
                    d.kind,
                    DropKind::Capsule { .. } | DropKind::Card(_) | DropKind::Power(_)
                ) && d.pos.y > 40.0
            });
            let has_thief = self
                .enemies
                .iter()
                .any(|e| e.kind == Kind::Thief && !e.dead);
            if loot_on_screen
                && !has_thief
                && tier >= 2
                && !self.classic()
                && matches!(self.phase, Phase::Playing)
                && gen_range(0.0, 1.0) < 0.3
            {
                let x = if gen_range(0, 2) == 0 { 20.0 } else { W - 20.0 };
                self.spawn(
                    Kind::Thief,
                    0,
                    0,
                    EState::Free { vel: Vec2::ZERO },
                    95,
                    false,
                );
                if let Some(t) = self.enemies.last_mut() {
                    t.pos = vec2(x, -20.0);
                }
                self.banner = Some((
                    "THIEF!".into(),
                    "It wants what's falling. Shoot it down!".into(),
                    2.0,
                ));
            }
        }
        let slow = if self.mods.thief_slow { 0.6 } else { 1.0 };
        let mut stolen: Vec<(u32, usize)> = vec![];
        let mut escaped: Vec<u32> = vec![];
        for e in &mut self.enemies {
            if e.kind != Kind::Thief || e.dead {
                continue;
            }
            let carrying = self.enc.loot.iter().any(|(id, _)| *id == e.id);
            if carrying {
                e.pos.y -= 260.0 * slow * dt;
                e.pos.x += (self.time * 6.0).sin() * 60.0 * dt;
            } else {
                let target = self
                    .drops
                    .iter()
                    .enumerate()
                    .filter(|(_, d)| {
                        matches!(
                            d.kind,
                            DropKind::Capsule { .. } | DropKind::Card(_) | DropKind::Power(_)
                        )
                    })
                    .min_by(|a, b| a.1.pos.distance(e.pos).total_cmp(&b.1.pos.distance(e.pos)));
                match target {
                    Some((i, d)) => {
                        let to = d.pos - e.pos;
                        if to.length() < 20.0 {
                            stolen.push((e.id, i));
                        } else {
                            e.pos += to.normalize() * 230.0 * slow * dt;
                        }
                    }
                    None => e.pos.y -= 220.0 * slow * dt,
                }
            }
            if e.pos.y < -40.0 {
                e.dead = true;
                e.hp = e.hp.max(1);
                if carrying {
                    escaped.push(e.id);
                }
            }
        }
        stolen.sort_by_key(|s| std::cmp::Reverse(s.1));
        for (id, i) in stolen {
            if i >= self.drops.len() {
                continue;
            }
            let d = self.drops.swap_remove(i);
            let l = match d.kind {
                DropKind::Card(c) => Loot::Card(c),
                DropKind::Power(k) => Loot::Power(k),
                _ => Loot::Capsule,
            };
            self.enc.loot.push((id, l));
            au.play(Sfx::Steal);
            self.fx.text(d.pos - vec2(0.0, 20.0), "STOLEN!", RED, 20.0);
        }
        for id in escaped {
            self.enc.loot.retain(|(i, _)| *i != id);
            self.banner = Some((
                "THE THIEF GOT AWAY".into(),
                "Next time, be faster.".into(),
                2.0,
            ));
        }
    }

    fn update_events(&mut self, dt: f32) {
        let Some(ev) = self.enc.event else { return };
        let playing = matches!(self.phase, Phase::Playing) && self.wave_kind == WaveKind::Normal;
        if !playing {
            return;
        }
        self.enc.event_timer -= dt;
        match ev {
            SEvent::Meteors if self.enc.event_timer <= 0.0 => {
                self.enc.event_timer = gen_range(1.4, 2.6);
                self.rocks.push(Rock {
                    pos: vec2(gen_range(30.0, W - 30.0), -30.0),
                    vel: vec2(gen_range(-50.0, 50.0), gen_range(90.0, 150.0)),
                    hp: 5,
                    big: gen_range(0, 3) == 0,
                    solid: false,
                    rot: 0.0,
                    vr: gen_range(-3.0, 3.0),
                    variant: gen_range(0, 2),
                    flash: 0.0,
                });
            }
            SEvent::Convoy if self.enc.event_timer <= 0.0 => {
                self.enc.event_timer = gen_range(6.0, 9.0);
                let y = gen_range(150.0, 300.0);
                let (a, b) = if gen_range(0, 2) == 0 {
                    (-50.0, W + 50.0)
                } else {
                    (W + 50.0, -50.0)
                };
                let path = Path::one(
                    vec2(a, y),
                    vec2(W * 0.35, y + 30.0),
                    vec2(W * 0.65, y - 30.0),
                    vec2(b, y),
                );
                self.spawn_special(Kind::Freighter, path, 7.0, 0.0);
            }
            SEvent::Gravity => {
                let pp = self.player.pos;
                for b in &mut self.ebullets {
                    if b.style == 3 {
                        continue;
                    }
                    let spd = b.vel.length();
                    let want = (pp - b.pos).normalize_or_zero();
                    b.vel = (b.vel.normalize_or_zero() + want * 0.9 * dt).normalize_or_zero() * spd;
                }
            }
            _ => {}
        }
    }

    fn update_fever(&mut self, dt: f32, au: &mut Audio) {
        if self.combo == 0 {
            self.enc.fever_lock = false;
        }
        if !self.enc.fever_lock && self.combo >= self.mods.fever_start && !self.classic() {
            self.enc.fever_lock = true;
            self.enc.fever_t = 8.0 * self.mods.fever_mul;
            self.enc.fevers += 1;
            au.play(Sfx::Fever);
            self.banner = Some((
                "FEVER!".into(),
                "Double points and crystals on every kill!".into(),
                2.0,
            ));
            self.ach(ach::FEVER);
            if self.enc.fevers >= 3 {
                self.ach(ach::FEVER3);
            }
        }
        self.enc.fever_t = (self.enc.fever_t - dt).max(0.0);
    }

    fn update_contracts(&mut self, au: &mut Audio) {
        let (combo, grazes, frozen) = (self.combo, self.grazes, self.frozen_count);
        let mut finished = vec![];
        for (i, c) in self.enc.contracts.iter_mut().enumerate() {
            if c.done {
                continue;
            }
            match c.kind {
                CKind::Combo => c.progress = c.progress.max(combo),
                CKind::Grazes => c.progress = grazes,
                CKind::Capsules => c.progress = self.enc.capsules,
                CKind::Crystals => c.progress = self.enc.crystals,
                CKind::Dives => c.progress = self.enc.dives,
                CKind::Parts => c.progress = self.enc.parts,
                CKind::Elites => c.progress = self.enc.elites,
                CKind::Frozen => c.progress = frozen,
                CKind::Fever => c.progress = self.enc.fevers,
                _ => {}
            }
            if c.progress >= c.target {
                c.done = true;
                finished.push(i);
            }
        }
        for i in finished {
            let mul = self.mods.contract_mul;
            let c = &self.enc.contracts[i];
            self.enc.contract_xp += c.xp * mul;
            self.enc.contracts_done += 1;
            self.crystals += 25.0 * mul as f32;
            let text = c.text();
            au.play(Sfx::Contract);
            self.banner = Some((
                "CONTRACT COMPLETE".into(),
                format!("{text}  ·  +{} XP  +{} crystals", c.xp * mul, 25 * mul),
                3.0,
            ));
            self.events.push(Event::Contract);
        }
    }

    fn update_calf(&mut self, dt: f32, au: &mut Audio) {
        if !self.mods.calf || !self.player.alive() {
            return;
        }
        let pp = self.player.pos;
        let target = pp + vec2(-52.0, 18.0 + (self.time * 3.0).sin() * 6.0);
        self.enc.calf += (target - self.enc.calf) * (dt * 4.0).min(1.0);
        let calf = self.enc.calf;
        for d in &mut self.drops {
            if matches!(d.kind, DropKind::Crystal(_)) && d.pos.distance(calf) < 70.0 {
                d.pos = d.pos.lerp(pp, (dt * 6.0).min(1.0));
            }
        }
        self.enc.calf_moo -= dt;
        if self.enc.calf_moo <= 0.0 {
            self.enc.calf_moo = gen_range(18.0, 35.0);
            au.play_vol(Sfx::Moo, 0.35);
            self.fx.text(calf - vec2(0.0, 18.0), "muu", WHITE, 14.0);
        }
    }

    // -----------------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------------

    pub(super) fn enc_draw(&self, off: Vec2, sp: &Sprites) {
        // mimics disguised as capsules
        for e in &self.enemies {
            if e.kind != Kind::Mimic || e.revealed || e.dead {
                continue;
            }
            let p = e.pos + off;
            let w = WEAPONS[(e.seed * 10.0) as usize % 3];
            let w2 = WEAPONS[(e.seed * 10.0 + 1.0) as usize % 3];
            let prog = (self.time * 0.55 + e.seed).fract();
            super::rifts::draw_capsule(p, w, Some(w2), prog, self.time, false);
            if self.mods.mimic_mark {
                draw_circle_lines(p.x, p.y, 22.0, 2.0, RED);
            }
        }
        // elites and frozen enemies
        for e in &self.enemies {
            if !e.active() {
                continue;
            }
            let p = e.pos + off;
            if e.elite {
                let pulse = (self.time * 6.0 + e.seed).sin() * 0.5 + 0.5;
                draw_circle_lines(
                    p.x,
                    p.y,
                    e.kind.radius() + 6.0,
                    2.0,
                    Color::new(1.0, 0.85, 0.3, 0.5 + pulse * 0.5),
                );
            }
            if e.frozen > 0.0 {
                draw_circle(
                    p.x,
                    p.y,
                    e.kind.radius() + 3.0,
                    Color::new(0.6, 0.9, 1.0, 0.35),
                );
                draw_circle_lines(
                    p.x,
                    p.y,
                    e.kind.radius() + 3.0,
                    1.5,
                    Color::new(0.9, 1.0, 1.0, 0.8),
                );
            }
            if e.kind == Kind::Thief && self.enc.loot.iter().any(|(id, _)| *id == e.id) {
                draw_circle(p.x, p.y + 14.0, 7.0, Color::from_rgba(170, 120, 60, 255));
                ui::text_c("$", p.x, p.y + 19.0, 13.0, GOLD);
            }
        }
        // Orbit saws
        for s in &self.enc.saws {
            let p = *s + off;
            let a = self.time * 900.0;
            draw_poly(p.x, p.y, 8, 10.0, a, Color::from_rgba(200, 205, 220, 255));
            draw_poly_lines(p.x, p.y, 4, 11.0, a + 22.0, 2.0, WHITE);
            draw_circle(p.x, p.y, 3.0, Color::from_rgba(60, 60, 80, 255));
        }
        // cosmic calf
        if self.mods.calf && self.player.alive() {
            let f = ((self.time * 4.0) as usize) % 2;
            sp.enemies[Kind::Cow.index()][f].draw(
                self.enc.calf + off,
                1.6,
                (self.time * 2.0).sin() * 0.2,
                WHITE,
                false,
            );
        }
    }

    /// Screen effects on top of everything (before the HUD).
    pub(super) fn enc_draw_late(&self) {
        if self.enc.event == Some(SEvent::Eclipse)
            && matches!(self.phase, Phase::Playing | Phase::WaveClear { .. })
            && self.wave_kind == WaveKind::Normal
        {
            let p = self.player.pos;
            let r = 200.0;
            let dark = Color::new(0.0, 0.0, 0.02, 0.86);
            for k in 0..14 {
                let rr = r + k as f32 * 12.0;
                draw_circle_lines(
                    p.x,
                    p.y,
                    rr,
                    13.0,
                    Color::new(0.0, 0.0, 0.02, 0.06 * k as f32),
                );
            }
            let rr = r + 14.0 * 12.0;
            draw_rectangle(0.0, 0.0, W, (p.y - rr).max(0.0), dark);
            draw_rectangle(0.0, p.y + rr, W, H, dark);
            draw_rectangle(0.0, p.y - rr, (p.x - rr).max(0.0), rr * 2.0, dark);
            draw_rectangle(p.x + rr, p.y - rr, W, rr * 2.0, dark);
            // enemies and shots glow in the dark
            for e in &self.enemies {
                if e.active() {
                    draw_circle(e.pos.x, e.pos.y, 4.0, Color::new(1.0, 0.3, 0.3, 0.8));
                    draw_circle(e.pos.x - 4.0, e.pos.y - 2.0, 1.5, WHITE);
                    draw_circle(e.pos.x + 4.0, e.pos.y - 2.0, 1.5, WHITE);
                }
            }
            for b in &self.ebullets {
                draw_circle(b.pos.x, b.pos.y, b.r, Color::new(1.0, 0.4, 0.6, 0.9));
            }
        }
        if self.enc.fever_t > 0.0 {
            let c = ui::hsl(self.time * 0.6, 0.9, 0.6);
            let a = (self.enc.fever_t * 2.0).min(1.0) * (0.5 + (self.time * 10.0).sin() * 0.2);
            for k in 0..6 {
                let w = 6.0 - k as f32;
                let col = Color::new(c.r, c.g, c.b, a * 0.15 * w / 6.0);
                draw_rectangle(k as f32 * 4.0, 0.0, 4.0, H, col);
                draw_rectangle(W - (k + 1) as f32 * 4.0, 0.0, 4.0, H, col);
            }
            ui::text_c(
                &format!("FEVER x2  {:.0}", self.enc.fever_t.ceil()),
                W / 2.0,
                92.0,
                18.0,
                Color::new(c.r, c.g, c.b, a + 0.3),
            );
        }
        if let (Phase::Intro { t }, Some(ev)) = (&self.phase, self.enc.event) {
            let a = ((t - 0.6) * 2.0).clamp(0.0, 1.0) * ((2.8 - t) * 2.0).clamp(0.0, 1.0);
            let y = H * 0.62;
            draw_rectangle(
                20.0,
                y - 26.0,
                W - 40.0,
                52.0,
                Color::new(0.3, 0.05, 0.1, 0.7 * a),
            );
            ui::text_c(
                &format!("EVENT: {}", ev.name()),
                W / 2.0,
                y - 4.0,
                20.0,
                Color::new(1.0, 0.6, 0.3, a),
            );
            ui::text_c(
                ev.desc(),
                W / 2.0,
                y + 16.0,
                14.0,
                Color::new(1.0, 1.0, 1.0, a),
            );
        }
    }
}
