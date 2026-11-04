//! Local two-player co-op. The second ship shares the run, the score, the crystals and
//! the shop; each ship has its own lives, weapon, bombs and ability bar.
//!
//! Playing together:
//! - DUO BEAM: while the two ships fly close, the DUO bar fills. When it is full, the
//!   ability button of either player fires a huge beam from between the ships.
//! - RESCUE: when a ship runs out of lives, it leaves a rescue capsule. If the partner
//!   flies into it, the ship comes back with one life. Each ship can be rescued only once
//!   per run; after that it stays out.
//!
//! Most of the game logic is written for `self.player`. For the second ship, `with_p2`
//! temporarily swaps it into that slot, so the same code drives both ships.

use super::*;

#[derive(Default)]
pub struct CoopState {
    /// DUO bar, 0 to 1.
    pub duo: f32,
    /// Seconds left of the duo beam.
    pub beam_t: f32,
    beam_tick: f32,
    /// A player asked for the duo beam this frame.
    pub duo_req: bool,
    /// The two ships were close last frame.
    close: bool,
    /// Rescue capsule left by a ship that ran out of lives.
    pub rescue: Option<Vec2>,
    /// A boss was destroyed by the duo beam in this run.
    pub duo_boss: bool,
}

const DUO_RANGE: f32 = 120.0;
const DUO_FILL: f32 = 5.0;
const BEAM_TIME: f32 = 2.6;

impl Game {
    fn both_alive(&self) -> bool {
        self.player.alive() && self.p2.as_ref().map(|p| p.alive()).unwrap_or(false)
    }

    /// Called from inside a player's update (where the other ship may be swapped out).
    pub(super) fn duo_ready(&self) -> bool {
        self.coop() && self.co.duo >= 1.0 && self.co.close && self.co.beam_t <= 0.0
    }

    pub(super) fn update_coop(&mut self, dt: f32, au: &mut Audio) {
        if !self.coop() {
            return;
        }
        let Some(p2) = &self.p2 else { return };
        let mid = (self.player.pos + p2.pos) / 2.0;
        let gap = (self.player.pos.x - p2.pos.x).abs();
        self.co.close = self.both_alive() && gap < DUO_RANGE;
        if self.co.close && self.co.beam_t <= 0.0 && matches!(self.phase, Phase::Playing) {
            self.co.duo = (self.co.duo + dt / DUO_FILL).min(1.0);
        }
        if std::mem::take(&mut self.co.duo_req) && self.co.close && self.co.duo >= 1.0 {
            self.co.duo = 0.0;
            self.co.beam_t = BEAM_TIME;
            self.co.beam_tick = 0.0;
            au.play(Sfx::Ability);
            au.play(Sfx::Thunder);
            self.banner = Some(("DUO BEAM!".into(), String::new(), 1.2));
            self.fx.flash = 0.5;
            self.fx.flash_color = Color::from_rgba(255, 140, 240, 255);
        }
        if self.co.beam_t > 0.0 {
            self.co.beam_t -= dt;
            self.duo_beam(mid, gap, dt, au);
        }
        self.update_rescue(au);
    }

    fn duo_beam(&mut self, mid: Vec2, gap: f32, dt: f32, au: &mut Audio) {
        let half = (gap * 0.5 + 26.0).max(34.0);
        self.fx.shake = self.fx.shake.max(0.12);
        // Enemy shots inside the beam burn away.
        self.ebullets
            .retain(|b| (b.pos.x - mid.x).abs() > half || b.pos.y > mid.y);
        self.co.beam_tick -= dt;
        if self.co.beam_tick > 0.0 {
            return;
        }
        self.co.beam_tick = 0.1;
        let hit = 2 + self.player.level;
        let boss_dmg = 7.0 + self.diff() * 0.8;
        for e in &mut self.enemies {
            if e.active()
                && !e.harmless
                && e.pos.y < mid.y
                && (e.pos.x - mid.x).abs() < half + e.kind.radius()
            {
                e.shield = 0;
                e.hp -= hit;
                e.flash = 0.08;
                e.damaged = true;
                e.last_weapon = Some(self.player.weapon);
            }
        }
        if let Some(b) = &mut self.boss
            && b.targetable()
        {
            let mut y = mid.y;
            while y > 0.0 {
                let probe = [
                    vec2(mid.x, y),
                    vec2(mid.x - half * 0.6, y),
                    vec2(mid.x + half * 0.6, y),
                ];
                if let Some(t) = probe.iter().find_map(|p| b.hit_test(*p)) {
                    let res = b.damage(t, boss_dmg);
                    if res.part_destroyed {
                        self.enc.parts += 1;
                        au.play(Sfx::Boom);
                    }
                    if b.hp <= 0.0 {
                        self.co.duo_boss = true;
                    }
                    break;
                }
                y -= 14.0;
            }
        }
        for r in &mut self.rocks {
            if !r.solid && (r.pos.x - mid.x).abs() < half + 20.0 {
                r.hp -= hit;
            }
        }
    }

    fn update_rescue(&mut self, au: &mut Audio) {
        let Some(p2) = &mut self.p2 else { return };
        let out = p2.lives <= 0 && p2.dead_t <= 0.0;
        if !out {
            self.co.rescue = None;
            return;
        }
        if p2.revived || !self.player.alive() {
            return;
        }
        let Some(pos) = self.co.rescue else {
            self.co.rescue = Some(vec2(p2.pos.x.clamp(30.0, W - 30.0), PLAYER_Y));
            self.banner = Some((
                "RESCUE YOUR PARTNER".into(),
                "Fly into the capsule. Only once per run!".into(),
                2.5,
            ));
            return;
        };
        if self.player.pos.distance(pos) < 30.0 {
            p2.lives = 1;
            p2.revived = true;
            p2.dead_t = 0.01;
            p2.level = p2.level.max(2);
            self.co.rescue = None;
            au.play(Sfx::OneUp);
            self.fx.ring(pos, 200.0, 0.5, GREEN, 4.0);
            self.fx.burst(pos, GREEN, 30, 200.0);
            self.banner = Some(("PARTNER RESCUED!".into(), String::new(), 2.0));
        }
    }

    pub(super) fn draw_coop_fx(&self, off: Vec2) {
        if let Some(pos) = self.co.rescue {
            let p = pos + off;
            let pulse = 1.0 + (self.time * 6.0).sin() * 0.12;
            draw_circle(p.x, p.y, 22.0 * pulse, Color::new(0.4, 1.0, 0.5, 0.18));
            draw_circle_lines(p.x, p.y, 18.0 * pulse, 2.5, GREEN);
            draw_rectangle(p.x - 3.0, p.y - 10.0, 6.0, 20.0, GREEN);
            draw_rectangle(p.x - 10.0, p.y - 3.0, 20.0, 6.0, GREEN);
            let seat = (!self.p_swapped) as usize;
            let tag = if seat == 0 { "P1" } else { "P2" };
            ui::text_c(tag, p.x, p.y - 30.0, 13.0, GREEN);
        }
        let Some(p2) = &self.p2 else { return };
        if self.co.beam_t > 0.0 {
            let a = self.player.pos + off;
            let b = p2.pos + off;
            let mid = (a + b) / 2.0;
            let half = ((a.x - b.x).abs() * 0.5 + 26.0).max(34.0);
            let k = (self.co.beam_t / BEAM_TIME).min(1.0);
            let flick = 0.75 + (self.time * 40.0).sin() * 0.25;
            draw_rectangle(
                mid.x - half * 1.5,
                0.0,
                half * 3.0,
                mid.y,
                Color::new(1.0, 0.4, 0.9, 0.15 * k),
            );
            draw_rectangle(
                mid.x - half,
                0.0,
                half * 2.0,
                mid.y,
                Color::new(1.0, 0.55, 0.95, 0.5 * k * flick),
            );
            draw_rectangle(
                mid.x - half * 0.35,
                0.0,
                half * 0.7,
                mid.y,
                Color::new(1.0, 1.0, 1.0, 0.9 * k),
            );
            // the link between the two ships
            let mut prev = a;
            for s in 1..=8 {
                let t = s as f32 / 8.0;
                let q = a.lerp(b, t) + vec2(0.0, if s < 8 { gen_range(-7.0, 7.0) } else { 0.0 });
                draw_line(prev.x, prev.y, q.x, q.y, 3.0, Color::new(1.0, 0.8, 1.0, k));
                prev = q;
            }
        } else if self.co.close && self.both_alive() {
            let a = self.player.pos + off;
            let b = p2.pos + off;
            let c = Color::new(1.0, 0.5, 0.9, 0.25 + self.co.duo * 0.5);
            draw_line(a.x, a.y + 6.0, b.x, b.y + 6.0, 1.0 + self.co.duo * 2.0, c);
        }
    }

    /// HUD line for the DUO bar.
    pub fn duo_label(&self) -> Option<(String, Color)> {
        if !self.coop() {
            return None;
        }
        Some(if self.co.duo >= 1.0 {
            (
                "DUO READY: ability".to_string(),
                Color::from_rgba(255, 140, 240, 255),
            )
        } else {
            (
                format!("DUO {}%", (self.co.duo * 100.0) as u32),
                Color::from_rgba(200, 120, 190, 255),
            )
        })
    }

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
        if seat == 0 || self.cfg.ship2 != self.cfg.ship {
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
        self.co.rescue = None;
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
                self.draw_ship_body(p2, self.seat_ship(seat2), self.seat_skin(seat2), p, art);
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
        out.extend(self.duo_label());
        out
    }
}
