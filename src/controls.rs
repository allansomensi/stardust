//! Remappable controls. Each player has a keyboard map and a gamepad map; every action has
//! two slots (primary and alternative). Menus keep fixed keys (arrows, Enter, Esc) so the
//! game can always be navigated.

use macroquad::prelude::KeyCode;

pub const ACTIONS: [&str; 6] = ["LEFT", "RIGHT", "SHOOT", "BOMB", "ABILITY", "PRECISION"];
/// Actions that a gamepad button can do (moving always uses the stick and the D-pad).
pub const PAD_ACTIONS: [usize; 4] = [2, 3, 4, 5];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PadButton {
    South,
    East,
    West,
    North,
    LB,
    RB,
    LT,
    RT,
}

pub const PAD_BUTTONS: [PadButton; 8] = [
    PadButton::South,
    PadButton::East,
    PadButton::West,
    PadButton::North,
    PadButton::LB,
    PadButton::RB,
    PadButton::LT,
    PadButton::RT,
];

impl PadButton {
    pub fn index(self) -> usize {
        PAD_BUTTONS.iter().position(|b| *b == self).unwrap_or(0)
    }
    pub fn name(self) -> &'static str {
        match self {
            PadButton::South => "A",
            PadButton::East => "B",
            PadButton::West => "X",
            PadButton::North => "Y",
            PadButton::LB => "LB",
            PadButton::RB => "RB",
            PadButton::LT => "LT",
            PadButton::RT => "RT",
        }
    }
    fn parse(s: &str) -> Option<PadButton> {
        PAD_BUTTONS.iter().copied().find(|b| b.name() == s)
    }
}

/// Keys that can be bound, with the names written in the save file.
const KEYS: &[(KeyCode, &str)] = &[
    (KeyCode::Left, "Left"),
    (KeyCode::Right, "Right"),
    (KeyCode::Up, "Up"),
    (KeyCode::Down, "Down"),
    (KeyCode::Space, "Space"),
    (KeyCode::LeftShift, "LShift"),
    (KeyCode::RightShift, "RShift"),
    (KeyCode::LeftControl, "LCtrl"),
    (KeyCode::RightControl, "RCtrl"),
    (KeyCode::LeftAlt, "LAlt"),
    (KeyCode::RightAlt, "RAlt"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::Comma, "Comma"),
    (KeyCode::Period, "Period"),
    (KeyCode::Slash, "Slash"),
    (KeyCode::Semicolon, "Semicolon"),
    (KeyCode::Apostrophe, "Quote"),
    (KeyCode::LeftBracket, "LBracket"),
    (KeyCode::RightBracket, "RBracket"),
    (KeyCode::Backslash, "Backslash"),
    (KeyCode::Minus, "Minus"),
    (KeyCode::Equal, "Equal"),
    (KeyCode::Insert, "Insert"),
    (KeyCode::Delete, "Delete"),
    (KeyCode::Home, "Home"),
    (KeyCode::End, "End"),
    (KeyCode::PageUp, "PageUp"),
    (KeyCode::PageDown, "PageDown"),
    (KeyCode::Kp0, "Num0"),
    (KeyCode::Kp1, "Num1"),
    (KeyCode::Kp2, "Num2"),
    (KeyCode::Kp3, "Num3"),
    (KeyCode::Kp4, "Num4"),
    (KeyCode::Kp5, "Num5"),
    (KeyCode::Kp6, "Num6"),
    (KeyCode::Kp7, "Num7"),
    (KeyCode::Kp8, "Num8"),
    (KeyCode::Kp9, "Num9"),
    (KeyCode::A, "A"),
    (KeyCode::B, "B"),
    (KeyCode::C, "C"),
    (KeyCode::D, "D"),
    (KeyCode::E, "E"),
    (KeyCode::F, "F"),
    (KeyCode::G, "G"),
    (KeyCode::H, "H"),
    (KeyCode::I, "I"),
    (KeyCode::J, "J"),
    (KeyCode::K, "K"),
    (KeyCode::L, "L"),
    (KeyCode::M, "M"),
    (KeyCode::N, "N"),
    (KeyCode::O, "O"),
    (KeyCode::Q, "Q"),
    (KeyCode::R, "R"),
    (KeyCode::S, "S"),
    (KeyCode::T, "T"),
    (KeyCode::U, "U"),
    (KeyCode::V, "V"),
    (KeyCode::W, "W"),
    (KeyCode::X, "X"),
    (KeyCode::Y, "Y"),
    (KeyCode::Z, "Z"),
    (KeyCode::Key0, "0"),
    (KeyCode::Key1, "1"),
    (KeyCode::Key2, "2"),
    (KeyCode::Key3, "3"),
    (KeyCode::Key4, "4"),
    (KeyCode::Key5, "5"),
    (KeyCode::Key6, "6"),
    (KeyCode::Key7, "7"),
    (KeyCode::Key8, "8"),
    (KeyCode::Key9, "9"),
];

pub fn key_name(k: KeyCode) -> &'static str {
    KEYS.iter()
        .find(|(c, _)| *c == k)
        .map(|(_, n)| *n)
        .unwrap_or("?")
}

/// True if `k` may be bound (Enter, Escape, P and F11 stay reserved for menus and pause).
pub fn bindable(k: KeyCode) -> bool {
    KEYS.iter().any(|(c, _)| *c == k)
}

fn parse_key(s: &str) -> Option<KeyCode> {
    KEYS.iter().find(|(_, n)| *n == s).map(|(c, _)| *c)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Bindings {
    /// [player][action][slot]
    pub keys: [[[Option<KeyCode>; 2]; 6]; 2],
    /// [player][action][slot] for the actions in `PAD_ACTIONS` (indexed by action).
    pub pad: [[[Option<PadButton>; 2]; 6]; 2],
}

impl Default for Bindings {
    fn default() -> Self {
        use KeyCode::*;
        let k = |a: KeyCode, b: Option<KeyCode>| [Some(a), b];
        let p1 = [
            k(Left, Some(A)),
            k(Right, Some(D)),
            k(Space, Some(Z)),
            k(X, Some(G)),
            k(C, Some(H)),
            k(LeftShift, None),
        ];
        let p2 = [
            k(Left, None),
            k(Right, None),
            k(Comma, Some(RightControl)),
            k(Period, Some(Kp2)),
            k(Slash, Some(Kp3)),
            k(RightShift, None),
        ];
        let b = |a: PadButton, c: PadButton| [Some(a), Some(c)];
        let pad = [
            [None, None],
            [None, None],
            b(PadButton::South, PadButton::RT),
            b(PadButton::West, PadButton::East),
            b(PadButton::North, PadButton::RB),
            b(PadButton::LB, PadButton::LT),
        ];
        Bindings {
            keys: [p1, p2],
            pad: [pad, pad],
        }
    }
}

impl Bindings {
    /// Assigns a key: it becomes the primary and the old primary moves to the second slot.
    /// The key is removed from every other action of the same player.
    pub fn bind_key(&mut self, player: usize, action: usize, key: KeyCode) {
        for (a, slots) in self.keys[player].iter_mut().enumerate() {
            if a != action {
                for s in slots.iter_mut() {
                    if *s == Some(key) {
                        *s = None;
                    }
                }
            }
        }
        let slots = &mut self.keys[player][action];
        if slots[0] != Some(key) {
            slots[1] = slots[0];
            slots[0] = Some(key);
        }
    }

    pub fn bind_pad(&mut self, player: usize, action: usize, b: PadButton) {
        for (a, slots) in self.pad[player].iter_mut().enumerate() {
            if a != action {
                for s in slots.iter_mut() {
                    if *s == Some(b) {
                        *s = None;
                    }
                }
            }
        }
        let slots = &mut self.pad[player][action];
        if slots[0] != Some(b) {
            slots[1] = slots[0];
            slots[0] = Some(b);
        }
    }

    pub fn key_label(&self, player: usize, action: usize) -> String {
        let names: Vec<&str> = self.keys[player][action]
            .iter()
            .flatten()
            .map(|k| key_name(*k))
            .collect();
        if names.is_empty() {
            "-".into()
        } else {
            names.join(" / ")
        }
    }

    pub fn pad_label(&self, player: usize, action: usize) -> String {
        let names: Vec<&str> = self.pad[player][action]
            .iter()
            .flatten()
            .map(|b| b.name())
            .collect();
        if names.is_empty() {
            "-".into()
        } else {
            names.join(" / ")
        }
    }

    /// Text for the save file, e.g. `Left|A,Right|D,...;...`.
    pub fn encode(&self) -> (String, String) {
        let keys = self
            .keys
            .iter()
            .map(|p| {
                p.iter()
                    .map(|s| {
                        s.iter()
                            .map(|k| k.map(key_name).unwrap_or("-"))
                            .collect::<Vec<_>>()
                            .join("|")
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join(";");
        let pad = self
            .pad
            .iter()
            .map(|p| {
                p.iter()
                    .map(|s| {
                        s.iter()
                            .map(|b| b.map(|b| b.name()).unwrap_or("-"))
                            .collect::<Vec<_>>()
                            .join("|")
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .collect::<Vec<_>>()
            .join(";");
        (keys, pad)
    }

    pub fn decode_keys(&mut self, v: &str) {
        for (p, part) in v.split(';').take(2).enumerate() {
            for (a, act) in part.split(',').take(6).enumerate() {
                let mut it = act.split('|');
                for s in 0..2 {
                    self.keys[p][a][s] = it.next().and_then(parse_key);
                }
            }
        }
    }

    pub fn decode_pad(&mut self, v: &str) {
        for (p, part) in v.split(';').take(2).enumerate() {
            for (a, act) in part.split(',').take(6).enumerate() {
                let mut it = act.split('|');
                for s in 0..2 {
                    self.pad[p][a][s] = it.next().and_then(PadButton::parse);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut b = Bindings::default();
        b.bind_key(0, 2, KeyCode::F);
        b.bind_pad(1, 3, PadButton::RB);
        let (k, p) = b.encode();
        let mut c = Bindings::default();
        c.decode_keys(&k);
        c.decode_pad(&p);
        assert_eq!(b, c);
    }

    #[test]
    fn binding_moves_old_key() {
        let mut b = Bindings::default();
        b.bind_key(0, 3, KeyCode::Space);
        assert_eq!(b.keys[0][3], [Some(KeyCode::Space), Some(KeyCode::X)]);
        assert!(!b.keys[0][2].contains(&Some(KeyCode::Space)));
    }
}
