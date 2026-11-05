//! Hits: player shots against enemies and bosses, hazards against the ship, deaths.

use super::*;

impl Game {
    pub(super) fn collisions(&mut self, au: &mut Audio) {
        let mut landed = 0u32;
        let time = self.time;
        let mut pirate = false;
        let mut charged5 = false;
        self.rail_hits(au);
        for b in &mut self.bullets {
            if b.life <= 0.0 || b.kind == BKind::Hole {
                continue;
            }
            // Plasma burns through six enemies, then fades.
            if b.kind == BKind::Plasma && b.hit.iter().all(|h| *h != 0) {
                b.life = 0.0;
                continue;
            }
            for e in &mut self.enemies {
                if !e.active() || e.hp <= 0 || e.pos.y < -10.0 || b.hit.contains(&e.id) {
                    continue;
                }
                if e.alpha(time) < 0.5 {
                    continue;
                }
                let r = e.kind.radius()
                    + match b.kind {
                        BKind::Laser => 2.0,
                        BKind::Plasma => 9.0,
                        BKind::Fireball => 7.0,
                        BKind::Meteor => 6.0,
                        _ => 3.0,
                    };
                if b.pos.distance(e.pos) < r {
                    if vanguard::mirror_open(e, time) {
                        Game::reflect_shot(
                            &mut self.ebullets,
                            b.pos,
                            b.vel,
                            self.cfg.diff.bullet_speed(),
                        );
                        self.fx.sparks(
                            b.pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(170, 250, 255, 255),
                            5,
                        );
                        b.life = 0.0;
                        break;
                    }
                    landed += 1;
                    if shield_blocks(e, b.pos) {
                        e.shield -= 1;
                        e.flash = 0.05;
                        au.play_vol(Sfx::Clang, 0.4);
                        self.fx.sparks(
                            b.pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(180, 200, 230, 255),
                            4,
                        );
                        if e.shield <= 0 {
                            self.fx.burst(e.pos + vec2(0.0, 12.0), GRAY, 12, 120.0);
                        }
                        if b.kind != BKind::Disc {
                            b.life = 0.0;
                        }
                        break;
                    }
                    let dmg = b.dmg.floor() as i32
                        + if gen_range(0.0, 1.0) < b.dmg.fract() {
                            1
                        } else {
                            0
                        };
                    e.hp -= if e.frozen > 0.0 { dmg * 2 } else { dmg };
                    if b.kind == BKind::Charge && e.hp <= 0 {
                        b.base_x += 1.0;
                        if b.base_x >= 5.0 {
                            charged5 = true;
                        }
                    }
                    e.flash = 0.07;
                    e.damaged = true;
                    e.last_weapon = b.weapon;
                    if b.kind == BKind::Ice {
                        if e.frozen <= 0.0 {
                            self.frozen_count += 1;
                        }
                        e.frozen = e.frozen.max(1.6 * self.mods.freeze_mul);
                    }
                    if e.kind == Kind::Merchant {
                        pirate = true;
                    }
                    if e.hp > 0 {
                        au.play_vol(Sfx::Clang, 0.4);
                        self.fx.sparks(b.pos, -b.vel.normalize_or_zero(), WHITE, 4);
                    }
                    if b.pierce > 0 {
                        if b.pierce != 255 {
                            b.pierce -= 1;
                        }
                        if let Some(slot) = b.hit.iter_mut().find(|h| **h == 0) {
                            *slot = e.id;
                        }
                    } else {
                        b.life = 0.0;
                        break;
                    }
                }
            }
        }
        if charged5 {
            self.ach(ach::CHARGED);
        }
        if pirate && !self.grudge {
            self.grudge = true;
            self.ach(ach::PIRATE);
            self.banner = Some((
                "SPACE PIRATE".into(),
                "The Merchant will never forget this.".into(),
                3.0,
            ));
        }
        // boss
        if let Some(boss) = &mut self.boss
            && boss.targetable()
        {
            for b in &mut self.bullets {
                if b.life <= 0.0 || b.kind == BKind::Hole {
                    continue;
                }
                if let Some(t) = boss.hit_test(b.pos) {
                    let mul = match b.kind {
                        BKind::Laser => 0.8,
                        BKind::Flame => 0.9,
                        BKind::Disc => 0.5,
                        _ => 1.0,
                    };
                    let res = boss.damage(t, b.dmg * mul);
                    landed += 1;
                    if b.kind == BKind::Disc {
                        if b.t < 0.35 {
                            b.vel = Vec2::ZERO;
                            b.t = 0.35;
                        }
                    } else {
                        b.life = 0.0;
                    }
                    if res.blocked {
                        self.fx.sparks(
                            b.pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(180, 150, 255, 255),
                            3,
                        );
                        au.play_vol(Sfx::Clang, 0.3);
                        continue;
                    }
                    self.fx.sparks(
                        b.pos,
                        vec2(0.0, 1.0),
                        Color::from_rgba(255, 220, 150, 255),
                        2,
                    );
                    if res.part_destroyed {
                        self.enc.parts += 1;
                        au.play(Sfx::Boom);
                        self.fx
                            .explosion(res.part_pos, Color::from_rgba(255, 160, 80, 255), true);
                        self.score += 5000;
                        self.fx.text(res.part_pos, "+5000", GOLD, 20.0);
                        self.hitstop = 0.05;
                        for _ in 0..3 {
                            self.drops.push(Drop {
                                pos: res.part_pos,
                                vel: vec2(gen_range(-70.0, 70.0), gen_range(-140.0, -40.0)),
                                kind: DropKind::Crystal(1),
                                age: 0.0,
                            });
                        }
                    }
                    if res.killed {
                        au.play(Sfx::BigBoom);
                        self.fx.flash = 0.8;
                        self.fx.flash_color = WHITE;
                        self.ebullets.clear();
                        self.hitstop = 0.25;
                    }
                }
            }
        }
        // rocks
        for b in &mut self.bullets {
            if b.life <= 0.0 {
                continue;
            }
            for r in &mut self.rocks {
                let rad = if r.solid {
                    28.0
                } else if r.big {
                    24.0
                } else {
                    13.0
                };
                if r.hp > 0 && b.pos.distance(r.pos) < rad {
                    if !r.solid {
                        r.hp -= b.dmg.ceil() as i32;
                    }
                    r.flash = 0.06;
                    b.life = 0.0;
                    self.fx.sparks(
                        b.pos,
                        vec2(0.0, 1.0),
                        Color::from_rgba(200, 160, 120, 255),
                        3,
                    );
                    break;
                }
            }
        }
        self.sector_landed += landed;
        self.expire_bullets();
        self.bullets.retain(|b| b.life > 0.0);

        let mut new_rocks = vec![];
        let mut rock_drops = vec![];
        for r in &mut self.rocks {
            if r.hp <= 0 && r.hp > -100 {
                r.hp = -1000;
                self.fx
                    .explosion(r.pos, Color::from_rgba(170, 120, 80, 255), false);
                self.score += if r.big { 200 } else { 100 };
                for _ in 0..(if r.big { 3 } else { 2 }) {
                    rock_drops.push(r.pos);
                }
                if r.big {
                    for s in [-1.0, 1.0] {
                        new_rocks.push(Rock {
                            pos: r.pos,
                            vel: vec2(s * 90.0, r.vel.y * 0.8),
                            hp: 2,
                            big: false,
                            solid: false,
                            rot: 0.0,
                            vr: s * 3.0,
                            variant: r.variant,
                            flash: 0.0,
                        });
                    }
                }
            }
        }
        if !rock_drops.is_empty() {
            au.play_vol(Sfx::Pop, 0.7);
        }
        for p in rock_drops {
            self.spawn_drop(p, DropKind::Crystal(1));
        }
        self.rocks.extend(new_rocks);

        self.player_hazards(au);
        self.with_p2(|g| g.player_hazards(au));
    }

    /// Enemy shots, bodies, rocks, lightning and bosses against the ship in `self.player`.
    pub(super) fn player_hazards(&mut self, au: &mut Audio) {
        let time = self.time;
        if !self.player.alive() {
            return;
        }
        let pp = self.player.pos;
        let hr = self.hit_r();
        let mut grazes = 0;
        let mut webbed = false;
        let mut hurt = false;
        let invuln = self.player.invuln > 0.0;
        self.ebullets.retain_mut(|b| {
            let d = b.pos.distance(pp);
            if b.style == 3 {
                if d < hr + b.r && !invuln {
                    webbed = true;
                    return false;
                }
                return true;
            }
            if d < hr + b.r - 1.0 && !invuln {
                hurt = true;
            } else if !b.grazed && d < hr + b.r + 16.0 {
                b.grazed = true;
                grazes += 1;
            }
            true
        });
        if webbed {
            self.web_t = 2.5;
            au.play(Sfx::Web);
            self.fx.burst(pp, WHITE, 10, 90.0);
        }
        if grazes > 0 && !self.classic() {
            self.grazes += grazes;
            let gm = self.mods.graze_mul;
            self.player.meter =
                (self.player.meter + 0.012 * gm * grazes as f32 * self.mods.meter_mul).min(1.0);
            self.score += (20.0 * gm) as u32 * grazes;
            au.play_vol(Sfx::Graze, 0.6);
            self.fx
                .sparks(pp, vec2(0.0, -1.0), Color::new(1.0, 1.0, 0.7, 1.0), 2);
            for _ in 0..grazes {
                self.events.push(Event::Graze);
            }
            if self.grazes >= 100 {
                self.ach(ach::CLOSE_SHAVE);
            }
        }
        if invuln {
            if self.star_t > 0.0 {
                for e in &mut self.enemies {
                    if e.active() && !e.harmless && e.pos.distance(pp) < e.kind.radius() + 10.0 {
                        e.hp = 0;
                    }
                }
            }
            return;
        }
        if !hurt {
            for e in &mut self.enemies {
                if e.active()
                    && !e.harmless
                    && e.hp > 0
                    && e.alpha(time) > 0.3
                    && e.pos.distance(pp) < e.kind.radius() + hr + 2.0
                {
                    e.hp = 0;
                    hurt = true;
                    break;
                }
            }
        }
        if !hurt {
            for r in &mut self.rocks {
                let rad = if r.solid {
                    26.0
                } else if r.big {
                    20.0
                } else {
                    11.0
                };
                if r.hp > 0 && r.pos.distance(pp) < rad + hr {
                    if !r.solid {
                        r.hp = 0;
                    }
                    hurt = true;
                    break;
                }
            }
        }
        if !hurt {
            for s in &self.strikes {
                if s.t >= 1.0 && (s.x - pp.x).abs() < 16.0 + hr {
                    hurt = true;
                }
            }
        }
        if !hurt
            && let Some(b) = &self.boss
            && b.touches_player(pp, hr)
        {
            hurt = true;
        }
        if !hurt && self.signal_touches(pp, hr) {
            hurt = true;
        }
        if hurt {
            self.hurt_player(au);
        }
    }

    pub(super) fn hurt_player(&mut self, au: &mut Audio) {
        self.enc.nohit_t = 0.0;
        self.rift_on_hit();
        let pp = self.player.pos;
        self.sector_hits += 1;
        self.boss_hits += 1;
        if self.mods.counter {
            for e in &mut self.enemies {
                if e.active() && !e.harmless {
                    e.hp -= 3;
                    e.flash = 0.15;
                }
            }
            self.fx
                .ring(pp, 600.0, 0.6, Color::new(1.0, 0.4, 0.3, 1.0), 4.0);
        }
        if self.player.shield {
            self.player.shield = false;
            self.events.push(Event::Rumble(Some(self.seat()), 0.5, 200));
            self.player.invuln = 1.5;
            au.play(Sfx::Shield);
            self.fx.ring(pp, 300.0, 0.4, CYAN, 4.0);
            self.fx.burst(pp, CYAN, 30, 220.0);
            self.ebullets.retain(|b| b.pos.distance(pp) > 90.0);
            return;
        }
        if self.mods.last_breath {
            self.detonate(au, true);
        }
        au.play(Sfx::PlayerDie);
        self.events.push(Event::Rumble(Some(self.seat()), 1.0, 450));
        self.fx.explosion(pp, CYAN, true);
        self.fx.shake = 0.9;
        self.hitstop = 0.18;
        self.deaths += 1;
        self.combo = 0;
        self.overdrive_t = 0.0;
        self.wall_t = 0.0;
        self.cannon_t = 0.0;
        let keep = self.mods.keep_level || self.insurance;
        if self.insurance && !self.mods.keep_level {
            self.insurance = false;
        }
        let p = &mut self.player;
        p.lives -= 1;
        if !keep {
            p.level = (p.level - 1).max(1);
        }
        self.rift_on_death();
        let p = &mut self.player;
        p.dead_t = 1.4;
        if p.lives <= 0 && self.mods.phoenix && !p.phoenix_used {
            p.phoenix_used = true;
            p.lives = 1;
            self.banner = Some(("PHOENIX!".into(), "You rose from the ashes.".into(), 2.5));
        }
        self.ebullets.clear();
        self.strikes.clear();
    }

    pub(super) fn process_deaths(&mut self, au: &mut Audio) {
        let mut i = 0;
        let mut pops = 0;
        while i < self.enemies.len() {
            let e = &self.enemies[i];
            if e.dead && e.hp > 0 {
                let e = self.enemies.swap_remove(i);
                let g = e.group as usize;
                if e.bonus && g < self.groups.len() {
                    self.groups[g].0 = 0;
                }
                if e.kind == Kind::Merchant && !e.damaged {
                    self.events.push(Event::MerchantSpared);
                }
                continue;
            }
            if e.hp > 0 {
                if e.kind == Kind::Merchant
                    && !e.damaged
                    && !e.gifted
                    && let EState::Flyby { t, .. } = e.state
                    && t > 0.8
                {
                    let pos = e.pos;
                    self.enemies[i].gifted = true;
                    self.spawn_drop(pos, DropKind::Power(PowerKind::random()));
                    self.drop_log(10, pos);
                    for _ in 0..4 {
                        self.spawn_drop(pos, DropKind::Crystal(5));
                    }
                    self.banner = Some((
                        "MERCHANT'S GIFT".into(),
                        "Thanks for your kindness, pilot!".into(),
                        2.5,
                    ));
                    au.play(Sfx::Buy);
                }
                i += 1;
                continue;
            }
            let e = self.enemies.swap_remove(i);
            pops += 1;
            self.kills += 1;
            self.combo += 1;
            self.best_combo = self.best_combo.max(self.combo);
            self.combo_t = 1.1 * self.mods.combo_mul;
            if self.combo == 50 {
                self.ach(ach::COMBO50);
            }
            if self.combo == 100 {
                self.ach(ach::COMBO100);
            }
            if self.combo.is_multiple_of(10) && self.multiplier() > 1 {
                au.play(Sfx::Combo);
                self.fx.text(
                    e.pos - vec2(0.0, 20.0),
                    format!("COMBO x{}", self.multiplier()),
                    PINK,
                    22.0,
                );
            }
            self.events.push(Event::Kill(e.kind, e.last_weapon));
            self.on_kill(&e, au);
            self.rift_on_kill(&e);
            self.arcade_on_kill(&e);
            self.vanguard_on_kill(&e);
            self.seen(crate::progress::lore_of_kind(e.kind.index()));
            if !self.classic() {
                self.player.meter = (self.player.meter + 0.025 * self.mods.meter_mul).min(1.0);
            }
            let dive = e.diving();
            let base = e.kind.points() * if dive { self.mods.dive_mul } else { 1 };
            let pts = self.add_score(base);
            if pts > 0 {
                self.fx.text(
                    e.pos,
                    pts.to_string(),
                    if dive {
                        GOLD
                    } else {
                        Color::new(1.0, 1.0, 0.8, 1.0)
                    },
                    16.0,
                );
            }
            self.fx.explosion(
                e.pos,
                e.kind.color(),
                matches!(
                    e.kind,
                    Kind::Carrier | Kind::Golden | Kind::Cow | Kind::Merchant
                ),
            );
            match e.kind {
                Kind::Golden => {
                    self.ach(ach::FOOLS_GOLD);
                    if !e.bonus {
                        au.play(Sfx::Fanfare);
                        for _ in 0..8 {
                            self.spawn_drop(e.pos, DropKind::Crystal(5));
                        }
                        self.banner = Some((
                            "GOLDEN SCARAB!".into(),
                            "A fortune in crystals!".into(),
                            2.0,
                        ));
                    }
                }
                Kind::Cow => {
                    au.play(Sfx::Moo);
                    self.ach(ach::MOO);
                    self.fx.text(e.pos - vec2(0.0, 30.0), "MUUUU!", WHITE, 30.0);
                    self.spawn_drop(e.pos, DropKind::Card(cards::id::COW));
                }
                Kind::Merchant => {
                    for _ in 0..12 {
                        self.spawn_drop(e.pos, DropKind::Crystal(5));
                    }
                }
                _ => {}
            }
            if e.bonus {
                self.challenge_hits += 1;
                let g = e.group as usize;
                if g < self.groups.len() && self.groups[g].0 > 0 {
                    self.groups[g].1 += 1;
                    if self.groups[g].1 == self.groups[g].0 {
                        let b = 1000 * (1 + self.cycle) * self.mods.bonus_mul;
                        self.score += b;
                        self.fx
                            .text(e.pos - vec2(0.0, 24.0), format!("GROUP! +{b}"), GREEN, 22.0);
                        au.play(Sfx::MenuOk);
                    }
                }
                if e.kind == Kind::Golden {
                    for _ in 0..2 {
                        self.spawn_drop(e.pos, DropKind::Crystal(1));
                    }
                }
                continue;
            }
            if e.kind == Kind::Splitter {
                for s in [-1.0, 1.0] {
                    self.spawn(
                        Kind::Mini,
                        e.col,
                        e.row,
                        EState::Free {
                            vel: vec2(s * 120.0, 60.0),
                        },
                        98,
                        false,
                    );
                    if let Some(m) = self.enemies.last_mut() {
                        m.pos = e.pos + vec2(s * 8.0, 0.0);
                    }
                }
            }
            if self.classic() || matches!(e.kind, Kind::Golden | Kind::Cow | Kind::Merchant) {
                continue;
            }
            let mut n_crystals = match e.kind {
                Kind::Beetle | Kind::Carrier => 3,
                Kind::Gunner | Kind::Splitter | Kind::Shieldbearer => 2,
                _ => (gen_range(0.0, 1.0) < 0.45) as i32,
            };
            if self.crystalizer && gen_range(0.0, 1.0) < 0.3 {
                n_crystals += 1;
            }
            let base = 1 + (self.stage().tier.saturating_sub(1) / 3).min(3);
            for _ in 0..n_crystals {
                let v = if gen_range(0, 20) == 0 {
                    5 * base
                } else {
                    base
                };
                self.spawn_drop(e.pos, DropKind::Crystal(v));
            }
            let rain = if self.cfg.daily == Some(Daily::CapsuleRain) {
                3.0
            } else {
                1.0
            };
            // Never more than a couple of capsules, power-ups and cards at once: the screen
            // must stay readable however many drop boosts were bought.
            let (caps, powers, cards_out) =
                self.drops.iter().fold((0, 0, 0), |n, d| match d.kind {
                    DropKind::Capsule { .. } => (n.0 + 1, n.1, n.2),
                    DropKind::Power(_) => (n.0, n.1 + 1, n.2),
                    DropKind::Card(_) => (n.0, n.1, n.2 + 1),
                    _ => n,
                });
            self.capsule_pity += 1;
            let cap_chance = if e.kind == Kind::Beetle { 0.06 } else { 0.011 } * rain;
            let power_mul = self.mods.powerup_mul.min(MAX_DROP_BOOST);
            if caps < 2 && (self.capsule_pity > 70 || gen_range(0.0, 1.0) < cap_chance) {
                self.capsule_pity = 0;
                let cap = self.make_capsule();
                self.spawn_drop(e.pos, cap);
            } else if powers < 2 && gen_range(0.0, 1.0) < 0.006 * power_mul * rain {
                self.spawn_drop(e.pos, DropKind::Power(PowerKind::random()));
            }
            let card_chance = if e.kind == Kind::Beetle {
                0.006
            } else {
                0.0018
            } * self.mods.card_mul.min(MAX_DROP_BOOST);
            if cards_out < 1 && gen_range(0.0, 1.0) < card_chance {
                self.spawn_drop(e.pos, DropKind::Card(cards::roll(0)));
            }
        }
        if pops > 0 {
            au.play_vol(Sfx::Pop, 0.7);
            self.fx.shake = self.fx.shake.max(0.08);
        }
    }
}
