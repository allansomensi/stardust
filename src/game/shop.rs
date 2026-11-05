//! The shop between sectors.

use super::*;

impl Game {
    pub(super) fn roll_offers(&mut self) {
        let mut pool: Vec<Item> = vec![
            Item::Upgrade,
            Item::Thruster,
            Item::Trigger,
            Item::Magnet,
            Item::Insurance,
            Item::EnergyCell,
            Item::Amulet,
            Item::Crystallizer,
            Item::MysteryCard,
            Item::SideCannons,
            Item::LaserSight,
            Item::Capacitor,
            Item::MysteryBox,
            Item::MysteryBox,
        ];
        if self.cfg.rank >= 5 {
            pool.push(Item::Drone);
        }
        if self.cfg.rank >= 11 {
            pool.extend([Item::Bank, Item::Radar, Item::TreasureMap, Item::Mercenary]);
        }
        for w in self.weapon_pool() {
            if w != self.player.weapon {
                pool.push(Item::Weapon(w));
            }
        }
        pool.retain(|i| self.shop_block(*i).is_none());
        // Always two weapons for sale, preferably stronger than the current one.
        let cur = self.player.weapon.tier();
        let mut weapons: Vec<Item> = pool
            .iter()
            .copied()
            .filter(|i| matches!(i, Item::Weapon(w) if w.tier() >= cur))
            .collect();
        if weapons.len() < 2 {
            weapons = pool
                .iter()
                .copied()
                .filter(|i| matches!(i, Item::Weapon(_)))
                .collect();
        }
        let mut offers: Vec<Item> = vec![];
        while offers.len() < 2 && !weapons.is_empty() {
            offers.push(weapons.swap_remove(gen_range(0, weapons.len())));
        }
        pool.retain(|i| !matches!(i, Item::Weapon(_)));
        while offers.len() < 4 && !pool.is_empty() {
            let it = pool.swap_remove(gen_range(0, pool.len()));
            offers.push(it);
        }
        offers.sort_by_key(|i| match i {
            Item::Weapon(w) => w.tier() as i32,
            _ => 10,
        });
        self.shop_offers = vec![Item::ExtraLife, Item::Bomb, Item::Shield];
        if !self.arcade.autofire {
            self.shop_offers.push(Item::Autofire);
        }
        if self.arcade.core < 6 {
            self.shop_offers.push(Item::DamageCore);
        }
        self.shop_offers.extend(offers);
    }

    pub(super) fn bought(&self, it: Item) -> u32 {
        self.shop_buys
            .iter()
            .find(|(k, _)| *k == it.key())
            .map(|(_, n)| *n)
            .unwrap_or(0)
    }

    pub(super) fn price(&self, it: Item) -> u32 {
        let mut p = it.price() as f32 * (1.0 + 0.75 * self.bought(it) as f32) * self.mods.shop_mul;
        if self.grudge {
            p *= 1.3;
        }
        if self.cfg.daily == Some(Daily::DoubleCrystals) {
            p *= 1.5;
        }
        p.round() as u32
    }

    pub(super) fn reroll_price(&self) -> u32 {
        if self.mods.free_reroll && !self.free_reroll_used {
            0
        } else {
            15 + 10 * self.rerolls
        }
    }

    pub(super) fn shop_block(&self, it: Item) -> Option<&'static str> {
        let p = &self.player;
        match it {
            Item::ExtraLife if p.lives >= 5 => Some("MAX"),
            Item::Bomb if p.bombs >= 5 => Some("MAX"),
            Item::Upgrade if p.level >= 5 => Some("MAX"),
            Item::Shield if p.shield => Some("OWNED"),
            Item::Thruster if p.speed_lvl >= 3 => Some("MAX"),
            Item::Trigger if p.fire_lvl >= 3 => Some("MAX"),
            Item::Magnet if p.magnet_lvl >= 3 || self.mods.magnet_all => Some("MAX"),
            Item::Drone if self.drones_total() >= 3 => Some("MAX"),
            Item::SideCannons if self.side_cannons => Some("OWNED"),
            Item::Insurance if self.insurance || self.mods.keep_level => Some("OWNED"),
            Item::Bank if self.bank => Some("OWNED"),
            Item::Crystallizer if self.crystalizer => Some("OWNED"),
            Item::Amulet if self.bought(Item::Amulet) >= 2 => Some("MAX"),
            Item::Radar if self.bought(Item::Radar) >= 1 => Some("OWNED"),
            Item::TreasureMap if self.treasure_map => Some("OWNED"),
            Item::Mercenary if self.ally_sectors > 0 => Some("OWNED"),
            Item::Weapon(w) if w == p.weapon => Some("EQUIPPED"),
            Item::Autofire if self.arcade.autofire => Some("OWNED"),
            Item::LaserSight if self.arcade.sight => Some("OWNED"),
            Item::Capacitor if self.arcade.capacitor => Some("OWNED"),
            Item::DamageCore if self.arcade.core >= 6 => Some("MAX"),
            _ => None,
        }
    }

    pub(super) fn update_shop(&mut self, inp: &Input, au: &mut Audio) -> Phase {
        let n = self.shop_offers.len() + 2;
        if let Some(h) = inp.hover
            && h < n
            && h != self.shop_sel
        {
            self.shop_sel = h;
            au.play_vol(Sfx::MenuMove, 0.5);
        }
        // Mouse: a click on the selected row buys it.
        let mouse_buy = inp.select_click == Some(self.shop_sel);
        if inp.up_p {
            self.shop_sel = (self.shop_sel + n - 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.down_p {
            self.shop_sel = (self.shop_sel + 1) % n;
            au.play(Sfx::MenuMove);
        }
        if inp.back {
            self.shop_sel = n - 1;
            au.play(Sfx::MenuMove);
        }
        if inp.confirm || mouse_buy {
            let i = self.shop_sel;
            if i == n - 1 {
                au.play(Sfx::MenuOk);
                return self.leave_shop();
            }
            if i == n - 2 {
                let price = self.reroll_price();
                if (self.crystals.floor() as u32) < price {
                    au.play(Sfx::Deny);
                } else {
                    if price == 0 {
                        self.free_reroll_used = true;
                    } else {
                        self.rerolls += 1;
                    }
                    self.crystals -= price as f32;
                    self.roll_offers();
                    au.play(Sfx::Portal);
                }
                return Phase::Shop;
            }
            let it = self.shop_offers[i];
            let price = self.price(it);
            if self.shop_block(it).is_some() || (self.crystals.floor() as u32) < price {
                au.play(Sfx::Deny);
            } else {
                self.crystals -= price as f32;
                let key = it.key();
                match self.shop_buys.iter_mut().find(|(k, _)| *k == key) {
                    Some((_, n)) => *n += 1,
                    None => self.shop_buys.push((key, 1)),
                }
                au.play(Sfx::Buy);
                let p = &mut self.player;
                match it {
                    Item::ExtraLife => p.lives += 1,
                    Item::Bomb => p.bombs += 1,
                    Item::Shield => p.shield = true,
                    Item::Upgrade => p.level += 1,
                    Item::Weapon(w) => encounters::pick_new_weapon(p, w),
                    Item::Thruster => p.speed_lvl += 1,
                    Item::Trigger => p.fire_lvl += 1,
                    Item::Magnet => p.magnet_lvl += 1,
                    Item::Drone => self.drones_bought += 1,
                    Item::SideCannons => self.side_cannons = true,
                    Item::Insurance => self.insurance = true,
                    Item::EnergyCell => {
                        p.meter = 1.0;
                        self.mods.meter_mul += 0.2;
                    }
                    Item::Bank => self.bank = true,
                    Item::Radar => self.mods.card_mul += 0.6,
                    Item::Amulet => self.mods.powerup_mul += 0.3,
                    Item::Crystallizer => self.crystalizer = true,
                    Item::TreasureMap => self.treasure_map = true,
                    Item::Mercenary => self.ally_sectors = 2,
                    Item::MysteryCard => {
                        let c = cards::roll(0);
                        self.give_card(c, au);
                    }
                    Item::Autofire => self.arcade.autofire = true,
                    Item::LaserSight => self.arcade.sight = true,
                    Item::Capacitor => self.arcade.capacitor = true,
                    Item::MysteryBox => self.open_box(au),
                    Item::DamageCore => self.arcade.core += 1,
                }
                self.mirror_purchase(it);
                self.check_mastery();
            }
        }
        Phase::Shop
    }

    pub(super) fn leave_shop(&mut self) -> Phase {
        self.rocks.clear();
        self.wave += 1;
        if self.cfg.mode == Mode::Campaign
            && self.cycle == 0
            && self.sector() == 4
            && self.deaths == 0
            && !self.secret_done
        {
            return Phase::Portal { t: 0.0 };
        }
        Phase::Intro { t: 0.0 }
    }

    pub(super) fn draw_shop(&self, sp: &Sprites) {
        draw_rectangle(0.0, 0.0, W, H, Color::new(0.0, 0.0, 0.05, 0.85));
        ui::text_outline(
            "SHOP",
            W / 2.0,
            60.0,
            44.0,
            GOLD,
            Color::from_rgba(50, 20, 0, 255),
        );
        let sub = if self.grudge {
            "The Merchant glares at you: prices +30%"
        } else {
            "Spend your crystals before the next sector"
        };
        ui::text_c(
            sub,
            W / 2.0,
            84.0,
            14.0,
            if self.grudge { RED } else { DIM },
        );
        sp.crystal[((self.time * 4.0) as usize) % 2].draw(
            vec2(W / 2.0 - 40.0, 108.0),
            3.0,
            0.0,
            WHITE,
            false,
        );
        ui::text(
            &format!("{}", self.crystals.floor() as u32),
            W / 2.0 - 24.0,
            116.0,
            26.0,
            CYAN,
        );

        let y0 = 134.0;
        let row_h = 46.0;
        for (i, it) in self.shop_offers.iter().enumerate() {
            let y = y0 + i as f32 * row_h;
            let sel = self.shop_sel == i;
            let blocked = self.shop_block(*it);
            let price = self.price(*it);
            let afford = self.crystals.floor() as u32 >= price;
            let bg = if sel {
                Color::new(0.3, 0.25, 0.05, 0.6)
            } else {
                Color::new(0.1, 0.1, 0.2, 0.5)
            };
            draw_rectangle(20.0, y, W - 40.0, row_h - 5.0, bg);
            ui::hot(
                Rect::new(20.0, y, W - 40.0, row_h - 5.0),
                ui::Hot::Select(i),
            );
            if sel {
                draw_rectangle_lines(20.0, y, W - 40.0, row_h - 5.0, 2.0, GOLD);
            }
            if i == 3 {
                draw_line(
                    24.0,
                    y - 3.0,
                    W - 24.0,
                    y - 3.0,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.15),
                );
            }
            match it {
                Item::Weapon(w) => crate::icons::weapon_badge(*w, vec2(44.0, y + 20.0), 14.0, 1.0),
                _ => sp
                    .icon(it.icon())
                    .draw(vec2(44.0, y + 20.0), 2.6, 0.0, WHITE, false),
            }
            let name_c = if blocked.is_some() { DIM } else { WHITE };
            ui::text(&it.name(), 68.0, y + 18.0, 17.0, name_c);
            ui::text(it.desc(), 68.0, y + 35.0, 12.0, DIM);
            match blocked {
                Some(why) => ui::text_r(why, W - 32.0, y + 27.0, 15.0, DIM),
                None => ui::text_r(
                    &format!("{price}"),
                    W - 32.0,
                    y + 27.0,
                    21.0,
                    if afford { CYAN } else { RED },
                ),
            }
        }
        let n = self.shop_offers.len();
        let y = y0 + n as f32 * row_h + 8.0;
        let rp = self.reroll_price();
        let label = if rp == 0 {
            "REROLL OFFERS (FREE)".to_string()
        } else {
            format!("REROLL OFFERS ({rp})")
        };
        let c = if self.shop_sel == n { GOLD } else { DIM };
        ui::text_c(&label, W / 2.0, y + 14.0, 18.0, c);
        ui::hot(
            Rect::new(60.0, y - 6.0, W - 120.0, 28.0),
            ui::Hot::Select(n),
        );
        ui::hot(
            Rect::new(40.0, y + 26.0, W - 80.0, 32.0),
            ui::Hot::Select(n + 1),
        );
        let next = match self.cfg.mode {
            Mode::BossRush => format!("FACE THE BOSS {}", self.wave + 1),
            _ => format!("DEPART FOR SECTOR {}", self.absolute_sector() + 1),
        };
        if self.shop_sel == n + 1 {
            ui::text_c(&format!("> {next} <"), W / 2.0, y + 48.0, 22.0, GOLD);
        } else {
            ui::text_c(&next, W / 2.0, y + 48.0, 20.0, DIM);
        }
        let p = &self.player;
        let status = format!(
            "SHIPS {}  BOMBS {}  {} LV{}  DRONES {}",
            p.lives,
            p.bombs,
            p.weapon.name(),
            p.level,
            self.drones_total()
        );
        ui::text_c(&status, W / 2.0, H - 30.0, 13.0, DIM);
        let power = format!(
            "FIREPOWER x{:.2}   (weapon {} x{:.2} · cores {} x{:.2})",
            self.power_mul(),
            "*".repeat(p.weapon.tier() as usize),
            p.weapon.tier_mul(),
            self.arcade.core,
            self.core_mul()
        );
        ui::text_c(&power, W / 2.0, H - 12.0, 13.0, GOLD);
    }
}
