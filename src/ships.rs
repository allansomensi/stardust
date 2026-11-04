//! Naves do hangar e armas.

use crate::progress::ach;
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ability {
    Sobrecarga,
    Rasante,
    Muralha,
    Pulso,
    Dobra,
    Canhao,
    Nostalgia,
    Aerogel,
    Saque,
    Reflexo,
}

pub enum Unlock {
    Rank(usize),
    Ach(usize),
}

pub struct ShipDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub speed: f32,
    pub lives: i32,
    pub fire: f32,
    pub dmg: f32,
    pub hit_r: f32,
    pub ability: Ability,
    pub ab_name: &'static str,
    pub ab_desc: &'static str,
    /// Fração da barra gasta ao usar a habilidade.
    pub ab_cost: f32,
    pub unlock: Unlock,
}

pub const SHIPS: &[ShipDef] = &[
    ShipDef {
        name: "VANGUARDA",
        desc: "Equilibrada e confiável. A nave de todo recruta.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Sobrecarga,
        ab_name: "SOBRECARGA",
        ab_desc: "5 s atirando o dobro com a arma um nível acima.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(0),
    },
    ShipDef {
        name: "FALCÃO",
        desc: "Rápida e pequena, mas com uma vida a menos.",
        speed: 1.25,
        lives: -1,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 3.5,
        ability: Ability::Rasante,
        ab_name: "RASANTE",
        ab_desc: "Um impulso instantâneo e invulnerável. Gasta 1/3 da barra.",
        ab_cost: 0.34,
        unlock: Unlock::Rank(3),
    },
    ShipDef {
        name: "BASTIÃO",
        desc: "Lenta e blindada: +1 vida e começa com escudo.",
        speed: 0.85,
        lives: 1,
        fire: 0.95,
        dmg: 1.0,
        hit_r: 5.5,
        ability: Ability::Muralha,
        ab_name: "MURALHA",
        ab_desc: "Ergue por 5 s uma barreira que bloqueia tiros.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(6),
    },
    ShipDef {
        name: "VEGA",
        desc: "Canaliza estrelas: a cada 6 disparos, um leque estelar.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Pulso,
        ab_name: "PULSO ESTELAR",
        ab_desc: "Uma onda de luz apaga tiros por perto e fere inimigos.",
        ab_cost: 0.6,
        unlock: Unlock::Rank(9),
    },
    ShipDef {
        name: "FANTASMA",
        desc: "Etérea. Hitbox menor e domínio sobre o tempo.",
        speed: 1.05,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.0,
        ability: Ability::Dobra,
        ab_name: "DOBRA TEMPORAL",
        ab_desc: "5 s com inimigos e tiros quase parados.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(12),
    },
    ShipDef {
        name: "COLOSSO",
        desc: "Atira devagar, mas cada tiro vale 60% a mais.",
        speed: 0.88,
        lives: 0,
        fire: 0.75,
        dmg: 1.6,
        hit_r: 5.0,
        ability: Ability::Canhao,
        ab_name: "CANHÃO ESTELAR",
        ab_desc: "Um raio gigante e contínuo por 3 s.",
        ab_cost: 1.0,
        unlock: Unlock::Rank(15),
    },
    ShipDef {
        name: "PROTÓTIPO",
        desc: "A primeira nave que existiu neste jogo. Feita de nostalgia.",
        speed: 1.0,
        lives: 0,
        fire: 1.1,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Nostalgia,
        ab_name: "NOSTALGIA",
        ab_desc: "Tudo na tela vira anos 2000 e leva dano.",
        ab_cost: 1.0,
        unlock: Unlock::Ach(ach::KONAMI),
    },
    ShipDef {
        name: "SONDA",
        desc: "A nave da missão Stardust. Ímã de cristais 50% maior.",
        speed: 1.0,
        lives: 0,
        fire: 1.0,
        dmg: 1.0,
        hit_r: 4.5,
        ability: Ability::Aerogel,
        ab_name: "AEROGEL",
        ab_desc: "4 s capturando os tiros por perto: cada um vira cristal.",
        ab_cost: 1.0,
        unlock: Unlock::Ach(ach::SALA_SECRETA),
    },
    ShipDef {
        name: "CORSÁRIA",
        desc: "Rápida e gananciosa: +20% de cristais.",
        speed: 1.15,
        lives: 0,
        fire: 1.05,
        dmg: 0.95,
        hit_r: 4.0,
        ability: Ability::Saque,
        ab_name: "SAQUE",
        ab_desc: "5 s puxando todos os itens da tela (menos maldições).",
        ab_cost: 0.7,
        unlock: Unlock::Ach(ach::FLIPERAMISTA),
    },
    ShipDef {
        name: "ÉGIDE",
        desc: "Pesada e paciente: +1 vida e tiros 10% mais fortes.",
        speed: 0.9,
        lives: 1,
        fire: 0.95,
        dmg: 1.1,
        hit_r: 5.0,
        ability: Ability::Reflexo,
        ab_name: "REFLEXO",
        ab_desc: "3 s devolvendo os tiros inimigos que chegam perto.",
        ab_cost: 0.8,
        unlock: Unlock::Ach(ach::PERFECCIONISTA),
    },
];

pub fn ship_unlocked(i: usize, rank: usize, achs: &[bool]) -> bool {
    match SHIPS[i].unlock {
        Unlock::Rank(r) => rank >= r,
        Unlock::Ach(a) => achs.get(a).copied().unwrap_or(false),
    }
}

pub fn unlock_hint(i: usize) -> String {
    match SHIPS[i].unlock {
        Unlock::Rank(r) => format!("Patente: {}", crate::progress::RANKS[r]),
        Unlock::Ach(a) if a == ach::SALA_SECRETA => "Uma Insígnia de Honra".to_string(),
        Unlock::Ach(a) if a == ach::FLIPERAMISTA => "Insígnia do Fliperama".to_string(),
        Unlock::Ach(a) if a == ach::PERFECCIONISTA => "Insígnia do Perfeccionista".to_string(),
        Unlock::Ach(_) => "???".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Armas
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Rajada,
    Laser,
    Missil,
    Onda,
    Estilhaco,
    Disco,
    Trovao,
    Chama,
    Pulsar,
    Ricochete,
    Granada,
    Crio,
    Orbita,
    BuracoNegro,
    Colmeia,
    Vaca,
    Agulha,
    Bumerangue,
    Prisma,
    Mina,
    Supernova,
    Poeira,
    Lanca,
    Singularidade,
}

/// As quatro Relíquias: armas absurdas que só aparecem por caminhos secretos.
pub const RELICS: [Weapon; 4] = [
    Weapon::Poeira,
    Weapon::Supernova,
    Weapon::Lanca,
    Weapon::Singularidade,
];

pub const WEAPONS: [Weapon; 24] = [
    Weapon::Rajada,
    Weapon::Laser,
    Weapon::Missil,
    Weapon::Onda,
    Weapon::Estilhaco,
    Weapon::Disco,
    Weapon::Trovao,
    Weapon::Chama,
    Weapon::Pulsar,
    Weapon::Ricochete,
    Weapon::Granada,
    Weapon::Crio,
    Weapon::Orbita,
    Weapon::BuracoNegro,
    Weapon::Colmeia,
    Weapon::Vaca,
    Weapon::Agulha,
    Weapon::Bumerangue,
    Weapon::Prisma,
    Weapon::Mina,
    Weapon::Supernova,
    Weapon::Poeira,
    Weapon::Lanca,
    Weapon::Singularidade,
];

impl Weapon {
    pub fn name(self) -> &'static str {
        match self {
            Weapon::Rajada => "RAJADA",
            Weapon::Laser => "LASER",
            Weapon::Missil => "MÍSSIL",
            Weapon::Onda => "ONDA",
            Weapon::Estilhaco => "ESTILHAÇO",
            Weapon::Disco => "DISCO",
            Weapon::Trovao => "TROVÃO",
            Weapon::Chama => "CHAMA",
            Weapon::Pulsar => "PULSAR",
            Weapon::Ricochete => "RICOCHETE",
            Weapon::Granada => "GRANADA",
            Weapon::Crio => "CRIO",
            Weapon::Orbita => "ÓRBITA",
            Weapon::BuracoNegro => "BURACO NEGRO",
            Weapon::Colmeia => "COLMEIA",
            Weapon::Vaca => "VACA-CANHÃO",
            Weapon::Agulha => "AGULHA",
            Weapon::Bumerangue => "BUMERANGUE",
            Weapon::Prisma => "PRISMA",
            Weapon::Mina => "MINA",
            Weapon::Supernova => "SUPERNOVA",
            Weapon::Poeira => "POEIRA ESTELAR",
            Weapon::Lanca => "LANÇA SOLAR",
            Weapon::Singularidade => "SINGULARIDADE",
        }
    }
    pub fn desc(self) -> &'static str {
        match self {
            Weapon::Rajada => "Leque largo de tiros rápidos.",
            Weapon::Laser => "Feixes que atravessam inimigos.",
            Weapon::Missil => "Mísseis teleguiados e fortes.",
            Weapon::Onda => "Tiros em hélice que ondulam pela tela.",
            Weapon::Estilhaco => "Escopeta: muito dano de perto.",
            Weapon::Disco => "Lâminas que vão e voltam.",
            Weapon::Trovao => "Raios que saltam entre inimigos.",
            Weapon::Chama => "Lança-chamas de curto alcance.",
            Weapon::Pulsar => "Estrelas cintilantes em leque. Secreta.",
            Weapon::Ricochete => "Tiros que quicam nas paredes.",
            Weapon::Granada => "Explode e acerta tudo em volta.",
            Weapon::Crio => "Congela: congelados levam dano dobrado.",
            Weapon::Orbita => "Serras giram em volta e bloqueiam tiros.",
            Weapon::BuracoNegro => "Engole tiros inimigos e tritura quem chega perto.",
            Weapon::Colmeia => "Abelhas teleguiadas. O Enxame contra o Enxame.",
            Weapon::Vaca => "Dispara vacas explosivas. Ninguém explica.",
            Weapon::Agulha => "Agulhas finíssimas e velozes que perfuram tudo.",
            Weapon::Bumerangue => "Vai, corta a formação inteira e volta para a mão.",
            Weapon::Prisma => "Um raio que se divide em três ao viajar.",
            Weapon::Mina => "Solta minas que sobem devagar e explodem em área.",
            Weapon::Supernova => "RELÍQUIA. Estrelas perfurantes e explosões em anel.",
            Weapon::Poeira => "RELÍQUIA. Uma nuvem de poeira estelar que caça tudo.",
            Weapon::Lanca => "RELÍQUIA. Um feixe de sol que atravessa a tela inteira.",
            Weapon::Singularidade => {
                "RELÍQUIA. Buracos negros em série e tiros que dobram o espaço."
            }
        }
    }
    pub fn color(self) -> Color {
        match self {
            Weapon::Rajada => Color::from_rgba(255, 200, 60, 255),
            Weapon::Laser => Color::from_rgba(90, 220, 255, 255),
            Weapon::Missil => Color::from_rgba(255, 100, 170, 255),
            Weapon::Onda => Color::from_rgba(120, 255, 170, 255),
            Weapon::Estilhaco => Color::from_rgba(255, 140, 60, 255),
            Weapon::Disco => Color::from_rgba(200, 200, 230, 255),
            Weapon::Trovao => Color::from_rgba(170, 150, 255, 255),
            Weapon::Chama => Color::from_rgba(255, 90, 40, 255),
            Weapon::Pulsar => Color::from_rgba(255, 220, 120, 255),
            Weapon::Ricochete => Color::from_rgba(120, 255, 230, 255),
            Weapon::Granada => Color::from_rgba(170, 200, 90, 255),
            Weapon::Crio => Color::from_rgba(170, 230, 255, 255),
            Weapon::Orbita => Color::from_rgba(230, 230, 240, 255),
            Weapon::BuracoNegro => Color::from_rgba(160, 90, 255, 255),
            Weapon::Colmeia => Color::from_rgba(255, 220, 60, 255),
            Weapon::Vaca => Color::from_rgba(255, 255, 255, 255),
            Weapon::Agulha => Color::from_rgba(220, 255, 140, 255),
            Weapon::Bumerangue => Color::from_rgba(255, 170, 90, 255),
            Weapon::Prisma => Color::from_rgba(150, 255, 255, 255),
            Weapon::Mina => Color::from_rgba(255, 110, 110, 255),
            Weapon::Supernova => Color::from_rgba(255, 240, 150, 255),
            Weapon::Poeira => Color::from_rgba(255, 190, 255, 255),
            Weapon::Lanca => Color::from_rgba(255, 200, 70, 255),
            Weapon::Singularidade => Color::from_rgba(200, 140, 255, 255),
        }
    }
    /// Classe de poder (1 a 5): armas de classe maior batem mais forte e custam mais na loja.
    pub fn tier(self) -> u32 {
        match self {
            Weapon::Rajada | Weapon::Laser | Weapon::Missil => 1,
            Weapon::Onda
            | Weapon::Estilhaco
            | Weapon::Disco
            | Weapon::Agulha
            | Weapon::Ricochete => 2,
            Weapon::Crio | Weapon::Trovao | Weapon::Granada | Weapon::Chama | Weapon::Mina => 3,
            Weapon::Colmeia | Weapon::Prisma | Weapon::Bumerangue | Weapon::Orbita => 4,
            _ => 5,
        }
    }
    /// Multiplicador de dano da classe.
    pub fn tier_mul(self) -> f32 {
        [1.0, 1.12, 1.25, 1.4, 1.55][(self.tier() - 1) as usize]
    }
    pub fn is_relic(self) -> bool {
        RELICS.contains(&self)
    }
    pub fn index(self) -> usize {
        WEAPONS.iter().position(|w| *w == self).unwrap_or(0)
    }
    /// Patente necessária (None = conquista secreta).
    pub fn unlock_rank(self) -> Option<usize> {
        match self {
            Weapon::Rajada | Weapon::Laser | Weapon::Missil => Some(0),
            Weapon::Onda => Some(2),
            Weapon::Estilhaco => Some(4),
            Weapon::Disco => Some(5),
            Weapon::Trovao => Some(8),
            Weapon::Chama => Some(10),
            Weapon::Ricochete => Some(6),
            Weapon::Crio => Some(7),
            Weapon::Granada => Some(9),
            Weapon::Colmeia => Some(11),
            Weapon::Orbita => Some(13),
            Weapon::BuracoNegro => Some(16),
            Weapon::Agulha => Some(3),
            Weapon::Mina => Some(8),
            Weapon::Prisma => Some(12),
            Weapon::Bumerangue => Some(14),
            _ => None,
        }
    }
    pub fn unlocked(self, rank: usize, achs: &[bool]) -> bool {
        if self.is_relic() {
            return false;
        }
        match self.unlock_rank() {
            Some(r) => rank >= r,
            None if self == Weapon::Vaca => achs.get(ach::MUUU).copied().unwrap_or(false),
            None => achs.get(ach::DISCO_OURO).copied().unwrap_or(false),
        }
    }
}
