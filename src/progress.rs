//! Permanent progression: ranks, achievements, bestiary, modes and difficulties.

use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// Modes and difficulties
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Campaign,
    Classic,
    Daily,
    Survival,
    BossRush,
    Expedition,
}

pub const MODES: [Mode; 6] = [
    Mode::Campaign,
    Mode::Classic,
    Mode::Daily,
    Mode::Survival,
    Mode::BossRush,
    Mode::Expedition,
];

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Campaign => "CAMPAIGN",
            Mode::Classic => "CLASSIC",
            Mode::Daily => "DAILY CHALLENGE",
            Mode::Survival => "SURVIVAL",
            Mode::BossRush => "BOSS RUSH",
            Mode::Expedition => "EXPEDITION",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Mode::Campaign => "100 levels, bosses, shop and cards. The full adventure.",
            Mode::Classic => {
                "Like the arcades: only the Blaster, no shop, no cards. Endless waves."
            }
            Mode::Daily => "3 sectors with a crazy rule that changes daily. Same for everyone.",
            Mode::Survival => "Non-stop waves, faster and faster. How long can you last?",
            Mode::BossRush => "Every boss in a row, against the clock.",
            Mode::Expedition => {
                "Endless sectors. Every sector, pick 1 of 3 blessings. How far can you go?"
            }
        }
    }
    pub fn unlock_rank(self) -> usize {
        match self {
            Mode::Campaign => 0,
            Mode::Classic => 1,
            Mode::Daily => 3,
            Mode::Survival => 4,
            Mode::BossRush => 7,
            Mode::Expedition => 6,
        }
    }
    pub fn tag(self) -> u8 {
        self as u8
    }
    pub fn from_tag(t: u8) -> Mode {
        MODES.get(t as usize).copied().unwrap_or(Mode::Campaign)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
    Nightmare,
}

pub const DIFFS: [Difficulty; 4] = [
    Difficulty::Easy,
    Difficulty::Normal,
    Difficulty::Hard,
    Difficulty::Nightmare,
];

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
            Difficulty::Nightmare => "NIGHTMARE",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Difficulty::Easy => Color::from_rgba(120, 230, 120, 255),
            Difficulty::Normal => Color::from_rgba(120, 200, 255, 255),
            Difficulty::Hard => Color::from_rgba(255, 170, 60, 255),
            Difficulty::Nightmare => Color::from_rgba(255, 60, 90, 255),
        }
    }
    pub fn bullet_speed(self) -> f32 {
        [0.8, 1.0, 1.15, 1.3][self as usize]
    }
    pub fn fire_rate(self) -> f32 {
        [0.55, 1.0, 1.25, 1.5][self as usize]
    }
    /// Enemy pace (movement, dives and bosses).
    pub fn pace(self) -> f32 {
        [0.8, 1.0, 1.15, 1.3][self as usize]
    }
    pub fn lives(self) -> i32 {
        [2, 0, -1, -1][self as usize]
    }
    pub fn xp_mul(self) -> f32 {
        [0.6, 1.0, 1.5, 2.2][self as usize]
    }
    pub fn score_mul(self) -> f32 {
        [0.5, 1.0, 1.5, 2.0][self as usize]
    }
    pub fn continues(self) -> u32 {
        [3, 2, 1, 0][self as usize]
    }
    pub fn from_index(i: u8) -> Difficulty {
        DIFFS.get(i as usize).copied().unwrap_or(Difficulty::Normal)
    }
}

/// Daily challenge rules.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Daily {
    DoubleShots,
    FastEnemies,
    LaserOnly,
    DoubleCrystals,
    Fog,
    Mirror,
    ThinGlass,
    CapsuleRain,
}

pub const DAILIES: [Daily; 8] = [
    Daily::DoubleShots,
    Daily::FastEnemies,
    Daily::LaserOnly,
    Daily::DoubleCrystals,
    Daily::Fog,
    Daily::Mirror,
    Daily::ThinGlass,
    Daily::CapsuleRain,
];

impl Daily {
    pub fn name(self) -> &'static str {
        match self {
            Daily::DoubleShots => "DOUBLE SHOTS",
            Daily::FastEnemies => "FAST ENEMIES",
            Daily::LaserOnly => "LASER ONLY",
            Daily::DoubleCrystals => "DOUBLE CRYSTALS",
            Daily::Fog => "FOG",
            Daily::Mirror => "MIRROR",
            Daily::ThinGlass => "THIN GLASS",
            Daily::CapsuleRain => "CAPSULE RAIN",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Daily::DoubleShots => "You and the enemies fire twice as much.",
            Daily::FastEnemies => "Dives 40% faster, points x2.",
            Daily::LaserOnly => "Only the Laser exists, but it starts at level 3.",
            Daily::DoubleCrystals => "Double crystals, shop 50% more expensive.",
            Daily::Fog => "You can only see near your ship.",
            Daily::Mirror => "Left becomes right. Good luck.",
            Daily::ThinGlass => "Only one life, but points x3.",
            Daily::CapsuleRain => "Capsules and power-ups everywhere.",
        }
    }
}

// ---------------------------------------------------------------------------
// Dates (no external dependencies)
// ---------------------------------------------------------------------------

/// Days since 1970-01-01 (local time approximated by UTC).
pub fn today_number() -> i64 {
    (miniquad::date::now() / 86_400.0).floor() as i64
}

/// Converts a day number into (year, month, day).
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn daily_for(day: i64) -> Daily {
    DAILIES[(day
        .wrapping_mul(2_654_435_761)
        .rem_euclid(DAILIES.len() as i64)) as usize]
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Season {
    None,
    Christmas,
    Halloween,
    NewYear,
}

pub fn season() -> Season {
    let (_, m, d) = civil(today_number());
    match (m, d) {
        (12, 20..=31) => Season::Christmas,
        (1, 1..=2) => Season::NewYear,
        (10, 25..=31) | (11, 1..=2) => Season::Halloween,
        _ => Season::None,
    }
}

// ---------------------------------------------------------------------------
// Ranks
// ---------------------------------------------------------------------------

pub const RANKS: [&str; 20] = [
    "RECRUIT",
    "CADET",
    "MIDSHIPMAN",
    "PILOT",
    "ELITE PILOT",
    "SERGEANT",
    "WARRANT OFFICER",
    "LIEUTENANT",
    "CAPTAIN",
    "MAJOR",
    "LIEUTENANT COLONEL",
    "COLONEL",
    "COMMODORE",
    "REAR ADMIRAL",
    "VICE ADMIRAL",
    "ADMIRAL",
    "FLEET ADMIRAL",
    "STAR MARSHAL",
    "GALAXY HERO",
    "STAR LEGEND",
];

/// Total XP needed to reach rank `r`.
pub fn xp_for(r: usize) -> u64 {
    if r == 0 {
        0
    } else {
        (700.0 * (r as f64).powf(2.05)).round() as u64
    }
}

pub fn rank_of(xp: u64) -> usize {
    (0..RANKS.len())
        .rev()
        .find(|&r| xp >= xp_for(r))
        .unwrap_or(0)
}

/// What each rank unlocks (shown on the promotion screen).
pub fn unlock_text(r: usize) -> &'static str {
    match r {
        1 => "CLASSIC mode",
        2 => "WAVE weapon",
        3 => "FALCON ship and DAILY CHALLENGE",
        4 => "SHRAPNEL weapon and SURVIVAL mode",
        5 => "DISC and FIREBALL weapons, HARD difficulty",
        6 => "BASTION ship and RICOCHET weapon",
        7 => "BOSS RUSH mode, CRYO and METEOR weapons",
        8 => "THUNDER weapon",
        9 => "VEGA ship and GRENADE weapon",
        10 => "FLAME and PLASMA weapons",
        11 => "Rare shop items and the HIVE weapon",
        12 => "PHANTOM ship and RAILGUN weapon",
        13 => "RUBY paint and ORBIT weapon",
        14 => "EMERALD paint",
        15 => "COLOSSUS ship",
        16 => "GOLD paint and BLACK HOLE weapon",
        17 => "SHADOW paint",
        18 => "ICE paint",
        19 => "LEGENDARY paint and a golden name in the high scores",
        _ => "",
    }
}

/// Rank needed for each paint (skin).
pub fn skin_rank(skin: usize) -> usize {
    [0, 13, 14, 16, 17, 18, 19][skin.min(6)]
}

pub const SKIN_NAMES: [&str; 7] = [
    "ORIGINAL",
    "RUBY",
    "EMERALD",
    "GOLD",
    "SHADOW",
    "ICE",
    "LEGENDARY",
];

/// Draws the rank insignia centered at (x, y).
pub fn draw_insignia(r: usize, x: f32, y: f32, scale: f32) {
    let gold = Color::from_rgba(255, 205, 80, 255);
    let silver = Color::from_rgba(200, 210, 230, 255);
    let dark = Color::from_rgba(20, 16, 40, 255);
    let s = scale;
    draw_circle(x, y, 15.0 * s, dark);
    draw_circle_lines(x, y, 15.0 * s, 2.0, if r >= 10 { gold } else { silver });
    match r {
        0..=4 => {
            // chevrons
            let n = r + 1;
            for i in 0..n {
                let yy = y + 5.0 * s - i as f32 * 4.0 * s;
                draw_line(x - 8.0 * s, yy, x, yy - 4.0 * s, 2.0 * s, silver);
                draw_line(x, yy - 4.0 * s, x + 8.0 * s, yy, 2.0 * s, silver);
            }
        }
        5..=9 => {
            // bars
            let n = r - 4;
            for i in 0..n {
                let yy = y - 8.0 * s + i as f32 * 4.0 * s;
                draw_rectangle(x - 7.0 * s, yy, 14.0 * s, 2.5 * s, silver);
            }
        }
        _ => {
            // stars
            let n = (r - 9).min(10);
            let per_row = 5.min(n);
            for i in 0..n {
                let row = i / 5;
                let col = i % 5;
                let cols = if row == 0 { per_row } else { n - 5 };
                let xx = x + (col as f32 - (cols as f32 - 1.0) / 2.0) * 5.5 * s;
                let yy = y - 3.0 * s + row as f32 * 7.0 * s;
                draw_poly(xx, yy, 5, 2.6 * s, -90.0, gold);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Achievements
// ---------------------------------------------------------------------------

pub struct AchDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub hidden: bool,
    pub xp: u64,
}

const fn a(name: &'static str, desc: &'static str, xp: u64) -> AchDef {
    AchDef {
        name,
        desc,
        hidden: false,
        xp,
    }
}
const fn h(name: &'static str, desc: &'static str, xp: u64) -> AchDef {
    AchDef {
        name,
        desc,
        hidden: true,
        xp,
    }
}

pub const ACHS: &[AchDef] = &[
    a("First Flight", "Clear the first wave.", 50),
    a("Regicide", "Defeat the Hive Queen.", 150),
    a("Scrapyard", "Defeat the Zenith Dreadnought.", 200),
    a("Snake Charmer", "Defeat the Comet Serpent.", 250),
    a("Arachnophobia", "Defeat the Abyssal Weaver.", 300),
    a("Don't Blink", "Defeat the Eye of the Void.", 350),
    a("Broken Heart", "Finish the campaign.", 1000),
    h(
        "True Ending",
        "Finish the campaign on Hard without using continues.",
        2500,
    ),
    h(
        "Nightmare Awakened",
        "Finish the campaign on Nightmare.",
        5000,
    ),
    a("Perfection", "Hit every enemy in a bonus stage.", 200),
    h("Pacifist", "Finish a bonus stage without firing.", 300),
    a("Combo 50", "Make a 50 combo.", 150),
    a("Combo 100", "Make a 100 combo.", 400),
    a("Close Shave", "Make 100 grazes in one run.", 300),
    a("Untouchable", "Finish a sector without getting hit.", 400),
    a(
        "Desmonte",
        "Destroy every part of a boss before its body.",
        300,
    ),
    a("Millionaire", "Score 1,000,000 points in one run.", 1500),
    a("Scrooge", "Hold 300 crystals at once.", 250),
    a("Collector", "Find 20 different cards.", 500),
    a(
        "Completionist",
        "Find every common, rare, epic and legendary card.",
        2000,
    ),
    h("Fool's Gold", "Shoot down a Golden Scarab.", 200),
    h("Moo!", "Shoot down the Cosmic Cow.", 300),
    h(
        "Good Samaritan",
        "Let the Merchant pass in peace three times.",
        300,
    ),
    h("Space Pirate", "Attack the Merchant.", 150),
    h(
        "Nostalgic",
        "Enter the Konami code on the title screen.",
        500,
    ),
    h("Dimension Traveler", "Find the Golden Dimension.", 800),
    a("Weapon Master", "Take 5 different weapons to level 5.", 600),
    a("Full Hangar", "Unlock the 7 classic ships.", 1000),
    a("Survivor", "Reach wave 30 in Survival mode.", 800),
    a(
        "Speedrunner",
        "Beat Boss Rush mode in under 12 minutes.",
        1000,
    ),
    a("Arcade", "Score 60,000 points in Classic mode.", 600),
    a("Routine", "Complete 3 daily challenges.", 500),
    a("Anti-Kamikaze", "Shoot down 150 Kamikazes in total.", 300),
    a("Bomber", "Use 50 bombs in total.", 250),
    h(
        "Golden Record",
        "Two probes carry a golden record. Enter their name in the high scores.",
        700,
    ),
    h("Creator", "Who made this game?", 300),
    a("Persistent", "Play 25 runs.", 400),
    a("Marathon", "Play for 2 hours in total.", 800),
    a("Pyromaniac", "Shoot down 300 enemies with the Flame.", 400),
    a(
        "Without a Scratch",
        "Defeat a boss without getting hit.",
        500,
    ),
    h(
        "Curious",
        "Something happens if you type the game's name on the title screen.",
        150,
    ),
    h("Sleepless", "Let the game play itself in demo mode.", 100),
    a("Fever!", "Enter Fever with a high combo.", 200),
    a("Triple Fever", "Enter Fever 3 times in one run.", 500),
    a("Hired", "Complete a contract.", 150),
    a("Elite Mercenary", "Complete 30 contracts in total.", 900),
    a("Archaeologist", "Find 6 flight logs.", 500),
    a("Historian", "Find all 12 flight logs.", 1500),
    a("Twins Apart", "Defeat the Twins.", 400),
    h("The Void Stares Back", "Defeat the Devourer.", 5000),
    a(
        "Thief Catcher",
        "Shoot down a Thief carrying its loot.",
        250,
    ),
    a("Suspicious", "Shoot down 10 Mimics in total.", 300),
    a(
        "Elite of the Elite",
        "Shoot down 50 elite enemies in total.",
        600,
    ),
    a("Ice Age", "Freeze 150 enemies in total.", 400),
    a(
        "Horizon",
        "Swallow 300 shots with black holes in total.",
        400,
    ),
    a(
        "Double Arsenal",
        "Take two different weapons to level 5 in the same run.",
        400,
    ),
    a(
        "Supreme Master",
        "Take 12 different weapons to level 5.",
        1500,
    ),
    a("Total Eclipse", "Finish a sector during an Eclipse.", 400),
    h("Farmer", "Shoot down the Cosmic Cow 3 times.", 600),
    a("Arcade Master", "Win 10 Rift minigames.", 600),
    h("Reliquary", "Find a Relic.", 1000),
    a("Cosmic Tourist", "Live through all 6 sector events.", 700),
    a("Exorcist", "Destroy 25 curses with your shots.", 500),
    h("Jinxed", "Pick up 10 curses. On purpose?", 200),
    h("Star Hunter", "Find a Secret Room.", 800),
    a("Perfectionist", "Get PERFECT in 5 minigames.", 700),
    h("Keeper of Relics", "Find the 4 Relics.", 4000),
    a("Expeditionary", "Reach sector 10 in Expedition.", 1200),
    h("Treasurer", "Open 5 secret reliquaries.", 900),
    a("Full Fleet", "Unlock all 10 ships.", 1500),
    a("Comet Hunter", "Defeat the Wild Comet.", 400),
    a("Speller", "Collect the letters S-T-A-R-D-U-S-T.", 800),
    h(
        "Close Encounters",
        "Shoot down the mysterious Mothership.",
        300,
    ),
    h("Shortcut", "Use a Warp Gate.", 600),
    a("Lightning Wave", "Clear a wave in under 7 seconds.", 400),
    a(
        "Cannon Shot",
        "Shoot down 5 enemies with a single charged shot.",
        500,
    ),
    a("Valkyrie", "Reach level 50 of the campaign.", 1500),
    a(
        "Dark Duel",
        "Defeat a boss in the dark without being hit.",
        1500,
    ),
    a(
        "Twin Stars",
        "Destroy a boss with the co-op DUO BEAM.",
        1200,
    ),
];

/// Flight logs: the story behind the Swarm.
pub const LOGS: [(&str, &str); 12] = [
    (
        "LOG 1 - THE QUEEN",
        "The human probe's dust called to us. I ordered the migration. No time to explain to the natives.",
    ),
    (
        "LOG 2 - THE DREADNOUGHT",
        "We took the humans' ship. We needed hull and cannons. Something has been following us for three cycles.",
    ),
    (
        "LOG 3 - THE SERPENT",
        "The Serpent was the guardian of the path. It heard first: a silence that swallows stars.",
    ),
    (
        "LOG 4 - THE WEAVER",
        "We wove nets between the moons to slow the thing down. The nets vanished. They didn't tear: they vanished.",
    ),
    (
        "LOG 5 - THE EYE",
        "The Eye was made to watch the dark. It saw. Since then it can't quite close.",
    ),
    (
        "LOG 6 - THE OMEGA",
        "We reinforced the Dreadnought. If the human pilot makes it this far, maybe they're strong enough.",
    ),
    (
        "LOG 7 - THE TWINS",
        "My daughters fight together. I told them not to hate the pilot. The pilot just doesn't know.",
    ),
    (
        "LOG 8 - THE HEART",
        "If I fall, the hive scatters and survives. If they fall, no one will hold the door anymore.",
    ),
    (
        "LOG 9 - GOLDEN DIMENSION",
        "We hid the hive's treasure between dimensions. Only those who never fall can find the door.",
    ),
    (
        "LOG 10 - THE COW",
        "The Cow is not ours. It showed up one day and started following us. Since then we call it luck.",
    ),
    (
        "LOG 11 - THE MERCHANT",
        "The Merchant sells to every side. He knows about the Devourer. He always knew.",
    ),
    (
        "LOG 12 - AFTERWARD",
        "Thank you, pilot. The Devourer retreated. The hive will survive. And now you know too.",
    ),
];

// ---------------------------------------------------------------------------
// Honor Insignias: ranks that don't come from XP, but from feats.
// ---------------------------------------------------------------------------

pub struct Honor {
    pub name: &'static str,
    pub feat: &'static str,
    pub reward: &'static str,
    pub short: &'static str,
    pub ach: usize,
}

pub const HONORS: [Honor; 10] = [
    Honor {
        name: "UNTOUCHABLE INSIGNIA",
        feat: "Finish a sector without getting hit.",
        reward: "RAINBOW trail",
        short: "RAINBOW",
        ach: ach::UNTOUCHABLE,
    },
    Honor {
        name: "EXORCIST INSIGNIA",
        feat: "Destroy 25 curses with your shots.",
        reward: "FIRE trail",
        short: "FIRE",
        ach: ach::EXORCIST,
    },
    Honor {
        name: "ARCADE INSIGNIA",
        feat: "Win 10 Rift minigames.",
        reward: "CORSAIR ship",
        short: "CORSAIR",
        ach: ach::ARCADE_MASTER,
    },
    Honor {
        name: "EXPLORER INSIGNIA",
        feat: "Find a Secret Room.",
        reward: "PROBE ship and STARDUST trail",
        short: "PROBE",
        ach: ach::SECRET_ROOM,
    },
    Honor {
        name: "PERFECTIONIST INSIGNIA",
        feat: "Get PERFECT in 5 minigames.",
        reward: "AEGIS ship",
        short: "AEGIS",
        ach: ach::PERFECTIONIST,
    },
    Honor {
        name: "ARSENAL INSIGNIA",
        feat: "Take 5 weapons to level 5.",
        reward: "ICE trail",
        short: "ICE",
        ach: ach::WEAPON_MASTER,
    },
    Honor {
        name: "RELIC INSIGNIA",
        feat: "Find a Relic.",
        reward: "GOLD trail",
        short: "GOLD",
        ach: ach::RELIQUARY,
    },
    Honor {
        name: "EXPEDITION INSIGNIA",
        feat: "Reach sector 10 in Expedition.",
        reward: "PLASMA trail",
        short: "PLASMA",
        ach: ach::EXPEDITIONARY,
    },
    Honor {
        name: "VOID INSIGNIA",
        feat: "Defeat the Devourer.",
        reward: "VOID trail",
        short: "VOID",
        ach: ach::DEVOURER,
    },
    Honor {
        name: "SUPREME INSIGNIA",
        feat: "Find the 4 Relics.",
        reward: "LEGENDARY paint for every ship",
        short: "LEGENDARY",
        ach: ach::ALL_RELICS,
    },
];

impl Honor {
    /// Insignias tied to hidden achievements reveal nothing until earned.
    pub fn masked(&self, achs: &[bool]) -> bool {
        ACHS[self.ach].hidden && !achs.get(self.ach).copied().unwrap_or(false)
    }
    pub fn shown_name(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) {
            "INSIGNIA ???"
        } else {
            self.name
        }
    }
    pub fn shown_short(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) { "???" } else { self.short }
    }
    pub fn shown_reward(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) {
            "???"
        } else {
            self.reward
        }
    }
    pub fn shown_feat(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) { "???" } else { self.feat }
    }
}

pub fn honors_earned(achs: &[bool]) -> usize {
    HONORS
        .iter()
        .filter(|h| achs.get(h.ach).copied().unwrap_or(false))
        .count()
}

pub const TRAILS: [&str; 8] = [
    "NO TRAIL",
    "RAINBOW",
    "FIRE",
    "COSMIC DUST",
    "VOID",
    "ICE",
    "GOLD",
    "PLASMA",
];

/// Achievement that unlocks each trail (0 is always free).
pub fn trail_ach(i: usize) -> Option<usize> {
    match i {
        1 => Some(ach::UNTOUCHABLE),
        2 => Some(ach::EXORCIST),
        3 => Some(ach::SECRET_ROOM),
        4 => Some(ach::DEVOURER),
        5 => Some(ach::WEAPON_MASTER),
        6 => Some(ach::RELIQUARY),
        7 => Some(ach::EXPEDITIONARY),
        _ => None,
    }
}

pub fn trail_unlocked(i: usize, s: &crate::save::Save) -> bool {
    i < TRAILS.len()
        && trail_ach(i)
            .map(|a| s.achs.get(a).copied().unwrap_or(false))
            .unwrap_or(true)
}

pub mod ach {
    pub const FIRST_FLIGHT: usize = 0;
    pub const REGICIDE: usize = 1;
    pub const SCRAPYARD: usize = 2;
    pub const SNAKE_CHARMER: usize = 3;
    pub const ARACHNO: usize = 4;
    pub const DONT_BLINK: usize = 5;
    pub const BROKEN_HEART: usize = 6;
    pub const TRUE_ENDING: usize = 7;
    pub const NIGHTMARE: usize = 8;
    pub const PERFECTION: usize = 9;
    pub const PACIFIST: usize = 10;
    pub const COMBO50: usize = 11;
    pub const COMBO100: usize = 12;
    pub const CLOSE_SHAVE: usize = 13;
    pub const UNTOUCHABLE: usize = 14;
    pub const DISMANTLE: usize = 15;
    pub const MILLIONAIRE: usize = 16;
    pub const SCROOGE: usize = 17;
    pub const COLLECTOR: usize = 18;
    pub const COMPLETIONIST: usize = 19;
    pub const FOOLS_GOLD: usize = 20;
    pub const MOO: usize = 21;
    pub const SAMARITAN: usize = 22;
    pub const PIRATE: usize = 23;
    pub const KONAMI: usize = 24;
    pub const DIMENSIONAL: usize = 25;
    pub const WEAPON_MASTER: usize = 26;
    pub const HANGAR: usize = 27;
    pub const SURVIVOR: usize = 28;
    pub const SPEEDRUNNER: usize = 29;
    pub const ARCADE_SCORE: usize = 30;
    pub const ROUTINE: usize = 31;
    pub const ANTI_KAMIKAZE: usize = 32;
    pub const BOMBER: usize = 33;
    pub const GOLDEN_RECORD: usize = 34;
    pub const CREATOR: usize = 35;
    pub const PERSISTENT: usize = 36;
    pub const MARATHON: usize = 37;
    pub const PYROMANIAC: usize = 38;
    pub const NO_SCRATCH: usize = 39;
    pub const CURIOUS: usize = 40;
    pub const SLEEPLESS: usize = 41;
    pub const FEVER: usize = 42;
    pub const FEVER3: usize = 43;
    pub const HIRED: usize = 44;
    pub const ELITE_MERCENARY: usize = 45;
    pub const ARCHAEOLOGIST: usize = 46;
    pub const HISTORIAN: usize = 47;
    pub const TWINS: usize = 48;
    pub const DEVOURER: usize = 49;
    pub const THIEF_CATCHER: usize = 50;
    pub const SUSPICIOUS: usize = 51;
    pub const ELITE: usize = 52;
    pub const ICE_AGE: usize = 53;
    pub const HORIZON: usize = 54;
    pub const DOUBLE_ARSENAL: usize = 55;
    pub const SUPREME_MASTER: usize = 56;
    pub const ECLIPSE: usize = 57;
    pub const FARMER: usize = 58;
    pub const ARCADE_MASTER: usize = 59;
    pub const RELIQUARY: usize = 60;
    pub const TOURIST: usize = 61;
    pub const EXORCIST: usize = 62;
    pub const JINXED: usize = 63;
    pub const SECRET_ROOM: usize = 64;
    pub const PERFECTIONIST: usize = 65;
    pub const ALL_RELICS: usize = 66;
    pub const EXPEDITIONARY: usize = 67;
    pub const TREASURER: usize = 68;
    pub const FULL_FLEET: usize = 69;
    pub const COMET: usize = 70;
    pub const SPELLER: usize = 71;
    pub const UFO: usize = 72;
    pub const SHORTCUT: usize = 73;
    pub const LIGHTNING_WAVE: usize = 74;
    pub const CHARGED: usize = 75;
    pub const VALKYRIE: usize = 76;
    pub const DARK_DUEL: usize = 77;
    pub const TWIN_STARS: usize = 78;
}

// ---------------------------------------------------------------------------
// Bestiary
// ---------------------------------------------------------------------------

pub struct Lore {
    pub name: &'static str,
    pub text: &'static str,
}

/// One entry per enemy type (order of game::Kind) followed by the bosses (order of BossKind).
pub const BESTIARY: &[Lore] = &[
    Lore {
        name: "DRONE",
        text: "The base of the Swarm. Harmless alone; in formation, a living wall.",
    },
    Lore {
        name: "WASP",
        text: "Fast and jumpy. Loves to dive in spirals before returning to formation.",
    },
    Lore {
        name: "BEETLE",
        text: "Armored commander. Takes two hits and always brings escorts when diving.",
    },
    Lore {
        name: "GUNNER",
        text: "Carries two biological cannons. Stops mid-screen to fire a spread.",
    },
    Lore {
        name: "SPLITTER",
        text: "When it dies, it splits into two hungry hatchlings that hunt your ship.",
    },
    Lore {
        name: "KAMIKAZE",
        text: "Doesn't shoot. Doesn't dodge. Just wants to touch you.",
    },
    Lore {
        name: "HATCHLING",
        text: "Born from Splitters and Carriers. Small, fast and stubborn.",
    },
    Lore {
        name: "SHIELDBEARER",
        text: "Carries a chitin plate in front. Break the shield before you can hurt it.",
    },
    Lore {
        name: "WEAVER",
        text: "Shoots sticky webs that slow your ship down for a few seconds.",
    },
    Lore {
        name: "CARRIER",
        text: "A flying nursery. Releases hatchlings while alive.",
    },
    Lore {
        name: "CHAMELEON",
        text: "Vanishes and reappears. Can only be hit while visible.",
    },
    Lore {
        name: "GOLDEN SCARAB",
        text: "Very rare. Flees fast, but carries a fortune in crystals.",
    },
    Lore {
        name: "COSMIC COW",
        text: "Nobody knows how it got to space. Nobody dares to ask.",
    },
    Lore {
        name: "MERCHANT",
        text: "A neutral trader. Returns kindness... and never forgets a betrayal.",
    },
    Lore {
        name: "HIVE QUEEN",
        text: "Mother of the whole Swarm. Her claws fire on their own and she calls minions.",
    },
    Lore {
        name: "ZENITH DREADNOUGHT",
        text: "A stolen, infested ship. The central cannon fires a telegraphed laser.",
    },
    Lore {
        name: "COMET SERPENT",
        text: "A tail of burning rings. Each ring destroyed weakens the beast.",
    },
    Lore {
        name: "ABYSSAL WEAVER",
        text: "A spider the size of a moon. Its webs cross the entire screen.",
    },
    Lore {
        name: "EYE OF THE VOID",
        text: "Can only be hurt with its eyelid open. Its gaze sweeps space with beams.",
    },
    Lore {
        name: "SUPREME QUEEN",
        text: "The heart of the Swarm. Crowned, furious and with all her sisters' tricks.",
    },
    Lore {
        name: "MIMIC",
        text: "Looks like a capsule. It isn't. When you get close, it opens its mouth.",
    },
    Lore {
        name: "THIEF",
        text: "Masked and fast. Steals what falls and flees. Shoot it down to get it back doubled.",
    },
    Lore {
        name: "FREIGHTER",
        text: "Swarm cargo ship. Doesn't shoot and is packed with crystals.",
    },
    Lore {
        name: "THE TWINS",
        text: "Two hive princesses who fight together. When one falls, the other goes berserk.",
    },
    Lore {
        name: "THE DEVOURER",
        text: "What the Swarm was fleeing from. A hunger the size of a galaxy.",
    },
    Lore {
        name: "WILD COMET",
        text: "A comet the Swarm tamed. Its tail sheds cosmic dust that hurts.",
    },
    Lore {
        name: "MINELAYER",
        text: "Dives while dropping mines that explode in a spread. Shoot the mines!",
    },
    Lore {
        name: "JUMPER",
        text: "Never stays in the same spot of the formation. Vanishes in a blink and reappears far away.",
    },
    Lore {
        name: "ARMORED",
        text: "Plated from head to claw. Frontal shots only chip the armor: hit it from behind or blow it up.",
    },
    Lore {
        name: "HEALER",
        text: "Every few seconds it pulses and mends the wounds and armor of everyone around it. Kill it first.",
    },
    Lore {
        name: "MIRROR",
        text: "Its living mirror opens and closes. While open, it throws your own shots back at you.",
    },
    Lore {
        name: "LEADER",
        text: "The crowned commander. When it falls, the formation scatters into new places in a rage.",
    },
];

pub const LORE_BOSS_START: usize = 14;
/// Special enemies and late bosses (after the original bosses).
pub const LORE_SPECIAL: usize = 20;
pub const LORE_TWINS: usize = 23;
pub const LORE_DEVOURER: usize = 24;
pub const LORE_COMET: usize = 25;
/// Late enemies (Minelayer and Jumper), after the Comet.
pub const LORE_LATE: usize = 26;
pub const KIND_LATE: usize = 17;

/// Bestiary index for an enemy type (game::Kind as usize).
pub fn lore_of_kind(kind: usize) -> usize {
    if kind < LORE_BOSS_START {
        kind
    } else if kind >= KIND_LATE {
        LORE_LATE + kind - KIND_LATE
    } else {
        LORE_SPECIAL + kind - LORE_BOSS_START
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_grow() {
        for r in 1..RANKS.len() {
            assert!(xp_for(r) > xp_for(r - 1));
            assert_eq!(rank_of(xp_for(r)), r);
            assert_eq!(rank_of(xp_for(r) - 1), r - 1);
        }
    }

    #[test]
    fn dates() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(20_454), (2026, 1, 1));
        assert_eq!(civil(20_760), (2026, 11, 3));
    }

    #[test]
    fn achievements_have_unique_names() {
        let mut names: Vec<&str> = ACHS.iter().map(|a| a.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ACHS.len());
        assert_eq!(ACHS[ach::SLEEPLESS].name, "Sleepless");
        assert_eq!(ACHS[ach::TOURIST].name, "Cosmic Tourist");
        assert_eq!(ACHS.len(), 79);
        assert_eq!(ACHS[ach::COMET].name, "Comet Hunter");
    }
}
