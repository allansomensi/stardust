//! Hand-drawn weapon icons: used on capsules, the HUD and menus.
//! Everything is built from simple shapes so it stays sharp at any resolution.

use crate::ships::Weapon;
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

fn with_a(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

fn star(c: Vec2, r_out: f32, r_in: f32, points: usize, rot: f32, col: Color) {
    let n = points * 2;
    for i in 0..n {
        let a0 = rot + i as f32 / n as f32 * TAU - PI / 2.0;
        let a1 = rot + (i + 1) as f32 / n as f32 * TAU - PI / 2.0;
        let r0 = if i % 2 == 0 { r_out } else { r_in };
        let r1 = if i % 2 == 0 { r_in } else { r_out };
        draw_triangle(
            c,
            c + vec2(a0.cos(), a0.sin()) * r0,
            c + vec2(a1.cos(), a1.sin()) * r1,
            col,
        );
    }
}

fn hexagon(c: Vec2, r: f32, col: Color) {
    draw_poly(c.x, c.y, 6, r, 30.0, col);
}

/// Draws the icon of `w` centered at `c`, fitting a square of side `s`.
pub fn weapon_icon(w: Weapon, c: Vec2, s: f32, col: Color, alpha: f32) {
    let u = s / 16.0;
    let col = with_a(col, alpha);
    let white = with_a(WHITE, alpha);
    let dark = with_a(Color::from_rgba(20, 10, 30, 255), alpha);
    let th = (1.6 * u).max(1.0);
    let p = |x: f32, y: f32| c + vec2(x * u, y * u);
    match w {
        Weapon::Blaster => {
            for (dx, ang) in [(-1.0f32, -0.35f32), (0.0, 0.0), (1.0, 0.35)] {
                let base = p(dx * 1.5, 6.0);
                let dir = vec2(ang.sin(), -ang.cos());
                let tip = base + dir * 12.0 * u;
                draw_line(base.x, base.y, tip.x, tip.y, 2.4 * u, col);
                draw_circle(tip.x, tip.y, 1.8 * u, white);
            }
        }
        Weapon::Laser => {
            draw_rectangle(
                c.x - 3.0 * u,
                c.y - 8.0 * u,
                6.0 * u,
                16.0 * u,
                with_a(col, 0.35),
            );
            draw_rectangle(c.x - 1.3 * u, c.y - 8.0 * u, 2.6 * u, 16.0 * u, col);
            draw_rectangle(c.x - 0.5 * u, c.y - 8.0 * u, 1.0 * u, 16.0 * u, white);
        }
        Weapon::Missile => {
            draw_triangle(p(0.0, -8.0), p(-3.0, -3.0), p(3.0, -3.0), white);
            draw_rectangle(c.x - 3.0 * u, c.y - 3.0 * u, 6.0 * u, 8.0 * u, col);
            draw_triangle(p(-3.0, 1.0), p(-6.0, 6.0), p(-3.0, 5.0), col);
            draw_triangle(p(3.0, 1.0), p(6.0, 6.0), p(3.0, 5.0), col);
            draw_triangle(
                p(-2.0, 5.0),
                p(2.0, 5.0),
                p(0.0, 9.0),
                with_a(ORANGE, alpha),
            );
        }
        Weapon::Wave => {
            let mut prev = p(0.0, 8.0);
            for i in 1..=16 {
                let y = 8.0 - i as f32;
                let q = p((y * 0.8).sin() * 5.0, y);
                draw_line(prev.x, prev.y, q.x, q.y, 2.2 * u, col);
                prev = q;
            }
            draw_circle(prev.x, prev.y, 2.0 * u, white);
        }
        Weapon::Shrapnel => {
            for (x, y) in [
                (0.0, -7.0),
                (-3.5, -6.0),
                (3.5, -6.0),
                (-6.0, -3.5),
                (6.0, -3.5),
                (-2.0, -2.5),
                (2.0, -2.5),
            ] {
                let q = p(x, y);
                draw_circle(q.x, q.y, 1.7 * u, col);
            }
            draw_triangle(p(-3.0, 7.0), p(3.0, 7.0), p(0.0, 1.0), white);
        }
        Weapon::Disc => {
            draw_poly(c.x, c.y, 6, 7.5 * u, 0.0, col);
            draw_poly_lines(c.x, c.y, 6, 7.5 * u, 30.0, th, white);
            draw_circle(c.x, c.y, 2.2 * u, dark);
        }
        Weapon::Thunder => {
            let pts = [p(2.0, -8.0), p(-3.0, -1.0), p(1.0, -1.0), p(-2.0, 8.0)];
            draw_triangle(pts[0], pts[1], pts[2], col);
            draw_triangle(pts[1], pts[2], pts[3], col);
            draw_line(pts[0].x, pts[0].y, pts[3].x, pts[3].y, 0.8 * u, white);
        }
        Weapon::Flame => {
            draw_circle(c.x, c.y + 3.0 * u, 5.0 * u, with_a(ORANGE, alpha));
            draw_triangle(
                p(-5.0, 3.0),
                p(5.0, 3.0),
                p(0.0, -8.0),
                with_a(ORANGE, alpha),
            );
            draw_circle(c.x, c.y + 4.0 * u, 2.8 * u, with_a(YELLOW, alpha));
            draw_triangle(
                p(-2.8, 4.0),
                p(2.8, 4.0),
                p(0.0, -3.0),
                with_a(YELLOW, alpha),
            );
        }
        Weapon::Pulsar => {
            star(c, 8.0 * u, 2.4 * u, 4, 0.0, col);
            draw_circle(c.x, c.y, 1.8 * u, white);
        }
        Weapon::Ricochet => {
            draw_line(
                c.x - 7.5 * u,
                c.y - 8.0 * u,
                c.x - 7.5 * u,
                c.y + 8.0 * u,
                1.0 * u,
                with_a(col, 0.5),
            );
            draw_line(
                c.x + 7.5 * u,
                c.y - 8.0 * u,
                c.x + 7.5 * u,
                c.y + 8.0 * u,
                1.0 * u,
                with_a(col, 0.5),
            );
            let pts = [p(-2.0, 8.0), p(6.5, 2.0), p(-6.5, -3.0), p(2.0, -8.0)];
            for k in 0..3 {
                draw_line(pts[k].x, pts[k].y, pts[k + 1].x, pts[k + 1].y, 2.2 * u, col);
            }
            draw_poly(pts[3].x, pts[3].y, 4, 2.4 * u, 0.0, white);
        }
        Weapon::Grenade => {
            draw_circle(c.x, c.y + 2.0 * u, 6.0 * u, col);
            draw_circle(c.x - 2.0 * u, c.y, 1.6 * u, with_a(col, 0.4));
            draw_rectangle(c.x - 2.0 * u, c.y - 6.5 * u, 4.0 * u, 3.0 * u, white);
            draw_circle_lines(c.x + 3.5 * u, c.y - 6.0 * u, 2.0 * u, th * 0.7, white);
        }
        Weapon::Cryo => {
            for k in 0..3 {
                let a = k as f32 / 3.0 * PI;
                let d = vec2(a.cos(), a.sin()) * 8.0 * u;
                draw_line(c.x - d.x, c.y - d.y, c.x + d.x, c.y + d.y, 2.0 * u, col);
                for sgn in [-1.0, 1.0] {
                    let tip = c + d * 0.6 * sgn;
                    let n = vec2(-d.y, d.x).normalize_or_zero() * 2.2 * u;
                    let back = c + d * 0.85 * sgn;
                    draw_line(tip.x + n.x, tip.y + n.y, back.x, back.y, 1.2 * u, white);
                    draw_line(tip.x - n.x, tip.y - n.y, back.x, back.y, 1.2 * u, white);
                }
            }
        }
        Weapon::Orbit => {
            draw_circle_lines(c.x, c.y, 6.5 * u, 1.0 * u, with_a(col, 0.6));
            draw_circle(c.x, c.y, 2.2 * u, white);
            for k in 0..2 {
                let a = k as f32 * PI + 0.6;
                let q = c + vec2(a.cos(), a.sin()) * 6.5 * u;
                star(q, 3.2 * u, 1.4 * u, 5, a, col);
            }
        }
        Weapon::BlackHole => {
            draw_circle(c.x, c.y, 8.0 * u, with_a(col, 0.25));
            draw_circle_lines(c.x, c.y, 7.0 * u, th, col);
            draw_circle_lines(c.x, c.y, 4.5 * u, th * 0.8, with_a(col, 0.7));
            draw_circle(c.x, c.y, 3.0 * u, with_a(BLACK, alpha));
        }
        Weapon::Hive => {
            hexagon(p(-3.4, -2.0), 3.6 * u, col);
            hexagon(p(3.4, -2.0), 3.6 * u, col);
            hexagon(p(0.0, 4.0), 3.6 * u, col);
            for q in [p(-3.4, -2.0), p(3.4, -2.0), p(0.0, 4.0)] {
                draw_poly_lines(q.x, q.y, 6, 3.6 * u, 30.0, 0.9 * u, dark);
            }
        }
        Weapon::Cow => {
            draw_triangle(p(-6.0, -5.0), p(-8.0, -9.0), p(-3.5, -5.5), white);
            draw_triangle(p(6.0, -5.0), p(8.0, -9.0), p(3.5, -5.5), white);
            draw_circle(c.x, c.y - 1.0 * u, 6.5 * u, white);
            draw_circle(c.x - 3.0 * u, c.y - 3.0 * u, 1.8 * u, dark);
            draw_circle(c.x + 3.5 * u, c.y - 1.0 * u, 1.3 * u, dark);
            draw_ellipse(
                c.x,
                c.y + 4.5 * u,
                5.0 * u,
                3.0 * u,
                0.0,
                with_a(PINK, alpha),
            );
            draw_circle(c.x - 1.6 * u, c.y + 4.5 * u, 0.8 * u, dark);
            draw_circle(c.x + 1.6 * u, c.y + 4.5 * u, 0.8 * u, dark);
        }
        Weapon::Needle => {
            for dx in [-4.0, 0.0, 4.0] {
                let top = p(dx, -8.0 + dx.abs() * 0.5);
                draw_line(top.x, top.y, top.x, c.y + 8.0 * u, 1.3 * u, col);
                draw_triangle(
                    top + vec2(-1.3 * u, 2.0 * u),
                    top + vec2(1.3 * u, 2.0 * u),
                    top - vec2(0.0, 1.5 * u),
                    white,
                );
            }
        }
        Weapon::Boomerang => {
            draw_line(
                c.x - 7.0 * u,
                c.y + 3.0 * u,
                c.x,
                c.y - 6.0 * u,
                3.2 * u,
                col,
            );
            draw_line(
                c.x,
                c.y - 6.0 * u,
                c.x + 7.0 * u,
                c.y + 3.0 * u,
                3.2 * u,
                col,
            );
            draw_circle(c.x, c.y - 6.0 * u, 1.6 * u, white);
            draw_circle_lines(c.x, c.y + 1.0 * u, 7.5 * u, 0.8 * u, with_a(col, 0.35));
        }
        Weapon::Prism => {
            draw_line(c.x, c.y + 8.0 * u, c.x, c.y + 2.0 * u, 2.0 * u, white);
            draw_triangle(p(0.0, -3.0), p(-4.0, 4.0), p(4.0, 4.0), with_a(col, 0.8));
            for (dx, cc) in [(-6.0, RED), (0.0, GREEN), (6.0, SKYBLUE)] {
                let q = p(dx, -8.0);
                draw_line(c.x, c.y - 2.0 * u, q.x, q.y, 1.5 * u, with_a(cc, alpha));
            }
        }
        Weapon::Mine => {
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU;
                let d = vec2(a.cos(), a.sin());
                let q = c + d * 7.5 * u;
                draw_line(c.x, c.y, q.x, q.y, 1.5 * u, col);
            }
            draw_circle(c.x, c.y, 4.8 * u, col);
            draw_circle(c.x, c.y, 2.0 * u, with_a(RED, alpha));
            draw_circle(c.x - 1.5 * u, c.y - 1.5 * u, 1.0 * u, white);
        }
        Weapon::Supernova => {
            star(c, 8.5 * u, 3.0 * u, 8, 0.2, with_a(ORANGE, alpha));
            star(c, 6.0 * u, 2.4 * u, 4, 0.0, col);
            draw_circle(c.x, c.y, 2.4 * u, white);
        }
        Weapon::CosmicDust => {
            for (x, y, r) in [
                (0.0, 0.0, 4.0),
                (-5.5, -4.5, 2.4),
                (5.0, -5.5, 2.0),
                (5.5, 4.5, 2.6),
                (-4.5, 5.5, 1.8),
                (0.5, -8.0, 1.4),
            ] {
                star(p(x, y), r * u, r * 0.35 * u, 4, 0.0, col);
            }
            draw_circle(c.x, c.y, 1.2 * u, white);
        }
        Weapon::SolarLance => {
            draw_rectangle(
                c.x - 4.5 * u,
                c.y - 8.0 * u,
                9.0 * u,
                13.0 * u,
                with_a(col, 0.35),
            );
            draw_rectangle(c.x - 2.5 * u, c.y - 8.0 * u, 5.0 * u, 13.0 * u, col);
            draw_rectangle(c.x - 1.0 * u, c.y - 8.0 * u, 2.0 * u, 13.0 * u, white);
            star(p(0.0, 5.5), 4.5 * u, 2.2 * u, 6, 0.0, with_a(ORANGE, alpha));
            draw_circle(c.x, c.y + 5.5 * u, 2.2 * u, white);
        }
        Weapon::Singularity => {
            draw_ellipse(
                c.x,
                c.y,
                8.5 * u,
                3.0 * u,
                -20.0,
                with_a(ORANGE, alpha * 0.8),
            );
            draw_circle(c.x, c.y, 5.0 * u, with_a(BLACK, alpha));
            draw_circle_lines(c.x, c.y, 5.2 * u, th, col);
            draw_ellipse_lines(c.x, c.y, 8.5 * u, 3.0 * u, -20.0, 0.8 * u, white);
        }
    }
}

/// Standard weapon badge: a dark disc with a colored rim and the icon on top.
pub fn weapon_badge(w: Weapon, c: Vec2, r: f32, alpha: f32) {
    let col = w.color();
    draw_circle(c.x, c.y, r, Color::new(0.06, 0.04, 0.12, 0.9 * alpha));
    draw_circle_lines(
        c.x,
        c.y,
        r,
        (r / 9.0).max(1.2),
        Color::new(col.r, col.g, col.b, alpha),
    );
    if w.is_relic() {
        draw_circle_lines(
            c.x,
            c.y,
            r + 2.5,
            1.0,
            Color::new(1.0, 0.85, 0.3, alpha * 0.8),
        );
    }
    weapon_icon(w, c, r * 1.35, col, alpha);
}
