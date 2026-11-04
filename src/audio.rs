//! Sound: chiptune effects and songs synthesized at startup.
//! To use your own music, put .ogg or .wav files in a `music/` folder next to the game:
//! title, stage, boss, shop, final, credits (e.g. `music/stage.ogg`).

use macroquad::audio::{
    PlaySoundParams, Sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound,
};

pub const RATE: u32 = 32000;
const RATE_F: f32 = RATE as f32;

pub fn wav_bytes(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&RATE.to_le_bytes());
    b.extend_from_slice(&(RATE * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32000.0) as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

// ---------------------------------------------------------------------------
// Oscillators
// ---------------------------------------------------------------------------

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn pulse(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty { 1.0 } else { -1.0 }
}

fn tri(phase: f32) -> f32 {
    let p = phase.fract();
    if p < 0.5 {
        p * 4.0 - 1.0
    } else {
        3.0 - p * 4.0
    }
}

fn note_freq(name: &str) -> Option<f32> {
    let b = name.as_bytes();
    if b.is_empty() {
        return None;
    }
    let base = match b[0] {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };
    let mut i = 1;
    let mut semi = base;
    if i < b.len() && b[i] == b'#' {
        semi += 1;
        i += 1;
    } else if i < b.len() && b[i] == b'b' {
        semi -= 1;
        i += 1;
    }
    let oct: i32 = name[i..].parse().ok()?;
    let midi = (oct + 1) * 12 + semi;
    Some(440.0 * 2f32.powf((midi as f32 - 69.0) / 12.0))
}

fn midi_freq(m: f32) -> f32 {
    440.0 * 2f32.powf((m - 69.0) / 12.0)
}

fn freq_midi(f: f32) -> f32 {
    69.0 + 12.0 * (f / 440.0).log2()
}

// ---------------------------------------------------------------------------
// Sound effects
// ---------------------------------------------------------------------------

fn sweep(f0: f32, f1: f32, dur: f32, vol: f32, duty: f32) -> Vec<f32> {
    let n = (dur * RATE_F) as usize;
    let mut ph = 0.0;
    (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            ph += (f0 + (f1 - f0) * k) / RATE_F;
            pulse(ph, duty) * vol * (1.0 - k)
        })
        .collect()
}

fn noise(dur: f32, vol: f32, smooth: f32, seed: u32, curve: f32) -> Vec<f32> {
    let n = (dur * RATE_F) as usize;
    let mut r = Rng(seed);
    let mut last = 0.0;
    (0..n)
        .map(|i| {
            last += (r.next() - last) * smooth;
            let k = i as f32 / n as f32;
            last * vol * (1.0 - k).powf(curve)
        })
        .collect()
}

fn notes(seq: &[f32], each: f32, vol: f32, duty: f32) -> Vec<f32> {
    let mut out = Vec::new();
    for &m in seq {
        let n = (each * RATE_F) as usize;
        let f = midi_freq(m);
        let mut ph = 0.0;
        for i in 0..n {
            ph += f / RATE_F;
            let k = i as f32 / n as f32;
            out.push(pulse(ph, duty) * vol * (1.0 - k * 0.7));
        }
    }
    out
}

fn mix(a: &mut Vec<f32>, b: &[f32], offset: usize) {
    if a.len() < b.len() + offset {
        a.resize(b.len() + offset, 0.0);
    }
    for (i, s) in b.iter().enumerate() {
        a[i + offset] += s;
    }
}

fn kick(vol: f32) -> Vec<f32> {
    let n = (0.14 * RATE_F) as usize;
    let mut ph = 0.0;
    (0..n)
        .map(|i| {
            let k = i as f32 / n as f32;
            ph += (150.0 * (1.0 - k) + 42.0 * k) / RATE_F;
            (ph * std::f32::consts::TAU).sin() * vol * (1.0 - k).powf(1.5)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Blaster,
    Laser,
    Missile,
    EnemyShot,
    Pop,
    Boom,
    BigBoom,
    Clang,
    PlayerDie,
    Powerup,
    Crystal,
    CardGet,
    Bomb,
    MenuMove,
    MenuOk,
    Buy,
    Deny,
    Warning,
    OneUp,
    Shield,
    Fanfare,
    GameOver,
    Combo,
    Moo,
    Thunder,
    Flame,
    Disc,
    Pellet,
    Graze,
    Achievement,
    RankUp,
    Dash,
    Ability,
    Web,
    Sparkle,
    Beep,
    Portal,
    WaveShot,
    Ice,
    Buzz,
    Fever,
    Contract,
    Log,
    Steal,
    Devour,
    Curse,
    Relic,
}

const SFX_COUNT: usize = 47;

fn sfx_samples(s: Sfx) -> Vec<f32> {
    match s {
        Sfx::Blaster => sweep(1400.0, 420.0, 0.06, 0.22, 0.5),
        Sfx::Laser => {
            let mut a = sweep(2200.0, 1500.0, 0.09, 0.14, 0.25);
            mix(&mut a, &sweep(1100.0, 750.0, 0.09, 0.08, 0.5), 0);
            a
        }
        Sfx::Missile => {
            let mut a = noise(0.18, 0.35, 0.5, 9, 1.0);
            mix(&mut a, &sweep(300.0, 700.0, 0.12, 0.12, 0.5), 0);
            a
        }
        Sfx::EnemyShot => sweep(700.0, 300.0, 0.08, 0.12, 0.25),
        Sfx::Pop => {
            let mut a = noise(0.22, 0.7, 0.45, 3, 2.0);
            mix(&mut a, &sweep(500.0, 80.0, 0.12, 0.25, 0.5), 0);
            a
        }
        Sfx::Boom => {
            let mut a = noise(0.45, 0.9, 0.25, 7, 2.0);
            mix(&mut a, &kick(0.6), 0);
            a
        }
        Sfx::BigBoom => {
            let mut a = noise(1.4, 1.0, 0.12, 21, 1.6);
            mix(&mut a, &kick(0.9), 0);
            mix(&mut a, &kick(0.7), (0.18 * RATE_F) as usize);
            mix(
                &mut a,
                &noise(0.6, 0.6, 0.4, 5, 2.0),
                (0.3 * RATE_F) as usize,
            );
            a
        }
        Sfx::Clang => {
            let mut a = sweep(1800.0, 1700.0, 0.07, 0.18, 0.5);
            mix(&mut a, &sweep(2700.0, 2600.0, 0.05, 0.1, 0.5), 0);
            a
        }
        Sfx::PlayerDie => {
            let mut a = noise(1.1, 1.0, 0.15, 42, 1.4);
            mix(&mut a, &sweep(900.0, 60.0, 0.8, 0.3, 0.5), 0);
            a
        }
        Sfx::Powerup => notes(&[72.0, 76.0, 79.0, 84.0, 88.0], 0.05, 0.3, 0.5),
        Sfx::Crystal => notes(&[93.0, 100.0], 0.04, 0.16, 0.25),
        Sfx::CardGet => {
            let mut a = notes(
                &[84.0, 88.0, 91.0, 96.0, 91.0, 96.0, 100.0],
                0.06,
                0.25,
                0.25,
            );
            mix(&mut a, &notes(&[72.0, 79.0, 84.0], 0.14, 0.15, 0.5), 0);
            a
        }
        Sfx::Bomb => {
            let mut a = noise(1.6, 1.0, 0.08, 77, 1.2);
            mix(&mut a, &sweep(80.0, 30.0, 1.0, 0.5, 0.5), 0);
            mix(&mut a, &kick(1.0), 0);
            a
        }
        Sfx::MenuMove => sweep(900.0, 900.0, 0.03, 0.15, 0.5),
        Sfx::MenuOk => notes(&[79.0, 84.0], 0.05, 0.22, 0.5),
        Sfx::Buy => notes(&[84.0, 88.0, 91.0, 96.0], 0.045, 0.22, 0.25),
        Sfx::Deny => notes(&[52.0, 49.0], 0.08, 0.25, 0.5),
        Sfx::Warning => {
            let mut a = Vec::new();
            for _ in 0..3 {
                a.extend(sweep(440.0, 880.0, 0.22, 0.25, 0.5));
                a.extend(sweep(880.0, 440.0, 0.22, 0.25, 0.5));
            }
            a
        }
        Sfx::OneUp => notes(&[76.0, 79.0, 88.0, 84.0, 86.0, 91.0], 0.07, 0.25, 0.5),
        Sfx::Shield => {
            let mut a = sweep(300.0, 1200.0, 0.25, 0.2, 0.125);
            mix(&mut a, &noise(0.25, 0.3, 0.6, 4, 1.0), 0);
            a
        }
        Sfx::Fanfare => {
            let mut a = notes(&[67.0, 72.0, 76.0, 79.0], 0.1, 0.25, 0.5);
            a.extend(notes(&[84.0], 0.5, 0.25, 0.5));
            let mut b = notes(&[55.0, 60.0, 64.0, 67.0], 0.1, 0.15, 0.25);
            b.extend(notes(&[72.0], 0.5, 0.15, 0.25));
            mix(&mut a, &b, 0);
            a
        }
        Sfx::GameOver => notes(
            &[67.0, 66.0, 65.0, 64.0, 63.0, 62.0, 61.0, 60.0, 55.0, 48.0],
            0.13,
            0.25,
            0.5,
        ),
        Sfx::Combo => notes(&[84.0, 91.0], 0.04, 0.15, 0.125),
        Sfx::Moo => {
            let n = (0.9 * RATE_F) as usize;
            let mut ph = 0.0;
            (0..n)
                .map(|i| {
                    let k = i as f32 / n as f32;
                    let f = 190.0 - 70.0 * k + (k * 40.0).sin() * 4.0;
                    ph += f / RATE_F;
                    let env = (k * 12.0).min(1.0) * (1.0 - k).powf(0.6);
                    (tri(ph) * 0.6 + pulse(ph, 0.3) * 0.25 + pulse(ph * 2.0, 0.5) * 0.1)
                        * env
                        * 0.45
                })
                .collect()
        }
        Sfx::Thunder => {
            let mut a = noise(0.8, 1.0, 0.5, 31, 2.2);
            mix(&mut a, &kick(0.8), 0);
            a
        }
        Sfx::Flame => noise(0.09, 0.22, 0.3, 13, 1.0),
        Sfx::Disc => sweep(600.0, 950.0, 0.08, 0.14, 0.125),
        Sfx::Pellet => {
            let mut a = noise(0.16, 0.6, 0.5, 17, 2.0);
            mix(&mut a, &kick(0.35), 0);
            a
        }
        Sfx::Graze => notes(&[100.0], 0.025, 0.07, 0.125),
        Sfx::Achievement => {
            let mut a = notes(&[79.0, 83.0, 86.0, 91.0], 0.07, 0.22, 0.25);
            a.extend(notes(&[95.0], 0.3, 0.22, 0.25));
            a
        }
        Sfx::RankUp => {
            let mut a = notes(&[72.0, 76.0, 79.0, 84.0, 79.0, 84.0, 88.0], 0.09, 0.24, 0.5);
            a.extend(notes(&[91.0], 0.6, 0.24, 0.5));
            let mut b = notes(
                &[60.0, 64.0, 67.0, 72.0, 67.0, 72.0, 76.0],
                0.09,
                0.14,
                0.25,
            );
            b.extend(notes(&[79.0], 0.6, 0.14, 0.25));
            mix(&mut a, &b, 0);
            a
        }
        Sfx::Dash => {
            let mut a = noise(0.15, 0.3, 0.8, 8, 1.5);
            mix(&mut a, &sweep(300.0, 1200.0, 0.12, 0.12, 0.25), 0);
            a
        }
        Sfx::Ability => {
            let mut a = sweep(200.0, 1600.0, 0.35, 0.22, 0.25);
            mix(&mut a, &sweep(100.0, 800.0, 0.35, 0.15, 0.5), 0);
            a
        }
        Sfx::Web => {
            let mut a = noise(0.12, 0.3, 0.2, 6, 1.5);
            mix(&mut a, &sweep(400.0, 200.0, 0.1, 0.12, 0.5), 0);
            a
        }
        Sfx::Sparkle => sparkle(),
        Sfx::Beep => notes(&[84.0], 0.08, 0.2, 0.5),
        Sfx::Portal => {
            let mut a = sweep(200.0, 900.0, 0.4, 0.2, 0.25);
            a.extend(sweep(900.0, 300.0, 0.4, 0.2, 0.25));
            a.extend(sweep(300.0, 1400.0, 0.5, 0.2, 0.125));
            a
        }
        Sfx::WaveShot => sweep(900.0, 1300.0, 0.06, 0.12, 0.25),
        Sfx::Ice => {
            let mut a = sweep(2400.0, 3200.0, 0.05, 0.1, 0.125);
            mix(&mut a, &noise(0.08, 0.15, 0.9, 11, 2.0), 0);
            a
        }
        Sfx::Buzz => {
            let n = (0.12 * RATE_F) as usize;
            let mut ph = 0.0;
            (0..n)
                .map(|i| {
                    let k = i as f32 / n as f32;
                    ph += (220.0 + (k * 90.0).sin() * 30.0) / RATE_F;
                    pulse(ph, 0.2) * 0.08 * (1.0 - k)
                })
                .collect()
        }
        Sfx::Fever => {
            let mut a = notes(
                &[72.0, 76.0, 79.0, 84.0, 88.0, 91.0, 96.0],
                0.05,
                0.22,
                0.25,
            );
            mix(&mut a, &sweep(200.0, 2000.0, 0.35, 0.1, 0.5), 0);
            a
        }
        Sfx::Contract => notes(&[84.0, 79.0, 84.0, 91.0], 0.08, 0.22, 0.5),
        Sfx::Log => {
            let mut a = notes(&[64.0, 71.0, 76.0], 0.12, 0.15, 0.125);
            mix(&mut a, &noise(0.36, 0.05, 0.9, 3, 1.0), 0);
            a
        }
        Sfx::Steal => notes(&[88.0, 84.0, 79.0, 76.0], 0.05, 0.2, 0.25),
        Sfx::Devour => {
            let mut a = sweep(120.0, 40.0, 1.6, 0.5, 0.5);
            mix(&mut a, &noise(1.6, 0.6, 0.05, 99, 1.0), 0);
            a
        }
        Sfx::Curse => {
            let mut a = notes(&[58.0, 57.0, 52.0, 51.0], 0.1, 0.3, 0.5);
            mix(&mut a, &sweep(300.0, 60.0, 0.5, 0.25, 0.3), 0);
            mix(&mut a, &noise(0.4, 0.15, 0.4, 13, 1.5), 0);
            a
        }
        Sfx::Relic => {
            let mut a = notes(&[60.0, 67.0, 72.0, 76.0, 79.0, 84.0], 0.07, 0.22, 0.25);
            mix(
                &mut a,
                &notes(&[84.0, 88.0, 91.0, 96.0], 0.12, 0.18, 0.125),
                3000,
            );
            mix(&mut a, &sweep(400.0, 2400.0, 0.6, 0.08, 0.5), 0);
            a
        }
    }
}

fn sparkle() -> Vec<f32> {
    // Stardust sparkle: a bright arpeggio that rises and spreads out.
    let seq: [f32; 10] = [72.0, 76.0, 79.0, 84.0, 88.0, 91.0, 96.0, 91.0, 96.0, 100.0];
    let mut out = Vec::new();
    for (i, m) in seq.iter().enumerate() {
        let f = midi_freq(*m);
        let dur = 0.5;
        let n = (dur * RATE_F) as usize;
        let mut v = vec![0.0f32; n];
        for (k, x) in v.iter_mut().enumerate() {
            let t = k as f32 / RATE_F;
            let env = (-t * 7.0).exp() * 0.32;
            *x = ((t * f * std::f32::consts::TAU).sin()
                + 0.4 * (t * f * 2.0 * std::f32::consts::TAU).sin())
                * env;
        }
        mix(&mut out, &v, (i as f32 * 0.07 * RATE_F) as usize);
    }
    for s in &mut out {
        *s = s.tanh();
    }
    out
}

// ---------------------------------------------------------------------------
// Songs (mini sequencer)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Music {
    Title,
    Stage,
    Boss,
    Shop,
    Final,
    Credits,
}

struct Song {
    bpm: f32,
    chords: &'static [&'static str],
    lead: &'static [&'static str],
    drums: &'static str,
    bass_octaves: bool,
    lead_duty: f32,
    lead_vol: f32,
    arp_vol: f32,
}

const TITLE: Song = Song {
    bpm: 96.0,
    chords: &["Am", "F", "C", "G", "Am", "F", "G", "E"],
    lead: &[
        "A4 - - - C5 - E5 - - - D5 - C5 - B4 -",
        "C5 - - - - - - - A4 - - - . . . .",
        "G4 - - - C5 - E5 - - - G5 - F5 - E5 -",
        "D5 - - - - - - - B4 - - - . . . .",
        "A4 - - - C5 - E5 - - - A5 - G5 - E5 -",
        "F5 - - - E5 - C5 - - - A4 - C5 - - -",
        "B4 - - - D5 - G5 - - - F5 - D5 - B4 -",
        "G#4 - - - - - - - B4 - - - E5 - - -",
    ],
    drums: "K...h...S...h..h",
    bass_octaves: false,
    lead_duty: 0.25,
    lead_vol: 0.16,
    arp_vol: 0.05,
};

const STAGE: Song = Song {
    bpm: 150.0,
    chords: &["Am", "G", "F", "G", "Am", "G", "F", "E"],
    lead: &[
        "A4 . A4 . C5 . E5 . A5 - G5 - E5 - C5 -",
        "D5 - - - B4 - G4 - D5 - - - E5 - D5 -",
        "C5 . C5 . F5 . A5 . C6 - A5 - F5 - C5 -",
        "D5 - - - - - B4 - G4 - B4 - D5 - G5 -",
        "A5 - - - G5 - E5 - G5 - - - A5 - C6 -",
        "B5 - - - G5 - D5 - B4 - D5 - G5 - - -",
        "A5 - G5 - F5 - E5 - F5 - G5 - A5 - C6 -",
        "B5 - - - G#5 - - - E5 - - - B4 - G#4 -",
    ],
    drums: "K.h.S.h.K.KhS.hH",
    bass_octaves: true,
    lead_duty: 0.5,
    lead_vol: 0.12,
    arp_vol: 0.045,
};

const BOSS: Song = Song {
    bpm: 168.0,
    chords: &["Dm", "Dm", "Bb", "A", "Dm", "Dm", "Bb", "A"],
    lead: &[
        "D5 . D5 . D5 . F5 . E5 . D5 . C#5 . D5 .",
        "A5 - - - G5 - F5 - E5 - F5 - D5 - - -",
        "D5 . D5 . F5 . Bb5 . A5 - G5 - F5 - D5 -",
        "C#5 - - - E5 - A5 - G5 - E5 - C#5 - A4 -",
        "D6 - - - A5 - - - F5 - - - D5 - - -",
        "E5 - F5 - G5 - A5 - Bb5 - A5 - G5 - F5 -",
        "F5 - - - D5 - - - Bb4 - D5 - F5 - Bb5 -",
        "A5 - - - - - - - C#6 - - - E6 - - -",
    ],
    drums: "K.hKS.hKK.hKS.SS",
    bass_octaves: true,
    lead_duty: 0.25,
    lead_vol: 0.12,
    arp_vol: 0.04,
};

const SHOP: Song = Song {
    bpm: 104.0,
    chords: &["C", "Am", "F", "G", "C", "Am", "Dm", "G"],
    lead: &[
        "E5 - G5 - C6 - - - B5 - G5 - E5 - - -",
        "A5 - - - G5 - E5 - C5 - - - E5 - - -",
        "F5 - A5 - C6 - - - A5 - F5 - C5 - - -",
        "D5 - - - G5 - - - F5 - E5 - D5 - - -",
        "E5 - G5 - C6 - - - D6 - C6 - G5 - - -",
        "A5 - - - C6 - - - E6 - - - C6 - - -",
        "D6 - C6 - A5 - F5 - A5 - C6 - D6 - - -",
        "B5 - - - - - - - G5 - - - . . . .",
    ],
    drums: "K...h.h.S...h...",
    bass_octaves: false,
    lead_duty: 0.125,
    lead_vol: 0.13,
    arp_vol: 0.05,
};

const FINAL: Song = Song {
    bpm: 176.0,
    chords: &["Em", "C", "D", "B", "Em", "C", "Am", "B"],
    lead: &[
        "E5 . E5 . G5 . B5 . E6 - D6 - B5 - G5 -",
        "C6 - - - B5 - G5 - E5 - G5 - C6 - - -",
        "D6 . D6 . A5 . F#5 . D5 - F#5 - A5 - D6 -",
        "D#6 - - - B5 - - - F#5 - - - D#5 - - -",
        "E6 - - - B5 - G5 - E5 - G5 - B5 - E6 -",
        "G6 - E6 - C6 - G5 - E5 - C5 - E5 - G5 -",
        "A5 - C6 - E6 - - - D6 - C6 - B5 - A5 -",
        "B5 - - - D#6 - - - F#6 - - - B6 - - -",
    ],
    drums: "KKh.S.hKK.hKSSSS",
    bass_octaves: true,
    lead_duty: 0.25,
    lead_vol: 0.11,
    arp_vol: 0.04,
};

const CREDITS: Song = Song {
    bpm: 84.0,
    chords: &["F", "C", "Dm", "Bb", "F", "C", "Bb", "C"],
    lead: &[
        "A5 - - - C6 - - - A5 - G5 - F5 - - -",
        "E5 - - - G5 - - - C6 - - - - - - -",
        "D6 - - - C6 - A5 - F5 - - - A5 - - -",
        "Bb5 - - - A5 - G5 - F5 - - - - - - -",
        "A5 - - - C6 - - - F6 - - - E6 - D6 -",
        "C6 - - - - - - - G5 - - - E5 - - -",
        "D6 - - - C6 - Bb5 - A5 - - - G5 - - -",
        "F5 - - - - - - - . . . . . . . .",
    ],
    drums: "K.......S.......",
    bass_octaves: false,
    lead_duty: 0.125,
    lead_vol: 0.14,
    arp_vol: 0.05,
};

fn chord_tones(name: &str) -> [f32; 3] {
    let minor = name.ends_with('m');
    let root_name = name.trim_end_matches('m');
    let root = freq_midi(note_freq(&format!("{root_name}3")).unwrap_or(220.0)).round();
    [root, root + if minor { 3.0 } else { 4.0 }, root + 7.0]
}

/// One voice playing one note: writes into the buffer.
fn voice(
    buf: &mut [f32],
    start: usize,
    len: usize,
    f: f32,
    vol: f32,
    kind: u8,
    duty: f32,
    decay: f32,
) {
    let release = (0.03 * RATE_F) as usize;
    let mut ph = 0.0f32;
    let total = len + release;
    for i in 0..total {
        let idx = start + i;
        if idx >= buf.len() {
            break;
        }
        let t = i as f32 / RATE_F;
        let mut env = (t / 0.004).min(1.0) * (0.55 + 0.45 * (-t * decay).exp());
        if i >= len {
            env *= 1.0 - (i - len) as f32 / release as f32;
        }
        // light vibrato on long notes
        let vib = if t > 0.18 {
            1.0 + (t * 34.0).sin() * 0.004
        } else {
            1.0
        };
        ph += f * vib / RATE_F;
        let s = match kind {
            0 => pulse(ph, duty),
            _ => tri(ph),
        };
        buf[idx] += s * vol * env;
    }
}

fn render_song(song: &Song) -> Vec<f32> {
    let step = 60.0 / song.bpm / 4.0;
    let step_n = (step * RATE_F) as usize;
    let bars = song.chords.len();
    let total = 2 * bars * 16 * step_n;
    let mut buf = vec![0.0f32; total + RATE as usize];
    let mut lead_buf = vec![0.0f32; buf.len()];
    let mut rng = Rng(12345);

    // Two passes: the second with a harmonized melody and fuller drums.
    for pass in 0..2usize {
        for (bar, chord) in song.chords.iter().enumerate() {
            let tones = chord_tones(chord);
            let bar_start = (pass * bars + bar) * 16 * step_n;

            // Bass
            for s in 0..16 {
                let (play, oct) = if song.bass_octaves {
                    (s % 2 == 0, if s % 4 == 2 { 12.0 } else { 0.0 })
                } else {
                    (s % 4 == 0, 0.0)
                };
                if play {
                    let f = midi_freq(tones[0] - 12.0 + oct);
                    let len = if song.bass_octaves {
                        step_n * 2 - step_n / 3
                    } else {
                        step_n * 4 - step_n / 2
                    };
                    voice(&mut buf, bar_start + s * step_n, len, f, 0.22, 1, 0.5, 6.0);
                }
            }
            // Arpeggio
            for s in 0..16 {
                let m = tones[s % 3] + 12.0 + if (s / 3) % 2 == 1 { 12.0 } else { 0.0 };
                voice(
                    &mut buf,
                    bar_start + s * step_n,
                    step_n * 2 / 3,
                    midi_freq(m),
                    song.arp_vol,
                    0,
                    0.125,
                    18.0,
                );
            }
            // Drums
            for (s, c) in song.drums.chars().enumerate() {
                let at = bar_start + s * step_n;
                let c = if pass == 1 && c == '.' && s % 2 == 0 {
                    'h'
                } else {
                    c
                };
                let c = if pass == 1 && bar == bars - 1 && s >= 12 {
                    'S'
                } else {
                    c
                };
                match c {
                    'K' => mix_into(&mut buf, &kick(0.5), at),
                    'S' => {
                        let mut sn = noise(0.13, 0.32, 0.7, rng.0.wrapping_add(s as u32), 1.6);
                        let tone = sweep(260.0, 180.0, 0.07, 0.12, 0.5);
                        for (i, t) in tone.iter().enumerate() {
                            if i < sn.len() {
                                sn[i] += t;
                            }
                        }
                        mix_into(&mut buf, &sn, at)
                    }
                    'h' => mix_into(&mut buf, &hat(0.03, 0.09, &mut rng), at),
                    'H' => mix_into(&mut buf, &hat(0.12, 0.08, &mut rng), at),
                    _ => {}
                }
            }
            // Melody
            let toks: Vec<&str> = song.lead[bar % song.lead.len()]
                .split_whitespace()
                .collect();
            let mut s = 0;
            while s < toks.len() {
                if let Some(f) = note_freq(toks[s]) {
                    let mut len = 1;
                    while s + len < toks.len() && toks[s + len] == "-" {
                        len += 1;
                    }
                    voice(
                        &mut lead_buf,
                        bar_start + s * step_n,
                        len * step_n - step_n / 5,
                        f,
                        song.lead_vol,
                        0,
                        song.lead_duty,
                        3.0,
                    );
                    if pass == 1 {
                        // third below (approximated by a minor sixth) to thicken the melody
                        voice(
                            &mut lead_buf,
                            bar_start + s * step_n,
                            len * step_n - step_n / 5,
                            f * 0.6674,
                            song.lead_vol * 0.55,
                            0,
                            0.25,
                            3.0,
                        );
                    }
                    s += len;
                } else {
                    s += 1;
                }
            }
        }
    }

    // Melody echo and final mix
    let delay = step_n * 3;
    for i in delay..lead_buf.len() {
        lead_buf[i] += lead_buf[i - delay] * 0.28;
    }
    for i in 0..buf.len() {
        buf[i] += lead_buf[i];
    }
    // The tail after the end wraps to the start (seamless loop)
    let (main, tail) = buf.split_at_mut(total);
    for (i, s) in tail.iter().enumerate() {
        if i < main.len() {
            main[i] += s;
        }
    }
    buf.truncate(total);
    for s in &mut buf {
        *s = (*s * 1.1).tanh() * 0.9;
    }
    buf
}

fn hat(dur: f32, vol: f32, rng: &mut Rng) -> Vec<f32> {
    let n = (dur * RATE_F) as usize;
    let mut last = 0.0;
    (0..n)
        .map(|i| {
            let x = rng.next();
            let hp = x - last; // simple high-pass
            last = x;
            hp * vol * (1.0 - i as f32 / n as f32).powf(2.0)
        })
        .collect()
}

fn mix_into(buf: &mut [f32], s: &[f32], at: usize) {
    for (i, v) in s.iter().enumerate() {
        if at + i < buf.len() {
            buf[at + i] += v;
        }
    }
}

// ---------------------------------------------------------------------------
// Manager
// ---------------------------------------------------------------------------

pub struct Audio {
    sfx: Vec<Option<Sound>>,
    music: Vec<(Music, Option<Sound>)>,
    current: Option<Music>,
    pub music_vol: f32,
    pub sfx_vol: f32,
    played_this_frame: [u8; SFX_COUNT],
}

const ALL_SFX: [Sfx; SFX_COUNT] = [
    Sfx::Blaster,
    Sfx::Laser,
    Sfx::Missile,
    Sfx::EnemyShot,
    Sfx::Pop,
    Sfx::Boom,
    Sfx::BigBoom,
    Sfx::Clang,
    Sfx::PlayerDie,
    Sfx::Powerup,
    Sfx::Crystal,
    Sfx::CardGet,
    Sfx::Bomb,
    Sfx::MenuMove,
    Sfx::MenuOk,
    Sfx::Buy,
    Sfx::Deny,
    Sfx::Warning,
    Sfx::OneUp,
    Sfx::Shield,
    Sfx::Fanfare,
    Sfx::GameOver,
    Sfx::Combo,
    Sfx::Moo,
    Sfx::Thunder,
    Sfx::Flame,
    Sfx::Disc,
    Sfx::Pellet,
    Sfx::Graze,
    Sfx::Achievement,
    Sfx::RankUp,
    Sfx::Dash,
    Sfx::Ability,
    Sfx::Web,
    Sfx::Sparkle,
    Sfx::Beep,
    Sfx::Portal,
    Sfx::WaveShot,
    Sfx::Ice,
    Sfx::Buzz,
    Sfx::Fever,
    Sfx::Contract,
    Sfx::Log,
    Sfx::Steal,
    Sfx::Devour,
    Sfx::Curse,
    Sfx::Relic,
];

impl Audio {
    pub async fn load(mut progress: impl FnMut(f32)) -> Audio {
        let mut sfx = Vec::new();
        for (i, s) in ALL_SFX.iter().enumerate() {
            sfx.push(
                load_sound_from_bytes(&wav_bytes(&sfx_samples(*s)))
                    .await
                    .ok(),
            );
            progress(i as f32 / SFX_COUNT as f32 * 0.3);
        }
        let mut music = Vec::new();
        // Custom music file names.
        let list = [
            (Music::Title, &TITLE, "title"),
            (Music::Stage, &STAGE, "stage"),
            (Music::Boss, &BOSS, "boss"),
            (Music::Shop, &SHOP, "shop"),
            (Music::Final, &FINAL, "final"),
            (Music::Credits, &CREDITS, "credits"),
        ];
        for (i, (id, song, file)) in list.iter().enumerate() {
            let mut snd = None;
            'search: for dir in music_dirs() {
                for ext in ["ogg", "wav"] {
                    let path = dir.join(format!("{file}.{ext}"));
                    if let Ok(bytes) = std::fs::read(&path) {
                        snd = load_sound_from_bytes(&bytes).await.ok();
                        if snd.is_some() {
                            break 'search;
                        }
                    }
                }
            }
            if snd.is_none() {
                snd = load_sound_from_bytes(&wav_bytes(&render_song(song)))
                    .await
                    .ok();
            }
            music.push((*id, snd));
            progress(0.3 + (i + 1) as f32 / list.len() as f32 * 0.7);
        }
        Audio {
            sfx,
            music,
            current: None,
            music_vol: 0.6,
            sfx_vol: 0.8,
            played_this_frame: [0; SFX_COUNT],
        }
    }

    pub fn end_frame(&mut self) {
        self.played_this_frame = [0; SFX_COUNT];
    }

    pub fn play(&mut self, s: Sfx) {
        self.play_vol(s, 1.0);
    }

    pub fn play_vol(&mut self, s: Sfx, vol: f32) {
        let i = ALL_SFX.iter().position(|x| *x == s).unwrap_or(0);
        // avoids stacking the same sound many times in the same frame
        if self.played_this_frame[i] >= 2 || self.sfx_vol <= 0.0 {
            return;
        }
        self.played_this_frame[i] += 1;
        if let Some(snd) = &self.sfx[i] {
            play_sound(
                snd,
                PlaySoundParams {
                    looped: false,
                    volume: (vol * self.sfx_vol).min(1.0),
                },
            );
        }
    }

    pub fn music(&mut self, m: Option<Music>) {
        if self.current == m {
            return;
        }
        if let Some(cur) = self.current
            && let Some((_, Some(s))) = self.music.iter().find(|(id, _)| *id == cur)
        {
            stop_sound(s);
        }
        self.current = m;
        if let Some(m) = m
            && let Some((_, Some(s))) = self.music.iter().find(|(id, _)| *id == m)
        {
            play_sound(
                s,
                PlaySoundParams {
                    looped: true,
                    volume: self.music_vol * 0.8,
                },
            );
        }
    }

    pub fn set_music_volume(&mut self, v: f32) {
        self.music_vol = v;
        for (_, s) in &self.music {
            if let Some(s) = s {
                set_sound_volume(s, v * 0.8);
            }
        }
    }
}

/// Used by the export tool (`--export-music`).
pub fn export_music(dir: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    for (song, name) in [
        (&TITLE, "title"),
        (&STAGE, "stage"),
        (&BOSS, "boss"),
        (&SHOP, "shop"),
        (&FINAL, "final"),
        (&CREDITS, "credits"),
    ] {
        std::fs::write(format!("{dir}/{name}.wav"), wav_bytes(&render_song(song)))?;
    }
    Ok(())
}

/// Folders where the game looks for custom music, in priority order:
/// user data folder, executable folder and current folder.
fn music_dirs() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![crate::save::data_dir().join("music")];
    if let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        dirs.push(exe_dir.join("music"));
    }
    dirs.push(std::path::PathBuf::from("music"));
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_notes() {
        assert!((note_freq("A4").unwrap() - 440.0).abs() < 0.01);
        assert!((note_freq("C#5").unwrap() - 554.37).abs() < 0.1);
        assert!((note_freq("Bb3").unwrap() - 233.08).abs() < 0.1);
        assert!(note_freq("-").is_none());
    }

    #[test]
    fn every_bar_has_16_steps_and_valid_notes() {
        for (name, song) in [
            ("title", &TITLE),
            ("stage", &STAGE),
            ("boss", &BOSS),
            ("shop", &SHOP),
            ("final", &FINAL),
            ("credits", &CREDITS),
        ] {
            assert_eq!(
                song.lead.len(),
                song.chords.len(),
                "{name}: melody and chords have different lengths"
            );
            assert_eq!(song.drums.chars().count(), 16, "{name}: bateria");
            for (i, bar) in song.lead.iter().enumerate() {
                let toks: Vec<&str> = bar.split_whitespace().collect();
                assert_eq!(
                    toks.len(),
                    16,
                    "{name}: compasso {} tem {} passos",
                    i + 1,
                    toks.len()
                );
                for t in toks {
                    assert!(
                        t == "-" || t == "." || note_freq(t).is_some(),
                        "{name}: invalid note {t}"
                    );
                }
            }
        }
    }

    #[test]
    fn music_does_not_clip() {
        let s = render_song(&STAGE);
        assert!(s.iter().all(|x| x.abs() <= 1.0));
        assert!(s.iter().map(|x| x.abs()).fold(0.0, f32::max) > 0.3);
    }
}
