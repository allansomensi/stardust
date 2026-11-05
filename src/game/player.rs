//! The ship: movement, firing every weapon, abilities and bombs.

use super::*;

impl Game {
    pub(super) fn update_world(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
        // Time warp and slow motion only affect the world, not the ship.
        let mut wdt = dt;
        if self.timewarp_t > 0.0 {
            wdt *= 0.3;
        }
        if self.slowmo_t > 0.0 {
            wdt *= 0.6;
        }
        self.update_timers(dt);
        self.update_vanguard(wdt);
        self.update_salvo(wdt);
        let i2 = self.p2_input.clone();
        let (in1, in2) = if self.p_swapped {
            (&i2, inp)
        } else {
            (inp, &i2)
        };
        self.update_player(dt, in1, au);
        self.scene_clamp();
        self.with_p2(|g| {
            g.update_player(dt, in2, au);
            g.scene_clamp();
        });
        self.update_coop(dt, au);
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
        if self.dbg_signal.is_some()
            && self.wave_age > 1.0
            && let Some(ph) = self.dbg_signal.take()
        {
            self.enemies.clear();
            self.start_signal(au);
            self.debug_signal_phase(ph);
        }
        self.update_signal(wdt, au);
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

    pub(super) fn update_timers(&mut self, dt: f32) {
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

    pub(super) fn update_player(&mut self, dt: f32, inp: &Input, au: &mut Audio) {
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
                p.invuln = 2.0;
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
            self.events.push(Event::Rumble(Some(self.seat()), 0.7, 300));
            self.enc.bombs_sector += 1;
            self.detonate(au, false);
        }

        if inp.ability_p && self.duo_ready() {
            self.co.duo_req = true;
        } else if inp.ability_p && !self.classic() {
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

    pub(super) fn fire(&mut self, au: &mut Audio) {
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
            * daily_x2
            * if self.evolved() { 1.1 } else { 1.0 };
        let evo = self.evolved();
        let lv = self.effective_level();
        let dmg = ship.dmg * self.power_mul() * if evo { 1.3 } else { 1.0 };
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
                    _ if evo => &[-27.0, -18.0, -8.0, 0.0, 8.0, 18.0, 27.0],
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
                    b.pierce = if evo {
                        8
                    } else if lv >= 3 {
                        3
                    } else {
                        2
                    };
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
                    let n =
                        [1, 2, 2, 3, 4][(lv - 1).clamp(0, 4) as usize] + if evo { 2 } else { 0 };
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
                let t = arsenal::Trigger {
                    w: weapon,
                    lv,
                    dmg,
                    o,
                    ss,
                    evo,
                };
                cd = arsenal::fire(t, &mut out, au);
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
        let star_ship = matches!(self.ship_idx(), 3 | 12);
        let p = &mut self.player;
        p.cd = cd / rate;
        p.shots += 1;
        if (self.mods.star_fan || star_ship) && p.shots.is_multiple_of(6) {
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

    pub(super) fn lightning(&mut self, lv: i32, dmg: f32, au: &mut Audio) {
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

    pub(super) fn use_ability(&mut self, dir: Vec2, au: &mut Audio) {
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

    pub(super) fn update_abilities(&mut self, dt: f32, au: &mut Audio) {
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

    pub(super) fn detonate(&mut self, au: &mut Audio, free: bool) {
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
}
