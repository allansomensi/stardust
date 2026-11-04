//! Light sources for the dynamic lighting (see `crate::light`).

use super::*;
use crate::light;

/// Brightens a color a little so glows read as light rather than paint.
fn lit(c: Color, a: f32) -> Color {
    Color::new(
        (c.r * 0.8 + 0.2).min(1.0),
        (c.g * 0.8 + 0.2).min(1.0),
        (c.b * 0.8 + 0.2).min(1.0),
        a,
    )
}

impl Game {
    /// Queues every light source of this frame.
    pub(super) fn queue_lights(&self, off: Vec2) {
        if !light::enabled() {
            return;
        }
        let ship = |p: &Player, c: Color| {
            if p.alive() {
                light::add(p.pos + off + vec2(0.0, 14.0), 70.0, lit(c, 0.55));
            }
        };
        ship(&self.player, CYAN);
        if let Some(p2) = &self.p2 {
            ship(p2, PINK);
        }
        for b in self.bullets.iter().take(140) {
            let (r, a) = match b.kind {
                BKind::Fireball => (90.0, 0.9),
                BKind::Plasma => (70.0, 0.8),
                BKind::Meteor => (45.0, 0.6),
                BKind::Laser | BKind::Lance => (34.0, 0.55),
                BKind::Flame => (26.0, 0.35),
                BKind::Hole => (60.0, 0.5),
                _ => (28.0, 0.4),
            };
            let c = b.weapon.map(|w| w.color()).unwrap_or(WHITE);
            light::add(b.pos + off, r, lit(c, a));
        }
        for b in self.ebullets.iter().take(90) {
            light::add(b.pos + off, 22.0, Color::new(1.0, 0.4, 0.35, 0.35));
        }
        for r in &self.arcade.rails {
            let k = (r.life / 0.22).clamp(0.0, 1.0);
            let mut y = r.y;
            while y > -40.0 {
                light::add(
                    vec2(r.x, y) + off,
                    70.0,
                    Color::new(0.5, 0.85, 1.0, 0.7 * k),
                );
                y -= 70.0;
            }
        }
        if self.co.beam_t > 0.0
            && let Some(p2) = &self.p2
        {
            let m = (self.player.pos + p2.pos) / 2.0;
            let mut y = m.y;
            while y > -40.0 {
                light::add(vec2(m.x, y) + off, 110.0, Color::new(1.0, 0.5, 0.9, 0.7));
                y -= 90.0;
            }
        }
        for (p, r, c, life) in &self.fx.lights {
            light::add(
                *p + off,
                *r,
                Color::new(c.r, c.g, c.b, (life / 0.4).min(1.0)),
            );
        }
        if let Some(b) = &self.boss {
            light::add(b.pos + off, 140.0, Color::new(1.0, 0.5, 0.7, 0.25));
        }
    }
}
