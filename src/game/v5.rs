//! Sistemas da versão 5: gatilho por toque com tiro carregado, tiro automático
//! comprável, letras S-T-A-R-D-U-S-T, a Nave-Mãe misteriosa, o Portal de Salto,
//! bônus de tempo e os itens novos da loja.

use super::*;
use std::f32::consts::TAU;

pub const WORD: [char; 8] = ['S', 'T', 'A', 'R', 'D', 'U', 'S', 'T'];

struct Ufo {
    pos: Vec2,
    vel: Vec2,
    hp: i32,
    flash: f32,
    beep: f32,
}

struct Mine {
    pos: Vec2,
    t: f32,
    hp: i32,
}

#[derive(Default)]
pub struct V5State {
    pub autofire: bool,
    trigger_t: f32,
    pub charge: f32,
    pub capacitor: bool,
    pub sight: bool,
    pub core: u32,
    pub letters: [bool; 8],
    pub words: u32,
    ufo: Option<Ufo>,
    ufo_t: f32,
    pub ufos: u32,
    mines: Vec<Mine>,
    mine_cd: f32,
    blink_cd: f32,
    warp: Option<(Vec2, f32)>,
    warp_go: bool,
    pub warps: u32,
    pub boss_t: f32,
    pub fast_waves: u32,
    shots0: u32,
    landed0: u32,
}

impl V5State {
    pub fn new() -> V5State {
        V5State {
            ufo_t: 28.0,
            mine_cd: 1.0,
            blink_cd: 3.0,
            ..Default::default()
        }
    }
}

pub(super) fn draw_letter_drop(p: Vec2, i: usize, time: f32) {
    let spin = (time * 3.0 + i as f32).cos();
    let w = 15.0 * spin.abs().max(0.2);
    draw_circle(p.x, p.y, 20.0, Color::new(1.0, 0.85, 0.3, 0.2));
    draw_ellipse(p.x, p.y, w, 15.0, 0.0, Color::from_rgba(255, 200, 60, 255));
    draw_ellipse_lines(
        p.x,
        p.y,
        w,
        15.0,
        0.0,
        2.0,
        Color::from_rgba(150, 90, 10, 255),
    );
    if spin.abs() > 0.45 {
        crate::ui::text_c(
            &WORD[i].to_string(),
            p.x,
            p.y + 7.0,
            19.0,
            Color::from_rgba(90, 40, 0, 255),
        );
    }
}

impl Game {
    /// Caixa Misteriosa da loja: quase sempre boa, às vezes uma pegadinha.
    pub(super) fn open_box(&mut self, au: &mut Audio) {
        let (title, sub) = match gen_range(0, 100) {
            0..=19 => {
                self.crystals += 45.0;
                ("CRISTAIS!", "+45 cristais")
            }
            20..=34 => {
                if self.player.level < 5 && !self.player.weapon.is_relic() {
                    self.player.level += 1;
                }
                ("ARMA REFORÇADA", "+1 nível de arma")
            }
            35..=46 => {
                self.player.bombs = (self.player.bombs + 2).min(9);
                ("BOMBAS", "+2 bombas")
            }
            47..=55 => {
                self.player.lives = (self.player.lives + 1).min(9);
                au.play(Sfx::OneUp);
                ("VIDA EXTRA!", "+1 nave")
            }
            56..=64 => {
                let c = cards::roll(1);
                self.give_card(c, au);
                ("UMA CARTA!", "direto para a coleção")
            }
            65..=72 => {
                self.player.shield = true;
                ("ESCUDO", "um golpe a menos")
            }
            73..=79 => {
                self.z.autofire = true;
                ("TIRO AUTOMÁTICO!", "de graça, que sorte")
            }
            80..=89 => ("VAZIA...", "só tinha poeira estelar"),
            _ => {
                self.crystals = (self.crystals - 20.0).max(0.0);
                ("UMA TRAÇA!", "comeu 20 cristais")
            }
        };
        self.banner = Some((title.into(), sub.into(), 2.2));
    }

    // -----------------------------------------------------------------------
    // Letras S-T-A-R-D-U-S-T
    // -----------------------------------------------------------------------

    pub(super) fn v5_on_kill(&mut self, e: &Enemy) {
        if self.secret_stage || self.y.room.is_some() || e.harmless {
            return;
        }
        let on_screen = self
            .drops
            .iter()
            .any(|d| matches!(d.kind, DropKind::Letter(_)));
        let chance = if e.kind == Kind::Cargueiro {
            0.2
        } else if e.elite {
            0.06
        } else {
            0.013
        };
        if !on_screen && gen_range(0.0, 1.0) < chance {
            self.drop_letter(e.pos);
        }
    }

    fn drop_letter(&mut self, pos: Vec2) {
        let missing: Vec<usize> = (0..8).filter(|&i| !self.z.letters[i]).collect();
        if let Some(&i) =
            missing.get(gen_range(0, missing.len().max(1)).min(missing.len().saturating_sub(1)))
        {
            self.spawn_drop(pos, DropKind::Letter(i));
        }
    }

    pub(super) fn take_letter(&mut self, i: usize, au: &mut Audio) {
        let pp = self.player.pos;
        self.z.letters[i] = true;
        au.play(Sfx::Contract);
        self.fx
            .text(pp - vec2(0.0, 44.0), WORD[i].to_string(), GOLD, 30.0);
        if self.z.letters.iter().all(|l| *l) {
            self.z.letters = [false; 8];
            self.z.words += 1;
            au.play(Sfx::Fanfare);
            au.play(Sfx::OneUp);
            self.fx.flash = 0.5;
            self.fx.flash_color = GOLD;
            let bonus = 50_000 * self.z.words;
            self.score += bonus;
            self.player.lives = (self.player.lives + 1).min(9);
            if self.z.words >= 2 && !self.player.weapon.is_relic() {
                self.player.level = 5;
            }
            self.events.push(Event::Word);
            self.ach(ach::SOLETRADOR);
            self.banner = Some((
                "S T A R D U S T !".into(),
                format!("Vida extra e +{} pontos", bonus),
                3.0,
            ));
        }
    }

    // -----------------------------------------------------------------------
    // Bônus de tempo
    // -----------------------------------------------------------------------

    pub(super) fn v5_wave_start(&mut self) {
        self.z.shots0 = self.sector_shots;
        self.z.landed0 = self.sector_landed;
    }

    pub(super) fn wave_speed_bonus(&mut self) {
        let shots = self.sector_shots - self.z.shots0.min(self.sector_shots);
        let landed = self.sector_landed - self.z.landed0.min(self.sector_landed);
        if shots >= 8 && landed >= shots && !self.classic() {
            let bonus = 3000 * (1 + self.cycle);
            self.score += bonus;
            self.fx.text(
                vec2(W / 2.0, H * 0.5),
                format!("TIRO CERTEIRO +{bonus}"),
                GOLD,
                20.0,
            );
        }
        let limit = 14.0;
        if self.wave_age < limit && !self.classic() {
            let bonus = (((limit - self.wave_age) * 400.0) as u32 / 100) * 100;
            if bonus > 0 {
                self.score += bonus;
                self.fx.text(
                    vec2(W / 2.0, H * 0.45),
                    format!("ONDA RELÂMPAGO +{bonus}"),
                    CYAN,
                    22.0,
                );
                if self.wave_age < 7.0 {
                    self.z.fast_waves += 1;
                    self.ach(ach::RELAMPAGO);
                }
            }
        }
    }

    pub(super) fn boss_time_line(&self) -> Option<(String, u32, bool)> {
        let t = self.z.boss_t;
        if t > 0.0 && t < 45.0 {
            let v = ((45.0 - t) * 600.0) as u32 / 100 * 100;
            Some((format!("CHEFE EM {:.0}s", t.ceil()), v, false))
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // Portal de Salto (estágio bônus perfeito)
    // -----------------------------------------------------------------------

    pub(super) fn offer_warp(&mut self) {
        let ok = matches!(self.cfg.mode, Mode::Campanha | Mode::Expedicao)
            && self.sector() + 1 < self.sectors_in_run().min(SECTORS + 100)
            && (self.cfg.mode == Mode::Expedicao || self.sector() <= 6)
            && !self.x.void;
        if ok {
            let x = if self.player.pos.x < W / 2.0 {
                W * 0.78
            } else {
                W * 0.22
            };
            self.z.warp = Some((vec2(x, PLAYER_Y), 9.0));
            self.banner = Some((
                "PORTAL DE SALTO".into(),
                "Entre nele para pular o chefe e o setor!".into(),
                3.0,
            ));
        }
    }

    /// Chamado no fim de Game::update: aplica o salto se a nave entrou no portal.
    pub(super) fn v5_post_update(&mut self, au: &mut Audio) {
        if !self.z.warp_go {
            return;
        }
        self.z.warp_go = false;
        self.z.warp = None;
        self.boss = None;
        self.enemies.clear();
        self.ebullets.clear();
        self.z.mines.clear();
        self.collect_all_drops(au);
        self.wave = self.sector() * 5 + 1;
        self.sectors_cleared += 1;
        self.score += 25_000;
        self.crystals += 30.0;
        self.z.warps += 1;
        self.ach(ach::ATALHO);
        au.play(Sfx::Portal);
        self.fx.flash = 1.0;
        self.fx.flash_color = WHITE;
        self.banner = Some((
            "SALTO ESPACIAL!".into(),
            "+25000 pontos e 30 cristais".into(),
            2.5,
        ));
        self.phase = Phase::Intro { t: 0.0 };
    }

    // -----------------------------------------------------------------------
    // Atualização
    // -----------------------------------------------------------------------

    pub(super) fn v5_update(&mut self, dt: f32, au: &mut Audio) {
        if self.boss.as_ref().map(|b| b.targetable()).unwrap_or(false) {
            self.z.boss_t += dt;
        }
        let pp = self.player.pos;
        let alive = self.player.alive();
        // portal
        if let Some((pos, t)) = &mut self.z.warp {
            *t -= dt;
            if alive && pos.distance(pp) < 30.0 {
                self.z.warp_go = true;
            } else if *t <= 0.0 {
                self.z.warp = None;
            }
        }
        // nave-mãe misteriosa
        let calm = self.wave_kind == WaveKind::Normal
            && self.boss.is_none()
            && !self.room_active()
            && !self.secret_stage
            && !self.enemies.is_empty();
        self.z.ufo_t -= dt;
        if self.z.ufo.is_none() && self.z.ufo_t <= 0.0 {
            self.z.ufo_t = gen_range(22.0, 38.0) * if self.mods.ufo_more { 0.5 } else { 1.0 };
            if calm && gen_range(0, 100) < 55 {
                let left = gen_range(0, 2) == 0;
                self.z.ufo = Some(Ufo {
                    pos: vec2(if left { -30.0 } else { W + 30.0 }, 46.0),
                    vel: vec2(if left { 95.0 } else { -95.0 }, 0.0),
                    hp: 5 + self.stage().tier as i32,
                    flash: 0.0,
                    beep: 0.0,
                });
            }
        }
        let mut ufo_dead = None;
        let mut by_charge = false;
        if let Some(u) = &mut self.z.ufo {
            u.pos += u.vel * dt;
            u.pos.y = 46.0 + (self.time * 3.0).sin() * 4.0;
            u.flash = (u.flash - dt).max(0.0);
            u.beep -= dt;
            if u.beep <= 0.0 {
                u.beep = 0.45;
                au.play_vol(Sfx::Beep, 0.18);
            }
            for b in &mut self.bullets {
                if b.life > 0.0 && b.kind != BKind::Hole && b.pos.distance(u.pos) < 22.0 {
                    if b.pierce != 255 {
                        b.life = 0.0;
                    }
                    u.hp -= b.dmg.ceil() as i32;
                    u.flash = 0.06;
                    if u.hp <= 0 && b.kind == BKind::Charge {
                        by_charge = true;
                    }
                }
            }
            if u.hp <= 0 {
                ufo_dead = Some(u.pos);
            } else if u.pos.x < -50.0 || u.pos.x > W + 50.0 {
                self.z.ufo = None;
            }
        }
        if let Some(pos) = ufo_dead {
            self.z.ufo = None;
            self.z.ufos += 1;
            let pts = [300, 500, 800, 1000, 1500, 3000, 5000][gen_range(0, 7)];
            self.score += pts * self.multiplier();
            au.play(Sfx::BigBoom);
            self.fx
                .explosion(pos, Color::from_rgba(255, 120, 200, 255), true);
            self.fx.text(
                pos + vec2(0.0, 20.0),
                format!("{}", pts * self.multiplier()),
                GOLD,
                26.0,
            );
            for _ in 0..3 {
                self.spawn_drop(pos, DropKind::Crystal(5));
            }
            if self.z.letters.iter().any(|l| !l) && gen_range(0, 100) < 60 {
                self.drop_letter(pos);
            } else {
                self.spawn_drop(pos, DropKind::Power(PowerKind::random()));
            }
            self.events.push(Event::Ufo);
            self.ach(ach::OVNI);
            if by_charge && !self.classic() {
                self.spawn_drop(pos, DropKind::Card(cards::id::CONTATO));
                self.banner = Some((
                    "SINAL CAPTADO".into(),
                    "A Nave-Mãe deixou algo para trás...".into(),
                    2.5,
                ));
            }
        }
        // minadores largam minas durante o mergulho
        self.z.mine_cd -= dt;
        if self.z.mine_cd <= 0.0 {
            self.z.mine_cd = 0.75;
            let drops: Vec<Vec2> = self
                .enemies
                .iter()
                .filter(|e| {
                    e.kind == Kind::Minador
                        && e.diving()
                        && e.active()
                        && e.pos.y > 30.0
                        && e.pos.y < H * 0.7
                })
                .map(|e| e.pos)
                .collect();
            for p in drops {
                self.z.mines.push(Mine {
                    pos: p,
                    t: 0.0,
                    hp: 1,
                });
            }
        }
        let bspeed = self.bullet_mul();
        let mut boom = vec![];
        for m in &mut self.z.mines {
            m.t += dt;
            m.pos.y += 55.0 * dt;
            for b in &mut self.bullets {
                if b.life > 0.0 && b.kind != BKind::Hole && b.pos.distance(m.pos) < 11.0 {
                    b.life = 0.0;
                    m.hp = 0;
                }
            }
            if m.hp <= 0 {
                m.t = 99.0;
            } else if m.t > 1.8 {
                boom.push(m.pos);
                m.t = 99.0;
            }
        }
        self.z.mines.retain(|m| m.t < 90.0);
        for p in boom {
            for k in 0..8 {
                let a = k as f32 / 8.0 * TAU;
                self.ebullets.push(EBullet::new(
                    p,
                    vec2(a.cos(), a.sin()) * 130.0 * bspeed,
                    4.0,
                    1,
                ));
            }
            au.play_vol(Sfx::Pop, 0.5);
            self.fx.ring(p, 160.0, 0.25, ORANGE, 2.0);
        }
        // saltadores pulam para outro lugar da formação
        self.z.blink_cd -= dt;
        if self.z.blink_cd <= 0.0 {
            self.z.blink_cd = gen_range(2.0, 3.5);
            let taken: Vec<(usize, usize)> = self.enemies.iter().map(|e| (e.col, e.row)).collect();
            let free: Vec<(usize, usize)> = (0..COLS)
                .flat_map(|c| (0..5).map(move |r| (c, r)))
                .filter(|s| !taken.contains(s))
                .collect();
            if !free.is_empty() {
                let mut moved = vec![];
                for e in &mut self.enemies {
                    if e.kind == Kind::Saltador
                        && matches!(e.state, EState::Formation)
                        && !e.dead
                        && gen_range(0, 2) == 0
                    {
                        let (c, r) = free[gen_range(0, free.len())];
                        moved.push(e.pos);
                        e.col = c;
                        e.row = r;
                    }
                }
                for p in moved {
                    self.fx
                        .burst(p, Color::from_rgba(255, 170, 255, 255), 10, 120.0);
                    au.play_vol(Sfx::Portal, 0.15);
                }
            }
        }
    }

    /// Minas encostando na nave.
    pub(super) fn v5_hazards(&mut self, au: &mut Audio) {
        let pp = self.player.pos;
        if !self.player.alive() || self.player.invuln > 0.0 || self.star_t > 0.0 {
            return;
        }
        let r = self.hit_r() + 7.0;
        if let Some(i) = self.z.mines.iter().position(|m| m.pos.distance(pp) < r) {
            self.z.mines.remove(i);
            self.hurt_player(au);
        }
    }

    pub(super) fn v5_draw(&self, off: Vec2) {
        for m in &self.z.mines {
            let p = m.pos + off;
            let blink = (m.t * (4.0 + m.t * 6.0)).sin() > 0.0;
            draw_circle(p.x, p.y, 9.0, Color::new(1.0, 0.5, 0.1, 0.2));
            draw_circle(p.x, p.y, 6.5, Color::from_rgba(25, 0, 15, 255));
            draw_circle(p.x, p.y, 5.0, Color::from_rgba(150, 60, 20, 255));
            draw_circle(p.x, p.y, 2.5, if blink { ORANGE } else { RED });
            for k in 0..4 {
                let a = k as f32 * TAU / 4.0 + m.t;
                let q = p + vec2(a.cos(), a.sin()) * 8.0;
                draw_circle(q.x, q.y, 1.5, Color::from_rgba(150, 60, 20, 255));
            }
        }
        if let Some(u) = &self.z.ufo {
            let p = u.pos + off;
            let body = if u.flash > 0.0 {
                WHITE
            } else {
                Color::from_rgba(110, 100, 140, 255)
            };
            draw_triangle(
                p + vec2(-10.0, 10.0),
                p + vec2(10.0, 10.0),
                p + vec2(0.0, 40.0),
                Color::new(1.0, 0.5, 0.9, 0.12),
            );
            draw_ellipse(
                p.x,
                p.y + 4.0,
                30.0,
                10.0,
                0.0,
                Color::from_rgba(20, 10, 30, 255),
            );
            draw_ellipse(p.x, p.y + 3.0, 28.0, 8.5, 0.0, body);
            draw_ellipse(
                p.x,
                p.y + 1.0,
                22.0,
                4.0,
                0.0,
                Color::from_rgba(170, 160, 200, 255),
            );
            draw_ellipse(
                p.x,
                p.y - 3.0,
                12.0,
                9.0,
                0.0,
                Color::from_rgba(20, 10, 30, 255),
            );
            draw_ellipse(
                p.x,
                p.y - 3.0,
                10.5,
                8.0,
                0.0,
                Color::from_rgba(255, 110, 210, 255),
            );
            draw_circle(p.x - 3.0, p.y - 6.0, 2.5, Color::new(1.0, 1.0, 1.0, 0.7));
            for k in 0..6 {
                let x = p.x - 20.0 + k as f32 * 8.0;
                let on = ((self.time * 10.0) as i32 + k) % 3 == 0;
                draw_circle(x, p.y + 6.0, 2.0, if on { YELLOW } else { ORANGE });
            }
        }
        if let Some((pos, t)) = self.z.warp {
            let p = pos + off;
            let a = (t * 3.0).min(1.0);
            for k in 0..5 {
                let r = 10.0 + k as f32 * 6.0 + (self.time * 5.0 - k as f32).sin() * 2.0;
                let col = crate::ui::hsl(self.time * 0.5 + k as f32 * 0.1, 0.9, 0.65);
                draw_circle_lines(
                    p.x,
                    p.y,
                    r,
                    2.0,
                    Color::new(col.r, col.g, col.b, a * (1.0 - k as f32 * 0.15)),
                );
            }
            draw_circle(p.x, p.y, 9.0, Color::new(1.0, 1.0, 1.0, 0.8 * a));
            crate::ui::text_c(
                &format!("SALTO {:.0}", t.ceil()),
                p.x,
                p.y - 44.0,
                13.0,
                WHITE,
            );
        }
    }

    pub fn debug_ufo_soon(&mut self) {
        self.z.ufo_t = 4.0;
    }

    /// Linha de letras para HUD e painéis.
    pub fn letters_line(&self) -> Vec<(char, bool)> {
        WORD.iter()
            .zip(self.z.letters.iter())
            .map(|(c, l)| (*c, *l))
            .collect()
    }

    /// Poder de fogo total: classe da arma x Núcleos de Dano comprados.
    pub fn power_mul(&self) -> f32 {
        self.player.weapon.tier_mul() * self.core_mul()
    }

    pub fn core_mul(&self) -> f32 {
        1.0 + 0.15 * self.z.core as f32
    }

    /// Tempo de carga para o tiro carregado.
    pub(super) fn charge_full(&self) -> f32 {
        if self.z.capacitor { 0.6 } else { 1.0 }
    }

    /// Decide se a nave atira neste quadro. Sem o Tiro Automático, cada toque
    /// dispara uma rajada curta; segurar o botão carrega um tiro especial.
    pub(super) fn trigger(&mut self, inp: &Input, dt: f32, au: &mut Audio) -> bool {
        if inp.fire_p {
            let cd = self.player.cd.max(0.0);
            self.z.trigger_t = (cd + 0.12).min(0.7);
        }
        self.z.trigger_t -= dt;
        if self.z.autofire {
            self.z.charge = 0.0;
            return inp.fire;
        }
        if inp.fire {
            self.z.charge += dt;
        } else {
            if self.z.charge >= self.charge_full() && self.y.jam_t <= 0.0 {
                self.charge_shot(au);
            }
            self.z.charge = 0.0;
        }
        self.z.trigger_t > 0.0
    }

    fn charge_shot(&mut self, au: &mut Audio) {
        let p = self.player.pos - vec2(0.0, 24.0);
        let lv = self.effective_level() as f32;
        let mut dmg = (5.0 + lv * 1.6) * self.ship().dmg * self.power_mul();
        if self.z.capacitor {
            dmg *= 1.4;
        }
        let mut b = PBullet::new(p, vec2(0.0, -560.0), dmg, BKind::Charge, self.player.weapon);
        b.pierce = 255;
        b.life = 2.0;
        b.phase = 12.0 + lv * 1.5;
        self.bullets.push(b);
        au.play(Sfx::Laser);
        au.play_vol(Sfx::Bomb, 0.35);
        self.fx.ring(p, 260.0, 0.3, WHITE, 3.0);
        self.fx.shake = self.fx.shake.max(0.15);
    }

    pub(super) fn draw_charge(&self, p: Vec2) {
        let full = self.charge_full();
        let c = self.z.charge;
        if c < 0.25 || self.z.autofire {
            return;
        }
        let k = ((c - 0.25) / (full - 0.25)).min(1.0);
        let tip = p - vec2(0.0, 26.0);
        let ready = c >= full;
        let col = if ready {
            crate::ui::hsl(self.time * 2.0, 0.9, 0.7)
        } else {
            Color::new(0.6, 0.9, 1.0, 0.8)
        };
        draw_circle(
            tip.x,
            tip.y,
            4.0 + k * 8.0,
            Color::new(col.r, col.g, col.b, 0.25 + k * 0.3),
        );
        draw_circle(tip.x, tip.y, 2.0 + k * 4.0, WHITE);
        if ready {
            let r = 14.0 + (self.time * 20.0).sin() * 2.0;
            draw_circle_lines(tip.x, tip.y, r, 1.5, col);
        } else {
            for i in 0..6 {
                let a = self.time * 8.0 + i as f32 / 6.0 * TAU;
                let q = tip + vec2(a.cos(), a.sin()) * (18.0 - k * 12.0);
                draw_circle(q.x, q.y, 1.5, col);
            }
        }
    }

    pub(super) fn draw_sight(&self, p: Vec2) {
        if !self.z.sight {
            return;
        }
        let mut y = p.y - 30.0;
        while y > 0.0 {
            let a = 0.35 * (y / H);
            draw_line(p.x, y, p.x, y - 8.0, 1.0, Color::new(1.0, 0.3, 0.3, a));
            y -= 16.0;
        }
    }
}

pub(super) fn draw_charge_bullet(b: &PBullet, p: Vec2, time: f32) {
    let r = b.phase;
    draw_circle(p.x, p.y, r * 1.6, Color::new(0.5, 0.9, 1.0, 0.15));
    draw_circle(p.x, p.y, r, Color::new(0.7, 0.95, 1.0, 0.6));
    draw_circle(p.x, p.y, r * 0.55, WHITE);
    for k in 0..4 {
        let a = time * 10.0 + k as f32 / 4.0 * TAU;
        let q = p + vec2(a.cos(), a.sin()) * r * 1.1;
        draw_circle(q.x, q.y, 2.0, WHITE);
    }
}
