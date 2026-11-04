//! Local two-player co-op. The second ship shares the run, the score, the crystals and
//! the shop; each ship has its own lives, weapon, bombs and ability bar.
//!
//! Most of the game logic is written for `self.player`. For the second ship, `with_p2`
//! temporarily swaps it into that slot, so the same code drives both ships.

use super::*;

impl Game {
    pub fn coop(&self) -> bool {
        self.cfg.coop
    }

    /// Runs `f` with the second player temporarily in the `player` slot.
    pub(super) fn with_p2(&mut self, f: impl FnOnce(&mut Game)) {
        let Some(mut p2) = self.p2.take() else {
            return;
        };
        std::mem::swap(&mut self.player, &mut p2);
        self.cur_p2 = true;
        f(self);
        self.cur_p2 = false;
        std::mem::swap(&mut self.player, &mut p2);
        self.p2 = Some(p2);
    }

    /// Which seat (0 = player one, 1 = player two) the ship in `self.player` belongs to.
    pub(super) fn seat(&self) -> usize {
        (self.cur_p2 != self.p_swapped) as usize
    }

    pub(super) fn spawn_x(&self) -> f32 {
        if !self.coop() {
            W / 2.0
        } else if self.seat() == 1 {
            W / 2.0 + 60.0
        } else {
            W / 2.0 - 60.0
        }
    }

    pub fn seat_skin(&self, seat: usize) -> usize {
        if seat == 0 {
            self.cfg.skin
        } else if self.cfg.skin == 1 {
            2
        } else {
            1
        }
    }

    /// Picks up drops touching the second ship (the main drop update only checks player one).
    pub(super) fn collect_near(&mut self, au: &mut Audio) {
        if !self.player.alive() {
            return;
        }
        let pp = self.player.pos;
        let mut got: Vec<usize> = self
            .drops
            .iter()
            .enumerate()
            .filter(|(_, d)| d.age > 0.1 && d.pos.distance(pp) < 24.0)
            .map(|(i, _)| i)
            .collect();
        got.sort_unstable();
        for i in got.into_iter().rev() {
            let d = self.drops.swap_remove(i);
            self.collect(d, au);
        }
    }

    /// The team is out only when every ship has run out of lives.
    pub(super) fn team_out(&self) -> bool {
        let out = |p: &Player| p.lives <= 0 && p.dead_t <= 0.0;
        out(&self.player) && self.p2.as_ref().map(out).unwrap_or(true)
    }

    /// When player one is out but player two still flies, swap the slots so enemies keep
    /// chasing a live ship and every system keeps working on `self.player`.
    pub(super) fn coop_rotate(&mut self) {
        let out = self.player.lives <= 0 && self.player.dead_t <= 0.0;
        if let Some(p2) = &mut self.p2
            && out
            && p2.lives > 0
        {
            std::mem::swap(&mut self.player, p2);
            self.p_swapped = !self.p_swapped;
        }
    }

    /// Continue screen: both ships come back.
    pub(super) fn coop_revive(&mut self) {
        let x = W / 2.0 + 60.0;
        if let Some(p2) = &mut self.p2 {
            p2.lives = 3;
            p2.level = 1;
            p2.bombs = p2.bombs.max(2);
            p2.dead_t = 0.01;
            p2.pos.x = x;
        }
    }

    /// Shop purchases that belong to a ship are given to both ships.
    pub(super) fn mirror_purchase(&mut self, it: Item) {
        let Some(p2) = &mut self.p2 else {
            return;
        };
        match it {
            Item::Weapon(w) => p2.weapon = w,
            Item::Upgrade => p2.level = (p2.level + 1).min(5),
            Item::ExtraLife => p2.lives += 1,
            Item::Bomb => p2.bombs += 1,
            Item::Shield => p2.shield = true,
            Item::Thruster => p2.speed_lvl += 1,
            Item::Trigger => p2.fire_lvl += 1,
            _ => {}
        }
    }

    pub(super) fn coop_draw(&self, off: Vec2, art: &Art) {
        let Some(p2) = &self.p2 else {
            return;
        };
        let seat2 = (!self.p_swapped) as usize;
        if p2.alive() {
            let p = p2.pos + off;
            let blink = p2.invuln > 0.0 && self.star_t <= 0.0 && (self.time * 18.0) as i32 % 2 == 0;
            if !blink {
                self.draw_ship_body(p2, self.seat_skin(seat2), p, art);
                self.draw_charge(p2, p);
            }
        }
        // seat tags above both ships
        let tags = [(&self.player, 1 - seat2), (p2, seat2)];
        for (pl, seat) in tags {
            if pl.alive() {
                let p = pl.pos + off;
                let c = if seat == 0 { CYAN } else { PINK };
                ui::text_c(
                    if seat == 0 { "P1" } else { "P2" },
                    p.x,
                    p.y + 38.0,
                    11.0,
                    c,
                );
            }
        }
    }

    /// Compact status line for both ships (lives, bombs, weapon).
    pub fn coop_status(&self) -> Vec<(String, Color)> {
        let Some(p2) = &self.p2 else {
            return vec![];
        };
        let seat2 = (!self.p_swapped) as usize;
        let mut rows = [(0usize, &self.player), (seat2, p2)];
        rows.sort_by_key(|(s, _)| if *s == 1 - seat2 { 0 } else { 1 });
        let mut out = vec![];
        for (i, (_, pl)) in rows.iter().enumerate() {
            let c = if i == 0 { CYAN } else { PINK };
            out.push((
                format!(
                    "{}  x{}  B{}  {} LV{}",
                    if i == 0 { "P1" } else { "P2" },
                    pl.lives.max(0),
                    pl.bombs,
                    pl.weapon.name(),
                    pl.level
                ),
                c,
            ));
        }
        out
    }
}
