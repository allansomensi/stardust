//! Collectible cards: passive bonuses you equip before each run.

use crate::sprites::Icon;
use macroquad::prelude::Color;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "COMMON",
            Rarity::Rare => "RARE",
            Rarity::Epic => "EPIC",
            Rarity::Legendary => "LEGENDARY",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(170, 180, 200, 255),
            Rarity::Rare => Color::from_rgba(80, 160, 255, 255),
            Rarity::Epic => Color::from_rgba(200, 100, 255, 255),
            Rarity::Legendary => Color::from_rgba(255, 200, 50, 255),
        }
    }
}

pub struct CardDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub rarity: Rarity,
    pub icon: Icon,
    /// Secret cards never drop at random: only through secrets.
    pub secret: bool,
}

const fn c(name: &'static str, desc: &'static str, rarity: Rarity, icon: Icon) -> CardDef {
    CardDef {
        name,
        desc,
        rarity,
        icon,
        secret: false,
    }
}
const fn s(name: &'static str, desc: &'static str, rarity: Rarity, icon: Icon) -> CardDef {
    CardDef {
        name,
        desc,
        rarity,
        icon,
        secret: true,
    }
}

use Icon::*;
use Rarity::*;

pub const CARDS: &[CardDef] = &[
    c(
        "Reinforced Hull",
        "+1 life at the start of the run.",
        Common,
        Heart,
    ),
    c(
        "Extra Gunpowder",
        "+1 bomb at the start of the run.",
        Common,
        Bomb,
    ),
    c(
        "Portable Magnet",
        "Crystal magnet radius +60%.",
        Common,
        Magnet,
    ),
    c("Turbo Engine", "Ship 12% faster.", Common, Engine),
    c("Light Trigger", "Fire rate +10%.", Common, Bolt),
    c(
        "Lucky Crystal",
        "Crystals are worth 25% more.",
        Common,
        Crystal,
    ),
    c(
        "Strong Start",
        "Your weapon starts at level 2.",
        Common,
        Star,
    ),
    c("Piggy Bank", "Starts with 40 crystals.", Common, Bag),
    c(
        "Starting Shield",
        "Starts with a shield that absorbs one hit.",
        Rare,
        Shield,
    ),
    c("Long Combo", "Combo window 50% longer.", Rare, Clock),
    c("Hunter", "Diving enemies are worth triple.", Rare, Target),
    c("Card Fisher", "Cards drop twice as often.", Rare, Card),
    c(
        "Regular Customer",
        "Everything in the shop costs 20% less.",
        Rare,
        Bag,
    ),
    c("Recharge", "Gain 1 bomb every sector.", Rare, Bomb),
    c(
        "Phoenix",
        "Once per run, revives when you lose your last life.",
        Epic,
        Flame,
    ),
    c("Drones", "Two drones fire at your side.", Epic, Drone),
    c("Full Arsenal", "Your weapon starts at level 3.", Epic, Star),
    c("Lightning Bomb", "Bombs deal double damage.", Epic, Bolt),
    c("Last Breath", "When hit, drops a free bomb.", Epic, Bomb),
    c(
        "Heart of the Swarm",
        "+1 life and a fresh shield every sector.",
        Legendary,
        Heart,
    ),
    c("Slow Time", "Enemy shots 20% slower.", Legendary, Clock),
    c(
        "Golden Multiplier",
        "Points x1.5 and max combo x10.",
        Legendary,
        Star,
    ),
    c(
        "Constellation",
        "Every 6 shots, a fan of 6 stars.",
        Legendary,
        Disc,
    ),
    c(
        "Crystal Wings",
        "Crystals come to you from anywhere.",
        Legendary,
        Crystal,
    ),
    c(
        "Dangerous Graze",
        "Grazes give double points and energy.",
        Rare,
        Target,
    ),
    c(
        "Spare Battery",
        "Starts with the ability charged.",
        Common,
        Bolt,
    ),
    c(
        "Quick Overdrive",
        "Ability charges 30% faster.",
        Rare,
        Engine,
    ),
    c("Escort", "An escort drone follows you.", Epic, Drone),
    c(
        "Varied Arsenal",
        "Capsules cycle through 4 weapons and last longer.",
        Common,
        Star,
    ),
    c("Aim", "Your shots fly 15% faster.", Common, Target),
    c(
        "Inheritance",
        "Losing a ship doesn't lower your weapon level.",
        Rare,
        Shield,
    ),
    c(
        "Living Crystal",
        "Every 100 crystals collected, +1 bomb.",
        Epic,
        Crystal,
    ),
    c("Investor", "The first shop reroll is free.", Rare, Bag),
    c(
        "Counterattack",
        "When hit, every enemy takes damage.",
        Epic,
        Flame,
    ),
    c("Card Magnet", "Cards fly to you.", Legendary, Magnet),
    c("Lucky", "Power-ups drop 50% more often.", Common, Star),
    s(
        "Queen's Crown",
        "Starts every sector in Frenzy.",
        Legendary,
        Star,
    ),
    s("Cosmic Cow", "+1 life and +1 bomb. Moo.", Legendary, Heart),
    s(
        "Konami Code",
        "+3 lives. You know what you did.",
        Legendary,
        Heart,
    ),
    s("Creator", "Crystals x1.5 and points x1.2.", Legendary, Disc),
    s(
        "Golden Dimension",
        "A Golden Scarab appears every sector.",
        Legendary,
        Crystal,
    ),
    s("Pacifist", "Bonus stages are worth double.", Epic, Shield),
    c(
        "Extra Pocket",
        "Starts the run with the LASER at level 2.",
        Common,
        Star,
    ),
    c("Lucrative Contract", "Double contract rewards.", Rare, Bag),
    c(
        "Early Fever",
        "Fever starts at combo 35 instead of 50.",
        Rare,
        Flame,
    ),
    c("Long Fever", "Fever lasts 50% longer.", Common, Clock),
    c(
        "Sharp Ricochet",
        "Bouncing shots get 2 extra bounces.",
        Common,
        Bolt,
    ),
    c("Thermal Shrapnel", "Explosions 40% bigger.", Rare, Bomb),
    c(
        "Absolute Zero",
        "Enemies stay frozen twice as long.",
        Epic,
        Shield,
    ),
    c(
        "Queen Bee",
        "The Hive releases twice as many bees.",
        Epic,
        Star,
    ),
    c(
        "Event Horizon",
        "Black holes last twice as long and suck in crystals.",
        Legendary,
        Crystal,
    ),
    c(
        "Elite Hunter",
        "Elites always drop a capsule and sometimes a card.",
        Rare,
        Target,
    ),
    c("Light Fingers", "Thieves are 40% slower.", Common, Bag),
    c("Keen Eye", "Mimics show up marked in red.", Common, Target),
    c("Longing", "+1 continue per run.", Common, Heart),
    c(
        "Reactive Shield",
        "The shield comes back after 25 s without taking damage.",
        Epic,
        Shield,
    ),
    c(
        "Capsule Magnet",
        "Capsules and power-ups come to you.",
        Rare,
        Magnet,
    ),
    s(
        "Cosmic Calf",
        "A calf follows you and gathers crystals. Moo.",
        Legendary,
        Heart,
    ),
    s(
        "Golden Record",
        "PULSAR drops from capsules and its stars are worth double.",
        Epic,
        Disc,
    ),
    s(
        "Eye of the Devourer",
        "Points x1.3 and enemy shots 10% slower.",
        Legendary,
        Target,
    ),
    c(
        "Ancient Amulet",
        "Half of the curses can't touch you.",
        Rare,
        Shield,
    ),
    c(
        "Open Rift",
        "Minigame rifts drop 2x more often.",
        Rare,
        Target,
    ),
    c("Rock Salt", "Curses drop 50% less often.", Common, Crystal),
    c(
        "Comet Dust",
        "Crystals +15% and 20% bigger magnet.",
        Epic,
        Star,
    ),
    s(
        "Contact Signal",
        "The mysterious Mothership shows up twice as often.",
        Epic,
        Target,
    ),
];

/// Card indices (same order as CARDS).
pub mod id {
    pub const HULL: usize = 0;
    pub const GUNPOWDER: usize = 1;
    pub const MAGNET: usize = 2;
    pub const TURBO: usize = 3;
    pub const TRIGGER: usize = 4;
    pub const LUCK: usize = 5;
    pub const STRONG_START: usize = 6;
    pub const PIGGY_BANK: usize = 7;
    pub const SHIELD: usize = 8;
    pub const COMBO: usize = 9;
    pub const HUNTER: usize = 10;
    pub const CARD_FISHER: usize = 11;
    pub const REGULAR: usize = 12;
    pub const RECHARGE: usize = 13;
    pub const PHOENIX: usize = 14;
    pub const DRONES: usize = 15;
    pub const ARSENAL: usize = 16;
    pub const LIGHTNING_WAVE: usize = 17;
    pub const LAST_BREATH: usize = 18;
    pub const HEART: usize = 19;
    pub const SLOW_TIME: usize = 20;
    pub const GOLDEN: usize = 21;
    pub const CONSTELLATION: usize = 22;
    pub const WINGS: usize = 23;
    pub const DANGEROUS_GRAZE: usize = 24;
    pub const BATTERY: usize = 25;
    pub const QUICK_OVERDRIVE: usize = 26;
    pub const ESCORT: usize = 27;
    pub const VARIED: usize = 28;
    pub const AIM: usize = 29;
    pub const INHERITANCE: usize = 30;
    pub const LIVING_CRYSTAL: usize = 31;
    pub const INVESTOR: usize = 32;
    pub const COUNTER: usize = 33;
    pub const CARD_MAGNET: usize = 34;
    pub const LUCKY: usize = 35;
    pub const CROWN: usize = 36;
    pub const COW: usize = 37;
    pub const KONAMI: usize = 38;
    pub const CREATOR: usize = 39;
    pub const DIMENSION: usize = 40;
    pub const PACIFIST: usize = 41;
    pub const POCKET: usize = 42;
    pub const LUCRATIVE: usize = 43;
    pub const EARLY_FEVER: usize = 44;
    pub const LONG_FEVER: usize = 45;
    pub const SHARP_RICOCHET: usize = 46;
    pub const THERMAL: usize = 47;
    pub const ZERO: usize = 48;
    pub const BEES: usize = 49;
    pub const HORIZON: usize = 50;
    pub const ELITE: usize = 51;
    pub const LIGHT_FINGERS: usize = 52;
    pub const KEEN_EYE: usize = 53;
    pub const LONGING: usize = 54;
    pub const REACTIVE: usize = 55;
    pub const CAPSULE_MAGNET: usize = 56;
    pub const CALF: usize = 57;
    pub const GOLDEN_RECORD: usize = 58;
    pub const DEVOURER: usize = 59;
    pub const AMULET: usize = 60;
    pub const OPEN_RIFT: usize = 61;
    pub const ROCK_SALT: usize = 62;
    pub const COMET_DUST: usize = 63;
    pub const CONTACT: usize = 64;
}

/// Rolls a card (never a secret one). `luck` 0 = normal, higher = pulls toward rarer cards.
pub fn roll(luck: u32) -> usize {
    use macroquad::rand::gen_range;
    let mut r = gen_range(0.0f32, 100.0);
    r += luck as f32 * 25.0;
    let rarity = if r >= 112.0 {
        Legendary
    } else if r >= 92.0 {
        Epic
    } else if r >= 65.0 {
        Rare
    } else {
        Common
    };
    let pool: Vec<usize> = CARDS
        .iter()
        .enumerate()
        .filter(|(_, c)| c.rarity == rarity && !c.secret)
        .map(|(i, _)| i)
        .collect();
    pool[gen_range(0, pool.len())]
}

/// Number of cards that can be equipped as the collection grows.
pub fn slots_for(unique: usize) -> usize {
    match unique {
        45.. => 5,
        30.. => 4,
        16.. => 3,
        8.. => 2,
        _ => 1,
    }
}

#[derive(Default, Clone)]
pub struct Mods {
    pub ufo_more: bool,
    pub curse_mul: f32,
    pub rift_mul: f32,
    pub amulet: bool,
    pub extra_lives: i32,
    pub extra_bombs: i32,
    pub magnet_mul: f32,
    pub speed_mul: f32,
    pub fire_mul: f32,
    pub crystal_mul: f32,
    pub start_level: i32,
    pub start_crystals: u32,
    pub start_shield: bool,
    pub combo_mul: f32,
    pub dive_mul: u32,
    pub card_mul: f32,
    pub shop_mul: f32,
    pub bomb_per_sector: bool,
    pub phoenix: bool,
    pub drones: u32,
    pub bomb_power: f32,
    pub last_breath: bool,
    pub shield_per_sector: bool,
    pub bullet_speed: f32,
    pub score_mul: f32,
    pub max_combo: u32,
    pub star_fan: bool,
    pub magnet_all: bool,
    pub graze_mul: f32,
    pub meter_full: bool,
    pub meter_mul: f32,
    pub capsule_four: bool,
    pub shot_speed: f32,
    pub keep_level: bool,
    pub crystal_bomb: bool,
    pub free_reroll: bool,
    pub counter: bool,
    pub card_magnet: bool,
    pub powerup_mul: f32,
    pub crown: bool,
    pub moo: bool,
    pub golden_every: bool,
    pub bonus_mul: u32,
    pub second_laser: bool,
    pub contract_mul: u32,
    pub fever_start: u32,
    pub fever_mul: f32,
    pub bounce_extra: f32,
    pub blast_mul: f32,
    pub freeze_mul: f32,
    pub bee_mul: f32,
    pub hole_mul: f32,
    pub hole_magnet: bool,
    pub elite_drop: bool,
    pub thief_slow: bool,
    pub mimic_mark: bool,
    pub extra_continue: u32,
    pub regen_shield: bool,
    pub capsule_magnet: bool,
    pub calf: bool,
    pub pulsar_card: bool,
}

impl Mods {
    pub fn from_cards(equipped: &[usize]) -> Mods {
        use id::*;
        let mut m = Mods {
            magnet_mul: 1.0,
            speed_mul: 1.0,
            fire_mul: 1.0,
            crystal_mul: 1.0,
            start_level: 1,
            combo_mul: 1.0,
            dive_mul: 2,
            card_mul: 1.0,
            shop_mul: 1.0,
            bomb_power: 1.0,
            bullet_speed: 1.0,
            score_mul: 1.0,
            max_combo: 5,
            graze_mul: 1.0,
            meter_mul: 1.0,
            shot_speed: 1.0,
            powerup_mul: 1.0,
            bonus_mul: 1,
            contract_mul: 1,
            fever_start: 50,
            fever_mul: 1.0,
            blast_mul: 1.0,
            freeze_mul: 1.0,
            bee_mul: 1.0,
            hole_mul: 1.0,
            curse_mul: 1.0,
            rift_mul: 1.0,
            ..Default::default()
        };
        for &c in equipped {
            match c {
                HULL => m.extra_lives += 1,
                GUNPOWDER => m.extra_bombs += 1,
                MAGNET => m.magnet_mul = 1.6,
                TURBO => m.speed_mul = 1.12,
                TRIGGER => m.fire_mul = 1.1,
                LUCK => m.crystal_mul *= 1.25,
                STRONG_START => m.start_level = m.start_level.max(2),
                PIGGY_BANK => m.start_crystals += 40,
                SHIELD => m.start_shield = true,
                COMBO => m.combo_mul = 1.5,
                HUNTER => m.dive_mul = 3,
                CARD_FISHER => m.card_mul = 2.0,
                REGULAR => m.shop_mul = 0.8,
                RECHARGE => m.bomb_per_sector = true,
                PHOENIX => m.phoenix = true,
                DRONES => m.drones += 2,
                ARSENAL => m.start_level = 3,
                LIGHTNING_WAVE => m.bomb_power = 2.0,
                LAST_BREATH => m.last_breath = true,
                HEART => {
                    m.extra_lives += 1;
                    m.shield_per_sector = true;
                }
                SLOW_TIME => m.bullet_speed = 0.8,
                GOLDEN => {
                    m.score_mul *= 1.5;
                    m.max_combo = 10;
                }
                CONSTELLATION => m.star_fan = true,
                WINGS => m.magnet_all = true,
                DANGEROUS_GRAZE => m.graze_mul = 2.0,
                BATTERY => m.meter_full = true,
                QUICK_OVERDRIVE => m.meter_mul = 1.3,
                ESCORT => m.drones += 1,
                VARIED => m.capsule_four = true,
                AIM => m.shot_speed = 1.15,
                INHERITANCE => m.keep_level = true,
                LIVING_CRYSTAL => m.crystal_bomb = true,
                INVESTOR => m.free_reroll = true,
                COUNTER => m.counter = true,
                CARD_MAGNET => m.card_magnet = true,
                LUCKY => m.powerup_mul = 1.5,
                CROWN => m.crown = true,
                COW => {
                    m.extra_lives += 1;
                    m.extra_bombs += 1;
                    m.moo = true;
                }
                KONAMI => m.extra_lives += 3,
                CREATOR => {
                    m.crystal_mul *= 1.5;
                    m.score_mul *= 1.2;
                }
                DIMENSION => m.golden_every = true,
                PACIFIST => m.bonus_mul = 2,
                POCKET => m.second_laser = true,
                LUCRATIVE => m.contract_mul = 2,
                EARLY_FEVER => m.fever_start = 35,
                LONG_FEVER => m.fever_mul = 1.5,
                SHARP_RICOCHET => m.bounce_extra = 2.0,
                THERMAL => m.blast_mul = 1.4,
                ZERO => m.freeze_mul = 2.0,
                BEES => m.bee_mul = 2.0,
                HORIZON => {
                    m.hole_mul = 2.0;
                    m.hole_magnet = true;
                }
                ELITE => m.elite_drop = true,
                LIGHT_FINGERS => m.thief_slow = true,
                KEEN_EYE => m.mimic_mark = true,
                LONGING => m.extra_continue += 1,
                REACTIVE => m.regen_shield = true,
                CAPSULE_MAGNET => m.capsule_magnet = true,
                CALF => m.calf = true,
                GOLDEN_RECORD => m.pulsar_card = true,
                DEVOURER => {
                    m.score_mul *= 1.3;
                    m.bullet_speed *= 0.9;
                }
                AMULET => m.amulet = true,
                CONTACT => m.ufo_more = true,
                OPEN_RIFT => m.rift_mul *= 2.0,
                ROCK_SALT => m.curse_mul *= 0.5,
                COMET_DUST => {
                    m.crystal_mul *= 1.15;
                    m.magnet_mul *= 1.2;
                }
                _ => {}
            }
        }
        m.drones = m.drones.min(3);
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rarity_has_rollable_cards() {
        for r in [Common, Rare, Epic, Legendary] {
            assert!(CARDS.iter().any(|c| c.rarity == r && !c.secret));
        }
    }

    #[test]
    fn indices_match_names() {
        assert_eq!(CARDS[id::CONSTELLATION].name, "Constellation");
        assert_eq!(CARDS[id::PACIFIST].name, "Pacifist");
        assert_eq!(CARDS[id::COW].name, "Cosmic Cow");
        assert_eq!(CARDS.len(), 65);
        assert_eq!(CARDS[id::DEVOURER].name, "Eye of the Devourer");
    }

    #[test]
    fn roll_never_gives_secret() {
        for _ in 0..500 {
            assert!(!CARDS[roll(2)].secret);
        }
    }

    #[test]
    fn slots_by_collection() {
        assert_eq!(slots_for(0), 1);
        assert_eq!(slots_for(8), 2);
        assert_eq!(slots_for(16), 3);
        assert_eq!(slots_for(30), 4);
    }

    #[test]
    fn effects_match() {
        let m = Mods::from_cards(&[id::HULL, id::HEART, id::ARSENAL, id::DRONES, id::ESCORT]);
        assert_eq!(m.extra_lives, 2);
        assert_eq!(m.start_level, 3);
        assert_eq!(m.drones, 3);
        assert!(m.shield_per_sector);
    }
}
