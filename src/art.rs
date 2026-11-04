//! Arte da versão 2: naves do hangar, skins, novos inimigos e novos chefes.

use crate::sprites::{Spr, Sprites, beetle, mirror, plain, player_rows, recolor, wasp_a, wasp_b};

pub const SKINS: usize = 7;

/// Trocas de paleta para cada skin (aplicadas sobre c / C / b).
fn skin_map(skin: usize) -> Option<[(char, char); 3]> {
    Some(match skin {
        1 => [('c', 'p'), ('C', 'r'), ('b', 'R')],
        2 => [('c', 'l'), ('C', 'L'), ('b', 'D')],
        3 => [('c', 'y'), ('C', 'Y'), ('b', 'n')],
        4 => [('c', 'G'), ('C', 'V'), ('b', 'k')],
        5 => [('c', 'w'), ('C', 'c'), ('b', 'C')],
        6 => [('c', 'm'), ('C', 'v'), ('b', 'M')],
        _ => return None,
    })
}

fn falcon() -> Vec<String> {
    mirror(&[
        "......k", ".....kw", ".....kc", ".....kc", "....kcb", "....kCw", "k...kCc", "kk..kCc",
        "kCk.kCc", ".kCkkCc", "..kCCCc", "...kCCC", "....kGk", "....k.k",
    ])
}

fn bastion() -> Vec<String> {
    mirror(&[
        "....kkk", "...kggw", "...kgcc", "..kgCcb", "kkkgCcb", "kgggCCc", "kgGgCCc", "kgGGgCc",
        "kgGGGgC", "kgGkGGg", "kkk.kGG", "....kkk",
    ])
}

fn lyre() -> Vec<String> {
    mirror(&[
        "kk....k", "kck..kw", "kck..kc", ".kck.kc", ".kck.kb", "..kckkb", "..kcCcw", "..kCCCc",
        ".kCCbCc", "kCCkkCC", "kkk..kk",
    ])
}

fn ghost() -> Vec<String> {
    mirror(&[
        "......k", ".....kw", "....kwc", "...kwcc", "..kwccb", ".kwccbw", "kwcccbb", "kcccccc",
        "kccCccc", ".kCkCcC", "kk.kk.k",
    ])
}

fn colossus() -> Vec<String> {
    mirror(&[
        "..kk..k", "..kg.kc", "..kg.kc", ".kkgkcc", "kcCgkcb", "kcCCkcb", "kcCCCcw", "kcCCCCc",
        "kCCbCCc", "kCbbbCC", "kCCCCkk", ".kkkk..",
    ])
}

fn prototype() -> Vec<String> {
    mirror(&[
        "......c", ".....cc", ".....cc", "....ccc", "....cCb", "...ccCb", "C..ccCc", "CC.ccCc",
        "CCCccCc", "CCCCCCc", "g....CC",
    ])
}

fn probe() -> Vec<String> {
    mirror(&[
        "......k", ".....kw", "....kcc", "....kcb", "kkkkkCc", "kbkbkCc", "kbkbkCw", "kkkkkCc",
        "....kCC", "...kCkk", "...kk..",
    ])
}

fn corsair() -> Vec<String> {
    mirror(&[
        "......k", ".....kc", ".....kc", "k...kcw", "kk..kcc", ".kk.kCc", "..kkCCb", "..kCCCc",
        ".kCCkCC", "kCCk.kk", "kkk....",
    ])
}

fn aegis() -> Vec<String> {
    mirror(&[
        "...kkkk", "..kcccw", ".kcCCcc", "kcCCbbc", "kcCbbbC", "kcCCbbC", ".kcCCCc", "..kkCCk",
        "....kkk", "....kGk",
    ])
}

/// Linhas base de cada nave (mesma ordem de ships::SHIPS) + paleta padrão.
fn ship_rows(i: usize) -> (Vec<String>, Vec<(char, char)>) {
    match i {
        0 => (player_rows(), vec![]),
        1 => (falcon(), vec![('c', 'y'), ('C', 'o'), ('b', 'R')]),
        2 => (bastion(), vec![('c', 't'), ('C', 'T'), ('b', 'b')]),
        3 => (lyre(), vec![('c', 'p'), ('C', 'm'), ('b', 'M')]),
        4 => (ghost(), vec![('c', 'g'), ('C', 'V'), ('b', 'k')]),
        5 => (colossus(), vec![('c', 'o'), ('C', 'r'), ('b', 'R')]),
        6 => (prototype(), vec![('c', 'l'), ('C', 'L')]),
        7 => (probe(), vec![('c', 'w'), ('C', 'g'), ('b', 'y')]),
        8 => (corsair(), vec![('c', 'r'), ('C', 'R'), ('b', 'y')]),
        _ => (aegis(), vec![('c', 't'), ('C', 'l'), ('b', 'w')]),
    }
}

pub fn ship_sprite(ship: usize, skin: usize) -> Spr {
    let (rows, default) = ship_rows(ship);
    let rows = match skin_map(skin) {
        Some(m) => recolor(&rows, &m),
        None => recolor(&rows, &default),
    };
    Spr::new(rows)
}

pub struct Art {
    /// [nave][skin]
    pub ships: Vec<Vec<Spr>>,
    pub plate: Spr,
    pub serpent_head: Spr,
    pub serpent_seg: Spr,
    pub spider: Spr,
    pub leg: Spr,
    pub eye_open: Spr,
    pub eye_closed: Spr,
}

fn shielder(frame: usize) -> Vec<String> {
    let top = if frame == 0 { "....kk" } else { "...k.k" };
    mirror(&[
        top, "...kCC", "..kCwk", ".kCCCC", "kgkCbC", "kgkCCC", ".kkbCC", "..k.kk",
    ])
}

fn weaver(frame: usize) -> Vec<String> {
    if frame == 0 {
        mirror(&[
            "k.....", ".k..kk", "..kkvv", "kkkvwk", "...kvv", "kkkVvV", "..kVgV", ".k.kVV",
            "k...kk",
        ])
    } else {
        mirror(&[
            ".k....", "k...kk", "..kkvv", "kkkvwk", "...kvv", "kkkVvV", "..kVgV", "k..kVV",
            ".k..kk",
        ])
    }
}

fn carrier(frame: usize) -> Vec<String> {
    let sac = if frame == 0 { "..kkooo" } else { "..kkyyy" };
    mirror(&[
        "...k...", "....kkk", "...knnn", "..knwkn", ".knnnnn", "knsnsns", "knnnnnn", "knsnsns",
        ".knnnnn", sac, "....kkk",
    ])
}

fn cow(frame: usize) -> Vec<String> {
    let nose = if frame == 0 {
        ".kppkpkppk."
    } else {
        ".kpppppppk."
    };
    plain(&[
        "kk.......kk",
        "kwk.....kwk",
        ".kkkkkkkkk.",
        "kwwwkwwwwwk",
        "kwkwwwwkwwk",
        "kwwwwwwwwwk",
        "kwkkwwwkkwk",
        ".kpppppppk.",
        nose,
        "..kpppppk..",
        "...kkkkk...",
    ])
}

fn merchant(frame: usize) -> Vec<String> {
    let lights = if frame == 0 { "kgggyggg" } else { "kgggggyg" };
    mirror(&[
        ".....kkk", "....kccc", "...kcwcc", ".kkkkkkk", lights, "kGGGGGGG", ".kkGGGGG", "...kkkkk",
    ])
}

fn mimic(frame: usize) -> Vec<String> {
    let mouth = if frame == 0 { "k.......k" } else { "kkkkkkkkk" };
    plain(&[
        "..kkkkk..",
        ".kwwwwgk.",
        "kweggeGGk",
        "kwgggggGk",
        "kwkwkwkwk",
        mouth,
        "kwkwkwkwk",
        "kgggggGGk",
        ".kGGGGGk.",
        "..kkkkk..",
    ])
}

fn thief(frame: usize) -> Vec<String> {
    let legs = if frame == 0 { "k.k.kk" } else { ".k.k.k" };
    mirror(&[
        "...k..", "....kk", "...kVV", "..kkkk", "..kwkk", "...kVV", "..kVvV", ".kVVVV", legs,
    ])
}

fn minelayer(frame: usize) -> Vec<String> {
    let light = if frame == 0 { "kRrGGe" } else { "kRrGGy" };
    let legs = if frame == 0 { "k.k.kk" } else { ".k.k.k" };
    mirror(&[
        "....kk", "..kkRr", ".kRrrr", light, "kRrrrr", ".kRGkG", legs,
    ])
}

fn jumper(frame: usize) -> Vec<String> {
    let eye = if frame == 0 { ".kmmwk" } else { ".kmmkw" };
    let tail = if frame == 0 { ".kMkkM" } else { "kM.kMk" };
    mirror(&["....kk", "...kmm", "..kmww", eye, "kmmmmm", "kMmmMm", tail])
}

pub fn build(sp: &mut Sprites) -> Art {
    let pair = |a: Vec<String>, b: Vec<String>| [Spr::new(a), Spr::new(b)];
    // Novos inimigos, na ordem do enum Kind (a partir do índice 7)
    sp.enemies.push(pair(shielder(0), shielder(1)));
    sp.enemies.push(pair(weaver(0), weaver(1)));
    sp.enemies.push(pair(carrier(0), carrier(1)));
    let cham = [('v', 'l'), ('V', 'L'), ('y', 't')];
    sp.enemies
        .push(pair(recolor(&wasp_a(), &cham), recolor(&wasp_b(), &cham)));
    let gold = [('r', 'y'), ('R', 'Y'), ('e', 'w')];
    sp.enemies
        .push(pair(recolor(&beetle(0), &gold), recolor(&beetle(1), &gold)));
    sp.enemies.push(pair(cow(0), cow(1)));
    sp.enemies.push(pair(merchant(0), merchant(1)));
    // v3: Mímico, Ladrão e Cargueiro
    sp.enemies.push(pair(mimic(0), mimic(1)));
    sp.enemies.push(pair(thief(0), thief(1)));
    let cargo = [('c', 'o'), ('w', 'y'), ('g', 's'), ('G', 'n')];
    sp.enemies.push(pair(
        recolor(&merchant(0), &cargo),
        recolor(&merchant(1), &cargo),
    ));
    // v5: Minador e Saltador
    sp.enemies.push(pair(minelayer(0), minelayer(1)));
    sp.enemies.push(pair(jumper(0), jumper(1)));

    let ships = (0..crate::ships::SHIPS.len())
        .map(|s| (0..SKINS).map(|k| ship_sprite(s, k)).collect())
        .collect();

    Art {
        ships,
        plate: Spr::new(plain(&["kgggggggggk", "kgGGGGGGGgk", ".kkkkkkkkk."])),
        serpent_head: Spr::new(mirror(&[
            "kk.....", "kok....", ".kok.kk", "..kokrr", "..krrrr", ".krRwkr", ".krRRrr", "krrrrrr",
            "krRrrRr", ".krrrrr", "..kwkrr", "...kk.k",
        ])),
        serpent_seg: Spr::new(mirror(&[
            "..kkk", ".krrr", "krRor", "krrRr", "krRor", ".krrr", "..kkk",
        ])),
        spider: Spr::new(mirror(&[
            "........kkk",
            "......kkvvv",
            ".....kvvvvv",
            "....kvvVVvv",
            "....kvVekev",
            "....kvVVVVV",
            "...kvvvwvvw",
            "...kvVvvwvv",
            "...kvvVvvvw",
            "....kvvVVvv",
            ".....kvvvvv",
            "......kkVVV",
            "........kkk",
        ])),
        leg: Spr::new(plain(&[
            "k......", "kk.....", ".kVk...", "..kVk..", "...kVk.", "....kVk", ".....kk",
        ])),
        eye_open: Spr::new(mirror(&[
            ".......kkkk",
            ".....kkgggg",
            "...kkgggwww",
            "..kggwwwwww",
            ".kgwwwwCCCC",
            "kgwwwwCcccc",
            "kgwwwCcckkk",
            "kgwwwCckkkk",
            "kgwwwCcckkk",
            "kgwwwwCcccc",
            ".kgwwwwCCCC",
            "..kggwwwwww",
            "...kkgggwww",
            ".....kkgggg",
            ".......kkkk",
        ])),
        eye_closed: Spr::new(mirror(&[
            ".......kkkk",
            ".....kkVVVV",
            "...kkVVVVVV",
            "..kVVvVVVVV",
            ".kVVvvvVVVV",
            "kVVvvvvvVVV",
            "kVvvvvvvvvv",
            "kkkkkkkkkkk",
            "kVvvvvvvvvv",
            "kVVvvvvvVVV",
            ".kVVvvvVVVV",
            "..kVVvVVVVV",
            "...kkVVVVVV",
            ".....kkVVVV",
            ".......kkkk",
        ])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(rows: &[String]) {
        let w = rows[0].chars().count();
        assert!(rows.iter().all(|r| r.chars().count() == w));
    }

    #[test]
    fn naves_e_inimigos_bem_formados() {
        for i in 0..10 {
            check(&ship_rows(i).0);
        }
        for f in 0..2 {
            check(&shielder(f));
            check(&weaver(f));
            check(&carrier(f));
            check(&cow(f));
            check(&merchant(f));
            check(&mimic(f));
            check(&thief(f));
            check(&minelayer(f));
            check(&jumper(f));
        }
    }
}
