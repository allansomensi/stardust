#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//! STARDUST — um shoot 'em up de formações com cara de shareware dos anos 2000.

mod art;
mod audio;
mod boss;
mod cards;
mod fx;
mod game;
mod i18n;
mod icons;
mod progress;
mod save;
mod ships;
mod sprites;
mod ui;

use art::Art;
use audio::{Audio, Music, Sfx};
use cards::CARDS;
use fx::{Background, Fx};
use game::{Event, Game, Kind, RunConfig};
use macroquad::miniquad::conf::Icon as WinIcon;
use macroquad::prelude::*;
use macroquad::rand::gen_range;
use progress::{ACHS, BESTIARY, DIFFS, Difficulty, MODES, Mode, RANKS, ach};
use save::Save;
use ships::{SHIPS, WEAPONS, Weapon};
use sprites::Sprites;
use ui::{CYAN, DIM, GOLD, GREEN, PINK, RED};

pub const W: f32 = 480.0;
pub const H: f32 = 720.0;
/// Variáveis de teste (robô, modo deus, telas diretas...). Só existem no build de
/// desenvolvimento: `cargo run --release --features dev`.
pub fn dev_env(k: &str) -> Option<String> {
    if cfg!(feature = "dev") {
        std::env::var(k).ok()
    } else {
        let _ = k;
        None
    }
}

pub const VERSION: &str = "0.1.0";

fn window_conf() -> Conf {
    let icon = |n: usize| sprites::window_icon_rgba(n);
    let mut small = [0u8; 16 * 16 * 4];
    small.copy_from_slice(&icon(16));
    let mut medium = [0u8; 32 * 32 * 4];
    medium.copy_from_slice(&icon(32));
    let mut big = [0u8; 64 * 64 * 4];
    big.copy_from_slice(&icon(64));
    Conf {
        window_title: "Stardust".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        window_resizable: true,
        icon: Some(WinIcon { small, medium, big }),
        ..Default::default()
    }
}

// ---------------------------------------------------------------------------
// Entrada
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct Input {
    pub swap: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub fire: bool,
    pub fire_p: bool,
    pub focus: bool,
    pub bomb: bool,
    pub ability_p: bool,
    pub pause: bool,
    pub confirm: bool,
    pub enter: bool,
    pub back: bool,
    pub up_p: bool,
    pub down_p: bool,
    pub left_p: bool,
    pub right_p: bool,
    pub backspace: bool,
    pub chars: Vec<char>,
    pub any: bool,
    pub key: Option<KeyCode>,
}

#[derive(Default)]
struct Repeat {
    held: [f32; 4],
}

impl Repeat {
    fn step(&mut self, i: usize, down: bool, dt: f32) -> bool {
        if !down {
            self.held[i] = 0.0;
            return false;
        }
        let before = self.held[i];
        self.held[i] += dt;
        if before == 0.0 {
            return true;
        }
        let delay = 0.35;
        let rate = 0.075;
        if self.held[i] > delay {
            let a = ((before - delay) / rate).floor();
            let b = ((self.held[i] - delay) / rate).floor();
            return before <= delay || b > a;
        }
        false
    }
}

fn any_down(keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| is_key_down(*k))
}
fn any_pressed(keys: &[KeyCode]) -> bool {
    keys.iter().any(|k| is_key_pressed(*k))
}

fn poll_input(rep: &mut Repeat, dt: f32) -> Input {
    use KeyCode::*;
    let left = any_down(&[Left, A]);
    let right = any_down(&[Right, D]);
    let up = any_down(&[Up, W]);
    let down = any_down(&[Down, S]);
    let mut chars = vec![];
    while let Some(c) = get_char_pressed() {
        chars.push(c);
    }
    let key = get_last_key_pressed();
    Input {
        left,
        right,
        up,
        down,
        fire: any_down(&[Space, Z, J]),
        fire_p: any_pressed(&[Space, Z, J]),
        focus: any_down(&[LeftShift, RightShift]),
        bomb: any_pressed(&[X, K]),
        swap: any_pressed(&[Q, Tab, E]),
        ability_p: any_pressed(&[C, L, V]),
        pause: any_pressed(&[Escape, P]),
        confirm: any_pressed(&[Enter, KpEnter, Space, Z]),
        enter: any_pressed(&[Enter, KpEnter]),
        back: any_pressed(&[Escape, Backspace]),
        up_p: rep.step(0, up, dt),
        down_p: rep.step(1, down, dt),
        left_p: rep.step(2, left, dt),
        right_p: rep.step(3, right, dt),
        backspace: is_key_pressed(Backspace),
        chars,
        any: key.is_some(),
        key,
    }
}

// ---------------------------------------------------------------------------
// Telas
// ---------------------------------------------------------------------------

enum Screen {
    Title {
        sel: usize,
    },
    ModeSelect {
        sel: usize,
        diff: usize,
        row: usize,
    },
    Hangar {
        sel: usize,
        skin: usize,
        row: usize,
    },
    Playing,
    Paused {
        sel: usize,
        tip: usize,
    },
    Summary {
        t: f32,
        gain: u64,
        from_xp: u64,
        shown_ranks: usize,
        next: Box<Screen>,
    },
    NameEntry {
        name: String,
        score: u32,
        sector: u32,
        mode: u8,
        ship: u8,
    },
    Scores {
        mode: usize,
        highlight: Option<usize>,
    },
    Album {
        sel: usize,
    },
    Archives {
        tab: usize,
        sel: usize,
    },
    Options {
        sel: usize,
        from_pause: bool,
    },
    HowTo {
        page: usize,
    },
    Prologue {
        t: f32,
        start: Option<(Mode, Difficulty)>,
    },
    Sheet,
}

struct Toast {
    title: String,
    sub: String,
    t: f32,
    color: Color,
    dur: f32,
}

struct App {
    screen: Screen,
    save: Save,
    audio: Audio,
    sp: Sprites,
    art: Art,
    game: Option<Game>,
    bg: Background,
    fx: Fx,
    time: f32,
    quit: bool,
    autoplay: bool,
    god: bool,
    start_wave: u32,
    theme_t: f32,
    toasts: Vec<Toast>,
    idle: f32,
    demo: bool,
    demo_t: f32,
    keys: Vec<KeyCode>,
    typed: String,
    logo_party: f32,
    panels: bool,
    hangar_trail: usize,
    reset_armed: f32,
}

const TITLE_ITEMS: [&str; 8] = [
    "JOGAR",
    "HANGAR",
    "CARTAS",
    "ARQUIVOS",
    "RECORDES",
    "CONFIGURAÇÕES",
    "COMO JOGAR",
    "SAIR",
];

const PROLOGUE: [&str; 9] = [
    "Ano 2097.",
    "A sonda STARDUST voltava de um cometa distante",
    "trazendo amostras de poeira estelar... e algo mais.",
    "Na poeira havia um sinal. E o sinal chamou o ENXAME.",
    "Milhões de insetos estelares cercaram a Terra em formação de ataque.",
    "A frota caiu em horas. Sobrou um hangar escondido, uma nave e você.",
    "Cada cristal recolhido vira armas melhores no hangar móvel.",
    "Atravesse os 8 setores. Chegue ao Coração da Colmeia.",
    "Ninguém sabe o que o Enxame quer. Boa sorte, piloto.",
];

const TIPS: [&str; 22] = [
    "Raspar nos tiros inimigos enche a barra de habilidade.",
    "Segure SHIFT para ver exatamente onde a nave pode ser atingida.",
    "Escudeiros só protegem a frente. Tiros em diagonal passam.",
    "O Olho só sente dor com a pálpebra aberta.",
    "No Cinturão de Gelo, sua nave escorrega. Antecipe-se.",
    "Tecelãs não machucam: elas te deixam lento. O que é pior.",
    "Cristais compram armas melhores. Guarde para o chefe.",
    "O NÚCLEO DE DANO da loja deixa todas as armas mais fortes.",
    "Armas mais caras na loja batem muito mais forte.",
    "Segure o tiro e solte: o tiro carregado atravessa a formação.",
    "Caveiras caem devagar e te seguem. Três tiros acabam com elas.",
    "Minas piscam cada vez mais rápido antes de explodir.",
    "Inimigos em mergulho valem o dobro de pontos.",
    "Combos altos multiplicam os pontos. Não deixe esfriar.",
    "Pegar a mesma arma de novo sobe o nível dela.",
    "Inimigos congelados levam dano em dobro.",
    "Ladrões fogem com o que cai. Derrube-os antes que sumam.",
    "Limpar uma onda rápido rende bônus de tempo.",
    "Contratos cumpridos dão XP extra no fim da partida.",
    "A cauda do Cometa Wild solta poeira. Fique na frente dele.",
    "Na Expedição, cada bênção muda a sua estratégia.",
    "Bombas apagam os tiros na tela. Use antes de ser atingido.",
];

const KONAMI: [KeyCode; 10] = [
    KeyCode::Up,
    KeyCode::Up,
    KeyCode::Down,
    KeyCode::Down,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::Left,
    KeyCode::Right,
    KeyCode::B,
    KeyCode::A,
];

impl App {
    fn rank(&self) -> usize {
        self.save.rank()
    }

    fn mode_unlocked(&self, m: Mode) -> bool {
        self.rank() >= m.unlock_rank()
    }

    fn diff_unlocked(&self, d: Difficulty) -> bool {
        match d {
            Difficulty::Facil | Difficulty::Normal => true,
            Difficulty::Dificil => self.rank() >= 5,
            Difficulty::Pesadelo => {
                self.save.achs[ach::FINAL_VERDADEIRO] || self.save.victories >= 3
            }
        }
    }

    fn toast(&mut self, title: impl Into<String>, sub: impl Into<String>, color: Color) {
        let dur = if color == CYAN { 2.2 } else { 3.6 };
        self.toasts.push(Toast {
            title: title.into(),
            sub: sub.into(),
            t: 0.0,
            color,
            dur,
        });
    }

    fn unlock(&mut self, a: usize) {
        if a >= self.save.achs.len() || self.save.achs[a] {
            return;
        }
        let old_rank = self.rank();
        self.save.achs[a] = true;
        self.save.xp += ACHS[a].xp;
        if let Some(g) = &mut self.game
            && a < g.cfg.achs.len()
        {
            g.cfg.achs[a] = true;
        }
        self.toast(
            format!("CONQUISTA: {}", ACHS[a].name),
            format!("{}  (+{} XP)", ACHS[a].desc, ACHS[a].xp),
            GOLD,
        );
        self.audio.play(Sfx::Achievement);
        if self.rank() > old_rank {
            self.toast(
                format!("PROMOÇÃO: {}", RANKS[self.rank()]),
                progress::unlock_text(self.rank()),
                CYAN,
            );
        }
        self.check_meta();
        self.save.write();
    }

    fn add_card(&mut self, c: usize) {
        if c < self.save.cards.len() {
            self.save.cards[c] += 1;
        }
    }

    /// Conquistas que dependem do progresso acumulado.
    fn check_meta(&mut self) {
        let s = &self.save;
        let mut list = vec![];
        if s.unique_cards() >= 20 {
            list.push(ach::COLECIONADOR);
        }
        if CARDS
            .iter()
            .enumerate()
            .all(|(i, c)| c.secret || s.cards[i] > 0)
        {
            list.push(ach::COMPLETISTA);
        }
        if s.runs >= 25 {
            list.push(ach::INSISTENTE);
        }
        if s.play_time >= 7200.0 {
            list.push(ach::MARATONA);
        }
        if s.kills_kind[Kind::Kamikaze.index()] >= 150 {
            list.push(ach::ANTI_KAMIKAZE);
        }
        if s.bombs_used >= 50 {
            list.push(ach::BOMBARDEIRO);
        }
        if s.weapon_kills[Weapon::Chama.index()] >= 300 {
            list.push(ach::PIROMANIACO);
        }
        if s.mastery.iter().filter(|m| **m).count() >= 5 {
            list.push(ach::MESTRE_ARMAS);
        }
        if s.merchants_spared >= 3 {
            list.push(ach::SAMARITANO);
        }
        if s.daily_done >= 3 {
            list.push(ach::ROTINA);
        }
        let logs = s.logs.iter().filter(|l| **l).count();
        for (cond, a) in [
            (s.contracts_done >= 30, ach::MERCENARIO),
            (logs >= 6, ach::ARQUEOLOGO),
            (logs >= progress::LOGS.len(), ach::HISTORIADOR),
            (s.mimics >= 10, ach::DESCONFIADO),
            (s.elites >= 50, ach::ELITE),
            (s.frozen_total >= 150, ach::GELO),
            (s.absorbed_total >= 300, ach::HORIZONTE),
            (
                s.mastery.iter().filter(|m| **m).count() >= 12,
                ach::MESTRE_SUPREMO,
            ),
            (s.cows >= 3, ach::FAZENDEIRO),
            (s.events_seen.iter().all(|e| *e), ach::TURISTA),
            (s.minigames_won >= 10, ach::FLIPERAMISTA),
            (s.curses_shot >= 25, ach::EXORCISTA),
            (s.curses_taken >= 10, ach::AZARADO),
            (s.perfects >= 5, ach::PERFECCIONISTA),
            (s.relics.iter().all(|r| *r), ach::RELIQUIAS),
            (s.rooms_found >= 5, ach::TESOUREIRO),
            (s.expedition_best >= 10, ach::EXPEDICIONARIO),
        ] {
            if cond {
                list.push(a);
            }
        }
        let rank = s.rank();
        if (0..7).all(|i| ships::ship_unlocked(i, rank, &s.achs)) {
            list.push(ach::HANGAR);
        }
        if (0..SHIPS.len()).all(|i| ships::ship_unlocked(i, rank, &s.achs)) {
            list.push(ach::FROTA);
        }
        for a in list {
            if !self.save.achs[a] {
                if a == ach::FAZENDEIRO {
                    self.add_card(cards::id::BEZERRO);
                    self.toast(
                        "UM BEZERRO TE SEGUE",
                        "Carta secreta: Bezerro Cósmico.",
                        PINK,
                    );
                }
                self.unlock(a);
            }
        }
    }

    fn new_game(&mut self, mode: Mode, diff: Difficulty, demo: bool) {
        let mut ship = self.save.ship;
        if !ships::ship_unlocked(ship, self.rank(), &self.save.achs) {
            ship = 0;
        }
        let daily = if mode == Mode::Diario {
            let day = progress::today_number();
            macroquad::rand::srand(day as u64 * 7919);
            Some(progress::daily_for(day))
        } else {
            None
        };
        let cfg = RunConfig {
            mode,
            diff,
            ship,
            skin: self.save.skin,
            daily,
            equipped: if demo {
                vec![]
            } else {
                self.save.equipped.clone()
            },
            rank: self.rank(),
            achs: self.save.achs.clone(),
            owned_cards: self.save.cards.clone(),
            found_logs: self.save.logs.clone(),
            highscore: self.save.best_for(mode.tag()),
            trail: if progress::trail_unlocked(self.save.trail, &self.save) {
                self.save.trail
            } else {
                0
            },
            show_hitbox: self.save.show_hitbox,
        };
        let mut g = Game::new(cfg);
        if self.start_wave > 1 {
            g.wave = self.start_wave;
        }
        if self.god {
            g.player.lives = 99;
            g.player.level = 5;
        }
        if let Some(w) = dev_env("STARDUST_WEAPON").and_then(|v| v.parse::<usize>().ok()) {
            g.player.weapon = WEAPONS[w.min(WEAPONS.len() - 1)];
        }
        if dev_env("STARDUST_DROPS").is_some() {
            g.debug_showcase();
        }
        if let Some(k) = dev_env("STARDUST_MINIGAME").and_then(|v| v.parse::<usize>().ok()) {
            g.debug_minigame(k);
        }
        if dev_env("STARDUST_V5").is_some() {
            g.debug_v5();
        }
        if dev_env("STARDUST_BLESS").is_some() {
            g.debug_bless();
        }
        if dev_env("STARDUST_ROOM").is_some() {
            g.debug_room();
        }
        if dev_env("STARDUST_STAR").is_some() {
            g.debug_star();
        }
        self.game = Some(g);
        self.demo = demo;
        self.demo_t = 0.0;
        self.screen = Screen::Playing;
    }

    fn end_run(&mut self) {
        let Some(g) = self.game.take() else { return };
        if self.demo {
            self.demo = false;
            self.idle = 0.0;
            self.screen = Screen::Title { sel: 0 };
            self.audio.music(Some(Music::Title));
            return;
        }
        let s = &mut self.save;
        s.runs += 1;
        s.best_sector = s.best_sector.max(g.absolute_sector());
        s.best_combo = s.best_combo.max(g.best_combo);
        s.deaths += g.deaths;
        s.frozen_total += g.frozen_count;
        s.absorbed_total += g.absorbed;
        let mode = g.cfg.mode;
        match mode {
            Mode::Classico if g.score >= 60_000 => {
                self.unlock(ach::FLIPERAMA);
            }
            Mode::Chefoes if g.victory => {
                let t = g.run_time;
                if self.save.best_rush == 0.0 || t < self.save.best_rush {
                    self.save.best_rush = t;
                }
            }
            Mode::Expedicao => {
                self.save.expedition_best = self.save.expedition_best.max(g.sector());
            }
            Mode::Diario => {
                self.save.daily_last = progress::today_number();
                self.save.daily_best = self.save.daily_best.max(g.score);
                if g.victory {
                    self.save.daily_done += 1;
                }
            }
            _ => {}
        }
        let base = g.score as f64 / 50.0
            + g.kills as f64 * 2.0
            + g.bosses_killed as f64 * 150.0
            + g.sectors_cleared as f64 * 100.0
            + g.new_cards.len() as f64 * 50.0
            + g.x.contract_xp as f64;
        let gain = (base * g.cfg.diff.xp_mul() as f64).round() as u64;
        let from_xp = self.save.xp;
        self.save.xp += gain;
        self.check_meta();
        self.save.write();
        let next = if self.save.qualifies(g.score, mode.tag()) {
            Screen::NameEntry {
                name: self.save.last_name.clone(),
                score: g.score,
                sector: g.absolute_sector(),
                mode: mode.tag(),
                ship: g.cfg.ship as u8,
            }
        } else {
            Screen::Title { sel: 0 }
        };
        self.screen = Screen::Summary {
            t: 0.0,
            gain,
            from_xp,
            shown_ranks: progress::rank_of(from_xp),
            next: Box::new(next),
        };
        self.audio.music(Some(Music::Title));
    }

    fn apply_settings(&mut self) {
        self.audio.set_music_volume(self.save.music as f32 / 10.0);
        self.audio.sfx_vol = self.save.sfx as f32 / 10.0;
        i18n::set_lang(if self.save.lang == 1 {
            i18n::Lang::En
        } else {
            i18n::Lang::Pt
        });
        fx::set_quality(
            [0.35, 0.7, 1.0][self.save.particles.min(2) as usize],
            if self.save.flashes { 1.0 } else { 0.3 },
            !self.save.bg_anim,
        );
    }

    fn handle_events(&mut self) {
        let Some(g) = &mut self.game else { return };
        if self.demo {
            g.events.clear();
            g.pending_cards.clear();
            return;
        }
        let events: Vec<Event> = g.events.drain(..).collect();
        let cards: Vec<usize> = g.pending_cards.drain(..).collect();
        let mut dirty = !cards.is_empty();
        for c in cards {
            self.add_card(c);
        }
        if let Some(g) = &mut self.game {
            g.cfg.owned_cards = self.save.cards.clone();
        }
        for ev in events {
            match ev {
                Event::Ach(a) => self.unlock(a),
                Event::Seen(i) => {
                    if i < self.save.seen.len() && !self.save.seen[i] {
                        self.save.seen[i] = true;
                        dirty = true;
                        self.toast("NOVO REGISTRO NO BESTIÁRIO", BESTIARY[i].name, CYAN);
                    }
                }
                Event::Kill(kind, weapon) => {
                    self.save.kills += 1;
                    if let Some(k) = self.save.kills_kind.get_mut(kind.index()) {
                        *k += 1;
                    }
                    if let Some(w) = weapon {
                        self.save.weapon_kills[w.index()] += 1;
                    }
                }
                Event::BossKilled => self.save.bosses += 1,
                Event::Bomb => self.save.bombs_used += 1,
                Event::Graze => self.save.grazes += 1,
                Event::Crystals(n) => self.save.crystals_total += n,
                Event::Mastery(w) => {
                    if !self.save.mastery[w] {
                        self.save.mastery[w] = true;
                        dirty = true;
                    }
                }
                Event::MerchantSpared => {
                    self.save.merchants_spared += 1;
                    dirty = true;
                }
                Event::Victory => {
                    self.save.victories += 1;
                    dirty = true;
                }
                Event::SurvivalWave(w) => {
                    self.save.best_survival = self.save.best_survival.max(w);
                    if w >= 30 {
                        self.unlock(ach::SOBREVIVENTE);
                    }
                }
                Event::Contract => {
                    self.save.contracts_done += 1;
                    self.unlock(ach::CONTRATADO);
                    dirty = true;
                }
                Event::Log(i) => {
                    if i < self.save.logs.len() && !self.save.logs[i] {
                        self.save.logs[i] = true;
                        self.toast(
                            "NOVO REGISTRO DE BORDO",
                            progress::LOGS[i].0,
                            Color::from_rgba(120, 255, 220, 255),
                        );
                        dirty = true;
                    }
                }
                Event::Elite => self.save.elites += 1,
                Event::Mimic => self.save.mimics += 1,
                Event::Cow => {
                    self.save.cows += 1;
                    dirty = true;
                }
                Event::SectorEvent(i) => {
                    if i < self.save.events_seen.len() && !self.save.events_seen[i] {
                        self.save.events_seen[i] = true;
                        dirty = true;
                    }
                }
                Event::CurseTaken => {
                    self.save.curses_taken += 1;
                    dirty = true;
                }
                Event::CurseShot => {
                    self.save.curses_shot += 1;
                    dirty = true;
                }
                Event::Relic(i) => {
                    if i < self.save.relics.len() && !self.save.relics[i] {
                        self.save.relics[i] = true;
                        self.toast("RELÍQUIA DESCOBERTA!", ships::RELICS[i].name(), GOLD);
                    }
                    self.unlock(ach::RELICARIO);
                    dirty = true;
                }
                Event::Minigame(grade) => {
                    self.save.minigames_played += 1;
                    if grade >= 1 {
                        self.save.minigames_won += 1;
                    }
                    if grade >= 2 {
                        self.save.perfects += 1;
                    }
                    dirty = true;
                }
                Event::Word => {
                    self.save.words += 1;
                    dirty = true;
                }
                Event::Ufo => {
                    self.save.ufos += 1;
                    dirty = true;
                }
                Event::Room => {
                    self.save.rooms_found += 1;
                    self.unlock(ach::SALA_SECRETA);
                    dirty = true;
                }
            }
        }
        if dirty {
            self.check_meta();
            self.save.write();
        }
    }

    fn update(&mut self, dt: f32, inp: &Input) {
        self.time += dt;
        self.save.play_time += dt as f64;
        self.fx.update(dt);
        self.theme_t += dt;
        self.logo_party = (self.logo_party - dt).max(0.0);
        for t in &mut self.toasts {
            t.t += dt;
        }
        // uma de cada vez: só a primeira anda
        if let Some(first) = self.toasts.first()
            && first.t > first.dur
        {
            self.toasts.remove(0);
        }
        for t in self.toasts.iter_mut().skip(1) {
            t.t = 0.0;
        }
        if !matches!(self.screen, Screen::Playing | Screen::Paused { .. }) {
            self.bg.update(dt);
            if self.theme_t > 12.0 {
                self.theme_t = 0.0;
                let next = (self.bg.theme + 1) % 8;
                self.bg.set_theme(next);
            }
        }

        let screen = std::mem::replace(&mut self.screen, Screen::Title { sel: 0 });
        self.screen = match screen {
            Screen::Sheet => Screen::Sheet,
            Screen::Prologue { t, start } => {
                let t = t + dt;
                self.audio.music(Some(Music::Credits));
                if (t > 1.0 && (inp.confirm || inp.back)) || t > PROLOGUE.len() as f32 * 3.2 + 4.0 {
                    self.save.seen_intro = true;
                    self.save.write();
                    self.audio.play(Sfx::MenuOk);
                    if let Some((m, d)) = start {
                        self.new_game(m, d, false);
                        Screen::Playing
                    } else {
                        Screen::Options {
                            sel: 204,
                            from_pause: false,
                        }
                    }
                } else {
                    Screen::Prologue { t, start }
                }
            }
            Screen::Title { sel } => self.update_title(sel, dt, inp),
            Screen::ModeSelect { sel, diff, row } => self.update_mode_select(sel, diff, row, inp),
            Screen::Hangar { sel, skin, row } => self.update_hangar(sel, skin, row, inp),
            Screen::Playing => {
                if self.demo {
                    self.demo_t += dt;
                    if self.demo_t > 60.0 {
                        self.unlock(ach::INSONE);
                    }
                    if (inp.any && !self.autoplay)
                        || self.demo_t > 90.0
                        || self.game.as_ref().map(|g| g.finished).unwrap_or(true)
                    {
                        self.end_run();
                        return;
                    }
                    let bot = bot_input(self.game.as_ref().unwrap(), self.time);
                    self.game
                        .as_mut()
                        .unwrap()
                        .update(dt, &bot, &mut self.audio);
                    self.handle_events();
                    self.screen = Screen::Playing;
                    return;
                }
                let g = self.game.as_mut().expect("jogo ativo");
                let blocked = matches!(
                    g.phase,
                    game::Phase::Shop | game::Phase::Continue { .. } | game::Phase::Credits { .. }
                );
                if inp.pause && !blocked {
                    self.audio.play(Sfx::MenuMove);
                    Screen::Paused {
                        sel: 0,
                        tip: gen_range(0, TIPS.len()),
                    }
                } else {
                    g.update(dt, inp, &mut self.audio);
                    let finished = g.finished;
                    self.handle_events();
                    if finished {
                        self.end_run();
                        return;
                    }
                    Screen::Playing
                }
            }
            Screen::Paused { mut sel, tip } => {
                let items = 3;
                if inp.up_p {
                    sel = (sel + items - 1) % items;
                    self.audio.play(Sfx::MenuMove);
                }
                if inp.down_p {
                    sel = (sel + 1) % items;
                    self.audio.play(Sfx::MenuMove);
                }
                if inp.pause {
                    Screen::Playing
                } else if inp.confirm {
                    self.audio.play(Sfx::MenuOk);
                    match sel {
                        0 => Screen::Playing,
                        1 => Screen::Options {
                            sel: 1,
                            from_pause: true,
                        },
                        _ => {
                            self.end_run();
                            return;
                        }
                    }
                } else {
                    Screen::Paused { sel, tip }
                }
            }
            Screen::Summary {
                t,
                gain,
                from_xp,
                mut shown_ranks,
                next,
            } => {
                let t = t + dt;
                let shown_xp = from_xp + ((t - 0.8).clamp(0.0, 2.0) / 2.0 * gain as f32) as u64;
                let r = progress::rank_of(shown_xp);
                if r > shown_ranks {
                    shown_ranks = r;
                    self.audio.play(Sfx::RankUp);
                    self.fx.burst(vec2(W / 2.0, 300.0), GOLD, 60, 300.0);
                }
                if t > 0.8 && t < 2.8 && (t * 20.0) as i32 % 2 == 0 {
                    self.audio.play_vol(Sfx::Graze, 0.3);
                }
                if t > 3.0 && (inp.confirm || self.autoplay) {
                    self.audio.play(Sfx::MenuOk);
                    *next
                } else {
                    Screen::Summary {
                        t,
                        gain,
                        from_xp,
                        shown_ranks,
                        next,
                    }
                }
            }
            Screen::NameEntry {
                mut name,
                score,
                sector,
                mode,
                ship,
            } => {
                for c in &inp.chars {
                    if name.chars().count() < save::NAME_MAX
                        && !c.is_control()
                        && *c != ';'
                        && *c != '='
                    {
                        name.extend(c.to_uppercase());
                    }
                }
                if inp.backspace {
                    name.pop();
                }
                if inp.enter || self.autoplay {
                    let n = save::clean_name(&name);
                    self.secret_names(&n);
                    self.save.last_name = n.clone();
                    let pos = self.save.add_score(&n, score, sector, mode, ship);
                    self.save.write();
                    self.audio.play(Sfx::MenuOk);
                    Screen::Scores {
                        mode: mode as usize,
                        highlight: Some(pos),
                    }
                } else {
                    Screen::NameEntry {
                        name,
                        score,
                        sector,
                        mode,
                        ship,
                    }
                }
            }
            Screen::Scores {
                mut mode,
                highlight,
            } => {
                if inp.left_p || inp.right_p {
                    let n = MODES.len();
                    mode = if inp.left_p {
                        (mode + n - 1) % n
                    } else {
                        (mode + 1) % n
                    };
                    self.audio.play(Sfx::MenuMove);
                    Screen::Scores {
                        mode,
                        highlight: None,
                    }
                } else if inp.confirm || inp.back {
                    self.audio.play(Sfx::MenuOk);
                    Screen::Title { sel: 4 }
                } else {
                    Screen::Scores { mode, highlight }
                }
            }
            Screen::Album { sel } => self.update_album(sel, inp),
            Screen::Archives { tab, sel } => self.update_archives(tab, sel, inp),
            Screen::Options { sel, from_pause } => self.update_options(sel, from_pause, inp),
            Screen::HowTo { mut page } => {
                let pages = 6;
                if inp.left_p && page > 0 {
                    page -= 1;
                    self.audio.play(Sfx::MenuMove);
                }
                if (inp.right_p || inp.confirm) && page < pages - 1 {
                    page += 1;
                    self.audio.play(Sfx::MenuMove);
                    Screen::HowTo { page }
                } else if inp.back || (inp.confirm && page == pages - 1) {
                    self.audio.play(Sfx::MenuOk);
                    Screen::Title { sel: 6 }
                } else {
                    Screen::HowTo { page }
                }
            }
        };
    }

    fn secret_names(&mut self, n: &str) {
        let up = n.to_uppercase();
        match up.as_str() {
            "ALLAN" => {
                if !self.save.achs[ach::CRIADOR] {
                    self.add_card(cards::id::CRIADOR);
                }
                self.unlock(ach::CRIADOR);
                self.toast("OLÁ, CRIADOR!", "Obrigado por fazer este jogo.", PINK);
            }
            "VOYAGER" | "VOYAGER 1" | "VOYAGER 2" | "VOYAGER1" | "VOYAGER2" => {
                self.audio.play(Sfx::Sparkle);
                self.unlock(ach::DISCO_OURO);
                self.toast(
                    "DISCO DE OURO",
                    "A arma PULSAR agora cai das cápsulas.",
                    GOLD,
                );
            }
            "MUUU" | "VACA" | "MOO" => self.audio.play(Sfx::Moo),
            _ => {}
        }
    }

    fn title_visible(&self) -> Vec<usize> {
        (0..TITLE_ITEMS.len()).collect()
    }

    fn update_title(&mut self, mut sel: usize, dt: f32, inp: &Input) -> Screen {
        self.audio.music(Some(Music::Title));
        let vis = self.title_visible();
        let n = vis.len();
        sel = sel.min(n - 1);
        // Konami
        if let Some(k) = inp.key {
            self.keys.push(k);
            if self.keys.len() > KONAMI.len() {
                self.keys.remove(0);
            }
            if self.keys.as_slice() == KONAMI {
                self.keys.clear();
                if !self.save.achs[ach::KONAMI] {
                    self.add_card(cards::id::KONAMI);
                    self.audio.play(Sfx::Fanfare);
                    self.toast(
                        "CÓDIGO ACEITO",
                        "Nave PROTÓTIPO liberada no Hangar. +1 carta secreta.",
                        GREEN,
                    );
                }
                self.unlock(ach::KONAMI);
                self.logo_party = 4.0;
            }
        }
        // digitar STARDUST
        for c in &inp.chars {
            self.typed.extend(c.to_uppercase());
            if self.typed.chars().count() > 12 {
                self.typed.remove(0);
            }
            if self.typed.ends_with("STARDUST") {
                self.typed.clear();
                self.logo_party = 5.0;
                self.audio.play(Sfx::Sparkle);
                for i in 0..8 {
                    self.fx.burst(
                        vec2(86.0 + i as f32 * 44.0, 190.0),
                        ui::hsl(i as f32 / 8.0, 0.9, 0.6),
                        30,
                        260.0,
                    );
                }
                self.unlock(ach::CURIOSO);
            }
        }
        // demonstração
        if inp.any || inp.up || inp.down {
            self.idle = 0.0;
        } else {
            self.idle += dt;
        }
        if self.idle > 25.0 {
            self.idle = 0.0;
            self.new_game(Mode::Campanha, Difficulty::Normal, true);
            return Screen::Playing;
        }
        if inp.up_p {
            sel = (sel + n - 1) % n;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.down_p {
            sel = (sel + 1) % n;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.enter || (inp.confirm && !matches!(inp.key, Some(KeyCode::Z))) || self.autoplay {
            self.audio.play(Sfx::MenuOk);
            match if self.autoplay {
                0
            } else {
                vis[sel.min(vis.len() - 1)]
            } {
                0 => {
                    if self.autoplay {
                        self.new_game(
                            Mode::from_tag(self.save.mode),
                            Difficulty::from_index(self.save.diff),
                            false,
                        );
                        return Screen::Playing;
                    }
                    let m = (self.save.mode as usize).min(MODES.len() - 1);
                    return Screen::ModeSelect {
                        sel: m,
                        diff: self.save.diff as usize,
                        row: 0,
                    };
                }
                1 => {
                    self.hangar_trail = self.save.trail;
                    return Screen::Hangar {
                        sel: self.save.ship,
                        skin: self.save.skin,
                        row: 0,
                    };
                }
                2 => return Screen::Album { sel: 0 },
                3 => return Screen::Archives { tab: 0, sel: 0 },
                4 => {
                    return Screen::Scores {
                        mode: self.save.mode as usize,
                        highlight: None,
                    };
                }
                5 => {
                    return Screen::Options {
                        sel: 1,
                        from_pause: false,
                    };
                }
                6 => return Screen::HowTo { page: 0 },
                _ => {
                    self.quit = true;
                }
            }
        } else if inp.back && sel != n - 1 {
            self.audio.play(Sfx::MenuMove);
            sel = n - 1;
        }
        Screen::Title { sel }
    }

    fn update_mode_select(
        &mut self,
        mut sel: usize,
        mut diff: usize,
        mut row: usize,
        inp: &Input,
    ) -> Screen {
        let n = MODES.len();
        if inp.up_p || inp.down_p {
            row = 1 - row;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.left_p || inp.right_p {
            let d = if inp.left_p { -1i32 } else { 1 };
            if row == 0 {
                sel = ((sel as i32 + d).rem_euclid(n as i32)) as usize;
            } else {
                diff = ((diff as i32 + d).rem_euclid(DIFFS.len() as i32)) as usize;
            }
            self.audio.play(Sfx::MenuMove);
        }
        if inp.back {
            self.audio.play(Sfx::MenuMove);
            return Screen::Title { sel: 0 };
        }
        if inp.confirm {
            let m = MODES[sel];
            let d = DIFFS[diff];
            if !self.mode_unlocked(m) || !self.diff_unlocked(d) {
                self.audio.play(Sfx::Deny);
            } else {
                self.audio.play(Sfx::MenuOk);
                self.save.mode = m.tag();
                self.save.diff = diff as u8;
                self.save.write();
                if m == Mode::Campanha && !self.save.seen_intro {
                    return Screen::Prologue {
                        t: 0.0,
                        start: Some((m, d)),
                    };
                }
                self.new_game(m, d, false);
                return Screen::Playing;
            }
        }
        Screen::ModeSelect { sel, diff, row }
    }

    fn update_hangar(
        &mut self,
        mut sel: usize,
        mut skin: usize,
        mut row: usize,
        inp: &Input,
    ) -> Screen {
        let n = SHIPS.len();
        if inp.up_p {
            row = (row + 2) % 3;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.down_p {
            row = (row + 1) % 3;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.left_p || inp.right_p {
            let d = if inp.left_p { -1i32 } else { 1 };
            match row {
                0 => sel = ((sel as i32 + d).rem_euclid(n as i32)) as usize,
                1 => skin = ((skin as i32 + d).rem_euclid(art::SKINS as i32)) as usize,
                _ => {
                    self.hangar_trail = ((self.hangar_trail as i32 + d)
                        .rem_euclid(progress::TRAILS.len() as i32))
                        as usize
                }
            }
            self.audio.play(Sfx::MenuMove);
        }
        if inp.confirm && row == 2 {
            if progress::trail_unlocked(self.hangar_trail, &self.save) {
                self.save.trail = self.hangar_trail;
                self.save.write();
                self.audio.play(Sfx::Buy);
                self.toast("RASTRO EQUIPADO", progress::TRAILS[self.hangar_trail], CYAN);
            } else {
                self.audio.play(Sfx::Deny);
            }
        } else if inp.confirm {
            let ok_ship = ships::ship_unlocked(sel, self.rank(), &self.save.achs);
            let ok_skin = self.skin_ok(skin);
            if ok_ship && ok_skin {
                self.save.ship = sel;
                self.save.skin = skin;
                self.save.write();
                self.audio.play(Sfx::Buy);
                self.toast(
                    format!("{} PRONTA PARA DECOLAR", SHIPS[sel].name),
                    progress::SKIN_NAMES[skin],
                    CYAN,
                );
            } else {
                self.audio.play(Sfx::Deny);
            }
        }
        if inp.back {
            self.audio.play(Sfx::MenuMove);
            return Screen::Title { sel: 1 };
        }
        Screen::Hangar { sel, skin, row }
    }

    fn skin_ok(&self, k: usize) -> bool {
        self.rank() >= progress::skin_rank(k) || (k == 6 && self.save.achs[ach::RELIQUIAS])
    }

    fn update_album(&mut self, mut sel: usize, inp: &Input) -> Screen {
        let cols = 6;
        let n = CARDS.len();
        if inp.left_p && !sel.is_multiple_of(cols) {
            sel -= 1;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.right_p && sel % cols < cols - 1 && sel + 1 < n {
            sel += 1;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.up_p && sel >= cols {
            sel -= cols;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.down_p && sel + cols < n {
            sel += cols;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.confirm {
            if self.save.cards[sel] == 0 {
                self.audio.play(Sfx::Deny);
            } else if let Some(i) = self.save.equipped.iter().position(|&c| c == sel) {
                self.save.equipped.remove(i);
                self.audio.play(Sfx::MenuMove);
                self.save.write();
            } else if self.save.equipped.len() < cards::slots_for(self.save.unique_cards()) {
                self.save.equipped.push(sel);
                self.audio.play(Sfx::Buy);
                self.save.write();
            } else {
                self.audio.play(Sfx::Deny);
                self.fx.text(
                    vec2(W / 2.0, H - 60.0),
                    "SEM ESPAÇO: DESEQUIPE OUTRA",
                    RED,
                    16.0,
                );
            }
        }
        if inp.back {
            self.audio.play(Sfx::MenuMove);
            return Screen::Title { sel: 2 };
        }
        Screen::Album { sel }
    }

    fn update_archives(&mut self, mut tab: usize, mut sel: usize, inp: &Input) -> Screen {
        let tabs = 6;
        if inp.left_p || inp.right_p {
            tab = if inp.left_p {
                (tab + tabs - 1) % tabs
            } else {
                (tab + 1) % tabs
            };
            sel = 0;
            self.audio.play(Sfx::MenuMove);
        }
        let len = match tab {
            0 => ACHS.len(),
            1 => BESTIARY.len(),
            2 => 1,
            3 => RANKS.len(),
            4 => progress::LOGS.len(),
            _ => progress::HONORS.len(),
        };
        if inp.up_p && sel > 0 {
            sel -= 1;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.down_p && sel + 1 < len {
            sel += 1;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.back || inp.enter {
            self.audio.play(Sfx::MenuOk);
            return Screen::Title { sel: 3 };
        }
        Screen::Archives { tab, sel }
    }

    fn opt_rows(&self, tab: usize) -> Vec<(String, String)> {
        let yes = |b: bool| if b { "SIM" } else { "NÃO" }.to_string();
        let bar = |v: u8| format!("{}{}", "|".repeat(v as usize), "·".repeat(10 - v as usize));
        let s = &self.save;
        let mut v: Vec<(String, String)> = match tab {
            0 => vec![
                ("TELA CHEIA".into(), yes(s.fullscreen)),
                ("FILTRO CRT".into(), yes(s.crt)),
                ("TREMOR DE TELA".into(), yes(s.shake)),
                (
                    "PARTÍCULAS".into(),
                    ["BAIXO", "MÉDIO", "ALTO"][s.particles as usize].into(),
                ),
                (
                    "CLARÕES".into(),
                    if s.flashes { "NORMAIS" } else { "SUAVES" }.into(),
                ),
                ("PAINÉIS LATERAIS".into(), yes(s.panels)),
                ("FUNDO ANIMADO".into(), yes(s.bg_anim)),
                ("MOSTRAR FPS".into(), yes(s.show_fps)),
            ],
            1 => vec![
                ("MÚSICA".into(), bar(s.music)),
                ("EFEITOS".into(), bar(s.sfx)),
                ("TESTAR EFEITOS".into(), "ENTER".into()),
            ],
            2 => vec![
                (
                    "IDIOMA / LANGUAGE".into(),
                    if s.lang == 1 { "ENGLISH" } else { "PORTUGUÊS" }.into(),
                ),
                ("HITBOX SEMPRE VISÍVEL".into(), yes(s.show_hitbox)),
                ("DICAS NA PAUSA".into(), yes(s.hints)),
                ("REVER O PRÓLOGO".into(), "ENTER".into()),
                (
                    "APAGAR PROGRESSO".into(),
                    if self.reset_armed > 0.0 {
                        "CONFIRMA?".into()
                    } else {
                        "ENTER".into()
                    },
                ),
            ],
            _ => vec![],
        };
        v.push(("VOLTAR".into(), String::new()));
        v
    }

    fn update_options(&mut self, sel: usize, from_pause: bool, inp: &Input) -> Screen {
        let mut tab = (sel / 100).min(3);
        let mut row = sel % 100;
        let n = self.opt_rows(tab).len() + 1; // linha 0 = abas
        self.reset_armed = (self.reset_armed - get_frame_time()).max(0.0);
        if inp.up_p {
            row = (row + n - 1) % n;
            self.audio.play(Sfx::MenuMove);
        }
        if inp.down_p {
            row = (row + 1) % n;
            self.audio.play(Sfx::MenuMove);
        }
        let delta: i32 = if inp.left_p {
            -1
        } else if inp.right_p {
            1
        } else {
            0
        };
        if inp.swap || (row == 0 && delta != 0) {
            let d = if delta == 0 { 1 } else { delta };
            tab = ((tab as i32 + d).rem_euclid(4)) as usize;
            if row != 0 {
                row = 1;
            }
            self.audio.play(Sfx::MenuMove);
            return Screen::Options {
                sel: tab * 100 + row,
                from_pause,
            };
        }
        let toggle = inp.confirm || delta != 0;
        let last = n - 1;
        let mut changed = false;
        if row > 0 && row < last {
            let s = &mut self.save;
            let step = |v: u8, max: u8| {
                ((v as i32 + if delta == 0 { 1 } else { delta }).rem_euclid(max as i32 + 1)) as u8
            };
            match (tab, row) {
                (0, 1) if toggle => {
                    s.fullscreen = !s.fullscreen;
                    set_fullscreen(s.fullscreen);
                    changed = true;
                }
                (0, 2) if toggle => {
                    s.crt = !s.crt;
                    changed = true;
                }
                (0, 3) if toggle => {
                    s.shake = !s.shake;
                    changed = true;
                }
                (0, 4) if toggle => {
                    s.particles = step(s.particles, 2);
                    changed = true;
                }
                (0, 5) if toggle => {
                    s.flashes = !s.flashes;
                    changed = true;
                }
                (0, 6) if toggle => {
                    s.panels = !s.panels;
                    changed = true;
                }
                (0, 7) if toggle => {
                    s.bg_anim = !s.bg_anim;
                    changed = true;
                }
                (0, 8) if toggle => {
                    s.show_fps = !s.show_fps;
                    changed = true;
                }
                (1, 1) if delta != 0 => {
                    s.music = (s.music as i32 + delta).clamp(0, 10) as u8;
                    changed = true;
                }
                (1, 2) if delta != 0 => {
                    s.sfx = (s.sfx as i32 + delta).clamp(0, 10) as u8;
                    changed = true;
                }
                (1, 3) if inp.confirm => {
                    self.audio.play(Sfx::Relic);
                }
                (2, 1) if toggle => {
                    s.lang = 1 - s.lang.min(1);
                    changed = true;
                }
                (2, 2) if toggle => {
                    s.show_hitbox = !s.show_hitbox;
                    changed = true;
                }
                (2, 3) if toggle => {
                    s.hints = !s.hints;
                    changed = true;
                }
                (2, 4) if inp.confirm && !from_pause => {
                    self.audio.play(Sfx::MenuOk);
                    return Screen::Prologue {
                        t: 0.0,
                        start: None,
                    };
                }
                (2, 5) if inp.confirm => {
                    if self.reset_armed > 0.0 {
                        let keep = self.save.clone();
                        let mut fresh = Save::default();
                        fresh.music = keep.music;
                        fresh.sfx = keep.sfx;
                        fresh.fullscreen = keep.fullscreen;
                        fresh.crt = keep.crt;
                        fresh.shake = keep.shake;
                        fresh.particles = keep.particles;
                        fresh.flashes = keep.flashes;
                        fresh.panels = keep.panels;
                        fresh.bg_anim = keep.bg_anim;
                        fresh.show_fps = keep.show_fps;
                        fresh.seen_intro = true;
                        fresh.lang = keep.lang;
                        self.save = fresh;
                        self.save.write();
                        self.reset_armed = 0.0;
                        self.audio.play(Sfx::GameOver);
                        self.toast("PROGRESSO APAGADO", "Um novo começo, piloto.", RED);
                    } else {
                        self.reset_armed = 3.0;
                        self.audio.play(Sfx::Warning);
                    }
                }
                _ => {}
            }
        }
        if changed {
            self.apply_settings();
            self.audio.play(Sfx::MenuMove);
        }
        if inp.back || (row == last && inp.confirm) {
            self.save.write();
            self.audio.play(Sfx::MenuOk);
            return if from_pause {
                Screen::Paused { sel: 1, tip: 0 }
            } else {
                Screen::Title { sel: 5 }
            };
        }
        Screen::Options {
            sel: tab * 100 + row,
            from_pause,
        }
    }

    // -----------------------------------------------------------------------
    // Desenho
    // -----------------------------------------------------------------------

    fn draw(&self) {
        match &self.screen {
            Screen::Playing => {
                if let Some(g) = &self.game {
                    g.draw(&self.sp, &self.art, self.save.shake, !self.panels);
                }
                if self.demo {
                    draw_rectangle(0.0, H - 80.0, W, 40.0, Color::new(0.0, 0.0, 0.0, 0.5));
                    if (self.time * 2.0) as i32 % 2 == 0 {
                        ui::text_c(
                            "DEMONSTRAÇÃO  -  APERTE QUALQUER TECLA",
                            W / 2.0,
                            H - 54.0,
                            16.0,
                            GOLD,
                        );
                    }
                }
            }
            Screen::Paused { sel, tip } => {
                if let Some(g) = &self.game {
                    g.draw(&self.sp, &self.art, false, !self.panels);
                }
                draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.05, 0.75));
                ui::text_outline(
                    "PAUSA",
                    W / 2.0,
                    H * 0.28,
                    52.0,
                    GOLD,
                    Color::from_rgba(40, 20, 0, 255),
                );
                let items: Vec<String> = ["CONTINUAR", "OPÇÕES", "ENCERRAR PARTIDA"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                ui::menu(&items, *sel, W / 2.0, H * 0.4, 40.0, 24.0, self.time);
                if self.save.hints {
                    ui::panel(30.0, H * 0.6, W - 60.0, 70.0, CYAN);
                    ui::text_c("DICA", W / 2.0, H * 0.6 + 22.0, 14.0, CYAN);
                    for (i, l) in ui::wrap(TIPS[*tip], 15.0, W - 90.0).iter().enumerate() {
                        ui::text_c(l, W / 2.0, H * 0.6 + 44.0 + i as f32 * 17.0, 15.0, WHITE);
                    }
                }
                if let Some(g) = &self.game
                    && !g.x.contracts.is_empty()
                    && !self.panels
                {
                    for (i, c) in g.x.contracts.iter().enumerate() {
                        let col = if c.done { GREEN } else { WHITE };
                        let txt = format!(
                            "{} {}  ({}/{})",
                            if c.done { "[OK]" } else { "[  ]" },
                            c.text(),
                            c.progress.min(c.target),
                            c.target
                        );
                        ui::text_c(&txt, W / 2.0, H * 0.12 + i as f32 * 18.0, 13.0, col);
                    }
                }
                if !self.save.equipped.is_empty() {
                    ui::text_c("CARTAS EQUIPADAS", W / 2.0, H * 0.78, 14.0, DIM);
                    for (i, &c) in self.save.equipped.iter().enumerate() {
                        ui::text_c(
                            CARDS[c].name,
                            W / 2.0,
                            H * 0.78 + 20.0 + i as f32 * 18.0,
                            15.0,
                            PINK,
                        );
                    }
                }
            }
            Screen::Sheet => self.draw_sheet(),
            Screen::Prologue { t, .. } => self.draw_prologue(*t),
            _ => {
                self.bg.draw(&self.sp, self.time);
                match &self.screen {
                    Screen::Title { sel } => self.draw_title(*sel),
                    Screen::ModeSelect { sel, diff, row } => {
                        self.draw_mode_select(*sel, *diff, *row)
                    }
                    Screen::Hangar { sel, skin, row } => self.draw_hangar(*sel, *skin, *row),
                    Screen::Summary {
                        t, gain, from_xp, ..
                    } => self.draw_summary(*t, *gain, *from_xp),
                    Screen::NameEntry {
                        name,
                        score,
                        sector,
                        ..
                    } => self.draw_name_entry(name, *score, *sector),
                    Screen::Scores { mode, highlight } => self.draw_scores(*mode, *highlight),
                    Screen::Album { sel } => self.draw_album(*sel),
                    Screen::Archives { tab, sel } => self.draw_archives(*tab, *sel),
                    Screen::Options { sel, .. } => self.draw_options(*sel),
                    Screen::HowTo { page } => self.draw_howto(*page),
                    _ => {}
                }
                self.fx.draw_world(Vec2::ZERO);
                self.fx.draw_texts();
            }
        }
        self.draw_toasts();
    }

    fn draw_toasts(&self) {
        if let Some(t) = self.toasts.first() {
            let slide = (t.t * 5.0).min(1.0) * ((t.dur - t.t) * 5.0).clamp(0.0, 1.0);
            let y = -70.0 + 80.0 * slide;
            let h = 58.0;
            draw_rectangle(16.0, y, W - 32.0, h, Color::new(0.04, 0.03, 0.12, 0.92));
            draw_rectangle_lines(16.0, y, W - 32.0, h, 2.0, t.color);
            draw_circle(40.0, y + h / 2.0, 12.0, t.color);
            ui::text_c(
                "!",
                40.0,
                y + h / 2.0 + 6.0,
                18.0,
                Color::from_rgba(20, 10, 30, 255),
            );
            ui::text(&t.title, 60.0, y + 24.0, 16.0, t.color);
            if let Some(l) = ui::wrap(&t.sub, 13.0, W - 100.0).first() {
                ui::text(l, 60.0, y + 44.0, 13.0, WHITE);
            }
        }
    }

    fn draw_title(&self, sel: usize) {
        for i in 0..6 {
            let x = W / 2.0 + (i as f32 - 2.5) * 52.0 + (self.time * 0.8).sin() * 20.0;
            let y = 62.0 + (self.time * 2.0 + i as f32).sin() * 4.0;
            let kind = [2usize, 7, 8, 9, 10, 2][i];
            let f = ((self.time * 6.0) as usize + i) % 2;
            self.sp.enemies[kind][f].draw(vec2(x, y), 3.0, 0.0, WHITE, false);
        }
        let party = self.logo_party > 0.0;
        let y = 168.0
            + if party {
                (self.time * 20.0).sin() * 8.0
            } else {
                0.0
            };
        ui::logo(y, if party { self.time * 6.0 } else { self.time });
        ui::text_c("um shoot 'em up de formações", W / 2.0, 200.0, 15.0, DIM);

        let p = vec2(W / 2.0, 254.0 + (self.time * 2.0).sin() * 5.0);
        let flame = 6.0 + (self.time * 40.0).sin().abs() * 7.0;
        draw_triangle(
            p + vec2(-4.0, 20.0),
            p + vec2(4.0, 20.0),
            p + vec2(0.0, 20.0 + flame),
            ORANGE,
        );
        let ship = self.save.ship.min(SHIPS.len() - 1);
        self.art.ships[ship][self.save.skin].draw(p, 3.0, 0.0, WHITE, false);

        let done = self.save.achs.iter().filter(|a| **a).count();
        let items: Vec<String> = self
            .title_visible()
            .iter()
            .map(|&i| match i {
                2 => format!("CARTAS {}/{}", self.save.unique_cards(), CARDS.len()),
                3 => format!("ARQUIVOS {}/{}", done, ACHS.len()),
                _ => TITLE_ITEMS[i].to_string(),
            })
            .collect();
        let spacing = if items.len() > 8 { 29.0 } else { 32.0 };
        ui::menu(&items, sel, W / 2.0, 322.0, spacing, 22.0, self.time);

        // patente
        let r = self.rank();
        progress::draw_insignia(r, 46.0, 620.0, 1.3);
        ui::text(RANKS[r], 72.0, 616.0, 16.0, GOLD);
        let (a, b) = (
            progress::xp_for(r),
            progress::xp_for((r + 1).min(RANKS.len() - 1)),
        );
        let frac = if b > a {
            (self.save.xp - a) as f32 / (b - a) as f32
        } else {
            1.0
        };
        draw_rectangle(72.0, 624.0, 180.0, 6.0, Color::new(0.2, 0.2, 0.3, 1.0));
        draw_rectangle(72.0, 624.0, 180.0 * frac.min(1.0), 6.0, GOLD);
        ui::text(&format!("{} XP", self.save.xp), 72.0, 644.0, 13.0, DIM);
        let best = self.save.best_for(0);
        ui::text_r("RECORDE", W - 20.0, 616.0, 13.0, DIM);
        ui::text_r(&format!("{:08}", best), W - 20.0, 638.0, 18.0, GOLD);
        if progress::season() == progress::Season::Natal {
            ui::text_c("FELIZ NATAL, PILOTO!", W / 2.0, 676.0, 14.0, GREEN);
        } else if progress::season() == progress::Season::Halloween {
            ui::text_c(
                "Os Besouros estão... diferentes hoje.",
                W / 2.0,
                676.0,
                14.0,
                Color::from_rgba(255, 140, 20, 255),
            );
        }
        ui::text(
            &format!("v{VERSION}"),
            8.0,
            H - 10.0,
            13.0,
            Color::new(0.4, 0.4, 0.5, 1.0),
        );
        ui::text_r(
            "SETAS · ENTER · ESC",
            W - 8.0,
            H - 10.0,
            13.0,
            Color::new(0.4, 0.4, 0.5, 1.0),
        );
    }

    fn draw_mode_select(&self, sel: usize, diff: usize, row: usize) {
        ui::text_outline(
            "MODO DE JOGO",
            W / 2.0,
            90.0,
            40.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let m = MODES[sel];
        let unlocked = self.mode_unlocked(m);
        ui::panel(
            30.0,
            120.0,
            W - 60.0,
            250.0,
            if row == 0 { GOLD } else { DIM },
        );
        let arrows = if row == 0 && (self.time * 3.0) as i32 % 2 == 0 {
            GOLD
        } else {
            DIM
        };
        ui::text("<", 46.0, 176.0, 30.0, arrows);
        ui::text_r(">", W - 46.0, 176.0, 30.0, arrows);
        ui::text_c(
            m.name(),
            W / 2.0,
            176.0,
            30.0,
            if unlocked { WHITE } else { DIM },
        );
        if unlocked {
            for (i, l) in ui::wrap(m.desc(), 16.0, W - 120.0).iter().enumerate() {
                ui::text_c(
                    l,
                    W / 2.0,
                    212.0 + i as f32 * 20.0,
                    16.0,
                    Color::new(0.85, 0.85, 1.0, 1.0),
                );
            }
            ui::text_c(
                &format!("RECORDE {:08}", self.save.best_for(m.tag())),
                W / 2.0,
                290.0,
                16.0,
                GOLD,
            );
            match m {
                Mode::Diario => {
                    let d = progress::daily_for(progress::today_number());
                    ui::text_c(
                        &format!("HOJE: {}", d.name()),
                        W / 2.0,
                        320.0,
                        17.0,
                        Color::from_rgba(255, 150, 60, 255),
                    );
                    ui::text_c(d.desc(), W / 2.0, 342.0, 14.0, WHITE);
                }
                Mode::Sobrevivencia => ui::text_c(
                    &format!("MELHOR ONDA: {}", self.save.best_survival),
                    W / 2.0,
                    320.0,
                    15.0,
                    CYAN,
                ),
                Mode::Expedicao => ui::text_c(
                    &format!("MAIS LONGE: SETOR {}", self.save.expedition_best),
                    W / 2.0,
                    320.0,
                    15.0,
                    CYAN,
                ),
                Mode::Chefoes if self.save.best_rush > 0.0 => {
                    let t = self.save.best_rush;
                    ui::text_c(
                        &format!(
                            "MELHOR TEMPO {:02}:{:02}",
                            (t / 60.0) as u32,
                            (t % 60.0) as u32
                        ),
                        W / 2.0,
                        320.0,
                        15.0,
                        CYAN,
                    )
                }
                _ => {}
            }
        } else {
            ui::text_c("BLOQUEADO", W / 2.0, 230.0, 22.0, RED);
            ui::text_c(
                &format!("Alcance a patente {}", RANKS[m.unlock_rank()]),
                W / 2.0,
                260.0,
                16.0,
                DIM,
            );
        }
        // dificuldade
        let d = DIFFS[diff];
        let ok = self.diff_unlocked(d);
        ui::panel(
            30.0,
            390.0,
            W - 60.0,
            160.0,
            if row == 1 { GOLD } else { DIM },
        );
        ui::text_c("DIFICULDADE", W / 2.0, 418.0, 15.0, DIM);
        let arrows = if row == 1 && (self.time * 3.0) as i32 % 2 == 0 {
            GOLD
        } else {
            DIM
        };
        ui::text("<", 46.0, 456.0, 30.0, arrows);
        ui::text_r(">", W - 46.0, 456.0, 30.0, arrows);
        ui::text_c(
            d.name(),
            W / 2.0,
            456.0,
            30.0,
            if ok { d.color() } else { DIM },
        );
        if ok {
            let info = format!(
                "vidas {:+}  ·  tiros x{:.2}  ·  XP x{:.1}  ·  continues {}",
                d.lives(),
                d.fire_rate(),
                d.xp_mul(),
                d.continues()
            );
            ui::text_c(&info, W / 2.0, 494.0, 13.0, WHITE);
            if d == Difficulty::Dificil {
                ui::text_c(
                    "Vença aqui sem continues para ver o final verdadeiro.",
                    W / 2.0,
                    520.0,
                    13.0,
                    GOLD,
                );
            }
        } else if d == Difficulty::Pesadelo {
            ui::text_c(
                "Vença a campanha 3 vezes para liberar.",
                W / 2.0,
                494.0,
                14.0,
                RED,
            );
        } else {
            ui::text_c("Alcance a patente SARGENTO.", W / 2.0, 494.0, 14.0, RED);
        }
        // resumo da nave
        let s = &SHIPS[self.save.ship];
        self.art.ships[self.save.ship][self.save.skin].draw(
            vec2(60.0, 600.0),
            3.0,
            0.0,
            WHITE,
            false,
        );
        ui::text(&format!("NAVE: {}", s.name), 96.0, 594.0, 18.0, CYAN);
        ui::text(
            &format!("HABILIDADE: {}", s.ab_name),
            96.0,
            616.0,
            14.0,
            DIM,
        );
        let eq = if self.save.equipped.is_empty() {
            "nenhuma carta equipada".to_string()
        } else {
            self.save
                .equipped
                .iter()
                .map(|&c| CARDS[c].name)
                .collect::<Vec<_>>()
                .join(", ")
        };
        for (i, l) in ui::wrap(&eq, 13.0, W - 120.0).iter().take(2).enumerate() {
            ui::text(l, 96.0, 638.0 + i as f32 * 16.0, 13.0, PINK);
        }
        ui::text_c(
            "SETAS escolhem  ·  ENTER decola  ·  ESC volta",
            W / 2.0,
            H - 14.0,
            13.0,
            DIM,
        );
    }

    fn draw_hangar(&self, sel: usize, skin: usize, row: usize) {
        ui::text_outline(
            "HANGAR",
            W / 2.0,
            70.0,
            44.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let rank = self.rank();
        let s = &SHIPS[sel];
        let unlocked = ships::ship_unlocked(sel, rank, &self.save.achs);
        // plataforma
        let c = vec2(W / 2.0, 200.0);
        draw_ellipse(
            c.x,
            c.y + 56.0,
            90.0,
            18.0,
            0.0,
            Color::new(0.3, 0.6, 1.0, 0.15),
        );
        draw_ellipse_lines(
            c.x,
            c.y + 56.0,
            90.0,
            18.0,
            0.0,
            2.0,
            Color::new(0.4, 0.8, 1.0, 0.5),
        );
        let bob = (self.time * 2.0).sin() * 6.0;
        let tint = if unlocked {
            if skin == 6 {
                ui::hsl(self.time * 0.8, 0.8, 0.75)
            } else {
                WHITE
            }
        } else {
            Color::new(0.0, 0.0, 0.0, 1.0)
        };
        self.art.ships[sel][skin].draw(c + vec2(0.0, bob), 6.0, 0.0, tint, false);
        if !unlocked {
            ui::text_c("?", c.x, c.y + 14.0, 60.0, DIM);
        }
        let arrows = if row == 0 && (self.time * 3.0) as i32 % 2 == 0 {
            GOLD
        } else {
            DIM
        };
        ui::text("<", 40.0, 210.0, 36.0, arrows);
        ui::text_r(">", W - 40.0, 210.0, 36.0, arrows);
        ui::text_c(
            &format!(
                "{}  ({}/{})",
                if unlocked { s.name } else { "???" },
                sel + 1,
                SHIPS.len()
            ),
            W / 2.0,
            300.0,
            26.0,
            if unlocked { WHITE } else { DIM },
        );
        if unlocked {
            ui::text_c(
                s.desc,
                W / 2.0,
                324.0,
                14.0,
                Color::new(0.85, 0.85, 1.0, 1.0),
            );
            let stats = [
                ("VELOCIDADE", s.speed / 1.3),
                ("CADÊNCIA", s.fire / 1.2),
                ("DANO", s.dmg / 1.7),
                ("BLINDAGEM", (3 + s.lives) as f32 / 5.0),
                ("TAMANHO DO ALVO", 1.0 - (s.hit_r - 3.0) / 3.5),
            ];
            for (i, (label, v)) in stats.iter().enumerate() {
                let y = 352.0 + i as f32 * 24.0;
                ui::text(label, 60.0, y + 10.0, 14.0, DIM);
                draw_rectangle(210.0, y, 200.0, 10.0, Color::new(0.2, 0.2, 0.3, 1.0));
                draw_rectangle(210.0, y, 200.0 * v.clamp(0.05, 1.0), 10.0, CYAN);
            }
            ui::panel(40.0, 480.0, W - 80.0, 74.0, PINK);
            ui::text(
                &format!("HABILIDADE: {}", s.ab_name),
                56.0,
                504.0,
                16.0,
                PINK,
            );
            for (i, l) in ui::wrap(s.ab_desc, 14.0, W - 120.0)
                .iter()
                .take(2)
                .enumerate()
            {
                ui::text(l, 56.0, 526.0 + i as f32 * 16.0, 14.0, WHITE);
            }
        } else {
            ui::text_c("BLOQUEADA", W / 2.0, 360.0, 22.0, RED);
            ui::text_c(&ships::unlock_hint(sel), W / 2.0, 390.0, 16.0, DIM);
        }
        // pinturas
        let srank = progress::skin_rank(skin);
        let skin_ok = self.skin_ok(skin);
        let blink = (self.time * 3.0) as i32 % 2 == 0;
        let arrows = if row == 1 && blink { GOLD } else { DIM };
        ui::text("<", 60.0, 584.0, 20.0, arrows);
        ui::text_r(">", W - 60.0, 584.0, 20.0, arrows);
        ui::text_c(
            &format!("PINTURA: {}", progress::SKIN_NAMES[skin]),
            W / 2.0,
            584.0,
            17.0,
            if skin_ok { GOLD } else { DIM },
        );
        if !skin_ok {
            ui::text_c(
                &format!("Patente {}", RANKS[srank]),
                W / 2.0,
                600.0,
                12.0,
                RED,
            );
        }
        for k in 0..art::SKINS {
            let x = W / 2.0 + (k as f32 - 3.0) * 30.0;
            let c = if self.skin_ok(k) {
                WHITE
            } else {
                Color::new(0.0, 0.0, 0.0, 1.0)
            };
            self.art.ships[sel][k].draw(vec2(x, 624.0), 1.3, 0.0, c, false);
            if k == skin {
                draw_rectangle_lines(x - 12.0, 612.0, 24.0, 24.0, 1.0, GOLD);
            }
        }
        // rastro
        let t = self.hangar_trail;
        let t_ok = progress::trail_unlocked(t, &self.save);
        let arrows = if row == 2 && blink { GOLD } else { DIM };
        ui::text("<", 60.0, 664.0, 20.0, arrows);
        ui::text_r(">", W - 60.0, 664.0, 20.0, arrows);
        let label = if self.save.trail == t {
            format!("RASTRO: {} (EM USO)", progress::TRAILS[t])
        } else {
            format!("RASTRO: {}", progress::TRAILS[t])
        };
        ui::text_c(&label, W / 2.0, 664.0, 17.0, if t_ok { CYAN } else { DIM });
        if !t_ok {
            if let Some(a) = progress::trail_ach(t) {
                let hint = progress::HONORS
                    .iter()
                    .find(|h| h.ach == a)
                    .map(|h| h.shown_name(&self.save.achs))
                    .unwrap_or("???");
                ui::text_c(hint, W / 2.0, 680.0, 12.0, RED);
            }
        } else if t > 0 {
            for k in 0..12 {
                let age = k as f32 * 0.04;
                let col = game::v4::trail_color(t, self.time, age);
                let y = c.y + 40.0 + bob + k as f32 * 7.0;
                let a = 1.0 - k as f32 / 12.0;
                draw_circle(
                    c.x + (self.time * 9.0 + k as f32).sin() * 1.5,
                    y,
                    5.0 * a + 1.0,
                    Color::new(col.r, col.g, col.b, a * 0.8),
                );
            }
        }
        let current = if row == 2 {
            "ENTER equipa o rastro"
        } else if self.save.ship == sel && self.save.skin == skin {
            "EQUIPADA"
        } else {
            "ENTER equipa"
        };
        ui::text_c(
            &format!("{current}  ·  CIMA/BAIXO: nave, pintura, rastro"),
            W / 2.0,
            H - 12.0,
            12.0,
            DIM,
        );
    }

    fn draw_summary(&self, t: f32, gain: u64, from_xp: u64) {
        ui::text_outline(
            "RELATÓRIO DE VOO",
            W / 2.0,
            110.0,
            38.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let shown = from_xp + ((t - 0.8).clamp(0.0, 2.0) / 2.0 * gain as f32) as u64;
        let r = progress::rank_of(shown);
        progress::draw_insignia(r, W / 2.0, 220.0, 3.0);
        ui::text_c(RANKS[r], W / 2.0, 300.0, 28.0, WHITE);
        let (a, b) = (
            progress::xp_for(r),
            progress::xp_for((r + 1).min(RANKS.len() - 1)),
        );
        let frac = if b > a {
            (shown - a) as f32 / (b - a) as f32
        } else {
            1.0
        };
        draw_rectangle(60.0, 330.0, W - 120.0, 16.0, Color::new(0.2, 0.2, 0.3, 1.0));
        draw_rectangle(60.0, 330.0, (W - 120.0) * frac.min(1.0), 16.0, GOLD);
        draw_rectangle_lines(60.0, 330.0, W - 120.0, 16.0, 1.0, WHITE);
        ui::text_c(&format!("+{gain} XP"), W / 2.0, 380.0, 26.0, CYAN);
        if r + 1 < RANKS.len() {
            ui::text_c(
                &format!("Próxima: {} ({} XP)", RANKS[r + 1], b.saturating_sub(shown)),
                W / 2.0,
                410.0,
                14.0,
                DIM,
            );
        }
        let from_r = progress::rank_of(from_xp);
        if r > from_r {
            ui::text_c(
                "PROMOVIDO!",
                W / 2.0,
                460.0,
                30.0,
                ui::hsl(self.time, 0.9, 0.65),
            );
            for (i, rr) in (from_r + 1..=r).enumerate() {
                let txt = progress::unlock_text(rr);
                if !txt.is_empty() {
                    ui::text_c(
                        &format!("NOVO: {txt}"),
                        W / 2.0,
                        496.0 + i as f32 * 22.0,
                        16.0,
                        GREEN,
                    );
                }
            }
        }
        if t > 3.0 && (t * 2.0) as i32 % 2 == 0 {
            ui::text_c("ENTER continua", W / 2.0, H - 40.0, 16.0, DIM);
        }
    }

    fn draw_name_entry(&self, name: &str, score: u32, sector: u32) {
        ui::text_outline(
            "NOVO RECORDE!",
            W / 2.0,
            170.0,
            46.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        ui::text_c(
            &format!("{score} PONTOS  ·  {sector}"),
            W / 2.0,
            214.0,
            20.0,
            WHITE,
        );
        ui::text_c("Digite seu nome:", W / 2.0, 300.0, 20.0, DIM);
        ui::panel(70.0, 320.0, W - 140.0, 64.0, CYAN);
        let cursor = if (self.time * 3.0) as i32 % 2 == 0 {
            "_"
        } else {
            " "
        };
        ui::text_c(&format!("{name}{cursor}"), W / 2.0, 364.0, 34.0, WHITE);
        ui::text_c(
            "ENTER confirma  ·  BACKSPACE apaga",
            W / 2.0,
            420.0,
            15.0,
            DIM,
        );
    }

    fn draw_scores(&self, mode: usize, highlight: Option<usize>) {
        ui::text_outline(
            "RECORDES",
            W / 2.0,
            80.0,
            44.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let m = MODES[mode];
        ui::text_c(&format!("<  {}  >", m.name()), W / 2.0, 118.0, 20.0, CYAN);
        ui::panel(24.0, 136.0, W - 48.0, 430.0, GOLD);
        ui::text("#", 40.0, 172.0, 15.0, DIM);
        ui::text("NOME", 70.0, 172.0, 15.0, DIM);
        ui::text("NAVE", 220.0, 172.0, 15.0, DIM);
        ui::text_r(
            if matches!(m, Mode::Campanha | Mode::Diario | Mode::Expedicao) {
                "SETOR"
            } else {
                "ONDA"
            },
            336.0,
            172.0,
            15.0,
            DIM,
        );
        ui::text_r("PONTOS", W - 40.0, 172.0, 15.0, DIM);
        let legend = self.rank() >= 19;
        for (i, e) in self.save.scores_for(m.tag()).iter().enumerate() {
            let y = 208.0 + i as f32 * 34.0;
            let hl = highlight == Some(i);
            let mine = legend && e.name == self.save.last_name;
            let c = if hl || mine {
                ui::hsl(self.time * 0.5, 0.9, 0.65)
            } else if i == 0 {
                GOLD
            } else {
                WHITE
            };
            if hl {
                draw_rectangle(
                    30.0,
                    y - 22.0,
                    W - 60.0,
                    30.0,
                    Color::new(1.0, 1.0, 1.0, 0.08),
                );
            }
            ui::text(&format!("{}", i + 1), 40.0, y, 19.0, c);
            ui::text(&e.name, 70.0, y, 19.0, c);
            self.art.ships[(e.ship as usize).min(SHIPS.len() - 1)][0].draw(
                vec2(234.0, y - 6.0),
                1.3,
                0.0,
                WHITE,
                false,
            );
            ui::text_r(&e.sector.to_string(), 336.0, y, 19.0, c);
            ui::text_r(&format!("{:08}", e.score), W - 40.0, y, 19.0, c);
        }
        ui::text_c(
            "ESQ/DIR troca o modo  ·  ENTER volta",
            W / 2.0,
            600.0,
            14.0,
            DIM,
        );
    }

    fn draw_album(&self, sel: usize) {
        ui::text_outline(
            "COLEÇÃO",
            W / 2.0,
            50.0,
            38.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let unique = self.save.unique_cards();
        ui::text_c(
            &format!("{} de {} cartas", unique, CARDS.len()),
            W / 2.0,
            74.0,
            14.0,
            DIM,
        );
        let scale = 0.5;
        let cw = ui::CARD_W * scale;
        let chh = ui::CARD_H * scale;
        let gap = 7.0;
        let cols = 6;
        let x0 = (W - (cols as f32 * cw + (cols - 1) as f32 * gap)) / 2.0;
        let y0 = 86.0;
        let page = sel / 30;
        let pages = CARDS.len().div_ceil(30);
        for i in page * 30..(page * 30 + 30).min(CARDS.len()) {
            let j = i - page * 30;
            let x = x0 + (j % cols) as f32 * (cw + gap);
            let y = y0 + (j / cols) as f32 * (chh + gap);
            let owned = self.save.cards[i] > 0;
            ui::card_face(
                &self.sp,
                i,
                x,
                y,
                scale,
                owned,
                if i == sel { 1.0 } else { 0.0 },
            );
            if self.save.equipped.contains(&i) {
                draw_rectangle_lines(x - 3.0, y - 3.0, cw + 6.0, chh + 6.0, 2.0, GREEN);
            }
            if i == sel {
                let pulse = (self.time * 6.0).sin() * 0.5 + 0.5;
                draw_rectangle_lines(
                    x - 4.0,
                    y - 4.0,
                    cw + 8.0,
                    chh + 8.0,
                    2.0,
                    Color::new(1.0, 0.85, 0.3, 0.5 + pulse * 0.5),
                );
            }
        }
        let rows = 5;
        ui::text_r(
            &format!("PÁGINA {}/{}", page + 1, pages),
            W - 14.0,
            y0 + rows as f32 * (chh + gap) - 2.0,
            12.0,
            DIM,
        );
        let dy = y0 + rows as f32 * (chh + gap) + 4.0;
        ui::panel(12.0, dy, W - 24.0, 128.0, CARDS[sel].rarity.color());
        let owned = self.save.cards[sel] > 0;
        ui::card_face(&self.sp, sel, 22.0, dy + 8.0, 0.98, owned, 0.0);
        let tx = 22.0 + ui::CARD_W * 0.98 + 14.0;
        let c = &CARDS[sel];
        if owned {
            ui::text(c.name, tx, dy + 28.0, 19.0, WHITE);
            ui::text(
                &format!(
                    "{}{}",
                    c.rarity.name(),
                    if c.secret { "  ·  SECRETA" } else { "" }
                ),
                tx,
                dy + 48.0,
                14.0,
                c.rarity.color(),
            );
            for (i, l) in ui::wrap(c.desc, 15.0, W - tx - 24.0).iter().enumerate() {
                ui::text(
                    l,
                    tx,
                    dy + 74.0 + i as f32 * 17.0,
                    15.0,
                    Color::new(0.9, 0.9, 1.0, 1.0),
                );
            }
            ui::text(
                &format!("cópias: {}", self.save.cards[sel]),
                tx,
                dy + 120.0,
                12.0,
                DIM,
            );
            let eq = self.save.equipped.contains(&sel);
            ui::text_r(
                if eq { "EQUIPADA" } else { "ENTER equipa" },
                W - 24.0,
                dy + 120.0,
                13.0,
                if eq { GREEN } else { DIM },
            );
        } else {
            ui::text("???", tx, dy + 30.0, 22.0, DIM);
            ui::text(c.rarity.name(), tx, dy + 52.0, 14.0, c.rarity.color());
            let hint = "Ainda não encontrada. Continue jogando.";
            for (i, l) in ui::wrap(hint, 14.0, W - tx - 24.0).iter().enumerate() {
                ui::text(l, tx, dy + 78.0 + i as f32 * 16.0, 14.0, DIM);
            }
        }
        let slots = cards::slots_for(unique);
        let sy = dy + 150.0;
        ui::text("ESPAÇOS:", 16.0, sy, 14.0, DIM);
        for s in 0..4 {
            let x = 90.0 + s as f32 * 96.0;
            let (label, col) = if s < slots {
                match self.save.equipped.get(s) {
                    Some(&ci) => (CARDS[ci].name.to_string(), PINK),
                    None => ("vazio".to_string(), DIM),
                }
            } else {
                (
                    ["", "8 cartas", "16 cartas", "30 cartas"][s].to_string(),
                    Color::new(0.35, 0.35, 0.45, 1.0),
                )
            };
            draw_rectangle_lines(x - 4.0, sy - 14.0, 90.0, 20.0, 1.0, col);
            let label = if ui::width(&label, 12.0) > 84.0 {
                format!("{}..", label.chars().take(10).collect::<String>())
            } else {
                label
            };
            ui::text(&label, x, sy, 12.0, col);
        }
    }

    fn draw_archives(&self, tab: usize, sel: usize) {
        let tabs = [
            "CONQUISTAS",
            "BESTIÁRIO",
            "NÚMEROS",
            "PATENTES",
            "REGISTROS",
            "HONRAS",
        ];
        ui::text_outline(
            "ARQUIVOS",
            W / 2.0,
            54.0,
            38.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let mut fs = 13.0;
        let mut total = 0.0;
        while fs > 9.0 {
            total = tabs.iter().map(|t| ui::width(t, fs) + 10.0).sum::<f32>()
                + 3.0 * (tabs.len() - 1) as f32;
            if total <= W - 12.0 {
                break;
            }
            fs -= 1.0;
        }
        let mut x = (W - total) / 2.0;
        for (i, t) in tabs.iter().enumerate() {
            let w = ui::width(t, fs) + 10.0;
            if i == tab {
                draw_rectangle(x, 70.0, w, 24.0, Color::new(0.3, 0.25, 0.05, 0.8));
            }
            draw_rectangle_lines(x, 70.0, w, 24.0, 1.0, if i == tab { GOLD } else { DIM });
            ui::text(t, x + 5.0, 87.0, fs, if i == tab { GOLD } else { DIM });
            x += w + 3.0;
        }
        if tab == 5 {
            self.draw_honors(sel);
            return;
        }
        let top = 110.0;
        match tab {
            0 => {
                let per = 13;
                let start = sel
                    .saturating_sub(per / 2)
                    .min(ACHS.len().saturating_sub(per));
                for (row, i) in (start..(start + per).min(ACHS.len())).enumerate() {
                    let a = &ACHS[i];
                    let got = self.save.achs[i];
                    let y = top + row as f32 * 40.0;
                    if i == sel {
                        draw_rectangle(
                            14.0,
                            y - 4.0,
                            W - 28.0,
                            38.0,
                            Color::new(0.3, 0.25, 0.05, 0.5),
                        );
                    }
                    let c = if got {
                        GOLD
                    } else {
                        Color::new(0.3, 0.3, 0.4, 1.0)
                    };
                    draw_poly(32.0, y + 14.0, 6, 11.0, 0.0, c);
                    ui::text_c(
                        if got { "*" } else { "?" },
                        32.0,
                        y + 20.0,
                        16.0,
                        Color::from_rgba(20, 10, 30, 255),
                    );
                    let (name, desc) = if got || !a.hidden {
                        (a.name, a.desc)
                    } else {
                        ("???", "Conquista secreta.")
                    };
                    ui::text(name, 52.0, y + 12.0, 16.0, if got { WHITE } else { DIM });
                    ui::text(desc, 52.0, y + 29.0, 12.0, DIM);
                    ui::text_r(
                        &format!("{} XP", a.xp),
                        W - 22.0,
                        y + 12.0,
                        12.0,
                        if got { CYAN } else { DIM },
                    );
                }
                let done = self.save.achs.iter().filter(|a| **a).count();
                ui::text_c(
                    &format!("{done} de {} conquistas", ACHS.len()),
                    W / 2.0,
                    H - 34.0,
                    15.0,
                    GOLD,
                );
            }
            1 => {
                for (i, l) in BESTIARY.iter().enumerate() {
                    let y = top + i as f32 * 21.0;
                    let seen = self.save.seen[i];
                    let c = if i == sel {
                        GOLD
                    } else if seen {
                        WHITE
                    } else {
                        DIM
                    };
                    let boss = i >= progress::LORE_BOSS_START;
                    ui::text(
                        if seen { l.name } else { "???" },
                        22.0,
                        y + 14.0,
                        14.0,
                        if boss && seen && i != sel { PINK } else { c },
                    );
                }
                let l = &BESTIARY[sel];
                let seen = self.save.seen[sel];
                let px = 250.0;
                ui::panel(
                    px - 10.0,
                    top - 4.0,
                    W - px,
                    460.0,
                    if seen { CYAN } else { DIM },
                );
                let cx = px - 10.0 + (W - px) / 2.0;
                let tint = if seen {
                    WHITE
                } else {
                    Color::new(0.0, 0.0, 0.0, 1.0)
                };
                let f = ((self.time * 5.0) as usize) % 2;
                let sprite_pos = vec2(cx, top + 90.0);
                if sel < progress::LORE_BOSS_START {
                    self.sp.enemies[sel][f].draw(sprite_pos, 5.0, 0.0, tint, false);
                } else if (progress::LORE_V3..progress::LORE_TWINS).contains(&sel) {
                    self.sp.enemies[progress::LORE_BOSS_START + sel - progress::LORE_V3][f]
                        .draw(sprite_pos, 5.0, 0.0, tint, false);
                } else if sel >= progress::LORE_V5 {
                    self.sp.enemies[progress::KIND_V5 + sel - progress::LORE_V5][f]
                        .draw(sprite_pos, 5.0, 0.0, tint, false);
                } else if sel == progress::LORE_TWINS {
                    self.sp.queen[f].draw(
                        sprite_pos - vec2(30.0, 0.0),
                        2.4,
                        0.0,
                        Color::new(0.55 * tint.r, tint.g, 0.9 * tint.b, 1.0),
                        false,
                    );
                    self.sp.queen[1 - f].draw(
                        sprite_pos + vec2(30.0, 0.0),
                        2.4,
                        0.0,
                        Color::new(tint.r, 0.6 * tint.g, 0.85 * tint.b, 1.0),
                        false,
                    );
                } else if sel == progress::LORE_COMET {
                    for k in 0..8 {
                        let q = sprite_pos + vec2(-k as f32 * 9.0, -k as f32 * 7.0);
                        draw_circle(
                            q.x,
                            q.y,
                            26.0 - k as f32 * 2.5,
                            Color::new(0.5 * tint.r, 0.85 * tint.g, tint.b, 0.12),
                        );
                    }
                    draw_poly(
                        sprite_pos.x,
                        sprite_pos.y,
                        9,
                        26.0,
                        0.0,
                        Color::new(0.42 * tint.r, 0.5 * tint.g, 0.62 * tint.b, 1.0),
                    );
                    draw_poly(
                        sprite_pos.x - 3.0,
                        sprite_pos.y - 3.0,
                        9,
                        19.0,
                        0.0,
                        Color::new(0.62 * tint.r, 0.72 * tint.g, 0.85 * tint.b, 1.0),
                    );
                } else if sel == progress::LORE_DEVOURER {
                    draw_circle(
                        sprite_pos.x,
                        sprite_pos.y,
                        44.0,
                        Color::new(0.05, 0.0, 0.07, 1.0),
                    );
                    draw_circle_lines(
                        sprite_pos.x,
                        sprite_pos.y,
                        44.0,
                        3.0,
                        Color::new(0.7, 0.1, 0.4, 1.0),
                    );
                    if seen {
                        draw_ellipse(sprite_pos.x, sprite_pos.y - 8.0, 18.0, 10.0, 0.0, WHITE);
                        draw_circle(sprite_pos.x, sprite_pos.y - 8.0, 6.0, RED);
                    }
                } else {
                    match sel - progress::LORE_BOSS_START {
                        0 | 5 => self.sp.queen[f].draw(sprite_pos, 3.0, 0.0, tint, false),
                        1 => self.sp.warship.draw(sprite_pos, 3.0, 0.0, tint, false),
                        2 => self
                            .art
                            .serpent_head
                            .draw(sprite_pos, 4.0, 0.0, tint, false),
                        3 => self.art.spider.draw(sprite_pos, 3.0, 0.0, tint, false),
                        _ => self.art.eye_open.draw(sprite_pos, 3.0, 0.0, tint, false),
                    }
                }
                ui::text_c(
                    if seen { l.name } else { "???" },
                    cx,
                    top + 190.0,
                    17.0,
                    if seen { GOLD } else { DIM },
                );
                if seen {
                    for (i, line) in ui::wrap(l.text, 14.0, W - px - 30.0).iter().enumerate() {
                        ui::text_c(line, cx, top + 220.0 + i as f32 * 18.0, 14.0, WHITE);
                    }
                    let kind = if sel < progress::LORE_BOSS_START {
                        Some(sel)
                    } else if (progress::LORE_V3..progress::LORE_TWINS).contains(&sel) {
                        Some(progress::LORE_BOSS_START + sel - progress::LORE_V3)
                    } else if sel >= progress::LORE_V5 {
                        Some(progress::KIND_V5 + sel - progress::LORE_V5)
                    } else {
                        None
                    };
                    if let Some(k) = kind {
                        ui::text_c(
                            &format!("abatidos: {}", self.save.kills_kind[k]),
                            cx,
                            top + 420.0,
                            13.0,
                            DIM,
                        );
                    }
                } else {
                    ui::text_c("Abata um para registrar.", cx, top + 220.0, 14.0, DIM);
                }
                let n = self.save.seen.iter().filter(|s| **s).count();
                ui::text_c(
                    &format!("{n} de {} registros", BESTIARY.len()),
                    W / 2.0,
                    H - 34.0,
                    15.0,
                    GOLD,
                );
            }
            2 => {
                let s = &self.save;
                let t = s.play_time as u64;
                let fav = s
                    .weapon_kills
                    .iter()
                    .enumerate()
                    .filter(|(_, k)| **k > 0)
                    .max_by_key(|(_, k)| **k)
                    .map(|(i, _)| WEAPONS[i].name())
                    .unwrap_or("-");
                let lines = [
                    (
                        "Tempo de jogo",
                        format!("{}h {:02}min", t / 3600, (t / 60) % 60),
                    ),
                    ("Partidas", s.runs.to_string()),
                    ("Inimigos abatidos", s.kills.to_string()),
                    ("Chefes derrotados", s.bosses.to_string()),
                    ("Vitórias", s.victories.to_string()),
                    ("Melhor setor", s.best_sector.to_string()),
                    ("Maior combo", s.best_combo.to_string()),
                    ("Raspões", s.grazes.to_string()),
                    ("Bombas usadas", s.bombs_used.to_string()),
                    ("Naves perdidas", s.deaths.to_string()),
                    ("Cristais coletados", s.crystals_total.to_string()),
                    ("Arma favorita", fav.to_string()),
                    (
                        "Armas dominadas",
                        format!(
                            "{}/{}",
                            s.mastery.iter().filter(|m| **m).count(),
                            WEAPONS.len()
                        ),
                    ),
                    ("Mercadores poupados", s.merchants_spared.to_string()),
                    ("Melhor onda (sobrevivência)", s.best_survival.to_string()),
                    ("Desafios diários vencidos", s.daily_done.to_string()),
                    (
                        "Mais longe na Expedição",
                        format!("setor {}", s.expedition_best),
                    ),
                    (
                        "Minigames vencidos",
                        format!("{} ({} perfeitos)", s.minigames_won, s.perfects),
                    ),
                    (
                        "Maldições exorcizadas / pegas",
                        format!("{} / {}", s.curses_shot, s.curses_taken),
                    ),
                    (
                        "Palavras STARDUST / Naves-Mãe",
                        format!("{} / {}", s.words, s.ufos),
                    ),
                    ("Experiência total", format!("{} XP", s.xp)),
                ];
                for (i, (k, v)) in lines.iter().enumerate() {
                    let y = top + 10.0 + i as f32 * 23.5;
                    ui::text(k, 36.0, y, 14.0, DIM);
                    ui::text_r(v, W - 36.0, y, 15.0, WHITE);
                }
            }
            4 => {
                for (i, (title, _)) in progress::LOGS.iter().enumerate() {
                    let y = top + 14.0 + i as f32 * 26.0;
                    let found = self.save.logs[i];
                    if i == sel {
                        draw_rectangle(
                            14.0,
                            y - 18.0,
                            W - 28.0,
                            24.0,
                            Color::new(0.1, 0.3, 0.3, 0.6),
                        );
                    }
                    ui::text(
                        if found { title } else { "REGISTRO ???" },
                        26.0,
                        y,
                        15.0,
                        if found {
                            Color::from_rgba(120, 255, 220, 255)
                        } else {
                            DIM
                        },
                    );
                }
                let py = top + 14.0 + 12.0 * 26.0 + 6.0;
                ui::panel(
                    14.0,
                    py,
                    W - 28.0,
                    120.0,
                    Color::from_rgba(120, 255, 220, 255),
                );
                if self.save.logs[sel] {
                    for (i, l) in ui::wrap(progress::LOGS[sel].1, 15.0, W - 60.0)
                        .iter()
                        .enumerate()
                    {
                        ui::text(l, 30.0, py + 30.0 + i as f32 * 19.0, 15.0, WHITE);
                    }
                } else {
                    ui::text(
                        "Registros caem dos chefes da campanha e de",
                        30.0,
                        py + 30.0,
                        14.0,
                        DIM,
                    );
                    ui::text(
                        "encontros raros. Cada um conta um pedaço",
                        30.0,
                        py + 50.0,
                        14.0,
                        DIM,
                    );
                    ui::text("da verdade sobre o Enxame.", 30.0, py + 70.0, 14.0, DIM);
                }
                let n = self.save.logs.iter().filter(|l| **l).count();
                ui::text_c(
                    &format!("{n} de {} registros", progress::LOGS.len()),
                    W / 2.0,
                    H - 34.0,
                    15.0,
                    Color::from_rgba(120, 255, 220, 255),
                );
            }
            _ => {
                let r = self.rank();
                let per = 13;
                let start = sel.saturating_sub(per / 2).min(RANKS.len() - per);
                for (row, i) in (start..start + per).enumerate() {
                    let y = top + row as f32 * 40.0;
                    if i == sel {
                        draw_rectangle(
                            14.0,
                            y - 4.0,
                            W - 28.0,
                            38.0,
                            Color::new(0.3, 0.25, 0.05, 0.5),
                        );
                    }
                    progress::draw_insignia(i, 34.0, y + 15.0, 0.9);
                    let c = if i == r {
                        GOLD
                    } else if i < r {
                        WHITE
                    } else {
                        DIM
                    };
                    ui::text(RANKS[i], 58.0, y + 12.0, 16.0, c);
                    let u = progress::unlock_text(i);
                    ui::text(
                        if u.is_empty() { "-" } else { u },
                        58.0,
                        y + 29.0,
                        12.0,
                        if i <= r { CYAN } else { DIM },
                    );
                    ui::text_r(
                        &format!("{} XP", progress::xp_for(i)),
                        W - 22.0,
                        y + 12.0,
                        12.0,
                        DIM,
                    );
                }
            }
        }
        ui::text_c(
            "ESQ/DIR abas  ·  CIMA/BAIXO navega  ·  ESC volta",
            W / 2.0,
            H - 12.0,
            13.0,
            DIM,
        );
    }

    fn draw_options(&self, sel: usize) {
        let tab = (sel / 100).min(3);
        let row = sel % 100;
        ui::text_outline_fit(
            "CONFIGURAÇÕES",
            W / 2.0,
            70.0,
            36.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
            W - 30.0,
        );
        let tabs = ["GRÁFICOS", "ÁUDIO", "JOGO", "CONTROLES"];
        let tw = (W - 40.0) / 4.0;
        for (i, t) in tabs.iter().enumerate() {
            let x = 20.0 + i as f32 * tw;
            let on = i == tab;
            draw_rectangle(
                x + 2.0,
                96.0,
                tw - 4.0,
                30.0,
                if on {
                    Color::new(0.35, 0.25, 0.05, 0.8)
                } else {
                    Color::new(0.1, 0.1, 0.18, 0.7)
                },
            );
            if on {
                draw_rectangle_lines(
                    x + 2.0,
                    96.0,
                    tw - 4.0,
                    30.0,
                    if row == 0 { 2.5 } else { 1.5 },
                    GOLD,
                );
            }
            ui::text_c(t, x + tw / 2.0, 116.0, 13.0, if on { GOLD } else { DIM });
        }
        let rows = self.opt_rows(tab);
        let y0 = 170.0;
        let gap = 40.0;
        for (i, (label, val)) in rows.iter().enumerate() {
            let y = y0 + i as f32 * gap;
            let on = row == i + 1;
            let c = if on { GOLD } else { DIM };
            if on {
                draw_rectangle(
                    30.0,
                    y - 24.0,
                    W - 60.0,
                    34.0,
                    Color::new(0.3, 0.25, 0.05, 0.4),
                );
            }
            if val.is_empty() {
                ui::text_c(label, W / 2.0, y, 18.0, c);
            } else {
                ui::text(label, 46.0, y, 16.0, c);
                let vc = if label == "APAGAR PROGRESSO" && self.reset_armed > 0.0 {
                    RED
                } else if on {
                    WHITE
                } else {
                    DIM
                };
                ui::text_r(val, W - 46.0, y, 16.0, vc);
            }
        }
        if tab == 3 {
            let keys = [
                ("Mover", "Setas ou A/D"),
                ("Atirar", "Aperte Espaço, Z ou J"),
                ("Bomba", "X ou K"),
                ("Habilidade", "C, L ou V"),
                ("Tiro carregado", "Segure o tiro e solte"),
                ("Modo preciso", "Shift"),
                ("Pausa", "Esc ou P"),
                ("Tela cheia", "F11 ou Alt+Enter"),
                ("Minigames", "Setas e Espaço"),
            ];
            let y = 250.0;
            ui::panel(
                30.0,
                y - 30.0,
                W - 60.0,
                keys.len() as f32 * 30.0 + 30.0,
                CYAN,
            );
            for (i, (a, k)) in keys.iter().enumerate() {
                let yy = y + i as f32 * 30.0;
                ui::text(a, 50.0, yy, 15.0, WHITE);
                ui::text_r(k, W - 50.0, yy, 15.0, CYAN);
            }
        }
        let hint = match (tab, row) {
            (_, 0) => "ESQ/DIR troca de aba",
            (0, 4) => "Menos partículas deixa o jogo mais leve.",
            (0, 5) => "Suaves: clarões de explosão bem mais fracos.",
            (0, 6) => "Painéis com pontos e arma nas laterais (telas largas).",
            (0, 7) => "Desligado: o fundo quase não se mexe.",
            (2, 1) => "Português ou inglês.",
            (2, 2) => "Mostra sempre o ponto vulnerável da nave.",
            (2, 5) => "Apaga patentes, cartas e conquistas. Aperte 2x.",
            _ => "Q/TAB troca de aba  ·  ESQ/DIR ajusta  ·  ESC volta",
        };
        ui::text_c(hint, W / 2.0, H - 70.0, 13.0, DIM);
        let path = save::data_dir();
        let p = path.display().to_string();
        let p = if ui::width(&p, 11.0) > W - 20.0 {
            format!("...{}", &p[p.len().saturating_sub(48)..])
        } else {
            p
        };
        ui::text_c(
            &format!("Progresso: {p}"),
            W / 2.0,
            H - 30.0,
            11.0,
            Color::new(0.4, 0.4, 0.5, 1.0),
        );
    }

    fn draw_honors(&self, sel: usize) {
        let top = 112.0;
        let found: Vec<Weapon> = ships::RELICS
            .iter()
            .enumerate()
            .filter(|(i, _)| self.save.relics.get(*i).copied().unwrap_or(false))
            .map(|(_, w)| *w)
            .collect();
        if !found.is_empty() {
            ui::text_c("RELÍQUIAS", W / 2.0, top + 8.0, 15.0, GOLD);
            let cw = (W - 30.0) / 4.0;
            let x0 = W / 2.0 - cw * found.len() as f32 / 2.0;
            for (i, w) in found.iter().enumerate() {
                let cx = x0 + cw * i as f32 + cw / 2.0;
                let c = vec2(cx, top + 46.0);
                draw_circle(c.x, c.y, 26.0, Color::new(1.0, 0.85, 0.3, 0.15));
                icons::weapon_badge(*w, c, 18.0, 1.0);
                ui::text_c(w.name(), cx, top + 86.0, 10.0, GOLD);
                for (j, l) in ui::wrap(w.desc(), 10.0, cw - 6.0)
                    .iter()
                    .take(3)
                    .enumerate()
                {
                    ui::text_c(
                        l,
                        cx,
                        top + 102.0 + j as f32 * 12.0,
                        10.0,
                        Color::new(0.8, 0.8, 0.9, 1.0),
                    );
                }
            }
        } else {
            ui::text_c(
                "Feitos raros são lembrados aqui.",
                W / 2.0,
                top + 60.0,
                13.0,
                DIM,
            );
        }
        let earned = progress::honors_earned(&self.save.achs);
        ui::text_c(
            &format!("INSÍGNIAS DE HONRA  {}/{}", earned, progress::HONORS.len()),
            W / 2.0,
            top + 166.0,
            15.0,
            GOLD,
        );
        ui::text_c(
            "Patentes que não vêm de XP: vêm de feitos.",
            W / 2.0,
            top + 184.0,
            11.0,
            DIM,
        );
        let y0 = top + 200.0;
        for (i, h) in progress::HONORS.iter().enumerate() {
            let got = self.save.achs.get(h.ach).copied().unwrap_or(false);
            let y = y0 + i as f32 * 27.0;
            if i == sel {
                draw_rectangle(
                    14.0,
                    y - 2.0,
                    W - 28.0,
                    26.0,
                    Color::new(0.3, 0.25, 0.05, 0.5),
                );
            }
            let c = if got {
                GOLD
            } else {
                Color::new(0.3, 0.3, 0.4, 1.0)
            };
            draw_poly(32.0, y + 11.0, 5, 9.0, -90.0, c);
            draw_poly_lines(
                32.0,
                y + 11.0,
                5,
                9.0,
                -90.0,
                1.2,
                if got { WHITE } else { DIM },
            );
            ui::text(
                h.shown_name(&self.save.achs),
                50.0,
                y + 16.0,
                13.0,
                if got { WHITE } else { DIM },
            );
            ui::text_r(
                h.shown_short(&self.save.achs),
                W - 22.0,
                y + 16.0,
                11.0,
                if got {
                    GREEN
                } else {
                    Color::new(0.45, 0.45, 0.55, 1.0)
                },
            );
        }
        if let Some(h) = progress::HONORS.get(sel) {
            let got = self.save.achs.get(h.ach).copied().unwrap_or(false);
            let py = y0 + progress::HONORS.len() as f32 * 27.0 + 6.0;
            ui::panel(20.0, py, W - 40.0, 56.0, if got { GOLD } else { DIM });
            ui::text(
                &format!("FEITO: {}", h.shown_feat(&self.save.achs)),
                32.0,
                py + 22.0,
                12.0,
                WHITE,
            );
            ui::text(
                &format!("PRÊMIO: {}", h.shown_reward(&self.save.achs)),
                32.0,
                py + 42.0,
                12.0,
                if got { GREEN } else { CYAN },
            );
        }
    }

    fn draw_prologue(&self, t: f32) {
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.02, 1.0));
        for i in 0..90 {
            let x = ((i as f32 * 97.31).sin() * 0.5 + 0.5) * W;
            let y = ((i as f32 * 53.7).cos() * 0.5 + 0.5) * H;
            let tw = ((t * 2.0 + i as f32).sin() * 0.5 + 0.5) * 0.7 + 0.2;
            draw_circle(
                x,
                y,
                if i % 7 == 0 { 1.6 } else { 1.0 },
                Color::new(1.0, 1.0, 1.0, tw),
            );
        }
        // a sonda cruzando a tela
        let sx = -40.0 + (t * 18.0) % (W + 80.0);
        let sy = 120.0 + (t * 0.5).sin() * 10.0;
        self.art.ships[7][0].draw(vec2(sx, sy), 2.0, std::f32::consts::FRAC_PI_2, WHITE, false);
        for k in 0..14 {
            let q = vec2(
                sx - 18.0 - k as f32 * 7.0,
                sy + (t * 6.0 + k as f32).sin() * 2.0,
            );
            draw_circle(
                q.x,
                q.y,
                2.5 - k as f32 * 0.15,
                Color::new(1.0, 0.9, 0.6, 0.6 - k as f32 * 0.04),
            );
        }
        let step = 3.2;
        for (i, line) in PROLOGUE.iter().enumerate() {
            let start = 0.8 + i as f32 * step;
            if t < start {
                break;
            }
            let a = ((t - start) / 0.8).min(1.0);
            let y = 230.0 + i as f32 * 46.0;
            for (j, l) in ui::wrap(line, 16.0, W - 50.0).iter().enumerate() {
                ui::text_c(
                    l,
                    W / 2.0,
                    y + j as f32 * 19.0,
                    16.0,
                    Color::new(0.9, 0.9, 1.0, a),
                );
            }
        }
        ui::text_outline(
            "STARDUST",
            W / 2.0,
            70.0,
            40.0,
            GOLD,
            Color::from_rgba(40, 20, 0, 255),
        );
        if t > 1.0 {
            ui::text_c("ENTER para pular", W / 2.0, H - 16.0, 12.0, DIM);
        }
    }

    fn draw_howto(&self, page: usize) {
        ui::text_outline(
            "COMO JOGAR",
            W / 2.0,
            80.0,
            40.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        ui::panel(20.0, 106.0, W - 40.0, 548.0, CYAN);
        if page == 1 {
            self.draw_weapons_page();
            ui::text_c(
                &format!("< {} / 6 >   ·   ESC volta", page + 1),
                W / 2.0,
                690.0,
                15.0,
                DIM,
            );
            return;
        }
        let lines: Vec<(String, Color)> = match page {
            0 => vec![
                ("CONTROLES".into(), GOLD),
                ("Setas / A D ........ mover para os lados".into(), WHITE),
                ("Espaço / Z / J ..... atirar (aperte)".into(), WHITE),
                ("Segure e solte ..... tiro carregado".into(), WHITE),
                ("X / K .............. bomba".into(), WHITE),
                ("C / L / V .......... habilidade da nave".into(), WHITE),
                ("Shift .............. modo preciso".into(), WHITE),
                ("Esc / P ............ pausa".into(), WHITE),
                ("F11 / Alt+Enter .... tela cheia".into(), WHITE),
                ("".into(), WHITE),
                ("O OBJETIVO".into(), GOLD),
                ("Atravesse 8 setores e derrote a Rainha".into(), WHITE),
                ("Suprema. Cada setor tem 3 ondas, um".into(), WHITE),
                ("estágio bônus e um chefe. Depois, loja.".into(), WHITE),
                ("".into(), WHITE),
                ("Só o centro da nave é vulnerável.".into(), CYAN),
                ("TIRO AUTOMÁTICO se compra na loja.".into(), GOLD),
            ],
            2 => vec![
                ("PODERES (bolinhas que caem)".into(), GOLD),
                (
                    "F  Frenesi: atira o dobro".into(),
                    Color::from_rgba(255, 80, 60, 255),
                ),
                ("*  Estrela: invencível e atropela".into(), GOLD),
                ("U  Super ímã: puxa todos os cristais".into(), PINK),
                (
                    "Z  Câmera lenta para o mundo".into(),
                    Color::from_rgba(120, 160, 255, 255),
                ),
                ("S  Escudo   B  Bomba   $  Cristais".into(), CYAN),
                ("1UP  Vida extra (rara!)".into(), GREEN),
                ("".into(), WHITE),
                ("HABILIDADES".into(), GOLD),
                ("Cada nave tem a sua. A barra enche".into(), WHITE),
                ("abatendo, raspando e com o tempo.".into(), WHITE),
                ("Troque de nave no HANGAR.".into(), WHITE),
                ("".into(), WHITE),
                ("PATENTES".into(), GOLD),
                ("Toda partida dá XP. Subir de patente".into(), WHITE),
                ("libera naves, armas, modos e pinturas.".into(), WHITE),
            ],
            3 => vec![
                ("FEBRE".into(), GOLD),
                ("Combo 50 = Febre: pontos em dobro e um".into(), WHITE),
                ("cristal em cada abate.".into(), WHITE),
                ("CONTRATOS".into(), GOLD),
                ("3 missões por partida. Cumpra e ganhe XP".into(), WHITE),
                ("e cristais. Veja na pausa ou no painel.".into(), WHITE),
                ("EVENTOS DE SETOR".into(), GOLD),
                ("Eclipse, meteoros, emboscadas, comboios...".into(), WHITE),
                ("CUIDADO COM".into(), GOLD),
                (
                    "Mímicos: cápsulas que mordem.".into(),
                    Color::from_rgba(255, 120, 120, 255),
                ),
                (
                    "Ladrões: roubam o que cai. Derrube-os!".into(),
                    Color::from_rgba(190, 140, 255, 255),
                ),
                ("Elites (borda dourada): fortes e ricos.".into(), GOLD),
                ("REGISTROS DE BORDO".into(), GOLD),
                (
                    "Caem dos chefes. Contam a verdade.".into(),
                    Color::from_rgba(120, 255, 220, 255),
                ),
            ],
            4 => vec![
                ("CAVEIRAS (bolhas vermelhas): desvie!".into(), RED),
                ("Tiram sua arma, níveis ou travam a nave.".into(), WHITE),
                ("3 tiros destroem uma maldição.".into(), WHITE),
                ("FENDAS (hexágono com ?)".into(), CYAN),
                ("Abrem um minigame rápido. Vença e ganhe".into(), WHITE),
                ("cristais, níveis, poderes e cartas.".into(), WHITE),
                ("LETRAS S-T-A-R-D-U-S-T".into(), GOLD),
                ("Junte as 8: vida extra e muitos pontos.".into(), WHITE),
                ("TIRO CARREGADO".into(), GOLD),
                ("Segure o tiro e solte: atravessa tudo.".into(), WHITE),
                ("MINAS".into(), GOLD),
                ("Minadores largam minas: atire nelas.".into(), WHITE),
                ("INSÍGNIAS DE HONRA".into(), GOLD),
                ("Feitos liberam naves e rastros. Veja nos".into(), WHITE),
                ("ARQUIVOS, aba HONRAS.".into(), WHITE),
            ],
            _ => vec![
                ("CARTAS".into(), GOLD),
                ("Caem raramente; chefes sempre soltam.".into(), WHITE),
                ("Equipe no menu CARTAS. Com 8, 16 e 30".into(), WHITE),
                ("cartas você ganha mais espaços.".into(), WHITE),
                ("".into(), WHITE),
                ("LOJA E PODER".into(), GOLD),
                ("Entre os setores, gaste cristais em armas".into(), WHITE),
                ("melhores, Núcleos de Dano, cadência,".into(), WHITE),
                ("drones e no TIRO AUTOMÁTICO.".into(), WHITE),
                ("Quanto mais cara a arma, mais forte.".into(), WHITE),
                ("".into(), WHITE),
                ("PATENTES".into(), GOLD),
                ("Cada partida dá XP e libera naves,".into(), WHITE),
                ("armas, modos e pinturas.".into(), WHITE),
                ("".into(), WHITE),
                ("Boa sorte, piloto.".into(), PINK),
            ],
        };
        for (i, (l, c)) in lines.iter().enumerate() {
            let size = if *c == GOLD { 19.0 } else { 15.0 };
            let step = if lines.len() > 17 {
                520.0 / lines.len() as f32
            } else {
                30.0
            };
            ui::text(l, 38.0, 140.0 + i as f32 * step, size, *c);
        }
        ui::text_c(
            &format!("< {} / 6 >   ·   ESC volta", page + 1),
            W / 2.0,
            690.0,
            15.0,
            DIM,
        );
    }

    fn draw_weapons_page(&self) {
        ui::text_c("ARMAS", W / 2.0, 136.0, 19.0, GOLD);
        ui::text_c(
            "Mesma arma sobe de nível · outra arma substitui a atual",
            W / 2.0,
            156.0,
            11.0,
            DIM,
        );
        let rank = self.rank();
        let col_w = (W - 60.0) / 2.0;
        let visible: Vec<Weapon> = WEAPONS
            .iter()
            .copied()
            .filter(|w| {
                if w.is_relic() {
                    ships::RELICS
                        .iter()
                        .position(|r| r == w)
                        .map(|k| self.save.relics.get(k).copied().unwrap_or(false))
                        .unwrap_or(false)
                } else {
                    w.unlock_rank().is_some() || w.unlocked(rank, &self.save.achs)
                }
            })
            .collect();
        let per_col = visible.len().div_ceil(2).max(1);
        let gap = (470.0 / per_col as f32).min(38.0);
        for (i, w) in visible.iter().enumerate() {
            let col = i / per_col;
            let row = i % per_col;
            let x = 34.0 + col as f32 * col_w;
            let y = 184.0 + row as f32 * gap;
            let c = vec2(x + 12.0, y);
            icons::weapon_badge(*w, c, 12.0, 1.0);
            ui::text(
                &format!("{} {}", w.name(), "*".repeat(w.tier() as usize)),
                x + 32.0,
                y,
                13.0,
                w.color(),
            );
            let sub = match w.unlock_rank() {
                Some(r) if r > rank => {
                    let full = format!("patente {}", RANKS[r]);
                    if ui::width(&full, 10.0) > col_w - 40.0 {
                        format!("patente nº {}", r + 1)
                    } else {
                        full
                    }
                }
                _ if w.is_relic() => "RELÍQUIA".to_string(),
                _ => String::new(),
            };
            if !sub.is_empty() {
                ui::text(
                    &sub,
                    x + 32.0,
                    y + 13.0,
                    10.0,
                    if w.is_relic() { GOLD } else { RED },
                );
            }
        }
    }

    fn draw_sheet(&self) {
        draw_rectangle(0.0, 0.0, W, H, Color::from_rgba(40, 40, 60, 255));
        let mut x = 30.0;
        let mut y = 40.0;
        let mut put = |spr: &sprites::Spr, scale: f32| {
            let w = spr.w * scale;
            if x + w > W - 10.0 {
                x = 30.0;
                y += 62.0;
            }
            spr.draw(vec2(x + w / 2.0, y), scale, 0.0, WHITE, false);
            x += w + 12.0;
        };
        for s in &self.art.ships {
            put(&s[0], 3.0);
        }
        for e in &self.sp.enemies {
            put(&e[0], 3.0);
        }
        put(&self.art.plate, 3.0);
        put(&self.art.serpent_seg, 3.0);
        put(&self.art.leg, 3.0);
        for k in 0..art::SKINS {
            put(&self.art.ships[0][k], 2.0);
        }
        self.art
            .serpent_head
            .draw(vec2(70.0, 470.0), 3.5, 0.0, WHITE, false);
        self.art
            .spider
            .draw(vec2(200.0, 470.0), 3.0, 0.0, WHITE, false);
        self.art
            .eye_open
            .draw(vec2(330.0, 470.0), 3.0, 0.0, WHITE, false);
        self.art
            .eye_closed
            .draw(vec2(330.0, 580.0), 3.0, 0.0, WHITE, false);
        for r in 0..10 {
            progress::draw_insignia(r * 2 + 1, 30.0 + r as f32 * 44.0, 680.0, 1.2);
        }
    }

    // -----------------------------------------------------------------------
    // Painéis laterais (tela larga / tela cheia)
    // -----------------------------------------------------------------------

    fn draw_panels(&self, total_w: f32) {
        let pw = (total_w - W) / 2.0;
        let shade = Color::new(0.0, 0.0, 0.04, 0.55);
        draw_rectangle(0.0, 0.0, pw, H, shade);
        draw_rectangle(pw + W, 0.0, pw, H, shade);
        let cw = (pw - 40.0).min(300.0);
        let lx = (pw - cw) / 2.0;
        let rx = pw + W + (pw - cw) / 2.0;
        let lc = lx + cw / 2.0;
        let rc = rx + cw / 2.0;

        // esquerda: logotipo, pontos e patente
        let title_size = (cw / 5.0).clamp(26.0, 54.0);
        ui::text_outline_fit(
            "STARDUST",
            lc,
            70.0,
            title_size,
            ui::hsl(0.55 + (self.time * 0.05).fract() * 0.3, 0.9, 0.62),
            Color::from_rgba(20, 8, 40, 255),
            cw - 16.0,
        );
        let r = self.rank();
        let ry = H - 120.0;
        progress::draw_insignia(r, lc, ry, 2.0);
        ui::text_c(RANKS[r], lc, ry + 50.0, 16.0, GOLD);
        let (a, b) = (
            progress::xp_for(r),
            progress::xp_for((r + 1).min(RANKS.len() - 1)),
        );
        let frac = if b > a {
            (self.save.xp - a) as f32 / (b - a) as f32
        } else {
            1.0
        };
        draw_rectangle(lx, ry + 60.0, cw, 6.0, Color::new(0.2, 0.2, 0.3, 1.0));
        draw_rectangle(lx, ry + 60.0, cw * frac.min(1.0), 6.0, GOLD);

        let Some(g) = &self.game else {
            // nos menus: dicas e um desfile de inimigos
            ui::text_c("DICA DO DIA", lc, 160.0, 15.0, CYAN);
            let tip = TIPS[((self.time / 8.0) as usize) % TIPS.len()];
            for (i, l) in ui::wrap(tip, 15.0, cw).iter().enumerate() {
                ui::text_c(l, lc, 186.0 + i as f32 * 18.0, 15.0, WHITE);
            }
            for i in 0..5 {
                let kind = ((self.time * 0.3) as usize + i) % 11;
                let y = 120.0 + i as f32 * 100.0 + (self.time * 1.5 + i as f32).sin() * 10.0;
                let f = ((self.time * 6.0) as usize + i) % 2;
                self.sp.enemies[kind][f].draw(
                    vec2(rc + (self.time + i as f32).sin() * cw * 0.3, y),
                    3.0,
                    0.0,
                    Color::new(1.0, 1.0, 1.0, 0.8),
                    false,
                );
            }
            ui::text_c(
                &format!("CARTAS {}/{}", self.save.unique_cards(), CARDS.len()),
                rc,
                H - 70.0,
                15.0,
                PINK,
            );
            ui::text_c(
                &format!(
                    "CONQUISTAS {}/{}",
                    self.save.achs.iter().filter(|a| **a).count(),
                    ACHS.len()
                ),
                rc,
                H - 48.0,
                15.0,
                GOLD,
            );
            return;
        };

        let mut y = 130.0;
        ui::text("PONTOS", lx, y, 14.0, DIM);
        y += 30.0;
        ui::text(
            &format!("{:08}", g.score),
            lx,
            y,
            (cw / 7.0).clamp(20.0, 34.0),
            WHITE,
        );
        y += 30.0;
        ui::text("RECORDE", lx, y, 14.0, DIM);
        ui::text_r(&format!("{:08}", g.cfg.highscore), lx + cw, y, 16.0, GOLD);
        y += 40.0;
        let mult = g.multiplier();
        ui::text("COMBO", lx, y, 14.0, DIM);
        ui::text_r(
            &format!("{}  x{}", g.combo, mult),
            lx + cw,
            y,
            20.0,
            if mult > 1 { PINK } else { WHITE },
        );
        y += 40.0;
        ui::text(g.cfg.mode.name(), lx, y, 15.0, CYAN);
        ui::text_r(g.cfg.diff.name(), lx + cw, y, 15.0, g.cfg.diff.color());
        y += 24.0;
        ui::text(&g.stage_label(), lx, y, 15.0, WHITE);
        y += 20.0;
        ui::text(g.theme_name(), lx, y, 13.0, DIM);
        if let Some(d) = g.cfg.daily {
            y += 26.0;
            ui::text(
                &format!("REGRA: {}", d.name()),
                lx,
                y,
                13.0,
                Color::from_rgba(255, 150, 60, 255),
            );
        }
        if g.cycle > 0 {
            y += 22.0;
            ui::text(&format!("CICLO {}", g.cycle + 1), lx, y, 13.0, PINK);
        }
        if let Some(ev) = g.x.event {
            y += 22.0;
            ui::text(
                &format!("EVENTO: {}", ev.name()),
                lx,
                y,
                13.0,
                Color::from_rgba(255, 140, 80, 255),
            );
        }
        if g.x.fever_t > 0.0 {
            y += 26.0;
            ui::text(
                &format!("FEBRE x2   {:.0}s", g.x.fever_t.ceil()),
                lx,
                y,
                18.0,
                ui::hsl(self.time * 0.6, 0.9, 0.6),
            );
        }
        // contratos
        let mut cy = (y + 40.0).max(400.0);
        if !g.x.contracts.is_empty() {
            ui::text("CONTRATOS", lx, cy, 14.0, DIM);
            cy += 20.0;
            for c in &g.x.contracts {
                let col = if c.done { GREEN } else { WHITE };
                ui::text(&c.text(), lx, cy, 13.0, col);
                let frac = (c.progress as f32 / c.target.max(1) as f32).min(1.0);
                draw_rectangle(lx, cy + 5.0, cw, 4.0, Color::new(0.2, 0.2, 0.3, 1.0));
                draw_rectangle(
                    lx,
                    cy + 5.0,
                    cw * frac,
                    4.0,
                    if c.done { GREEN } else { CYAN },
                );
                ui::text_r(if c.done { "OK" } else { "" }, lx + cw, cy, 13.0, GREEN);
                cy += 28.0;
            }
        }
        // letras S-T-A-R-D-U-S-T
        cy += 8.0;
        ui::text("LETRAS", lx, cy, 14.0, DIM);
        let letters = g.letters_line();
        let step = (cw / 8.0).min(30.0);
        for (i, (ch, got)) in letters.iter().enumerate() {
            let x = lx + step * i as f32 + step / 2.0;
            let y = cy + 22.0;
            draw_circle(
                x,
                y - 5.0,
                step * 0.42,
                if *got {
                    Color::from_rgba(255, 200, 60, 255)
                } else {
                    Color::new(0.15, 0.15, 0.22, 1.0)
                },
            );
            ui::text_c(
                &ch.to_string(),
                x,
                y + 1.0,
                15.0,
                if *got {
                    Color::from_rgba(90, 40, 0, 255)
                } else {
                    DIM
                },
            );
        }

        // direita: nave, arma, habilidade, poderes e cartas
        let p = &g.player;
        let mut y = 60.0;
        ui::text("NAVES", rx, y, 14.0, DIM);
        let ship = &self.art.ships[g.cfg.ship][g.cfg.skin];
        for i in 0..p.lives.max(0).min(7) {
            ship.draw(
                vec2(rx + 10.0 + i as f32 * 24.0, y + 20.0),
                1.4,
                0.0,
                WHITE,
                false,
            );
        }
        y += 54.0;
        ui::text("BOMBAS", rx, y, 14.0, DIM);
        for i in 0..p.bombs.max(0).min(7) {
            self.sp.icon(sprites::Icon::Bomb).draw(
                vec2(rx + 10.0 + i as f32 * 22.0, y + 18.0),
                1.8,
                0.0,
                WHITE,
                false,
            );
        }
        if g.crystals_visible() {
            ui::text_r(
                &format!("{}", g.crystals.floor() as u32),
                rx + cw,
                y + 24.0,
                22.0,
                CYAN,
            );
            self.sp.crystal[0].draw(vec2(rx + cw - 60.0, y + 16.0), 2.5, 0.0, WHITE, false);
        }
        y += 56.0;
        let w = p.weapon;
        ui::text("ARMA", rx, y, 14.0, DIM);
        icons::weapon_badge(w, vec2(rx + 58.0, y - 5.0), 11.0, 1.0);
        ui::text_r(w.name(), rx + cw, y, 18.0, w.color());
        y += 12.0;
        let lv = g.effective_level();
        for i in 0..5 {
            let c = if i < lv {
                w.color()
            } else {
                Color::new(0.25, 0.25, 0.35, 1.0)
            };
            draw_rectangle(rx + i as f32 * (cw / 5.0), y, cw / 5.0 - 4.0, 10.0, c);
        }
        y += 28.0;
        for (i, l) in ui::wrap(w.desc(), 13.0, cw).iter().take(2).enumerate() {
            ui::text(l, rx, y + i as f32 * 15.0, 13.0, DIM);
        }
        y += 36.0;
        ui::text("PODER DE FOGO", rx, y, 13.0, DIM);
        ui::text_r(&format!("x{:.2}", g.power_mul()), rx + cw, y, 15.0, GOLD);
        y += 26.0;
        y += 28.0;
        for (t, c) in g.curse_status() {
            ui::text(&format!("MALDIÇÃO: {t}"), rx, y - 8.0, 13.0, c);
            y += 16.0;
        }
        if g.cfg.mode != Mode::Classico {
            let s = g.ship();
            ui::text("HABILIDADE  [C]", rx, y, 14.0, DIM);
            ui::text_r(s.ab_name, rx + cw, y, 15.0, PINK);
            y += 10.0;
            draw_rectangle(rx, y, cw, 10.0, Color::new(0.2, 0.2, 0.3, 1.0));
            let full = p.meter >= s.ab_cost;
            let c = if full {
                ui::hsl(self.time, 0.8, 0.6)
            } else {
                CYAN
            };
            draw_rectangle(rx, y, cw * p.meter, 10.0, c);
            if s.ab_cost < 1.0 {
                draw_line(
                    rx + cw * s.ab_cost,
                    y,
                    rx + cw * s.ab_cost,
                    y + 10.0,
                    2.0,
                    WHITE,
                );
            }
            y += 28.0;
            ui::text(&format!("RASPÕES {}", g.grazes), rx, y, 13.0, DIM);
            ui::text_r(
                &format!("DRONES {}", g.drones_total()),
                rx + cw,
                y,
                13.0,
                DIM,
            );
            y += 30.0;
        }
        for (label, t, c) in g.active_powers() {
            ui::text(label, rx, y, 15.0, c);
            ui::text_r(&format!("{:.0}s", t.ceil()), rx + cw, y, 15.0, c);
            y += 20.0;
        }
        if !g.cfg.equipped.is_empty() && g.cfg.mode != Mode::Classico {
            let cy = H - 230.0;
            ui::text("CARTAS", rx, cy, 14.0, DIM);
            let n = g.cfg.equipped.len();
            let scale = ((cw - (n as f32 - 1.0) * 8.0) / n as f32 / ui::CARD_W).min(0.75);
            for (i, &c) in g.cfg.equipped.iter().enumerate() {
                ui::card_face(
                    &self.sp,
                    c,
                    rx + i as f32 * (ui::CARD_W * scale + 8.0),
                    cy + 10.0,
                    scale,
                    true,
                    0.0,
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Robô (demonstração e testes)
// ---------------------------------------------------------------------------

fn bot_input(g: &Game, t: f32) -> Input {
    let mut inp = Input {
        fire: true,
        fire_p: true,
        ..Default::default()
    };
    let p = g.player.pos;
    let mut threat: Option<Vec2> = None;
    for b in &g.ebullets {
        let d = b.pos - p;
        if d.y > -120.0 && d.y < 20.0 && d.x.abs() < 40.0 {
            threat = Some(b.pos);
            break;
        }
    }
    match threat {
        Some(b) => {
            if b.x < p.x {
                inp.right = true
            } else {
                inp.left = true
            }
        }
        None => {
            let target = W / 2.0 + (t * 0.7).sin() * 170.0;
            inp.left = p.x > target + 10.0;
            inp.right = p.x < target - 10.0;
        }
    }
    inp.down = p.y < H - 90.0;
    inp.bomb = g
        .ebullets
        .iter()
        .filter(|b| b.pos.distance(p) < 60.0)
        .count()
        > 4
        && (t * 10.0) as i32 % 17 == 0;
    inp.ability_p = g.player.meter >= 1.0 && (t * 10.0) as i32 % 23 == 0;
    let tick = (t * 2.0).fract() < 0.05 && (t * 2.0) as i32 % 3 == 0;
    inp.confirm = tick;
    inp.enter = tick;
    inp.down_p = matches!(g.phase, game::Phase::Shop) && (t * 3.0).fract() < 0.05;
    inp
}

// ---------------------------------------------------------------------------
// Tela final (escala, painéis e filtro CRT)
// ---------------------------------------------------------------------------

const CRT_VS: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}
"#;

const CRT_FS: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    vec3 c = texture2D(Texture, uv).rgb;
    float line = sin(uv.y * 720.0 * 3.14159);
    c *= 0.86 + 0.14 * line * line;
    vec2 d = uv - 0.5;
    c *= 1.0 - dot(d, d) * 0.5;
    c = c * 1.06;
    gl_FragColor = vec4(c, 1.0);
}
"#;

/// Layout da tela: tudo é desenhado em unidades lógicas (altura 720) direto num
/// alvo de renderização do tamanho real da tela em pixels físicos (respeita o DPI).
struct Layout {
    ww: f32,
    k: f32,
    px_w: u32,
    px_h: u32,
    play_x: f32,
    dest: Rect,
    panels: bool,
}

struct Screens {
    rt: Option<(RenderTarget, u32, u32)>,
    crt: Option<Material>,
}

impl Screens {
    fn new() -> Screens {
        let crt = load_material(
            ShaderSource::Glsl {
                vertex: CRT_VS,
                fragment: CRT_FS,
            },
            MaterialParams::default(),
        )
        .ok();
        Screens { rt: None, crt }
    }

    fn layout() -> Layout {
        let sw = screen_width().max(1.0);
        let sh = screen_height().max(1.0);
        let dpi = screen_dpi_scale().max(0.5);
        let (pw, ph) = (sw * dpi, sh * dpi);
        let ww = (H * sw / sh).round().max(W);
        let k = (pw / ww).min(ph / H);
        let px_w = (ww * k).round().max(1.0) as u32;
        let px_h = (H * k).round().max(1.0) as u32;
        let (dw, dh) = (px_w as f32 / dpi, px_h as f32 / dpi);
        Layout {
            ww,
            k,
            px_w,
            px_h,
            play_x: ((ww - W) / 2.0).floor(),
            dest: Rect::new(((sw - dw) / 2.0).floor(), ((sh - dh) / 2.0).floor(), dw, dh),
            panels: ww - W >= 300.0,
        }
    }

    fn ensure(&mut self, l: &Layout) {
        let need = match &self.rt {
            Some((_, w, h)) => *w != l.px_w || *h != l.px_h,
            None => true,
        };
        if need {
            let rt = render_target(l.px_w, l.px_h);
            rt.texture.set_filter(FilterMode::Nearest);
            self.rt = Some((rt, l.px_w, l.px_h));
        }
    }

    fn wide_cam(&self, l: &Layout) -> Camera2D {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, l.ww, H));
        cam.render_target = self.rt.as_ref().map(|r| r.0.clone());
        cam
    }

    fn play_cam(&self, l: &Layout) -> Camera2D {
        let mut cam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, W, H));
        cam.render_target = self.rt.as_ref().map(|r| r.0.clone());
        let x = (l.play_x * l.k).round() as i32;
        cam.viewport = Some((x, 0, (W * l.k).round() as i32, l.px_h as i32));
        cam
    }

    fn present(&self, l: &Layout, use_crt: bool) {
        let Some((rt, _, _)) = &self.rt else { return };
        set_default_camera();
        clear_background(Color::from_rgba(4, 3, 12, 255));
        if let (Some(m), true) = (&self.crt, use_crt) {
            gl_use_material(m);
        }
        draw_texture_ex(
            &rt.texture,
            l.dest.x,
            l.dest.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(l.dest.w, l.dest.h)),
                flip_y: true,
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args
        .iter()
        .position(|a| a == "--exportar-icone" || a == "--export-icon")
    {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| ".".into());
        for n in [16usize, 24, 32, 48, 64, 128, 256] {
            let img = Image {
                bytes: sprites::window_icon_rgba(n),
                width: n as u16,
                height: n as u16,
            };
            img.export_png(&format!("{dir}/icone_{n}.png"));
        }
        return;
    }
    if args
        .iter()
        .any(|a| a == "--exportar-musicas" || a == "--export-music")
    {
        let _ = audio::export_music("musica_exportada");
        return;
    }
    macroquad::rand::srand(miniquad::date::now() as u64);

    let mut screens = Screens::new();
    for _ in 0..2 {
        let l = Screens::layout();
        ui::set_pixel_scale(l.k);
        screens.ensure(&l);
        set_camera(&screens.wide_cam(&l));
        draw_rectangle(0.0, 0.0, l.ww, H, Color::from_rgba(4, 4, 16, 255));
        set_camera(&screens.play_cam(&l));
        ui::text_outline(
            "STARDUST",
            W / 2.0,
            H / 2.0 - 20.0,
            48.0,
            CYAN,
            Color::from_rgba(20, 8, 40, 255),
        );
        ui::text_c(
            "compondo a trilha sonora...",
            W / 2.0,
            H / 2.0 + 24.0,
            16.0,
            DIM,
        );
        screens.present(&l, false);
        next_frame().await;
    }

    let save = Save::load();
    if save.fullscreen {
        set_fullscreen(true);
    }
    let audio = Audio::load(|_| {}).await;
    let mut bg = Background::new();
    bg.set_theme(0);
    let env = dev_env;
    let mut sp = sprites::build();
    let art = art::build(&mut sp);
    let mut app = App {
        screen: Screen::Title { sel: 0 },
        save,
        audio,
        sp,
        art,
        game: None,
        bg,
        fx: Fx::default(),
        time: 0.0,
        quit: false,
        autoplay: env("STARDUST_AUTOPLAY").is_some(),
        god: env("STARDUST_GOD").is_some(),
        start_wave: env("STARDUST_WAVE")
            .and_then(|v| v.parse().ok())
            .unwrap_or(1),
        theme_t: 0.0,
        toasts: vec![],
        idle: 0.0,
        demo: false,
        demo_t: 0.0,
        keys: vec![],
        typed: String::new(),
        logo_party: 0.0,
        hangar_trail: 0,
        reset_armed: 0.0,
        panels: false,
    };
    if let Some(x) = env("STARDUST_XP").and_then(|v| v.parse().ok()) {
        app.save.xp = x;
    }
    if let Some(s) = env("STARDUST_SHIP").and_then(|v| v.parse::<usize>().ok()) {
        app.save.ship = s.min(SHIPS.len() - 1);
    }
    if env("STARDUST_SHEET").is_some() {
        app.screen = Screen::Sheet;
    }
    match env("STARDUST_SCREEN").as_deref() {
        Some("cartas") => app.screen = Screen::Album { sel: 9 },
        Some("opcoes") => {
            app.screen = Screen::Options {
                sel: 0,
                from_pause: false,
            }
        }
        Some("ajuda") => app.screen = Screen::HowTo { page: 1 },
        Some("registros") => app.screen = Screen::Archives { tab: 4, sel: 0 },
        Some("honras") => app.screen = Screen::Archives { tab: 5, sel: 2 },
        Some("config") => {
            app.screen = Screen::Options {
                sel: 4,
                from_pause: false,
            }
        }
        Some("config3") => {
            app.screen = Screen::Options {
                sel: 300,
                from_pause: false,
            }
        }
        Some("prologo") => {
            app.screen = Screen::Prologue {
                t: 20.0,
                start: None,
            }
        }
        Some("bestiario") => app.screen = Screen::Archives { tab: 1, sel: 24 },
        Some("album2") => app.screen = Screen::Album { sel: 45 },
        Some("ajuda2") => app.screen = Screen::HowTo { page: 1 },
        Some("ajuda4") => app.screen = Screen::HowTo { page: 3 },
        Some("ajuda5") => app.screen = Screen::HowTo { page: 4 },
        Some("hangar") => {
            app.screen = Screen::Hangar {
                sel: 3,
                skin: 0,
                row: 0,
            }
        }
        Some("modo") => {
            app.screen = Screen::ModeSelect {
                sel: 2,
                diff: 1,
                row: 0,
            }
        }
        Some("arquivos") => app.screen = Screen::Archives { tab: 1, sel: 14 },
        Some("conquistas") => app.screen = Screen::Archives { tab: 0, sel: 0 },
        Some("patentes") => app.screen = Screen::Archives { tab: 3, sel: 5 },
        Some("resumo") => {
            app.screen = Screen::Summary {
                t: 0.0,
                gain: 2400,
                from_xp: 700,
                shown_ranks: 1,
                next: Box::new(Screen::Title { sel: 0 }),
            };
        }
        Some("nome") => {
            app.screen = Screen::NameEntry {
                name: "ALLAN".into(),
                score: 123_450,
                sector: 4,
                mode: 0,
                ship: 0,
            }
        }
        _ => {}
    }
    if let Some(d) = env("STARDUST_DIFF").and_then(|v| v.parse::<u8>().ok()) {
        app.save.diff = d.min(3);
    }
    if let Some(m) = env("STARDUST_MODE").and_then(|v| v.parse::<u8>().ok()) {
        app.save.mode = m;
    }
    app.apply_settings();

    let mut rep = Repeat::default();
    loop {
        let dt = get_frame_time().min(1.0 / 30.0);
        let mut inp = poll_input(&mut rep, dt);
        let alt = is_key_down(KeyCode::LeftAlt) || is_key_down(KeyCode::RightAlt);
        if is_key_pressed(KeyCode::F11) || (alt && is_key_pressed(KeyCode::Enter)) {
            app.save.fullscreen = !app.save.fullscreen;
            set_fullscreen(app.save.fullscreen);
            app.save.write();
            inp.confirm = false;
            inp.enter = false;
        }
        if app.autoplay {
            if let Some(g) = &app.game {
                inp = bot_input(g, app.time);
            } else {
                inp.enter = (app.time * 2.0).fract() < 0.05;
                inp.confirm = inp.enter;
            }
        }
        let l = Screens::layout();
        app.panels = l.panels && app.save.panels;
        app.update(dt, &inp);
        if app.quit {
            app.save.write();
            break;
        }

        ui::set_pixel_scale(l.k);
        screens.ensure(&l);
        set_camera(&screens.wide_cam(&l));
        let bg = app.game.as_ref().map(|g| &g.bg).unwrap_or(&app.bg);
        bg.draw_wide(l.ww);
        if app.panels {
            app.draw_panels(l.ww);
        }
        draw_rectangle(
            l.play_x - 3.0,
            -2.0,
            W + 6.0,
            H + 4.0,
            Color::from_rgba(70, 40, 110, 255),
        );
        set_camera(&screens.play_cam(&l));
        app.draw();
        if app.save.show_fps {
            ui::text_r(&format!("{} FPS", get_fps()), W - 6.0, 14.0, 11.0, GREEN);
        }
        screens.present(&l, app.save.crt);
        app.audio.end_frame();
        next_frame().await;
    }
}
