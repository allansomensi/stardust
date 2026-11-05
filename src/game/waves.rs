//! Waves: sector start, wave building, bonus stages, dives and the end of waves and sectors.

use super::*;

impl Game {
    pub(super) fn begin_sector(&mut self) {
        if self.cfg.mode == Mode::Campaign && self.cycle == 0 && self.level() >= 50 {
            self.ach(ach::VALKYRIE);
        }
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
            let interest = (self.crystals * 0.1).floor();
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

    pub(super) fn start_wave(&mut self, au: &mut Audio) {
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
                self.level_begin();
                self.build_level();
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

    pub(super) fn maybe_specials(&mut self) {
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

    pub(super) fn spawn_special(&mut self, kind: Kind, path: Path, dur: f32, delay: f32) {
        self.spawn(kind, 0, 0, EState::Waiting { delay, path, dur }, 97, false);
        if let Some(e) = self.enemies.last_mut() {
            e.harmless = true;
        }
    }

    pub(super) fn spawn(
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
        } else if bonus {
            0
        } else {
            kind.base_armor(self.diff())
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

    pub(super) fn row_kind(&self, row: usize, col: usize) -> Kind {
        if let Some(k) = self.vanguard_kind(row, col) {
            return k;
        }
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

    pub(super) fn flyby_path(template: u32, s: f32) -> Path {
        let mx = |x: f32| if s < 0.0 { x } else { W - x };
        let v = |x: f32, y: f32| vec2(mx(x), y);
        match template % 8 {
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
            3 => Path::two(
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
            4 => Path::two(
                // a sine wave across the screen
                [
                    v(-30.0, H * 0.25),
                    v(W * 0.2, H * 0.05),
                    v(W * 0.3, H * 0.55),
                    v(W * 0.5, H * 0.3),
                ],
                [
                    v(W * 0.5, H * 0.3),
                    v(W * 0.7, H * 0.05),
                    v(W * 0.8, H * 0.55),
                    v(W + 40.0, H * 0.3),
                ],
            ),
            5 => Path::one(
                // a U-turn from the top
                v(W * 0.2, -30.0),
                v(W * 0.25, H * 0.95),
                v(W * 0.75, H * 0.95),
                v(W * 0.8, -40.0),
            ),
            6 => Path::two(
                // a full circle in the middle, then out through the top
                [
                    v(-30.0, H * 0.4),
                    v(W * 0.6, H * 0.15),
                    v(W * 0.95, H * 0.5),
                    v(W * 0.5, H * 0.6),
                ],
                [
                    v(W * 0.5, H * 0.6),
                    v(W * 0.05, H * 0.7),
                    v(W * 0.15, H * 0.1),
                    v(W * 0.6, -40.0),
                ],
            ),
            _ => Path::two(
                // a zigzag down and back up the side
                [
                    v(W * 0.05, -30.0),
                    v(W * 0.95, H * 0.15),
                    v(W * 0.05, H * 0.3),
                    v(W * 0.9, H * 0.45),
                ],
                [
                    v(W * 0.9, H * 0.45),
                    v(W * 1.2, H * 0.6),
                    v(W * 0.95, H * 0.1),
                    v(W + 40.0, -40.0),
                ],
            ),
        }
    }

    pub(super) fn build_challenge(&mut self, golden: bool) {
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
        // Every bonus stage has its own choreography, picked from the sector number.
        let k = self.sector() + self.cycle * 7;
        let groups = if golden { 6 } else { 5 + k % 2 };
        let step = 1 + k % 3;
        let twin = k % 4 == 3;
        let spacing = 2.2 + (k % 3) as f32 * 0.3;
        for g in 0..groups {
            let s = match k % 3 {
                0 => {
                    if g % 2 == 0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
                1 => -1.0,
                _ => {
                    if (g / 2) % 2 == 0 {
                        -1.0
                    } else {
                        1.0
                    }
                }
            };
            let path = Self::flyby_path(k * 3 + g * step, s);
            let mirror = Self::flyby_path(k * 3 + g * step, -s);
            for j in 0..8 {
                let delay = 0.8 + g as f32 * spacing + (j / if twin { 2 } else { 1 }) as f32 * 0.14;
                let path = if twin && j % 2 == 1 { mirror } else { path };
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

    pub(super) fn start_dive(&mut self, i: usize) {
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

    pub(super) fn after_sector(&mut self, au: &mut Audio) -> Phase {
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

    pub(super) fn check_wave_end(&mut self, au: &mut Audio) -> Phase {
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
                if self.signal_finished() {
                    self.sig = None;
                    self.sig_done = true;
                    let lines = self.sector_tally();
                    return Phase::SectorClear {
                        t: 0.0,
                        lines,
                        shown: 0,
                    };
                }
                Phase::Playing
            }
        }
    }

    pub(super) fn sector_tally(&mut self) -> Vec<(String, u32, bool)> {
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

    pub(super) fn finish_boss(&mut self, au: &mut Audio) {
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
            self.events.push(Event::Rumble(None, 1.0, 700));
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
                if self.sc.dark_fight {
                    self.ach(ach::DARK_DUEL);
                }
            }
            if std::mem::take(&mut self.co.duo_boss) {
                self.ach(ach::TWIN_STARS);
            }
        }
        self.ebullets.clear();
    }
}
