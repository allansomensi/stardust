//! Plain-text save in the user's data folder.

use crate::cards::CARDS;
use crate::progress::{ACHS, BESTIARY, MODES, Mode};
use std::path::PathBuf;

pub const NAME_MAX: usize = 10;

#[derive(Clone)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
    pub sector: u32,
    pub mode: u8,
    pub ship: u8,
}

#[derive(Clone)]
pub struct Save {
    // settings
    pub music: u8,
    pub sfx: u8,
    pub fullscreen: bool,
    pub crt: bool,
    pub shake: bool,
    // choices
    pub ship: usize,
    pub skin: usize,
    pub mode: u8,
    pub diff: u8,
    pub equipped: Vec<usize>,
    pub last_name: String,
    // progress
    pub xp: u64,
    pub achs: Vec<bool>,
    pub seen: Vec<bool>,
    pub cards: Vec<u32>,
    pub mastery: Vec<bool>,
    pub scores: Vec<ScoreEntry>,
    // stats
    pub runs: u32,
    pub kills: u32,
    pub play_time: f64,
    pub best_sector: u32,
    /// Furthest campaign sector reached (unlocks the checkpoints).
    pub checkpoint: u32,
    /// Ship flown by player two in co-op.
    pub ship2: usize,
    pub victories: u32,
    pub bombs_used: u32,
    pub grazes: u32,
    pub crystals_total: u32,
    pub best_combo: u32,
    pub bosses: u32,
    pub deaths: u32,
    pub kills_kind: Vec<u32>,
    pub weapon_kills: Vec<u32>,
    pub merchants_spared: u32,
    pub best_survival: u32,
    pub best_rush: f32,
    pub daily_done: u32,
    pub daily_last: i64,
    pub daily_best: u32,
    // encounters
    pub logs: Vec<bool>,
    pub contracts_done: u32,
    pub mimics: u32,
    pub elites: u32,
    pub frozen_total: u32,
    pub absorbed_total: u32,
    pub cows: u32,
    pub events_seen: Vec<bool>,
    // extra settings
    pub particles: u8,
    pub flashes: bool,
    pub show_fps: bool,
    /// Dynamic lighting (glows around shots and blasts).
    pub lighting: bool,
    /// Gamepad vibration.
    pub rumble: bool,
    /// Remapped keys and gamepad buttons for both players.
    pub binds: crate::controls::Bindings,
    pub autofire: bool,
    pub game_speed: u8,
    pub panels: bool,
    pub bg_anim: bool,
    pub show_hitbox: bool,
    pub hints: bool,
    pub trail: usize,
    // more progress
    pub relics: Vec<bool>,
    pub curses_taken: u32,
    pub curses_shot: u32,
    pub minigames_won: u32,
    pub minigames_played: u32,
    pub perfects: u32,
    pub rooms_found: u32,
    pub expedition_best: u32,
    pub seen_intro: bool,
    pub words: u32,
    pub ufos: u32,
    /// 0 = English, 1 = Portuguese.
    pub lang: u8,
}

fn default_scores() -> Vec<ScoreEntry> {
    let names = [
        "ACE", "COMET", "NEBULA", "ORION", "VEGA", "PULSAR", "QUASAR", "LUNA", "TITAN", "ASTRO",
    ];
    let mut v = vec![];
    for m in MODES {
        let base: u32 = match m {
            Mode::Campaign => 60_000,
            Mode::Classic => 30_000,
            Mode::Daily => 25_000,
            Mode::Survival => 40_000,
            Mode::BossRush => 50_000,
            Mode::Expedition => 80_000,
        };
        for (i, n) in names.iter().enumerate() {
            v.push(ScoreEntry {
                name: n.to_string(),
                score: base - i as u32 * base / 12,
                sector: (6 - i as u32 / 2).max(1),
                mode: m.tag(),
                ship: 0,
            });
        }
    }
    v
}

impl Default for Save {
    fn default() -> Self {
        Save {
            music: 6,
            sfx: 8,
            fullscreen: false,
            crt: true,
            shake: true,
            ship: 0,
            skin: 0,
            mode: 0,
            diff: 1,
            equipped: vec![],
            last_name: "PILOT".into(),
            xp: 0,
            achs: vec![false; ACHS.len()],
            seen: vec![false; BESTIARY.len()],
            cards: vec![0; CARDS.len()],
            mastery: vec![false; crate::ships::WEAPONS.len()],
            scores: default_scores(),
            runs: 0,
            kills: 0,
            play_time: 0.0,
            best_sector: 0,
            checkpoint: 1,
            ship2: 0,
            victories: 0,
            bombs_used: 0,
            grazes: 0,
            crystals_total: 0,
            best_combo: 0,
            bosses: 0,
            deaths: 0,
            kills_kind: vec![0; 32],
            weapon_kills: vec![0; crate::ships::WEAPONS.len()],
            merchants_spared: 0,
            best_survival: 0,
            best_rush: 0.0,
            daily_done: 0,
            daily_last: -1,
            daily_best: 0,
            logs: vec![false; crate::progress::LOGS.len()],
            contracts_done: 0,
            mimics: 0,
            elites: 0,
            frozen_total: 0,
            absorbed_total: 0,
            cows: 0,
            events_seen: vec![false; 6],
            particles: 2,
            flashes: true,
            show_fps: false,
            lighting: true,
            rumble: true,
            binds: Default::default(),
            autofire: false,
            game_speed: 0,
            panels: true,
            bg_anim: true,
            show_hitbox: false,
            hints: true,
            trail: 0,
            relics: vec![false; crate::ships::RELICS.len()],
            curses_taken: 0,
            curses_shot: 0,
            minigames_won: 0,
            minigames_played: 0,
            perfects: 0,
            rooms_found: 0,
            expedition_best: 0,
            seen_intro: false,
            words: 0,
            ufos: 0,
            lang: system_lang(),
        }
    }
}

/// System language on first launch: Portuguese if the system is in Portuguese, otherwise English.
pub fn system_lang() -> u8 {
    let loc = sys_locale::get_locale().unwrap_or_default().to_lowercase();
    if loc.starts_with("pt") { 1 } else { 0 }
}

pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Ok(a) = std::env::var("APPDATA") {
        return PathBuf::from(a).join("Stardust");
    }
    #[cfg(target_os = "macos")]
    if let Ok(h) = std::env::var("HOME") {
        return PathBuf::from(h).join("Library/Application Support/Stardust");
    }
    if let Ok(x) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(x).join("stardust");
    }
    if let Ok(h) = std::env::var("HOME") {
        return PathBuf::from(h).join(".local/share/stardust");
    }
    PathBuf::from(".")
}

fn file() -> PathBuf {
    data_dir().join("save.txt")
}

fn parse_list<T: std::str::FromStr + Copy>(v: &str, out: &mut [T]) {
    for (i, n) in v.split(',').enumerate() {
        if i < out.len()
            && let Ok(x) = n.trim().parse()
        {
            out[i] = x;
        }
    }
}

fn bools(v: &str, out: &mut [bool]) {
    for (i, c) in v.chars().enumerate() {
        if i < out.len() {
            out[i] = c == '1';
        }
    }
}

fn join<T: ToString>(v: &[T]) -> String {
    v.iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn bits(v: &[bool]) -> String {
    v.iter().map(|b| if *b { '1' } else { '0' }).collect()
}

impl Save {
    pub fn load() -> Save {
        let mut s = Save::default();
        let Ok(text) = std::fs::read_to_string(file()) else {
            return s;
        };
        let mut scores = Vec::new();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            let v = v.trim();
            let num = |d: u32| v.parse::<u32>().unwrap_or(d);
            match k.trim() {
                "music" => s.music = (num(6) as u8).min(10),
                "sfx" => s.sfx = (num(8) as u8).min(10),
                "fullscreen" => s.fullscreen = v == "1",
                "crt" => s.crt = v == "1",
                "shake" => s.shake = v == "1",
                "ship" => s.ship = (num(0) as usize).min(crate::ships::SHIPS.len() - 1),
                "skin" => s.skin = (num(0) as usize).min(crate::art::SKINS - 1),
                "mode" => s.mode = (num(0) as u8).min(MODES.len() as u8 - 1),
                "diff" => s.diff = (num(1) as u8).min(3),
                "last_name" => s.last_name = clean_name(v),
                "xp" => s.xp = v.parse().unwrap_or(0),
                "achs" => bools(v, &mut s.achs),
                "seen" => bools(v, &mut s.seen),
                "mastery" => bools(v, &mut s.mastery),
                "cards" => parse_list(v, &mut s.cards),
                "kills_kind" => parse_list(v, &mut s.kills_kind),
                "weapon_kills" => parse_list(v, &mut s.weapon_kills),
                "equipped" => {
                    s.equipped = v
                        .split(',')
                        .filter_map(|n| n.trim().parse().ok())
                        .filter(|&i: &usize| i < CARDS.len())
                        .collect();
                }
                "runs" => s.runs = num(0),
                "kills" => s.kills = num(0),
                "play_time" => s.play_time = v.parse().unwrap_or(0.0),
                "best_sector" => s.best_sector = num(0),
                "ship2" => s.ship2 = (num(0) as usize).min(crate::ships::SHIPS.len() - 1),
                "checkpoint" => s.checkpoint = num(1).clamp(1, crate::game::SECTORS),
                "victories" => s.victories = num(0),
                "bombs_used" => s.bombs_used = num(0),
                "grazes" => s.grazes = num(0),
                "crystals_total" => s.crystals_total = num(0),
                "best_combo" => s.best_combo = num(0),
                "bosses" => s.bosses = num(0),
                "deaths" => s.deaths = num(0),
                "merchants_spared" => s.merchants_spared = num(0),
                "best_survival" => s.best_survival = num(0),
                "best_rush" => s.best_rush = v.parse().unwrap_or(0.0),
                "daily_done" => s.daily_done = num(0),
                "daily_last" => s.daily_last = v.parse().unwrap_or(-1),
                "daily_best" => s.daily_best = num(0),
                "logs" => bools(v, &mut s.logs),
                "events_seen" => bools(v, &mut s.events_seen),
                "contracts_done" => s.contracts_done = num(0),
                "mimics" => s.mimics = num(0),
                "elites" => s.elites = num(0),
                "frozen_total" => s.frozen_total = num(0),
                "absorbed_total" => s.absorbed_total = num(0),
                "cows" => s.cows = num(0),
                "particles" => s.particles = (num(2) as u8).min(2),
                "flashes" => s.flashes = v == "1",
                "show_fps" => s.show_fps = v == "1",
                "lighting" => s.lighting = v == "1",
                "rumble" => s.rumble = v == "1",
                "keys" => s.binds.decode_keys(v),
                "buttons" => s.binds.decode_pad(v),
                "autofire" => s.autofire = v == "1",
                "game_speed" => s.game_speed = (num(0) as u8).min(2),
                "panels" => s.panels = v == "1",
                "bg_anim" => s.bg_anim = v == "1",
                "show_hitbox" => s.show_hitbox = v == "1",
                "hints" => s.hints = v == "1",
                "trail" => s.trail = num(0) as usize,
                "relics" => bools(v, &mut s.relics),
                "expedition_best" => s.expedition_best = num(0),
                "seen_intro" => s.seen_intro = v == "1",
                "words" => s.words = num(0),
                "lang" => s.lang = if v == "pt" { 1 } else { 0 },
                "ufos" => s.ufos = num(0),
                "curses_taken" => s.curses_taken = num(0),
                "curses_shot" => s.curses_shot = num(0),
                "minigames_won" => s.minigames_won = num(0),
                "minigames_played" => s.minigames_played = num(0),
                "perfects" => s.perfects = num(0),
                "rooms_found" => s.rooms_found = num(0),
                "score" => {
                    let p: Vec<&str> = v.split(';').collect();
                    if p.len() >= 3
                        && let (Ok(sc), Ok(se)) = (p[1].parse(), p[2].parse())
                    {
                        let mode = p.get(3).and_then(|m| m.parse().ok()).unwrap_or(0);
                        let ship = p.get(4).and_then(|m| m.parse().ok()).unwrap_or(0);
                        scores.push(ScoreEntry {
                            name: clean_name(p[0]),
                            score: sc,
                            sector: se,
                            mode,
                            ship,
                        });
                    }
                }
                _ => {}
            }
        }
        if !scores.is_empty() {
            // Modes without saved scores get the default table.
            for m in MODES {
                if !scores.iter().any(|e| e.mode == m.tag()) {
                    scores.extend(default_scores().into_iter().filter(|e| e.mode == m.tag()));
                }
            }
            s.scores = scores;
            s.sort_scores();
        }
        let cards = s.cards.clone();
        s.equipped.retain(|&i| cards[i] > 0);
        s.equipped.dedup();
        let slots = crate::cards::slots_for(s.unique_cards());
        s.equipped.truncate(slots);
        s
    }

    fn sort_scores(&mut self) {
        self.scores
            .sort_by(|a, b| a.mode.cmp(&b.mode).then(b.score.cmp(&a.score)));
        let mut out: Vec<ScoreEntry> = vec![];
        for m in MODES {
            out.extend(
                self.scores
                    .iter()
                    .filter(|e| e.mode == m.tag())
                    .take(10)
                    .cloned(),
            );
        }
        self.scores = out;
    }

    pub fn write(&self) {
        let mut o = String::new();
        o.push_str("# Stardust save file\nversion=1\n");
        let mut kv = |k: &str, v: String| {
            o.push_str(k);
            o.push('=');
            o.push_str(&v);
            o.push('\n');
        };
        kv("music", self.music.to_string());
        kv("sfx", self.sfx.to_string());
        kv("fullscreen", (self.fullscreen as u8).to_string());
        kv("crt", (self.crt as u8).to_string());
        kv("shake", (self.shake as u8).to_string());
        kv("ship", self.ship.to_string());
        kv("skin", self.skin.to_string());
        kv("mode", self.mode.to_string());
        kv("diff", self.diff.to_string());
        kv("last_name", self.last_name.clone());
        kv("xp", self.xp.to_string());
        kv("achs", bits(&self.achs));
        kv("seen", bits(&self.seen));
        kv("mastery", bits(&self.mastery));
        kv("cards", join(&self.cards));
        kv("equipped", join(&self.equipped));
        kv("kills_kind", join(&self.kills_kind));
        kv("weapon_kills", join(&self.weapon_kills));
        kv("runs", self.runs.to_string());
        kv("kills", self.kills.to_string());
        kv("play_time", format!("{:.0}", self.play_time));
        kv("best_sector", self.best_sector.to_string());
        kv("checkpoint", self.checkpoint.to_string());
        kv("ship2", self.ship2.to_string());
        kv("victories", self.victories.to_string());
        kv("bombs_used", self.bombs_used.to_string());
        kv("grazes", self.grazes.to_string());
        kv("crystals_total", self.crystals_total.to_string());
        kv("best_combo", self.best_combo.to_string());
        kv("bosses", self.bosses.to_string());
        kv("deaths", self.deaths.to_string());
        kv("merchants_spared", self.merchants_spared.to_string());
        kv("best_survival", self.best_survival.to_string());
        kv("best_rush", format!("{:.1}", self.best_rush));
        kv("daily_done", self.daily_done.to_string());
        kv("daily_last", self.daily_last.to_string());
        kv("daily_best", self.daily_best.to_string());
        kv("logs", bits(&self.logs));
        kv("events_seen", bits(&self.events_seen));
        kv("contracts_done", self.contracts_done.to_string());
        kv("mimics", self.mimics.to_string());
        kv("elites", self.elites.to_string());
        kv("frozen_total", self.frozen_total.to_string());
        kv("absorbed_total", self.absorbed_total.to_string());
        kv("cows", self.cows.to_string());
        let b = |x: bool| if x { "1" } else { "0" }.to_string();
        kv("particles", self.particles.to_string());
        kv("flashes", b(self.flashes));
        kv("show_fps", b(self.show_fps));
        kv("lighting", b(self.lighting));
        kv("rumble", b(self.rumble));
        let (keys, buttons) = self.binds.encode();
        kv("keys", keys);
        kv("buttons", buttons);
        kv("autofire", b(self.autofire));
        kv("game_speed", self.game_speed.to_string());
        kv("panels", b(self.panels));
        kv("bg_anim", b(self.bg_anim));
        kv("show_hitbox", b(self.show_hitbox));
        kv("hints", b(self.hints));
        kv("trail", self.trail.to_string());
        kv("relics", bits(&self.relics));
        kv("expedition_best", self.expedition_best.to_string());
        kv("seen_intro", b(self.seen_intro));
        kv("words", self.words.to_string());
        kv("lang", if self.lang == 1 { "pt" } else { "en" }.to_string());
        kv("ufos", self.ufos.to_string());
        kv("curses_taken", self.curses_taken.to_string());
        kv("curses_shot", self.curses_shot.to_string());
        kv("minigames_won", self.minigames_won.to_string());
        kv("minigames_played", self.minigames_played.to_string());
        kv("perfects", self.perfects.to_string());
        kv("rooms_found", self.rooms_found.to_string());
        for e in &self.scores {
            kv(
                "score",
                format!("{};{};{};{};{}", e.name, e.score, e.sector, e.mode, e.ship),
            );
        }
        let dir = data_dir();
        let _ = std::fs::create_dir_all(&dir);
        let tmp = dir.join("save.tmp");
        if std::fs::write(&tmp, o).is_ok() {
            let _ = std::fs::rename(&tmp, file());
        }
    }

    pub fn unique_cards(&self) -> usize {
        self.cards.iter().filter(|&&n| n > 0).count()
    }

    pub fn rank(&self) -> usize {
        crate::progress::rank_of(self.xp)
    }

    pub fn scores_for(&self, mode: u8) -> Vec<&ScoreEntry> {
        self.scores.iter().filter(|e| e.mode == mode).collect()
    }

    pub fn best_for(&self, mode: u8) -> u32 {
        self.scores_for(mode).first().map(|e| e.score).unwrap_or(0)
    }

    pub fn qualifies(&self, score: u32, mode: u8) -> bool {
        let list = self.scores_for(mode);
        score > 0 && (list.len() < 10 || score > list.last().map(|e| e.score).unwrap_or(0))
    }

    /// Inserts and returns the position within the mode (0 = first).
    pub fn add_score(&mut self, name: &str, score: u32, sector: u32, mode: u8, ship: u8) -> usize {
        let e = ScoreEntry {
            name: clean_name(name),
            score,
            sector,
            mode,
            ship,
        };
        self.scores.push(e);
        self.sort_scores();
        self.scores_for(mode)
            .iter()
            .position(|x| x.score == score && x.name == clean_name(name))
            .unwrap_or(0)
    }
}

pub fn clean_name(s: &str) -> String {
    let n: String = s
        .chars()
        .filter(|c| !c.is_control() && *c != ';' && *c != '=')
        .take(NAME_MAX)
        .collect();
    let n = n.trim().to_string();
    if n.is_empty() { "PILOT".into() } else { n }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_and_loads() {
        let dir = std::env::temp_dir().join(format!("stardust-teste-{}", std::process::id()));
        // SAFETY: the only test that touches this variable.
        unsafe { std::env::set_var("XDG_DATA_HOME", &dir) };
        let mut s = Save::default();
        s.music = 3;
        s.crt = false;
        s.cards[5] = 2;
        s.equipped = vec![5];
        s.xp = 12_345;
        s.achs[3] = true;
        s.seen[7] = true;
        s.kills_kind[2] = 99;
        s.add_score("Allan", 99_999, 4, 0, 2);
        s.add_score("Bia", 77_777, 2, 3, 1);
        s.write();
        let l = Save::load();
        assert_eq!(l.music, 3);
        assert!(!l.crt);
        assert_eq!(l.cards[5], 2);
        assert_eq!(l.equipped, vec![5]);
        assert_eq!(l.xp, 12_345);
        assert!(l.achs[3] && !l.achs[2]);
        assert!(l.seen[7]);
        assert_eq!(l.kills_kind[2], 99);
        assert_eq!(l.scores_for(0)[0].name, "Allan");
        assert_eq!(l.scores_for(3)[0].name, "Bia");
        assert_eq!(l.scores_for(0).len(), 10);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ranking() {
        let mut s = Save::default();
        assert!(!s.qualifies(1_000, 0));
        assert!(s.qualifies(1_000_000, 0));
        assert_eq!(s.add_score("X", 1_000_000, 5, 0, 0), 0);
        assert_eq!(s.scores_for(0).len(), 10);
        assert_eq!(clean_name("  a;b=c\n "), "abc");
        assert_eq!(clean_name(""), "PILOT");
    }
}
