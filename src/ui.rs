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

/// One font per size (and per typeface), each with its own small glyph atlas. A single
/// shared atlas grew to tens of megabytes at 4K and was re-uploaded to the GPU every time
/// a glyph was added, which froze the game when going fullscreen.
struct Fonts {
    /// [0] body, [1] title; indexed like `SIZES`, created on first use.
    faces: [Vec<Option<Font>>; 2],
    k: f32,
    /// Glyphs still to rasterize: (title, index in SIZES, character).
    queue: Vec<(bool, usize, char)>,
}

impl Fonts {
    fn get(&mut self, title: bool, idx: usize) -> &Font {
        let slot = &mut self.faces[title as usize][idx];
        slot.get_or_insert_with(|| {
            let mut f = load_ttf_font_from_bytes(if title { TITLE_TTF } else { BODY_TTF })
                .expect("embedded font");
            f.set_filter(FilterMode::Linear);
            f
        })
    }
}

fn size_index(size: u16) -> usize {
    SIZES.iter().position(|&s| s == size).unwrap_or_else(|| {
        SIZES
            .iter()
            .enumerate()
            .min_by_key(|(_, v)| (**v as i32 - size as i32).abs())
            .map(|(i, _)| i)
            .unwrap_or(4)
    })
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

/// Rebuilds the fonts only once the pixel scale has settled. While a window is being
/// resized or goes fullscreen it reports several sizes in a row; rebuilding the fonts for
/// each of them used to freeze the game for a moment.
#[derive(Default)]
pub struct Scaler {
    pending: f32,
    stable_t: f32,
}

impl Scaler {
    pub fn update(&mut self, k: f32, dt: f32) {
        let have = FONTS.with(|f| f.borrow().as_ref().map(|f| f.k));
        let Some(cur) = have else {
            // First frame: build right away.
            set_pixel_scale(k);
            self.pending = k;
            return;
        };
        let target = quantize(k);
        if (target - cur).abs() < 0.001 {
            self.stable_t = 0.0;
            self.pending = target;
            return;
        }
        if (target - self.pending).abs() > 0.001 {
            self.pending = target;
            self.stable_t = 0.0;
        }
        self.stable_t += dt;
        if self.stable_t > 0.25 {
            set_pixel_scale(k);
        }
    }
}

fn quantize(k_raw: f32) -> f32 {
    ((k_raw * 8.0).round() / 8.0).clamp(0.75, 4.5)
}

/// Sets how many physical pixels exist per logical unit (called every frame).
/// When the scale changes, the fonts are recreated (new atlas) and every glyph is
/// pre-rasterized at the exact screen size, before anything is drawn that frame.
pub fn set_pixel_scale(k_raw: f32) {
    let k = quantize(k_raw);
    let same = FONTS.with(|f| {
        f.borrow()
            .as_ref()
            .map(|f| (f.k - k).abs() < 0.001)
            .unwrap_or(false)
    });
    if same {
        return;
    }
    let t0 = std::time::Instant::now();
    let mut fonts = Fonts {
        faces: [
            (0..SIZES.len()).map(|_| None).collect(),
            (0..SIZES.len()).map(|_| None).collect(),
        ],
        k,
        queue: Vec::new(),
    };
    let chars: Vec<char> = charset().chars().collect();
    let text: String = chars.iter().collect();
    // Small body text (HUD and menus) is ready right away. Medium sizes are queued and
    // rasterized a little every frame by `warm_step`; big titles only ever use a few
    // letters, so they are rasterized when first drawn.
    // Glyphs bigger than this (in screen pixels) are only rasterized when drawn: they
    // belong to titles that use a handful of letters, and pre-rasterizing whole alphabets
    // at that size would make the atlases grow again and again.
    const MAX_WARM_PX: u16 = 72;
    for (i, &s) in SIZES.iter().enumerate().rev() {
        let (fs, sc) = raster(s, BODY_SF, k);
        if s <= 13 && fs <= MAX_WARM_PX {
            // the tiniest HUD text is needed on the very first frame
            let font = fonts.get(false, i);
            measure_text(&text, Some(font), fs, sc);
        } else if s <= 38 && fs <= MAX_WARM_PX {
            fonts.queue.extend(chars.iter().map(|c| (false, i, *c)));
        }
        let (tfs, _) = raster(s, TITLE_SF, k);
        if (14..=30).contains(&s) && tfs <= MAX_WARM_PX {
            fonts.queue.extend(chars.iter().map(|c| (true, i, *c)));
        }
    }
    if crate::dev_env("STARDUST_FONTBENCH").is_some() {
        eprintln!(
            "FONTS k={k} took {:?}, {} glyphs queued",
            t0.elapsed(),
            fonts.queue.len()
        );
    }
    FONTS.with(|f| *f.borrow_mut() = Some(fonts));
}

/// True when every queued glyph has been rasterized.
pub fn warm_done() -> bool {
    FONTS.with(|f| {
        f.borrow()
            .as_ref()
            .map(|f| f.queue.is_empty())
            .unwrap_or(false)
    })
}

/// Rasterizes queued glyphs for up to `budget` (call once per frame, before drawing).
pub fn warm_step(budget: std::time::Duration) {
    let t0 = std::time::Instant::now();
    FONTS.with(|f| {
        let mut f = f.borrow_mut();
        let Some(fonts) = f.as_mut() else { return };
        let k = fonts.k;
        let mut buf = [0u8; 4];
        while let Some((title, idx, c)) = fonts.queue.pop() {
            let (fs, sc) = raster(SIZES[idx], if title { TITLE_SF } else { BODY_SF }, k);
            let font = fonts.get(title, idx);
            measure_text(c.encode_utf8(&mut buf), Some(font), fs, sc);
            if t0.elapsed() >= budget {
                break;
            }
        }
    });
}

fn raster(size: u16, sf: f32, k: f32) -> (u16, f32) {
    let logical = size as f32 * sf;
    let fs = (logical * k).round().max(4.0) as u16;
    (fs, logical / fs as f32)
}

fn with_font<R>(title: bool, size: u16, f: impl FnOnce(Option<&Font>, f32) -> R) -> R {
    FONTS.with(|fonts| {
        let mut fonts = fonts.borrow_mut();
        match fonts.as_mut() {
            Some(x) => {
                let k = x.k;
                f(Some(x.get(title, size_index(size))), k)
            }
            None => f(None, 1.0),
        }
    })
}

fn measure(s: &str, size: u16, title: bool) -> f32 {
    measure_raw(&crate::i18n::tr(s), size, title)
}

/// Like `measure`, for text that is already translated.
fn measure_raw(s: &str, size: u16, title: bool) -> f32 {
    with_font(title, size, |font, k| {
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
    font_text(&crate::i18n::tr(s), x, y, size, color, title);
}

/// Like `raw_font`, for text that is already translated.
fn font_text(s: &str, x: f32, y: f32, size: u16, color: Color, title: bool) {
    with_font(title, size, |font, k| {
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
        hot(
            Rect::new(x - 160.0, yy - size - 4.0, 320.0, spacing.min(size + 16.0)),
            Hot::Row(i),
        );
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

// ---------------------------------------------------------------------------
// Mouse: clickable areas and the cursor
// ---------------------------------------------------------------------------

/// What a clickable area does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hot {
    /// A menu entry: hovering selects it, clicking selects and confirms.
    Row(usize),
    /// A setting row: hovering selects it, clicking does not confirm.
    Select(usize),
    /// The "<" and ">" arrows of a row.
    Left(usize),
    Right(usize),
    /// A tab header.
    Tab(usize),
    /// A button that confirms (launch, buy, continue...).
    Confirm,
    /// A button that goes back.
    Back,
}

type HotList = Vec<(Rect, Hot)>;

thread_local! {
    /// (areas of the last finished frame, areas being registered this frame)
    static HOTS: std::cell::RefCell<(HotList, HotList)> =
        const { std::cell::RefCell::new((Vec::new(), Vec::new())) };
}

/// Registers a clickable area (in play-area coordinates) for the frame being drawn.
pub fn hot(r: Rect, h: Hot) {
    HOTS.with(|v| v.borrow_mut().1.push((r, h)));
}

/// Call once per frame, before drawing: the areas drawn last frame become the live ones.
pub fn hot_frame() {
    HOTS.with(|v| {
        let mut v = v.borrow_mut();
        let (cur, next) = &mut *v;
        std::mem::swap(cur, next);
        next.clear();
    });
}

/// The topmost clickable area under `p`.
pub fn hot_at(p: Vec2) -> Option<Hot> {
    HOTS.with(|v| {
        v.borrow()
            .0
            .iter()
            .rev()
            .find(|(r, _)| r.contains(p))
            .map(|(_, h)| *h)
    })
}

/// A small clickable button with a label; returns nothing, the click arrives as input.
pub fn button(label: &str, cx: f32, y: f32, size: f32, h: Hot, hovered: bool) {
    let w = width(label, size) + 28.0;
    let r = Rect::new(cx - w / 2.0, y - size - 6.0, w, size + 14.0);
    let c = if hovered { GOLD } else { DIM };
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.08, 0.06, 0.18, 0.85));
    draw_rectangle_lines(r.x, r.y, r.w, r.h, if hovered { 2.0 } else { 1.0 }, c);
    text_c(label, cx, y, size, if hovered { WHITE } else { c });
    hot(r, h);
}

/// The mouse cursor: a small pixel-art star pointer in the game's palette. `pos` is in
/// window coordinates and `scale` is the size of one pixel.
pub fn cursor(pos: Vec2, scale: f32, time: f32) {
    const ROWS: [&str; 12] = [
        "k.........",
        "kk........",
        "kwk.......",
        "kwck......",
        "kwcck.....",
        "kwccck....",
        "kwcccck...",
        "kwccckkk..",
        "kwkkck....",
        "kk..kck...",
        "k....kck..",
        ".....kk...",
    ];
    let px = scale.max(1.0).round();
    for (y, row) in ROWS.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let c = match ch {
                'k' => Color::from_rgba(18, 14, 34, 255),
                'w' => WHITE,
                'c' => CYAN,
                _ => continue,
            };
            draw_rectangle(pos.x + x as f32 * px, pos.y + y as f32 * px, px, px, c);
        }
    }
    // a twinkling star at the tip
    let a = 0.5 + (time * 5.0).sin() * 0.5;
    let s = px * (1.5 + a);
    let c = Color::new(1.0, 0.9, 0.5, 0.5 + a * 0.5);
    draw_rectangle(pos.x - s * 0.5, pos.y - px * 0.5, s, px, c);
    draw_rectangle(pos.x - px * 0.5, pos.y - s * 0.5, px, s, c);
}

// ---------------------------------------------------------------------------
// Input hints: keys or gamepad buttons, whichever the player is using
// ---------------------------------------------------------------------------

/// Labels for the hint tokens, for the keyboard and for the gamepad.
#[derive(Clone, Default)]
pub struct HintLabels {
    pub pad: bool,
    /// (token, keyboard label, gamepad label)
    pub map: Vec<(&'static str, String, String)>,
}

thread_local! {
    static HINTS: std::cell::RefCell<HintLabels> = std::cell::RefCell::new(HintLabels::default());
}

/// Sets which device the hints show and the labels of every token.
pub fn set_hints(h: HintLabels) {
    HINTS.with(|v| *v.borrow_mut() = h);
}

pub fn pad_mode() -> bool {
    HINTS.with(|v| v.borrow().pad)
}

fn token_label(tok: &str) -> Option<(String, bool)> {
    HINTS.with(|v| {
        let v = v.borrow();
        v.map.iter().find(|(t, _, _)| *t == tok).map(|(_, k, p)| {
            if v.pad {
                (p.clone(), true)
            } else {
                (crate::i18n::tr(k), false)
            }
        })
    })
}

/// Splits a hint into plain text and tokens like `{CONFIRM}`.
fn hint_parts(s: &str) -> Vec<(String, bool)> {
    let mut out = vec![];
    let mut rest = s;
    while let Some(a) = rest.find('{') {
        let Some(b) = rest[a..].find('}') else { break };
        if a > 0 {
            out.push((rest[..a].to_string(), false));
        }
        out.push((rest[a + 1..a + b].to_string(), true));
        rest = &rest[a + b + 1..];
    }
    if !rest.is_empty() {
        out.push((rest.to_string(), false));
    }
    out
}

/// Color of a gamepad button glyph.
fn pad_color(label: &str) -> Option<Color> {
    match label {
        "A" => Some(Color::from_rgba(90, 200, 90, 255)),
        "B" => Some(Color::from_rgba(230, 80, 80, 255)),
        "X" => Some(Color::from_rgba(80, 140, 240, 255)),
        "Y" => Some(Color::from_rgba(240, 200, 60, 255)),
        _ => None,
    }
}

/// Width of a key cap or button glyph.
fn glyph_w(label: &str, pad: bool, size: f32) -> f32 {
    let s = size * 0.78;
    if pad && pad_color(label).is_some() {
        size * 1.1
    } else {
        measure_raw(label, snap(s), false) + size * 0.7
    }
}

/// Draws one key cap (keyboard) or button (gamepad) with its left edge at `x`.
fn glyph(label: &str, pad: bool, x: f32, y: f32, size: f32, color: Color) -> f32 {
    let w = glyph_w(label, pad, size);
    let h = size * 1.05;
    let top = y - size * 0.86;
    let s = size * 0.78;
    if pad && let Some(c) = pad_color(label) {
        let r = h * 0.52;
        let cx = x + w / 2.0;
        let cy = top + h / 2.0;
        draw_circle(
            cx,
            cy,
            r,
            Color::new(c.r * 0.35, c.g * 0.35, c.b * 0.35, color.a),
        );
        draw_circle_lines(cx, cy, r, 1.5, Color::new(c.r, c.g, c.b, color.a));
        let lw = measure_raw(label, snap(s), false);
        font_text(
            label,
            cx - lw / 2.0,
            y - size * 0.06,
            snap(s),
            Color::new(c.r, c.g, c.b, color.a),
            false,
        );
    } else {
        let fill = Color::new(0.12, 0.12, 0.22, 0.9 * color.a);
        let edge = Color::new(color.r, color.g, color.b, color.a * 0.9);
        if pad {
            // a pill for the other gamepad buttons (D-PAD, START, LB...)
            let r = h / 2.0;
            draw_rectangle(x + r, top, w - r * 2.0, h, fill);
            draw_circle(x + r, top + r, r, fill);
            draw_circle(x + w - r, top + r, r, fill);
        } else {
            // a key cap
            draw_rectangle(x, top, w, h, fill);
            draw_rectangle_lines(x, top, w, h, 1.0, edge);
            draw_line(
                x + 1.0,
                top + h - 1.5,
                x + w - 1.0,
                top + h - 1.5,
                1.5,
                edge,
            );
        }
        let lw = measure_raw(label, snap(s), false);
        font_text(
            label,
            x + w / 2.0 - lw / 2.0,
            y - size * 0.06,
            snap(s),
            color,
            false,
        );
    }
    w
}

/// Draws a hint where tokens like `{CONFIRM}` become key caps or gamepad buttons.
/// `align`: -1 left at `x`, 0 centered on `x`, 1 right at `x`.
pub fn hint(s: &str, x: f32, y: f32, size: f32, color: Color, align: i32) {
    let s = crate::i18n::tr(s);
    let parts: Vec<(String, Option<(String, bool)>)> = hint_parts(&s)
        .into_iter()
        .map(|(t, is_tok)| {
            if is_tok {
                match token_label(&t) {
                    Some(l) => (t, Some(l)),
                    None => (format!("{{{t}}}"), None),
                }
            } else {
                (t, None)
            }
        })
        .collect();
    let gap = 3.0;
    let total: f32 = parts
        .iter()
        .map(|(t, l)| match l {
            Some((lab, pad)) => glyph_w(lab, *pad, size) + gap * 2.0,
            None => measure_raw(t, snap(size), false),
        })
        .sum();
    let mut cx = match align {
        a if a < 0 => x,
        0 => x - total / 2.0,
        _ => x - total,
    };
    for (t, l) in parts {
        match l {
            Some((lab, pad)) => cx += glyph(&lab, pad, cx + gap, y, size, color) + gap * 2.0,
            None => {
                raw_text_left(&t, cx, y, size, color);
                cx += measure_raw(&t, snap(size), false);
            }
        }
    }
}

pub fn hint_c(s: &str, x: f32, y: f32, size: f32, color: Color) {
    hint(s, x, y, size, color, 0);
}

pub fn hint_r(s: &str, x: f32, y: f32, size: f32, color: Color) {
    hint(s, x, y, size, color, 1);
}

/// Draws already-translated text at its left edge (no second translation).
fn raw_text_left(s: &str, x: f32, y: f32, size: f32, color: Color) {
    let fs = snap(size);
    let sh = (fs as f32 / 16.0).clamp(0.75, 2.0);
    font_text(
        s,
        x + sh,
        y + sh,
        fs,
        Color::new(0.0, 0.0, 0.0, color.a * 0.7),
        false,
    );
    font_text(s, x, y, fs, color, false);
}
