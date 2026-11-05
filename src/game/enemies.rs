//! Enemies, bosses, stage hazards and every bullet in flight.

use super::*;

impl Game {
    pub(super) fn update_enemies(&mut self, dt: f32, au: &mut Audio) {
        let diff = self.diff();
        let player = self.player.pos;
        let alive = self.player.alive();
        let bspeed = (195.0 + diff * 15.0) * self.bullet_mul();
        let mut new_bullets: Vec<EBullet> = vec![];
        let mut spawns: Vec<(Vec2, Vec2)> = vec![];
        let mut fired = false;
        let dive_mul = self.fire_rate_enemy().sqrt();

        if self.wave_kind == WaveKind::Normal && alive && self.wave_age > 3.0 {
            self.dive_timer -= dt;
            let remaining = self.enemies.len();
            if self.dive_timer <= 0.0 {
                let mut t = gen_range(1.4, 2.8)
                    / (1.0 + diff * 0.18)
                    / dive_mul
                    / self.rage_mul()
                    / self.dive_rate();
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
                        let partners = self.dive_partners(i);
                        self.start_dive(i);
                        for j in partners {
                            if matches!(self.enemies[j].state, EState::Formation) {
                                self.start_dive(j);
                            }
                        }
                    }
                }
            }
        }

        let slots: Vec<Vec2> = self
            .enemies
            .iter()
            .map(|e| self.slot(e.col, e.row))
            .collect();
        let formation_rate = 0.024 * (1.0 + diff * 0.3) * self.fire_rate_enemy() * self.rage_mul();
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

    pub(super) fn update_boss(&mut self, dt: f32, au: &mut Audio) {
        self.update_scenery(dt, au);
        self.update_boss_trait(dt);
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

    pub(super) fn spawn_boss(&mut self, au: &mut Audio) {
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
        if matches!(
            self.cfg.mode,
            Mode::Campaign | Mode::Daily | Mode::Expedition
        ) {
            self.boss_trait_begin();
        }
        if let Some(k) = crate::dev_env("STARDUST_SCENE").and_then(|v| v.parse::<usize>().ok()) {
            self.debug_scene(k);
        }
        au.music(Some(if final_boss || variant || self.enc.void {
            Music::Final
        } else {
            Music::Boss
        }));
    }

    pub(super) fn update_hazards(&mut self, dt: f32, au: &mut Audio) {
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

    pub(super) fn update_bullets(&mut self, dt: f32, wdt: f32) {
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
}
