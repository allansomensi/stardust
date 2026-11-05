//! Drawing the play field: background, enemies, bullets, the ships and effects.

use super::*;

impl Game {
    pub fn draw(&self, sp: &Sprites, art: &Art, shake_on: bool, show_hud: bool) {
        self.bg.draw(sp, self.time);
        let off = self.fx.offset(shake_on);
        // Lighting, first pass: wide and faint, it lights the background under everything.
        self.queue_lights(off);
        crate::light::flush(1.0, 0.42, true);
        let season = crate::progress::season();

        for r in &self.rocks {
            if r.hp <= -100 {
                continue;
            }
            if r.solid {
                let p = r.pos + off;
                draw_rectangle(
                    p.x - 26.0,
                    p.y - 32.0,
                    52.0,
                    64.0,
                    Color::from_rgba(90, 70, 45, 255),
                );
                draw_rectangle_lines(
                    p.x - 26.0,
                    p.y - 32.0,
                    52.0,
                    64.0,
                    3.0,
                    Color::from_rgba(40, 28, 16, 255),
                );
                draw_rectangle(
                    p.x - 20.0,
                    p.y - 26.0,
                    40.0,
                    6.0,
                    Color::from_rgba(130, 100, 60, 255),
                );
                draw_circle_lines(
                    p.x,
                    p.y + 4.0,
                    10.0,
                    2.0,
                    Color::from_rgba(255, 200, 90, if r.flash > 0.0 { 255 } else { 120 }),
                );
                draw_line(
                    p.x - 6.0,
                    p.y + 4.0,
                    p.x + 6.0,
                    p.y + 4.0,
                    2.0,
                    Color::from_rgba(255, 200, 90, 120),
                );
                continue;
            }
            sp.asteroid[r.variant].draw(
                r.pos + off,
                if r.big { 5.0 } else { 3.0 },
                r.rot,
                WHITE,
                r.flash > 0.0,
            );
        }

        for s in &self.strikes {
            if s.t < 1.0 {
                let a = if (self.time * 14.0) as i32 % 2 == 0 {
                    0.5
                } else {
                    0.2
                };
                draw_rectangle(
                    s.x - 16.0,
                    0.0,
                    32.0,
                    H,
                    Color::new(0.5, 0.8, 1.0, a * 0.25),
                );
                draw_line(s.x, 0.0, s.x, H, 1.0, Color::new(0.7, 0.9, 1.0, a));
            } else {
                let mut p = vec2(s.x, 0.0);
                for i in 0..14 {
                    let q = vec2(
                        s.x + ((self.time * 50.0 + i as f32).sin() * 12.0),
                        p.y + H / 14.0,
                    );
                    draw_line(p.x, p.y, q.x, q.y, 6.0, Color::new(0.6, 0.85, 1.0, 0.5));
                    draw_line(p.x, p.y, q.x, q.y, 2.0, WHITE);
                    p = q;
                }
            }
        }

        for d in &self.drops {
            let p = d.pos + off;
            match &d.kind {
                DropKind::Crystal(v) => {
                    let f = ((self.time * 8.0 + d.age) as usize) % 2;
                    let big = *v > 1;
                    let r = if big { 11.0 } else { 7.0 };
                    draw_circle(p.x, p.y, r, Color::new(0.3, 0.8, 1.0, 0.12));
                    if big {
                        sp.crystal_big[f].draw(p, 3.0, 0.0, WHITE, false);
                    } else {
                        sp.crystal[f].draw(p, 2.0, 0.0, WHITE, false);
                    }
                }
                DropKind::Capsule { weapons, idx, t, n } => {
                    let w = weapons[*idx];
                    let cycle = if self.mods.capsule_four { 2.4 } else { 1.8 };
                    let blink = *n > 1 && *t > cycle - 0.4 && (self.time * 16.0) as i32 % 2 == 0;
                    let next = if *n > 1 {
                        Some(weapons[(*idx + 1) % *n])
                    } else {
                        None
                    };
                    rifts::draw_capsule(p, w, next, *t / cycle, self.time, blink);
                }
                DropKind::Curse(..) | DropKind::Relic(_) | DropKind::Rift => {
                    rifts::draw_drop(&d.kind, p, self.time)
                }
                DropKind::Letter(i) => arcade::draw_letter_drop(p, *i, self.time),
                DropKind::Card(c) => {
                    let rc = cards::CARDS[*c].rarity.color();
                    let pulse = (self.time * 6.0).sin() * 0.5 + 0.5;
                    draw_circle(
                        p.x,
                        p.y,
                        22.0 + pulse * 4.0,
                        Color::new(rc.r, rc.g, rc.b, 0.25),
                    );
                    let sx = (self.time * 3.0).cos();
                    draw_texture_ex(
                        &sp.card.tex,
                        p.x - 10.5 * sx.abs(),
                        p.y - 13.5,
                        WHITE,
                        DrawTextureParams {
                            dest_size: Some(vec2(21.0 * sx.abs().max(0.15), 27.0)),
                            ..Default::default()
                        },
                    );
                }
                DropKind::Log(_) => encounters::draw_log_drop(p, self.time),
                DropKind::Power(k) => {
                    let c = k.color();
                    let pulse = (self.time * 8.0).sin() * 2.0;
                    draw_circle(p.x, p.y, 16.0 + pulse, Color::new(c.r, c.g, c.b, 0.25));
                    draw_circle(p.x, p.y, 12.0, c);
                    draw_circle_lines(p.x, p.y, 12.0, 2.0, WHITE);
                    let size = if k.letter().len() > 1 { 12.0 } else { 18.0 };
                    ui::text_c(
                        k.letter(),
                        p.x,
                        p.y + size * 0.33,
                        size,
                        Color::from_rgba(20, 10, 30, 255),
                    );
                }
            }
        }

        for e in &self.enemies {
            if !e.active() || (e.kind == Kind::Mimic && !e.revealed) {
                continue;
            }
            let frame = ((self.time * 6.0 + e.seed) as usize) % 2;
            let spr = &sp.enemies[e.kind.index()][frame];
            let mut tint = if e.kind.hp() > 1
                && e.hp < e.kind.hp()
                && !e.bonus
                && !matches!(e.kind, Kind::Merchant | Kind::Cow)
            {
                Color::new(1.0, 0.75, 0.75, 1.0)
            } else {
                WHITE
            };
            tint.a = e.alpha(self.time);
            if e.kind == Kind::Golden {
                draw_circle(
                    e.pos.x + off.x,
                    e.pos.y + off.y,
                    20.0,
                    Color::new(1.0, 0.85, 0.3, 0.2 + (self.time * 10.0).sin().abs() * 0.15),
                );
            }
            spr.draw(e.pos + off, S, e.rot, tint, e.flash > 0.0);
            self.draw_vanguard(e, off);
            if e.shield > 0 && e.kind == Kind::Shieldbearer {
                art.plate.draw(
                    e.pos + off + vec2(0.0, 18.0),
                    3.0,
                    e.rot,
                    WHITE,
                    e.flash > 0.0,
                );
            }
            if season == crate::progress::Season::Christmas
                && !matches!(e.kind, Kind::Cow | Kind::Merchant)
            {
                let h = e.pos + off + vec2(0.0, -16.0);
                draw_triangle(
                    h + vec2(-7.0, 4.0),
                    h + vec2(7.0, 4.0),
                    h + vec2(2.0, -9.0),
                    Color::from_rgba(220, 40, 50, 255),
                );
                draw_rectangle(h.x - 8.0, h.y + 3.0, 16.0, 3.0, WHITE);
                draw_circle(h.x + 2.0, h.y - 9.0, 2.5, WHITE);
            }
            if season == crate::progress::Season::Halloween && e.kind == Kind::Beetle {
                let h = e.pos + off + vec2(0.0, -14.0);
                draw_circle(h.x, h.y, 6.0, Color::from_rgba(255, 140, 20, 255));
                draw_rectangle(
                    h.x - 1.0,
                    h.y - 9.0,
                    2.0,
                    4.0,
                    Color::from_rgba(60, 140, 40, 255),
                );
            }
        }

        if let Some(b) = &self.boss {
            b.draw(sp, art, self.time, off);
        }
        self.draw_signal(off, art);

        for b in &self.bolts {
            for w in b.pts.windows(2) {
                draw_line(
                    w[0].x + off.x,
                    w[0].y + off.y,
                    w[1].x + off.x,
                    w[1].y + off.y,
                    5.0,
                    Color::new(0.6, 0.5, 1.0, 0.4),
                );
                draw_line(
                    w[0].x + off.x,
                    w[0].y + off.y,
                    w[1].x + off.x,
                    w[1].y + off.y,
                    2.0,
                    WHITE,
                );
            }
        }

        self.draw_vanguard_fx(off);
        for r in &self.arcade.rails {
            arsenal::draw_rail(r, off);
        }
        for b in &self.bullets {
            let p = b.pos + off;
            match b.kind {
                BKind::Shot | BKind::Pellet => {
                    let dir = b.vel.normalize_or_zero();
                    let len = if b.kind == BKind::Pellet { 6.0 } else { 11.0 };
                    let (c1, c2) = if b.kind == BKind::Pellet {
                        (
                            Color::new(1.0, 0.5, 0.1, 0.5),
                            Color::new(1.0, 0.85, 0.5, 1.0),
                        )
                    } else {
                        (
                            Color::new(1.0, 0.6, 0.1, 0.5),
                            Color::new(1.0, 0.95, 0.6, 1.0),
                        )
                    };
                    draw_line(p.x, p.y, p.x - dir.x * len, p.y - dir.y * len, 5.0, c1);
                    draw_line(
                        p.x,
                        p.y,
                        p.x - dir.x * (len - 2.0),
                        p.y - dir.y * (len - 2.0),
                        2.5,
                        c2,
                    );
                }
                BKind::Laser => {
                    draw_rectangle(
                        p.x - 3.0,
                        p.y - 12.0,
                        6.0,
                        26.0,
                        Color::new(0.3, 0.8, 1.0, 0.35),
                    );
                    draw_rectangle(
                        p.x - 1.0,
                        p.y - 12.0,
                        2.0,
                        26.0,
                        Color::new(0.85, 1.0, 1.0, 1.0),
                    );
                }
                BKind::Missile => {
                    let dir = b.vel.normalize_or_zero();
                    draw_line(
                        p.x,
                        p.y,
                        p.x - dir.x * 9.0,
                        p.y - dir.y * 9.0,
                        4.0,
                        Color::from_rgba(255, 110, 170, 255),
                    );
                    draw_circle(p.x, p.y, 2.5, WHITE);
                }
                BKind::Star => {
                    let a = b.t * 360.0 + b.phase * 40.0;
                    draw_circle(p.x, p.y, 6.0, Color::new(1.0, 0.85, 0.4, 0.25));
                    draw_poly(p.x, p.y, 4, 5.5, a, Color::from_rgba(255, 220, 120, 255));
                    draw_poly(
                        p.x,
                        p.y,
                        4,
                        5.5,
                        a + 45.0,
                        Color::from_rgba(255, 245, 200, 255),
                    );
                    draw_circle(p.x, p.y, 1.8, WHITE);
                }
                BKind::Wave => {
                    draw_circle(p.x, p.y, 6.0, Color::new(0.4, 1.0, 0.6, 0.3));
                    draw_circle(p.x, p.y, 3.0, Color::from_rgba(200, 255, 220, 255));
                }
                BKind::Disc => {
                    let r = 7.0 + b.dmg.min(3.0);
                    let a = b.t * 25.0;
                    draw_poly(
                        p.x,
                        p.y,
                        6,
                        r,
                        a.to_degrees(),
                        Color::from_rgba(200, 200, 230, 255),
                    );
                    draw_poly_lines(p.x, p.y, 6, r + 2.0, a.to_degrees() + 30.0, 1.5, WHITE);
                }
                BKind::Bounce
                | BKind::Grenade
                | BKind::Ice
                | BKind::Bee
                | BKind::Hole
                | BKind::Cow => encounters::draw_bullet(b, p, sp, self.time),
                BKind::Needle
                | BKind::Boomerang
                | BKind::Prism
                | BKind::Mine
                | BKind::Nova
                | BKind::Dust
                | BKind::Lance => rifts::draw_bullet(b, p, self.time),
                BKind::Charge => arcade::draw_charge_bullet(b, p, self.time),
                BKind::Fireball | BKind::Plasma | BKind::Meteor => {
                    arsenal::draw_bullet(b, p, self.time)
                }
                BKind::Rail => {}
                BKind::Flame => {
                    let k = (b.t / 0.45).min(1.0);
                    let c = Color::new(1.0, 0.85 - k * 0.6, 0.3 - k * 0.3, 1.0 - k * 0.7);
                    draw_circle(p.x, p.y, 3.0 + k * 7.0, c);
                }
            }
        }

        self.enc_draw(off, sp);
        self.rift_draw(off);
        self.arcade_draw(off);
        if self.ally_sectors > 0 && self.player.alive() {
            art.ships[1][0].draw(
                self.ally_pos + off,
                2.0,
                0.0,
                Color::new(1.0, 1.0, 1.0, 0.9),
                false,
            );
        }

        if self.player.alive() {
            let p = self.player.pos + off;
            let blink = self.player.invuln > 0.0
                && self.star_t <= 0.0
                && (self.time * 18.0) as i32 % 2 == 0;
            self.draw_trail(off);
            self.draw_sight(p);
            if !blink {
                let seat = self.p_swapped as usize;
                self.draw_ship_body(
                    &self.player,
                    self.seat_ship(seat),
                    self.seat_skin(seat),
                    p,
                    art,
                );
                self.rift_draw_player_fx(p);
                self.draw_charge(&self.player, p);
            }
            for d in self.drones.iter().take(self.drones_total() as usize) {
                sp.drone.draw(*d + off, 2.0, 0.0, WHITE, false);
            }
            if self.wall_t > 0.0 {
                let wy = p.y - 60.0;
                let a = 0.5 + (self.time * 10.0).sin() * 0.2;
                draw_rectangle(
                    p.x - 70.0,
                    wy - 5.0,
                    140.0,
                    10.0,
                    Color::new(0.4, 0.9, 1.0, a * 0.5),
                );
                draw_rectangle_lines(
                    p.x - 70.0,
                    wy - 5.0,
                    140.0,
                    10.0,
                    2.0,
                    Color::new(0.7, 1.0, 1.0, a),
                );
            }
            if self.cannon_t > 0.0 {
                let flick = 0.8 + (self.time * 40.0).sin() * 0.2;
                draw_rectangle(
                    p.x - 26.0,
                    0.0,
                    52.0,
                    p.y - 20.0,
                    Color::new(1.0, 0.5, 0.2, 0.25 * flick),
                );
                draw_rectangle(
                    p.x - 16.0,
                    0.0,
                    32.0,
                    p.y - 20.0,
                    Color::new(1.0, 0.75, 0.3, 0.7 * flick),
                );
                draw_rectangle(
                    p.x - 6.0,
                    0.0,
                    12.0,
                    p.y - 20.0,
                    Color::new(1.0, 1.0, 0.9, flick),
                );
            }
        }

        self.draw_scenery(off);
        for b in &self.ebullets {
            let p = b.pos + off;
            if b.style == 3 {
                draw_circle(p.x, p.y, b.r + 2.0, Color::new(1.0, 1.0, 1.0, 0.2));
                draw_circle_lines(p.x, p.y, b.r, 1.5, Color::new(1.0, 1.0, 1.0, 0.85));
                for k in 0..3 {
                    let a = k as f32 * 1.05 + self.time;
                    draw_line(
                        p.x - a.cos() * b.r,
                        p.y - a.sin() * b.r,
                        p.x + a.cos() * b.r,
                        p.y + a.sin() * b.r,
                        1.0,
                        Color::new(1.0, 1.0, 1.0, 0.7),
                    );
                }
                continue;
            }
            // Enemy shots: always warm or purple colors, with a dark outline
            // and a white core. Crystals are always cyan-blue: they never get mixed up.
            let (outer, inner) = match b.style {
                1 => (
                    Color::from_rgba(255, 140, 40, 255),
                    Color::from_rgba(255, 240, 180, 255),
                ),
                2 => (
                    Color::from_rgba(255, 70, 50, 255),
                    Color::from_rgba(255, 230, 210, 255),
                ),
                4 => (Color::from_rgba(200, 80, 255, 255), WHITE),
                _ => (
                    Color::from_rgba(255, 50, 120, 255),
                    Color::from_rgba(255, 220, 235, 255),
                ),
            };
            let pulse = (self.time * 14.0 + b.pos.x * 0.05).sin() * 0.5 + 0.5;
            draw_circle(
                p.x,
                p.y,
                b.r + 3.0 + pulse,
                Color::new(outer.r, outer.g, outer.b, 0.28),
            );
            draw_circle(p.x, p.y, b.r + 1.4, Color::from_rgba(25, 0, 15, 255));
            draw_circle(p.x, p.y, b.r, outer);
            draw_circle(p.x, p.y, b.r * 0.5, inner);
        }

        self.coop_draw(off, art);
        self.draw_coop_fx(off);
        if let Some(b) = &self.boss {
            b.draw_beam(self.time);
        }

        self.fx.draw_world(off);
        // Lighting, second pass: tight and on top, a bloom for shots, beams and blasts.
        crate::light::flush(0.45, 0.38, false);
        self.fx.draw_texts();

        self.enc_draw_late();
        if self.timewarp_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.3, 0.2, 0.8, 0.15));
        }
        if self.slowmo_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.2, 0.3, 0.9, 0.08));
        }
        if self.nostalgia_t > 0.0 {
            draw_rectangle(0.0, 0.0, W, H, Color::new(0.1, 0.9, 0.3, 0.18));
            for y in (0..H as i32).step_by(4) {
                draw_line(
                    0.0,
                    y as f32,
                    W,
                    y as f32,
                    1.0,
                    Color::new(0.0, 0.0, 0.0, 0.25),
                );
            }
            ui::text_c(
                "YEAR 2000",
                W / 2.0,
                H * 0.3,
                40.0,
                Color::new(0.5, 1.0, 0.5, self.nostalgia_t.min(1.0)),
            );
        }
        if let Some(fog) = &self.fog {
            let p = self.player.pos;
            let r = 230.0;
            draw_texture_ex(
                fog,
                p.x - r,
                p.y - r,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(r * 2.0, r * 2.0)),
                    ..Default::default()
                },
            );
            let dark = Color::new(0.0, 0.0, 0.0, 0.94);
            draw_rectangle(0.0, 0.0, W, (p.y - r).max(0.0), dark);
            draw_rectangle(0.0, p.y + r, W, H, dark);
            draw_rectangle(0.0, p.y - r, (p.x - r).max(0.0), r * 2.0, dark);
            draw_rectangle(p.x + r, p.y - r, W, r * 2.0, dark);
        }

        if self.fx.flash > 0.0 {
            let c = self.fx.flash_color;
            draw_rectangle(
                0.0,
                0.0,
                W,
                H,
                Color::new(c.r, c.g, c.b, self.fx.flash * 0.6 * crate::fx::flash_mul()),
            );
        }

        if show_hud {
            self.draw_hud(sp, art);
        } else {
            self.draw_mini_hud();
        }
        self.draw_overlays(sp);
        self.rift_draw_top(sp, art);
    }

    /// Only the essentials when the side panels show the rest.
    /// Engine flame, hull, shield and hitbox of one ship (used for both players).
    pub(super) fn draw_ship_body(&self, pl: &Player, ship: usize, skin: usize, p: Vec2, art: &Art) {
        let flame = 6.0 + (self.time * 40.0 + pl.pos.x * 0.01).sin().abs() * 7.0;
        draw_triangle(
            p + vec2(-4.0, 20.0),
            p + vec2(4.0, 20.0),
            p + vec2(0.0, 20.0 + flame),
            Color::from_rgba(255, 140, 40, 255),
        );
        draw_triangle(
            p + vec2(-2.0, 20.0),
            p + vec2(2.0, 20.0),
            p + vec2(0.0, 18.0 + flame * 0.6),
            Color::from_rgba(255, 240, 150, 255),
        );
        let spr = &art.ships[ship][skin];
        let squash = 1.0 - pl.tilt.abs() * 0.18;
        let sz = vec2(spr.w * S * squash, spr.h * S);
        let tint = if self.star_t > 0.0 || skin == 6 {
            ui::hsl(self.time * 0.8, 0.8, 0.75)
        } else if self.web_t > 0.0 {
            Color::new(0.8, 0.8, 0.9, 1.0)
        } else {
            WHITE
        };
        if ship == 4 || ship == 11 {
            draw_circle(p.x, p.y, 20.0, Color::new(0.7, 0.6, 1.0, 0.12));
        }
        draw_texture_ex(
            &spr.tex,
            (p.x - sz.x / 2.0).round(),
            (p.y - sz.y / 2.0).round(),
            tint,
            DrawTextureParams {
                dest_size: Some(sz),
                ..Default::default()
            },
        );
        if self.web_t > 0.0 {
            for k in 0..4 {
                let a = k as f32 * 0.8 + self.time;
                draw_line(
                    p.x - a.cos() * 18.0,
                    p.y - a.sin() * 18.0,
                    p.x + a.cos() * 18.0,
                    p.y + a.sin() * 18.0,
                    1.0,
                    Color::new(1.0, 1.0, 1.0, 0.6),
                );
            }
        }
        if pl.shield {
            let pulse = (self.time * 5.0).sin() * 2.0;
            draw_circle_lines(p.x, p.y, 28.0 + pulse, 2.0, Color::new(0.5, 0.9, 1.0, 0.7));
            draw_circle(p.x, p.y, 28.0 + pulse, Color::new(0.4, 0.8, 1.0, 0.08));
        }
        if pl.focus || self.cfg.show_hitbox {
            draw_circle(p.x, p.y, self.hit_r() + 1.5, WHITE);
            draw_circle(p.x, p.y, self.hit_r() - 1.0, RED);
        }
    }
}
