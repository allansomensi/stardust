//! The HUD, banners, phase overlays and the credits.

use super::*;

impl Game {
    pub(super) fn draw_mini_hud(&self) {
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
        self.draw_signal_bar();
    }

    pub(super) fn draw_hud(&self, sp: &Sprites, art: &Art) {
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
        for i in 0..self.player.bombs.clamp(0, 4) {
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

    pub(super) fn draw_overlays(&self, sp: &Sprites) {
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
                ui::button(
                    "CONTINUE",
                    W / 2.0 - 70.0,
                    H * 0.76,
                    16.0,
                    ui::Hot::Confirm,
                    false,
                );
                ui::button(
                    "GIVE UP",
                    W / 2.0 + 70.0,
                    H * 0.76,
                    16.0,
                    ui::Hot::Back,
                    false,
                );
            }
            Phase::GameOver { t } => {
                ui::hot(Rect::new(0.0, 0.0, W, H), ui::Hot::Confirm);
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

    pub(super) fn draw_credits(&self, t: f32) {
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
}
