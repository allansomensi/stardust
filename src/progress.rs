//! Progressão permanente: patentes, conquistas, bestiário, modos e dificuldades.

use macroquad::prelude::*;

// ---------------------------------------------------------------------------
// Modos e dificuldades
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Campanha,
    Classico,
    Diario,
    Sobrevivencia,
    Chefoes,
    Expedicao,
}

pub const MODES: [Mode; 6] = [
    Mode::Campanha,
    Mode::Classico,
    Mode::Diario,
    Mode::Sobrevivencia,
    Mode::Chefoes,
    Mode::Expedicao,
];

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Campanha => "CAMPANHA",
            Mode::Classico => "CLÁSSICO",
            Mode::Diario => "DESAFIO DIÁRIO",
            Mode::Sobrevivencia => "SOBREVIVÊNCIA",
            Mode::Chefoes => "CHEFÕES",
            Mode::Expedicao => "EXPEDIÇÃO",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Mode::Campanha => "8 setores, chefes, loja e cartas. A aventura completa.",
            Mode::Classico => {
                "Como nos fliperamas: só a Rajada, sem loja, sem cartas. Ondas sem fim."
            }
            Mode::Diario => "3 setores com uma regra maluca que muda todo dia. Igual para todos.",
            Mode::Sobrevivencia => "Ondas sem pausa, cada vez mais rápidas. Quanto você aguenta?",
            Mode::Chefoes => "Todos os chefes em sequência, contra o relógio.",
            Mode::Expedicao => {
                "Setores sem fim. A cada setor, escolha 1 de 3 bênçãos. Até onde você vai?"
            }
        }
    }
    pub fn unlock_rank(self) -> usize {
        match self {
            Mode::Campanha => 0,
            Mode::Classico => 1,
            Mode::Diario => 3,
            Mode::Sobrevivencia => 4,
            Mode::Chefoes => 7,
            Mode::Expedicao => 6,
        }
    }
    pub fn tag(self) -> u8 {
        self as u8
    }
    pub fn from_tag(t: u8) -> Mode {
        MODES.get(t as usize).copied().unwrap_or(Mode::Campanha)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    Facil,
    Normal,
    Dificil,
    Pesadelo,
}

pub const DIFFS: [Difficulty; 4] = [
    Difficulty::Facil,
    Difficulty::Normal,
    Difficulty::Dificil,
    Difficulty::Pesadelo,
];

impl Difficulty {
    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Facil => "FÁCIL",
            Difficulty::Normal => "NORMAL",
            Difficulty::Dificil => "DIFÍCIL",
            Difficulty::Pesadelo => "PESADELO",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Difficulty::Facil => Color::from_rgba(120, 230, 120, 255),
            Difficulty::Normal => Color::from_rgba(120, 200, 255, 255),
            Difficulty::Dificil => Color::from_rgba(255, 170, 60, 255),
            Difficulty::Pesadelo => Color::from_rgba(255, 60, 90, 255),
        }
    }
    pub fn bullet_speed(self) -> f32 {
        [0.8, 1.0, 1.15, 1.3][self as usize]
    }
    pub fn fire_rate(self) -> f32 {
        [0.55, 1.0, 1.25, 1.5][self as usize]
    }
    /// Ritmo dos inimigos (movimento, mergulhos e chefes).
    pub fn pace(self) -> f32 {
        [0.8, 1.0, 1.15, 1.3][self as usize]
    }
    pub fn lives(self) -> i32 {
        [2, 0, -1, -1][self as usize]
    }
    pub fn xp_mul(self) -> f32 {
        [0.6, 1.0, 1.5, 2.2][self as usize]
    }
    pub fn score_mul(self) -> f32 {
        [0.5, 1.0, 1.5, 2.0][self as usize]
    }
    pub fn continues(self) -> u32 {
        [3, 2, 1, 0][self as usize]
    }
    pub fn from_index(i: u8) -> Difficulty {
        DIFFS.get(i as usize).copied().unwrap_or(Difficulty::Normal)
    }
}

/// Regras do desafio diário.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Daily {
    TirosDobrados,
    InimigosVelozes,
    SoLaser,
    CristaisEmDobro,
    Neblina,
    Espelho,
    VidroFino,
    ChuvaDeCapsulas,
}

pub const DAILIES: [Daily; 8] = [
    Daily::TirosDobrados,
    Daily::InimigosVelozes,
    Daily::SoLaser,
    Daily::CristaisEmDobro,
    Daily::Neblina,
    Daily::Espelho,
    Daily::VidroFino,
    Daily::ChuvaDeCapsulas,
];

impl Daily {
    pub fn name(self) -> &'static str {
        match self {
            Daily::TirosDobrados => "TIROS DOBRADOS",
            Daily::InimigosVelozes => "INIMIGOS VELOZES",
            Daily::SoLaser => "SÓ LASER",
            Daily::CristaisEmDobro => "CRISTAIS EM DOBRO",
            Daily::Neblina => "NEBLINA",
            Daily::Espelho => "ESPELHO",
            Daily::VidroFino => "VIDRO FINO",
            Daily::ChuvaDeCapsulas => "CHUVA DE CÁPSULAS",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Daily::TirosDobrados => "Você e os inimigos atiram o dobro.",
            Daily::InimigosVelozes => "Mergulhos 40% mais rápidos, pontos x2.",
            Daily::SoLaser => "Só existe Laser, mas ele começa no nível 3.",
            Daily::CristaisEmDobro => "Cristais em dobro, loja 50% mais cara.",
            Daily::Neblina => "Você só enxerga perto da nave.",
            Daily::Espelho => "Esquerda vira direita. Boa sorte.",
            Daily::VidroFino => "Uma vida só, mas pontos x3.",
            Daily::ChuvaDeCapsulas => "Cápsulas e power-ups por toda parte.",
        }
    }
}

// ---------------------------------------------------------------------------
// Datas (sem dependências externas)
// ---------------------------------------------------------------------------

/// Dias desde 1970-01-01 (horário local aproximado pelo UTC).
pub fn today_number() -> i64 {
    (miniquad::date::now() / 86_400.0).floor() as i64
}

/// Converte número de dias em (ano, mês, dia).
pub fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

pub fn daily_for(day: i64) -> Daily {
    DAILIES[(day
        .wrapping_mul(2_654_435_761)
        .rem_euclid(DAILIES.len() as i64)) as usize]
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Season {
    None,
    Natal,
    Halloween,
    AnoNovo,
}

pub fn season() -> Season {
    let (_, m, d) = civil(today_number());
    match (m, d) {
        (12, 20..=31) => Season::Natal,
        (1, 1..=2) => Season::AnoNovo,
        (10, 25..=31) | (11, 1..=2) => Season::Halloween,
        _ => Season::None,
    }
}

// ---------------------------------------------------------------------------
// Patentes
// ---------------------------------------------------------------------------

pub const RANKS: [&str; 20] = [
    "RECRUTA",
    "CADETE",
    "ASPIRANTE",
    "PILOTO",
    "PILOTO DE ELITE",
    "SARGENTO",
    "SUBOFICIAL",
    "TENENTE",
    "CAPITÃO",
    "MAJOR",
    "TENENTE-CORONEL",
    "CORONEL",
    "COMODORO",
    "CONTRA-ALMIRANTE",
    "VICE-ALMIRANTE",
    "ALMIRANTE",
    "ALMIRANTE DE ESQUADRA",
    "MARECHAL ESTELAR",
    "HERÓI DA GALÁXIA",
    "LENDA ESTELAR",
];

/// XP total necessário para alcançar a patente `r`.
pub fn xp_for(r: usize) -> u64 {
    if r == 0 {
        0
    } else {
        (250.0 * (r as f64).powf(1.8)).round() as u64
    }
}

pub fn rank_of(xp: u64) -> usize {
    (0..RANKS.len())
        .rev()
        .find(|&r| xp >= xp_for(r))
        .unwrap_or(0)
}

/// O que cada patente desbloqueia (texto mostrado na tela de promoção).
pub fn unlock_text(r: usize) -> &'static str {
    match r {
        1 => "Modo CLÁSSICO",
        2 => "Arma ONDA",
        3 => "Nave FALCÃO e DESAFIO DIÁRIO",
        4 => "Arma ESTILHAÇO e modo SOBREVIVÊNCIA",
        5 => "Arma DISCO e dificuldade DIFÍCIL",
        6 => "Nave BASTIÃO e arma RICOCHETE",
        7 => "Modo CHEFÕES e arma CRIO",
        8 => "Arma TROVÃO",
        9 => "Nave VEGA e arma GRANADA",
        10 => "Arma CHAMA",
        11 => "Itens raros na loja e arma COLMEIA",
        12 => "Nave FANTASMA",
        13 => "Pintura RUBI e arma ÓRBITA",
        14 => "Pintura ESMERALDA",
        15 => "Nave COLOSSO",
        16 => "Pintura OURO e arma BURACO NEGRO",
        17 => "Pintura SOMBRA",
        18 => "Pintura GELO",
        19 => "Pintura LENDÁRIA e nome dourado no ranking",
        _ => "",
    }
}

/// Patente necessária para cada pintura (skin).
pub fn skin_rank(skin: usize) -> usize {
    [0, 13, 14, 16, 17, 18, 19][skin.min(6)]
}

pub const SKIN_NAMES: [&str; 7] = [
    "ORIGINAL",
    "RUBI",
    "ESMERALDA",
    "OURO",
    "SOMBRA",
    "GELO",
    "LENDÁRIA",
];

/// Desenha a insígnia da patente centrada em (x, y).
pub fn draw_insignia(r: usize, x: f32, y: f32, scale: f32) {
    let gold = Color::from_rgba(255, 205, 80, 255);
    let silver = Color::from_rgba(200, 210, 230, 255);
    let dark = Color::from_rgba(20, 16, 40, 255);
    let s = scale;
    draw_circle(x, y, 15.0 * s, dark);
    draw_circle_lines(x, y, 15.0 * s, 2.0, if r >= 10 { gold } else { silver });
    match r {
        0..=4 => {
            // divisas (chevrons)
            let n = r + 1;
            for i in 0..n {
                let yy = y + 5.0 * s - i as f32 * 4.0 * s;
                draw_line(x - 8.0 * s, yy, x, yy - 4.0 * s, 2.0 * s, silver);
                draw_line(x, yy - 4.0 * s, x + 8.0 * s, yy, 2.0 * s, silver);
            }
        }
        5..=9 => {
            // barras
            let n = r - 4;
            for i in 0..n {
                let yy = y - 8.0 * s + i as f32 * 4.0 * s;
                draw_rectangle(x - 7.0 * s, yy, 14.0 * s, 2.5 * s, silver);
            }
        }
        _ => {
            // estrelas
            let n = (r - 9).min(10);
            let per_row = 5.min(n);
            for i in 0..n {
                let row = i / 5;
                let col = i % 5;
                let cols = if row == 0 { per_row } else { n - 5 };
                let xx = x + (col as f32 - (cols as f32 - 1.0) / 2.0) * 5.5 * s;
                let yy = y - 3.0 * s + row as f32 * 7.0 * s;
                draw_poly(xx, yy, 5, 2.6 * s, -90.0, gold);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Conquistas
// ---------------------------------------------------------------------------

pub struct AchDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub hidden: bool,
    pub xp: u64,
}

const fn a(name: &'static str, desc: &'static str, xp: u64) -> AchDef {
    AchDef {
        name,
        desc,
        hidden: false,
        xp,
    }
}
const fn h(name: &'static str, desc: &'static str, xp: u64) -> AchDef {
    AchDef {
        name,
        desc,
        hidden: true,
        xp,
    }
}

pub const ACHS: &[AchDef] = &[
    a("Primeiro Voo", "Termine a primeira onda.", 50),
    a("Regicida", "Derrote a Rainha da Colmeia.", 150),
    a("Ferro-Velho", "Derrote o Couraçado Zênite.", 200),
    a("Encantador de Serpentes", "Derrote a Serpente Cometa.", 250),
    a("Aracnofobia", "Derrote a Tecelã Abissal.", 300),
    a("Não Pisque", "Derrote o Olho do Vazio.", 350),
    a("Coração Partido", "Termine a campanha.", 1000),
    h(
        "Final Verdadeiro",
        "Termine a campanha no Difícil sem usar continues.",
        2500,
    ),
    h("Pesadelo Desperto", "Termine a campanha no Pesadelo.", 5000),
    a(
        "Perfeição",
        "Acerte todos os inimigos de um estágio bônus.",
        200,
    ),
    h("Pacifista", "Termine um estágio bônus sem disparar.", 300),
    a("Combo 50", "Faça um combo de 50.", 150),
    a("Combo 100", "Faça um combo de 100.", 400),
    a("Na Raspa", "Faça 100 raspões numa partida.", 300),
    a("Intocável", "Termine um setor sem ser atingido.", 400),
    a(
        "Desmonte",
        "Destrua todas as partes de um chefe antes do corpo.",
        300,
    ),
    a("Milionário", "Faça 1.000.000 de pontos numa partida.", 1500),
    a("Tio Patinhas", "Tenha 300 cristais ao mesmo tempo.", 250),
    a("Colecionador", "Encontre 20 cartas diferentes.", 500),
    a(
        "Completista",
        "Encontre todas as cartas comuns, raras, épicas e lendárias.",
        2000,
    ),
    h("Ouro de Tolo", "Abata um Escaravelho Dourado.", 200),
    h("Muuu!", "Abata a Vaca Cósmica.", 300),
    h(
        "Bom Samaritano",
        "Deixe o Mercador passar em paz três vezes.",
        300,
    ),
    h("Pirata Espacial", "Ataque o Mercador.", 150),
    h(
        "Saudosista",
        "Digite o código Konami na tela de título.",
        500,
    ),
    h("Viajante Dimensional", "Encontre a Dimensão Dourada.", 800),
    a(
        "Mestre de Armas",
        "Leve 5 armas diferentes ao nível 5.",
        600,
    ),
    a("Hangar Cheio", "Desbloqueie as 7 naves clássicas.", 1000),
    a(
        "Sobrevivente",
        "Chegue à onda 30 no modo Sobrevivência.",
        800,
    ),
    a(
        "Velocista",
        "Vença o modo Chefões em menos de 12 minutos.",
        1000,
    ),
    a("Fliperama", "Faça 60.000 pontos no modo Clássico.", 600),
    a("Rotina", "Termine 3 desafios diários.", 500),
    a("Anti-Kamikaze", "Abata 150 Kamikazes no total.", 300),
    a("Bombardeiro", "Use 50 bombas no total.", 250),
    h(
        "Disco de Ouro",
        "Duas sondas levam um disco de ouro. Diga o nome delas no ranking.",
        700,
    ),
    h("Criador", "Quem fez este jogo?", 300),
    a("Insistente", "Jogue 25 partidas.", 400),
    a("Maratona", "Jogue por 2 horas no total.", 800),
    a("Piromaníaco", "Abata 300 inimigos com a Chama.", 400),
    a("Sem Arranhões", "Derrote um chefe sem ser atingido.", 500),
    h(
        "Curioso",
        "Algo acontece se você digitar o nome do jogo no título.",
        150,
    ),
    h("Insone", "Deixe o jogo rodar sozinho na demonstração.", 100),
    a("Febre!", "Entre em Febre com um combo alto.", 200),
    a("Febre Tripla", "Entre em Febre 3 vezes numa partida.", 500),
    a("Contratado", "Complete um contrato.", 150),
    a(
        "Mercenário de Elite",
        "Complete 30 contratos no total.",
        900,
    ),
    a("Arqueólogo", "Encontre 6 registros de bordo.", 500),
    a(
        "Historiador",
        "Encontre todos os 12 registros de bordo.",
        1500,
    ),
    a("Gêmeas Separadas", "Derrote as Gêmeas.", 400),
    h("O Vazio Olha de Volta", "Derrote o Devorador.", 5000),
    a(
        "Pega Ladrão",
        "Abata um Ladrão carregando o que roubou.",
        250,
    ),
    a("Desconfiado", "Abata 10 Mímicos no total.", 300),
    a(
        "Elite da Elite",
        "Abata 50 inimigos de elite no total.",
        600,
    ),
    a("Era do Gelo", "Congele 150 inimigos no total.", 400),
    a(
        "Horizonte",
        "Engula 300 tiros com buracos negros no total.",
        400,
    ),
    a(
        "Arsenal Duplo",
        "Leve duas armas diferentes ao nível 5 na mesma partida.",
        400,
    ),
    a(
        "Mestre Supremo",
        "Leve 12 armas diferentes ao nível 5.",
        1500,
    ),
    a("Eclipse Total", "Termine um setor durante um Eclipse.", 400),
    h("Fazendeiro", "Abata a Vaca Cósmica 3 vezes.", 600),
    a("Fliperamista", "Vença 10 minigames das Fendas.", 600),
    h("Relicário", "Encontre uma Relíquia.", 1000),
    a("Turista Cósmico", "Viva os 6 eventos de setor.", 700),
    a("Exorcista", "Destrua 25 maldições a tiros.", 500),
    h("Azarado", "Pegue 10 maldições. De propósito?", 200),
    h("Caçador de Estrelas", "Encontre uma Sala Secreta.", 800),
    a("Perfeccionista", "Consiga PERFEITO em 5 minigames.", 700),
    h("Guardião das Relíquias", "Encontre as 4 Relíquias.", 4000),
    a("Expedicionário", "Chegue ao setor 10 da Expedição.", 1200),
    h("Tesoureiro", "Abra 5 relicários secretos.", 900),
    a("Frota Completa", "Desbloqueie as 10 naves.", 1500),
    a("Caçador de Cometas", "Derrote o Cometa Wild.", 400),
    a("Soletrador", "Junte as letras S-T-A-R-D-U-S-T.", 800),
    h("Contatos Imediatos", "Derrube a Nave-Mãe misteriosa.", 300),
    h("Atalho", "Use um Portal de Salto.", 600),
    a(
        "Onda Relâmpago",
        "Limpe uma onda em menos de 7 segundos.",
        400,
    ),
    a(
        "Tiro de Canhão",
        "Abata 5 inimigos com um só tiro carregado.",
        500,
    ),
];

/// Registros de bordo: a história por trás do Enxame.
pub const LOGS: [(&str, &str); 12] = [
    (
        "REGISTRO 1 - A RAINHA",
        "A poeira da sonda humana chamou por nós. Ordenei a migração. Não há tempo para explicar aos nativos.",
    ),
    (
        "REGISTRO 2 - O COURAÇADO",
        "Tomamos a nave dos humanos. Precisávamos de casco e de canhões. Algo nos segue há três ciclos.",
    ),
    (
        "REGISTRO 3 - A SERPENTE",
        "A Serpente era a guardiã do caminho. Ela ouviu primeiro: um silêncio que engole estrelas.",
    ),
    (
        "REGISTRO 4 - A TECELÃ",
        "Tecemos redes entre as luas para atrasar a coisa. As redes sumiram. Não rasgaram: sumiram.",
    ),
    (
        "REGISTRO 5 - O OLHO",
        "O Olho foi feito para vigiar o escuro. Ele viu. Desde então não consegue mais fechar direito.",
    ),
    (
        "REGISTRO 6 - O ÔMEGA",
        "Reforçamos o Couraçado. Se o piloto humano chegar até aqui, talvez seja forte o bastante.",
    ),
    (
        "REGISTRO 7 - AS GÊMEAS",
        "Minhas filhas lutam juntas. Disse a elas para não odiarem o piloto. Ele só não sabe.",
    ),
    (
        "REGISTRO 8 - O CORAÇÃO",
        "Se eu cair, a colmeia se espalha e sobrevive. Se ele cair, ninguém mais vai segurar a porta.",
    ),
    (
        "REGISTRO 9 - DIMENSÃO DOURADA",
        "Escondemos o tesouro da colmeia entre dimensões. Só quem nunca cai consegue achar a porta.",
    ),
    (
        "REGISTRO 10 - A VACA",
        "A Vaca não é nossa. Ela apareceu um dia e começou a nos seguir. Desde então chamamos ela de sorte.",
    ),
    (
        "REGISTRO 11 - O MERCADOR",
        "O Mercador vende para todos os lados. Ele sabe do Devorador. Ele sempre soube.",
    ),
    (
        "REGISTRO 12 - DEPOIS",
        "Obrigada, piloto. O Devorador recuou. A colmeia vai sobreviver. E agora você também sabe.",
    ),
];

// ---------------------------------------------------------------------------
// Insígnias de Honra: patentes que não vêm de XP, e sim de feitos.
// ---------------------------------------------------------------------------

pub struct Honor {
    pub name: &'static str,
    pub feat: &'static str,
    pub reward: &'static str,
    pub short: &'static str,
    pub ach: usize,
}

pub const HONORS: [Honor; 10] = [
    Honor {
        name: "INSÍGNIA DO INTOCÁVEL",
        feat: "Termine um setor sem ser atingido.",
        reward: "Rastro ARCO-ÍRIS",
        short: "ARCO-ÍRIS",
        ach: ach::INTOCAVEL,
    },
    Honor {
        name: "INSÍGNIA DO EXORCISTA",
        feat: "Destrua 25 maldições a tiros.",
        reward: "Rastro FOGO",
        short: "FOGO",
        ach: ach::EXORCISTA,
    },
    Honor {
        name: "INSÍGNIA DO FLIPERAMA",
        feat: "Vença 10 minigames das Fendas.",
        reward: "Nave CORSÁRIA",
        short: "CORSÁRIA",
        ach: ach::FLIPERAMISTA,
    },
    Honor {
        name: "INSÍGNIA DO EXPLORADOR",
        feat: "Encontre uma Sala Secreta.",
        reward: "Nave SONDA e rastro POEIRA ESTELAR",
        short: "SONDA",
        ach: ach::SALA_SECRETA,
    },
    Honor {
        name: "INSÍGNIA DO PERFECCIONISTA",
        feat: "Consiga PERFEITO em 5 minigames.",
        reward: "Nave ÉGIDE",
        short: "ÉGIDE",
        ach: ach::PERFECCIONISTA,
    },
    Honor {
        name: "INSÍGNIA DO ARSENAL",
        feat: "Leve 5 armas ao nível 5.",
        reward: "Rastro GELO",
        short: "GELO",
        ach: ach::MESTRE_ARMAS,
    },
    Honor {
        name: "INSÍGNIA DA RELÍQUIA",
        feat: "Encontre uma Relíquia.",
        reward: "Rastro OURO",
        short: "OURO",
        ach: ach::RELICARIO,
    },
    Honor {
        name: "INSÍGNIA DA EXPEDIÇÃO",
        feat: "Chegue ao setor 10 da Expedição.",
        reward: "Rastro PLASMA",
        short: "PLASMA",
        ach: ach::EXPEDICIONARIO,
    },
    Honor {
        name: "INSÍGNIA DO VAZIO",
        feat: "Derrote o Devorador.",
        reward: "Rastro VAZIO",
        short: "VAZIO",
        ach: ach::DEVORADOR,
    },
    Honor {
        name: "INSÍGNIA SUPREMA",
        feat: "Encontre as 4 Relíquias.",
        reward: "Pintura LENDÁRIA para todas as naves",
        short: "LENDÁRIA",
        ach: ach::RELIQUIAS,
    },
];

impl Honor {
    /// Insígnias ligadas a conquistas escondidas não revelam nada até serem ganhas.
    pub fn masked(&self, achs: &[bool]) -> bool {
        ACHS[self.ach].hidden && !achs.get(self.ach).copied().unwrap_or(false)
    }
    pub fn shown_name(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) {
            "INSÍGNIA ???"
        } else {
            self.name
        }
    }
    pub fn shown_short(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) { "???" } else { self.short }
    }
    pub fn shown_reward(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) {
            "???"
        } else {
            self.reward
        }
    }
    pub fn shown_feat(&self, achs: &[bool]) -> &'static str {
        if self.masked(achs) { "???" } else { self.feat }
    }
}

pub fn honors_earned(achs: &[bool]) -> usize {
    HONORS
        .iter()
        .filter(|h| achs.get(h.ach).copied().unwrap_or(false))
        .count()
}

pub const TRAILS: [&str; 8] = [
    "SEM RASTRO",
    "ARCO-ÍRIS",
    "FOGO",
    "POEIRA ESTELAR",
    "VAZIO",
    "GELO",
    "OURO",
    "PLASMA",
];

/// Conquista que libera cada rastro (o 0 é sempre livre).
pub fn trail_ach(i: usize) -> Option<usize> {
    match i {
        1 => Some(ach::INTOCAVEL),
        2 => Some(ach::EXORCISTA),
        3 => Some(ach::SALA_SECRETA),
        4 => Some(ach::DEVORADOR),
        5 => Some(ach::MESTRE_ARMAS),
        6 => Some(ach::RELICARIO),
        7 => Some(ach::EXPEDICIONARIO),
        _ => None,
    }
}

pub fn trail_unlocked(i: usize, s: &crate::save::Save) -> bool {
    i < TRAILS.len()
        && trail_ach(i)
            .map(|a| s.achs.get(a).copied().unwrap_or(false))
            .unwrap_or(true)
}

pub mod ach {
    pub const PRIMEIRO_VOO: usize = 0;
    pub const REGICIDA: usize = 1;
    pub const FERRO_VELHO: usize = 2;
    pub const SERPENTES: usize = 3;
    pub const ARACNO: usize = 4;
    pub const NAO_PISQUE: usize = 5;
    pub const CORACAO_PARTIDO: usize = 6;
    pub const FINAL_VERDADEIRO: usize = 7;
    pub const PESADELO: usize = 8;
    pub const PERFEICAO: usize = 9;
    pub const PACIFISTA: usize = 10;
    pub const COMBO50: usize = 11;
    pub const COMBO100: usize = 12;
    pub const RASPA: usize = 13;
    pub const INTOCAVEL: usize = 14;
    pub const DESMONTE: usize = 15;
    pub const MILIONARIO: usize = 16;
    pub const PATINHAS: usize = 17;
    pub const COLECIONADOR: usize = 18;
    pub const COMPLETISTA: usize = 19;
    pub const OURO: usize = 20;
    pub const MUUU: usize = 21;
    pub const SAMARITANO: usize = 22;
    pub const PIRATA: usize = 23;
    pub const KONAMI: usize = 24;
    pub const DIMENSIONAL: usize = 25;
    pub const MESTRE_ARMAS: usize = 26;
    pub const HANGAR: usize = 27;
    pub const SOBREVIVENTE: usize = 28;
    pub const VELOCISTA: usize = 29;
    pub const FLIPERAMA: usize = 30;
    pub const ROTINA: usize = 31;
    pub const ANTI_KAMIKAZE: usize = 32;
    pub const BOMBARDEIRO: usize = 33;
    pub const DISCO_OURO: usize = 34;
    pub const CRIADOR: usize = 35;
    pub const INSISTENTE: usize = 36;
    pub const MARATONA: usize = 37;
    pub const PIROMANIACO: usize = 38;
    pub const SEM_ARRANHOES: usize = 39;
    pub const CURIOSO: usize = 40;
    pub const INSONE: usize = 41;
    pub const FEBRE: usize = 42;
    pub const FEBRE3: usize = 43;
    pub const CONTRATADO: usize = 44;
    pub const MERCENARIO: usize = 45;
    pub const ARQUEOLOGO: usize = 46;
    pub const HISTORIADOR: usize = 47;
    pub const GEMEAS: usize = 48;
    pub const DEVORADOR: usize = 49;
    pub const PEGA_LADRAO: usize = 50;
    pub const DESCONFIADO: usize = 51;
    pub const ELITE: usize = 52;
    pub const GELO: usize = 53;
    pub const HORIZONTE: usize = 54;
    pub const ARSENAL_DUPLO: usize = 55;
    pub const MESTRE_SUPREMO: usize = 56;
    pub const ECLIPSE: usize = 57;
    pub const FAZENDEIRO: usize = 58;
    pub const FLIPERAMISTA: usize = 59;
    pub const RELICARIO: usize = 60;
    pub const TURISTA: usize = 61;
    pub const EXORCISTA: usize = 62;
    pub const AZARADO: usize = 63;
    pub const SALA_SECRETA: usize = 64;
    pub const PERFECCIONISTA: usize = 65;
    pub const RELIQUIAS: usize = 66;
    pub const EXPEDICIONARIO: usize = 67;
    pub const TESOUREIRO: usize = 68;
    pub const FROTA: usize = 69;
    pub const COMETA: usize = 70;
    pub const SOLETRADOR: usize = 71;
    pub const OVNI: usize = 72;
    pub const ATALHO: usize = 73;
    pub const RELAMPAGO: usize = 74;
    pub const CARREGADO: usize = 75;
}

// ---------------------------------------------------------------------------
// Bestiário
// ---------------------------------------------------------------------------

pub struct Lore {
    pub name: &'static str,
    pub text: &'static str,
}

/// Uma entrada por tipo de inimigo (ordem de game::Kind) seguida pelos chefes (ordem de BossId).
pub const BESTIARY: &[Lore] = &[
    Lore {
        name: "ZANGÃO",
        text: "A base do Enxame. Sozinho é inofensivo; em formação, é uma muralha viva.",
    },
    Lore {
        name: "VESPA",
        text: "Rápida e nervosa. Gosta de mergulhar em espiral antes de voltar à formação.",
    },
    Lore {
        name: "BESOURO",
        text: "Comandante blindado. Aguenta dois tiros e sempre leva escolta nos mergulhos.",
    },
    Lore {
        name: "ATIRADOR",
        text: "Carrega dois canhões biológicos. Para no meio da tela para disparar em leque.",
    },
    Lore {
        name: "DIVISOR",
        text: "Quando morre, se divide em dois filhotes famintos que caçam a sua nave.",
    },
    Lore {
        name: "KAMIKAZE",
        text: "Não atira. Não desvia. Só quer encostar em você.",
    },
    Lore {
        name: "FILHOTE",
        text: "Nasce dos Divisores e das Portadoras. Pequeno, rápido e teimoso.",
    },
    Lore {
        name: "ESCUDEIRO",
        text: "Carrega uma placa de quitina na frente. Quebre o escudo antes de feri-lo.",
    },
    Lore {
        name: "TECELÃ",
        text: "Dispara teias pegajosas que deixam sua nave lenta por alguns segundos.",
    },
    Lore {
        name: "PORTADORA",
        text: "Uma creche voadora. Solta filhotes enquanto estiver viva.",
    },
    Lore {
        name: "CAMALEÃO",
        text: "Some e reaparece. Só é atingido quando está visível.",
    },
    Lore {
        name: "ESCARAVELHO DOURADO",
        text: "Raríssimo. Foge rápido, mas carrega uma fortuna em cristais.",
    },
    Lore {
        name: "VACA CÓSMICA",
        text: "Ninguém sabe como ela chegou ao espaço. Ninguém tem coragem de perguntar.",
    },
    Lore {
        name: "MERCADOR",
        text: "Um comerciante neutro. Retribui a gentileza... e nunca esquece uma traição.",
    },
    Lore {
        name: "RAINHA DA COLMEIA",
        text: "Mãe de todo o Enxame. Suas garras disparam sozinhas e ela chama lacaios.",
    },
    Lore {
        name: "COURAÇADO ZÊNITE",
        text: "Uma nave roubada e infestada. O canhão central dispara um laser avisado.",
    },
    Lore {
        name: "SERPENTE COMETA",
        text: "Uma cauda de anéis em brasa. Cada anel destruído enfraquece a fera.",
    },
    Lore {
        name: "TECELÃ ABISSAL",
        text: "Aranha do tamanho de uma lua. Suas teias atravessam a tela inteira.",
    },
    Lore {
        name: "OLHO DO VAZIO",
        text: "Só pode ser ferido com a pálpebra aberta. Seu olhar varre o espaço com raios.",
    },
    Lore {
        name: "RAINHA SUPREMA",
        text: "O coração do Enxame. Coroada, furiosa e com todos os truques das irmãs.",
    },
    Lore {
        name: "MÍMICO",
        text: "Parece uma cápsula. Não é. Quando você chega perto, ele abre a boca.",
    },
    Lore {
        name: "LADRÃO",
        text: "Mascarado e rápido. Rouba o que cai na tela e foge. Derrube-o para recuperar em dobro.",
    },
    Lore {
        name: "CARGUEIRO",
        text: "Nave de carga do Enxame. Não atira e está lotada de cristais.",
    },
    Lore {
        name: "AS GÊMEAS",
        text: "Duas princesas da colmeia que lutam juntas. Quando uma cai, a outra enlouquece.",
    },
    Lore {
        name: "O DEVORADOR",
        text: "Aquilo de que o Enxame fugia. Uma fome do tamanho de uma galáxia.",
    },
    Lore {
        name: "COMETA WILD",
        text: "Um cometa que o Enxame domou. Sua cauda solta poeira estelar que fere.",
    },
    Lore {
        name: "MINADOR",
        text: "Desce em mergulho largando minas que explodem em leque. Atire nas minas!",
    },
    Lore {
        name: "SALTADOR",
        text: "Nunca fica no mesmo lugar da formação. Some num piscar e reaparece longe.",
    },
];

pub const LORE_BOSS_START: usize = 14;
/// Novos registros da v3 (depois dos chefes originais, para manter saves antigos).
pub const LORE_V3: usize = 20;
pub const LORE_TWINS: usize = 23;
pub const LORE_DEVOURER: usize = 24;
pub const LORE_COMET: usize = 25;
/// Inimigos da v5 (Minador e Saltador), depois do Cometa.
pub const LORE_V5: usize = 26;
pub const KIND_V5: usize = 17;

/// Índice do bestiário para um tipo de inimigo (game::Kind as usize).
pub fn lore_of_kind(kind: usize) -> usize {
    if kind < LORE_BOSS_START {
        kind
    } else if kind >= KIND_V5 {
        LORE_V5 + kind - KIND_V5
    } else {
        LORE_V3 + kind - LORE_BOSS_START
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patentes_crescem() {
        for r in 1..RANKS.len() {
            assert!(xp_for(r) > xp_for(r - 1));
            assert_eq!(rank_of(xp_for(r)), r);
            assert_eq!(rank_of(xp_for(r) - 1), r - 1);
        }
    }

    #[test]
    fn datas() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(20_454), (2026, 1, 1));
        assert_eq!(civil(20_760), (2026, 11, 3));
    }

    #[test]
    fn conquistas_tem_nomes_unicos() {
        let mut names: Vec<&str> = ACHS.iter().map(|a| a.name).collect();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), ACHS.len());
        assert_eq!(ACHS[ach::INSONE].name, "Insone");
        assert_eq!(ACHS[ach::TURISTA].name, "Turista Cósmico");
        assert_eq!(ACHS.len(), 76);
        assert_eq!(ACHS[ach::COMETA].name, "Caçador de Cometas");
    }
}
