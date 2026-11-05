//! The run's state machine: one `update` per frame, moving between phases.

use super::*;

impl Game {
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
                    if self.signal_ready() {
                        self.enemies.clear();
                        self.start_signal(au);
                        Phase::Playing
                    } else if self.cfg.mode == Mode::Survival {
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
                if let Some(h) = inp.hover
                    && can_continue
                {
                    sel = h.min(1);
                }
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

    pub(super) fn collect_all_drops(&mut self, au: &mut Audio) {
        let drops: Vec<Drop> = self.drops.drain(..).collect();
        for d in drops {
            self.collect(d, au);
        }
    }

    pub(super) fn give_card(&mut self, c: usize, au: &mut Audio) {
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
}
