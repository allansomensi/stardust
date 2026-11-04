//! Hangar ships and weapons.

use crate::progress::ach;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ability {
    Overdrive,
    Dash,
    Bulwark,
    StarPulse,
    TimeWarp,
    Cannon,
    Nostalgia,
    Aerogel,
    Loot,
    Reflect,
}

pub enum Unlock {
    Rank(usize),
    Ach(usize),
}

pub struct ShipDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub speed: f32,
    pub lives: i32,
    pub fire: f32,
    pub dmg: f32,
    pub hit_r: f32,
    pub ability: Ability,
    pub ab_name: &'static str,
    pub ab_desc: &'static str,
    /// Fraction of the bar spent when using the ability.
    pub ab_cost: f32,
    pub unlock: Unlock,
}

pub const SHIPS: &[ShipDef] = &[
    ShipDef {
        name: "VANGUARD",
        desc: "Balanced and reliable. Every recruit's ship.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Overdrive,
        ab_name: "OVERDRIVE",
        ab_desc: "5 s firing twice as fast, one weapon level higher.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(0),
    },
    ShipDef {
        name: "FALCON",
        desc: "Fast and small, but with one less life.",
        speed: 1.25,
        lives: -1,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 3.5,
        ability: Ability::Dash,
        ab_name: "DASH",
        ab_desc: "An instant, invulnerable burst. Costs 1/3 of the bar.",
        ab_cost: 0.34,
        unlock: Unlock::Rank(3),
    },
    ShipDef {
        name: "BASTION",
        desc: "Slow and armored: +1 life and starts with a shield.",
        speed: 0.85,
        lives: 1,
        fire: 0.95,
        dmg: 1.0,
        hit_r: 5.5,
        ability: Ability::Bulwark,
        ab_name: "BULWARK",
        ab_desc: "Raises a shot-blocking barrier for 5 s.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(6),
    },
    ShipDef {
        name: "VEGA",
        desc: "Channels stars: every 6 shots, a star fan.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::StarPulse,
        ab_name: "STAR PULSE",
        ab_desc: "A wave of light wipes nearby shots and hurts enemies.",
        ab_cost: 0.6,
        unlock: Unlock::Rank(9),
    },
    ShipDef {
        name: "PHANTOM",
        desc: "Ethereal. Smaller hitbox and mastery over time.",
        speed: 1.05,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.0,
        ability: Ability::TimeWarp,
        ab_name: "TIME WARP",
        ab_desc: "5 s with enemies and shots almost frozen.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(12),
    },
    ShipDef {
        name: "COLOSSUS",
        desc: "Fires slowly, but each shot deals 60% more.",
        speed: 0.88,
        lives: 0,
        fire: 0.75,
        dmg: 1.6,
        hit_r: 5.0,
        ability: Ability::Cannon,
        ab_name: "STAR CANNON",
        ab_desc: "A giant, continuous beam for 3 s.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(15),
    },
    ShipDef {
        name: "PROTOTYPE",
        desc: "The first ship ever made for this game. Built from nostalgia.",
        speed: 1.0,
        lives: 0,
        fire: 1.1,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Nostalgia,
        ab_name: "NOSTALGIA",
        ab_desc: "Everything on screen turns 2000s and takes damage.",
        ab_cost: 1.0,
        unlock: Unlock::Ach(ach::KONAMI),
    },
    ShipDef {
        name: "PROBE",
        desc: "The ship of the Stardust mission. 50% bigger crystal magnet.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Aerogel,
        ab_name: "AEROGEL",
        ab_desc: "4 s catching nearby shots: each one becomes a crystal.",
        ab_cost: 1.0,
        unlock: Unlock::Ach(ach::SECRET_ROOM),
    },
    ShipDef {
        name: "CORSAIR",
        desc: "Fast and greedy: +20% crystals.",
        speed: 1.15,
        lives: 0,
        fire: 1.05,
        dmg: 0.95,
        hit_r: 4.0,
        ability: Ability::Loot,
        ab_name: "LOOT",
        ab_desc: "5 s pulling every item on screen (except curses).",
        ab_cost: 0.7,
        unlock: Unlock::Ach(ach::ARCADE_MASTER),
    },
    ShipDef {
        name: "AEGIS",
        desc: "Heavy and patient: +1 life and 10% stronger shots.",
        speed: 0.9,
        lives: 1,
        fire: 0.95,
        dmg: 1.1,
        hit_r: 5.0,
        ability: Ability::Reflect,
        ab_name: "REFLECT",
        ab_desc: "3 s sending nearby enemy shots back.",
        ab_cost: 0.8,
        unlock: Unlock::Ach(ach::PERFECTIONIST),
    },
    ShipDef {
        name: "VALKYRIE",
        desc: "A war machine: fast, with a hair trigger.",
        speed: 1.15,
        lives: 0,
        fire: 1.2,
        dmg: 1.0,
        hit_r: 4.0,
        ability: Ability::Overdrive,
        ab_name: "OVERDRIVE",
        ab_desc: "5 s firing twice as fast, one weapon level higher.",
        ab_cost: 0.8,
        unlock: Unlock::Ach(ach::VALKYRIE),
    },
    ShipDef {
        name: "ECLIPSE",
        desc: "Born in the dark: tiny hitbox, 20% stronger shots, one less life.",
        speed: 1.05,
        lives: -1,
        fire: 1.0,
        dmg: 1.2,
        hit_r: 3.2,
        ability: Ability::TimeWarp,
        ab_name: "TIME WARP",
        ab_desc: "5 s with enemies and shots almost frozen.",
        ab_cost: 0.85,
        unlock: Unlock::Ach(ach::DARK_DUEL),
    },
    ShipDef {
        name: "TWIN STAR",
        desc: "Made for two: +1 life and a star fan every 6 shots.",
        speed: 1.0,
        lives: 1,
        fire: 1.05,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::StarPulse,
        ab_name: "STAR PULSE",
        ab_desc: "A wave of light wipes nearby shots and hurts enemies.",
        ab_cost: 0.55,
        unlock: Unlock::Ach(ach::TWIN_STARS),
    },
];

pub fn ship_unlocked(i: usize, rank: usize, achs: &[bool]) -> bool {
    match SHIPS[i].unlock {
        Unlock::Rank(r) => rank >= r,
        Unlock::Ach(a) => achs.get(a).copied().unwrap_or(false),
    }
}

pub fn unlock_hint(i: usize) -> String {
    match SHIPS[i].unlock {
        Unlock::Rank(r) => format!("Rank: {}", crate::progress::RANKS[r]),
        Unlock::Ach(a) if a == ach::SECRET_ROOM => "A Honor Insignia".to_string(),
        Unlock::Ach(a) if a == ach::ARCADE_MASTER => "Arcade Insignia".to_string(),
        Unlock::Ach(a) if a == ach::PERFECTIONIST => "Perfectionist Insignia".to_string(),
        Unlock::Ach(a) if a >= ach::VALKYRIE => {
            format!("Challenge: {}", crate::progress::ACHS[a].desc)
        }
        Unlock::Ach(_) => "???".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Weapons
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Blaster,
    Laser,
    Missile,
    Wave,
    Shrapnel,
    Disc,
    Thunder,
    Flame,
    Pulsar,
    Ricochet,
    Grenade,
    Cryo,
    Orbit,
    BlackHole,
    Hive,
    Cow,
    Needle,
    Boomerang,
    Prism,
    Mine,
    Supernova,
    CosmicDust,
    SolarLance,
    Singularity,
    Fireball,
    Plasma,
    Railgun,
    Meteor,
}

/// The four Relics: absurd weapons that only appear through secret paths.
pub const RELICS: [Weapon; 4] = [
    Weapon::CosmicDust,
    Weapon::Supernova,
    Weapon::SolarLance,
    Weapon::Singularity,
];

pub const WEAPONS: [Weapon; 28] = [
    Weapon::Blaster,
    Weapon::Laser,
    Weapon::Missile,
    Weapon::Wave,
    Weapon::Shrapnel,
    Weapon::Disc,
    Weapon::Thunder,
    Weapon::Flame,
    Weapon::Pulsar,
    Weapon::Ricochet,
    Weapon::Grenade,
    Weapon::Cryo,
    Weapon::Orbit,
    Weapon::BlackHole,
    Weapon::Hive,
    Weapon::Cow,
    Weapon::Needle,
    Weapon::Boomerang,
    Weapon::Prism,
    Weapon::Mine,
    Weapon::Supernova,
    Weapon::CosmicDust,
    Weapon::SolarLance,
    Weapon::Singularity,
    Weapon::Fireball,
    Weapon::Plasma,
    Weapon::Railgun,
    Weapon::Meteor,
];

impl Weapon {
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Blaster => "BLASTER",
            Weapon::Laser => "LASER",
            Weapon::Missile => "MISSILE",
            Weapon::Wave => "WAVE",
            Weapon::Shrapnel => "SHRAPNEL",
            Weapon::Disc => "DISC",
            Weapon::Thunder => "THUNDER",
            Weapon::Flame => "FLAME",
            Weapon::Pulsar => "PULSAR",
            Weapon::Ricochet => "RICOCHET",
            Weapon::Grenade => "GRENADE",
            Weapon::Cryo => "CRYO",
            Weapon::Orbit => "ORBIT",
            Weapon::BlackHole => "BLACK HOLE",
            Weapon::Hive => "HIVE",
            Weapon::Cow => "COW CANNON",
            Weapon::Needle => "NEEDLE",
            Weapon::Boomerang => "BOOMERANG",
            Weapon::Prism => "PRISM",
            Weapon::Mine => "MINE",
            Weapon::Supernova => "SUPERNOVA",
            Weapon::CosmicDust => "COSMIC DUST",
            Weapon::SolarLance => "SOLAR LANCE",
            Weapon::Singularity => "SINGULARITY",
            Weapon::Fireball => "FIREBALL",
            Weapon::Plasma => "PLASMA",
            Weapon::Railgun => "RAILGUN",
            Weapon::Meteor => "METEOR",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Weapon::Blaster => "Wide fan of fast shots.",
            Weapon::Laser => "Beams that pierce through enemies.",
            Weapon::Missile => "Strong homing missiles.",
            Weapon::Wave => "Helix shots that undulate across the screen.",
            Weapon::Shrapnel => "Shotgun: huge damage up close.",
            Weapon::Disc => "Blades that fly out and come back.",
            Weapon::Thunder => "Bolts that jump between enemies.",
            Weapon::Flame => "Short-range flamethrower.",
            Weapon::Pulsar => "Twinkling stars in a fan. Secret.",
            Weapon::Ricochet => "Shots that bounce off the walls.",
            Weapon::Grenade => "Explodes and hits everything around it.",
            Weapon::Cryo => "Freezes: frozen enemies take double damage.",
            Weapon::Orbit => "Saws spin around you and block shots.",
            Weapon::BlackHole => "Swallows enemy shots and grinds anything that gets close.",
            Weapon::Hive => "Homing bees. The Swarm against the Swarm.",
            Weapon::Cow => "Fires exploding cows. Nobody can explain it.",
            Weapon::Needle => "Ultra-thin, fast needles that pierce everything.",
            Weapon::Boomerang => {
                "Throw it, it cuts through the whole formation and comes back to your hand."
            }
            Weapon::Prism => "A beam that splits into three as it travels.",
            Weapon::Mine => "Drops mines that rise slowly and explode in an area.",
            Weapon::Supernova => "RELIC. Piercing stars and ring explosions.",
            Weapon::CosmicDust => "RELIC. A cloud of cosmic dust that hunts everything.",
            Weapon::SolarLance => "RELIC. A beam of sunlight that crosses the whole screen.",
            Weapon::Singularity => "RELIC. Black holes in a row and shots that bend space.",
            Weapon::Fireball => "A blazing fireball that explodes and scatters embers.",
            Weapon::Plasma => "Plasma orbs that grow and burn through six enemies.",
            Weapon::Railgun => "Instant beam: hits the whole column and ignores shields.",
            Weapon::Meteor => "A heavy rock that pierces and shatters into shards.",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Weapon::Blaster => Color::from_rgba(255, 200, 60, 255),
            Weapon::Laser => Color::from_rgba(90, 220, 255, 255),
            Weapon::Missile => Color::from_rgba(255, 100, 170, 255),
            Weapon::Wave => Color::from_rgba(120, 255, 170, 255),
            Weapon::Shrapnel => Color::from_rgba(255, 140, 60, 255),
            Weapon::Disc => Color::from_rgba(200, 200, 230, 255),
            Weapon::Thunder => Color::from_rgba(170, 150, 255, 255),
            Weapon::Flame => Color::from_rgba(255, 90, 40, 255),
            Weapon::Pulsar => Color::from_rgba(255, 220, 120, 255),
            Weapon::Ricochet => Color::from_rgba(120, 255, 230, 255),
            Weapon::Grenade => Color::from_rgba(170, 200, 90, 255),
            Weapon::Cryo => Color::from_rgba(170, 230, 255, 255),
            Weapon::Orbit => Color::from_rgba(230, 230, 240, 255),
            Weapon::BlackHole => Color::from_rgba(160, 90, 255, 255),
            Weapon::Hive => Color::from_rgba(255, 220, 60, 255),
            Weapon::Cow => Color::from_rgba(255, 255, 255, 255),
            Weapon::Needle => Color::from_rgba(220, 255, 140, 255),
            Weapon::Boomerang => Color::from_rgba(255, 170, 90, 255),
            Weapon::Prism => Color::from_rgba(150, 255, 255, 255),
            Weapon::Mine => Color::from_rgba(255, 110, 110, 255),
            Weapon::Supernova => Color::from_rgba(255, 240, 150, 255),
            Weapon::CosmicDust => Color::from_rgba(255, 190, 255, 255),
            Weapon::SolarLance => Color::from_rgba(255, 200, 70, 255),
            Weapon::Singularity => Color::from_rgba(200, 140, 255, 255),
            Weapon::Fireball => Color::from_rgba(255, 120, 30, 255),
            Weapon::Plasma => Color::from_rgba(190, 120, 255, 255),
            Weapon::Railgun => Color::from_rgba(140, 220, 255, 255),
            Weapon::Meteor => Color::from_rgba(220, 150, 100, 255),
        }
    }
    /// Power tier (1 to 5): higher tier weapons hit harder and cost more in the shop.
    pub fn tier(self) -> u32 {
        match self {
            Weapon::Blaster | Weapon::Laser | Weapon::Missile => 1,
            Weapon::Wave | Weapon::Shrapnel | Weapon::Disc | Weapon::Needle | Weapon::Ricochet => 2,
            Weapon::Cryo | Weapon::Thunder | Weapon::Grenade | Weapon::Flame | Weapon::Mine => 3,
            Weapon::Hive
            | Weapon::Prism
            | Weapon::Boomerang
            | Weapon::Orbit
            | Weapon::Fireball
            | Weapon::Meteor => 4,
            _ => 5,
        }
    }
    /// Tier damage multiplier.
    pub fn tier_mul(self) -> f32 {
        [1.0, 1.12, 1.25, 1.4, 1.55][(self.tier() - 1) as usize]
    }
    pub fn is_relic(self) -> bool {
        RELICS.contains(&self)
    }
    pub fn index(self) -> usize {
        WEAPONS.iter().position(|w| *w == self).unwrap_or(0)
    }
    /// Rank required (None = secret achievement).
    pub fn unlock_rank(self) -> Option<usize> {
        match self {
            Weapon::Blaster | Weapon::Laser | Weapon::Missile => Some(0),
            Weapon::Wave => Some(2),
            Weapon::Shrapnel => Some(4),
            Weapon::Disc => Some(5),
            Weapon::Thunder => Some(8),
            Weapon::Flame => Some(10),
            Weapon::Ricochet => Some(6),
            Weapon::Cryo => Some(7),
            Weapon::Grenade => Some(9),
            Weapon::Hive => Some(11),
            Weapon::Orbit => Some(13),
            Weapon::BlackHole => Some(16),
            Weapon::Needle => Some(3),
            Weapon::Mine => Some(8),
            Weapon::Prism => Some(12),
            Weapon::Boomerang => Some(14),
            Weapon::Fireball => Some(5),
            Weapon::Meteor => Some(7),
            Weapon::Plasma => Some(10),
            Weapon::Railgun => Some(12),
            _ => None,
        }
    }
    pub fn unlocked(self, rank: usize, achs: &[bool]) -> bool {
        if self.is_relic() {
            return false;
        }
        match self.unlock_rank() {
            Some(r) => rank >= r,
            None if self == Weapon::Cow => achs.get(ach::MOO).copied().unwrap_or(false),
            None => achs.get(ach::GOLDEN_RECORD).copied().unwrap_or(false),
        }
    }
}
