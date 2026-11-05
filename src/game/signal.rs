//! THE SIGNAL: the hidden last fight. It waits behind the final boss of the campaign.
//!
//! The STARDUST probe brought back dust and a signal; the signal called the Swarm. When the
//! Supreme Queen falls, the signal itself wakes up. It has three phases, each with its own
//! rules:
//!
//! 1. ECHO: a mirrored copy of your ship flies at the top. It always stands where you are
//!    mirrored, and every shot you fire comes back from it a moment later. To hit it you
//!    must stand near the middle, where its echoes also fall, so shoot and step aside.
//! 2. CONSTELLATION: stars light up in an order. Shoot them in that same order to crack the
//!    shield and open the core for a few seconds. A wrong star resets the sequence and
//!    bursts. Three cracks open the last phase.
//! 3. COLLAPSE: space closes in from both sides. Every hit on the core pushes the walls
//!    back; curtains of dust fall with a single gap to slip through.

use super::*;
use crate::light;
use std::f32::consts::TAU;

const CORE_Y: f32 = 82.0;
const ECHO_Y: f32 = 150.0;

#[derive(Default)]
pub struct Signal {
    /// 0 intro, 1 echo, 2 constellation, 3 collapse, 4 dying.
    pub phase: u8,
    t: f32,
    phase_t: f32,
    hp_mul: f32,
    flash: f32,
    // echo
    echo_x: f32,
    echo_hp: f32,
    echo_max: f32,
    echo_queue: Vec<(f32, f32)>,
    last_shots: u32,
    ring_cd: f32,
    // constellation
    nodes: Vec<Vec2>,
    order: Vec<usize>,
    progress: usize,
    show_t: f32,
    show_k: usize,
    show_next: f32,
    node_flash: [f32; 6],
    exposed: f32,
    cracks: u32,
    crack_hp: f32,
    spiral: f32,
    spiral_cd: f32,
    // collapse
    inset: f32,
    core_hp: f32,
    core_max: f32,
    curtain_cd: f32,
    burst_cd: f32,
    // end
    die_t: f32,
    pub done: bool,
}

impl Game {
    /// True right after the campaign's final boss falls, the first time.
    pub(super) fn signal_ready(&self) -> bool {
        self.cfg.mode == Mode::Campaign
            && self.sector() == SECTORS
            && !self.enc.void
            && self.sig.is_none()
            && !self.sig_done
    }

    pub(super) fn start_signal(&mut self, au: &mut Audio) {
        let hp_mul = 1.0 + self.diff() * 0.06;
        let nodes = (0..6)
            .map(|i| {
                let x = W * (0.13 + i as f32 * 0.148);
                let y = 125.0 + ((i as f32) * 1.7).sin() * 55.0;
                vec2(x, y)
            })
            .collect();
        self.sig = Some(Signal {
            phase: 0,
            hp_mul,
            echo_x: W / 2.0,
            echo_max: 260.0 * hp_mul,
            echo_hp: 260.0 * hp_mul,
            last_shots: self.player.shots,
            ring_cd: 3.0,
            nodes,
            core_max: 1000.0 * hp_mul,
            core_hp: 1000.0 * hp_mul,
            curtain_cd: 2.0,
            burst_cd: 3.0,
            ..Default::default()
        });
        self.ebullets.clear();
        self.wave_kind = WaveKind::Boss;
        self.fx.flash = 1.0;
        self.fx.flash_color = WHITE;
        self.fx.shake = 1.0;
        self.events.push(Event::Rumble(None, 1.0, 900));
        au.music(None);
        au.play(Sfx::Devour);
        self.banner = Some((
            "SIGNAL DETECTED".into(),
            "It called the Swarm. Now it calls you.".into(),
            4.0,
        ));
    }

    /// Development helper: jump to a phase of the fight.
    pub(super) fn debug_signal_phase(&mut self, ph: u8) {
        if let Some(s) = &mut self.sig {
            s.phase = ph.clamp(1, 3);
            s.phase_t = 0.0;
            if s.phase == 2 {
                new_sequence(s);
            }
        }
    }

    /// True when the fight is over (the signal has gone silent).
    pub(super) fn signal_finished(&self) -> bool {
        self.sig.as_ref().map(|s| s.done).unwrap_or(false)
    }

    pub(super) fn update_signal(&mut self, dt: f32, au: &mut Audio) {
        let Some(mut s) = self.sig.take() else { return };
        s.t += dt;
        s.phase_t += dt;
        s.flash = (s.flash - dt).max(0.0);
        for f in &mut s.node_flash {
            *f = (*f - dt).max(0.0);
        }
        let core = vec2(W / 2.0, CORE_Y);
        let pp = self.player.pos;
        let alive = self.player.alive();
        let bmul = self.bullet_mul();
        match s.phase {
            0 => {
                if s.phase_t > 4.0 {
                    s.phase = 1;
                    s.phase_t = 0.0;
                    au.music(Some(Music::Final));
                    self.banner = Some(("ECHO".into(), "It fires what you fire.".into(), 2.5));
                }
            }
            1 => {
                // The echo mirrors your position and repeats your shots.
                let target = W - pp.x;
                s.echo_x += (target - s.echo_x) * (dt * 6.0).min(1.0);
                if self.player.shots != s.last_shots {
                    s.last_shots = self.player.shots;
                    if alive {
                        s.echo_queue.push((0.4, s.echo_x));
                    }
                }
                let mut fire = vec![];
                for q in &mut s.echo_queue {
                    q.0 -= dt;
                    if q.0 <= 0.0 {
                        fire.push(q.1);
                    }
                }
                s.echo_queue.retain(|q| q.0 > 0.0);
                for x in fire {
                    for a in [-0.12f32, 0.0, 0.12] {
                        self.ebullets.push(EBullet::new(
                            vec2(x, ECHO_Y + 14.0),
                            vec2(a.sin(), a.cos()) * 300.0 * bmul,
                            4.0,
                            4,
                        ));
                    }
                }
                s.ring_cd -= dt;
                if s.ring_cd <= 0.0 {
                    s.ring_cd = 3.4;
                    for k in 0..14 {
                        let a = k as f32 / 14.0 * TAU + s.t;
                        self.ebullets.push(EBullet::new(
                            core,
                            vec2(a.cos(), a.sin().abs() * 0.8 + 0.2) * 110.0 * bmul,
                            5.0,
                            4,
                        ));
                    }
                }
                if s.echo_hp <= 0.0 {
                    s.phase = 2;
                    s.phase_t = 0.0;
                    self.fx.explosion(
                        vec2(s.echo_x, ECHO_Y),
                        Color::from_rgba(200, 140, 255, 255),
                        true,
                    );
                    au.play(Sfx::BigBoom);
                    self.ebullets.clear();
                    new_sequence(&mut s);
                    self.banner = Some((
                        "CONSTELLATION".into(),
                        "Shoot the stars in the order they shine.".into(),
                        3.0,
                    ));
                }
            }
            2 => {
                if s.show_t > 0.0 {
                    // light each star of the sequence in turn
                    s.show_t -= dt;
                    s.show_next -= dt;
                    if s.show_next <= 0.0 && s.show_k < s.order.len() {
                        s.node_flash[s.order[s.show_k]] = 0.65;
                        s.show_k += 1;
                        s.show_next = 0.75;
                        au.play(Sfx::Beep);
                    }
                }
                if s.exposed > 0.0 {
                    s.exposed -= dt;
                    if s.exposed <= 0.0 {
                        // too slow: the shield closes and a new sequence begins
                        new_sequence(&mut s);
                    }
                }
                s.spiral += dt * (1.1 + s.cracks as f32 * 0.25);
                s.spiral_cd -= dt;
                if s.spiral_cd <= 0.0 && s.show_t <= 0.0 {
                    s.spiral_cd = (0.24 - self.diff() * 0.006).max(0.14);
                    let arms = 2 + s.cracks.min(2);
                    for k in 0..arms {
                        let a = s.spiral + k as f32 / arms as f32 * TAU;
                        self.ebullets.push(EBullet::new(
                            core,
                            vec2(a.cos(), a.sin()) * 120.0 * bmul,
                            4.0,
                            0,
                        ));
                    }
                }
                if s.cracks >= 3 {
                    s.phase = 3;
                    s.phase_t = 0.0;
                    self.ebullets.clear();
                    self.fx.flash = 0.8;
                    self.fx.flash_color = Color::from_rgba(200, 160, 255, 255);
                    au.play(Sfx::BigBoom);
                    self.banner = Some((
                        "COLLAPSE".into(),
                        "Space is closing. Hit the core to push it back!".into(),
                        3.0,
                    ));
                }
            }
            3 => {
                let max_inset = (W - 140.0) / 2.0;
                s.inset = (s.inset + dt * (7.0 + self.diff() * 0.4)).min(max_inset);
                s.curtain_cd -= dt;
                if s.curtain_cd <= 0.0 {
                    s.curtain_cd = (2.5 - self.diff() * 0.06).max(1.4);
                    let lo = s.inset + 20.0;
                    let hi = W - s.inset - 20.0;
                    let gap_w = 100.0;
                    let gap = gen_range(lo, (hi - gap_w).max(lo + 1.0));
                    let mut x = s.inset + 8.0;
                    while x < W - s.inset - 8.0 {
                        if x < gap || x > gap + gap_w {
                            self.ebullets.push(EBullet::new(
                                vec2(x, 20.0),
                                vec2(0.0, 150.0 * bmul),
                                5.0,
                                2,
                            ));
                        }
                        x += 22.0;
                    }
                }
                s.burst_cd -= dt;
                if s.burst_cd <= 0.0 && alive {
                    s.burst_cd = 3.0;
                    let aim = (pp - core).normalize_or_zero();
                    for k in -2..=2 {
                        let a = k as f32 * 0.14;
                        let d = vec2(
                            aim.x * a.cos() - aim.y * a.sin(),
                            aim.x * a.sin() + aim.y * a.cos(),
                        );
                        self.ebullets
                            .push(EBullet::new(core, d * 230.0 * bmul, 5.0, 4));
                    }
                }
                if s.core_hp <= 0.0 {
                    s.phase = 4;
                    s.die_t = 0.0;
                    self.ebullets.clear();
                    au.music(None);
                }
            }
            _ => {
                s.die_t += dt;
                if (s.die_t * 8.0) as i32 != ((s.die_t - dt) * 8.0) as i32 {
                    let p = core + vec2(gen_range(-60.0, 60.0), gen_range(-30.0, 40.0));
                    self.fx
                        .explosion(p, ui::hsl(s.t * 0.5, 0.8, 0.7), gen_range(0, 3) == 0);
                    au.play_vol(Sfx::Boom, 0.6);
                    self.fx.shake = self.fx.shake.max(0.4);
                }
                if s.die_t > 4.5 && !s.done {
                    s.done = true;
                    self.fx.flash = 1.0;
                    self.fx.flash_color = WHITE;
                    au.play(Sfx::BigBoom);
                    au.play(Sfx::Fanfare);
                    self.events.push(Event::Rumble(None, 1.0, 1200));
                    self.score += 250_000;
                    self.ach(ach::SIGNAL_LOST);
                    self.banner = Some((
                        "THE SIGNAL GOES SILENT".into(),
                        "The stardust is only dust again.".into(),
                        5.0,
                    ));
                }
            }
        }
        // the walls of the collapse keep the ship inside
        if s.phase >= 3 {
            let hr = self.hit_r() + 6.0;
            let lo = s.inset + hr;
            let hi = W - s.inset - hr;
            self.player.pos.x = self.player.pos.x.clamp(lo, hi.max(lo));
            if let Some(p2) = &mut self.p2 {
                p2.pos.x = p2.pos.x.clamp(lo, hi.max(lo));
            }
        }
        self.signal_hits(&mut s, au);
        self.sig = Some(s);
    }

    /// Player shots against the signal's parts.
    fn signal_hits(&mut self, s: &mut Signal, au: &mut Audio) {
        let core = vec2(W / 2.0, CORE_Y);
        // railgun beams fired this frame count as a hit on whatever stands in their column
        let rails: Vec<f32> = self
            .arcade
            .rails
            .iter()
            .filter(|r| (r.life - arsenal::RAIL_LIFE).abs() < 1e-4)
            .map(|r| r.x)
            .collect();
        let mut hits: Vec<(Vec2, f32)> = vec![];
        for b in &mut self.bullets {
            if b.life <= 0.0 || matches!(b.kind, BKind::Hole | BKind::Rail) || b.pos.y > H * 0.7 {
                continue;
            }
            hits.push((b.pos, b.dmg));
            if signal_target(s, b.pos, core).is_some() {
                b.life = 0.0;
            }
        }
        for x in rails {
            let mut y = PLAYER_Y;
            while y > 0.0 {
                if signal_target(s, vec2(x, y), core).is_some() {
                    hits.push((vec2(x, y), 8.0));
                    break;
                }
                y -= 10.0;
            }
        }
        for (pos, dmg) in hits {
            let Some(t) = signal_target(s, pos, core) else {
                continue;
            };
            match t {
                Target::Echo => {
                    s.echo_hp -= dmg;
                    s.flash = 0.06;
                    self.fx
                        .sparks(pos, vec2(0.0, 1.0), Color::from_rgba(220, 160, 255, 255), 3);
                }
                Target::Node(i) => {
                    if s.show_t > 0.0 || s.exposed > 0.0 {
                        continue;
                    }
                    if s.order.get(s.progress) == Some(&i) {
                        s.progress += 1;
                        s.node_flash[i] = 0.5;
                        au.play(Sfx::Sparkle);
                        if s.progress >= s.order.len() {
                            s.exposed = 5.0;
                            s.crack_hp = 0.0;
                            au.play(Sfx::Fanfare);
                            self.fx.ring(core, 400.0, 0.5, GOLD, 4.0);
                        }
                    } else if !s.order[..s.progress].contains(&i) {
                        // wrong star: it bursts and the sequence starts over
                        au.play(Sfx::Deny);
                        let p = s.nodes[i];
                        for k in 0..12 {
                            let a = k as f32 / 12.0 * TAU;
                            self.ebullets.push(EBullet::new(
                                p,
                                vec2(a.cos(), a.sin()) * 150.0,
                                5.0,
                                2,
                            ));
                        }
                        let len = s.order.len();
                        replay(s, len);
                    }
                }
                Target::Core => match s.phase {
                    2 if s.exposed > 0.0 => {
                        s.crack_hp += dmg;
                        s.flash = 0.06;
                        if s.crack_hp >= 150.0 * s.hp_mul {
                            s.cracks += 1;
                            s.exposed = 0.0;
                            au.play(Sfx::BigBoom);
                            self.fx
                                .explosion(core, Color::from_rgba(255, 220, 140, 255), true);
                            if s.cracks < 3 {
                                new_sequence(s);
                            }
                        }
                    }
                    3 => {
                        s.core_hp -= dmg;
                        s.inset = (s.inset - dmg * 0.9).max(0.0);
                        s.flash = 0.06;
                        self.fx.sparks(pos, vec2(0.0, 1.0), WHITE, 3);
                    }
                    _ => {
                        self.fx.sparks(
                            pos,
                            vec2(0.0, 1.0),
                            Color::from_rgba(150, 140, 255, 255),
                            2,
                        );
                    }
                },
            }
        }
    }

    /// The signal touches the ship (the echo's body or the collapse walls do not).
    pub(super) fn signal_touches(&self, p: Vec2, r: f32) -> bool {
        let Some(s) = &self.sig else { return false };
        s.phase == 1 && p.distance(vec2(s.echo_x, ECHO_Y)) < 14.0 + r
    }

    pub(super) fn draw_signal(&self, off: Vec2, art: &Art) {
        let Some(s) = &self.sig else { return };
        let core = vec2(W / 2.0, CORE_Y) + off;
        let fade = (s.t / 2.0).min(1.0);
        let hue = s.t * 0.15;
        // collapse walls
        if s.phase >= 3 && s.inset > 0.0 {
            let c = Color::from_rgba(30, 10, 50, 245);
            draw_rectangle(off.x, 0.0, s.inset, H, c);
            draw_rectangle(W - s.inset + off.x, 0.0, s.inset, H, c);
            let edge = Color::new(0.8, 0.5, 1.0, 0.6 + (s.t * 6.0).sin() * 0.3);
            draw_line(s.inset + off.x, 0.0, s.inset + off.x, H, 3.0, edge);
            draw_line(W - s.inset + off.x, 0.0, W - s.inset + off.x, H, 3.0, edge);
            light::add(
                vec2(s.inset, PLAYER_Y) + off,
                90.0,
                Color::new(0.8, 0.4, 1.0, 0.5),
            );
            light::add(
                vec2(W - s.inset, PLAYER_Y) + off,
                90.0,
                Color::new(0.8, 0.4, 1.0, 0.5),
            );
        }
        if s.phase >= 4 && s.done {
            return;
        }
        // constellation
        if s.phase == 2 {
            for w in s.order.windows(2) {
                let a = s.nodes[w[0]] + off;
                let b = s.nodes[w[1]] + off;
                if s.show_t > 0.0 {
                    draw_line(a.x, a.y, b.x, b.y, 1.0, Color::new(0.8, 0.8, 1.0, 0.25));
                }
            }
            for (i, n) in s.nodes.iter().enumerate() {
                let p = *n + off;
                let done = s.order[..s.progress].contains(&i);
                let lit = s.node_flash[i] > 0.0;
                let c = if done {
                    GOLD
                } else if lit {
                    WHITE
                } else {
                    Color::new(0.55, 0.55, 0.85, 0.8)
                };
                let r = if lit { 16.0 } else { 12.0 };
                draw_circle(p.x, p.y, r + 8.0, Color::new(c.r, c.g, c.b, 0.15));
                star_shape(p, r, s.t * 1.5 + i as f32, c);
                if lit || done {
                    light::add(p, 60.0, Color::new(c.r, c.g, c.b, 0.7));
                }
                if s.show_t > 0.0 && lit {
                    if let Some(k) = s.order.iter().position(|o| *o == i) {
                        ui::text_c(&format!("{}", k + 1), p.x, p.y - 22.0, 14.0, WHITE);
                    }
                }
            }
        }
        // the core
        let shield = s.phase < 3 && !(s.phase == 2 && s.exposed > 0.0);
        let pulse = 1.0 + (s.t * 3.0).sin() * 0.08;
        let base = ui::hsl(hue, 0.7, 0.6);
        let flash = if s.flash > 0.0 { 1.0 } else { 0.0 };
        draw_circle(
            core.x,
            core.y,
            46.0 * pulse * fade,
            Color::new(base.r, base.g, base.b, 0.18),
        );
        draw_circle(
            core.x,
            core.y,
            30.0 * pulse * fade,
            Color::new(base.r, base.g, base.b, 0.6),
        );
        draw_circle(
            core.x,
            core.y,
            16.0 * fade,
            Color::new(1.0, 1.0 - 0.2 * (1.0 - flash), 1.0, 1.0),
        );
        for k in 0..9 {
            let a = s.t * (0.8 + (k % 3) as f32 * 0.3) + k as f32 / 9.0 * TAU;
            let r = 40.0 + (k % 3) as f32 * 14.0;
            let p = core + vec2(a.cos() * r, a.sin() * r * 0.55);
            star_shape(p, 4.0, a, ui::hsl(hue + k as f32 * 0.1, 0.6, 0.8));
        }
        if shield {
            draw_poly_lines(
                core.x,
                core.y,
                6,
                40.0,
                s.t * 30.0,
                2.0,
                Color::new(0.7, 0.7, 1.0, 0.5),
            );
        }
        light::add(core, 160.0, Color::new(base.r, base.g, base.b, 0.6));
        // the echo ship
        if s.phase == 1 {
            let p = vec2(s.echo_x, ECHO_Y) + off;
            let spr = &art.ships[self.ship_idx()][self.cfg.skin];
            let tint = Color::new(0.75, 0.55, 1.0, 0.9);
            draw_circle(p.x, p.y, 22.0, Color::new(0.6, 0.3, 1.0, 0.18));
            spr.draw(p, S, std::f32::consts::PI, tint, s.flash > 0.0);
            light::add(p, 70.0, Color::new(0.7, 0.4, 1.0, 0.5));
        }
    }

    pub(super) fn draw_signal_bar(&self) {
        let Some(s) = &self.sig else { return };
        if s.phase == 0 || s.done {
            return;
        }
        let (name, frac) = match s.phase {
            1 => ("THE SIGNAL · ECHO", s.echo_hp / s.echo_max),
            2 => (
                "THE SIGNAL · CONSTELLATION",
                1.0 - (s.cracks as f32 + (s.crack_hp / (150.0 * s.hp_mul)).min(1.0)) / 3.0,
            ),
            _ => ("THE SIGNAL · COLLAPSE", s.core_hp / s.core_max),
        };
        let w = W - 40.0;
        draw_rectangle(20.0, 10.0, w, 8.0, Color::new(0.1, 0.0, 0.2, 0.8));
        draw_rectangle(
            20.0,
            10.0,
            w * frac.clamp(0.0, 1.0),
            8.0,
            ui::hsl(s.t * 0.15, 0.7, 0.65),
        );
        draw_rectangle_lines(20.0, 10.0, w, 8.0, 1.0, WHITE);
        ui::text_c(
            name,
            W / 2.0,
            36.0,
            16.0,
            Color::from_rgba(210, 170, 255, 255),
        );
        if s.phase == 2 && s.exposed > 0.0 {
            ui::text_c("THE CORE IS OPEN!", W / 2.0, 56.0, 14.0, GOLD);
        }
    }
}

enum Target {
    Echo,
    Node(usize),
    Core,
}

fn signal_target(s: &Signal, p: Vec2, core: Vec2) -> Option<Target> {
    match s.phase {
        1 => {
            if p.distance(vec2(s.echo_x, ECHO_Y)) < 20.0 {
                return Some(Target::Echo);
            }
        }
        2 => {
            for (i, n) in s.nodes.iter().enumerate() {
                if p.distance(*n) < 17.0 {
                    return Some(Target::Node(i));
                }
            }
        }
        _ => {}
    }
    if (1..4).contains(&s.phase) && p.distance(core) < 34.0 {
        return Some(Target::Core);
    }
    None
}

/// A fresh random sequence, one star longer for every crack already made.
fn new_sequence(s: &mut Signal) {
    let len = 3 + s.cracks as usize;
    let mut idx: Vec<usize> = (0..s.nodes.len()).collect();
    for i in (1..idx.len()).rev() {
        let j = gen_range(0, i + 1);
        idx.swap(i, j);
    }
    s.order = idx.into_iter().take(len).collect();
    replay(s, len);
}

/// Shows the current sequence again from the start.
fn replay(s: &mut Signal, len: usize) {
    s.progress = 0;
    s.show_k = 0;
    s.show_next = 0.4;
    s.show_t = len as f32 * 0.75 + 0.6;
}

fn star_shape(p: Vec2, r: f32, rot: f32, c: Color) {
    for k in 0..5 {
        let a = rot + k as f32 / 5.0 * TAU;
        let b = a + TAU / 10.0;
        let c0 = p + vec2(a.cos(), a.sin()) * r;
        let c1 = p + vec2((b - TAU / 5.0).cos(), (b - TAU / 5.0).sin()) * r * 0.4;
        let c2 = p + vec2(b.cos(), b.sin()) * r * 0.4;
        draw_triangle(p, c0, c1, c);
        draw_triangle(p, c0, c2, c);
    }
}
