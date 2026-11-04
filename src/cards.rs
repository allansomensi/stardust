//! Cartas colecionáveis: bônus passivos que você equipa antes de cada partida.

use crate::sprites::Icon;
use macroquad::prelude::Color;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        match self {
            Rarity::Common => "COMUM",
            Rarity::Rare => "RARA",
            Rarity::Epic => "ÉPICA",
            Rarity::Legendary => "LENDÁRIA",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Rarity::Common => Color::from_rgba(170, 180, 200, 255),
            Rarity::Rare => Color::from_rgba(80, 160, 255, 255),
            Rarity::Epic => Color::from_rgba(200, 100, 255, 255),
            Rarity::Legendary => Color::from_rgba(255, 200, 50, 255),
        }
    }
}

pub struct CardDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub rarity: Rarity,
    pub icon: Icon,
    /// Cartas secretas não caem por sorteio: só por segredos.
    pub secret: bool,
}

const fn c(name: &'static str, desc: &'static str, rarity: Rarity, icon: Icon) -> CardDef {
    CardDef {
        name,
        desc,
        rarity,
        icon,
        secret: false,
    }
}
const fn s(name: &'static str, desc: &'static str, rarity: Rarity, icon: Icon) -> CardDef {
    CardDef {
        name,
        desc,
        rarity,
        icon,
        secret: true,
    }
}

use Icon::*;
use Rarity::*;

pub const CARDS: &[CardDef] = &[
    c(
        "Casco Reforçado",
        "+1 vida no início da partida.",
        Common,
        Heart,
    ),
    c(
        "Pólvora Extra",
        "+1 bomba no início da partida.",
        Common,
        Bomb,
    ),
    c(
        "Ímã Portátil",
        "Raio do ímã de cristais +60%.",
        Common,
        Magnet,
    ),
    c("Motor Turbo", "Nave 12% mais rápida.", Common, Engine),
    c("Gatilho Leve", "Cadência de tiro +10%.", Common, Bolt),
    c(
        "Cristal da Sorte",
        "Cristais valem 25% a mais.",
        Common,
        Crystal,
    ),
    c("Começo Forte", "Sua arma começa no nível 2.", Common, Star),
    c("Cofrinho", "Começa com 40 cristais.", Common, Bag),
    c(
        "Escudo Inicial",
        "Começa com um escudo que absorve um golpe.",
        Rare,
        Shield,
    ),
    c("Combo Longo", "Janela do combo 50% maior.", Rare, Clock),
    c(
        "Caçador",
        "Inimigos em mergulho valem o triplo.",
        Rare,
        Target,
    ),
    c(
        "Pescador de Cartas",
        "Cartas caem com o dobro de frequência.",
        Rare,
        Card,
    ),
    c("Freguês", "Tudo na loja custa 20% menos.", Rare, Bag),
    c("Recarga", "Ganha 1 bomba a cada setor.", Rare, Bomb),
    c(
        "Fênix",
        "Uma vez por partida, renasce ao perder a última vida.",
        Epic,
        Flame,
    ),
    c("Drones", "Dois drones atiram ao seu lado.", Epic, Drone),
    c(
        "Arsenal Completo",
        "Sua arma começa no nível 3.",
        Epic,
        Star,
    ),
    c(
        "Bomba Relâmpago",
        "Bombas causam o dobro de dano.",
        Epic,
        Bolt,
    ),
    c(
        "Último Suspiro",
        "Ao ser atingido, solta uma bomba de graça.",
        Epic,
        Bomb,
    ),
    c(
        "Coração do Enxame",
        "+1 vida e escudo novo a cada setor.",
        Legendary,
        Heart,
    ),
    c(
        "Tempo Lento",
        "Tiros inimigos 20% mais lentos.",
        Legendary,
        Clock,
    ),
    c(
        "Multiplicador Dourado",
        "Pontos x1,5 e combo máximo x10.",
        Legendary,
        Star,
    ),
    c(
        "Constelação",
        "A cada 6 disparos, um leque de 6 estrelas.",
        Legendary,
        Disc,
    ),
    c(
        "Asas de Cristal",
        "Cristais vêm até você de qualquer lugar.",
        Legendary,
        Crystal,
    ),
    c(
        "Raspão Perigoso",
        "Raspões valem o dobro de pontos e de energia.",
        Rare,
        Target,
    ),
    c(
        "Bateria Extra",
        "Começa com a habilidade carregada.",
        Common,
        Bolt,
    ),
    c(
        "Sobrecarga Rápida",
        "Habilidade carrega 30% mais rápido.",
        Rare,
        Engine,
    ),
    c(
        "Escolta",
        "Um drone de escolta acompanha você.",
        Epic,
        Drone,
    ),
    c(
        "Arsenal Variado",
        "Cápsulas alternam entre 4 armas e duram mais.",
        Common,
        Star,
    ),
    c(
        "Pontaria",
        "Seus tiros voam 15% mais rápido.",
        Common,
        Target,
    ),
    c(
        "Herança",
        "Perder uma nave não reduz o nível da arma.",
        Rare,
        Shield,
    ),
    c(
        "Cristal Vivo",
        "A cada 100 cristais coletados, +1 bomba.",
        Epic,
        Crystal,
    ),
    c(
        "Investidor",
        "A primeira troca de ofertas na loja é grátis.",
        Rare,
        Bag,
    ),
    c(
        "Contra-Ataque",
        "Ao ser atingido, todos os inimigos levam dano.",
        Epic,
        Flame,
    ),
    c("Ímã de Cartas", "Cartas voam até você.", Legendary, Magnet),
    c("Sortudo", "Power-ups caem 50% mais.", Common, Star),
    s(
        "Coroa da Rainha",
        "Começa cada setor em Frenesi.",
        Legendary,
        Star,
    ),
    s(
        "Vaca Cósmica",
        "+1 vida e +1 bomba. Muuu.",
        Legendary,
        Heart,
    ),
    s(
        "Código Konami",
        "+3 vidas. Você sabe o que fez.",
        Legendary,
        Heart,
    ),
    s("Criador", "Cristais x1,5 e pontos x1,2.", Legendary, Disc),
    s(
        "Dimensão Dourada",
        "Um Escaravelho Dourado aparece em todo setor.",
        Legendary,
        Crystal,
    ),
    s("Pacifista", "Estágios bônus valem o dobro.", Epic, Shield),
    c(
        "Bolso Extra",
        "Começa a partida com o LASER no nível 2.",
        Common,
        Star,
    ),
    c(
        "Contrato Lucrativo",
        "Recompensas de contratos em dobro.",
        Rare,
        Bag,
    ),
    c(
        "Febre Precoce",
        "A Febre começa com combo 35 em vez de 50.",
        Rare,
        Flame,
    ),
    c("Febre Longa", "A Febre dura 50% mais.", Common, Clock),
    c(
        "Ricochete Afiado",
        "Tiros que quicam ganham mais 2 quiques.",
        Common,
        Bolt,
    ),
    c("Estilhaço Térmico", "Explosões 40% maiores.", Rare, Bomb),
    c(
        "Zero Absoluto",
        "Inimigos ficam congelados o dobro do tempo.",
        Epic,
        Shield,
    ),
    c(
        "Rainha das Abelhas",
        "A Colmeia solta o dobro de abelhas.",
        Epic,
        Star,
    ),
    c(
        "Horizonte de Eventos",
        "Buracos negros duram o dobro e sugam cristais.",
        Legendary,
        Crystal,
    ),
    c(
        "Caçador de Elite",
        "Elites sempre soltam cápsula e às vezes uma carta.",
        Rare,
        Target,
    ),
    c("Mão Leve", "Ladrões ficam 40% mais lentos.", Common, Bag),
    c(
        "Olho Clínico",
        "Mímicos aparecem marcados em vermelho.",
        Common,
        Target,
    ),
    c("Saudade", "+1 continue por partida.", Common, Heart),
    c(
        "Escudo Reativo",
        "O escudo volta depois de 25 s sem levar dano.",
        Epic,
        Shield,
    ),
    c(
        "Ímã de Cápsulas",
        "Cápsulas e poderes vêm até você.",
        Rare,
        Magnet,
    ),
    s(
        "Bezerro Cósmico",
        "Um bezerro te acompanha e recolhe cristais. Muuu.",
        Legendary,
        Heart,
    ),
    s(
        "Disco de Ouro",
        "A PULSAR cai das cápsulas e as estrelas valem o dobro.",
        Epic,
        Disc,
    ),
    s(
        "Olho do Devorador",
        "Pontos x1,3 e tiros inimigos 10% mais lentos.",
        Legendary,
        Target,
    ),
    c(
        "Amuleto Antigo",
        "Metade das maldições não te pega.",
        Rare,
        Shield,
    ),
    c(
        "Fenda Aberta",
        "Fendas de minigame caem 2x mais.",
        Rare,
        Target,
    ),
    c("Sal Grosso", "Maldições caem 50% menos.", Common, Crystal),
    c(
        "Poeira de Cometa",
        "Cristais +15% e ímã 20% maior.",
        Epic,
        Star,
    ),
    s(
        "Sinal de Contato",
        "A Nave-Mãe misteriosa aparece com o dobro de frequência.",
        Epic,
        Target,
    ),
];

/// Índices das cartas (mesma ordem de CARDS).
pub mod id {
    pub const CASCO: usize = 0;
    pub const POLVORA: usize = 1;
    pub const IMA: usize = 2;
    pub const MOTOR: usize = 3;
    pub const GATILHO: usize = 4;
    pub const SORTE: usize = 5;
    pub const COMECO: usize = 6;
    pub const COFRE: usize = 7;
    pub const ESCUDO: usize = 8;
    pub const COMBO: usize = 9;
    pub const CACADOR: usize = 10;
    pub const PESCADOR: usize = 11;
    pub const FREGUES: usize = 12;
    pub const RECARGA: usize = 13;
    pub const FENIX: usize = 14;
    pub const DRONES: usize = 15;
    pub const ARSENAL: usize = 16;
    pub const RELAMPAGO: usize = 17;
    pub const SUSPIRO: usize = 18;
    pub const CORACAO: usize = 19;
    pub const LENTO: usize = 20;
    pub const DOURADO: usize = 21;
    pub const CONSTELACAO: usize = 22;
    pub const ASAS: usize = 23;
    pub const RASPAO: usize = 24;
    pub const BATERIA: usize = 25;
    pub const SOBRECARGA: usize = 26;
    pub const ESCOLTA: usize = 27;
    pub const VARIADO: usize = 28;
    pub const PONTARIA: usize = 29;
    pub const HERANCA: usize = 30;
    pub const CRISTAL_VIVO: usize = 31;
    pub const INVESTIDOR: usize = 32;
    pub const CONTRA: usize = 33;
    pub const IMA_CARTAS: usize = 34;
    pub const SORTUDO: usize = 35;
    pub const COROA: usize = 36;
    pub const VACA: usize = 37;
    pub const KONAMI: usize = 38;
    pub const CRIADOR: usize = 39;
    pub const DIMENSAO: usize = 40;
    pub const PACIFISTA: usize = 41;
    pub const BOLSO: usize = 42;
    pub const LUCRATIVO: usize = 43;
    pub const FEBRE_PRECOCE: usize = 44;
    pub const FEBRE_LONGA: usize = 45;
    pub const RICOCHETE: usize = 46;
    pub const TERMICO: usize = 47;
    pub const ZERO: usize = 48;
    pub const ABELHAS: usize = 49;
    pub const HORIZONTE: usize = 50;
    pub const ELITE: usize = 51;
    pub const MAO_LEVE: usize = 52;
    pub const OLHO_CLINICO: usize = 53;
    pub const SAUDADE: usize = 54;
    pub const REATIVO: usize = 55;
    pub const IMA_CAPSULAS: usize = 56;
    pub const BEZERRO: usize = 57;
    pub const DISCO_OURO: usize = 58;
    pub const DEVORADOR: usize = 59;
    pub const AMULETO: usize = 60;
    pub const FENDA: usize = 61;
    pub const SAL: usize = 62;
    pub const POEIRA: usize = 63;
    pub const CONTATO: usize = 64;
}

/// Sorteia uma carta (nunca secreta). `luck` 0 = normal, maior = puxa para raridades altas.
pub fn roll(luck: u32) -> usize {
    use macroquad::rand::gen_range;
    let mut r = gen_range(0.0f32, 100.0);
    r += luck as f32 * 25.0;
    let rarity = if r >= 112.0 {
        Legendary
    } else if r >= 92.0 {
        Epic
    } else if r >= 65.0 {
        Rare
    } else {
        Common
    };
    let pool: Vec<usize> = CARDS
        .iter()
        .enumerate()
        .filter(|(_, c)| c.rarity == rarity && !c.secret)
        .map(|(i, _)| i)
        .collect();
    pool[gen_range(0, pool.len())]
}

/// Número de cartas que podem ser equipadas, conforme a coleção cresce.
pub fn slots_for(unique: usize) -> usize {
    match unique {
        45.. => 5,
        30.. => 4,
        16.. => 3,
        8.. => 2,
        _ => 1,
    }
}

#[derive(Default, Clone)]
pub struct Mods {
    pub ufo_more: bool,
    pub curse_mul: f32,
    pub rift_mul: f32,
    pub amulet: bool,
    pub extra_lives: i32,
    pub extra_bombs: i32,
    pub magnet_mul: f32,
    pub speed_mul: f32,
    pub fire_mul: f32,
    pub crystal_mul: f32,
    pub start_level: i32,
    pub start_crystals: u32,
    pub start_shield: bool,
    pub combo_mul: f32,
    pub dive_mul: u32,
    pub card_mul: f32,
    pub shop_mul: f32,
    pub bomb_per_sector: bool,
    pub phoenix: bool,
    pub drones: u32,
    pub bomb_power: f32,
    pub last_breath: bool,
    pub shield_per_sector: bool,
    pub bullet_speed: f32,
    pub score_mul: f32,
    pub max_combo: u32,
    pub star_fan: bool,
    pub magnet_all: bool,
    pub graze_mul: f32,
    pub meter_full: bool,
    pub meter_mul: f32,
    pub capsule_four: bool,
    pub shot_speed: f32,
    pub keep_level: bool,
    pub crystal_bomb: bool,
    pub free_reroll: bool,
    pub counter: bool,
    pub card_magnet: bool,
    pub powerup_mul: f32,
    pub crown: bool,
    pub moo: bool,
    pub golden_every: bool,
    pub bonus_mul: u32,
    pub second_laser: bool,
    pub contract_mul: u32,
    pub fever_start: u32,
    pub fever_mul: f32,
    pub bounce_extra: f32,
    pub blast_mul: f32,
    pub freeze_mul: f32,
    pub bee_mul: f32,
    pub hole_mul: f32,
    pub hole_magnet: bool,
    pub elite_drop: bool,
    pub thief_slow: bool,
    pub mimic_mark: bool,
    pub extra_continue: u32,
    pub regen_shield: bool,
    pub capsule_magnet: bool,
    pub calf: bool,
    pub pulsar_card: bool,
}

impl Mods {
    pub fn from_cards(equipped: &[usize]) -> Mods {
        use id::*;
        let mut m = Mods {
            magnet_mul: 1.0,
            speed_mul: 1.0,
            fire_mul: 1.0,
            crystal_mul: 1.0,
            start_level: 1,
            combo_mul: 1.0,
            dive_mul: 2,
            card_mul: 1.0,
            shop_mul: 1.0,
            bomb_power: 1.0,
            bullet_speed: 1.0,
            score_mul: 1.0,
            max_combo: 5,
            graze_mul: 1.0,
            meter_mul: 1.0,
            shot_speed: 1.0,
            powerup_mul: 1.0,
            bonus_mul: 1,
            contract_mul: 1,
            fever_start: 50,
            fever_mul: 1.0,
            blast_mul: 1.0,
            freeze_mul: 1.0,
            bee_mul: 1.0,
            hole_mul: 1.0,
            curse_mul: 1.0,
            rift_mul: 1.0,
            ..Default::default()
        };
        for &c in equipped {
            match c {
                CASCO => m.extra_lives += 1,
                POLVORA => m.extra_bombs += 1,
                IMA => m.magnet_mul = 1.6,
                MOTOR => m.speed_mul = 1.12,
                GATILHO => m.fire_mul = 1.1,
                SORTE => m.crystal_mul *= 1.25,
                COMECO => m.start_level = m.start_level.max(2),
                COFRE => m.start_crystals += 40,
                ESCUDO => m.start_shield = true,
                COMBO => m.combo_mul = 1.5,
                CACADOR => m.dive_mul = 3,
                PESCADOR => m.card_mul = 2.0,
                FREGUES => m.shop_mul = 0.8,
                RECARGA => m.bomb_per_sector = true,
                FENIX => m.phoenix = true,
                DRONES => m.drones += 2,
                ARSENAL => m.start_level = 3,
                RELAMPAGO => m.bomb_power = 2.0,
                SUSPIRO => m.last_breath = true,
                CORACAO => {
                    m.extra_lives += 1;
                    m.shield_per_sector = true;
                }
                LENTO => m.bullet_speed = 0.8,
                DOURADO => {
                    m.score_mul *= 1.5;
                    m.max_combo = 10;
                }
                CONSTELACAO => m.star_fan = true,
                ASAS => m.magnet_all = true,
                RASPAO => m.graze_mul = 2.0,
                BATERIA => m.meter_full = true,
                SOBRECARGA => m.meter_mul = 1.3,
                ESCOLTA => m.drones += 1,
                VARIADO => m.capsule_four = true,
                PONTARIA => m.shot_speed = 1.15,
                HERANCA => m.keep_level = true,
                CRISTAL_VIVO => m.crystal_bomb = true,
                INVESTIDOR => m.free_reroll = true,
                CONTRA => m.counter = true,
                IMA_CARTAS => m.card_magnet = true,
                SORTUDO => m.powerup_mul = 1.5,
                COROA => m.crown = true,
                VACA => {
                    m.extra_lives += 1;
                    m.extra_bombs += 1;
                    m.moo = true;
                }
                KONAMI => m.extra_lives += 3,
                CRIADOR => {
                    m.crystal_mul *= 1.5;
                    m.score_mul *= 1.2;
                }
                DIMENSAO => m.golden_every = true,
                PACIFISTA => m.bonus_mul = 2,
                BOLSO => m.second_laser = true,
                LUCRATIVO => m.contract_mul = 2,
                FEBRE_PRECOCE => m.fever_start = 35,
                FEBRE_LONGA => m.fever_mul = 1.5,
                RICOCHETE => m.bounce_extra = 2.0,
                TERMICO => m.blast_mul = 1.4,
                ZERO => m.freeze_mul = 2.0,
                ABELHAS => m.bee_mul = 2.0,
                HORIZONTE => {
                    m.hole_mul = 2.0;
                    m.hole_magnet = true;
                }
                ELITE => m.elite_drop = true,
                MAO_LEVE => m.thief_slow = true,
                OLHO_CLINICO => m.mimic_mark = true,
                SAUDADE => m.extra_continue += 1,
                REATIVO => m.regen_shield = true,
                IMA_CAPSULAS => m.capsule_magnet = true,
                BEZERRO => m.calf = true,
                DISCO_OURO => m.pulsar_card = true,
                DEVORADOR => {
                    m.score_mul *= 1.3;
                    m.bullet_speed *= 0.9;
                }
                AMULETO => m.amulet = true,
                CONTATO => m.ufo_more = true,
                FENDA => m.rift_mul *= 2.0,
                SAL => m.curse_mul *= 0.5,
                POEIRA => {
                    m.crystal_mul *= 1.15;
                    m.magnet_mul *= 1.2;
                }
                _ => {}
            }
        }
        m.drones = m.drones.min(3);
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toda_raridade_tem_cartas_sorteaveis() {
        for r in [Common, Rare, Epic, Legendary] {
            assert!(CARDS.iter().any(|c| c.rarity == r && !c.secret));
        }
    }

    #[test]
    fn indices_batem_com_nomes() {
        assert_eq!(CARDS[id::CONSTELACAO].name, "Constelação");
        assert_eq!(CARDS[id::PACIFISTA].name, "Pacifista");
        assert_eq!(CARDS[id::VACA].name, "Vaca Cósmica");
        assert_eq!(CARDS.len(), 65);
        assert_eq!(CARDS[id::DEVORADOR].name, "Olho do Devorador");
    }

    #[test]
    fn sorteio_nunca_da_secreta() {
        for _ in 0..500 {
            assert!(!CARDS[roll(2)].secret);
        }
    }

    #[test]
    fn espacos_por_colecao() {
        assert_eq!(slots_for(0), 1);
        assert_eq!(slots_for(8), 2);
        assert_eq!(slots_for(16), 3);
        assert_eq!(slots_for(30), 4);
    }

    #[test]
    fn efeitos_combinam() {
        let m = Mods::from_cards(&[id::CASCO, id::CORACAO, id::ARSENAL, id::DRONES, id::ESCOLTA]);
        assert_eq!(m.extra_lives, 2);
        assert_eq!(m.start_level, 3);
        assert_eq!(m.drones, 3);
        assert!(m.shield_per_sector);
    }
}
