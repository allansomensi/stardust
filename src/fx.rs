//! Partículas, ondas de choque, textos flutuantes e fundo animado.

use crate::{H, W};
use macroquad::prelude::*;
use macroquad::rand::gen_range;

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max: f32,
    pub color: Color,
    pub size: f32,
    pub drag: f32,
}

pub struct Ring {
    pub pos: Vec2,
    pub r: f32,
    pub speed: f32,
    pub life: f32,
    pub max: f32,
    pub color: Color,
    pub width: f32,
}

pub struct FloatText {
    pub pos: Vec2,
    pub text: String,
    pub life: f32,
    pub color: Color,
    pub size: f32,
}

#[derive(Default)]
pub struct Fx {
    pub particles: Vec<Particle>,
    pub rings: Vec<Ring>,
    pub texts: Vec<FloatText>,
    pub shake: f32,
    pub flash: f32,
    pub flash_color: Color,
}

use std::sync::atomic::{AtomicU32, Ordering};

static DENSITY: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0
static FLASH: AtomicU32 = AtomicU32::new(0x3f80_0000);
static STILL_BG: AtomicU32 = AtomicU32::new(0);

/// Configurações gráficas globais (vindas da tela de Configurações).
pub fn set_quality(density: f32, flash: f32, still_bg: bool) {
    DENSITY.store(density.to_bits(), Ordering::Relaxed);
    FLASH.store(flash.to_bits(), Ordering::Relaxed);
    STILL_BG.store(still_bg as u32, Ordering::Relaxed);
}

pub fn density() -> f32 {
    f32::from_bits(DENSITY.load(Ordering::Relaxed))
}

pub fn flash_mul() -> f32 {
    f32::from_bits(FLASH.load(Ordering::Relaxed))
}

fn still_bg() -> bool {
    STILL_BG.load(Ordering::Relaxed) != 0
}

fn scaled(n: usize) -> usize {
    ((n as f32 * density()).ceil() as usize).max(if n > 0 { 1 } else { 0 })
}

impl Fx {
    pub fn burst(&mut self, pos: Vec2, color: Color, n: usize, speed: f32) {
        for _ in 0..scaled(n) {
            let a = gen_range(0.0, std::f32::consts::TAU);
            let sp = gen_range(0.15, 1.0) * speed;
            let life = gen_range(0.25, 0.75);
            self.particles.push(Particle {
                pos,
                vel: vec2(a.cos(), a.sin()) * sp,
                life,
                max: life,
                color: match gen_range(0, 4) {
                    0 => WHITE,
                    1 => Color::from_rgba(255, 220, 120, 255),
                    _ => color,
                },
                size: gen_range(2.0f32, 4.0).round(),
                drag: 2.6,
            });
        }
    }

    pub fn sparks(&mut self, pos: Vec2, dir: Vec2, color: Color, n: usize) {
        for _ in 0..scaled(n) {
            let a = dir.y.atan2(dir.x) + gen_range(-0.7, 0.7);
            let sp = gen_range(80.0, 260.0);
            let life = gen_range(0.1, 0.25);
            self.particles.push(Particle {
                pos,
                vel: vec2(a.cos(), a.sin()) * sp,
                life,
                max: life,
                color,
                size: 2.0,
                drag: 4.0,
            });
        }
    }

    pub fn ring(&mut self, pos: Vec2, speed: f32, life: f32, color: Color, width: f32) {
        self.rings.push(Ring {
            pos,
            r: 4.0,
            speed,
            life,
            max: life,
            color,
            width,
        });
    }

    pub fn explosion(&mut self, pos: Vec2, color: Color, big: bool) {
        if big {
            self.burst(pos, color, 70, 340.0);
            self.ring(pos, 420.0, 0.5, Color::new(1.0, 0.9, 0.6, 1.0), 4.0);
            self.ring(pos, 240.0, 0.7, color, 3.0);
        } else {
            self.burst(pos, color, 22, 200.0);
            self.ring(pos, 160.0, 0.25, Color::new(1.0, 1.0, 1.0, 0.8), 2.0);
        }
    }

    pub fn text(&mut self, pos: Vec2, text: impl Into<String>, color: Color, size: f32) {
        self.texts.push(FloatText {
            pos,
            text: text.into(),
            life: 0.9,
            color,
            size,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.pos += p.vel * dt;
            p.vel *= 1.0 - (p.drag * dt).min(0.9);
            p.life -= dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        if self.particles.len() > 2500 {
            let extra = self.particles.len() - 2500;
            self.particles.drain(0..extra);
        }
        for r in &mut self.rings {
            r.r += r.speed * dt;
            r.life -= dt;
        }
        self.rings.retain(|r| r.life > 0.0);
        for t in &mut self.texts {
            t.pos.y -= 34.0 * dt;
            t.life -= dt;
        }
        self.texts.retain(|t| t.life > 0.0);
        self.shake = (self.shake - dt * 1.8).max(0.0);
        self.flash = (self.flash - dt * 2.5).max(0.0);
    }

    pub fn draw_world(&self, off: Vec2) {
        for r in &self.rings {
            let mut c = r.color;
            c.a *= r.life / r.max;
            draw_circle_lines(r.pos.x + off.x, r.pos.y + off.y, r.r, r.width, c);
        }
        for p in &self.particles {
            let mut c = p.color;
            c.a = (p.life / p.max).min(1.0);
            let s = p.size;
            draw_rectangle(
                (p.pos.x + off.x - s / 2.0).round(),
                (p.pos.y + off.y - s / 2.0).round(),
                s,
                s,
                c,
            );
        }
    }

    pub fn draw_texts(&self) {
        for t in &self.texts {
            let mut c = t.color;
            c.a = (t.life * 2.0).min(1.0);
            crate::ui::text_c(&t.text, t.pos.x, t.pos.y, t.size, c);
        }
    }

    pub fn offset(&self, enabled: bool) -> Vec2 {
        if self.shake > 0.0 && enabled {
            let m = (self.shake * 14.0).min(10.0);
            vec2(gen_range(-m, m), gen_range(-m, m)).round()
        } else {
            Vec2::ZERO
        }
    }
}

// ---------------------------------------------------------------------------
// Fundo
// ---------------------------------------------------------------------------

struct Star {
    pos: Vec2,
    speed: f32,
    size: f32,
    color: Color,
}

struct Rock {
    pos: Vec2,
    speed: f32,
    rot: f32,
    vr: f32,
    scale: f32,
    variant: usize,
}

pub struct Background {
    stars: Vec<Star>,
    nebula: Option<Texture2D>,
    neb_y: f32,
    pub theme: usize,
    planet_y: f32,
    rocks: Vec<Rock>,
    pub speed_mul: f32,
}

pub struct Theme {
    pub name: &'static str,
    pub base: Color,
    pub a: [f32; 3],
    pub b: [f32; 3],
}

pub const THEMES: [Theme; 10] = [
    Theme {
        name: "ÓRBITA BAIXA",
        base: Color::new(0.02, 0.02, 0.08, 1.0),
        a: [0.10, 0.20, 0.55],
        b: [0.30, 0.15, 0.60],
    },
    Theme {
        name: "CAMPO DE ASTEROIDES",
        base: Color::new(0.04, 0.03, 0.04, 1.0),
        a: [0.40, 0.25, 0.15],
        b: [0.20, 0.25, 0.40],
    },
    Theme {
        name: "NEBULOSA CARMESIM",
        base: Color::new(0.06, 0.01, 0.04, 1.0),
        a: [0.65, 0.10, 0.25],
        b: [0.90, 0.40, 0.20],
    },
    Theme {
        name: "COLMEIA EXTERIOR",
        base: Color::new(0.01, 0.05, 0.05, 1.0),
        a: [0.10, 0.50, 0.40],
        b: [0.40, 0.60, 0.15],
    },
    Theme {
        name: "CINTURÃO DE GELO",
        base: Color::new(0.02, 0.04, 0.08, 1.0),
        a: [0.45, 0.70, 0.95],
        b: [0.70, 0.85, 1.00],
    },
    Theme {
        name: "TEMPESTADE IÔNICA",
        base: Color::new(0.01, 0.03, 0.05, 1.0),
        a: [0.10, 0.35, 0.45],
        b: [0.30, 0.20, 0.55],
    },
    Theme {
        name: "RUÍNAS ANCESTRAIS",
        base: Color::new(0.05, 0.035, 0.02, 1.0),
        a: [0.55, 0.40, 0.15],
        b: [0.35, 0.25, 0.20],
    },
    Theme {
        name: "CORAÇÃO DO ENXAME",
        base: Color::new(0.05, 0.01, 0.07, 1.0),
        a: [0.50, 0.10, 0.60],
        b: [0.80, 0.20, 0.30],
    },
    Theme {
        name: "DIMENSÃO DOURADA",
        base: Color::new(0.08, 0.05, 0.01, 1.0),
        a: [0.95, 0.75, 0.20],
        b: [1.00, 0.50, 0.30],
    },
    Theme {
        name: "O VAZIO ALÉM",
        base: Color::new(0.0, 0.0, 0.01, 1.0),
        a: [0.18, 0.02, 0.10],
        b: [0.05, 0.02, 0.15],
    },
];

fn hash(x: i32, y: i32, seed: i32) -> f32 {
    let mut h =
        (x.wrapping_mul(374761393)) ^ (y.wrapping_mul(668265263)) ^ seed.wrapping_mul(2147483647);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    ((h ^ (h >> 16)) & 0xffff) as f32 / 65535.0
}

fn value_noise(x: f32, y: f32, period_y: i32, seed: i32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let fx = x - xi as f32;
    let fy = y - yi as f32;
    let s = |a: i32, b: i32| hash(a, b.rem_euclid(period_y), seed);
    let u = fx * fx * (3.0 - 2.0 * fx);
    let v = fy * fy * (3.0 - 2.0 * fy);
    let a = s(xi, yi) * (1.0 - u) + s(xi + 1, yi) * u;
    let b = s(xi, yi + 1) * (1.0 - u) + s(xi + 1, yi + 1) * u;
    a * (1.0 - v) + b * v
}

fn make_nebula(theme: &Theme, seed: i32) -> Texture2D {
    let (w, h) = (120usize, 180usize);
    let mut bytes = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let mut n = 0.0;
            let mut amp = 0.5;
            let mut f = 1.0;
            for o in 0..5 {
                let period = (6.0 * f) as i32;
                n += value_noise(
                    x as f32 / w as f32 * 4.0 * f,
                    y as f32 / h as f32 * 6.0 * f,
                    period,
                    seed + o,
                ) * amp;
                amp *= 0.5;
                f *= 2.0;
            }
            let m = value_noise(
                x as f32 / w as f32 * 2.0,
                y as f32 / h as f32 * 3.0,
                3,
                seed + 99,
            );
            let d = ((n - 0.42) * 2.2).clamp(0.0, 1.0);
            let alpha = d * d * 0.85;
            let i = (y * w + x) * 4;
            for c in 0..3 {
                let v = theme.a[c] * (1.0 - m) + theme.b[c] * m;
                bytes[i + c] = ((v + d * 0.15) * 255.0).min(255.0) as u8;
            }
            bytes[i + 3] = (alpha * 255.0) as u8;
        }
    }
    let t = Texture2D::from_rgba8(w as u16, h as u16, &bytes);
    t.set_filter(FilterMode::Linear);
    t
}

impl Background {
    pub fn new() -> Self {
        let stars = (0..140)
            .map(|_| {
                let layer = gen_range(0, 3);
                let tint = gen_range(0, 6);
                let b = 0.35 + layer as f32 * 0.25;
                Star {
                    pos: vec2(gen_range(0.0, W), gen_range(0.0, H)),
                    speed: 14.0 + layer as f32 * 38.0 + gen_range(0.0, 10.0),
                    size: if layer == 2 { 2.0 } else { 1.0 },
                    color: match tint {
                        0 => Color::new(b, b * 0.85, b * 0.6, 1.0),
                        1 => Color::new(b * 0.7, b * 0.85, b, 1.0),
                        _ => Color::new(b, b, b, 1.0),
                    },
                }
            })
            .collect();
        Background {
            stars,
            nebula: None,
            neb_y: 0.0,
            theme: usize::MAX,
            planet_y: 0.0,
            rocks: vec![],
            speed_mul: 1.0,
        }
    }

    pub fn set_theme(&mut self, theme: usize) {
        let theme = theme % THEMES.len();
        if theme == self.theme {
            return;
        }
        self.theme = theme;
        self.nebula = Some(make_nebula(&THEMES[theme], 17 + theme as i32 * 31));
        self.planet_y = -260.0;
        self.rocks.clear();
        if theme == 1 {
            for _ in 0..10 {
                self.rocks.push(Rock {
                    pos: vec2(gen_range(0.0, W), gen_range(-H, H)),
                    speed: gen_range(10.0, 30.0),
                    rot: gen_range(0.0, 6.0),
                    vr: gen_range(-0.4, 0.4),
                    scale: gen_range(1.0, 2.5),
                    variant: gen_range(0, 2),
                });
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        let sm = if still_bg() { 0.15 } else { self.speed_mul };
        for s in &mut self.stars {
            s.pos.y += s.speed * dt * sm;
            if s.pos.y > H {
                s.pos.y -= H;
                s.pos.x = gen_range(0.0, W);
            }
        }
        self.neb_y = (self.neb_y + 9.0 * dt * sm) % (H * 1.0);
        self.planet_y += 6.0 * dt * sm;
        for r in &mut self.rocks {
            r.pos.y += r.speed * dt * sm;
            r.rot += r.vr * dt;
            if r.pos.y > H + 60.0 {
                r.pos.y = -60.0;
                r.pos.x = gen_range(0.0, W);
            }
        }
    }

    /// Fundo esticado para a tela inteira (laterais dos painéis).
    pub fn draw_wide(&self, total_w: f32) {
        let theme = &THEMES[self.theme.min(THEMES.len() - 1)];
        draw_rectangle(0.0, 0.0, total_w, H, theme.base);
        if let Some(n) = &self.nebula {
            let tiles = (total_w / W).ceil() as i32 + 1;
            for tx in 0..tiles {
                for k in 0..2 {
                    let y = self.neb_y - H * k as f32;
                    let x = tx as f32 * W - (tiles as f32 * W - total_w) / 2.0;
                    draw_texture_ex(
                        n,
                        x,
                        y,
                        Color::new(1.0, 1.0, 1.0, 0.7),
                        DrawTextureParams {
                            dest_size: Some(vec2(W, H)),
                            flip_x: tx % 2 == 1,
                            ..Default::default()
                        },
                    );
                }
            }
        }
        let tiles = (total_w / W).ceil() as i32 + 1;
        for tx in 0..tiles {
            let ox = tx as f32 * W - (tiles as f32 * W - total_w) / 2.0;
            for (i, s) in self.stars.iter().enumerate() {
                let x = (s.pos.x + i as f32 * 37.0 * tx as f32).rem_euclid(W) + ox;
                let stretch = 1.0 + (self.speed_mul - 1.0) * s.speed / 30.0;
                draw_rectangle(
                    x.round(),
                    s.pos.y.round(),
                    s.size,
                    (s.size * stretch).max(1.0),
                    s.color,
                );
            }
        }
    }

    pub fn draw(&self, sprites: &crate::sprites::Sprites, time: f32) {
        let theme = &THEMES[self.theme.min(THEMES.len() - 1)];
        draw_rectangle(0.0, 0.0, W, H, theme.base);
        if let Some(n) = &self.nebula {
            for k in 0..2 {
                let y = self.neb_y - H * k as f32;
                draw_texture_ex(
                    n,
                    0.0,
                    y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(W, H)),
                        ..Default::default()
                    },
                );
            }
        }
        // Elementos de cenário por tema
        match self.theme {
            0 => {
                // Planeta azul enorme lá embaixo, ficando para trás
                let c = vec2(W * 0.72, H + 400.0 + self.planet_y.min(900.0) * 0.6);
                draw_circle(c.x, c.y, 420.0, Color::from_rgba(14, 30, 70, 255));
                draw_circle(
                    c.x - 24.0,
                    c.y + 16.0,
                    404.0,
                    Color::from_rgba(20, 48, 100, 255),
                );
                draw_circle(
                    c.x - 140.0,
                    c.y - 110.0,
                    150.0,
                    Color::from_rgba(30, 80, 60, 110),
                );
                draw_circle_lines(c.x, c.y, 423.0, 3.0, Color::from_rgba(90, 160, 230, 70));
            }
            1 => {
                for r in &self.rocks {
                    let s = &sprites.asteroid[r.variant];
                    let shade = Color::new(0.45, 0.4, 0.4, 1.0);
                    s.draw(r.pos, r.scale, r.rot, shade, false);
                }
            }
            2 => {
                let c = vec2(W * 0.2, 160.0 + self.planet_y * 0.25);
                draw_circle(c.x, c.y, 70.0, Color::from_rgba(120, 30, 40, 255));
                draw_circle(
                    c.x + 12.0,
                    c.y - 10.0,
                    60.0,
                    Color::from_rgba(170, 60, 50, 255),
                );
                draw_line(
                    c.x - 120.0,
                    c.y + 20.0,
                    c.x + 120.0,
                    c.y - 20.0,
                    3.0,
                    Color::from_rgba(255, 160, 120, 120),
                );
            }
            3 => {
                // favos distantes
                let col = Color::from_rgba(60, 160, 130, 40);
                let oy = (self.neb_y * 1.5) % 52.0;
                for row in -1..16 {
                    for i in 0..10 {
                        let x = i as f32 * 54.0 + if row % 2 == 0 { 0.0 } else { 27.0 };
                        let y = row as f32 * 46.0 + oy;
                        draw_poly_lines(x, y, 6, 24.0, 30.0, 1.5, col);
                    }
                }
            }
            4 => {
                // neve / cristais de gelo
                for (i, s) in self.stars.iter().enumerate().take(60) {
                    let x = (s.pos.x + (time * 0.8 + i as f32).sin() * 12.0).rem_euclid(W);
                    let y = (s.pos.y * 1.3 + time * 20.0).rem_euclid(H);
                    draw_rectangle(x, y, 2.0, 2.0, Color::new(0.85, 0.95, 1.0, 0.5));
                }
                let c = vec2(W * 0.85, 120.0 + self.planet_y * 0.2);
                draw_circle(c.x, c.y, 50.0, Color::from_rgba(150, 200, 240, 255));
                draw_circle(
                    c.x - 10.0,
                    c.y - 8.0,
                    42.0,
                    Color::from_rgba(210, 235, 255, 255),
                );
                draw_poly_lines(
                    c.x,
                    c.y,
                    6,
                    70.0,
                    time * 3.0,
                    1.0,
                    Color::new(0.7, 0.9, 1.0, 0.4),
                );
            }
            5 => {
                // relâmpagos distantes
                let k = (time * 0.7).floor();
                let phase = (time * 0.7).fract();
                if phase < 0.06 {
                    let x = ((k * 97.3).sin() * 0.5 + 0.5) * W;
                    draw_rectangle(0.0, 0.0, W, H, Color::new(0.6, 0.8, 1.0, 0.12));
                    let mut p = vec2(x, 0.0);
                    for i in 0..10 {
                        let q = p + vec2(((k + i as f32) * 13.7).sin() * 30.0, H / 12.0);
                        draw_line(p.x, p.y, q.x, q.y, 2.0, Color::new(0.8, 0.9, 1.0, 0.5));
                        p = q;
                    }
                }
            }
            6 => {
                // pilares e glifos ancestrais
                let oy = (self.neb_y * 0.6) % 240.0;
                for row in -1..4 {
                    for i in 0..3 {
                        let x = 40.0 + i as f32 * 170.0 + if row % 2 == 0 { 0.0 } else { 60.0 };
                        let y = row as f32 * 240.0 + oy;
                        draw_rectangle(x, y, 26.0, 160.0, Color::new(0.35, 0.27, 0.15, 0.25));
                        draw_rectangle_lines(
                            x,
                            y,
                            26.0,
                            160.0,
                            1.0,
                            Color::new(0.8, 0.6, 0.3, 0.2),
                        );
                        draw_circle_lines(
                            x + 13.0,
                            y + 30.0,
                            7.0,
                            1.0,
                            Color::new(1.0, 0.8, 0.4, 0.25),
                        );
                    }
                }
            }
            9 => {
                // olhos no escuro
                for i in 0..14 {
                    let fx = ((i as f32 * 73.1).sin() * 0.5 + 0.5) * W;
                    let fy = ((i as f32 * 41.7).cos() * 0.5 + 0.5) * H;
                    let blink = ((time * 0.7 + i as f32 * 1.3).sin() * 0.5 + 0.5).powf(6.0);
                    if blink > 0.05 {
                        let c = Color::new(0.9, 0.1, 0.2, blink * 0.8);
                        draw_ellipse(fx, fy, 7.0, 2.5 * blink + 0.5, 0.0, c);
                        draw_ellipse(fx + 22.0, fy, 7.0, 2.5 * blink + 0.5, 0.0, c);
                    }
                }
            }
            8 => {
                let pulse = (time * 2.0).sin() * 0.5 + 0.5;
                draw_circle(
                    W / 2.0,
                    H / 2.0,
                    300.0 + pulse * 20.0,
                    Color::new(1.0, 0.8, 0.2, 0.06),
                );
                for i in 0..12 {
                    let a = time * 0.3 + i as f32 / 12.0 * std::f32::consts::TAU;
                    let p = vec2(W / 2.0, H / 2.0) + vec2(a.cos(), a.sin()) * 260.0;
                    draw_line(
                        W / 2.0,
                        H / 2.0,
                        p.x,
                        p.y,
                        2.0,
                        Color::new(1.0, 0.85, 0.3, 0.07),
                    );
                }
            }
            _ => {
                let pulse = (time * 1.6).sin() * 0.5 + 0.5;
                draw_circle(
                    W / 2.0,
                    -80.0,
                    220.0 + pulse * 10.0,
                    Color::new(0.5, 0.05, 0.35, 0.18),
                );
                draw_circle(
                    W / 2.0,
                    -80.0,
                    140.0 + pulse * 8.0,
                    Color::new(0.8, 0.1, 0.3, 0.15),
                );
            }
        }
        for s in &self.stars {
            let stretch = 1.0 + (self.speed_mul - 1.0) * s.speed / 30.0;
            draw_rectangle(
                s.pos.x.round(),
                s.pos.y.round(),
                s.size,
                (s.size * stretch).max(1.0),
                s.color,
            );
        }
    }
}
