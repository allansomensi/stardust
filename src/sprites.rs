//! The game's pixel art, drawn as text. Each letter is a palette color.
//! "Mirrored" sprites are written as the left half + center column only.

use macroquad::prelude::*;

pub(crate) fn palette(c: char) -> Option<[u8; 4]> {
    Some(match c {
        'k' => [18, 14, 34, 255],
        'w' => [255, 255, 255, 255],
        'g' => [192, 202, 218, 255],
        'G' => [108, 116, 140, 255],
        'c' => [120, 232, 255, 255],
        'C' => [52, 132, 232, 255],
        'b' => [28, 50, 120, 255],
        'y' => [255, 232, 100, 255],
        'Y' => [205, 150, 35, 255],
        'o' => [255, 150, 45, 255],
        'r' => [238, 62, 62, 255],
        'R' => [132, 24, 42, 255],
        'p' => [255, 122, 196, 255],
        'v' => [186, 112, 255, 255],
        'V' => [96, 46, 168, 255],
        'l' => [164, 242, 92, 255],
        'L' => [56, 172, 82, 255],
        'D' => [24, 92, 56, 255],
        't' => [72, 226, 200, 255],
        'T' => [20, 116, 122, 255],
        'n' => [150, 96, 56, 255],
        's' => [214, 170, 120, 255],
        'm' => [236, 72, 236, 255],
        'M' => [116, 26, 122, 255],
        'e' => [255, 52, 92, 255],
        _ => return None,
    })
}

pub(crate) fn mirror(half: &[&str]) -> Vec<String> {
    half.iter()
        .map(|r| {
            let ch: Vec<char> = r.chars().collect();
            let mut s: String = r.to_string();
            for c in ch[..ch.len() - 1].iter().rev() {
                s.push(*c);
            }
            s
        })
        .collect()
}

pub(crate) fn plain(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|s| s.to_string()).collect()
}

pub(crate) fn recolor(rows: &[String], map: &[(char, char)]) -> Vec<String> {
    rows.iter()
        .map(|r| {
            r.chars()
                .map(|c| {
                    map.iter()
                        .find(|(a, _)| *a == c)
                        .map(|(_, b)| *b)
                        .unwrap_or(c)
                })
                .collect()
        })
        .collect()
}

/// Converts text rows into RGBA. Returns (width, height, bytes).
pub fn rasterize(rows: &[String]) -> (u16, u16, Vec<u8>) {
    let h = rows.len();
    let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0);
    let mut bytes = vec![0u8; w * h * 4];
    for (y, r) in rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            if let Some(px) = palette(c) {
                bytes[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&px);
            }
        }
    }
    (w as u16, h as u16, bytes)
}

pub struct Spr {
    pub tex: Texture2D,
    pub white: Texture2D,
    pub w: f32,
    pub h: f32,
}

impl Spr {
    pub(crate) fn new(rows: Vec<String>) -> Spr {
        let (w, h, bytes) = rasterize(&rows);
        let white_bytes: Vec<u8> = bytes
            .chunks(4)
            .flat_map(|p| {
                if p[3] > 0 {
                    [255, 255, 255, 255]
                } else {
                    [0, 0, 0, 0]
                }
            })
            .collect();
        let tex = Texture2D::from_rgba8(w, h, &bytes);
        tex.set_filter(FilterMode::Nearest);
        let white = Texture2D::from_rgba8(w, h, &white_bytes);
        white.set_filter(FilterMode::Nearest);
        Spr {
            tex,
            white,
            w: w as f32,
            h: h as f32,
        }
    }

    /// Draws centered at `pos`.
    pub fn draw(&self, pos: Vec2, scale: f32, rot: f32, color: Color, flash: bool) {
        let size = vec2(self.w, self.h) * scale;
        let tex = if flash { &self.white } else { &self.tex };
        draw_texture_ex(
            tex,
            (pos.x - size.x / 2.0).round(),
            (pos.y - size.y / 2.0).round(),
            color,
            DrawTextureParams {
                dest_size: Some(size),
                rotation: rot,
                ..Default::default()
            },
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Heart,
    Bomb,
    Magnet,
    Engine,
    Bolt,
    Crystal,
    Shield,
    Star,
    Disc,
    Flame,
    Target,
    Clock,
    Bag,
    Drone,
    Card,
}

pub struct Sprites {
    /// [type][frame]
    pub enemies: Vec<[Spr; 2]>,
    pub queen: [Spr; 2],
    pub claw: Spr,
    pub warship: Spr,
    pub turret: Spr,
    pub cannon: Spr,
    pub asteroid: [Spr; 2],
    pub crystal: [Spr; 2],
    pub crystal_big: [Spr; 2],
    pub card: Spr,
    pub drone: Spr,
    icons: Vec<(Icon, Spr)>,
}

impl Sprites {
    pub fn icon(&self, i: Icon) -> &Spr {
        &self.icons.iter().find(|(k, _)| *k == i).unwrap().1
    }
}

pub fn player_rows() -> Vec<String> {
    mirror(&[
        "......k", //
        ".....kw", //
        ".....kc", //
        "....kcc", //
        "....kcb", //
        "...kCbw", //
        "...kCcb", //
        "k..kCcc", //
        "kk.kCCc", //
        "kgkkCCc", //
        "kgCCCCc", //
        "kgCbCCc", //
        "kkCbkCC", //
        ".kk..kG", //
        "......k", //
    ])
}

fn bug_a() -> Vec<String> {
    mirror(&[
        "...k..", //
        "....k.", //
        "kk.kLL", //
        "klkLlL", //
        "kllkwk", //
        ".kLLLL", //
        "..kLDL", //
        "...kLD", //
        "....kk", //
    ])
}

fn bug_b() -> Vec<String> {
    mirror(&[
        "...k..", //
        "....k.", //
        "...kLL", //
        ".kkLlL", //
        "kllkwk", //
        "klkLLL", //
        "kkkLDL", //
        "...kLD", //
        "....kk", //
    ])
}

pub(crate) fn wasp_a() -> Vec<String> {
    mirror(&[
        "..k...", //
        "...kkk", //
        "kk.kwv", //
        "kvkvvv", //
        "kvvkyy", //
        ".kkkVV", //
        "...kyy", //
        "...kVV", //
        "....ky", //
        ".....k", //
    ])
}

pub(crate) fn wasp_b() -> Vec<String> {
    mirror(&[
        "..k...", //
        "...kkk", //
        "...kwv", //
        ".kkvvv", //
        "kvvkyy", //
        "kvkkVV", //
        "...kyy", //
        "...kVV", //
        "....ky", //
        ".....k", //
    ])
}

pub(crate) fn beetle(frame: usize) -> Vec<String> {
    let legs = if frame == 0 { "k.k.kk" } else { ".k.kkk" };
    mirror(&[
        "...k..", //
        "....kk", //
        "...krr", //
        "..kekr", //
        ".krRRk", //
        "krrRrk", //
        "krrrRk", //
        "kRrrrk", //
        ".kRRRk", //
        legs,
    ])
}

fn shooter(frame: usize) -> Vec<String> {
    let (a, b) = if frame == 0 {
        ("kGk...", ".k....")
    } else {
        ("kGk...", "kyk...")
    };
    mirror(&[
        "..kkkk", //
        ".kyyoo", //
        "kyokwk", //
        "kooooo", //
        "kYoYoY", //
        ".kYYYk", //
        "kgk.k.", //
        a, b,
    ])
}

fn kamikaze(frame: usize) -> Vec<String> {
    let top = if frame == 0 { "kk..." } else { "ko..." };
    mirror(&[
        top,     //
        "kpk..", //
        "kppk.", //
        ".kppk", //
        ".kpMp", //
        "..kpw", //
        "..kMp", //
        "...kp", //
        "....k", //
    ])
}

fn mini(frame: usize) -> Vec<String> {
    if frame == 0 {
        mirror(&[".k..", "kktt", "ktwk", ".ktt", "..kT"])
    } else {
        mirror(&["k...", ".ktt", "ktwk", "kktt", "..kT"])
    }
}

fn queen(frame: usize) -> Vec<String> {
    let mut rows: Vec<&str> = vec![
        ".........k.k", //
        ".........kyk", //
        "........kyyy", //
        ".......kmmmm", //
        "......kmmmmm", //
        "......kmeemm", //
    ];
    if frame == 0 {
        rows.extend([
            ".kk...kMmmmm", //
            "kvvk..kMMmmm", //
            "kvvvkkkMMMmm", //
            "kvVvvkmMMMMm", //
            ".kvVvkmMyMMM", //
            "..kvvkmMMMMM", //
            "...kkkkMyMMM", //
        ]);
    } else {
        rows.extend([
            "......kMmmmm", //
            ".kk...kMMmmm", //
            "kvvkkkkMMMmm", //
            "kvvvvkmMMMMm", //
            "kvVVvkmMyMMM", //
            ".kvvvkmMMMMM", //
            "..kkkkkMyMMM", //
        ]);
    }
    rows.extend([
        ".....kMMMMMM", //
        "......kMMyMM", //
        ".......kMMMM", //
        "........kkMM", //
        "..........kk", //
    ]);
    mirror(&rows)
}

fn warship() -> Vec<String> {
    mirror(&[
        "...........kk", //
        "..........kgg", //
        ".........kgGG", //
        "kkk.....kgGbb", //
        "kgGk...kgGbcc", //
        "kgGGkkkgGGbcw", //
        "kgGGGGGGGGbbc", //
        "kGrGGGGGGGGbb", //
        ".kGGGGGGGGGGG", //
        "..kGGkGGGGkGG", //
        "...kk.kGGGGkG", //
        "......kGkkkGG", //
        ".......k...kk", //
    ])
}

pub fn build() -> Sprites {
    let s = |rows: Vec<String>| Spr::new(rows);
    let pair = |a: Vec<String>, b: Vec<String>| [Spr::new(a), Spr::new(b)];

    let bug_green = pair(bug_a(), bug_b());
    let wasp = pair(wasp_a(), wasp_b());
    let red = pair(beetle(0), beetle(1));
    let shoot = pair(shooter(0), shooter(1));
    let teal_map = [('r', 't'), ('R', 'T'), ('e', 'y')];
    let divisor = pair(
        recolor(&beetle(0), &teal_map),
        recolor(&beetle(1), &teal_map),
    );
    let kami = pair(kamikaze(0), kamikaze(1));
    let mini = pair(mini(0), mini(1));

    let crystal_a = plain(&[
        "..k..", ".kck.", "kcwck", "kccCk", "kcCCk", ".kCk.", "..k..",
    ]);
    let crystal_b = plain(&[
        "..k..", ".kck.", "kccck", "kcwCk", "kcCCk", ".kCk.", "..k..",
    ]);
    let big_c = [('c', 'w'), ('C', 'c')];

    let icons = vec![
        (
            Icon::Heart,
            s(plain(&[
                ".kk...kk.",
                "krrk.krrk",
                "krwrkrrrk",
                "krrrrrrrk",
                ".krrrrrk.",
                "..krrrk..",
                "...krk...",
                "....k....",
            ])),
        ),
        (
            Icon::Bomb,
            s(plain(&[
                "......yo.",
                ".....k...",
                "...kkkk..",
                "..kGGGGk.",
                ".kGwGGGGk",
                ".kGGGGGGk",
                ".kGGGGGGk",
                "..kGGGGk.",
                "...kkkk..",
            ])),
        ),
        (
            Icon::Magnet,
            s(plain(&[
                "kgk...kgk",
                "kgk...kgk",
                "krk...krk",
                "krk...krk",
                "krk...krk",
                "krrk.krrk",
                ".krrrrrk.",
                "..kkkkk..",
            ])),
        ),
        (
            Icon::Engine,
            s(plain(&[
                "....k....",
                "...kck...",
                "..kcCck..",
                ".kcCCCck.",
                "kkkcCckkk",
                "...kCk...",
                "...yoy...",
                "....o....",
            ])),
        ),
        (
            Icon::Bolt,
            s(plain(&[
                "....kkk..",
                "...kyyk..",
                "..kyyk...",
                ".kyyykkk.",
                ".kkkyyyk.",
                "...kyyk..",
                "..kyyk...",
                "..kyk....",
                "..kk.....",
            ])),
        ),
        (Icon::Crystal, s(crystal_a.clone())),
        (
            Icon::Shield,
            s(plain(&[
                ".kkkkkkk.",
                "kcwcccCCk",
                "kcccccCCk",
                "kcccccCCk",
                ".kccccCk.",
                ".kcccCCk.",
                "..kccCk..",
                "...kCk...",
                "....k....",
            ])),
        ),
        (
            Icon::Star,
            s(plain(&[
                "....k....",
                "...kyk...",
                "...kyk...",
                "kkkyyykkk",
                "kyyywyyyk",
                ".kyyyyyk.",
                "..kyyyk..",
                ".kyykyyk.",
                ".kyk.kyk.",
                ".kk...kk.",
            ])),
        ),
        (
            Icon::Disc,
            s(plain(&[
                "..kkkkk..",
                ".kyyyyyk.",
                "kyywyyyYk",
                "kyyykyyYk",
                "kyykwkyYk",
                "kyyykyyYk",
                "kyyyyyYYk",
                ".kYYYYYk.",
                "..kkkkk..",
            ])),
        ),
        (
            Icon::Flame,
            s(plain(&[
                "....k....",
                "...kok...",
                "..koyok..",
                "..koyok..",
                ".koyyyok.",
                "koyywyyok",
                "koyyyyyok",
                ".koyyyok.",
                "..kkkkk..",
            ])),
        ),
        (
            Icon::Target,
            s(plain(&[
                "...kkk...",
                "..k.r.k..",
                ".k..r..k.",
                "krrrwrrrk",
                ".k..r..k.",
                "..k.r.k..",
                "...kkk...",
            ])),
        ),
        (
            Icon::Clock,
            s(plain(&[
                "..kkkkk..",
                ".kwwwwwk.",
                "kwwwkwwwk",
                "kwwwkwwwk",
                "kwwwkkkwk",
                "kwwwwwwwk",
                ".kwwwwwk.",
                "..kkkkk..",
            ])),
        ),
        (
            Icon::Bag,
            s(plain(&[
                "...kkk...",
                "....k....",
                "..kyyyk..",
                ".kyYYYyk.",
                "kyyYyyyyk",
                "kyyyYYyyk",
                "kyyyyyYyk",
                ".kyYYYyk.",
                "..kkkkk..",
            ])),
        ),
        (
            Icon::Drone,
            s(plain(&[
                "..kkk..", ".kcCck.", "kcwCCck", "kCCCCCk", ".kCbCk.", "..kGk..", "...k...",
            ])),
        ),
        (
            Icon::Card,
            s(plain(&[
                "kkkkkkk", "kyyyyyk", "kyoooyk", "kyowoyk", "kyoooyk", "kyyyyyk", "kyYyYyk",
                "kyyyyyk", "kkkkkkk",
            ])),
        ),
    ];

    Sprites {
        enemies: vec![bug_green, wasp, red, shoot, divisor, kami, mini],
        queen: pair(queen(0), queen(1)),
        claw: s(plain(&[
            "..kkk..", ".kmmmk.", "kmMmMmk", "kmMMMmk", ".kmMmk.", ".kmkmk.", "kmk.kmk", "kk...kk",
        ])),
        warship: s(warship()),
        turret: s(plain(&[
            ".kkkkk.", "kgGGGgk", "kGbcbGk", "kGcwcGk", "kGbcbGk", "kgGGGgk", ".kkGkk.", "..kGk..",
            "..kkk..",
        ])),
        cannon: s(plain(&[
            "kkkkkkkkk",
            "kGGgggGGk",
            "kGrrrrrGk",
            ".kGrerGk.",
            "..kGeGk..",
            "...kek...",
        ])),
        asteroid: [
            s(plain(&[
                "..kkkkk..",
                ".knnsnnk.",
                "knsnnnGnk",
                "knnnGnnnk",
                "kGnnnnsnk",
                "knnGnnnnk",
                ".knnnGnk.",
                "..kknnk..",
                "....kk...",
            ])),
            s(plain(&[
                "...kkkk..",
                "..knnnnk.",
                ".knGnnsnk",
                "knnnnnnnk",
                "knsnnGnk.",
                "kGnnnnnk.",
                ".knnsnnk.",
                "..kkkkk..",
            ])),
        ],
        crystal: [s(crystal_a.clone()), s(crystal_b.clone())],
        crystal_big: [
            s(recolor(&crystal_a, &big_c)),
            s(recolor(&crystal_b, &big_c)),
        ],
        card: s(plain(&[
            "kkkkkkk", "kyyyyyk", "kyoooyk", "kyowoyk", "kyoooyk", "kyyyyyk", "kyYyYyk", "kyyyyyk",
            "kkkkkkk",
        ])),
        drone: s(plain(&[
            "..kkk..", ".kcCck.", "kcwCCck", "kCCCCCk", ".kCbCk.", "..kGk..", "...k...",
        ])),
        icons,
    }
}

/// Window icon generated from the player's ship.
pub fn window_icon_rgba(size: usize) -> Vec<u8> {
    let (w, h, src) = rasterize(&player_rows());
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; size * size * 4];
    let bg = [24u8, 18, 48, 255];
    let scale = (size as f32 * 0.8) / (w.max(h) as f32);
    let ox = (size as f32 - w as f32 * scale) / 2.0;
    let oy = (size as f32 - h as f32 * scale) / 2.0;
    for y in 0..size {
        for x in 0..size {
            // rounded background
            let cx = x as f32 + 0.5 - size as f32 / 2.0;
            let cy = y as f32 + 0.5 - size as f32 / 2.0;
            let r = size as f32 / 2.0;
            let corner = r * 0.75;
            let dx = (cx.abs() - corner).max(0.0);
            let dy = (cy.abs() - corner).max(0.0);
            let inside = dx * dx + dy * dy <= (r - corner) * (r - corner);
            let i = (y * size + x) * 4;
            if inside {
                out[i..i + 4].copy_from_slice(&bg);
            }
            let sx = ((x as f32 - ox) / scale).floor();
            let sy = ((y as f32 - oy) / scale).floor();
            if sx >= 0.0 && sy >= 0.0 && (sx as usize) < w && (sy as usize) < h {
                let j = (sy as usize * w + sx as usize) * 4;
                if src[j + 3] > 0 {
                    out[i..i + 4].copy_from_slice(&src[j..j + 4]);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(name: &str, rows: &[String]) {
        let w = rows[0].chars().count();
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(
                r.chars().count(),
                w,
                "{name}: row {i} has a different width"
            );
            for c in r.chars() {
                assert!(
                    c == '.' || palette(c).is_some(),
                    "{name}: cor desconhecida '{c}'"
                );
            }
        }
    }

    #[test]
    fn sprites_well_formed() {
        check("jogador", &player_rows());
        check("drone", &bug_a());
        check("drone b", &bug_b());
        check("vespa", &wasp_a());
        check("vespa b", &wasp_b());
        for f in 0..2 {
            check("besouro", &beetle(f));
            check("atirador", &shooter(f));
            check("kamikaze", &kamikaze(f));
            check("mini", &mini(f));
            check("rainha", &queen(f));
        }
        check("warship", &warship());
    }

    #[test]
    fn icon_has_right_size() {
        for n in [16, 32, 64] {
            assert_eq!(window_icon_rgba(n).len(), n * n * 4);
        }
    }
}
