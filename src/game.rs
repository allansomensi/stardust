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
pub mod coop;
pub mod encounters;
pub mod minigame;
pub mod rifts;

pub const COLS: usize = 8;
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
        }
    }
    fn radius(self) -> f32 {
        match self {
            Kind::Mini => 9.0,
            Kind::Kamikaze => 11.0,
            Kind::Carrier | Kind::Cow => 17.0,
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
            Item::Radar => "Cards drop twice as often.",
            Item::Amulet => "More power-ups until the end of the run.",
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
            Item::ExtraLife => 70,
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
        if cfg.ship == 3 {
            mods.star_fan = true;
        }
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
        &SHIPS[self.cfg.ship]
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
                            x * 7.0 + x * x * 3.0 + self.cycle as f32 * 4.0,
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

    // -----------------------------------------------------------------------
    // Waves
    // -----------------------------------------------------------------------

    fn begin_sector(&mut self) {
        self.rift_begin_sector();
        self.sector_hits = 0;
        self.sector_shots = 0;
        self.sector_landed = 0;
        self.golden_this_sector = false;
        self.merchant_this_sector = false;
        if self.mods.bomb_per_sector && self.wave > 1 {
            self.player.bombs = (self.player.bombs + 1).min(9);
        }
        if self.mods.shield_per_sector {
            self.player.shield = true;
        }
        if self.mods.crown {
            self.frenzy_t = self.frenzy_t.max(10.0);
        }
        if self.bank && self.wave > 1 {
            let interest = (self.crystals * 0.15).floor();
            self.crystals += interest;
            if interest > 0.0 {
                self.fx.text(
                    vec2(W / 2.0, H * 0.55),
                    format!("BANK: +{interest} CRYSTALS"),
                    CYAN,
                    18.0,
                );
            }
        }
        self.enc_begin_sector();
        let th = self.stage().theme;
        self.bg.set_theme(th);
        self.rocks.clear();
        self.strikes.clear();
    }

    fn start_wave(&mut self, au: &mut Audio) {
        let st = self.stage();
        self.wave_kind = st.kind;
        self.wave_age = 0.0;
        self.dive_timer = 3.5;
        self.groups.clear();
        self.bg.set_theme(st.theme);
        if self.cfg.mode == Mode::Survival {
            self.events.push(Event::SurvivalWave(self.wave));
        }
        match st.kind {
            WaveKind::Normal => {
                self.build_normal_wave();
                self.wave_size = self.enemies.len().max(1);
                au.music(Some(if st.theme == 7 {
                    Music::Final
                } else {
                    Music::Stage
                }));
                self.maybe_specials();
                self.enc_wave_start();
                self.rift_wave_start();
                self.arcade_wave_start();
            }
            WaveKind::Challenge => {
                self.build_challenge(false);
                self.fired_in_bonus = false;
                self.banner = Some((
                    "BONUS STAGE".into(),
                    "They don't shoot. Hit them all!".into(),
                    3.0,
                ));
                au.music(Some(Music::Stage));
            }
            WaveKind::Golden => {
                self.build_challenge(true);
                self.fired_in_bonus = false;
                self.banner = Some((
                    "GOLDEN DIMENSION".into(),
                    "A treasure hidden between dimensions...".into(),
                    3.0,
                ));
                au.music(Some(Music::Shop));
            }
            WaveKind::Boss => {
                self.boss_warning = if self.cfg.mode == Mode::BossRush && self.wave == 1 {
                    2.0
                } else {
                    3.2
                };
                au.music(None);
                au.play(Sfx::Warning);
            }
        }
    }

    fn maybe_specials(&mut self) {
        if self.classic() {
            return;
        }
        let campaign = matches!(
            self.cfg.mode,
            Mode::Campaign | Mode::Daily | Mode::Expedition
        );
        let want_golden = (self.mods.golden_every
            || self.treasure_map
            || self.enc.event == Some(encounters::SEvent::GoldRush))
            && !self.golden_this_sector;
        if want_golden || gen_range(0, 100) < 6 {
            self.golden_this_sector = true;
            self.treasure_map = false;
            let s = if gen_range(0, 2) == 0 { -1.0 } else { 1.0 };
            let mx = |x: f32| if s < 0.0 { x } else { W - x };
            let path = Path::two(
                [
                    vec2(mx(-30.0), 70.0),
                    vec2(mx(W * 0.4), 40.0),
                    vec2(mx(W * 0.2), 200.0),
                    vec2(mx(W * 0.5), 140.0),
                ],
                [
                    vec2(mx(W * 0.5), 140.0),
                    vec2(mx(W * 0.8), 80.0),
                    vec2(mx(W * 0.6), 260.0),
                    vec2(mx(W + 40.0), 120.0),
                ],
            );
            self.spawn_special(Kind::Golden, path, 5.5, gen_range(4.0, 12.0));
        }
        if campaign && !self.cow_this_run && gen_range(0, 100) < 3 {
            self.cow_this_run = true;
            let path = Path::one(
                vec2(-40.0, H * 0.35),
                vec2(W * 0.3, H * 0.25),
                vec2(W * 0.7, H * 0.45),
                vec2(W + 40.0, H * 0.3),
            );
            self.spawn_special(Kind::Cow, path, 11.0, gen_range(6.0, 14.0));
        }
        if campaign
            && self.sector() >= 2
            && self.sub() == 2
            && !self.merchant_this_sector
            && gen_range(0, 100) < 40
        {
            self.merchant_this_sector = true;
            let path = Path::one(
                vec2(W + 50.0, 210.0),
                vec2(W * 0.66, 260.0),
                vec2(W * 0.33, 200.0),
                vec2(-50.0, 240.0),
            );
            self.spawn_special(Kind::Merchant, path, 9.0, gen_range(5.0, 9.0));
            self.banner = Some((
                "A MERCHANT APPROACHES".into(),
                "It looks peaceful...".into(),
                2.5,
            ));
        }
    }

    fn spawn_special(&mut self, kind: Kind, path: Path, dur: f32, delay: f32) {
        self.spawn(kind, 0, 0, EState::Waiting { delay, path, dur }, 97, false);
        if let Some(e) = self.enemies.last_mut() {
            e.harmless = true;
        }
    }

    fn spawn(
        &mut self,
        kind: Kind,
        col: usize,
        row: usize,
        state: EState,
        group: u32,
        bonus: bool,
    ) {
        let pos = match &state {
            EState::Waiting { path, .. } => path.eval(0.0, None),
            _ => vec2(W / 2.0, -40.0),
        };
        let hp = if bonus { 1 } else { kind.hp() + self.armor() };
        let shield = if kind == Kind::Shieldbearer && !bonus {
            3 + (self.diff() / 2.0) as i32
        } else {
            0
        };
        self.enemies.push(Enemy {
            id: self.next_id,
            kind,
            hp,
            shield,
            col,
            row,
            pos,
            rot: 0.0,
            state,
            flash: 0.0,
            seed: gen_range(0.0, 10.0),
            group,
            bonus,
            harmless: bonus,
            dead: false,
            entry_shot: gen_range(0.25, 0.8),
            spawn_cd: gen_range(4.0, 7.0),
            last_weapon: None,
            damaged: false,
            gifted: false,
            frozen: 0.0,
            saw_cd: 0.0,
            elite: false,
            revealed: false,
        });
        self.next_id += 1;
    }

    fn entry_path(template: u32, s: f32) -> Path {
        let mx = |x: f32| if s < 0.0 { x } else { W - x };
        let v = |x: f32, y: f32| vec2(mx(x), y);
        match template % 4 {
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
            _ => Path::one(
                v(W * 0.05, -40.0),
                v(W * 1.1, H * 0.3),
                v(-W * 0.1, H * 0.45),
                Vec2::ZERO,
            ),
        }
    }

    fn row_kind(&self, row: usize, col: usize) -> Kind {
        let s = self.stage().tier;
        if s > 8 && gen_range(0, 4) == 0 {
            let pool = [
                Kind::Wasp,
                Kind::Gunner,
                Kind::Splitter,
                Kind::Kamikaze,
                Kind::Shieldbearer,
                Kind::Weaver,
                Kind::Chameleon,
                Kind::Minelayer,
                Kind::Jumper,
            ];
            return pool[gen_range(0, pool.len())];
        }
        match row {
            0 => {
                if s >= 6 && (col == 3 || col == 4) {
                    Kind::Carrier
                } else {
                    Kind::Beetle
                }
            }
            1 => {
                if s >= 2 && (col == 1 || col == 6) {
                    Kind::Gunner
                } else if s >= 4 && (col == 3 || col == 4) {
                    Kind::Shieldbearer
                } else {
                    Kind::Wasp
                }
            }
            2 => {
                if s >= 6 && (col == 1 || col == 6) {
                    Kind::Minelayer
                } else if s >= 3 && (col == 0 || col == 2 || col == 5 || col == 7) {
                    Kind::Splitter
                } else if s >= 5 && (col == 3 || col == 4) {
                    Kind::Weaver
                } else {
                    Kind::Wasp
                }
            }
            3 => {
                if s >= 3 && (col == 3 || col == 4) {
                    Kind::Jumper
                } else if s >= 4 && (col == 0 || col == 7) {
                    Kind::Kamikaze
                } else if s >= 7 && (col == 2 || col == 5) {
                    Kind::Chameleon
                } else {
                    Kind::Drone
                }
            }
            _ => {
                if s >= 4 && (col == 3 || col == 4) {
                    Kind::Kamikaze
                } else if s >= 7 && (col == 1 || col == 6) {
                    Kind::Chameleon
                } else if s >= 5 && col.is_multiple_of(3) {
                    Kind::Gunner
                } else {
                    Kind::Drone
                }
            }
        }
    }

    fn build_normal_wave(&mut self) {
        let first = self.wave == 1 && self.cycle == 0;
        let rows = if first { 4 } else { 5 };
        let order: Vec<usize> = if rows == 4 {
            vec![1, 2, 0, 3]
        } else {
            vec![3, 1, 2, 0, 4]
        };
        let base_t = gen_range(0, 4) as u32;
        let fast = self.cfg.mode == Mode::Survival;
        for (gi, &row) in order.iter().enumerate() {
            let cols: Vec<usize> = if row == 0 {
                (2..6).collect()
            } else {
                (0..COLS).collect()
            };
            let template = base_t + gi as u32;
            let half = cols.len() / 2;
            for (ci, &col) in cols.iter().enumerate() {
                let left = ci < half;
                let s = if left { -1.0 } else { 1.0 };
                let idx = if left { half - 1 - ci } else { ci - half };
                let path = Self::entry_path(template, s);
                let gap = if fast { 1.1 } else { 1.9 };
                let delay = 0.6 + gi as f32 * gap + idx as f32 * 0.16;
                let kind = self.row_kind(row, col);
                self.spawn(
                    kind,
                    col,
                    row,
                    EState::Waiting {
                        delay,
                        path,
                        dur: 2.3,
                    },
                    gi as u32,
                    false,
                );
            }
        }
    }

    fn flyby_path(template: u32, s: f32) -> Path {
        let mx = |x: f32| if s < 0.0 { x } else { W - x };
        let v = |x: f32, y: f32| vec2(mx(x), y);
        match template % 4 {
            0 => Path::two(
                [
                    v(-30.0, H * 0.3),
                    v(W * 0.6, H * 0.05),
                    v(W * 0.95, H * 0.6),
                    v(W * 0.5, H * 0.62),
                ],
                [
                    v(W * 0.5, H * 0.62),
                    v(W * 0.05, H * 0.64),
                    v(W * 0.25, H * 0.05),
                    v(W + 40.0, H * 0.15),
                ],
            ),
            1 => Path::two(
                [
                    v(W * 0.5, -30.0),
                    v(W * 0.5 + 200.0, H * 0.3),
                    v(W * 0.5 - 200.0, H * 0.5),
                    v(W * 0.5, H * 0.6),
                ],
                [
                    v(W * 0.5, H * 0.6),
                    v(W * 0.5 + 200.0, H * 0.72),
                    v(W * 0.5 + 220.0, H * 0.2),
                    v(W + 40.0, -40.0),
                ],
            ),
            2 => Path::one(
                v(-30.0, H * 0.12),
                v(W * 0.5, H * 0.95),
                v(W * 0.5, -H * 0.1),
                v(W + 40.0, H * 0.7),
            ),
            _ => Path::two(
                [
                    v(-30.0, H * 0.5),
                    v(W * 0.3, H * 0.2),
                    v(W * 0.7, H * 0.2),
                    v(W * 0.8, H * 0.45),
                ],
                [
                    v(W * 0.8, H * 0.45),
                    v(W * 0.9, H * 0.7),
                    v(W * 0.2, H * 0.7),
                    v(-40.0, H * 0.25),
                ],
            ),
        }
    }

    fn build_challenge(&mut self, golden: bool) {
        let kinds = [
            Kind::Drone,
            Kind::Wasp,
            Kind::Kamikaze,
            Kind::Beetle,
            Kind::Splitter,
            Kind::Weaver,
        ];
        self.challenge_total = 0;
        self.challenge_hits = 0;
        let groups = if golden { 6 } else { 5 };
        for g in 0..groups {
            let s = if g % 2 == 0 { -1.0 } else { 1.0 };
            let path = Self::flyby_path(g + self.wave, s);
            for j in 0..8 {
                let delay = 0.8 + g as f32 * 2.6 + j as f32 * 0.14;
                let kind = if golden {
                    Kind::Golden
                } else {
                    kinds[(g as usize + self.wave as usize) % kinds.len()]
                };
                self.spawn(
                    kind,
                    0,
                    0,
                    EState::Waiting {
                        delay,
                        path,
                        dur: 3.6,
                    },
                    g,
                    true,
                );
                self.challenge_total += 1;
            }
            self.groups.push((8, 0));
        }
    }

    fn start_dive(&mut self, i: usize) {
        let p = self.player.pos;
        let diff = self.diff();
        let speed = if self.cfg.daily == Some(Daily::FastEnemies) {
            0.6
        } else {
            1.0
        };
        let e = &self.enemies[i];
        let pos = e.pos;
        let s = if pos.x < W / 2.0 { 1.0 } else { -1.0 };
        let (path, dur) = match e.kind {
            Kind::Kamikaze => (
                Path::one(
                    pos,
                    pos + vec2(0.0, -50.0),
                    vec2(p.x, p.y - 140.0),
                    vec2(p.x + (p.x - pos.x) * 0.3, H + 60.0),
                ),
                (1.3 - diff * 0.03).max(0.9),
            ),
            Kind::Gunner | Kind::Weaver => (
                Path::two(
                    [
                        pos,
                        pos + vec2(-s * 60.0, -40.0),
                        vec2(pos.x + s * 60.0, H * 0.3),
                        vec2(pos.x + s * 50.0, H * 0.42),
                    ],
                    [
                        vec2(pos.x + s * 50.0, H * 0.42),
                        vec2(pos.x + s * 40.0, H * 0.5),
                        vec2(p.x, H * 0.75),
                        vec2(p.x - s * 100.0, H + 60.0),
                    ],
                ),
                3.2,
            ),
            _ => {
                let a = pos + vec2(-s * 40.0, 110.0);
                (
                    Path::two(
                        [
                            pos,
                            pos + vec2(-s * 100.0, -90.0),
                            pos + vec2(-s * 160.0, 40.0),
                            a,
                        ],
                        [
                            a,
                            vec2(a.x + s * 60.0, H * 0.5),
                            vec2(p.x, H * 0.72),
                            vec2(p.x + s * 140.0, H + 60.0),
                        ],
                    ),
                    (2.6 - diff * 0.1).max(1.5),
                )
            }
        };
        let dur = dur * speed;
        let kind = e.kind;
        let col = e.col;
        let row = e.row;
        self.enemies[i].state = EState::Diving {
            t: 0.0,
            dur,
            path,
            shots: 0,
        };
        if kind == Kind::Beetle {
            let mut escorts = 0;
            for j in 0..self.enemies.len() {
                if escorts >= 2 {
                    break;
                }
                let o = &self.enemies[j];
                if o.row == row + 1
                    && (o.col as i32 - col as i32).abs() <= 1
                    && matches!(o.state, EState::Formation)
                    && !o.dead
                {
                    let off = vec2(if escorts == 0 { -30.0 } else { 30.0 }, 26.0);
                    let delta = o.pos - pos;
                    let mut p2 = path.offset(off);
                    p2.seg[0][0] = pos + delta;
                    self.enemies[j].state = EState::Diving {
                        t: 0.0,
                        dur,
                        path: p2,
                        shots: 0,
                    };
                    escorts += 1;
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Update
    // -----------------------------------------------------------------------

    pub fn update(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
        self.time += dt;
        self.bg.update(dt);
        self.fx.update(dt);
        for b in &mut self.bolts {
            b.life -= dt;
        }
        self.bolts.retain(|b| b.life > 0.0);
        if let Some(b) = &mut self.banner {
            b.2 -= dt;
            if b.2 <= 0.0 {
                self.banner = None;
            }
        }
        if self.hitstop > 0.0 {
            self.hitstop -= dt;
            return;
        }
        if self.rift_minigame(dt, inp, au) || self.rift_blessing(dt, inp, au) {
            return;
        }
        if !matches!(
            self.phase,
            Phase::Shop
                | Phase::Victory { .. }
                | Phase::Credits { .. }
                | Phase::GameOver { .. }
                | Phase::Continue { .. }
        ) {
            self.run_time += dt;
        }

        let phase = std::mem::replace(&mut self.phase, Phase::Playing);
        self.intro_lock = matches!(phase, Phase::Intro { t } if t < 1.5);
        let next = match phase {
            Phase::Intro { t } => {
                if t == 0.0 {
                    self.begin_sector();
                    let st = self.stage();
                    au.music(Some(if st.theme == 7 {
                        Music::Final
                    } else {
                        Music::Stage
                    }));
                    if self.wave == 1 && self.cycle == 0 {
                        self.player.pos = vec2(W / 2.0, H + 40.0);
                    }
                }
                let t = t + dt;
                self.bg.speed_mul = if t < 1.6 { 1.0 + (1.6 - t) * 4.0 } else { 1.0 };
                if self.player.pos.y > H - 80.0 && t < 1.5 {
                    self.player.pos.y -= 120.0 * dt;
                }
                self.update_world(dt, inp, au);
                if t > 2.8 && !self.room_active() {
                    self.bg.speed_mul = 1.0;
                    self.start_wave(au);
                    Phase::Playing
                } else {
                    Phase::Intro { t }
                }
            }
            Phase::Playing => {
                self.wave_age += dt;
                self.update_world(dt, inp, au);
                if self.room_active() {
                    Phase::Playing
                } else {
                    self.check_wave_end(au)
                }
            }
            Phase::WaveClear { t } => {
                self.update_world(dt, inp, au);
                let t = t + dt;
                if t > 1.6 && !self.room_active() {
                    self.wave += 1;
                    if self.cfg.mode == Mode::Classic && (self.wave - 1).is_multiple_of(3) {
                        Phase::Intro { t: 0.0 }
                    } else {
                        self.start_wave(au);
                        Phase::Playing
                    }
                } else {
                    Phase::WaveClear { t }
                }
            }
            Phase::ChallengeEnd {
                t,
                hits,
                total,
                bonus,
            } => {
                self.update_world(dt, inp, au);
                let t = t + dt;
                if t > 4.0 {
                    if self.secret_stage {
                        self.secret_stage = false;
                        self.secret_done = true;
                        Phase::Intro { t: 0.0 }
                    } else {
                        self.wave += 1;
                        if self.cfg.mode == Mode::Classic && (self.wave - 1).is_multiple_of(3) {
                            Phase::Intro { t: 0.0 }
                        } else {
                            self.start_wave(au);
                            Phase::Playing
                        }
                    }
                } else {
                    Phase::ChallengeEnd {
                        t,
                        hits,
                        total,
                        bonus,
                    }
                }
            }
            Phase::BossDeath { t } => {
                self.update_world(dt, inp, au);
                let t = t + dt;
                if t > 3.4 {
                    self.finish_boss(au);
                    if self.cfg.mode == Mode::Survival {
                        self.wave += 1;
                        self.start_wave(au);
                        Phase::Playing
                    } else {
                        let lines = self.sector_tally();
                        Phase::SectorClear {
                            t: 0.0,
                            lines,
                            shown: 0,
                        }
                    }
                } else {
                    Phase::BossDeath { t }
                }
            }
            Phase::SectorClear {
                t,
                lines,
                mut shown,
            } => {
                self.update_world(dt, inp, au);
                let t = t + dt;
                let want = ((t - 0.8) / 0.55).max(0.0) as usize;
                while shown < want.min(lines.len()) {
                    let (_, v, crystal) = &lines[shown];
                    if *crystal {
                        self.crystals += *v as f32;
                    } else {
                        self.score += *v;
                    }
                    au.play(Sfx::MenuOk);
                    shown += 1;
                }
                let done = shown >= lines.len() && t > 1.0 + lines.len() as f32 * 0.55 + 1.2;
                if done && (inp.enter || t > 7.0) {
                    self.after_sector(au)
                } else {
                    Phase::SectorClear { t, lines, shown }
                }
            }
            Phase::Shop => self.update_shop(inp, au),
            Phase::Portal { t } => {
                let t = t + dt;
                if t == dt {
                    au.play(Sfx::Portal);
                }
                self.bg.speed_mul = 1.0 + t * 3.0;
                if t > 3.5 {
                    self.bg.speed_mul = 1.0;
                    if !self.enc.void {
                        self.secret_stage = true;
                        self.ach(ach::DIMENSIONAL);
                    }
                    Phase::Intro { t: 0.0 }
                } else {
                    Phase::Portal { t }
                }
            }
            Phase::Credits { t } => {
                let t = t + dt;
                if (t > 4.0 && inp.confirm) || t > 46.0 {
                    au.music(Some(Music::Title));
                    Phase::Victory { sel: 0, t: 0.0 }
                } else {
                    Phase::Credits { t }
                }
            }
            Phase::Victory { mut sel, t } => {
                let t = t + dt;
                let can_continue = self.cfg.mode == Mode::Campaign;
                if can_continue && (inp.up_p || inp.down_p) {
                    sel = 1 - sel;
                    au.play(Sfx::MenuMove);
                }
                if t > 1.5 && inp.confirm {
                    au.play(Sfx::MenuOk);
                    if sel == 0 && can_continue {
                        self.cycle += 1;
                        self.wave = 1;
                        self.enc.void = false;
                        self.rocks.clear();
                        Phase::Intro { t: 0.0 }
                    } else {
                        self.finished = true;
                        Phase::Victory { sel, t }
                    }
                } else {
                    Phase::Victory { sel, t }
                }
            }
            Phase::Continue { t } => {
                let nt = t - dt;
                if inp.enter || (inp.confirm && nt < 9.5) {
                    au.play(Sfx::OneUp);
                    self.continues_left -= 1;
                    self.continues_used += 1;
                    self.score /= 2;
                    self.player.lives = 3;
                    self.player.level = 1;
                    self.player.bombs = self.player.bombs.max(2);
                    self.player.dead_t = 0.01;
                    self.coop_revive();
                    self.combo = 0;
                    au.music(Some(Music::Stage));
                    Phase::Playing
                } else if nt <= 0.0 || inp.back {
                    au.music(None);
                    au.play(Sfx::GameOver);
                    Phase::GameOver { t: 0.0 }
                } else {
                    if t.floor() != nt.floor() {
                        au.play(Sfx::Beep);
                    }
                    Phase::Continue { t: nt }
                }
            }
            Phase::GameOver { t } => {
                self.update_world(dt, inp, au);
                let t = t + dt;
                if t > 3.5 || (t > 1.2 && inp.confirm) {
                    self.finished = true;
                }
                Phase::GameOver { t }
            }
        };
        self.phase = next;
        self.arcade_post_update(au);

        let waiting = matches!(self.phase, Phase::GameOver { .. } | Phase::Continue { .. });
        self.coop_rotate();
        if !waiting && self.team_out() {
            au.music(None);
            if self.continues_left > 0 {
                self.phase = Phase::Continue { t: 10.0 };
            } else {
                au.play(Sfx::GameOver);
                self.phase = Phase::GameOver { t: 0.0 };
            }
        }
        if self.score > self.cfg.highscore {
            self.cfg.highscore = self.score;
        }
        if self.score >= 1_000_000 {
            self.ach(ach::MILLIONAIRE);
        }
        if self.crystals >= 300.0 {
            self.ach(ach::SCROOGE);
        }
        if self.score >= self.next_life_at {
            let classic = self.classic();
            self.next_life_at = match (classic, self.next_life_at) {
                (true, 20_000) => 70_000,
                (true, n) => n + 70_000,
                (false, n) if n < 100_000 => 100_000,
                (false, n) => n + 100_000,
            };
            if self.player.lives > 0 {
                self.player.lives = (self.player.lives + 1).min(9);
                au.play(Sfx::OneUp);
                self.fx.text(
                    self.player.pos - vec2(0.0, 40.0),
                    "EXTRA LIFE!",
                    GREEN,
                    22.0,
                );
            }
        }
    }

    fn after_sector(&mut self, au: &mut Audio) -> Phase {
        self.enc_sector_clear();
        if self.devour_ready() {
            self.sectors_cleared += 1;
            return self.enter_void(au);
        }
        self.sectors_cleared += 1;
        self.ally_sectors = self.ally_sectors.saturating_sub(1);
        let last = match self.cfg.mode {
            Mode::BossRush => self.wave >= 9,
            _ => self.sector() >= self.sectors_in_run(),
        };
        if last {
            self.collect_all_drops(au);
            self.victory = true;
            self.events.push(Event::Victory);
            if self.cfg.mode == Mode::Campaign {
                self.ach(ach::BROKEN_HEART);
                if self.cfg.diff as u8 >= Difficulty::Hard as u8 && self.continues_used == 0 {
                    self.ach(ach::TRUE_ENDING);
                }
                if self.cfg.diff == Difficulty::Nightmare {
                    self.ach(ach::NIGHTMARE);
                }
                if self.cycle == 0 {
                    self.give_card(cards::id::CROWN, au);
                }
            }
            if self.cfg.mode == Mode::BossRush && self.run_time < 12.0 * 60.0 {
                self.ach(ach::SPEEDRUNNER);
            }
            au.play(Sfx::Fanfare);
            if self.cfg.mode == Mode::Campaign && !self.credits_shown {
                self.credits_shown = true;
                au.music(Some(Music::Credits));
                return Phase::Credits { t: 0.0 };
            }
            au.music(Some(Music::Title));
            return Phase::Victory { sel: 0, t: 0.0 };
        }
        self.ebullets.clear();
        self.collect_all_drops(au);
        au.music(Some(Music::Shop));
        self.shop_sel = 0;
        self.rerolls = 0;
        self.free_reroll_used = false;
        if self.cfg.mode == Mode::BossRush {
            self.crystals += 50.0;
        }
        self.roll_offers();
        if self.cfg.mode == Mode::Expedition {
            self.offer_blessing();
        }
        Phase::Shop
    }

    fn check_wave_end(&mut self, au: &mut Audio) -> Phase {
        match self.wave_kind {
            WaveKind::Normal => {
                if self.cfg.mode == Mode::Survival {
                    let real = self.enemies.iter().filter(|e| !e.harmless).count();
                    if self.wave_age > 5.0 && real <= self.wave_size / 4 {
                        self.wave += 1;
                        self.start_wave(au);
                    }
                    return Phase::Playing;
                }
                if self.enemies.is_empty() && self.wave_age > 1.0 {
                    if self.wave == 1 {
                        self.ach(ach::FIRST_FLIGHT);
                    }
                    self.wave_speed_bonus();
                    Phase::WaveClear { t: 0.0 }
                } else {
                    Phase::Playing
                }
            }
            WaveKind::Challenge | WaveKind::Golden => {
                if self.enemies.is_empty() && self.wave_age > 1.0 {
                    let hits = self.challenge_hits;
                    let total = self.challenge_total;
                    let golden = self.wave_kind == WaveKind::Golden;
                    let mut bonus = hits * 100 * (1 + self.cycle) * self.mods.bonus_mul;
                    if hits == total {
                        bonus += 10_000 * (1 + self.cycle) * self.mods.bonus_mul;
                        au.play(Sfx::Fanfare);
                        self.ach(ach::PERFECTION);
                        if !golden {
                            self.offer_warp();
                        }
                        if !self.classic() {
                            let c = cards::roll(1);
                            self.spawn_drop(vec2(W / 2.0, 60.0), DropKind::Card(c));
                        }
                    } else {
                        au.play(Sfx::MenuOk);
                    }
                    if !self.fired_in_bonus && !golden {
                        self.ach(ach::PACIFIST);
                        bonus += 5000;
                        if !self.classic() {
                            self.spawn_drop(
                                vec2(W / 2.0, 40.0),
                                DropKind::Card(cards::id::PACIFIST),
                            );
                        }
                        self.banner = Some((
                            "PACIFIST".into(),
                            "Not a single shot fired. Respect.".into(),
                            3.0,
                        ));
                    }
                    if golden {
                        self.spawn_drop(vec2(W / 2.0, 50.0), DropKind::Card(cards::id::DIMENSION));
                        self.drop_log(8, vec2(W / 2.0 + 40.0, 50.0));
                    }
                    self.score += bonus;
                    Phase::ChallengeEnd {
                        t: 0.0,
                        hits,
                        total,
                        bonus,
                    }
                } else {
                    Phase::Playing
                }
            }
            WaveKind::Boss => {
                if let Some(b) = &self.boss
                    && b.hp <= 0.0
                {
                    return Phase::BossDeath { t: 0.0 };
                }
                Phase::Playing
            }
        }
    }

    fn sector_tally(&mut self) -> Vec<(String, u32, bool)> {
        let mut lines = vec![];
        let sec = self.absolute_sector();
        lines.push(("BOSS DEFEATED".to_string(), 20_000 * sec, false));
        if let Some(l) = self.boss_time_line() {
            lines.push(l);
        }
        if self.sector_hits == 0 {
            lines.push(("UNTOUCHABLE".to_string(), 15_000 * sec, false));
            self.ach(ach::UNTOUCHABLE);
        }
        let acc = if self.sector_shots > 0 {
            (self.sector_landed as f32 / self.sector_shots as f32 * 100.0).min(100.0)
        } else {
            0.0
        };
        lines.push((format!("ACCURACY {:.0}%", acc), (acc * 80.0) as u32, false));
        lines.push((
            format!("BEST COMBO {}", self.best_combo),
            self.best_combo * 50,
            false,
        ));
        lines.push(("BONUS CRYSTALS".to_string(), 15 + 8 * sec, true));
        lines
    }

    fn finish_boss(&mut self, au: &mut Audio) {
        if let Some(b) = self.boss.take() {
            self.fx
                .explosion(b.pos, Color::from_rgba(255, 200, 120, 255), true);
            self.fx.explosion(b.pos + vec2(30.0, 10.0), PINK, true);
            self.fx.flash = 1.0;
            self.fx.flash_color = WHITE;
            self.fx.shake = 1.0;
            au.play(Sfx::BigBoom);
            let base = 1 + (self.stage().tier.saturating_sub(1) / 3).min(3);
            for _ in 0..20 {
                self.spawn_drop(
                    b.pos + vec2(gen_range(-40.0, 40.0), gen_range(-20.0, 20.0)),
                    DropKind::Crystal(2 * base),
                );
            }
            let luck = if b.final_boss { 2 } else { 1 };
            if !self.classic() {
                let c = cards::roll(luck);
                self.spawn_drop(b.pos, DropKind::Card(c));
            }
            self.kills += 1;
            self.bosses_killed += 1;
            self.events.push(Event::BossKilled);
            let lore = b.lore_index();
            self.on_boss_killed(&b);
            self.seen(lore);
            let a = match b.kind {
                BossKind::Queen => ach::REGICIDE,
                BossKind::Warship => ach::SCRAPYARD,
                BossKind::Serpent => ach::SNAKE_CHARMER,
                BossKind::Weaver => ach::ARACHNO,
                BossKind::Eye => ach::DONT_BLINK,
                BossKind::Twins => ach::TWINS,
                BossKind::Devourer => ach::DEVOURER,
                BossKind::Comet => ach::COMET,
            };
            self.ach(a);
            if b.all_parts_destroyed() {
                self.ach(ach::DISMANTLE);
            }
            if self.boss_hits == 0 {
                self.ach(ach::NO_SCRATCH);
            }
        }
        self.ebullets.clear();
    }

    fn collect_all_drops(&mut self, au: &mut Audio) {
        let drops: Vec<Drop> = self.drops.drain(..).collect();
        for d in drops {
            self.collect(d, au);
        }
    }

    fn give_card(&mut self, c: usize, au: &mut Audio) {
        au.play(Sfx::CardGet);
        let new =
            self.cfg.owned_cards.get(c).copied().unwrap_or(0) == 0 && !self.new_cards.contains(&c);
        if new {
            self.new_cards.push(c);
        }
        self.pending_cards.push(c);
        let def = &cards::CARDS[c];
        let label = if new {
            format!("NEW CARD: {}", def.name)
        } else {
            format!("CARD: {}", def.name)
        };
        self.banner = Some((label, format!("{} - {}", def.rarity.name(), def.desc), 3.0));
        let pp = self.player.pos;
        self.fx.ring(pp, 260.0, 0.5, def.rarity.color(), 4.0);
        self.fx.burst(pp, def.rarity.color(), 30, 200.0);
    }

    // -----------------------------------------------------------------------
    // World
    // -----------------------------------------------------------------------

    fn update_world(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
        // Time warp and slow motion only affect the world, not the ship.
        let mut wdt = dt;
        if self.timewarp_t > 0.0 {
            wdt *= 0.3;
        }
        if self.slowmo_t > 0.0 {
            wdt *= 0.6;
        }
        self.update_timers(dt);
        let i2 = self.p2_input.clone();
        let (in1, in2) = if self.p_swapped {
            (&i2, inp)
        } else {
            (inp, &i2)
        };
        self.update_player(dt, in1, au);
        self.with_p2(|g| g.update_player(dt, in2, au));
        let pace = self.cfg.diff.pace();
        self.update_enemies(wdt * pace, au);
        self.update_boss(wdt * pace, au);
        self.update_hazards(wdt, au);
        self.update_bullets(dt, wdt);
        self.update_drops(dt, au);
        self.with_p2(|g| g.collect_near(au));
        self.update_abilities(dt, au);
        self.enc_update(dt, au);
        self.rift_update(dt, au);
        self.arcade_update(dt, au);
        self.collisions(au);
        self.rift_collisions(au);
        self.arcade_hazards(au);
        self.with_p2(|g| g.arcade_hazards(au));
        self.process_deaths(au);
        if self.combo_t > 0.0 {
            self.combo_t -= dt;
            if self.combo_t <= 0.0 {
                self.combo = 0;
            }
        }
    }

    fn update_timers(&mut self, dt: f32) {
        self.update_rails(dt);
        for t in [
            &mut self.frenzy_t,
            &mut self.star_t,
            &mut self.magnet_t,
            &mut self.slowmo_t,
            &mut self.web_t,
            &mut self.overdrive_t,
            &mut self.wall_t,
            &mut self.timewarp_t,
            &mut self.cannon_t,
            &mut self.nostalgia_t,
        ] {
            *t = (*t - dt).max(0.0);
        }
        if self.player.alive() && !self.classic() {
            self.player.meter = (self.player.meter + 0.01 * dt * self.mods.meter_mul).min(1.0);
        }
    }

    fn update_player(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
        let ship = self.ship();
        let ice = self.stage().theme == 4 && !self.secret_stage;
        let spawn_x = self.spawn_x();
        let lead = !self.cur_p2;
        let p = &mut self.player;
        p.cd -= dt;
        p.missile_cd -= dt;
        p.invuln = (p.invuln - dt).max(0.0);
        if p.dead_t > 0.0 {
            p.dead_t -= dt;
            if p.dead_t <= 0.0 && p.lives > 0 {
                p.pos = vec2(spawn_x, H - 80.0);
                p.vel = Vec2::ZERO;
                p.invuln = 2.5;
                p.tilt = 0.0;
            }
            return;
        }
        if p.lives <= 0 {
            return;
        }
        let mirror = (self.cfg.daily == Some(Daily::Mirror)) != (self.rift.mirror_t > 0.0);
        let (l, r) = if mirror {
            (inp.right, inp.left)
        } else {
            (inp.left, inp.right)
        };
        let mut d = vec2(0.0, 0.0);
        if l {
            d.x -= 1.0;
        }
        if r {
            d.x += 1.0;
        }
        if d.length() > 0.0 {
            d = d.normalize();
        }
        p.focus = inp.focus;
        let speed = 290.0
            * ship.speed
            * (1.0 + 0.08 * p.speed_lvl as f32)
            * self.mods.speed_mul
            * if inp.focus { 0.5 } else { 1.0 }
            * if self.web_t > 0.0 { 0.5 } else { 1.0 }
            * if self.rift.anchor_t > 0.0 { 0.55 } else { 1.0 };
        let intro = self.intro_lock;
        if !intro {
            let target = d * speed;
            if ice {
                p.vel += (target - p.vel) * (dt * 2.8).min(1.0);
            } else {
                p.vel = target;
            }
            p.pos += p.vel * dt;
            if p.pos.x < 18.0 || p.pos.x > W - 18.0 {
                p.vel.x = 0.0;
            }
            p.pos.x = p.pos.x.clamp(18.0, W - 18.0);
            // like the classics: the ship only moves sideways
            p.pos.y += (PLAYER_Y - p.pos.y) * (dt * 10.0).min(1.0);
        }
        p.tilt += (d.x - p.tilt) * (dt * 10.0).min(1.0);

        let pp = p.pos;
        let n_drones = if lead {
            self.drones_total() as usize
        } else {
            0
        };
        for (i, dr) in self.drones.iter_mut().enumerate().take(n_drones) {
            let off = match i {
                0 => vec2(-38.0, 14.0),
                1 => vec2(38.0, 14.0),
                _ => vec2(0.0, 36.0),
            };
            *dr += (pp + off - *dr) * (dt * 9.0).min(1.0);
        }
        let ally_target = vec2(
            pp.x + (self.time * 1.3).sin() * 70.0,
            (pp.y - 70.0).max(H * 0.5),
        );
        if lead {
            self.ally_pos += (ally_target - self.ally_pos) * (dt * 3.0).min(1.0);
        }

        if intro {
            return;
        }

        if inp.bomb && self.player.bombs > 0 {
            self.player.bombs -= 1;
            self.events.push(Event::Bomb);
            self.enc.bombs_sector += 1;
            self.detonate(au, false);
        }

        if inp.ability_p && !self.classic() {
            let cost = ship.ab_cost;
            if self.player.meter >= cost - 0.001 {
                self.player.meter -= cost;
                self.use_ability(d, au);
            } else {
                au.play_vol(Sfx::Deny, 0.4);
            }
        }

        let firing = self.trigger(inp, dt, au);
        if firing {
            if self.wave_kind == WaveKind::Challenge && self.player.cd <= 0.0 {
                self.fired_in_bonus = true;
            }
            self.fire(au);
        }
        if firing && n_drones > 0 {
            self.drone_cd -= dt;
            if self.drone_cd <= 0.0 {
                self.drone_cd = 0.28;
                for d in self.drones.iter().take(n_drones) {
                    self.bullets.push(PBullet::new(
                        *d - vec2(0.0, 8.0),
                        vec2(0.0, -600.0),
                        1.0,
                        BKind::Shot,
                        Weapon::Blaster,
                    ));
                }
            }
        }
        if self.ally_sectors > 0 && firing && lead {
            self.ally_cd -= dt;
            if self.ally_cd <= 0.0 {
                self.ally_cd = 0.2;
                for dx in [-6.0, 6.0] {
                    self.bullets.push(PBullet::new(
                        self.ally_pos + vec2(dx, -10.0),
                        vec2(0.0, -640.0),
                        1.0,
                        BKind::Shot,
                        Weapon::Blaster,
                    ));
                }
            }
        }
    }

    fn fire(&mut self, au: &mut Audio) {
        let ship = self.ship();
        let p = &self.player;
        if p.cd > 0.0 || self.rift.jam_t > 0.0 {
            return;
        }
        let daily_x2 = if self.cfg.daily == Some(Daily::DoubleShots) {
            2.0
        } else {
            1.0
        };
        let rate = self.mods.fire_mul
            * ship.fire
            * (1.0 + 0.1 * p.fire_lvl as f32)
            * if self.frenzy_t > 0.0 { 2.0 } else { 1.0 }
            * if self.overdrive_t > 0.0 { 2.0 } else { 1.0 }
            * daily_x2;
        let lv = self.effective_level();
        let dmg = ship.dmg * self.power_mul();
        let o = p.pos - vec2(0.0, 18.0);
        let pos = p.pos;
        let weapon = p.weapon;
        let shots_before = p.shots;
        let ss = self.mods.shot_speed;
        let mut out: Vec<PBullet> = vec![];
        let mk = |pos: Vec2, ang_deg: f32, speed: f32, dmg: f32, kind: BKind| {
            let a = ang_deg.to_radians();
            PBullet::new(pos, vec2(a.sin(), -a.cos()) * speed * ss, dmg, kind, weapon)
        };
        let mut cd = 0.14;
        match weapon {
            Weapon::Blaster => {
                let sp = 640.0;
                let angles: &[f32] = match lv {
                    1 => &[0.0],
                    2 => &[],
                    3 => &[-8.0, 0.0, 8.0],
                    4 => &[-11.0, 11.0],
                    _ => &[-18.0, -8.0, 0.0, 8.0, 18.0],
                };
                if lv == 2 || lv == 4 {
                    out.push(mk(o + vec2(-6.0, 0.0), 0.0, sp, dmg, BKind::Shot));
                    out.push(mk(o + vec2(6.0, 0.0), 0.0, sp, dmg, BKind::Shot));
                }
                for a in angles {
                    out.push(mk(o, *a, sp, dmg, BKind::Shot));
                }
                au.play_vol(Sfx::Blaster, 0.45);
            }
            Weapon::Laser => {
                cd = 0.11;
                let sp = 950.0;
                let spec: &[(f32, f32, f32)] = match lv {
                    1 => &[(0.0, 0.0, 1.0)],
                    2 => &[(-5.0, 0.0, 1.0), (5.0, 0.0, 1.0)],
                    3 => &[(-9.0, 0.0, 1.0), (0.0, 0.0, 1.0), (9.0, 0.0, 1.0)],
                    4 => &[(0.0, 0.0, 2.0), (-10.0, 0.0, 1.0), (10.0, 0.0, 1.0)],
                    _ => &[
                        (0.0, 0.0, 2.0),
                        (-10.0, 0.0, 2.0),
                        (10.0, 0.0, 2.0),
                        (-16.0, -4.0, 1.0),
                        (16.0, 4.0, 1.0),
                    ],
                };
                for (x, a, dm) in spec {
                    let mut b = mk(o + vec2(*x, x.abs() * 0.4), *a, sp, dm * dmg, BKind::Laser);
                    b.pierce = if lv >= 3 { 3 } else { 2 };
                    out.push(b);
                }
                au.play_vol(Sfx::Laser, 0.4);
            }
            Weapon::Missile => {
                cd = 0.16;
                out.push(mk(
                    o + vec2(if lv >= 3 { -5.0 } else { 0.0 }, 0.0),
                    0.0,
                    620.0,
                    dmg,
                    BKind::Shot,
                ));
                if lv >= 3 {
                    out.push(mk(o + vec2(5.0, 0.0), 0.0, 620.0, dmg, BKind::Shot));
                }
                if self.player.missile_cd <= 0.0 {
                    self.player.missile_cd = 0.42 / rate;
                    let n = [1, 2, 2, 3, 4][(lv - 1).clamp(0, 4) as usize];
                    for k in 0..n {
                        let side = if n == 1 {
                            if shots_before.is_multiple_of(2) {
                                -1.0
                            } else {
                                1.0
                            }
                        } else if k % 2 == 0 {
                            -1.0
                        } else {
                            1.0
                        };
                        let spread = 1.0 + (k / 2) as f32;
                        let mut b = PBullet::new(
                            pos + vec2(side * 12.0 * spread, 0.0),
                            vec2(side * 160.0 * spread, -120.0),
                            3.0 * dmg,
                            BKind::Missile,
                            weapon,
                        );
                        b.life = 3.0;
                        out.push(b);
                    }
                    au.play_vol(Sfx::Missile, 0.35);
                } else {
                    au.play_vol(Sfx::Blaster, 0.3);
                }
            }
            Weapon::Wave => {
                cd = 0.12;
                let n = match lv {
                    1 => 1,
                    2 | 3 => 2,
                    _ => 3,
                };
                for k in 0..n {
                    let mut b = mk(
                        o,
                        0.0,
                        480.0,
                        dmg * (1.0 + 0.12 * (lv - 1) as f32),
                        BKind::Wave,
                    );
                    b.phase = k as f32 * std::f32::consts::TAU / n as f32;
                    b.pierce = 1;
                    b.base_x = o.x;
                    out.push(b);
                }
                au.play_vol(Sfx::WaveShot, 0.4);
            }
            Weapon::Shrapnel => {
                cd = 0.36;
                let n = 4 + lv * 2;
                let spread = 34.0 + lv as f32 * 4.0;
                for k in 0..n {
                    let a =
                        -spread / 2.0 + spread * k as f32 / (n - 1) as f32 + gen_range(-2.0, 2.0);
                    let mut b = mk(o, a, gen_range(560.0, 720.0), dmg, BKind::Pellet);
                    b.life = 0.42 + lv as f32 * 0.03;
                    out.push(b);
                }
                au.play_vol(Sfx::Pellet, 0.5);
            }
            Weapon::Disc => {
                cd = 0.32;
                let n = match lv {
                    1 | 2 => 1,
                    3 | 4 => 2,
                    _ => 3,
                };
                for k in 0..n {
                    let a = (k as f32 - (n - 1) as f32 / 2.0) * 18.0;
                    let mut b = mk(o, a, 620.0, dmg * (1.0 + lv as f32 * 0.25), BKind::Disc);
                    b.pierce = 255;
                    b.life = 2.5;
                    out.push(b);
                }
                au.play_vol(Sfx::Disc, 0.4);
            }
            Weapon::Thunder => {
                cd = 0.17;
                self.lightning(lv, dmg, au);
            }
            Weapon::Flame => {
                cd = 0.035;
                let n = 1 + (lv as usize).div_ceil(2);
                let width = 10.0 + lv as f32 * 4.0;
                for _ in 0..n {
                    let a = gen_range(-width, width);
                    let mut b = mk(
                        o + vec2(gen_range(-4.0, 4.0), 0.0),
                        a,
                        gen_range(380.0, 470.0),
                        0.4 * dmg,
                        BKind::Flame,
                    );
                    b.life = 0.26 + lv as f32 * 0.035;
                    out.push(b);
                }
                if shots_before.is_multiple_of(4) {
                    au.play_vol(Sfx::Flame, 0.35);
                }
            }
            Weapon::Pulsar => {
                cd = 0.15;
                let n = (lv + 1).min(5) as usize;
                for k in 0..n {
                    let x = (k as f32 - (n - 1) as f32 / 2.0) * 9.0;
                    let mut b = mk(o + vec2(x, 0.0), x * 0.4, 560.0, 1.5 * dmg, BKind::Star);
                    b.phase = k as f32;
                    out.push(b);
                }
                au.play_vol(Sfx::WaveShot, 0.3);
            }
            Weapon::Fireball | Weapon::Plasma | Weapon::Railgun | Weapon::Meteor => {
                cd = arsenal::fire(weapon, lv, dmg, o, ss, &mut out, au);
            }
            other => {
                cd = encounters::fire_extra(
                    other,
                    lv,
                    dmg,
                    o,
                    ss,
                    &mut self.player.missile_cd,
                    rate,
                    &self.mods,
                    &mut out,
                    au,
                );
            }
        }
        if self.side_cannons && weapon != Weapon::Flame {
            for a in [-35.0, 35.0] {
                out.push(mk(pos, a, 560.0, dmg, BKind::Shot));
            }
        }
        let p = &mut self.player;
        p.cd = cd / rate;
        p.shots += 1;
        if self.mods.star_fan && p.shots.is_multiple_of(6) {
            for k in 0..6 {
                let a = -30.0 + k as f32 * 12.0;
                out.push(mk(o, a, 520.0, 2.0, BKind::Star));
            }
        }
        self.sector_shots += out
            .iter()
            .filter(|b| !matches!(b.kind, BKind::Missile | BKind::Flame))
            .count() as u32;
        self.bullets.extend(out);
    }

    fn lightning(&mut self, lv: i32, dmg: f32, au: &mut Audio) {
        let origin = self.player.pos - vec2(0.0, 18.0);
        let mut pts = vec![origin];
        let mut hit: Vec<u32> = vec![];
        let mut cur = origin;
        let chains = lv as usize + 1;
        let time = self.time;
        for step in 0..chains {
            let range = if step == 0 { 360.0 } else { 140.0 };
            let mut best: Option<(usize, f32)> = None;
            for (i, e) in self.enemies.iter().enumerate() {
                if !e.active()
                    || e.hp <= 0
                    || hit.contains(&e.id)
                    || e.pos.y < 0.0
                    || e.alpha(time) < 0.5
                {
                    continue;
                }
                if step == 0 && e.pos.y > origin.y {
                    continue;
                }
                let d = e.pos.distance(cur);
                if d < range && best.map(|(_, bd)| d < bd).unwrap_or(true) {
                    best = Some((i, d));
                }
            }
            let Some((i, _)) = best else { break };
            let e = &mut self.enemies[i];
            if shield_blocks(e, cur) {
                e.shield -= 1;
                e.flash = 0.07;
            } else {
                e.hp -= (1.3 * dmg).ceil() as i32;
                e.flash = 0.07;
                e.damaged = true;
                e.last_weapon = Some(Weapon::Thunder);
            }
            hit.push(e.id);
            cur = e.pos;
            pts.push(cur);
            self.sector_landed += 1;
        }
        if pts.len() == 1
            && let Some(b) = &mut self.boss
            && b.targetable()
            && b.pos.distance(origin) < 420.0
        {
            b.damage(crate::boss::HitTarget::Body, 1.2 * dmg);
            pts.push(b.pos);
        }
        if pts.len() == 1 {
            pts.push(origin - vec2(gen_range(-20.0, 20.0), 120.0));
        }
        let mut jag = vec![pts[0]];
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            for k in 1..6 {
                let t = k as f32 / 6.0;
                let n = vec2(-(b - a).y, (b - a).x).normalize_or_zero();
                jag.push(a.lerp(b, t) + n * gen_range(-9.0, 9.0));
            }
            jag.push(b);
        }
        self.bolts.push(Bolt {
            pts: jag,
            life: 0.09,
        });
        self.sector_shots += 1;
        au.play_vol(Sfx::Thunder, 0.12);
    }

    fn use_ability(&mut self, dir: Vec2, au: &mut Audio) {
        let ship = self.ship();
        let pp = self.player.pos;
        self.banner = Some((ship.ab_name.to_string(), String::new(), 1.2));
        match ship.ability {
            Ability::Overdrive => {
                self.overdrive_t = 5.0;
                au.play(Sfx::Ability);
                self.fx.ring(pp, 300.0, 0.4, GOLD, 4.0);
            }
            Ability::Dash => {
                let d = if dir.x.abs() > 0.0 {
                    vec2(dir.x.signum(), 0.0)
                } else if pp.x < W / 2.0 {
                    vec2(1.0, 0.0)
                } else {
                    vec2(-1.0, 0.0)
                };
                let from = pp;
                let to = pp + d * 150.0;
                self.player.pos = vec2(to.x.clamp(18.0, W - 18.0), pp.y);
                self.player.invuln = self.player.invuln.max(0.45);
                for k in 0..10 {
                    let p = from.lerp(self.player.pos, k as f32 / 10.0);
                    self.fx.sparks(p, -d, CYAN, 2);
                }
                au.play(Sfx::Dash);
                self.banner = None;
            }
            Ability::Bulwark => {
                self.wall_t = 5.0;
                au.play(Sfx::Shield);
            }
            Ability::StarPulse => {
                au.play(Sfx::Sparkle);
                self.fx.ring(pp, 500.0, 0.5, GOLD, 6.0);
                self.fx.ring(pp, 350.0, 0.6, WHITE, 3.0);
                self.ebullets.retain(|b| b.pos.distance(pp) > 180.0);
                for e in &mut self.enemies {
                    if e.active() && e.pos.distance(pp) < 220.0 && !e.harmless {
                        e.hp -= 3;
                        e.flash = 0.1;
                    }
                }
                if let Some(b) = &mut self.boss
                    && b.pos.distance(pp) < 300.0
                {
                    b.bomb_damage(12.0);
                }
            }
            Ability::TimeWarp => {
                self.timewarp_t = 5.0;
                au.play(Sfx::Portal);
            }
            Ability::Cannon => {
                self.cannon_t = 3.0;
                au.play(Sfx::Ability);
            }
            Ability::Aerogel | Ability::Loot | Ability::Reflect => {
                self.rift_ability(ship.ability, au)
            }
            Ability::Nostalgia => {
                self.nostalgia_t = 4.0;
                au.play(Sfx::Fanfare);
                for e in &mut self.enemies {
                    if e.active() && e.pos.y > 0.0 && !e.harmless {
                        e.hp -= 4;
                        e.flash = 0.2;
                    }
                }
                if let Some(b) = &mut self.boss {
                    b.bomb_damage(25.0);
                }
            }
        }
    }

    fn update_abilities(&mut self, dt: f32, au: &mut Audio) {
        let pp = self.player.pos;
        if self.wall_t > 0.0 && self.player.alive() {
            let wy = pp.y - 60.0;
            let mut blocked = 0;
            self.ebullets.retain(|b| {
                let hit = (b.pos.y - wy).abs() < 8.0 && (b.pos.x - pp.x).abs() < 70.0;
                if hit {
                    blocked += 1;
                }
                !hit
            });
            if blocked > 0 {
                self.fx.sparks(vec2(pp.x, wy), vec2(0.0, -1.0), CYAN, 3);
            }
        }
        if self.cannon_t > 0.0 && self.player.alive() {
            let x = pp.x;
            let ship_dmg = self.ship().dmg;
            let dmg = 9.0 * dt * ship_dmg;
            self.ebullets
                .retain(|b| !((b.pos.x - x).abs() < 22.0 && b.pos.y < pp.y));
            for e in &mut self.enemies {
                if e.active()
                    && !e.harmless
                    && (e.pos.x - x).abs() < 26.0
                    && e.pos.y < pp.y
                    && e.pos.y > -10.0
                    && gen_range(0.0, 1.0) < dmg
                {
                    e.hp -= 1;
                    e.shield = 0;
                    e.flash = 0.05;
                    e.damaged = true;
                }
            }
            if let Some(b) = &mut self.boss
                && b.targetable()
            {
                let mut y = pp.y - 20.0;
                while y > 0.0 {
                    if let Some(t) = b.hit_test(vec2(x, y)) {
                        b.damage(t, 14.0 * dt * ship_dmg);
                        break;
                    }
                    y -= 16.0;
                }
            }
            if gen_range(0, 3) == 0 {
                au.play_vol(Sfx::Flame, 0.25);
            }
        }
    }

    fn detonate(&mut self, au: &mut Audio, free: bool) {
        let p = self.player.pos;
        au.play(Sfx::Bomb);
        if self.mods.moo {
            au.play(Sfx::Moo);
        }
        self.fx.flash = 0.9;
        self.fx.flash_color = Color::new(1.0, 0.95, 0.8, 1.0);
        self.fx.shake = 0.8;
        self.fx
            .ring(p, 700.0, 1.0, Color::new(1.0, 0.9, 0.5, 1.0), 6.0);
        self.fx
            .ring(p, 500.0, 1.2, Color::new(1.0, 0.5, 0.8, 1.0), 3.0);
        if !free {
            self.player.invuln = self.player.invuln.max(1.2);
        }
        let bullets: Vec<Vec2> = self.ebullets.drain(..).map(|b| b.pos).collect();
        for b in bullets {
            self.fx.sparks(b, vec2(0.0, -1.0), GOLD, 2);
            self.score += 10;
        }
        let dmg = (4.0 * self.mods.bomb_power) as i32;
        for e in &mut self.enemies {
            if e.active() && e.pos.y > -20.0 && e.pos.y < H && !e.harmless {
                e.hp -= dmg;
                e.shield = 0;
                e.flash = 0.15;
            }
        }
        if let Some(b) = &mut self.boss {
            b.bomb_damage(30.0 * self.mods.bomb_power);
        }
        for r in &mut self.rocks {
            if !r.solid {
                r.hp -= dmg;
            }
        }
        self.strikes.clear();
    }

    fn update_enemies(&mut self, dt: f32, au: &mut Audio) {
        let diff = self.diff();
        let player = self.player.pos;
        let alive = self.player.alive();
        let bspeed = (185.0 + diff * 14.0) * self.bullet_mul();
        let mut new_bullets: Vec<EBullet> = vec![];
        let mut spawns: Vec<(Vec2, Vec2)> = vec![];
        let mut fired = false;
        let dive_mul = self.fire_rate_enemy().sqrt();

        if self.wave_kind == WaveKind::Normal && alive && self.wave_age > 3.0 {
            self.dive_timer -= dt;
            let remaining = self.enemies.len();
            if self.dive_timer <= 0.0 {
                let mut t = gen_range(1.4, 2.8) / (1.0 + diff * 0.18) / dive_mul;
                if remaining < 8 {
                    t *= 0.5;
                }
                self.dive_timer = t;
                let diving = self
                    .enemies
                    .iter()
                    .filter(|e| matches!(e.state, EState::Diving { .. }))
                    .count();
                if diving < 2 + diff as usize {
                    let candidates: Vec<usize> = self
                        .enemies
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| {
                            matches!(e.state, EState::Formation)
                                && !e.dead
                                && e.kind != Kind::Carrier
                        })
                        .map(|(i, _)| i)
                        .collect();
                    if !candidates.is_empty() {
                        let i = candidates[gen_range(0, candidates.len())];
                        self.start_dive(i);
                    }
                }
            }
        }

        let slots: Vec<Vec2> = self
            .enemies
            .iter()
            .map(|e| self.slot(e.col, e.row))
            .collect();
        let formation_rate = 0.018 * (1.0 + diff * 0.3) * self.fire_rate_enemy();
        let can_shoot = alive
            && ((self.wave_age > 2.5 && self.wave_kind == WaveKind::Normal)
                || self.wave_kind == WaveKind::Boss);
        let aim = |from: Vec2, speed: f32| -> Vec2 { (player - from).normalize_or_zero() * speed };

        for (i, e) in self.enemies.iter_mut().enumerate() {
            e.flash = (e.flash - dt).max(0.0);
            e.saw_cd = (e.saw_cd - dt).max(0.0);
            if e.frozen > 0.0 {
                e.frozen -= dt;
                if !matches!(e.state, EState::Waiting { .. }) {
                    continue;
                }
            }
            if matches!(e.kind, Kind::Mimic | Kind::Thief) {
                continue;
            }
            let prev = e.pos;
            let slot = slots[i];
            match e.state {
                EState::Waiting { delay, path, dur } => {
                    let delay = delay - dt;
                    e.state = if delay <= 0.0 {
                        e.pos = path.eval(0.0, Some(slot));
                        if e.harmless {
                            EState::Flyby { t: 0.0, dur, path }
                        } else {
                            EState::Entering { t: 0.0, path, dur }
                        }
                    } else {
                        EState::Waiting { delay, path, dur }
                    };
                }
                EState::Entering { t, path, dur } => {
                    let t = (t + dt / dur).min(1.0);
                    e.pos = path.eval(t, Some(slot));
                    if diff >= 2.0
                        && can_shoot
                        && t > e.entry_shot
                        && t - dt / dur <= e.entry_shot
                        && gen_range(0, 3) == 0
                    {
                        new_bullets.push(EBullet::new(e.pos, aim(e.pos, bspeed), 4.0, 0));
                        fired = true;
                    }
                    e.state = if t >= 1.0 {
                        EState::Formation
                    } else {
                        EState::Entering { t, path, dur }
                    };
                }
                EState::Formation => {
                    e.pos = slot;
                    if e.kind == Kind::Carrier && can_shoot {
                        e.spawn_cd -= dt;
                        if e.spawn_cd <= 0.0 {
                            e.spawn_cd = gen_range(5.0, 8.0);
                            spawns.push((
                                e.pos + vec2(0.0, 18.0),
                                vec2(gen_range(-80.0, 80.0), 90.0),
                            ));
                        }
                    }
                    if can_shoot && gen_range(0.0, 1.0) < formation_rate * dt {
                        match e.kind {
                            Kind::Gunner => {
                                for a in [-0.25f32, 0.0, 0.25] {
                                    new_bullets.push(EBullet::new(
                                        e.pos + vec2(0.0, 10.0),
                                        vec2(a.sin(), a.cos()) * bspeed,
                                        4.0,
                                        1,
                                    ));
                                }
                            }
                            Kind::Weaver => new_bullets.push(EBullet::new(
                                e.pos + vec2(0.0, 10.0),
                                aim(e.pos, bspeed * 0.6),
                                8.0,
                                3,
                            )),
                            Kind::Carrier => {}
                            _ => new_bullets.push(EBullet::new(
                                e.pos + vec2(0.0, 10.0),
                                vec2(0.0, bspeed),
                                4.0,
                                0,
                            )),
                        }
                        fired = true;
                    }
                }
                EState::Diving {
                    t,
                    dur,
                    path,
                    mut shots,
                } => {
                    let t = t + dt / dur;
                    e.pos = path.eval(t.min(1.0), None);
                    if can_shoot {
                        let marks: &[f32] = match e.kind {
                            Kind::Kamikaze => &[],
                            Kind::Gunner | Kind::Weaver => &[0.4, 0.5],
                            _ => {
                                if diff >= 2.0 {
                                    &[0.3, 0.55]
                                } else {
                                    &[0.32]
                                }
                            }
                        };
                        if (shots as usize) < marks.len() && t > marks[shots as usize] {
                            shots += 1;
                            match e.kind {
                                Kind::Gunner => {
                                    let base = (player - e.pos).normalize_or_zero();
                                    let ang = base.y.atan2(base.x);
                                    for k in -2..=2 {
                                        let a = ang + k as f32 * 0.2;
                                        new_bullets.push(EBullet::new(
                                            e.pos,
                                            vec2(a.cos(), a.sin()) * bspeed * 0.9,
                                            4.0,
                                            1,
                                        ));
                                    }
                                }
                                Kind::Weaver => new_bullets.push(EBullet::new(
                                    e.pos,
                                    aim(e.pos, bspeed * 0.6),
                                    8.0,
                                    3,
                                )),
                                _ => new_bullets.push(EBullet::new(
                                    e.pos,
                                    aim(e.pos, bspeed * 1.15),
                                    4.0,
                                    0,
                                )),
                            }
                            fired = true;
                        }
                    }
                    if t >= 1.0 {
                        e.pos = vec2(slot.x, -30.0);
                        e.state = EState::Returning;
                    } else {
                        e.state = EState::Diving {
                            t,
                            dur,
                            path,
                            shots,
                        };
                    }
                }
                EState::Returning => {
                    let to = slot - e.pos;
                    let step = 260.0 * dt;
                    if to.length() <= step {
                        e.pos = slot;
                        e.state = EState::Formation;
                    } else {
                        e.pos += to.normalize() * step;
                    }
                }
                EState::Flyby { t, dur, path } => {
                    let t = t + dt / dur;
                    e.pos = path.eval(t.min(1.0), None);
                    if t >= 1.0 {
                        e.dead = true;
                        e.hp = e.hp.max(1);
                    } else {
                        e.state = EState::Flyby { t, dur, path };
                    }
                }
                EState::Free { mut vel } => {
                    let dx = player.x - e.pos.x;
                    vel.x += dx.signum() * 120.0 * dt;
                    vel.x = vel.x.clamp(-180.0, 180.0);
                    vel.y = (vel.y + 160.0 * dt).min(330.0);
                    e.pos += vel * dt;
                    if e.pos.y > H + 40.0 {
                        e.dead = true;
                        e.hp = e.hp.max(1);
                    } else {
                        e.state = EState::Free { vel };
                    }
                }
            }
            let v = e.pos - prev;
            let keep_up = matches!(e.state, EState::Formation)
                || v.length() < 0.05
                || matches!(e.kind, Kind::Cow | Kind::Merchant);
            let target = if keep_up {
                0.0
            } else {
                v.y.atan2(v.x) - std::f32::consts::FRAC_PI_2
            };
            let mut delta = target - e.rot;
            while delta > std::f32::consts::PI {
                delta -= std::f32::consts::TAU;
            }
            while delta < -std::f32::consts::PI {
                delta += std::f32::consts::TAU;
            }
            e.rot += delta * (dt * 10.0).min(1.0);
        }
        if fired {
            au.play_vol(Sfx::EnemyShot, 0.5);
        }
        self.ebullets.extend(new_bullets);
        let count = self.enemies.len();
        for (pos, vel) in spawns {
            if count < 60 {
                self.spawn(Kind::Mini, 0, 0, EState::Free { vel }, 98, false);
                if let Some(m) = self.enemies.last_mut() {
                    m.pos = pos;
                }
            }
        }
    }

    fn update_boss(&mut self, dt: f32, au: &mut Audio) {
        if self.boss_warning > 0.0 {
            self.boss_warning -= dt;
            if self.boss_warning <= 0.0 {
                self.spawn_boss(au);
            }
        }
        let bmul = self.bullet_mul();
        let Some(boss) = &mut self.boss else { return };
        let mut out = BossOut::default();
        let target = if self.player.alive() {
            self.player.pos
        } else {
            vec2(W / 2.0, H)
        };
        boss.update(dt, target, bmul, &mut out);
        let minions = self.enemies.len();
        for (kind, pos, vel) in out.spawns {
            if minions < 12 {
                self.spawn(kind, 0, 0, EState::Free { vel }, 99, false);
                if let Some(e) = self.enemies.last_mut() {
                    e.pos = pos;
                }
            }
        }
        self.ebullets.extend(out.bullets);
        for s in out.sfx {
            au.play(s);
        }
        for (p, big) in out.explosions {
            self.fx
                .explosion(p, Color::from_rgba(255, 170, 80, 255), big);
            self.fx.shake = self.fx.shake.max(0.3);
        }
    }

    fn spawn_boss(&mut self, au: &mut Audio) {
        self.boss_hits = 0;
        self.arcade.boss_t = 0.0;
        let (kind, variant, final_boss, lvl) = match self.cfg.mode {
            Mode::Expedition => {
                let sec = self.sector();
                let list = [
                    BossKind::Queen,
                    BossKind::Warship,
                    BossKind::Serpent,
                    BossKind::Weaver,
                    BossKind::Comet,
                    BossKind::Eye,
                    BossKind::Twins,
                ];
                let k = list[(sec as usize - 1) % list.len()];
                (k, sec > 7, false, 1.0 + 0.28 * (sec - 1) as f32)
            }
            Mode::Campaign | Mode::Daily => {
                let sec = self.sector();
                let third = if gen_range(0, 2) == 0 {
                    BossKind::Serpent
                } else {
                    BossKind::Comet
                };
                let list = [
                    (BossKind::Queen, false),
                    (BossKind::Warship, false),
                    (third, false),
                    (BossKind::Weaver, false),
                    (BossKind::Eye, false),
                    (BossKind::Warship, true),
                    (BossKind::Twins, false),
                    (BossKind::Queen, false),
                ];
                let campaign = self.cfg.mode == Mode::Campaign;
                let theme = self.stage().theme;
                // The region's own boss waits at the end of each region; the sectors
                // before it are guarded by the other bosses in rotation.
                let region_end = !campaign || ((sec * 8) / SECTORS) as usize != theme;
                let (k, v) = if region_end {
                    list[if campaign {
                        theme
                    } else {
                        (sec as usize - 1) % 8
                    }]
                } else {
                    list[(sec as usize * 3 + 1) % 7]
                };
                let fin = campaign && sec == SECTORS;
                let lvl = if campaign {
                    1.0 + 0.16 * (sec - 1) as f32 + self.cycle as f32 * 1.2
                } else {
                    1.0 + 0.3 * (sec - 1) as f32
                };
                (k, v, fin, lvl)
            }
            Mode::BossRush => {
                let list = [
                    (BossKind::Queen, false, false),
                    (BossKind::Warship, false, false),
                    (BossKind::Serpent, false, false),
                    (BossKind::Weaver, false, false),
                    (BossKind::Eye, false, false),
                    (BossKind::Warship, true, false),
                    (BossKind::Comet, false, false),
                    (BossKind::Twins, false, false),
                    (BossKind::Queen, false, true),
                ];
                let (k, v, f) = list[(self.wave as usize - 1).min(8)];
                (k, v, f, 1.0 + 0.3 * (self.wave - 1) as f32)
            }
            _ => {
                let kinds = [
                    BossKind::Queen,
                    BossKind::Warship,
                    BossKind::Serpent,
                    BossKind::Weaver,
                    BossKind::Eye,
                    BossKind::Twins,
                    BossKind::Comet,
                ];
                let k = kinds[((self.wave / 6) as usize + gen_range(0, 5)) % kinds.len()];
                (k, self.wave > 30, false, 1.0 + self.wave as f32 * 0.08)
            }
        };
        let (kind, variant, final_boss, lvl) = if self.enc.void {
            (BossKind::Devourer, false, false, 3.0)
        } else {
            (kind, variant, final_boss, lvl)
        };
        let mut boss = Boss::new(kind, lvl, final_boss, variant);
        if crate::dev_env("STARDUST_EASYBOSS").is_some() {
            boss.hp *= 0.05;
            boss.max_hp *= 0.05;
        }
        self.boss = Some(boss);
        au.music(Some(if final_boss || variant || self.enc.void {
            Music::Final
        } else {
            Music::Boss
        }));
    }

    fn update_hazards(&mut self, dt: f32, au: &mut Audio) {
        let playing = matches!(self.phase, Phase::Playing)
            && self.wave_kind == WaveKind::Normal
            && !self.classic();
        let theme = self.stage().theme;
        if playing && !self.secret_stage {
            self.hazard_timer -= dt;
            if self.hazard_timer <= 0.0 {
                match theme {
                    1 => {
                        self.hazard_timer = gen_range(3.0, 6.0);
                        self.rocks.push(Rock {
                            pos: vec2(gen_range(40.0, W - 40.0), -30.0),
                            vel: vec2(gen_range(-40.0, 40.0), gen_range(70.0, 120.0)),
                            hp: 6,
                            big: true,
                            solid: false,
                            rot: 0.0,
                            vr: gen_range(-2.0, 2.0),
                            variant: gen_range(0, 2),
                            flash: 0.0,
                        });
                    }
                    5 => {
                        self.hazard_timer = gen_range(3.0, 5.0);
                        let x = (self.player.pos.x + gen_range(-70.0, 70.0)).clamp(20.0, W - 20.0);
                        self.strikes.push(Strike { x, t: 0.0 });
                        au.play_vol(Sfx::Beep, 0.5);
                    }
                    6 => {
                        self.hazard_timer = gen_range(4.0, 6.5);
                        self.rocks.push(Rock {
                            pos: vec2(gen_range(40.0, W - 40.0), -50.0),
                            vel: vec2(0.0, gen_range(60.0, 90.0)),
                            hp: 9999,
                            big: true,
                            solid: true,
                            rot: 0.0,
                            vr: 0.0,
                            variant: 0,
                            flash: 0.0,
                        });
                    }
                    _ => self.hazard_timer = 5.0,
                }
            }
        }
        for r in &mut self.rocks {
            r.pos += r.vel * dt;
            r.rot += r.vr * dt;
            r.flash = (r.flash - dt).max(0.0);
        }
        for s in &mut self.strikes {
            let before = s.t;
            s.t += dt;
            if before < 1.0 && s.t >= 1.0 {
                au.play_vol(Sfx::Thunder, 0.7);
                self.fx.shake = self.fx.shake.max(0.25);
            }
        }
        self.strikes.retain(|s| s.t < 1.3);
    }

    fn update_bullets(&mut self, dt: f32, wdt: f32) {
        let targets: Vec<Vec2> = self
            .enemies
            .iter()
            .filter(|e| e.active() && e.pos.y > 0.0 && !e.harmless)
            .map(|e| e.pos)
            .collect();
        let boss_pos = self.boss.as_ref().filter(|b| b.targetable()).map(|b| b.pos);
        let pp = self.player.pos;
        for b in &mut self.bullets {
            b.t += dt;
            match b.kind {
                BKind::Missile => {
                    let mut best: Option<Vec2> = boss_pos;
                    let mut bd = boss_pos.map(|p| p.distance(b.pos)).unwrap_or(f32::MAX);
                    for t in &targets {
                        let d = t.distance(b.pos);
                        if d < bd && t.y < b.pos.y + 40.0 {
                            bd = d;
                            best = Some(*t);
                        }
                    }
                    let speed = (b.vel.length() + 700.0 * dt).min(560.0);
                    let desired = best
                        .map(|t| (t - b.pos).normalize_or_zero())
                        .unwrap_or(vec2(0.0, -1.0));
                    let cur = b.vel.normalize_or_zero();
                    b.vel = (cur + desired * 6.0 * dt).normalize_or_zero() * speed;
                    if gen_range(0, 2) == 0 {
                        self.fx.particles.push(crate::fx::Particle {
                            pos: b.pos,
                            vel: -b.vel * 0.1,
                            life: 0.25,
                            max: 0.25,
                            color: Color::new(1.0, 0.6, 0.3, 0.8),
                            size: 2.0,
                            drag: 3.0,
                        });
                    }
                    b.pos += b.vel * dt;
                }
                BKind::Wave => {
                    b.pos.y += b.vel.y * dt;
                    b.pos.x =
                        b.base_x + (b.t * 11.0 + b.phase).sin() * (18.0 + b.t * 40.0).min(46.0);
                }
                BKind::Disc => {
                    if b.t < 0.35 {
                        b.vel *= 1.0 - (dt * 2.5).min(1.0);
                    } else {
                        let to = pp - b.pos;
                        b.vel = to.normalize_or_zero() * 620.0;
                        if to.length() < 22.0 && b.t > 0.5 {
                            b.life = 0.0;
                        }
                        if b.hit.iter().all(|h| *h != 0) {
                            b.hit = [0; 6];
                        }
                    }
                    b.pos += b.vel * dt;
                }
                BKind::Pellet | BKind::Flame => {
                    b.vel *= 1.0 - (dt * 1.5).min(1.0);
                    b.pos += b.vel * dt;
                }
                BKind::Star => {
                    b.pos += b.vel * dt;
                    b.pos.x += (b.t * 14.0 + b.phase).sin() * 40.0 * dt;
                }
                BKind::Bounce
                | BKind::Grenade
                | BKind::Bee
                | BKind::Hole
                | BKind::Cow
                | BKind::Ice => encounters::move_bullet(b, dt, &targets, boss_pos),
                BKind::Needle
                | BKind::Boomerang
                | BKind::Prism
                | BKind::Mine
                | BKind::Nova
                | BKind::Dust
                | BKind::Lance => rifts::move_bullet(b, dt, &targets, boss_pos, pp),
                BKind::Fireball | BKind::Plasma | BKind::Meteor => {
                    arsenal::move_bullet(b, dt, &mut self.fx)
                }
                BKind::Rail => {}
                _ => b.pos += b.vel * dt,
            }
            b.life -= dt;
        }
        self.expire_bullets();
        self.bullets.retain(|b| {
            b.life > 0.0
                && b.pos.y > -30.0
                && b.pos.y < H + 30.0
                && b.pos.x > -40.0
                && b.pos.x < W + 40.0
        });
        for b in &mut self.ebullets {
            b.pos += b.vel * wdt;
        }
        self.ebullets.retain(|b| {
            b.pos.y > -40.0 && b.pos.y < H + 30.0 && b.pos.x > -30.0 && b.pos.x < W + 30.0
        });
        self.rocks.retain(|r| r.pos.y < H + 60.0 && r.hp > -100);
    }

    fn spawn_drop(&mut self, pos: Vec2, kind: DropKind) {
        // Too many crystals on screen clutter the view: new ones merge into the ones already falling.
        if let DropKind::Crystal(v) = kind {
            let n = self
                .drops
                .iter()
                .filter(|d| matches!(d.kind, DropKind::Crystal(_)))
                .count();
            if n >= 24 {
                let mut best: Option<usize> = None;
                let mut bd = f32::MAX;
                for (i, d) in self.drops.iter().enumerate() {
                    if matches!(d.kind, DropKind::Crystal(_)) {
                        let dist = d.pos.distance(pos);
                        if dist < bd {
                            bd = dist;
                            best = Some(i);
                        }
                    }
                }
                if let Some(i) = best
                    && let DropKind::Crystal(c) = &mut self.drops[i].kind
                {
                    *c += v;
                    return;
                }
            }
        }
        let vel = match kind {
            DropKind::Crystal(_) => vec2(gen_range(-70.0, 70.0), gen_range(-140.0, -40.0)),
            _ => vec2(0.0, -60.0),
        };
        self.drops.push(Drop {
            pos,
            vel,
            kind,
            age: 0.0,
        });
    }

    fn make_capsule(&self) -> DropKind {
        let pool = self.weapon_pool();
        let want = if self.mods.capsule_four { 4 } else { 3 };
        let mut list: Vec<Weapon> = vec![];
        let mut tries = 0;
        while list.len() < want.min(pool.len()) && tries < 40 {
            tries += 1;
            let w = pool[gen_range(0, pool.len())];
            if !list.contains(&w) {
                list.push(w);
            }
        }
        if list.is_empty() {
            list.push(Weapon::Blaster);
        }
        let mut arr = [list[0]; 4];
        for (i, w) in list.iter().enumerate() {
            arr[i] = *w;
        }
        DropKind::Capsule {
            weapons: arr,
            n: list.len(),
            idx: 0,
            t: 0.0,
        }
    }

    fn update_drops(&mut self, dt: f32, au: &mut Audio) {
        let pp = self.player.pos;
        let alive = self.player.alive();
        let magnet = (70.0 + 35.0 * self.player.magnet_lvl as f32)
            * self.mods.magnet_mul
            * if self.cfg.ship == 7 { 1.5 } else { 1.0 };
        let all = self.mods.magnet_all || self.magnet_t > 0.0;
        let card_magnet = self.mods.card_magnet;
        let cap_magnet = self.mods.capsule_magnet;
        let cycle = if self.mods.capsule_four { 2.4 } else { 1.8 };
        let mut collected = vec![];
        for (i, d) in self.drops.iter_mut().enumerate() {
            d.age += dt;
            let pulled = match &d.kind {
                DropKind::Crystal(_) => {
                    alive && (d.pos.distance(pp) < magnet || all) && d.age > 0.25
                }
                DropKind::Card(_) | DropKind::Log(_) => alive && card_magnet && d.age > 0.6,
                DropKind::Capsule { .. }
                | DropKind::Power(_)
                | DropKind::Relic(_)
                | DropKind::Rift
                | DropKind::Letter(_) => alive && cap_magnet && d.age > 0.6,
                DropKind::Curse(..) => false,
            };
            if pulled {
                let target = (pp - d.pos).normalize_or_zero() * 560.0;
                d.vel += (target - d.vel) * (dt * 8.0).min(1.0);
            } else {
                match &mut d.kind {
                    DropKind::Crystal(_) => {
                        d.vel.y = (d.vel.y + 260.0 * dt).min(150.0);
                        d.vel.x *= 1.0 - (dt * 1.5).min(1.0);
                    }
                    DropKind::Capsule { n, idx, t, .. } => {
                        *t += dt;
                        if *t > cycle {
                            *t = 0.0;
                            *idx = (*idx + 1) % *n;
                        }
                        d.vel.y = (d.vel.y + 120.0 * dt).min(85.0);
                        d.vel.x = (d.age * 2.0).sin() * 30.0;
                    }
                    DropKind::Card(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(75.0);
                        d.vel.x = (d.age * 2.5).cos() * 40.0;
                    }
                    DropKind::Power(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(95.0);
                        d.vel.x = (d.age * 3.0).sin() * 20.0;
                    }
                    DropKind::Log(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(60.0);
                        d.vel.x = (d.age * 2.0).cos() * 25.0;
                    }
                    other => rifts::drop_motion(other, &mut d.vel, d.age, d.pos, pp, dt),
                }
            }
            d.pos += d.vel * dt;
            if alive && d.pos.distance(pp) < 24.0 {
                collected.push(i);
            }
        }
        for &i in collected.iter().rev() {
            let d = self.drops.swap_remove(i);
            self.collect(d, au);
        }
        self.drops.retain(|d| d.pos.y < H + 30.0);
    }

    fn collect(&mut self, d: Drop, au: &mut Audio) {
        let pp = self.player.pos;
        match d.kind {
            DropKind::Crystal(v) => {
                let mut amount = v as f32 * self.mods.crystal_mul * self.event_crystal_mul();
                self.enc.crystals += v;
                if self.cfg.daily == Some(Daily::DoubleCrystals) {
                    amount *= 2.0;
                }
                if self.cfg.mode == Mode::Survival {
                    self.score += (amount * 50.0) as u32;
                }
                self.crystals += amount;
                self.events.push(Event::Crystals(amount.round() as u32));
                if self.mods.crystal_bomb {
                    self.crystal_bomb_acc += amount;
                    if self.crystal_bomb_acc >= 100.0 {
                        self.crystal_bomb_acc -= 100.0;
                        self.player.bombs = (self.player.bombs + 1).min(9);
                        self.fx.text(pp - vec2(0.0, 30.0), "+1 BOMB", GOLD, 18.0);
                    }
                }
                au.play_vol(Sfx::Crystal, 0.5);
                self.fx.sparks(d.pos, vec2(0.0, -1.0), CYAN, 3);
            }
            DropKind::Capsule { weapons, idx, .. } => {
                let weapon = weapons[idx];
                self.enc.capsules += 1;
                let p = &mut self.player;
                if weapon == p.weapon {
                    if p.level < 5 {
                        p.level += 1;
                        self.fx.text(
                            pp - vec2(0.0, 40.0),
                            format!("{} LEVEL {}", weapon.name(), p.level),
                            weapon.color(),
                            20.0,
                        );
                    } else {
                        self.score += 2000;
                        self.fx.text(pp - vec2(0.0, 40.0), "+2000", GOLD, 20.0);
                    }
                } else {
                    encounters::pick_new_weapon(p, weapon);
                    self.fx
                        .text(pp - vec2(0.0, 40.0), weapon.name(), weapon.color(), 22.0);
                }
                self.check_mastery();
                au.play(Sfx::Powerup);
                self.fx.ring(pp, 200.0, 0.35, weapon.color(), 3.0);
            }
            DropKind::Card(c) => self.give_card(c, au),
            DropKind::Log(i) => self.collect_log(i, au),
            DropKind::Curse(c, _) => self.take_curse(c, au),
            DropKind::Relic(w) => self.take_relic(w, au),
            DropKind::Rift => self.start_minigame(au),
            DropKind::Letter(i) => self.take_letter(i, au),
            DropKind::Power(k) => {
                au.play(Sfx::Powerup);
                self.fx.ring(pp, 240.0, 0.4, k.color(), 4.0);
                self.fx
                    .text(pp - vec2(0.0, 40.0), k.name(), k.color(), 22.0);
                match k {
                    PowerKind::Frenzy => self.frenzy_t = 8.0,
                    PowerKind::Star => {
                        self.star_t = 6.0;
                        self.player.invuln = self.player.invuln.max(6.0);
                    }
                    PowerKind::Magnet => self.magnet_t = 10.0,
                    PowerKind::SlowMo => self.slowmo_t = 6.0,
                    PowerKind::ExtraLife => {
                        self.player.lives = (self.player.lives + 1).min(9);
                        au.play(Sfx::OneUp);
                    }
                    PowerKind::Bomb => self.player.bombs = (self.player.bombs + 1).min(9),
                    PowerKind::Shield => self.player.shield = true,
                    PowerKind::Crystals => {
                        for _ in 0..5 {
                            self.spawn_drop(pp - vec2(0.0, 30.0), DropKind::Crystal(5));
                        }
                    }
                }
            }
        }
    }

    fn check_mastery(&mut self) {
        if self.player.level >= 5 {
            let wi = self.player.weapon.index();
            if !self.mastery_run[wi] {
                self.mastery_run[wi] = true;
                self.events.push(Event::Mastery(wi));
            }
        }
    }

    // -----------------------------------------------------------------------
    // Collisions
    // -----------------------------------------------------------------------

    fn collisions(&mut self, au: &mut Audio) {
        let mut landed = 0u32;
        let time = self.time;
        let mut pirate = false;
        let mut charged5 = false;
        self.rail_hits(au);
        for b in &mut self.bullets {
            if b.life <= 0.0 || b.kind == BKind::Hole {
                continue;
            }
            // Plasma burns through six enemies, then fades.
            if b.kind == BKind::Plasma && b.hit.iter().all(|h| *h != 0) {
                b.life = 0.0;
                continue;
            }
            for e in &mut self.enemies {
                if !e.active() || e.hp <= 0 || e.pos.y < -10.0 || b.hit.contains(&e.id) {
                    continue;
                }
                if e.alpha(time) < 0.5 {
                    continue;
                }
                let r = e.kind.radius()
                    + match b.kind {
                        BKind::Laser => 2.0,
                        BKind::Plasma => 9.0,
                        BKind::Fireball => 7.0,
                        BKind::Meteor => 6.0,
                        _ => 3.0,
                    };
                if b.pos.distance(e.pos) < r {
                    landed += 1;
                    if shield_blocks(e, b.pos) {
                        e.shield -= 1;
                        e.flash = 0.05;
                        au.play_vol(Sfx::Clang, 0.4);
                        self.fx.sparks(
                            b.pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(180, 200, 230, 255),
                            4,
                        );
                        if e.shield <= 0 {
                            self.fx.burst(e.pos + vec2(0.0, 12.0), GRAY, 12, 120.0);
                        }
                        if b.kind != BKind::Disc {
                            b.life = 0.0;
                        }
                        break;
                    }
                    let dmg = b.dmg.floor() as i32
                        + if gen_range(0.0, 1.0) < b.dmg.fract() {
                            1
                        } else {
                            0
                        };
                    e.hp -= if e.frozen > 0.0 { dmg * 2 } else { dmg };
                    if b.kind == BKind::Charge && e.hp <= 0 {
                        b.base_x += 1.0;
                        if b.base_x >= 5.0 {
                            charged5 = true;
                        }
                    }
                    e.flash = 0.07;
                    e.damaged = true;
                    e.last_weapon = b.weapon;
                    if b.kind == BKind::Ice {
                        if e.frozen <= 0.0 {
                            self.frozen_count += 1;
                        }
                        e.frozen = e.frozen.max(1.6 * self.mods.freeze_mul);
                    }
                    if e.kind == Kind::Merchant {
                        pirate = true;
                    }
                    if e.hp > 0 {
                        au.play_vol(Sfx::Clang, 0.4);
                        self.fx.sparks(b.pos, -b.vel.normalize_or_zero(), WHITE, 4);
                    }
                    if b.pierce > 0 {
                        if b.pierce != 255 {
                            b.pierce -= 1;
                        }
                        if let Some(slot) = b.hit.iter_mut().find(|h| **h == 0) {
                            *slot = e.id;
                        }
                    } else {
                        b.life = 0.0;
                        break;
                    }
                }
            }
        }
        if charged5 {
            self.ach(ach::CHARGED);
        }
        if pirate && !self.grudge {
            self.grudge = true;
            self.ach(ach::PIRATE);
            self.banner = Some((
                "SPACE PIRATE".into(),
                "The Merchant will never forget this.".into(),
                3.0,
            ));
        }
        // boss
        if let Some(boss) = &mut self.boss
            && boss.targetable()
        {
            for b in &mut self.bullets {
                if b.life <= 0.0 || b.kind == BKind::Hole {
                    continue;
                }
                if let Some(t) = boss.hit_test(b.pos) {
                    let mul = match b.kind {
                        BKind::Laser => 0.8,
                        BKind::Flame => 0.9,
                        BKind::Disc => 0.5,
                        _ => 1.0,
                    };
                    let res = boss.damage(t, b.dmg * mul);
                    landed += 1;
                    if b.kind == BKind::Disc {
                        if b.t < 0.35 {
                            b.vel = Vec2::ZERO;
                            b.t = 0.35;
                        }
                    } else {
                        b.life = 0.0;
                    }
                    if res.blocked {
                        self.fx.sparks(
                            b.pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(180, 150, 255, 255),
                            3,
                        );
                        au.play_vol(Sfx::Clang, 0.3);
                        continue;
                    }
                    self.fx.sparks(
                        b.pos,
                        vec2(0.0, 1.0),
                        Color::from_rgba(255, 220, 150, 255),
                        2,
                    );
                    if res.part_destroyed {
                        self.enc.parts += 1;
                        au.play(Sfx::Boom);
                        self.fx
                            .explosion(res.part_pos, Color::from_rgba(255, 160, 80, 255), true);
                        self.score += 5000;
                        self.fx.text(res.part_pos, "+5000", GOLD, 20.0);
                        self.hitstop = 0.05;
                        for _ in 0..3 {
                            self.drops.push(Drop {
                                pos: res.part_pos,
                                vel: vec2(gen_range(-70.0, 70.0), gen_range(-140.0, -40.0)),
                                kind: DropKind::Crystal(1),
                                age: 0.0,
                            });
                        }
                    }
                    if res.killed {
                        au.play(Sfx::BigBoom);
                        self.fx.flash = 0.8;
                        self.fx.flash_color = WHITE;
                        self.ebullets.clear();
                        self.hitstop = 0.25;
                    }
                }
            }
        }
        // rocks
        for b in &mut self.bullets {
            if b.life <= 0.0 {
                continue;
            }
            for r in &mut self.rocks {
                let rad = if r.solid {
                    28.0
                } else if r.big {
                    24.0
                } else {
                    13.0
                };
                if r.hp > 0 && b.pos.distance(r.pos) < rad {
                    if !r.solid {
                        r.hp -= b.dmg.ceil() as i32;
                    }
                    r.flash = 0.06;
                    b.life = 0.0;
                    self.fx.sparks(
                        b.pos,
                        vec2(0.0, 1.0),
                        Color::from_rgba(200, 160, 120, 255),
                        3,
                    );
                    break;
                }
            }
        }
        self.sector_landed += landed;
        self.expire_bullets();
        self.bullets.retain(|b| b.life > 0.0);

        let mut new_rocks = vec![];
        let mut rock_drops = vec![];
        for r in &mut self.rocks {
            if r.hp <= 0 && r.hp > -100 {
                r.hp = -1000;
                self.fx
                    .explosion(r.pos, Color::from_rgba(170, 120, 80, 255), false);
                self.score += if r.big { 200 } else { 100 };
                for _ in 0..(if r.big { 3 } else { 2 }) {
                    rock_drops.push(r.pos);
                }
                if r.big {
                    for s in [-1.0, 1.0] {
                        new_rocks.push(Rock {
                            pos: r.pos,
                            vel: vec2(s * 90.0, r.vel.y * 0.8),
                            hp: 2,
                            big: false,
                            solid: false,
                            rot: 0.0,
                            vr: s * 3.0,
                            variant: r.variant,
                            flash: 0.0,
                        });
                    }
                }
            }
        }
        if !rock_drops.is_empty() {
            au.play_vol(Sfx::Pop, 0.7);
        }
        for p in rock_drops {
            self.spawn_drop(p, DropKind::Crystal(1));
        }
        self.rocks.extend(new_rocks);

        self.player_hazards(au);
        self.with_p2(|g| g.player_hazards(au));
    }

    /// Enemy shots, bodies, rocks, lightning and bosses against the ship in `self.player`.
    fn player_hazards(&mut self, au: &mut Audio) {
        let time = self.time;
        if !self.player.alive() {
            return;
        }
        let pp = self.player.pos;
        let hr = self.hit_r();
        let mut grazes = 0;
        let mut webbed = false;
        let mut hurt = false;
        let invuln = self.player.invuln > 0.0;
        self.ebullets.retain_mut(|b| {
            let d = b.pos.distance(pp);
            if b.style == 3 {
                if d < hr + b.r && !invuln {
                    webbed = true;
                    return false;
                }
                return true;
            }
            if d < hr + b.r - 1.0 && !invuln {
                hurt = true;
            } else if !b.grazed && d < hr + b.r + 16.0 {
                b.grazed = true;
                grazes += 1;
            }
            true
        });
        if webbed {
            self.web_t = 2.5;
            au.play(Sfx::Web);
            self.fx.burst(pp, WHITE, 10, 90.0);
        }
        if grazes > 0 && !self.classic() {
            self.grazes += grazes;
            let gm = self.mods.graze_mul;
            self.player.meter =
                (self.player.meter + 0.012 * gm * grazes as f32 * self.mods.meter_mul).min(1.0);
            self.score += (20.0 * gm) as u32 * grazes;
            au.play_vol(Sfx::Graze, 0.6);
            self.fx
                .sparks(pp, vec2(0.0, -1.0), Color::new(1.0, 1.0, 0.7, 1.0), 2);
            for _ in 0..grazes {
                self.events.push(Event::Graze);
            }
            if self.grazes >= 100 {
                self.ach(ach::CLOSE_SHAVE);
            }
        }
        if invuln {
            if self.star_t > 0.0 {
                for e in &mut self.enemies {
                    if e.active() && !e.harmless && e.pos.distance(pp) < e.kind.radius() + 10.0 {
                        e.hp = 0;
                    }
                }
            }
            return;
        }
        if !hurt {
            for e in &mut self.enemies {
                if e.active()
                    && !e.harmless
                    && e.hp > 0
                    && e.alpha(time) > 0.3
                    && e.pos.distance(pp) < e.kind.radius() + hr + 2.0
                {
                    e.hp = 0;
                    hurt = true;
                    break;
                }
            }
        }
        if !hurt {
            for r in &mut self.rocks {
                let rad = if r.solid {
                    26.0
                } else if r.big {
                    20.0
                } else {
                    11.0
                };
                if r.hp > 0 && r.pos.distance(pp) < rad + hr {
                    if !r.solid {
                        r.hp = 0;
                    }
                    hurt = true;
                    break;
                }
            }
        }
        if !hurt {
            for s in &self.strikes {
                if s.t >= 1.0 && (s.x - pp.x).abs() < 16.0 + hr {
                    hurt = true;
                }
            }
        }
        if !hurt
            && let Some(b) = &self.boss
            && b.touches_player(pp, hr)
        {
            hurt = true;
        }
        if hurt {
            self.hurt_player(au);
        }
    }

    fn hurt_player(&mut self, au: &mut Audio) {
        self.enc.nohit_t = 0.0;
        self.rift_on_hit();
        let pp = self.player.pos;
        self.sector_hits += 1;
        self.boss_hits += 1;
        if self.mods.counter {
            for e in &mut self.enemies {
                if e.active() && !e.harmless {
                    e.hp -= 3;
                    e.flash = 0.15;
                }
            }
            self.fx
                .ring(pp, 600.0, 0.6, Color::new(1.0, 0.4, 0.3, 1.0), 4.0);
        }
        if self.player.shield {
            self.player.shield = false;
            self.player.invuln = 1.5;
            au.play(Sfx::Shield);
            self.fx.ring(pp, 300.0, 0.4, CYAN, 4.0);
            self.fx.burst(pp, CYAN, 30, 220.0);
            self.ebullets.retain(|b| b.pos.distance(pp) > 90.0);
            return;
        }
        if self.mods.last_breath {
            self.detonate(au, true);
        }
        au.play(Sfx::PlayerDie);
        self.fx.explosion(pp, CYAN, true);
        self.fx.shake = 0.9;
        self.hitstop = 0.18;
        self.deaths += 1;
        self.combo = 0;
        self.overdrive_t = 0.0;
        self.wall_t = 0.0;
        self.cannon_t = 0.0;
        let keep = self.mods.keep_level || self.insurance;
        if self.insurance && !self.mods.keep_level {
            self.insurance = false;
        }
        let p = &mut self.player;
        p.lives -= 1;
        if !keep {
            p.level = (p.level - 1).max(1);
        }
        self.rift_on_death();
        let p = &mut self.player;
        p.dead_t = 1.4;
        if p.lives <= 0 && self.mods.phoenix && !p.phoenix_used {
            p.phoenix_used = true;
            p.lives = 1;
            self.banner = Some(("PHOENIX!".into(), "You rose from the ashes.".into(), 2.5));
        }
        self.ebullets.clear();
        self.strikes.clear();
    }

    fn process_deaths(&mut self, au: &mut Audio) {
        let mut i = 0;
        let mut pops = 0;
        while i < self.enemies.len() {
            let e = &self.enemies[i];
            if e.dead && e.hp > 0 {
                let e = self.enemies.swap_remove(i);
                let g = e.group as usize;
                if e.bonus && g < self.groups.len() {
                    self.groups[g].0 = 0;
                }
                if e.kind == Kind::Merchant && !e.damaged {
                    self.events.push(Event::MerchantSpared);
                }
                continue;
            }
            if e.hp > 0 {
                if e.kind == Kind::Merchant
                    && !e.damaged
                    && !e.gifted
                    && let EState::Flyby { t, .. } = e.state
                    && t > 0.8
                {
                    let pos = e.pos;
                    self.enemies[i].gifted = true;
                    self.spawn_drop(pos, DropKind::Power(PowerKind::random()));
                    self.drop_log(10, pos);
                    for _ in 0..4 {
                        self.spawn_drop(pos, DropKind::Crystal(5));
                    }
                    self.banner = Some((
                        "MERCHANT'S GIFT".into(),
                        "Thanks for your kindness, pilot!".into(),
                        2.5,
                    ));
                    au.play(Sfx::Buy);
                }
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            pops += 1;
            self.kills += 1;
            self.combo += 1;
            self.best_combo = self.best_combo.max(self.combo);
            self.combo_t = 1.1 * self.mods.combo_mul;
            if self.combo == 50 {
                self.ach(ach::COMBO50);
            }
            if self.combo == 100 {
                self.ach(ach::COMBO100);
            }
            if self.combo.is_multiple_of(10) && self.multiplier() > 1 {
                au.play(Sfx::Combo);
                self.fx.text(
                    e.pos - vec2(0.0, 20.0),
                    format!("COMBO x{}", self.multiplier()),
                    PINK,
                    22.0,
                );
            }
            self.events.push(Event::Kill(e.kind, e.last_weapon));
            self.on_kill(&e, au);
            self.rift_on_kill(&e);
            self.arcade_on_kill(&e);
            self.seen(crate::progress::lore_of_kind(e.kind.index()));
            if !self.classic() {
                self.player.meter = (self.player.meter + 0.025 * self.mods.meter_mul).min(1.0);
            }
            let dive = e.diving();
            let base = e.kind.points() * if dive { self.mods.dive_mul } else { 1 };
            let pts = self.add_score(base);
            if pts > 0 {
                self.fx.text(
                    e.pos,
                    pts.to_string(),
                    if dive {
                        GOLD
                    } else {
                        Color::new(1.0, 1.0, 0.8, 1.0)
                    },
                    16.0,
                );
            }
            self.fx.explosion(
                e.pos,
                e.kind.color(),
                matches!(
                    e.kind,
                    Kind::Carrier | Kind::Golden | Kind::Cow | Kind::Merchant
                ),
            );
            match e.kind {
                Kind::Golden => {
                    self.ach(ach::FOOLS_GOLD);
                    if !e.bonus {
                        au.play(Sfx::Fanfare);
                        for _ in 0..8 {
                            self.spawn_drop(e.pos, DropKind::Crystal(5));
                        }
                        self.banner = Some((
                            "GOLDEN SCARAB!".into(),
                            "A fortune in crystals!".into(),
                            2.0,
                        ));
                    }
                }
                Kind::Cow => {
                    au.play(Sfx::Moo);
                    self.ach(ach::MOO);
                    self.fx.text(e.pos - vec2(0.0, 30.0), "MUUUU!", WHITE, 30.0);
                    self.spawn_drop(e.pos, DropKind::Card(cards::id::COW));
                }
                Kind::Merchant => {
                    for _ in 0..12 {
                        self.spawn_drop(e.pos, DropKind::Crystal(5));
                    }
                }
                _ => {}
            }
            if e.bonus {
                self.challenge_hits += 1;
                let g = e.group as usize;
                if g < self.groups.len() && self.groups[g].0 > 0 {
                    self.groups[g].1 += 1;
                    if self.groups[g].1 == self.groups[g].0 {
                        let b = 1000 * (1 + self.cycle) * self.mods.bonus_mul;
                        self.score += b;
                        self.fx
                            .text(e.pos - vec2(0.0, 24.0), format!("GROUP! +{b}"), GREEN, 22.0);
                        au.play(Sfx::MenuOk);
                    }
                }
                if e.kind == Kind::Golden {
                    for _ in 0..2 {
                        self.spawn_drop(e.pos, DropKind::Crystal(1));
                    }
                }
                continue;
            }
            if e.kind == Kind::Splitter {
                for s in [-1.0, 1.0] {
                    self.spawn(
                        Kind::Mini,
                        e.col,
                        e.row,
                        EState::Free {
                            vel: vec2(s * 120.0, 60.0),
                        },
                        98,
                        false,
                    );
                    if let Some(m) = self.enemies.last_mut() {
                        m.pos = e.pos + vec2(s * 8.0, 0.0);
                    }
                }
            }
            if self.classic() || matches!(e.kind, Kind::Golden | Kind::Cow | Kind::Merchant) {
                continue;
            }
            let mut n_crystals = match e.kind {
                Kind::Beetle | Kind::Carrier => 3,
                Kind::Gunner | Kind::Splitter | Kind::Shieldbearer => 2,
                _ => (gen_range(0.0, 1.0) < 0.45) as i32,
            };
            if self.crystalizer && gen_range(0.0, 1.0) < 0.3 {
                n_crystals += 1;
            }
            let base = 1 + (self.stage().tier.saturating_sub(1) / 3).min(3);
            for _ in 0..n_crystals {
                let v = if gen_range(0, 20) == 0 {
                    5 * base
                } else {
                    base
                };
                self.spawn_drop(e.pos, DropKind::Crystal(v));
            }
            let rain = if self.cfg.daily == Some(Daily::CapsuleRain) {
                3.0
            } else {
                1.0
            };
            self.capsule_pity += 1;
            let cap_chance = if e.kind == Kind::Beetle { 0.06 } else { 0.011 } * rain;
            if self.capsule_pity > 70 || gen_range(0.0, 1.0) < cap_chance {
                self.capsule_pity = 0;
                let cap = self.make_capsule();
                self.spawn_drop(e.pos, cap);
            } else if gen_range(0.0, 1.0) < 0.006 * self.mods.powerup_mul * rain {
                self.spawn_drop(e.pos, DropKind::Power(PowerKind::random()));
            }
            let card_chance = if e.kind == Kind::Beetle {
                0.006
            } else {
                0.0018
            } * self.mods.card_mul;
            if gen_range(0.0, 1.0) < card_chance {
                self.spawn_drop(e.pos, DropKind::Card(cards::roll(0)));
            }
        }
        if pops > 0 {
            au.play_vol(Sfx::Pop, 0.7);
            self.fx.shake = self.fx.shake.max(0.08);
        }
    }

    // -----------------------------------------------------------------------
    // Shop
    // -----------------------------------------------------------------------

    fn roll_offers(&mut self) {
        let mut pool: Vec<Item> = vec![
            Item::Upgrade,
            Item::Thruster,
            Item::Trigger,
            Item::Magnet,
            Item::Insurance,
            Item::EnergyCell,
            Item::Amulet,
            Item::Crystallizer,
            Item::MysteryCard,
            Item::SideCannons,
            Item::LaserSight,
            Item::Capacitor,
            Item::MysteryBox,
            Item::MysteryBox,
        ];
        if self.cfg.rank >= 5 {
            pool.push(Item::Drone);
        }
        if self.cfg.rank >= 11 {
            pool.extend([Item::Bank, Item::Radar, Item::TreasureMap, Item::Mercenary]);
        }
        for w in self.weapon_pool() {
            if w != self.player.weapon {
                pool.push(Item::Weapon(w));
            }
        }
        pool.retain(|i| self.shop_block(*i).is_none());
        // Always two weapons for sale, preferably stronger than the current one.
        let cur = self.player.weapon.tier();
        let mut weapons: Vec<Item> = pool
            .iter()
            .copied()
            .filter(|i| matches!(i, Item::Weapon(w) if w.tier() >= cur))
            .collect();
        if weapons.len() < 2 {
            weapons = pool
                .iter()
                .copied()
                .filter(|i| matches!(i, Item::Weapon(_)))
                .collect();
        }
        let mut offers: Vec<Item> = vec![];
        while offers.len() < 2 && !weapons.is_empty() {
            offers.push(weapons.swap_remove(gen_range(0, weapons.len())));
        }
        pool.retain(|i| !matches!(i, Item::Weapon(_)));
        while offers.len() < 4 && !pool.is_empty() {
            let it = pool.swap_remove(gen_range(0, pool.len()));
            offers.push(it);
        }
        offers.sort_by_key(|i| match i {
            Item::Weapon(w) => w.tier() as i32,
            _ => 10,
        });
        self.shop_offers = vec![Item::ExtraLife, Item::Bomb, Item::Shield];
        if !self.arcade.autofire {
            self.shop_offers.push(Item::Autofire);
        }
        if self.arcade.core < 6 {
            self.shop_offers.push(Item::DamageCore);
        }
        self.shop_offers.extend(offers);
    }

    fn bought(&self, it: Item) -> u32 {
        self.shop_buys
            .iter()
            .find(|(k, _)| *k == it.key())
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    fn price(&self, it: Item) -> u32 {
        let mut p = it.price() as f32 * (1.0 + 0.5 * self.bought(it) as f32) * self.mods.shop_mul;
        if self.grudge {
            p *= 1.3;
        }
        if self.cfg.daily == Some(Daily::DoubleCrystals) {
            p *= 1.5;
        }
        p.round() as u32
    }

    fn reroll_price(&self) -> u32 {
        if self.mods.free_reroll && !self.free_reroll_used {
            0
        } else {
            15 + 10 * self.rerolls
        }
    }

    fn shop_block(&self, it: Item) -> Option<&'static str> {
        let p = &self.player;
        match it {
            Item::ExtraLife if p.lives >= 7 => Some("MAX"),
            Item::Bomb if p.bombs >= 7 => Some("MAX"),
            Item::Upgrade if p.level >= 5 => Some("MAX"),
            Item::Shield if p.shield => Some("OWNED"),
            Item::Thruster if p.speed_lvl >= 3 => Some("MAX"),
            Item::Trigger if p.fire_lvl >= 3 => Some("MAX"),
            Item::Magnet if p.magnet_lvl >= 3 || self.mods.magnet_all => Some("MAX"),
            Item::Drone if self.drones_total() >= 3 => Some("MAX"),
            Item::SideCannons if self.side_cannons => Some("OWNED"),
            Item::Insurance if self.insurance || self.mods.keep_level => Some("OWNED"),
            Item::Bank if self.bank => Some("OWNED"),
            Item::Crystallizer if self.crystalizer => Some("OWNED"),
            Item::TreasureMap if self.treasure_map => Some("OWNED"),
            Item::Mercenary if self.ally_sectors > 0 => Some("OWNED"),
            Item::Weapon(w) if w == p.weapon => Some("EQUIPPED"),
            Item::Autofire if self.arcade.autofire => Some("OWNED"),
            Item::LaserSight if self.arcade.sight => Some("OWNED"),
            Item::Capacitor if self.arcade.capacitor => Some("OWNED"),
            Item::DamageCore if self.arcade.core >= 6 => Some("MAX"),
            _ => None,
        }
    }

    fn update_shop(&mut self, inp: &Input, au: &mut Audio) -> Phase {
        let n = self.shop_offers.len() + 2;
        if inp.up_p {
            self.shop_sel = (self.shop_sel + n - 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.down_p {
            self.shop_sel = (self.shop_sel + 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.back {
            self.shop_sel = n - 1;
            au.play(Sfx::MenuMove);
        }
        if inp.confirm {
            let i = self.shop_sel;
            if i == n - 1 {
                au.play(Sfx::MenuOk);
                return self.leave_shop();
            }
            if i == n - 2 {
                let price = self.reroll_price();
                if (self.crystals.floor() as u32) < price {
                    au.play(Sfx::Deny);
                } else {
                    if price == 0 {
                        self.free_reroll_used = true;
                    } else {
                        self.rerolls += 1;
                    }
                    self.crystals -= price as f32;
                    self.roll_offers();
                    au.play(Sfx::Portal);
                }
                return Phase::Shop;
            }
            let it = self.shop_offers[i];
            let price = self.price(it);
            if self.shop_block(it).is_some() || (self.crystals.floor() as u32) < price {
                au.play(Sfx::Deny);
            } else {
                self.crystals -= price as f32;
                let key = it.key();
                match self.shop_buys.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, n)) => *n += 1,
                    None => self.shop_buys.push((key, 1)),
                }
                au.play(Sfx::Buy);
                let p = &mut self.player;
                match it {
                    Item::ExtraLife => p.lives += 1,
                    Item::Bomb => p.bombs += 1,
                    Item::Shield => p.shield = true,
                    Item::Upgrade => p.level += 1,
                    Item::Weapon(w) => encounters::pick_new_weapon(p, w),
                    Item::Thruster => p.speed_lvl += 1,
                    Item::Trigger => p.fire_lvl += 1,
                    Item::Magnet => p.magnet_lvl += 1,
                    Item::Drone => self.drones_bought += 1,
                    Item::SideCannons => self.side_cannons = true,
                    Item::Insurance => self.insurance = true,
                    Item::EnergyCell => {
                        p.meter = 1.0;
                        self.mods.meter_mul += 0.2;
                    }
                    Item::Bank => self.bank = true,
                    Item::Radar => self.mods.card_mul *= 2.0,
                    Item::Amulet => self.mods.powerup_mul *= 1.6,
                    Item::Crystallizer => self.crystalizer = true,
                    Item::TreasureMap => self.treasure_map = true,
                    Item::Mercenary => self.ally_sectors = 2,
                    Item::MysteryCard => {
                        let c = cards::roll(0);
                        self.give_card(c, au);
                    }
                    Item::Autofire => self.arcade.autofire = true,
                    Item::LaserSight => self.arcade.sight = true,
                    Item::Capacitor => self.arcade.capacitor = true,
                    Item::MysteryBox => self.open_box(au),
                    Item::DamageCore => self.arcade.core += 1,
                }
                self.mirror_purchase(it);
                self.check_mastery();
            }
        }
        Phase::Shop
    }

    fn leave_shop(&mut self) -> Phase {
        self.rocks.clear();
        self.wave += 1;
        if self.cfg.mode == Mode::Campaign
            && self.cycle == 0
            && self.sector() == 4
            && self.deaths == 0
            && !self.secret_done
        {
            return Phase::Portal { t: 0.0 };
        }
        Phase::Intro { t: 0.0 }
    }

    // -----------------------------------------------------------------------
    // Drawing
    // -----------------------------------------------------------------------

    pub fn draw(&self, sp: &Sprites, art: &Art, shake_on: bool, show_hud: bool) {
        self.bg.draw(sp, self.time);
        let off = self.fx.offset(shake_on);
        let season = crate::progress::season();

        for r in &self.rocks {
            if r.hp <= -100 {
                continue;
            }
            if r.solid {
                let p = r.pos + off;
                draw_rectangle(
                    p.x - 26.0,
                    p.y - 32.0,
                    52.0,
                    64.0,
                    Color::from_rgba(90, 70, 45, 255),
                );
                draw_rectangle_lines(
                    p.x - 26.0,
                    p.y - 32.0,
                    52.0,
                    64.0,
                    3.0,
                    Color::from_rgba(40, 28, 16, 255),
                );
                draw_rectangle(
                    p.x - 20.0,
                    p.y - 26.0,
                    40.0,
                    6.0,
                    Color::from_rgba(130, 100, 60, 255),
                );
                draw_circle_lines(
                    p.x,
                    p.y + 4.0,
                    10.0,
                    2.0,
                    Color::from_rgba(255, 200, 90, if r.flash > 0.0 { 255 } else { 120 }),
                );
                draw_line(
                    p.x - 6.0,
                    p.y + 4.0,
                    p.x + 6.0,
                    p.y + 4.0,
                    2.0,
                    Color::from_rgba(255, 200, 90, 120),
                );
                continue;
            }
            sp.asteroid[r.variant].draw(
                r.pos + off,
                if r.big { 5.0 } else { 3.0 },
                r.rot,
                WHITE,
                r.flash > 0.0,
            );
        }

        for s in &self.strikes {
            if s.t < 1.0 {
                let a = if (self.time * 14.0) as i32 % 2 == 0 {
                    0.5
                } else {
                    0.2
                };
                draw_rectangle(
                    s.x - 16.0,
                    0.0,
                    32.0,
                    H,
                    Color::new(0.5, 0.8, 1.0, a * 0.25),
                );
                draw_line(s.x, 0.0, s.x, H, 1.0, Color::new(0.7, 0.9, 1.0, a));
            } else {
                let mut p = vec2(s.x, 0.0);
                for i in 0..14 {
                    let q = vec2(
                        s.x + ((self.time * 50.0 + i as f32).sin() * 12.0),
                        p.y + H / 14.0,
                    );
                    draw_line(p.x, p.y, q.x, q.y, 6.0, Color::new(0.6, 0.85, 1.0, 0.5));
                    draw_line(p.x, p.y, q.x, q.y, 2.0, WHITE);
                    p = q;
                }
            }
        }

        for d in &self.drops {
            let p = d.pos + off;
            match &d.kind {
                DropKind::Crystal(v) => {
                    let f = ((self.time * 8.0 + d.age) as usize) % 2;
                    let big = *v > 1;
                    let r = if big { 11.0 } else { 7.0 };
                    draw_circle(p.x, p.y, r, Color::new(0.3, 0.8, 1.0, 0.12));
                    if big {
                        sp.crystal_big[f].draw(p, 3.0, 0.0, WHITE, false);
                    } else {
                        sp.crystal[f].draw(p, 2.0, 0.0, WHITE, false);
                    }
                }
                DropKind::Capsule { weapons, idx, t, n } => {
                    let w = weapons[*idx];
                    let cycle = if self.mods.capsule_four { 2.4 } else { 1.8 };
                    let blink = *n > 1 && *t > cycle - 0.4 && (self.time * 16.0) as i32 % 2 == 0;
                    let next = if *n > 1 {
                        Some(weapons[(*idx + 1) % *n])
                    } else {
                        None
                    };
                    rifts::draw_capsule(p, w, next, *t / cycle, self.time, blink);
                }
                DropKind::Curse(..) | DropKind::Relic(_) | DropKind::Rift => {
                    rifts::draw_drop(&d.kind, p, self.time)
                }
                DropKind::Letter(i) => arcade::draw_letter_drop(p, *i, self.time),
                DropKind::Card(c) => {
                    let rc = cards::CARDS[*c].rarity.color();
                    let pulse = (self.time * 6.0).sin() * 0.5 + 0.5;
                    draw_circle(
                        p.x,
                        p.y,
                        22.0 + pulse * 4.0,
                        Color::new(rc.r, rc.g, rc.b, 0.25),
                    );
                    let sx = (self.time * 3.0).cos();
                    draw_texture_ex(
                        &sp.card.tex,
                        p.x - 10.5 * sx.abs(),
                        p.y - 13.5,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(21.0 * sx.abs().max(0.15), 27.0)),
                            ..Default::default()
                        },
                    );
                }
                DropKind::Log(_) => encounters::draw_log_drop(p, self.time),
                DropKind::Power(k) => {
                    let c = k.color();
                    let pulse = (self.time * 8.0).sin() * 2.0;
                    draw_circle(p.x, p.y, 16.0 + pulse, Color::new(c.r, c.g, c.b, 0.25));
                    draw_circle(p.x, p.y, 12.0, c);
                    draw_circle_lines(p.x, p.y, 12.0, 2.0, WHITE);
                    let size = if k.letter().len() > 1 { 12.0 } else { 18.0 };
                    ui::text_c(
                        k.letter(),
                        p.x,
                        p.y + size * 0.33,
                        size,
                        Color::from_rgba(20, 10, 30, 255),
                    );
                }
            }
        }

        for e in &self.enemies {
            if !e.active() || (e.kind == Kind::Mimic && !e.revealed) {
                continue;
            }
            let frame = ((self.time * 6.0 + e.seed) as usize) % 2;
            let spr = &sp.enemies[e.kind.index()][frame];
            let mut tint = if e.kind.hp() > 1
                && e.hp < e.kind.hp()
                && !e.bonus
                && !matches!(e.kind, Kind::Merchant | Kind::Cow)
            {
                Color::new(1.0, 0.75, 0.75, 1.0)
            } else {
                WHITE
            };
            tint.a = e.alpha(self.time);
            if e.kind == Kind::Golden {
                draw_circle(
                    e.pos.x + off.x,
                    e.pos.y + off.y,
                    20.0,
                    Color::new(1.0, 0.85, 0.3, 0.2 + (self.time * 10.0).sin().abs() * 0.15),
                );
            }
            spr.draw(e.pos + off, S, e.rot, tint, e.flash > 0.0);
            if e.shield > 0 {
                art.plate.draw(
                    e.pos + off + vec2(0.0, 18.0),
                    3.0,
                    e.rot,
                    WHITE,
                    e.flash > 0.0,
                );
            }
            if season == crate::progress::Season::Christmas
                && !matches!(e.kind, Kind::Cow | Kind::Merchant)
            {
                let h = e.pos + off + vec2(0.0, -16.0);
                draw_triangle(
                    h + vec2(-7.0, 4.0),
                    h + vec2(7.0, 4.0),
                    h + vec2(2.0, -9.0),
                    Color::from_rgba(220, 40, 50, 255),
                );
                draw_rectangle(h.x - 8.0, h.y + 3.0, 16.0, 3.0, WHITE);
                draw_circle(h.x + 2.0, h.y - 9.0, 2.5, WHITE);
            }
            if season == crate::progress::Season::Halloween && e.kind == Kind::Beetle {
                let h = e.pos + off + vec2(0.0, -14.0);
                draw_circle(h.x, h.y, 6.0, Color::from_rgba(255, 140, 20, 255));
                draw_rectangle(
                    h.x - 1.0,
                    h.y - 9.0,
                    2.0,
                    4.0,
                    Color::from_rgba(60, 140, 40, 255),
                );
            }
        }

        if let Some(b) = &self.boss {
            b.draw(sp, art, self.time, off);
        }

        for b in &self.bolts {
            for w in b.pts.windows(2) {
                draw_line(
                    w[0].x + off.x,
                    w[0].y + off.y,
                    w[1].x + off.x,
                    w[1].y + off.y,
                    5.0,
                    Color::new(0.6, 0.5, 1.0, 0.4),
                );
                draw_line(
                    w[0].x + off.x,
                    w[0].y + off.y,
                    w[1].x + off.x,
                    w[1].y + off.y,
                    2.0,
                    WHITE,
                );
            }
        }

        for r in &self.arcade.rails {
            arsenal::draw_rail(r, off);
        }
        for b in &self.bullets {
            let p = b.pos + off;
            match b.kind {
                BKind::Shot | BKind::Pellet => {
                    let dir = b.vel.normalize_or_zero();
                    let len = if b.kind == BKind::Pellet { 6.0 } else { 11.0 };
                    let (c1, c2) = if b.kind == BKind::Pellet {
                        (
                            Color::new(1.0, 0.5, 0.1, 0.5),
                            Color::new(1.0, 0.85, 0.5, 1.0),
                        )
                    } else {
                        (
                            Color::new(1.0, 0.6, 0.1, 0.5),
                            Color::new(1.0, 0.95, 0.6, 1.0),
                        )
                    };
                    draw_line(p.x, p.y, p.x - dir.x * len, p.y - dir.y * len, 5.0, c1);
                    draw_line(
                        p.x,
                        p.y,
                        p.x - dir.x * (len - 2.0),
                        p.y - dir.y * (len - 2.0),
                        2.5,
                        c2,
                    );
                }
                BKind::Laser => {
                    draw_rectangle(
                        p.x - 3.0,
                        p.y - 12.0,
                        6.0,
                        26.0,
                        Color::new(0.3, 0.8, 1.0, 0.35),
                    );
                    draw_rectangle(
                        p.x - 1.0,
                        p.y - 12.0,
                        2.0,
                        26.0,
                        Color::new(0.85, 1.0, 1.0, 1.0),
                    );
                }
                BKind::Missile => {
                    let dir = b.vel.normalize_or_zero();
                    draw_line(
                        p.x,
                        p.y,
                        p.x - dir.x * 9.0,
                        p.y - dir.y * 9.0,
                        4.0,
                        Color::from_rgba(255, 110, 170, 255),
                    );
                    draw_circle(p.x, p.y, 2.5, WHITE);
                }
                BKind::Star => {
                    let a = b.t * 360.0 + b.phase * 40.0;
                    draw_circle(p.x, p.y, 6.0, Color::new(1.0, 0.85, 0.4, 0.25));
                    draw_poly(p.x, p.y, 4, 5.5, a, Color::from_rgba(255, 220, 120, 255));
                    draw_poly(
                        p.x,
                        p.y,
                        4,
                        5.5,
                        a + 45.0,
                        Color::from_rgba(255, 245, 200, 255),
                    );
                    draw_circle(p.x, p.y, 1.8, WHITE);
                }
                BKind::Wave => {
                    draw_circle(p.x, p.y, 6.0, Color::new(0.4, 1.0, 0.6, 0.3));
                    draw_circle(p.x, p.y, 3.0, Color::from_rgba(200, 255, 220, 255));
                }
                BKind::Disc => {
                    let r = 7.0 + b.dmg.min(3.0);
                    let a = b.t * 25.0;
                    draw_poly(
                        p.x,
                        p.y,
                        6,
                        r,
                        a.to_degrees(),
                        Color::from_rgba(200, 200, 230, 255),
                    );
                    draw_poly_lines(p.x, p.y, 6, r + 2.0, a.to_degrees() + 30.0, 1.5, WHITE);
                }
                BKind::Bounce
                | BKind::Grenade
                | BKind::Ice
                | BKind::Bee
                | BKind::Hole
                | BKind::Cow => encounters::draw_bullet(b, p, sp, self.time),
                BKind::Needle
                | BKind::Boomerang
                | BKind::Prism
                | BKind::Mine
                | BKind::Nova
                | BKind::Dust
                | BKind::Lance => rifts::draw_bullet(b, p, self.time),
                BKind::Charge => arcade::draw_charge_bullet(b, p, self.time),
                BKind::Fireball | BKind::Plasma | BKind::Meteor => {
                    arsenal::draw_bullet(b, p, self.time)
                }
                BKind::Rail => {}
                BKind::Flame => {
                    let k = (b.t / 0.45).min(1.0);
                    let c = Color::new(1.0, 0.85 - k * 0.6, 0.3 - k * 0.3, 1.0 - k * 0.7);
                    draw_circle(p.x, p.y, 3.0 + k * 7.0, c);
                }
            }
        }

        self.enc_draw(off, sp);
        self.rift_draw(off);
        self.arcade_draw(off);
        if self.ally_sectors > 0 && self.player.alive() {
            art.ships[1][0].draw(
                self.ally_pos + off,
                2.0,
                0.0,
                Color::new(1.0, 1.0, 1.0, 0.9),
                false,
            );
        }

        if self.player.alive() {
            let p = self.player.pos + off;
            let blink = self.player.invuln > 0.0
                && self.star_t <= 0.0
                && (self.time * 18.0) as i32 % 2 == 0;
            self.draw_trail(off);
            self.draw_sight(p);
            if !blink {
                self.draw_ship_body(
                    &self.player,
                    self.seat_skin(self.p_swapped as usize),
                    p,
                    art,
                );
                self.rift_draw_player_fx(p);
                self.draw_charge(&self.player, p);
            }
            for d in self.drones.iter().take(self.drones_total() as usize) {
                sp.drone.draw(*d + off, 2.0, 0.0, WHITE, false);
            }
            if self.wall_t > 0.0 {
                let wy = p.y - 60.0;
                let a = 0.5 + (self.time * 10.0).sin() * 0.2;
                draw_rectangle(
                    p.x - 70.0,
                    wy - 5.0,
                    140.0,
                    10.0,
                    Color::new(0.4, 0.9, 1.0, a * 0.5),
                );
                draw_rectangle_lines(
                    p.x - 70.0,
                    wy - 5.0,
                    140.0,
                    10.0,
                    2.0,
                    Color::new(0.7, 1.0, 1.0, a),
                );
            }
            if self.cannon_t > 0.0 {
                let flick = 0.8 + (self.time * 40.0).sin() * 0.2;
                draw_rectangle(
                    p.x - 26.0,
                    0.0,
                    52.0,
                    p.y - 20.0,
                    Color::new(1.0, 0.5, 0.2, 0.25 * flick),
                );
                draw_rectangle(
                    p.x - 16.0,
                    0.0,
                    32.0,
                    p.y - 20.0,
                    Color::new(1.0, 0.75, 0.3, 0.7 * flick),
                );
                draw_rectangle(
                    p.x - 6.0,
                    0.0,
                    12.0,
                    p.y - 20.0,
                    Color::new(1.0, 1.0, 0.9, flick),
                );
            }
        }

        for b in &self.ebullets {
            let p = b.pos + off;
            if b.style == 3 {
                draw_circle(p.x, p.y, b.r + 2.0, Color::new(1.0, 1.0, 1.0, 0.2));
                draw_circle_lines(p.x, p.y, b.r, 1.5, Color::new(1.0, 1.0, 1.0, 0.85));
                for k in 0..3 {
                    let a = k as f32 * 1.05 + self.time;
                    draw_line(
                        p.x - a.cos() * b.r,
                        p.y - a.sin() * b.r,
                        p.x + a.cos() * b.r,
                        p.y + a.sin() * b.r,
                        1.0,
                        Color::new(1.0, 1.0, 1.0, 0.7),
                    );
                }
                continue;
            }
            // Enemy shots: always warm or purple colors, with a dark outline
            // and a white core. Crystals are always cyan-blue: they never get mixed up.
            let (outer, inner) = match b.style {
                1 => (
                    Color::from_rgba(255, 140, 40, 255),
                    Color::from_rgba(255, 240, 180, 255),
                ),
                2 => (
                    Color::from_rgba(255, 70, 50, 255),
                    Color::from_rgba(255, 230, 210, 255),
                ),
                4 => (Color::from_rgba(200, 80, 255, 255), WHITE),
                _ => (
                    Color::from_rgba(255, 50, 120, 255),
                    Color::from_rgba(255, 220, 235, 255),
                ),
            };
            let pulse = (self.time * 14.0 + b.pos.x * 0.05).sin() * 0.5 + 0.5;
            draw_circle(
                p.x,
                p.y,
                b.r + 3.0 + pulse,
                Color::new(outer.r, outer.g, outer.b, 0.28),
            );
            draw_circle(p.x, p.y, b.r + 1.4, Color::from_rgba(25, 0, 15, 255));
            draw_circle(p.x, p.y, b.r, outer);
            draw_circle(p.x, p.y, b.r * 0.5, inner);
        }

        self.coop_draw(off, art);
        if let Some(b) = &self.boss {
            b.draw_beam(self.time);
        }

        self.fx.draw_world(off);
        self.fx.draw_texts();

        self.enc_draw_late();
        if self.timewarp_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.3, 0.2, 0.8, 0.15));
        }
        if self.slowmo_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.2, 0.3, 0.9, 0.08));
        }
        if self.nostalgia_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.1, 0.9, 0.3, 0.18));
            for y in (0..H as i32).step_by(4) {
                draw_line(
                    0.0,
                    y as f32,
                    W,
                    y as f32,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.25),
                );
            }
            ui::text_c(
                "YEAR 2000",
                W / 2.0,
                H * 0.3,
                40.0,
                Color::new(0.5, 1.0, 0.5, self.nostalgia_t.min(1.0)),
            );
        }
        if let Some(fog) = &self.fog {
            let p = self.player.pos;
            let r = 230.0;
            draw_texture_ex(
                fog,
                p.x - r,
                p.y - r,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(r * 2.0, r * 2.0)),
                    ..Default::default()
                },
            );
            let dark = Color::new(0.0, 0.0, 0.0, 0.94);
            draw_rectangle(0.0, 0.0, W, (p.y - r).max(0.0), dark);
            draw_rectangle(0.0, p.y + r, W, H, dark);
            draw_rectangle(0.0, p.y - r, (p.x - r).max(0.0), r * 2.0, dark);
            draw_rectangle(p.x + r, p.y - r, W, r * 2.0, dark);
        }

        if self.fx.flash > 0.0 {
            let c = self.fx.flash_color;
            draw_rectangle(
                0.0,
                0.0,
                W,
                H,
                Color::new(c.r, c.g, c.b, self.fx.flash * 0.6 * crate::fx::flash_mul()),
            );
        }

        if show_hud {
            self.draw_hud(sp, art);
        } else {
            self.draw_mini_hud();
        }
        self.draw_overlays(sp);
        self.rift_draw_top(sp, art);
    }

    /// Only the essentials when the side panels show the rest.
    /// Engine flame, hull, shield and hitbox of one ship (used for both players).
    fn draw_ship_body(&self, pl: &Player, skin: usize, p: Vec2, art: &Art) {
        let flame = 6.0 + (self.time * 40.0 + pl.pos.x * 0.01).sin().abs() * 7.0;
        draw_triangle(
            p + vec2(-4.0, 20.0),
            p + vec2(4.0, 20.0),
            p + vec2(0.0, 20.0 + flame),
            Color::from_rgba(255, 140, 40, 255),
        );
        draw_triangle(
            p + vec2(-2.0, 20.0),
            p + vec2(2.0, 20.0),
            p + vec2(0.0, 18.0 + flame * 0.6),
            Color::from_rgba(255, 240, 150, 255),
        );
        let spr = &art.ships[self.cfg.ship][skin];
        let squash = 1.0 - pl.tilt.abs() * 0.18;
        let sz = vec2(spr.w * S * squash, spr.h * S);
        let tint = if self.star_t > 0.0 || skin == 6 {
            ui::hsl(self.time * 0.8, 0.8, 0.75)
        } else if self.web_t > 0.0 {
            Color::new(0.8, 0.8, 0.9, 1.0)
        } else {
            WHITE
        };
        if self.cfg.ship == 4 {
            draw_circle(p.x, p.y, 20.0, Color::new(0.7, 0.6, 1.0, 0.12));
        }
        draw_texture_ex(
            &spr.tex,
            (p.x - sz.x / 2.0).round(),
            (p.y - sz.y / 2.0).round(),
            tint,
            DrawTextureParams {
                dest_size: Some(sz),
                ..Default::default()
            },
        );
        if self.web_t > 0.0 {
            for k in 0..4 {
                let a = k as f32 * 0.8 + self.time;
                draw_line(
                    p.x - a.cos() * 18.0,
                    p.y - a.sin() * 18.0,
                    p.x + a.cos() * 18.0,
                    p.y + a.sin() * 18.0,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.6),
                );
            }
        }
        if pl.shield {
            let pulse = (self.time * 5.0).sin() * 2.0;
            draw_circle_lines(p.x, p.y, 28.0 + pulse, 2.0, Color::new(0.5, 0.9, 1.0, 0.7));
            draw_circle(p.x, p.y, 28.0 + pulse, Color::new(0.4, 0.8, 1.0, 0.08));
        }
        if pl.focus || self.cfg.show_hitbox {
            draw_circle(p.x, p.y, self.hit_r() + 1.5, WHITE);
            draw_circle(p.x, p.y, self.hit_r() - 1.0, RED);
        }
    }

    fn draw_mini_hud(&self) {
        for (i, (t, c)) in self.coop_status().iter().enumerate() {
            ui::text(t, 8.0, H - 26.0 + i as f32 * 14.0, 11.0, *c);
        }
        if let Some(b) = &self.boss
            && b.entered()
            && b.hp > 0.0
        {
            let w = W - 40.0;
            draw_rectangle(20.0, 10.0, w, 8.0, Color::new(0.2, 0.0, 0.05, 0.8));
            draw_rectangle(
                20.0,
                10.0,
                w * (b.hp / b.max_hp).max(0.0),
                8.0,
                Color::from_rgba(255, 60, 110, 255),
            );
            draw_rectangle_lines(20.0, 10.0, w, 8.0, 1.0, WHITE);
            ui::text_c(b.name(), W / 2.0, 36.0, 16.0, PINK);
        }
    }

    fn draw_hud(&self, sp: &Sprites, art: &Art) {
        for (i, (t, c)) in self.coop_status().iter().enumerate() {
            ui::text_r(t, W - 8.0, 64.0 + i as f32 * 14.0, 11.0, *c);
        }
        draw_rectangle(0.0, 0.0, W, 34.0, Color::new(0.0, 0.0, 0.05, 0.55));
        for (i, (ch, got)) in self.letters_line().iter().enumerate() {
            let x = W / 2.0 - 52.0 + i as f32 * 15.0;
            ui::text_c(
                &ch.to_string(),
                x,
                46.0,
                12.0,
                if *got {
                    GOLD
                } else {
                    Color::new(0.35, 0.35, 0.45, 0.8)
                },
            );
        }
        ui::text(&format!("{:08}", self.score), 10.0, 23.0, 24.0, WHITE);
        ui::text_r(
            &format!("HI {:08}", self.cfg.highscore),
            W - 10.0,
            23.0,
            18.0,
            GOLD,
        );
        let mult = self.multiplier();
        if self.combo >= 3 {
            let c = if mult > 1 { PINK } else { DIM };
            ui::text_c(
                &format!("COMBO {}  x{}", self.combo, mult),
                W / 2.0,
                21.0,
                18.0,
                c,
            );
            let w = 100.0 * (self.combo_t / (1.1 * self.mods.combo_mul)).clamp(0.0, 1.0);
            draw_rectangle(W / 2.0 - 50.0, 26.0, w, 3.0, c);
        }
        draw_rectangle(0.0, H - 26.0, W, 26.0, Color::new(0.0, 0.0, 0.05, 0.55));
        let lives = self.player.lives.max(0);
        let ship = &art.ships[self.cfg.ship][self.cfg.skin];
        for i in 0..lives.min(5) {
            ship.draw(
                vec2(14.0 + i as f32 * 20.0, H - 13.0),
                1.2,
                0.0,
                WHITE,
                false,
            );
        }
        if lives > 5 {
            ui::text(&format!("+{}", lives - 5), 108.0, H - 7.0, 16.0, WHITE);
        }
        for i in 0..self.player.bombs.max(0).min(4) {
            sp.icon(Icon::Bomb).draw(
                vec2(146.0 + i as f32 * 17.0, H - 13.0),
                1.5,
                0.0,
                WHITE,
                false,
            );
        }
        if !self.classic() {
            let mx = 218.0;
            draw_rectangle(mx, H - 18.0, 50.0, 8.0, Color::new(0.2, 0.2, 0.3, 1.0));
            let full = self.player.meter >= self.ship().ab_cost;
            let c = if full {
                ui::hsl(self.time, 0.8, 0.6)
            } else {
                CYAN
            };
            draw_rectangle(mx, H - 18.0, 50.0 * self.player.meter, 8.0, c);
            sp.crystal[0].draw(vec2(284.0, H - 13.0), 2.0, 0.0, WHITE, false);
            ui::text(
                &format!("{}", self.crystals.floor() as u32),
                294.0,
                H - 6.0,
                18.0,
                CYAN,
            );
        }
        let w = self.player.weapon;
        ui::text_r(w.name(), W - 62.0, H - 7.0, 15.0, w.color());
        let nx = W - 66.0 - ui::width(w.name(), 15.0);
        crate::icons::weapon_badge(w, vec2(nx - 9.0, H - 12.0), 9.0, 1.0);
        for i in 0..5 {
            let c = if i < self.effective_level() {
                w.color()
            } else {
                Color::new(0.25, 0.25, 0.35, 1.0)
            };
            draw_rectangle(W - 56.0 + i as f32 * 10.0, H - 18.0, 7.0, 10.0, c);
        }
        if self.boss.is_none() {
            ui::text(&self.stage_label(), 10.0, 52.0, 14.0, DIM);
        }
        if let Some(b) = &self.boss
            && b.entered()
            && b.hp > 0.0
        {
            let w = W - 80.0;
            draw_rectangle(40.0, 40.0, w, 8.0, Color::new(0.2, 0.0, 0.05, 0.8));
            draw_rectangle(
                40.0,
                40.0,
                w * (b.hp / b.max_hp).max(0.0),
                8.0,
                Color::from_rgba(255, 60, 110, 255),
            );
            draw_rectangle_lines(40.0, 40.0, w, 8.0, 1.0, WHITE);
            ui::text_c(b.name(), W / 2.0, 66.0, 16.0, PINK);
        }
        let mut y = 70.0;
        for (label, t, c) in self.active_powers() {
            ui::text_r(&format!("{label} {:.0}", t.ceil()), W - 10.0, y, 14.0, c);
            y += 16.0;
        }
    }

    pub fn active_powers(&self) -> Vec<(&'static str, f32, Color)> {
        let mut v = vec![];
        for (label, t, c) in [
            ("FRENZY", self.frenzy_t, Color::from_rgba(255, 80, 60, 255)),
            ("INVINCIBLE", self.star_t, GOLD),
            ("SUPER MAGNET", self.magnet_t, PINK),
            ("SLOW", self.slowmo_t, Color::from_rgba(120, 160, 255, 255)),
            ("WEB", self.web_t, WHITE),
            ("OVERDRIVE", self.overdrive_t, GOLD),
            ("BULWARK", self.wall_t, CYAN),
            (
                "WARP",
                self.timewarp_t,
                Color::from_rgba(170, 140, 255, 255),
            ),
            ("CANNON", self.cannon_t, Color::from_rgba(255, 170, 60, 255)),
        ] {
            if t > 0.0 {
                v.push((label, t, c));
            }
        }
        v
    }

    pub fn stage_label(&self) -> String {
        match (self.cfg.mode, self.wave_kind) {
            (Mode::Survival, _) => format!("WAVE {}", self.wave),
            (Mode::BossRush, _) => format!("BOSS {}/8", self.wave),
            (Mode::Classic, WaveKind::Challenge) => "BONUS".into(),
            (Mode::Classic, _) => format!("WAVE {}", self.wave),
            (_, WaveKind::Challenge) => "BONUS".into(),
            (_, WaveKind::Golden) => "???".into(),
            (_, WaveKind::Boss) => "BOSS".into(),
            (Mode::Campaign, _) if self.cycle == 0 => format!("LEVEL {}/100", self.level()),
            _ => format!("SECTOR {}-{}", self.absolute_sector(), self.sub()),
        }
    }

    pub fn theme_name(&self) -> &'static str {
        THEMES[self.stage().theme.min(THEMES.len() - 1)].name
    }

    pub fn drones_total(&self) -> u32 {
        (self.mods.drones + self.drones_bought).min(3)
    }

    pub fn crystals_visible(&self) -> bool {
        !self.classic()
    }

    fn draw_overlays(&self, sp: &Sprites) {
        if self.boss_warning > 0.0 {
            let on = (self.boss_warning * 4.0) as i32 % 2 == 0;
            draw_rectangle(
                0.0,
                H * 0.4,
                W,
                80.0,
                Color::new(0.6, 0.0, 0.1, if on { 0.45 } else { 0.25 }),
            );
            ui::text_outline(
                "WARNING!",
                W / 2.0,
                H * 0.4 + 40.0,
                44.0,
                if on { RED } else { WHITE },
                Color::from_rgba(40, 0, 10, 255),
            );
            ui::text_c("A BOSS APPROACHES", W / 2.0, H * 0.4 + 66.0, 18.0, WHITE);
        }

        match &self.phase {
            Phase::Intro { t } => {
                let a = (t * 2.0).min(1.0) * ((2.8 - t) * 2.0).clamp(0.0, 1.0);
                let (title, sub) = match self.cfg.mode {
                    Mode::Campaign | Mode::Daily if self.secret_stage => {
                        ("SECTOR ???".to_string(), "GOLDEN DIMENSION".to_string())
                    }
                    Mode::Campaign if self.cycle == 0 => (
                        format!("SECTOR {}/{}", self.sector(), SECTORS),
                        self.theme_name().to_string(),
                    ),
                    Mode::Campaign | Mode::Daily | Mode::Expedition => (
                        format!("SECTOR {}", self.absolute_sector()),
                        self.theme_name().to_string(),
                    ),
                    Mode::BossRush => (
                        format!("BOSS {} OF 9", self.wave),
                        self.theme_name().to_string(),
                    ),
                    _ => (format!("WAVE {}", self.wave), self.theme_name().to_string()),
                };
                ui::text_outline(
                    &title,
                    W / 2.0,
                    H * 0.4,
                    52.0,
                    Color::new(1.0, 0.85, 0.3, a),
                    Color::new(0.1, 0.02, 0.15, a),
                );
                ui::text_c(
                    &sub,
                    W / 2.0,
                    H * 0.4 + 34.0,
                    22.0,
                    Color::new(1.0, 1.0, 1.0, a),
                );
                let mut y = H * 0.4 + 60.0;
                if self.cycle > 0 {
                    ui::text_c(
                        &format!("CYCLE {}", self.cycle + 1),
                        W / 2.0,
                        y,
                        18.0,
                        Color::new(1.0, 0.4, 0.6, a),
                    );
                    y += 22.0;
                }
                if let Some(d) = self.cfg.daily
                    && self.wave == 1
                {
                    ui::text_c(
                        &format!("RULE OF THE DAY: {}", d.name()),
                        W / 2.0,
                        y,
                        18.0,
                        Color::new(1.0, 0.6, 0.3, a),
                    );
                    ui::text_c(
                        d.desc(),
                        W / 2.0,
                        y + 20.0,
                        15.0,
                        Color::new(1.0, 1.0, 1.0, a),
                    );
                }
            }
            Phase::WaveClear { t } => {
                let a = (t * 3.0).min(1.0);
                ui::text_outline(
                    "WAVE CLEAR!",
                    W / 2.0,
                    H * 0.45,
                    36.0,
                    Color::new(0.5, 1.0, 0.6, a),
                    Color::new(0.0, 0.1, 0.05, a),
                );
            }
            Phase::ChallengeEnd {
                t,
                hits,
                total,
                bonus,
            } => {
                let a = (t * 3.0).min(1.0);
                ui::panel(
                    60.0,
                    H * 0.33,
                    W - 120.0,
                    150.0,
                    Color::new(1.0, 0.8, 0.3, a),
                );
                ui::text_c("BONUS RESULT", W / 2.0, H * 0.33 + 36.0, 24.0, GOLD);
                ui::text_c(
                    &format!("HITS  {hits} / {total}"),
                    W / 2.0,
                    H * 0.33 + 72.0,
                    22.0,
                    WHITE,
                );
                ui::text_c(
                    &format!("BONUS  {bonus}"),
                    W / 2.0,
                    H * 0.33 + 102.0,
                    22.0,
                    CYAN,
                );
                if hits == total {
                    let c = if (t * 6.0) as i32 % 2 == 0 {
                        GOLD
                    } else {
                        PINK
                    };
                    ui::text_c("PERFECT!", W / 2.0, H * 0.33 + 134.0, 24.0, c);
                }
            }
            Phase::SectorClear { t, lines, shown } => {
                let a = (t * 2.0).min(1.0);
                let y0 = H * 0.22;
                ui::panel(
                    40.0,
                    y0,
                    W - 80.0,
                    110.0 + lines.len() as f32 * 32.0,
                    Color::new(0.5, 1.0, 0.6, a),
                );
                let title = if self.cfg.mode == Mode::BossRush {
                    "BOSS DOWN!"
                } else {
                    "SECTOR CLEAR!"
                };
                ui::text_outline(
                    title,
                    W / 2.0,
                    y0 + 46.0,
                    40.0,
                    GREEN,
                    Color::from_rgba(0, 30, 10, 255),
                );
                for (i, (label, v, crystal)) in lines.iter().enumerate().take(*shown) {
                    let y = y0 + 90.0 + i as f32 * 32.0;
                    ui::text(label, 64.0, y, 18.0, WHITE);
                    let val = if *crystal {
                        format!("+{v}")
                    } else {
                        format!("{v}")
                    };
                    ui::text_r(&val, W - 64.0, y, 20.0, if *crystal { CYAN } else { GOLD });
                }
                if *shown >= lines.len() && (t * 2.0) as i32 % 2 == 0 {
                    ui::text_c(
                        "ENTER to continue",
                        W / 2.0,
                        y0 + 100.0 + lines.len() as f32 * 32.0,
                        16.0,
                        DIM,
                    );
                }
            }
            Phase::Shop => self.draw_shop(sp),
            Phase::Portal { t } => {
                let a = (t * 1.5).min(1.0);
                let r = t * 140.0;
                for k in 0..6 {
                    let rr = r - k as f32 * 18.0;
                    if rr > 0.0 {
                        draw_circle_lines(
                            W / 2.0,
                            H * 0.4,
                            rr,
                            4.0,
                            ui::hsl(self.time * 0.5 + k as f32 * 0.1, 0.9, 0.6),
                        );
                    }
                }
                let (t1, t2) = if self.enc.void {
                    ("THE VOID OPENS", "What the Swarm was fleeing from is here.")
                } else {
                    (
                        "A PORTAL OPENED",
                        "No ship lost... the Swarm never saw you coming.",
                    )
                };
                ui::text_outline(
                    t1,
                    W / 2.0,
                    H * 0.65,
                    30.0,
                    Color::new(1.0, 0.85, 0.3, a),
                    Color::new(0.1, 0.05, 0.0, a),
                );
                ui::text_c(
                    t2,
                    W / 2.0,
                    H * 0.65 + 28.0,
                    14.0,
                    Color::new(1.0, 1.0, 1.0, a),
                );
            }
            Phase::Credits { t } => self.draw_credits(*t),
            Phase::Victory { sel, t } => {
                draw_rectangle(
                    0.0,
                    0.0,
                    W,
                    H,
                    Color::new(0.0, 0.0, 0.0, (t * 0.8).min(0.7)),
                );
                ui::text_outline(
                    "VICTORY!",
                    W / 2.0,
                    H * 0.24,
                    60.0,
                    GOLD,
                    Color::from_rgba(60, 20, 0, 255),
                );
                let true_end = self.cfg.mode == Mode::Campaign
                    && self.cfg.diff as u8 >= 2
                    && self.continues_used == 0;
                let lines: Vec<&str> = match self.cfg.mode {
                    Mode::Campaign if self.enc.void_done => vec![
                        "The Devourer retreated into the dark between the stars.",
                        "The Queen was right: the Swarm only wanted to pass through.",
                        "Now the hive lives, and you know the truth.",
                        "Thanks for playing to the very end, pilot.",
                    ],
                    Mode::Campaign if true_end => vec![
                        "The Supreme Queen fell without you ever retreating.",
                        "In the ashes of the hive, one last truth:",
                        "the Swarm was never the enemy. It was fleeing.",
                        "From what? ...that's for the next cycle.",
                    ],
                    Mode::Campaign => vec![
                        "The heart of the Swarm has been destroyed.",
                        "The galaxy breathes again... for now.",
                        "But not every question was answered.",
                        "Thanks for playing, pilot.",
                    ],
                    Mode::BossRush => vec![
                        "Every boss has been defeated.",
                        "Your time has been recorded.",
                    ],
                    _ => vec!["Challenge complete!", "Come back tomorrow for a new rule."],
                };
                for (i, l) in lines.iter().enumerate() {
                    ui::text_c(
                        l,
                        W / 2.0,
                        H * 0.32 + i as f32 * 24.0,
                        16.0,
                        if true_end { GOLD } else { WHITE },
                    );
                }
                ui::text_c(
                    &format!("SCORE {}", self.score),
                    W / 2.0,
                    H * 0.52,
                    26.0,
                    CYAN,
                );
                if self.cfg.mode == Mode::BossRush {
                    let t = self.run_time;
                    ui::text_c(
                        &format!("TIME {:02}:{:02}", (t / 60.0) as u32, (t % 60.0) as u32),
                        W / 2.0,
                        H * 0.57,
                        22.0,
                        GOLD,
                    );
                }
                let items = if self.cfg.mode == Mode::Campaign {
                    vec![
                        format!("CONTINUE - CYCLE {}", self.cycle + 2),
                        "END THE RUN".to_string(),
                    ]
                } else {
                    vec!["END THE RUN".to_string()]
                };
                let sel = if items.len() == 1 { 0 } else { *sel };
                ui::menu(&items, sel, W / 2.0, H * 0.66, 36.0, 22.0, self.time);
            }
            Phase::Continue { t } => {
                draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.0, 0.6));
                ui::text_outline(
                    "CONTINUE?",
                    W / 2.0,
                    H * 0.36,
                    50.0,
                    GOLD,
                    Color::from_rgba(40, 20, 0, 255),
                );
                let n = t.ceil() as i32;
                let pulse = 1.0 + (t.fract() * 6.0).min(1.0) * 0.2;
                ui::text_outline(
                    &n.to_string(),
                    W / 2.0,
                    H * 0.52,
                    90.0 * pulse,
                    WHITE,
                    Color::from_rgba(40, 0, 10, 255),
                );
                ui::text_c(
                    &format!("CREDITS: {}", self.continues_left),
                    W / 2.0,
                    H * 0.6,
                    20.0,
                    CYAN,
                );
                ui::text_c(
                    "ENTER continues (lose half your points)",
                    W / 2.0,
                    H * 0.65,
                    15.0,
                    WHITE,
                );
                ui::text_c("ESC gives up", W / 2.0, H * 0.68, 15.0, DIM);
            }
            Phase::GameOver { t } => {
                let a = (t * 1.5).min(1.0);
                draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.0, a * 0.5));
                ui::text_outline(
                    "GAME OVER",
                    W / 2.0,
                    H * 0.42,
                    52.0,
                    Color::new(1.0, 0.3, 0.3, a),
                    Color::new(0.1, 0.0, 0.0, a),
                );
                ui::text_c(
                    &format!("SCORE {}", self.score),
                    W / 2.0,
                    H * 0.42 + 44.0,
                    24.0,
                    Color::new(1.0, 1.0, 1.0, a),
                );
            }
            _ => {}
        }

        if let Some((title, sub, t)) = &self.banner {
            let a = t.min(1.0).min((3.0 - t) * 4.0).clamp(0.0, 1.0);
            let y = H * 0.62;
            let h = if sub.is_empty() { 40.0 } else { 62.0 };
            draw_rectangle(0.0, y - 30.0, W, h, Color::new(0.05, 0.02, 0.15, 0.75 * a));
            ui::text_c(title, W / 2.0, y, 22.0, Color::new(1.0, 0.85, 0.35, a));
            let lines = ui::wrap(sub, 15.0, W - 30.0);
            for (i, l) in lines.iter().take(2).enumerate() {
                ui::text_c(
                    l,
                    W / 2.0,
                    y + 20.0 + i as f32 * 16.0,
                    15.0,
                    Color::new(1.0, 1.0, 1.0, a),
                );
            }
        }
    }

    fn draw_credits(&self, t: f32) {
        draw_rectangle(
            0.0,
            0.0,
            W,
            H,
            Color::new(0.0, 0.0, 0.0, (t * 0.5).min(0.85)),
        );
        let lines: &[(&str, f32, Color)] = &[
            ("STARDUST", 54.0, GOLD),
            ("", 20.0, WHITE),
            ("CREATED BY", 18.0, PINK),
            ("Allan Somensi", 24.0, WHITE),
            ("", 20.0, WHITE),
            ("SOUNDTRACK", 18.0, PINK),
            ("Chiptune synthesized by the game itself,", 16.0, WHITE),
            ("note by note", 16.0, WHITE),
            ("", 20.0, WHITE),
            ("ART", 18.0, PINK),
            ("Pixel art written letter by letter", 16.0, WHITE),
            ("", 20.0, WHITE),
            ("MADE WITH", 18.0, PINK),
            ("Rust + macroquad", 16.0, WHITE),
            ("", 20.0, WHITE),
            ("INSPIRED BY THE SHAREWARE GAMES", 18.0, PINK),
            ("that we used to play on the family PC", 16.0, WHITE),
            ("", 20.0, WHITE),
            ("CAST", 18.0, PINK),
            ("Drones, Wasps, Beetles, Gunners,", 15.0, WHITE),
            ("Splitters, Kamikazes, Shieldbearers, Weavers,", 15.0, WHITE),
            ("Carriers, Chameleons, Mimics, Thieves,", 15.0, WHITE),
            ("Minelayers, Jumpers and a tamed Comet", 15.0, WHITE),
            ("", 20.0, WHITE),
            ("IN MEMORY OF THE STARDUST PROBE", 18.0, PINK),
            ("It went looking for comet dust", 15.0, WHITE),
            ("and brought back a story", 15.0, WHITE),
            ("", 20.0, WHITE),
            ("SPECIAL THANKS", 18.0, PINK),
            ("To you, who played this far.", 18.0, WHITE),
            ("", 30.0, WHITE),
        ];
        let mut y = H + 20.0 - t * 34.0;
        for (s, size, c) in lines {
            if y > -40.0 && y < H + 40.0 {
                ui::text_c(s, W / 2.0, y, *size, *c);
            }
            y += size * 1.6;
        }
        if t > 4.0 {
            ui::text_c("ENTER skips", W / 2.0, H - 12.0, 13.0, DIM);
        }
    }

    fn draw_shop(&self, sp: &Sprites) {
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.05, 0.85));
        ui::text_outline(
            "SHOP",
            W / 2.0,
            60.0,
            44.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let sub = if self.grudge {
            "The Merchant glares at you: prices +30%"
        } else {
            "Spend your crystals before the next sector"
        };
        ui::text_c(
            sub,
            W / 2.0,
            84.0,
            14.0,
            if self.grudge { RED } else { DIM },
        );
        sp.crystal[((self.time * 4.0) as usize) % 2].draw(
            vec2(W / 2.0 - 40.0, 108.0),
            3.0,
            0.0,
            WHITE,
            false,
        );
        ui::text(
            &format!("{}", self.crystals.floor() as u32),
            W / 2.0 - 24.0,
            116.0,
            26.0,
            CYAN,
        );

        let y0 = 134.0;
        let row_h = 46.0;
        for (i, it) in self.shop_offers.iter().enumerate() {
            let y = y0 + i as f32 * row_h;
            let sel = self.shop_sel == i;
            let blocked = self.shop_block(*it);
            let price = self.price(*it);
            let afford = self.crystals.floor() as u32 >= price;
            let bg = if sel {
                Color::new(0.3, 0.25, 0.05, 0.6)
            } else {
                Color::new(0.1, 0.1, 0.2, 0.5)
            };
            draw_rectangle(20.0, y, W - 40.0, row_h - 5.0, bg);
            if sel {
                draw_rectangle_lines(20.0, y, W - 40.0, row_h - 5.0, 2.0, GOLD);
            }
            if i == 3 {
                draw_line(
                    24.0,
                    y - 3.0,
                    W - 24.0,
                    y - 3.0,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.15),
                );
            }
            match it {
                Item::Weapon(w) => crate::icons::weapon_badge(*w, vec2(44.0, y + 20.0), 14.0, 1.0),
                _ => sp
                    .icon(it.icon())
                    .draw(vec2(44.0, y + 20.0), 2.6, 0.0, WHITE, false),
            }
            let name_c = if blocked.is_some() { DIM } else { WHITE };
            ui::text(&it.name(), 68.0, y + 18.0, 17.0, name_c);
            ui::text(it.desc(), 68.0, y + 35.0, 12.0, DIM);
            match blocked {
                Some(why) => ui::text_r(why, W - 32.0, y + 27.0, 15.0, DIM),
                None => ui::text_r(
                    &format!("{price}"),
                    W - 32.0,
                    y + 27.0,
                    21.0,
                    if afford { CYAN } else { RED },
                ),
            }
        }
        let n = self.shop_offers.len();
        let y = y0 + n as f32 * row_h + 8.0;
        let rp = self.reroll_price();
        let label = if rp == 0 {
            "REROLL OFFERS (FREE)".to_string()
        } else {
            format!("REROLL OFFERS ({rp})")
        };
        let c = if self.shop_sel == n { GOLD } else { DIM };
        ui::text_c(&label, W / 2.0, y + 14.0, 18.0, c);
        let next = match self.cfg.mode {
            Mode::BossRush => format!("FACE THE BOSS {}", self.wave + 1),
            _ => format!("DEPART FOR SECTOR {}", self.absolute_sector() + 1),
        };
        if self.shop_sel == n + 1 {
            ui::text_c(&format!("> {next} <"), W / 2.0, y + 48.0, 22.0, GOLD);
        } else {
            ui::text_c(&next, W / 2.0, y + 48.0, 20.0, DIM);
        }
        let p = &self.player;
        let status = format!(
            "SHIPS {}  BOMBS {}  {} LV{}  DRONES {}",
            p.lives,
            p.bombs,
            p.weapon.name(),
            p.level,
            self.drones_total()
        );
        ui::text_c(&status, W / 2.0, H - 30.0, 13.0, DIM);
        let power = format!(
            "FIREPOWER x{:.2}   (weapon {} x{:.2} · cores {} x{:.2})",
            self.power_mul(),
            "*".repeat(p.weapon.tier() as usize),
            p.weapon.tier_mul(),
            self.arcade.core,
            self.core_mul()
        );
        ui::text_c(&power, W / 2.0, H - 12.0, 13.0, GOLD);
    }
}

fn shield_blocks(e: &Enemy, from: Vec2) -> bool {
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
