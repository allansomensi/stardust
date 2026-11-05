//! A run: modes, sectors, waves, player, enemies, bosses, shop and secrets.

use crate::art::Art;
use crate::audio::{Audio, Music, Sfx};
use crate::boss::{Boss, BossKind, BossOut};
use crate::cards::{self, Mods};
use crate::fx::{Background, Fx, THEMES};
use crate::progress::{Daily, Difficulty, Mode, ach};
use crate::ships::{Ability, SHIPS, ShipDef, WEAPONS, Weapon};
use crate::sprites::{Icon, Sprites};
use crate::ui::{self, CYAN, DIM, GOLD, GREEN, PINK, RED};
use crate::{H, Input, W};
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub mod arcade;
pub mod arsenal;
mod collisions;
pub mod coop;
mod draw;
mod drops;
pub mod encounters;
mod enemies;
pub mod glow;
mod hud;
pub mod levels;
pub mod minigame;
mod player;
pub mod rifts;
pub mod scenery;
mod shop;
pub mod signal;
mod update;
pub mod vanguard;
mod waves;

pub const COLS: usize = 8;
/// Ceiling for every drop-rate boost (cards, shop items, blessings), so the screen never
/// floods with items.
const MAX_DROP_BOOST: f32 = 1.6;

/// Campaign length: 20 sectors of 5 levels each = 100 levels.
pub const SECTORS: u32 = 20;
/// Levels per sector.
pub const SUBS: u32 = 5;
/// Sectors where a campaign can be resumed (levels 1, 26, 51 and 76).
pub const CHECKPOINTS: [u32; 4] = [1, 6, 11, 16];
/// Fixed ship height: it only moves sideways.
pub const PLAYER_Y: f32 = H - 80.0;
const S: f32 = 3.0;

// ---------------------------------------------------------------------------
// Enemies
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Drone,
    Wasp,
    Beetle,
    Gunner,
    Splitter,
    Kamikaze,
    Mini,
    Shieldbearer,
    Weaver,
    Carrier,
    Chameleon,
    Golden,
    Cow,
    Merchant,
    Mimic,
    Thief,
    Freighter,
    Minelayer,
    Jumper,
    Armored,
    Healer,
    Mirror,
    Leader,
}

impl Kind {
    pub fn hp(self) -> i32 {
        match self {
            Kind::Beetle | Kind::Splitter => 2,
            Kind::Carrier => 5,
            Kind::Golden => 8,
            Kind::Cow => 6,
            Kind::Merchant => 12,
            Kind::Freighter => 8,
            Kind::Thief => 3,
            Kind::Mimic => 4,
            Kind::Minelayer => 2,
            Kind::Armored => 3,
            Kind::Healer => 3,
            Kind::Mirror => 3,
            Kind::Leader => 6,
            _ => 1,
        }
    }
    pub fn points(self) -> u32 {
        match self {
            Kind::Drone => 50,
            Kind::Wasp => 80,
            Kind::Beetle => 150,
            Kind::Gunner => 120,
            Kind::Splitter => 100,
            Kind::Kamikaze => 100,
            Kind::Mini => 40,
            Kind::Shieldbearer => 140,
            Kind::Weaver => 110,
            Kind::Carrier => 300,
            Kind::Chameleon => 130,
            Kind::Golden => 5000,
            Kind::Cow => 1000,
            Kind::Merchant => 0,
            Kind::Mimic => 400,
            Kind::Thief => 500,
            Kind::Freighter => 300,
            Kind::Minelayer => 160,
            Kind::Jumper => 140,
            Kind::Armored => 250,
            Kind::Healer => 300,
            Kind::Mirror => 260,
            Kind::Leader => 1500,
        }
    }
    pub fn color(self) -> Color {
        match self {
            Kind::Drone => Color::from_rgba(110, 230, 110, 255),
            Kind::Wasp => Color::from_rgba(190, 120, 255, 255),
            Kind::Beetle => Color::from_rgba(255, 90, 80, 255),
            Kind::Gunner => Color::from_rgba(255, 170, 60, 255),
            Kind::Splitter | Kind::Mini => Color::from_rgba(80, 230, 210, 255),
            Kind::Kamikaze => Color::from_rgba(255, 120, 200, 255),
            Kind::Shieldbearer => Color::from_rgba(90, 150, 240, 255),
            Kind::Weaver => Color::from_rgba(220, 200, 255, 255),
            Kind::Carrier => Color::from_rgba(200, 130, 70, 255),
            Kind::Chameleon => Color::from_rgba(120, 240, 150, 255),
            Kind::Golden => Color::from_rgba(255, 220, 80, 255),
            Kind::Cow => WHITE,
            Kind::Merchant => Color::from_rgba(150, 220, 255, 255),
            Kind::Mimic => Color::from_rgba(255, 80, 80, 255),
            Kind::Thief => Color::from_rgba(140, 90, 200, 255),
            Kind::Freighter => Color::from_rgba(255, 170, 60, 255),
            Kind::Minelayer => Color::from_rgba(255, 110, 70, 255),
            Kind::Jumper => Color::from_rgba(255, 150, 255, 255),
            Kind::Armored => Color::from_rgba(190, 200, 220, 255),
            Kind::Healer => Color::from_rgba(130, 240, 110, 255),
            Kind::Mirror => Color::from_rgba(150, 240, 255, 255),
            Kind::Leader => Color::from_rgba(255, 210, 80, 255),
        }
    }
    fn radius(self) -> f32 {
        match self {
            Kind::Mini => 9.0,
            Kind::Kamikaze => 11.0,
            Kind::Carrier | Kind::Cow | Kind::Leader => 17.0,
            Kind::Merchant | Kind::Freighter => 21.0,
            _ => 14.0,
        }
    }
    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy)]
pub struct Path {
    seg: [[Vec2; 4]; 2],
    n: usize,
}

fn bez(p: &[Vec2; 4], t: f32) -> Vec2 {
    let u = 1.0 - t;
    p[0] * (u * u * u) + p[1] * (3.0 * u * u * t) + p[2] * (3.0 * u * t * t) + p[3] * (t * t * t)
}

impl Path {
    fn one(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Path {
        Path {
            seg: [[a, b, c, d], [d, d, d, d]],
            n: 1,
        }
    }
    fn two(s1: [Vec2; 4], s2: [Vec2; 4]) -> Path {
        Path {
            seg: [s1, s2],
            n: 2,
        }
    }
    fn eval(&self, t: f32, end: Option<Vec2>) -> Vec2 {
        let t = t.clamp(0.0, 1.0);
        let tt = t * self.n as f32;
        let i = (tt.floor() as usize).min(self.n - 1);
        let mut s = self.seg[i];
        if i == self.n - 1
            && let Some(e) = end
        {
            s[3] = e;
        }
        bez(&s, tt - i as f32)
    }
    fn offset(mut self, o: Vec2) -> Path {
        for s in &mut self.seg {
            for p in s.iter_mut() {
                *p += o;
            }
        }
        self
    }
}

#[derive(Clone, Copy)]
enum EState {
    Waiting {
        delay: f32,
        path: Path,
        dur: f32,
    },
    Entering {
        t: f32,
        path: Path,
        dur: f32,
    },
    Formation,
    Diving {
        t: f32,
        dur: f32,
        path: Path,
        shots: u8,
    },
    Returning,
    Flyby {
        t: f32,
        dur: f32,
        path: Path,
    },
    Free {
        vel: Vec2,
    },
}

pub struct Enemy {
    id: u32,
    pub kind: Kind,
    hp: i32,
    shield: i32,
    col: usize,
    row: usize,
    pub pos: Vec2,
    rot: f32,
    state: EState,
    flash: f32,
    seed: f32,
    group: u32,
    bonus: bool,
    harmless: bool,
    dead: bool,
    entry_shot: f32,
    spawn_cd: f32,
    last_weapon: Option<Weapon>,
    damaged: bool,
    gifted: bool,
    frozen: f32,
    saw_cd: f32,
    elite: bool,
    revealed: bool,
}

impl Enemy {
    fn active(&self) -> bool {
        !matches!(self.state, EState::Waiting { .. }) && !self.dead
    }
    fn diving(&self) -> bool {
        matches!(self.state, EState::Diving { .. } | EState::Free { .. })
    }
    fn alpha(&self, time: f32) -> f32 {
        if self.kind == Kind::Chameleon {
            (((time * 2.1 + self.seed).sin() * 0.5 + 0.5) * 1.15).clamp(0.08, 1.0)
        } else {
            1.0
        }
    }
}

// ---------------------------------------------------------------------------
// Projectiles, drops and hazards
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum BKind {
    Shot,
    Laser,
    Missile,
    Star,
    Wave,
    Pellet,
    Disc,
    Flame,
    Bounce,
    Grenade,
    Ice,
    Bee,
    Hole,
    Cow,
    Needle,
    Boomerang,
    Prism,
    Mine,
    Nova,
    Dust,
    Lance,
    Charge,
    Fireball,
    Plasma,
    Rail,
    Meteor,
}

struct PBullet {
    pos: Vec2,
    vel: Vec2,
    dmg: f32,
    kind: BKind,
    pierce: u8,
    hit: [u32; 6],
    life: f32,
    t: f32,
    base_x: f32,
    phase: f32,
    weapon: Option<Weapon>,
}

impl PBullet {
    fn new(pos: Vec2, vel: Vec2, dmg: f32, kind: BKind, weapon: Weapon) -> PBullet {
        PBullet {
            pos,
            vel,
            dmg,
            kind,
            pierce: 0,
            hit: [0; 6],
            life: 2.0,
            t: 0.0,
            base_x: pos.x,
            phase: 0.0,
            weapon: Some(weapon),
        }
    }
}

pub struct EBullet {
    pub pos: Vec2,
    pub vel: Vec2,
    pub r: f32,
    /// 0 pink, 1 orange, 2 red, 3 web (slows you down), 4 purple
    pub style: u8,
    grazed: bool,
}

impl EBullet {
    pub fn new(pos: Vec2, vel: Vec2, r: f32, style: u8) -> EBullet {
        EBullet {
            pos,
            vel,
            r,
            style,
            grazed: false,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PowerKind {
    Frenzy,
    Star,
    Magnet,
    SlowMo,
    ExtraLife,
    Bomb,
    Shield,
    Crystals,
}

impl PowerKind {
    fn letter(self) -> &'static str {
        match self {
            PowerKind::Frenzy => "F",
            PowerKind::Star => "*",
            PowerKind::Magnet => "U",
            PowerKind::SlowMo => "Z",
            PowerKind::ExtraLife => "1UP",
            PowerKind::Bomb => "B",
            PowerKind::Shield => "S",
            PowerKind::Crystals => "$",
        }
    }
    fn name(self) -> &'static str {
        match self {
            PowerKind::Frenzy => "FRENZY!",
            PowerKind::Star => "INVINCIBLE!",
            PowerKind::Magnet => "SUPER MAGNET!",
            PowerKind::SlowMo => "SLOW MOTION!",
            PowerKind::ExtraLife => "EXTRA LIFE!",
            PowerKind::Bomb => "+1 BOMB",
            PowerKind::Shield => "SHIELD!",
            PowerKind::Crystals => "+25 CRYSTALS",
        }
    }
    fn color(self) -> Color {
        match self {
            PowerKind::Frenzy => Color::from_rgba(255, 80, 60, 255),
            PowerKind::Star => Color::from_rgba(255, 230, 80, 255),
            PowerKind::Magnet => Color::from_rgba(255, 90, 140, 255),
            PowerKind::SlowMo => Color::from_rgba(120, 160, 255, 255),
            PowerKind::ExtraLife => Color::from_rgba(120, 255, 120, 255),
            PowerKind::Bomb => Color::from_rgba(200, 200, 210, 255),
            PowerKind::Shield => Color::from_rgba(100, 230, 255, 255),
            PowerKind::Crystals => Color::from_rgba(120, 220, 255, 255),
        }
    }
    fn random() -> PowerKind {
        match gen_range(0, 100) {
            0..=17 => PowerKind::Frenzy,
            18..=27 => PowerKind::Star,
            28..=42 => PowerKind::Magnet,
            43..=55 => PowerKind::SlowMo,
            56..=59 => PowerKind::ExtraLife,
            60..=74 => PowerKind::Bomb,
            75..=86 => PowerKind::Shield,
            _ => PowerKind::Crystals,
        }
    }
}

enum DropKind {
    Capsule {
        weapons: [Weapon; 4],
        n: usize,
        idx: usize,
        t: f32,
    },
    Crystal(u32),
    Card(usize),
    Power(PowerKind),
    Log(usize),
    Curse(rifts::Curse, i32),
    Relic(Weapon),
    Rift,
    Letter(usize),
}

struct Drop {
    pos: Vec2,
    vel: Vec2,
    kind: DropKind,
    age: f32,
}

struct Rock {
    pos: Vec2,
    vel: Vec2,
    hp: i32,
    big: bool,
    solid: bool,
    rot: f32,
    vr: f32,
    variant: usize,
    flash: f32,
}

struct Strike {
    x: f32,
    t: f32,
}

struct Bolt {
    pts: Vec<Vec2>,
    life: f32,
}

// ---------------------------------------------------------------------------
// Player
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct Player {
    pub pos: Vec2,
    vel: Vec2,
    pub lives: i32,
    pub bombs: i32,
    pub weapon: Weapon,
    pub level: i32,
    cd: f32,
    missile_cd: f32,
    pub invuln: f32,
    dead_t: f32,
    pub shield: bool,
    pub speed_lvl: u32,
    pub fire_lvl: u32,
    pub magnet_lvl: u32,
    shots: u32,
    phoenix_used: bool,
    tilt: f32,
    focus: bool,
    pub meter: f32,
    trigger_t: f32,
    pub charge: f32,
    /// Co-op: this ship was already rescued once in this run.
    revived: bool,
}

impl Player {
    fn alive(&self) -> bool {
        self.dead_t <= 0.0 && self.lives > 0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum WaveKind {
    Normal,
    Challenge,
    Golden,
    Boss,
}

pub enum Phase {
    Intro {
        t: f32,
    },
    Playing,
    WaveClear {
        t: f32,
    },
    ChallengeEnd {
        t: f32,
        hits: u32,
        total: u32,
        bonus: u32,
    },
    BossDeath {
        t: f32,
    },
    SectorClear {
        t: f32,
        lines: Vec<(String, u32, bool)>,
        shown: usize,
    },
    Shop,
    Portal {
        t: f32,
    },
    Credits {
        t: f32,
    },
    Victory {
        sel: usize,
        t: f32,
    },
    Continue {
        t: f32,
    },
    GameOver {
        t: f32,
    },
}

/// Facts the interface turns into stats, bestiary entries and achievements.
pub enum Event {
    Ach(usize),
    Seen(usize),
    Kill(Kind, Option<Weapon>),
    BossKilled,
    Bomb,
    Graze,
    Crystals(u32),
    Mastery(usize),
    MerchantSpared,
    Victory,
    SurvivalWave(u32),
    Contract,
    Log(usize),
    Elite,
    Mimic,
    Cow,
    SectorEvent(usize),
    CurseTaken,
    CurseShot,
    Relic(usize),
    Minigame(u8),
    Room,
    Word,
    Ufo,
    /// Gamepad vibration for a seat (None = everyone).
    Rumble(Option<usize>, f32, u32),
}

// ---------------------------------------------------------------------------
// Shop
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Item {
    ExtraLife,
    Bomb,
    Shield,
    Upgrade,
    Weapon(Weapon),
    Thruster,
    Trigger,
    Magnet,
    Drone,
    SideCannons,
    Insurance,
    EnergyCell,
    Bank,
    Radar,
    Amulet,
    Crystallizer,
    TreasureMap,
    Mercenary,
    MysteryCard,
    Autofire,
    LaserSight,
    Capacitor,
    MysteryBox,
    DamageCore,
}

impl Item {
    fn name(self) -> String {
        match self {
            Item::ExtraLife => "EXTRA LIFE".into(),
            Item::Bomb => "BOMB".into(),
            Item::Shield => "SHIELD".into(),
            Item::Upgrade => "WEAPON UPGRADE".into(),
            Item::Weapon(w) => format!("{}  {}", w.name(), "*".repeat(w.tier() as usize)),
            Item::Thruster => "THRUSTER".into(),
            Item::Trigger => "TRIGGER".into(),
            Item::Magnet => "MAGNET".into(),
            Item::Drone => "ATTACK DRONE".into(),
            Item::SideCannons => "SIDE CANNONS".into(),
            Item::Insurance => "WEAPON INSURANCE".into(),
            Item::EnergyCell => "ENERGY CELL".into(),
            Item::Bank => "STAR BANK".into(),
            Item::Radar => "CARD RADAR".into(),
            Item::Amulet => "AMULET".into(),
            Item::Crystallizer => "CRYSTALLIZER".into(),
            Item::TreasureMap => "TREASURE MAP".into(),
            Item::Mercenary => "MERCENARY".into(),
            Item::MysteryCard => "MYSTERY CARD".into(),
            Item::Autofire => "AUTOFIRE".into(),
            Item::LaserSight => "LASER SIGHT".into(),
            Item::Capacitor => "CAPACITOR".into(),
            Item::MysteryBox => "MYSTERY BOX".into(),
            Item::DamageCore => "DAMAGE CORE".into(),
        }
    }
    fn desc(self) -> &'static str {
        match self {
            Item::ExtraLife => "+1 reserve ship.",
            Item::Bomb => "+1 bomb (X key).",
            Item::Shield => "Absorbs one hit.",
            Item::Upgrade => "+1 level on the current weapon.",
            Item::Weapon(w) => w.desc(),
            Item::Thruster => "Ship 8% faster.",
            Item::Trigger => "Fires 10% faster.",
            Item::Magnet => "Pulls crystals from farther away.",
            Item::Drone => "A drone that fires with you.",
            Item::SideCannons => "Extra diagonal shots.",
            Item::Insurance => "Your next death won't cost a weapon level.",
            Item::EnergyCell => "Fills the ability and speeds up recharge.",
            Item::Bank => "Earns 15% of your crystals every sector.",
            Item::Radar => "Cards drop more often.",
            Item::Amulet => "A few more power-ups until the end of the run (max 2).",
            Item::Crystallizer => "Enemies drop more crystals.",
            Item::TreasureMap => "A Golden Scarab in the next sector.",
            Item::Mercenary => "A wingman fires at your side for 2 sectors.",
            Item::MysteryCard => "A random card for your collection.",
            Item::Autofire => "Hold fire to shoot nonstop.",
            Item::LaserSight => "A line shows where your shot will go.",
            Item::Capacitor => "Charged shot 40% faster and stronger.",
            Item::MysteryBox => "Who knows what's inside? It's almost always good.",
            Item::DamageCore => "+15% damage on every weapon. Stacks up to 6x.",
        }
    }
    fn icon(self) -> Icon {
        match self {
            Item::ExtraLife => Icon::Heart,
            Item::Bomb => Icon::Bomb,
            Item::Shield | Item::Insurance => Icon::Shield,
            Item::Upgrade | Item::Weapon(_) | Item::Amulet => Icon::Star,
            Item::Thruster | Item::Mercenary => Icon::Engine,
            Item::Trigger | Item::EnergyCell => Icon::Bolt,
            Item::Magnet => Icon::Magnet,
            Item::Drone | Item::SideCannons => Icon::Drone,
            Item::Bank => Icon::Bag,
            Item::Radar | Item::MysteryCard => Icon::Card,
            Item::Crystallizer | Item::TreasureMap => Icon::Crystal,
            Item::Autofire | Item::Capacitor => Icon::Bolt,
            Item::LaserSight => Icon::Target,
            Item::MysteryBox => Icon::Bag,
            Item::DamageCore => Icon::Star,
        }
    }
    fn price(self) -> u32 {
        match self {
            Item::ExtraLife => 95,
            Item::Bomb => 25,
            Item::Shield => 40,
            Item::Upgrade => 50,
            Item::Weapon(w) => [40, 80, 130, 190, 260][(w.tier() - 1) as usize],
            Item::Thruster => 35,
            Item::Trigger => 45,
            Item::Magnet => 25,
            Item::Drone => 80,
            Item::SideCannons => 60,
            Item::Insurance => 30,
            Item::EnergyCell => 40,
            Item::Bank => 60,
            Item::Radar => 70,
            Item::Amulet => 50,
            Item::Crystallizer => 55,
            Item::TreasureMap => 45,
            Item::Mercenary => 75,
            Item::MysteryCard => 90,
            Item::Autofire => 75,
            Item::LaserSight => 25,
            Item::Capacitor => 50,
            Item::MysteryBox => 20,
            Item::DamageCore => 70,
        }
    }
    /// Unique key (different weapons count separately).
    fn key(self) -> usize {
        match self {
            Item::Weapon(w) => 100 + w.index(),
            Item::ExtraLife => 0,
            Item::Bomb => 1,
            Item::Shield => 2,
            Item::Upgrade => 3,
            Item::Thruster => 5,
            Item::Trigger => 6,
            Item::Magnet => 7,
            Item::Drone => 8,
            Item::SideCannons => 9,
            Item::Insurance => 10,
            Item::EnergyCell => 11,
            Item::Bank => 12,
            Item::Radar => 13,
            Item::Amulet => 14,
            Item::Crystallizer => 15,
            Item::TreasureMap => 16,
            Item::Mercenary => 17,
            Item::MysteryCard => 18,
            Item::Autofire => 19,
            Item::LaserSight => 20,
            Item::Capacitor => 21,
            Item::MysteryBox => 22,
            Item::DamageCore => 23,
        }
    }
}

// ---------------------------------------------------------------------------
// Run configuration
// ---------------------------------------------------------------------------

pub struct RunConfig {
    pub mode: Mode,
    pub diff: Difficulty,
    pub ship: usize,
    pub skin: usize,
    /// Co-op: the ship flown by player two (chosen from player one's hangar).
    pub ship2: usize,
    pub daily: Option<Daily>,
    pub equipped: Vec<usize>,
    pub rank: usize,
    pub achs: Vec<bool>,
    pub owned_cards: Vec<u32>,
    pub highscore: u32,
    pub found_logs: Vec<bool>,
    pub trail: usize,
    pub show_hitbox: bool,
    pub coop: bool,
}

struct Stage {
    kind: WaveKind,
    theme: usize,
    tier: u32,
    diff: f32,
}

// ---------------------------------------------------------------------------
// Game
// ---------------------------------------------------------------------------

pub struct Game {
    pub time: f32,
    pub run_time: f32,
    pub phase: Phase,
    pub cfg: RunConfig,
    pub mods: Mods,
    pub player: Player,
    enemies: Vec<Enemy>,
    next_id: u32,
    bullets: Vec<PBullet>,
    pub ebullets: Vec<EBullet>,
    drops: Vec<Drop>,
    rocks: Vec<Rock>,
    strikes: Vec<Strike>,
    bolts: Vec<Bolt>,
    hazard_timer: f32,
    pub boss: Option<Boss>,
    boss_warning: f32,
    pub fx: Fx,
    pub bg: Background,
    pub score: u32,
    pub crystals: f32,
    pub wave: u32,
    pub cycle: u32,
    pub combo: u32,
    combo_t: f32,
    pub best_combo: u32,
    dive_timer: f32,
    wave_kind: WaveKind,
    wave_age: f32,
    wave_size: usize,
    challenge_total: u32,
    challenge_hits: u32,
    fired_in_bonus: bool,
    groups: Vec<(u32, u32)>,
    capsule_pity: u32,
    next_life_at: u32,
    hitstop: f32,
    sector_hits: u32,
    sector_shots: u32,
    sector_landed: u32,
    boss_hits: u32,
    banner: Option<(String, String, f32)>,
    drones: [Vec2; 3],
    drone_cd: f32,
    ally_pos: Vec2,
    ally_cd: f32,
    // shop
    shop_sel: usize,
    shop_offers: Vec<Item>,
    shop_buys: Vec<(usize, u32)>,
    rerolls: u32,
    free_reroll_used: bool,
    // purchased upgrades
    drones_bought: u32,
    side_cannons: bool,
    insurance: bool,
    bank: bool,
    crystalizer: bool,
    treasure_map: bool,
    ally_sectors: u32,
    grudge: bool,
    // power-ups and abilities
    frenzy_t: f32,
    star_t: f32,
    magnet_t: f32,
    slowmo_t: f32,
    web_t: f32,
    overdrive_t: f32,
    wall_t: f32,
    timewarp_t: f32,
    cannon_t: f32,
    nostalgia_t: f32,
    // secrets and events
    pub events: Vec<Event>,
    blasts: Vec<(Vec2, f32, f32, Option<Weapon>)>,
    pub vg: vanguard::Vanguard,
    pub lv: levels::LevelState,
    pub sig: Option<signal::Signal>,
    sig_done: bool,
    /// Development: start the Signal fight at this phase on the next update.
    pub dbg_signal: Option<u8>,
    pub sc: scenery::Scenery,
    pub co: coop::CoopState,
    pub frozen_count: u32,
    pub absorbed: u32,
    seen_run: Vec<bool>,
    mastery_run: Vec<bool>,
    golden_this_sector: bool,
    cow_this_run: bool,
    merchant_this_sector: bool,
    secret_stage: bool,
    secret_done: bool,
    pub deaths: u32,
    pub kills: u32,
    pub bosses_killed: u32,
    pub sectors_cleared: u32,
    pub grazes: u32,
    crystal_bomb_acc: f32,
    pub continues_used: u32,
    continues_left: u32,
    pub new_cards: Vec<usize>,
    pub pending_cards: Vec<usize>,
    pub finished: bool,
    pub victory: bool,
    credits_shown: bool,
    fog: Option<Texture2D>,
    intro_lock: bool,
    pub enc: encounters::Encounters,
    pub rift: rifts::Rifts,
    pub arcade: arcade::Arcade,
    /// Second ship in local co-op.
    pub p2: Option<Player>,
    /// Input for the second ship, set by the app before each update.
    pub p2_input: Input,
    /// True while `with_p2` has the second ship in the `player` slot.
    cur_p2: bool,
    /// True once the ships traded slots because player one ran out of lives.
    p_swapped: bool,
}

impl Game {
    pub fn new(cfg: RunConfig) -> Game {
        let classic = cfg.mode == Mode::Classic;
        let equipped = if classic {
            vec![]
        } else {
            cfg.equipped.clone()
        };
        let mut mods = Mods::from_cards(&equipped);
        let ship: &ShipDef = &SHIPS[cfg.ship];
        if cfg.ship == 8 {
            mods.crystal_mul *= 1.2;
        }
        let mut lives = 3 + mods.extra_lives + ship.lives + cfg.diff.lives();
        let mut weapon = Weapon::Blaster;
        let mut level = mods.start_level;
        if mods.second_laser {
            weapon = Weapon::Laser;
            level = level.max(2);
        }
        match cfg.daily {
            Some(Daily::ThinGlass) => lives = 1,
            Some(Daily::LaserOnly) => {
                weapon = Weapon::Laser;
                level = level.max(3);
            }
            _ => {}
        }
        let player = Player {
            pos: vec2(W / 2.0, H - 80.0),
            vel: Vec2::ZERO,
            lives: lives.max(1),
            bombs: 2 + mods.extra_bombs,
            weapon,
            level,
            cd: 0.0,
            missile_cd: 0.0,
            invuln: 2.0,
            dead_t: 0.0,
            shield: mods.start_shield || cfg.ship == 2,
            speed_lvl: 0,
            fire_lvl: 0,
            magnet_lvl: 0,
            shots: 0,
            phoenix_used: false,
            revived: false,
            tilt: 0.0,
            focus: false,
            meter: if mods.meter_full { 1.0 } else { 0.3 },
            trigger_t: 0.0,
            charge: 0.0,
        };
        let mut bg = Background::new();
        bg.set_theme(0);
        let pos = player.pos;
        let crystals = mods.start_crystals as f32;
        let fog = if cfg.daily == Some(Daily::Fog) {
            Some(make_fog())
        } else {
            None
        };
        let continues = if classic || cfg.mode == Mode::Survival {
            0
        } else {
            cfg.diff.continues() + mods.extra_continue
        };
        let mut g = Game {
            time: 0.0,
            run_time: 0.0,
            phase: Phase::Intro { t: 0.0 },
            mods,
            player,
            enemies: vec![],
            next_id: 1,
            bullets: vec![],
            ebullets: vec![],
            drops: vec![],
            rocks: vec![],
            strikes: vec![],
            bolts: vec![],
            hazard_timer: 4.0,
            boss: None,
            boss_warning: 0.0,
            fx: Fx::default(),
            bg,
            score: 0,
            crystals,
            wave: 1,
            cycle: 0,
            combo: 0,
            combo_t: 0.0,
            best_combo: 0,
            dive_timer: 3.0,
            wave_kind: WaveKind::Normal,
            wave_age: 0.0,
            wave_size: 1,
            challenge_total: 0,
            challenge_hits: 0,
            fired_in_bonus: false,
            groups: vec![],
            capsule_pity: 0,
            next_life_at: if classic { 20_000 } else { 30_000 },
            hitstop: 0.0,
            sector_hits: 0,
            sector_shots: 0,
            sector_landed: 0,
            boss_hits: 0,
            banner: None,
            drones: [pos, pos, pos],
            drone_cd: 0.0,
            ally_pos: pos,
            ally_cd: 0.0,
            shop_sel: 0,
            shop_offers: vec![],
            shop_buys: vec![],
            rerolls: 0,
            free_reroll_used: false,
            drones_bought: 0,
            side_cannons: false,
            insurance: false,
            bank: false,
            crystalizer: false,
            treasure_map: false,
            ally_sectors: 0,
            grudge: false,
            frenzy_t: 0.0,
            star_t: 0.0,
            magnet_t: 0.0,
            slowmo_t: 0.0,
            web_t: 0.0,
            overdrive_t: 0.0,
            wall_t: 0.0,
            timewarp_t: 0.0,
            cannon_t: 0.0,
            nostalgia_t: 0.0,
            events: vec![],
            blasts: vec![],
            vg: Default::default(),
            lv: Default::default(),
            sig: None,
            sig_done: false,
            dbg_signal: None,
            sc: Default::default(),
            co: Default::default(),
            frozen_count: 0,
            absorbed: 0,
            seen_run: vec![false; crate::progress::BESTIARY.len()],
            mastery_run: vec![false; WEAPONS.len()],
            golden_this_sector: false,
            cow_this_run: false,
            merchant_this_sector: false,
            secret_stage: false,
            secret_done: false,
            deaths: 0,
            kills: 0,
            bosses_killed: 0,
            sectors_cleared: 0,
            grazes: 0,
            crystal_bomb_acc: 0.0,
            continues_used: 0,
            continues_left: continues,
            new_cards: vec![],
            pending_cards: vec![],
            finished: false,
            victory: false,
            credits_shown: false,
            fog,
            intro_lock: true,
            enc: encounters::Encounters::new(&cfg),
            rift: rifts::Rifts::new(),
            arcade: arcade::Arcade::new(),
            p2: None,
            p2_input: Input::default(),
            cur_p2: false,
            p_swapped: false,
            cfg,
        };
        if g.cfg.coop {
            let mut p2 = g.player.clone();
            let s2 = &SHIPS[g.cfg.ship2.min(SHIPS.len() - 1)];
            p2.lives = (p2.lives - ship.lives + s2.lives).max(1);
            p2.shield = g.mods.start_shield || g.cfg.ship2 == 2;
            p2.pos.x = W / 2.0 + 60.0;
            g.player.pos.x = W / 2.0 - 60.0;
            g.p2 = Some(p2);
        }
        g
    }

    /// Starts a campaign at a later checkpoint. The ship gets the crystals and the weapon
    /// power a pilot would have gathered on the way, so the jump is fair but not free.
    pub fn start_at_checkpoint(&mut self, sector: u32) {
        self.wave = (sector - 1) * SUBS + 1;
        self.crystals += 45.0 * (sector - 1) as f32;
        let lv = (1 + sector as i32 / 5).min(4);
        self.player.level = lv;
        if let Some(p2) = &mut self.p2 {
            p2.level = lv;
        }
    }

    pub fn ship(&self) -> &'static ShipDef {
        &SHIPS[self.ship_idx()]
    }

    /// Hangar index of the ship currently in the `player` slot.
    pub fn ship_idx(&self) -> usize {
        self.seat_ship(self.seat())
    }

    /// Hangar index of the ship flown by a seat (0 = player one).
    pub fn seat_ship(&self, seat: usize) -> usize {
        if seat == 1 && self.coop() {
            self.cfg.ship2.min(SHIPS.len() - 1)
        } else {
            self.cfg.ship
        }
    }

    fn classic(&self) -> bool {
        self.cfg.mode == Mode::Classic
    }

    fn sectors_in_run(&self) -> u32 {
        match self.cfg.mode {
            Mode::Daily => 3,
            Mode::Expedition => u32::MAX,
            _ => SECTORS,
        }
    }

    fn stage(&self) -> Stage {
        if self.enc.void {
            return Stage {
                kind: WaveKind::Boss,
                theme: 9,
                tier: 9,
                diff: 7.0,
            };
        }
        let w = self.wave;
        match self.cfg.mode {
            Mode::Campaign | Mode::Daily | Mode::Expedition => {
                let sector = (w - 1) / 5 + 1;
                let sub = (w - 1) % 5 + 1;
                let kind = match sub {
                    4 => WaveKind::Challenge,
                    5 => WaveKind::Boss,
                    _ => WaveKind::Normal,
                };
                let kind = if self.secret_stage {
                    WaveKind::Golden
                } else {
                    kind
                };
                let level = (sector - 1) * SUBS + sub;
                let (theme, tier, diff) = match self.cfg.mode {
                    // 100 levels: the eight regions are spread over 20 sectors and the
                    // difficulty climbs every single level, steeply in the last quarter.
                    Mode::Campaign => {
                        let x = (level - 1) as f32 / 99.0;
                        (
                            ((sector - 1) * 8 / SECTORS) as usize,
                            (sector * 2).div_ceil(3) + self.cycle * 14,
                            0.5 + x * 8.0 + x * x * 3.5 + self.cycle as f32 * 4.0,
                        )
                    }
                    Mode::Daily => (
                        (sector - 1) as usize % 8,
                        sector * 2,
                        (sector - 1) as f32 * 1.6 + (sub - 1) as f32 * 0.2 + 1.0,
                    ),
                    _ => (
                        (sector - 1) as usize % 8,
                        sector + 1,
                        (sector - 1) as f32 * 0.8 + (sub - 1) as f32 * 0.15 + 0.6,
                    ),
                };
                Stage {
                    kind,
                    theme: if self.secret_stage { 8 } else { theme },
                    tier,
                    diff,
                }
            }
            Mode::Classic => {
                let block = (w - 1) / 3;
                Stage {
                    kind: if w.is_multiple_of(3) {
                        WaveKind::Challenge
                    } else {
                        WaveKind::Normal
                    },
                    theme: block as usize % 8,
                    tier: (block + 1).min(8),
                    diff: (w - 1) as f32 * 0.22,
                }
            }
            Mode::Survival => Stage {
                kind: if w.is_multiple_of(6) {
                    WaveKind::Boss
                } else {
                    WaveKind::Normal
                },
                theme: ((w - 1) / 6) as usize % 8,
                tier: ((w - 1) / 3 + 1).min(8),
                diff: (w - 1) as f32 * 0.3,
            },
            Mode::BossRush => Stage {
                kind: WaveKind::Boss,
                theme: (w - 1) as usize % 8,
                tier: w.min(8),
                diff: (w - 1) as f32 * 0.6,
            },
        }
    }

    pub fn sector(&self) -> u32 {
        match self.cfg.mode {
            Mode::Campaign | Mode::Daily | Mode::Expedition => (self.wave - 1) / 5 + 1,
            Mode::Classic => (self.wave - 1) / 3 + 1,
            Mode::Survival => (self.wave - 1) / 6 + 1,
            Mode::BossRush => self.wave,
        }
    }
    fn sub(&self) -> u32 {
        (self.wave - 1) % SUBS + 1
    }
    /// Campaign level, 1 to 100.
    pub fn level(&self) -> u32 {
        (self.sector() - 1) * SUBS + self.sub()
    }
    /// Extra hit points every enemy gets as the campaign goes on.
    fn armor(&self) -> i32 {
        (self.diff() / 2.2) as i32
    }
    fn diff(&self) -> f32 {
        self.stage().diff
    }
    pub fn absolute_sector(&self) -> u32 {
        match self.cfg.mode {
            Mode::Campaign | Mode::Daily | Mode::Expedition => self.cycle * SECTORS + self.sector(),
            _ => self.wave,
        }
    }
    pub fn multiplier(&self) -> u32 {
        (1 + self.combo / 10).min(self.mods.max_combo)
    }
    fn fire_rate_enemy(&self) -> f32 {
        let d = if self.cfg.daily == Some(Daily::DoubleShots) {
            2.0
        } else {
            1.0
        };
        self.cfg.diff.fire_rate() * d
    }
    fn bullet_mul(&self) -> f32 {
        self.mods.bullet_speed * self.cfg.diff.bullet_speed()
    }
    fn score_mul(&self) -> f32 {
        let daily = match self.cfg.daily {
            Some(Daily::FastEnemies) => 2.0,
            Some(Daily::ThinGlass) => 3.0,
            _ => 1.0,
        };
        self.mods.score_mul * self.cfg.diff.score_mul() * daily
    }
    fn hit_r(&self) -> f32 {
        self.ship().hit_r
    }
    pub fn effective_level(&self) -> i32 {
        if self.overdrive_t > 0.0 {
            (self.player.level + 1).min(5)
        } else {
            self.player.level
        }
    }

    fn slot(&self, col: usize, row: usize) -> Vec2 {
        if let Some(p) = self.level_slot(col, row) {
            return p;
        }
        let breath = if self.stage().tier >= 2 {
            1.0 + 0.07 * (self.time * 1.4).sin()
        } else {
            1.0
        };
        let cx = W / 2.0 + (self.time * 0.6).sin() * 30.0;
        vec2(
            cx + (col as f32 - (COLS as f32 - 1.0) / 2.0) * 44.0 * breath,
            100.0 + row as f32 * 38.0 * (1.0 + (breath - 1.0) * 0.5),
        )
    }

    fn add_score(&mut self, base: u32) -> u32 {
        let fever = if self.enc.fever_t > 0.0 { 2.0 } else { 1.0 };
        let pts = (base as f32 * self.multiplier() as f32 * self.score_mul() * fever) as u32;
        self.score += pts;
        pts
    }

    fn ach(&mut self, a: usize) {
        if !self.cfg.achs.get(a).copied().unwrap_or(true) {
            self.cfg.achs[a] = true;
            self.events.push(Event::Ach(a));
        }
    }

    fn seen(&mut self, lore: usize) {
        if lore < self.seen_run.len() && !self.seen_run[lore] {
            self.seen_run[lore] = true;
            self.events.push(Event::Seen(lore));
        }
    }

    fn weapon_pool(&self) -> Vec<Weapon> {
        if self.cfg.daily == Some(Daily::LaserOnly) {
            return vec![Weapon::Laser];
        }
        WEAPONS
            .iter()
            .copied()
            .filter(|w| {
                w.unlocked(self.cfg.rank, &self.cfg.achs)
                    || (self.mods.pulsar_card && *w == Weapon::Pulsar)
            })
            .collect()
    }
}

fn shield_blocks(e: &Enemy, from: Vec2) -> bool {
    if matches!(e.kind, Kind::Armored | Kind::Leader) {
        return vanguard::armor_blocks(e, from);
    }
    // The plate protects the front (from below) while the enemy faces down.
    e.shield > 0 && from.y > e.pos.y + 4.0 && (from.x - e.pos.x).abs() < 12.0 && e.rot.abs() < 0.9
}

fn make_fog() -> Texture2D {
    let n = 128usize;
    let mut bytes = vec![0u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let dx = (x as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / n as f32 * 2.0 - 1.0;
            let d = (dx * dx + dy * dy).sqrt();
            let a = ((d - 0.45) / 0.55).clamp(0.0, 1.0);
            let a = if d >= 1.0 {
                1.0
            } else {
                a * a * (3.0 - 2.0 * a)
            };
            bytes[(y * n + x) * 4 + 3] = (a * 240.0) as u8;
        }
    }
    let t = Texture2D::from_rgba8(n as u16, n as u16, &bytes);
    t.set_filter(FilterMode::Linear);
    t
}
