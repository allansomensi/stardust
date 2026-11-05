//! Drops: crystals, capsules, power-ups, cards and what picking them up does.

use super::*;

impl Game {
    pub(super) fn spawn_drop(&mut self, pos: Vec2, kind: DropKind) {
        // Too many crystals on screen clutter the view: new ones merge into the ones already falling.
        if let DropKind::Crystal(v) = kind {
            let n = self
                .drops
                .iter()
                .filter(|d| matches!(d.kind, DropKind::Crystal(_)))
                .count();
            if n >= 24 {
                let mut best: Option<usize> = None;
                let mut bd = f32::MAX;
                for (i, d) in self.drops.iter().enumerate() {
                    if matches!(d.kind, DropKind::Crystal(_)) {
                        let dist = d.pos.distance(pos);
                        if dist < bd {
                            bd = dist;
                            best = Some(i);
                        }
                    }
                }
                if let Some(i) = best
                    && let DropKind::Crystal(c) = &mut self.drops[i].kind
                {
                    *c += v;
                    return;
                }
            }
        }
        let vel = match kind {
            DropKind::Crystal(_) => vec2(gen_range(-70.0, 70.0), gen_range(-140.0, -40.0)),
            _ => vec2(0.0, -60.0),
        };
        self.drops.push(Drop {
            pos,
            vel,
            kind,
            age: 0.0,
        });
    }

    pub(super) fn make_capsule(&self) -> DropKind {
        let pool = self.weapon_pool();
        let want = if self.mods.capsule_four { 4 } else { 3 };
        let mut list: Vec<Weapon> = vec![];
        let mut tries = 0;
        while list.len() < want.min(pool.len()) && tries < 40 {
            tries += 1;
            let w = pool[gen_range(0, pool.len())];
            if !list.contains(&w) {
                list.push(w);
            }
        }
        if list.is_empty() {
            list.push(Weapon::Blaster);
        }
        let mut arr = [list[0]; 4];
        for (i, w) in list.iter().enumerate() {
            arr[i] = *w;
        }
        DropKind::Capsule {
            weapons: arr,
            n: list.len(),
            idx: 0,
            t: 0.0,
        }
    }

    pub(super) fn update_drops(&mut self, dt: f32, au: &mut Audio) {
        let pp = self.player.pos;
        let alive = self.player.alive();
        let magnet = (70.0 + 35.0 * self.player.magnet_lvl as f32)
            * self.mods.magnet_mul
            * if self.ship_idx() == 7 { 1.5 } else { 1.0 };
        let all = self.mods.magnet_all || self.magnet_t > 0.0;
        let card_magnet = self.mods.card_magnet;
        let cap_magnet = self.mods.capsule_magnet;
        let cycle = if self.mods.capsule_four { 2.4 } else { 1.8 };
        let mut collected = vec![];
        for (i, d) in self.drops.iter_mut().enumerate() {
            d.age += dt;
            let pulled = match &d.kind {
                DropKind::Crystal(_) => {
                    alive && (d.pos.distance(pp) < magnet || all) && d.age > 0.25
                }
                DropKind::Card(_) | DropKind::Log(_) => alive && card_magnet && d.age > 0.6,
                DropKind::Capsule { .. }
                | DropKind::Power(_)
                | DropKind::Relic(_)
                | DropKind::Rift
                | DropKind::Letter(_) => alive && cap_magnet && d.age > 0.6,
                DropKind::Curse(..) => false,
            };
            if pulled {
                let target = (pp - d.pos).normalize_or_zero() * 560.0;
                d.vel += (target - d.vel) * (dt * 8.0).min(1.0);
            } else {
                match &mut d.kind {
                    DropKind::Crystal(_) => {
                        d.vel.y = (d.vel.y + 260.0 * dt).min(150.0);
                        d.vel.x *= 1.0 - (dt * 1.5).min(1.0);
                    }
                    DropKind::Capsule { n, idx, t, .. } => {
                        *t += dt;
                        if *t > cycle {
                            *t = 0.0;
                            *idx = (*idx + 1) % *n;
                        }
                        d.vel.y = (d.vel.y + 120.0 * dt).min(85.0);
                        d.vel.x = (d.age * 2.0).sin() * 30.0;
                    }
                    DropKind::Card(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(75.0);
                        d.vel.x = (d.age * 2.5).cos() * 40.0;
                    }
                    DropKind::Power(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(95.0);
                        d.vel.x = (d.age * 3.0).sin() * 20.0;
                    }
                    DropKind::Log(_) => {
                        d.vel.y = (d.vel.y + 120.0 * dt).min(60.0);
                        d.vel.x = (d.age * 2.0).cos() * 25.0;
                    }
                    other => rifts::drop_motion(other, &mut d.vel, d.age, d.pos, pp, dt),
                }
            }
            d.pos += d.vel * dt;
            if alive && d.pos.distance(pp) < 24.0 {
                collected.push(i);
            }
        }
        for &i in collected.iter().rev() {
            let d = self.drops.swap_remove(i);
            self.collect(d, au);
        }
        self.drops.retain(|d| d.pos.y < H + 30.0);
    }

    pub(super) fn collect(&mut self, d: Drop, au: &mut Audio) {
        let pp = self.player.pos;
        match d.kind {
            DropKind::Crystal(v) => {
                let mut amount = v as f32 * self.mods.crystal_mul * self.event_crystal_mul();
                self.enc.crystals += v;
                if self.cfg.daily == Some(Daily::DoubleCrystals) {
                    amount *= 2.0;
                }
                if self.cfg.mode == Mode::Survival {
                    self.score += (amount * 50.0) as u32;
                }
                self.crystals += amount;
                self.events.push(Event::Crystals(amount.round() as u32));
                if self.mods.crystal_bomb {
                    self.crystal_bomb_acc += amount;
                    if self.crystal_bomb_acc >= 100.0 {
                        self.crystal_bomb_acc -= 100.0;
                        self.player.bombs = (self.player.bombs + 1).min(9);
                        self.fx.text(pp - vec2(0.0, 30.0), "+1 BOMB", GOLD, 18.0);
                    }
                }
                au.play_vol(Sfx::Crystal, 0.5);
                self.fx.sparks(d.pos, vec2(0.0, -1.0), CYAN, 3);
            }
            DropKind::Capsule { weapons, idx, .. } => {
                let weapon = weapons[idx];
                self.enc.capsules += 1;
                let p = &mut self.player;
                if weapon == p.weapon {
                    if p.level < 5 {
                        p.level += 1;
                        self.fx.text(
                            pp - vec2(0.0, 40.0),
                            format!("{} LEVEL {}", weapon.name(), p.level),
                            weapon.color(),
                            20.0,
                        );
                    } else {
                        self.score += 2000;
                        self.fx.text(pp - vec2(0.0, 40.0), "+2000", GOLD, 20.0);
                    }
                } else {
                    encounters::pick_new_weapon(p, weapon);
                    self.fx
                        .text(pp - vec2(0.0, 40.0), weapon.name(), weapon.color(), 22.0);
                }
                self.check_mastery();
                au.play(Sfx::Powerup);
                self.fx.ring(pp, 200.0, 0.35, weapon.color(), 3.0);
            }
            DropKind::Card(c) => self.give_card(c, au),
            DropKind::Log(i) => self.collect_log(i, au),
            DropKind::Curse(c, _) => self.take_curse(c, au),
            DropKind::Relic(w) => self.take_relic(w, au),
            DropKind::Rift => self.start_minigame(au),
            DropKind::Letter(i) => self.take_letter(i, au),
            DropKind::Power(k) => {
                au.play(Sfx::Powerup);
                self.fx.ring(pp, 240.0, 0.4, k.color(), 4.0);
                self.fx
                    .text(pp - vec2(0.0, 40.0), k.name(), k.color(), 22.0);
                match k {
                    PowerKind::Frenzy => self.frenzy_t = 8.0,
                    PowerKind::Star => {
                        self.star_t = 6.0;
                        self.player.invuln = self.player.invuln.max(6.0);
                    }
                    PowerKind::Magnet => self.magnet_t = 10.0,
                    PowerKind::SlowMo => self.slowmo_t = 6.0,
                    PowerKind::ExtraLife => {
                        self.player.lives = (self.player.lives + 1).min(9);
                        au.play(Sfx::OneUp);
                    }
                    PowerKind::Bomb => self.player.bombs = (self.player.bombs + 1).min(9),
                    PowerKind::Shield => self.player.shield = true,
                    PowerKind::Crystals => {
                        for _ in 0..5 {
                            self.spawn_drop(pp - vec2(0.0, 30.0), DropKind::Crystal(5));
                        }
                    }
                }
            }
        }
    }

    pub(super) fn check_mastery(&mut self) {
        if self.player.level >= 5 {
            let wi = self.player.weapon.index();
            if !self.mastery_run[wi] {
                self.mastery_run[wi] = true;
                self.events.push(Event::Mastery(wi));
            }
        }
    }
}
