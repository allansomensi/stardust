//! Interface drawing helpers: text, panels, logo and cards.

use crate::W;
use crate::cards::{CARDS, CardDef};
use crate::sprites::Sprites;
use macroquad::prelude::*;

pub const GOLD: Color = Color::new(1.0, 0.82, 0.3, 1.0);
pub const CYAN: Color = Color::new(0.45, 0.88, 1.0, 1.0);
pub const DIM: Color = Color::new(0.6, 0.62, 0.72, 1.0);
pub const PINK: Color = Color::new(1.0, 0.45, 0.7, 1.0);
pub const GREEN: Color = Color::new(0.5, 0.95, 0.45, 1.0);
pub const RED: Color = Color::new(1.0, 0.35, 0.35, 1.0);

/// Allowed font sizes (in logical units). Everything is rounded to one of them.
pub const SIZES: [u16; 22] = [
    12, 13, 14, 15, 16, 17, 18, 19, 20, 22, 24, 26, 28, 30, 34, 38, 44, 48, 52, 60, 84, 96,
];

static BODY_TTF: &[u8] = include_bytes!("../assets/fonts/PixelifySans.ttf");
static TITLE_TTF: &[u8] = include_bytes!("../assets/fonts/PressStart2P.ttf");

/// Size adjustment for each font to match the layout in logical units.
const BODY_SF: f32 = 1.12;
const TITLE_SF: f32 = 0.6;

struct Fonts {
    body: Font,
    title: Font,
    k: f32,
}

thread_local! {
    static FONTS: std::cell::RefCell<Option<Fonts>> = const { std::cell::RefCell::new(None) };
}

pub fn snap(size: f32) -> u16 {
    let s = size.round() as i32;
    *SIZES
        .iter()
        .min_by_key(|&&v| (v as i32 - s).abs())
        .unwrap_or(&16)
}

fn charset() -> String {
    let mut chars: String = (32u8..127).map(|c| c as char).collect();
    chars.push_str("ÁÀÂÃÉÊÍÓÔÕÚÜÇáàâãéêíóôõúüç·°ºª");
    chars
}

/// Sets how many physical pixels exist per logical unit (called every frame).
/// When the scale changes, the fonts are recreated (new atlas) and every glyph is
/// pre-rasterized at the exact screen size, before anything is drawn that frame.
pub fn set_pixel_scale(k_raw: f32) {
    let k = ((k_raw * 8.0).round() / 8.0).clamp(0.75, 2.5);
    let same = FONTS.with(|f| {
        f.borrow()
            .as_ref()
            .map(|f| (f.k - k).abs() < 0.001)
            .unwrap_or(false)
    });
    if same {
        return;
    }
    let mut body = load_ttf_font_from_bytes(BODY_TTF).expect("embedded font");
    let mut title = load_ttf_font_from_bytes(TITLE_TTF).expect("embedded font");
    body.set_filter(FilterMode::Linear);
    title.set_filter(FilterMode::Linear);
    let chars = charset();
    for &s in SIZES.iter() {
        let (fs, sc) = raster(s, BODY_SF, k);
        measure_text(&chars, Some(&body), fs, sc);
        if s >= 14 {
            let (fs, sc) = raster(s, TITLE_SF, k);
            measure_text(&chars, Some(&title), fs, sc);
        }
    }
    FONTS.with(|f| *f.borrow_mut() = Some(Fonts { body, title, k }));
}

fn raster(size: u16, sf: f32, k: f32) -> (u16, f32) {
    let logical = size as f32 * sf;
    let fs = (logical * k).round().max(4.0) as u16;
    (fs, logical / fs as f32)
}

fn with_font<R>(title: bool, f: impl FnOnce(Option<&Font>, f32) -> R) -> R {
    FONTS.with(|fonts| {
        let fonts = fonts.borrow();
        match fonts.as_ref() {
            Some(x) => f(Some(if title { &x.title } else { &x.body }), x.k),
            None => f(None, 1.0),
        }
    })
}

fn measure(s: &str, size: u16, title: bool) -> f32 {
    let s = &crate::i18n::tr(s);
    with_font(title, |font, k| {
        let (fs, sc) = raster(size, if title { TITLE_SF } else { BODY_SF }, k);
        measure_text(s, font, fs, sc).width
    })
}

pub fn width(s: &str, size: f32) -> f32 {
    measure(s, snap(size), false)
}

pub fn title_width(s: &str, size: f32) -> f32 {
    measure(s, snap(size), true)
}

fn raw_font(s: &str, x: f32, y: f32, size: u16, color: Color, title: bool) {
    let s = &crate::i18n::tr(s);
    with_font(title, |font, k| {
        let (fs, sc) = raster(size, if title { TITLE_SF } else { BODY_SF }, k);
        draw_text_ex(
            s,
            x,
            y,
            TextParams {
                font,
                font_size: fs,
                font_scale: sc,
                color,
                ..Default::default()
            },
        );
    })
}

pub fn text(s: &str, x: f32, y: f32, size: f32, color: Color) {
    let fs = snap(size);
    let sh = (fs as f32 / 16.0).clamp(0.75, 2.0);
    raw_font(
        s,
        x + sh,
        y + sh,
        fs,
        Color::new(0.0, 0.0, 0.0, color.a * 0.7),
        false,
    );
    raw_font(s, x, y, fs, color, false);
}

pub fn text_c(s: &str, x: f32, y: f32, size: f32, color: Color) {
    text(s, x - width(s, size) / 2.0, y, size, color);
}

pub fn text_r(s: &str, x: f32, y: f32, size: f32, color: Color) {
    text(s, x - width(s, size), y, size, color);
}

/// Title with the arcade font and a thick outline. Shrinks itself to fit `max_w`.
pub fn text_outline_fit(
    s: &str,
    cx: f32,
    y: f32,
    size: f32,
    color: Color,
    outline: Color,
    max_w: f32,
) {
    let mut sz = snap(size);
    while title_width(s, sz as f32) > max_w && sz > 14 {
        let i = SIZES.iter().position(|&v| v == sz).unwrap_or(1);
        sz = SIZES[i.saturating_sub(1)];
    }
    let x = cx - title_width(s, sz as f32) / 2.0;
    let o = (sz as f32 / 22.0).clamp(1.0, 3.0);
    for (dx, dy) in [
        (-o, 0.0),
        (o, 0.0),
        (0.0, -o),
        (0.0, o),
        (-o, -o),
        (o, o),
        (-o, o),
        (o, -o),
    ] {
        raw_font(s, x + dx, y + dy, sz, outline, true);
    }
    raw_font(s, x + o, y + o * 2.0, sz, outline, true);
    raw_font(s, x, y, sz, color, true);
}

pub fn text_outline(s: &str, cx: f32, y: f32, size: f32, color: Color, outline: Color) {
    text_outline_fit(s, cx, y, size, color, outline, W - 24.0);
}

/// Simple word wrapping by width.
pub fn wrap(s: &str, size: f32, max_w: f32) -> Vec<String> {
    let s = &crate::i18n::tr(s);
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        let candidate = if cur.is_empty() {
            word.to_string()
        } else {
            format!("{cur} {word}")
        };
        if width(&candidate, size) > max_w && !cur.is_empty() {
            lines.push(std::mem::take(&mut cur));
            cur = word.to_string();
        } else {
            cur = candidate;
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

pub fn panel(x: f32, y: f32, w: f32, h: f32, border: Color) {
    draw_rectangle(x, y, w, h, Color::new(0.03, 0.03, 0.1, 0.88));
    draw_rectangle_lines(x, y, w, h, 2.0, border);
    draw_rectangle_lines(
        x + 4.0,
        y + 4.0,
        w - 8.0,
        h - 8.0,
        1.0,
        Color::new(border.r, border.g, border.b, 0.35),
    );
}

pub fn logo(y: f32, time: f32) {
    let title = "STARDUST";
    let mut size = 84.0;
    let mut total = title_width(title, size);
    if total > W - 40.0 {
        size *= (W - 40.0) / total;
        total = title_width(title, size);
    }
    let mut x = W / 2.0 - total / 2.0;
    for (i, ch) in title.chars().enumerate() {
        let s = ch.to_string();
        let bob = ((time * 3.0) + i as f32 * 0.6).sin() * 4.0;
        let hue = (time * 0.15 + i as f32 * 0.07).fract();
        let c = hsl(0.5 + hue * 0.35, 0.9, 0.62);
        let cw = title_width(&s, size);
        text_outline(
            &s,
            x + cw / 2.0,
            y + bob,
            size,
            c,
            Color::from_rgba(20, 8, 40, 255),
        );
        x += cw;
    }
}

pub fn hsl(h: f32, s: f32, l: f32) -> Color {
    let h = h.fract();
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    let f = |t: f32| {
        let t = (t + 1.0).fract();
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    Color::new(f(h + 1.0 / 3.0), f(h), f(h - 1.0 / 3.0), 1.0)
}

/// Centered vertical menu list.
pub fn menu(items: &[String], sel: usize, x: f32, y: f32, spacing: f32, size: f32, time: f32) {
    for (i, it) in items.iter().enumerate() {
        let yy = y + i as f32 * spacing;
        if i == sel {
            let w = width(it, size);
            let pulse = (time * 6.0).sin() * 3.0;
            text(">", x - w / 2.0 - 26.0 - pulse, yy, size, GOLD);
            text("<", x + w / 2.0 + 12.0 + pulse, yy, size, GOLD);
            text_c(it, x, yy, size, GOLD);
        } else {
            text_c(it, x, yy, size, DIM);
        }
    }
}

pub const CARD_W: f32 = 84.0;
pub const CARD_H: f32 = 112.0;

/// Draws a card (front) with its top-left corner at (x, y).
pub fn card_face(
    sprites: &Sprites,
    idx: usize,
    x: f32,
    y: f32,
    scale: f32,
    owned: bool,
    glow: f32,
) {
    let w = CARD_W * scale;
    let h = CARD_H * scale;
    let def: &CardDef = &CARDS[idx];
    if !owned {
        draw_rectangle(x, y, w, h, Color::from_rgba(20, 20, 36, 255));
        draw_rectangle_lines(x, y, w, h, 2.0, Color::from_rgba(60, 60, 90, 255));
        text_c(
            "?",
            x + w / 2.0,
            y + h / 2.0 + 14.0 * scale,
            44.0 * scale,
            Color::from_rgba(70, 70, 100, 255),
        );
        return;
    }
    let rc = def.rarity.color();
    if glow > 0.0 {
        draw_rectangle(
            x - 4.0,
            y - 4.0,
            w + 8.0,
            h + 8.0,
            Color::new(rc.r, rc.g, rc.b, 0.25 * glow),
        );
    }
    draw_rectangle(
        x,
        y,
        w,
        h,
        Color::new(rc.r * 0.25, rc.g * 0.25, rc.b * 0.3, 1.0),
    );
    // art
    let art_h = h * 0.58;
    draw_rectangle(
        x + 6.0 * scale,
        y + 6.0 * scale,
        w - 12.0 * scale,
        art_h,
        Color::from_rgba(10, 10, 26, 255),
    );
    let shine = Color::new(rc.r, rc.g, rc.b, 0.18);
    draw_circle(
        x + w / 2.0,
        y + 6.0 * scale + art_h / 2.0,
        art_h * 0.42,
        shine,
    );
    sprites.icon(def.icon).draw(
        vec2(x + w / 2.0, y + 6.0 * scale + art_h / 2.0),
        4.0 * scale,
        0.0,
        WHITE,
        false,
    );
    // frame
    draw_rectangle_lines(x, y, w, h, 3.0 * scale, rc);
    draw_rectangle_lines(
        x + 6.0 * scale,
        y + 6.0 * scale,
        w - 12.0 * scale,
        art_h,
        1.0,
        Color::new(rc.r, rc.g, rc.b, 0.6),
    );
    // name (only when the card is big enough to read)
    let lines = if scale >= 0.8 {
        wrap(def.name, 13.0 * scale, w - 8.0)
    } else {
        vec![]
    };
    for (i, l) in lines.iter().take(2).enumerate() {
        text_c(
            l,
            x + w / 2.0,
            y + art_h + 22.0 * scale + i as f32 * 12.0 * scale,
            13.0 * scale,
            WHITE,
        );
    }
    // rarity gems
    let gems = def.rarity as usize + 1;
    for g in 0..gems {
        let gx = x + w / 2.0 + (g as f32 - (gems as f32 - 1.0) / 2.0) * 9.0 * scale;
        draw_poly(gx, y + h - 8.0 * scale, 4, 3.5 * scale, 45.0, rc);
    }
}
