//! User interface translation. The game is written in English; when Portuguese is selected,
//! every string drawn on screen goes through `tr`, which looks up the exact phrase or a
//! template with placeholders (for example `SECTOR {}`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Pt,
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    LANG.store(l as u8, Ordering::Relaxed);
    CACHE.with(|c| c.borrow_mut().clear());
}

pub fn lang() -> Lang {
    if LANG.load(Ordering::Relaxed) == 1 {
        Lang::Pt
    } else {
        Lang::En
    }
}

thread_local! {
    static CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Translates a finished string into the current language.
pub fn tr(s: &str) -> String {
    if lang() == Lang::En || s.is_empty() {
        return s.to_string();
    }
    if let Some(v) = CACHE.with(|c| c.borrow().get(s).cloned()) {
        return v;
    }
    let out = translate_depth(s, 0);
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 4000 {
            c.clear();
        }
        c.insert(s.to_string(), out.clone());
    });
    out
}

fn exact(s: &str) -> Option<&'static str> {
    EXACT
        .binary_search_by(|(k, _)| (*k).cmp(s))
        .ok()
        .map(|i| EXACT[i].1)
}

fn has_alpha(s: &str) -> bool {
    s.chars().any(|c| c.is_alphabetic())
}

fn translate_depth(s: &str, depth: u32) -> String {
    if let Some(v) = exact(s) {
        return v.to_string();
    }
    if depth > 3 || !has_alpha(s) {
        return s.to_string();
    }
    // Weapon tier stars at the end ("THUNDER ***").
    let trimmed = s.trim_end_matches('*');
    if trimmed.len() < s.len() && trimmed.ends_with(' ') {
        let base = trimmed.trim_end();
        return format!("{}{}", translate_depth(base, depth + 1), &s[base.len()..]);
    }
    let mut fallback: Option<String> = None;
    for (from, to) in TEMPLATES {
        let Some(vals) = matches(s, from) else {
            continue;
        };
        let mut clean = true;
        let mut out = String::new();
        for (i, part) in to.iter().enumerate() {
            out.push_str(part);
            if let Some(v) = vals.get(i) {
                if let Some(t) = exact(v) {
                    out.push_str(t);
                } else if has_alpha(v) {
                    let t = translate_depth(v, depth + 1);
                    if t == *v {
                        clean = false;
                    }
                    out.push_str(&t);
                } else {
                    out.push_str(v);
                }
            }
        }
        if clean {
            return out;
        }
        if fallback.is_none() {
            fallback = Some(out);
        }
    }
    // Strings joined with separators ("RARE - description").
    for sep in ["  ·  ", " · ", " - "] {
        if s.contains(sep) {
            let parts: Vec<String> = s
                .split(sep)
                .map(|p| translate_depth(p, depth + 1))
                .collect();
            let joined = parts.join(sep);
            if joined != s {
                return joined;
            }
        }
    }
    if let Some(f) = fallback {
        return f;
    }
    if cfg!(feature = "dev") {
        eprintln!("I18N-MISS\t{s}");
    }
    s.to_string()
}

/// Matches `s` against a template split into its fixed parts; returns the placeholder values.
fn matches<'a>(s: &'a str, parts: &[&str]) -> Option<Vec<&'a str>> {
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !s.starts_with(first) || !s.ends_with(last) || s.len() < first.len() + last.len() {
        return None;
    }
    let mut rest = &s[first.len()..s.len() - last.len()];
    let mut vals = vec![];
    for part in &parts[1..parts.len() - 1] {
        let i = rest.find(part)?;
        if i == 0 {
            return None;
        }
        vals.push(&rest[..i]);
        rest = &rest[i + part.len()..];
    }
    if rest.is_empty() {
        return None;
    }
    vals.push(rest);
    Some(vals)
}

/// English -> Brazilian Portuguese, sorted by key for binary search.
static EXACT: &[(&str, &str)] = &[
    ("  ·  SECRET", "  ·  SECRETA"),
    (" (IN USE)", " (EM USO)"),
    (
        "*  Star: invincible, rams through",
        "*  Estrela: invencível e atropela",
    ),
    ("+1 BOMB", "+1 BOMBA"),
    ("+1 bomb (X key).", "+1 bomba (tecla X)."),
    (
        "+1 bomb at the start of the run.",
        "+1 bomba no início da partida.",
    ),
    ("+1 continue per run.", "+1 continue por partida."),
    ("+1 level on the current weapon.", "+1 nível na arma atual."),
    ("+1 life and +1 bomb. Moo.", "+1 vida e +1 bomba. Muuu."),
    (
        "+1 life and a fresh shield every sector.",
        "+1 vida e escudo novo a cada setor.",
    ),
    (
        "+1 life at the start of the run.",
        "+1 vida no início da partida.",
    ),
    ("+1 reserve ship.", "+1 nave reserva."),
    ("+1 ship", "+1 nave"),
    ("+1 ship now.", "+1 nave agora."),
    ("+1 support drone (max 3).", "+1 drone de apoio (máximo 3)."),
    ("+1 weapon level", "+1 nível de arma"),
    (
        "+15% damage on every weapon. Stacks up to 6x.",
        "+15% de dano em todas as armas. Acumula até 6x.",
    ),
    ("+2 bombs", "+2 bombas"),
    ("+2 bombs now.", "+2 bombas agora."),
    ("+25 CRYSTALS", "+25 CRISTAIS"),
    (
        "+25000 points and 30 crystals",
        "+25000 pontos e 30 cristais",
    ),
    (
        "+3 lives. You know what you did.",
        "+3 vidas. Você sabe o que fez.",
    ),
    ("+45 crystals", "+45 cristais"),
    (
        "100 levels, bosses, shop and cards. The full adventure.",
        "100 níveis, chefes, loja e cartas. A aventura completa.",
    ),
    (
        "12 to win · 22 without crashing = perfect",
        "12 para vencer · 22 sem bater = perfeito",
    ),
    ("16 cards", "16 cartas"),
    ("1UP  Extra life (rare!)", "1UP  Vida extra (rara!)"),
    (
        "3 missions per run. Complete them for XP",
        "3 missões por partida. Cumpra e ganhe XP",
    ),
    (
        "3 s sending nearby enemy shots back.",
        "3 s devolvendo os tiros inimigos que chegam perto.",
    ),
    (
        "3 sectors with a crazy rule that changes daily. Same for everyone.",
        "3 setores com uma regra maluca que muda todo dia. Igual para todos.",
    ),
    ("3 shots destroy a curse.", "3 tiros destroem uma maldição."),
    ("30 cards", "30 cartas"),
    (
        "4 s catching nearby shots: each one becomes a crystal.",
        "4 s capturando os tiros por perto: cada um vira cristal.",
    ),
    (
        "5 s firing twice as fast, one weapon level higher.",
        "5 s atirando o dobro com a arma um nível acima.",
    ),
    (
        "5 s pulling every item on screen (except curses).",
        "5 s puxando todos os itens da tela (menos maldições).",
    ),
    (
        "5 s with enemies and shots almost frozen.",
        "5 s com inimigos e tiros quase parados.",
    ),
    ("8 cards", "8 cartas"),
    ("A BOSS APPROACHES", "UM CHEFE SE APROXIMA"),
    ("A CALF FOLLOWS YOU", "UM BEZERRO TE SEGUE"),
    ("A CARD!", "UMA CARTA!"),
    (
        "A Golden Scarab appears every sector.",
        "Um Escaravelho Dourado aparece em todo setor.",
    ),
    (
        "A Golden Scarab in the next sector.",
        "Um Escaravelho Dourado no próximo setor.",
    ),
    ("A Honor Insignia", "Uma Insígnia de Honra"),
    ("A MERCHANT APPROACHES", "UM MERCADOR SE APROXIMA"),
    ("A MOTH!", "UMA TRAÇA!"),
    ("A PORTAL OPENED", "UM PORTAL SE ABRIU"),
    ("A RELIC APPEARED!", "UMA RELÍQUIA SURGIU!"),
    (
        "A beam that splits into three as it travels.",
        "Um raio que se divide em três ao viajar.",
    ),
    (
        "A blazing fireball that explodes and scatters embers.",
        "Uma bola de fogo ardente que explode e espalha brasas.",
    ),
    (
        "A calf follows you and gathers crystals. Moo.",
        "Um bezerro te acompanha e recolhe cristais. Muuu.",
    ),
    (
        "A comet the Swarm tamed. Its tail sheds cosmic dust that hurts.",
        "Um cometa que o Enxame domou. Sua cauda solta poeira estelar que fere.",
    ),
    (
        "A consolation prize: 5 crystals.",
        "Uma lembrança de consolo: 5 cristais.",
    ),
    (
        "A drone that fires with you.",
        "Um drone que atira com você.",
    ),
    (
        "A flying nursery. Releases hatchlings while alive.",
        "Uma creche voadora. Solta filhotes enquanto estiver viva.",
    ),
    ("A fortune in crystals!", "Uma fortuna em cristais!"),
    (
        "A fresh shield at the start of every sector.",
        "Um escudo novo no começo de cada setor.",
    ),
    ("A fresh start, pilot.", "Um novo começo, piloto."),
    (
        "A giant, continuous beam for 3 s.",
        "Um raio gigante e contínuo por 3 s.",
    ),
    (
        "A heavy rock that pierces and shatters into shards.",
        "Uma rocha pesada que perfura e se estilhaça em fragmentos.",
    ),
    (
        "A line shows where your shot will go.",
        "Uma linha mostra onde o tiro vai passar.",
    ),
    (
        "A neutral trader. Returns kindness... and never forgets a betrayal.",
        "Um comerciante neutro. Retribui a gentileza... e nunca esquece uma traição.",
    ),
    (
        "A random card for your collection.",
        "Uma carta aleatória para a coleção.",
    ),
    (
        "A spider the size of a moon. Its webs cross the entire screen.",
        "Aranha do tamanho de uma lua. Suas teias atravessam a tela inteira.",
    ),
    (
        "A stolen, infested ship. The central cannon fires a telegraphed laser.",
        "Uma nave roubada e infestada. O canhão central dispara um laser avisado.",
    ),
    (
        "A tail of burning rings. Each ring destroyed weakens the beast.",
        "Uma cauda de anéis em brasa. Cada anel destruído enfraquece a fera.",
    ),
    (
        "A treasure hidden between dimensions...",
        "Um tesouro escondido entre as dimensões...",
    ),
    (
        "A war machine: fast, with a hair trigger.",
        "Uma máquina de guerra: rápida e com gatilho sensível.",
    ),
    (
        "A wave of light wipes nearby shots and hurts enemies.",
        "Uma onda de luz apaga tiros por perto e fere inimigos.",
    ),
    (
        "A wingman fires at your side for 2 sectors.",
        "Um ala atira ao seu lado por 2 setores.",
    ),
    ("ABILITIES", "HABILIDADES"),
    ("ABILITY", "HABILIDADE"),
    ("ABILITY  [C]", "HABILIDADE  [C]"),
    ("ABYSSAL WEAVER", "TECELÃ ABISSAL"),
    ("ACE", "ÁS"),
    ("ACHIEVEMENTS", "CONQUISTAS"),
    ("ADMIRAL", "ALMIRANTE"),
    ("AEGIS", "ÉGIDE"),
    ("AEGIS ship", "Nave ÉGIDE"),
    ("ALWAYS SHOW HITBOX", "HITBOX SEMPRE VISÍVEL"),
    ("AMBUSH", "EMBOSCADA"),
    ("AMULET", "AMULETO"),
    ("ANCHOR", "ÂNCORA"),
    ("ANCHOR!", "ÂNCORA!"),
    ("ANCIENT EYE", "OLHO ANCESTRAL"),
    ("ANCIENT RUINS", "RUÍNAS ANCESTRAIS"),
    ("ANCIENT SERPENT", "SERPENTE ANCESTRAL"),
    ("ANCIENT WILD COMET", "COMETA WILD ANCESTRAL"),
    ("ANIMATED BACKGROUND", "FUNDO ANIMADO"),
    ("ARCADE INSIGNIA", "INSÍGNIA DO FLIPERAMA"),
    ("ARCHIVES", "ARQUIVOS"),
    ("ARCHIVES, HONORS tab.", "ARQUIVOS, aba HONRAS."),
    ("ARMOR", "BLINDAGEM"),
    ("ARMORED", "BLINDADO"),
    (
        "ARROWS choose  ·  ENTER launch  ·  ESC back",
        "SETAS escolhem  ·  ENTER decola  ·  ESC volta",
    ),
    ("ARROWS · ENTER · ESC", "SETAS · ENTER · ESC"),
    ("ARSENAL", "PAIOL"),
    ("ARSENAL INSIGNIA", "INSÍGNIA DO ARSENAL"),
    ("ART", "ARTE"),
    ("ASTEROID FIELD", "CAMPO DE ASTEROIDES"),
    ("ATTACK DRONE", "DRONE DE ATAQUE"),
    ("AUDIO", "ÁUDIO"),
    ("AUTOFIRE", "TIRO AUTOMÁTICO"),
    (
        "AUTOFIRE is sold in the shop.",
        "TIRO AUTOMÁTICO se compra na loja.",
    ),
    ("AUTOFIRE!", "TIRO AUTOMÁTICO!"),
    ("Ability", "Habilidade"),
    (
        "Ability charges 30% faster.",
        "Habilidade carrega 30% mais rápido.",
    ),
    ("Absolute Zero", "Zero Absoluto"),
    ("Absorbs one hit.", "Absorve um golpe."),
    ("Aim", "Pontaria"),
    (
        "Always shows your ship's weak spot.",
        "Mostra sempre o ponto vulnerável da nave.",
    ),
    (
        "An escort drone follows you.",
        "Um drone de escolta acompanha você.",
    ),
    (
        "An instant, invulnerable burst. Costs 1/3 of the bar.",
        "Um impulso instantâneo e invulnerável. Gasta 1/3 da barra.",
    ),
    ("Ancient Amulet", "Amuleto Antigo"),
    ("Arachnophobia", "Aracnofobia"),
    ("Arcade", "Fliperama"),
    ("Arcade Insignia", "Insígnia do Fliperama"),
    ("Arcade Master", "Fliperamista"),
    ("Archaeologist", "Arqueólogo"),
    (
        "Armored commander. Takes two hits and always brings escorts when diving.",
        "Comandante blindado. Aguenta dois tiros e sempre leva escolta nos mergulhos.",
    ),
    (
        "Arrows / A D ....... move sideways",
        "Setas / A D ........ mover para os lados",
    ),
    ("Arrows and Space", "Setas e Espaço"),
    ("Arrows or A/D", "Setas ou A/D"),
    ("Arrows or WASD", "Setas ou WASD"),
    ("Attack the Merchant.", "Ataque o Mercador."),
    ("BACK", "VOLTAR"),
    ("BASTION", "BASTIÃO"),
    (
        "BASTION ship and RICOCHET weapon",
        "Nave BASTIÃO e arma RICOCHETE",
    ),
    ("BEETLE", "BESOURO"),
    ("BESTIARY", "BESTIÁRIO"),
    ("BLACK HOLE", "BURACO NEGRO"),
    ("BLASTER", "RAJADA"),
    ("BOMB", "BOMBA"),
    ("BOMBS", "BOMBAS"),
    ("BONUS", "BÔNUS"),
    ("BONUS CRYSTALS", "CRISTAIS DE BÔNUS"),
    ("BONUS RESULT", "RESULTADO DO BÔNUS"),
    ("BONUS STAGE", "ESTÁGIO BÔNUS"),
    ("BOOMERANG", "BUMERANGUE"),
    ("BOSS", "CHEFE"),
    ("BOSS DEFEATED", "CHEFE DERROTADO"),
    ("BOSS DOWN!", "CHEFE ABATIDO!"),
    ("BOSS RUSH", "CHEFÕES"),
    ("BOSS RUSH mode and CRYO weapon", "Modo CHEFÕES e arma CRIO"),
    (
        "BOSS RUSH mode, CRYO and METEOR weapons",
        "Modo CHEFÕES, armas CRIO e METEORO",
    ),
    ("BULWARK", "MURALHA"),
    (
        "Balanced and reliable. Every recruit's ship.",
        "Equilibrada e confiável. A nave de todo recruta.",
    ),
    (
        "Beams that pierce through enemies.",
        "Feixes que atravessam inimigos.",
    ),
    (
        "Beat Boss Rush mode in under 12 minutes.",
        "Vença o modo Chefões em menos de 12 minutos.",
    ),
    ("Best combo", "Maior combo"),
    ("Best sector", "Melhor setor"),
    ("Best wave (survival)", "Melhor onda (sobrevivência)"),
    (
        "Between sectors, spend crystals on better",
        "Entre os setores, gaste cristais em armas",
    ),
    (
        "Black holes last twice as long and suck in crystals.",
        "Buracos negros duram o dobro e sugam cristais.",
    ),
    (
        "Blades that fly out and come back.",
        "Lâminas que vão e voltam.",
    ),
    (
        "Bolts that jump between enemies.",
        "Raios que saltam entre inimigos.",
    ),
    ("Bomb", "Bomba"),
    ("Bomber", "Bombardeiro"),
    (
        "Bombs deal double damage.",
        "Bombas causam o dobro de dano.",
    ),
    ("Bombs used", "Bombas usadas"),
    (
        "Bombs wipe the shots on screen. Use one before you get hit.",
        "Bombas apagam os tiros na tela. Use antes de ser atingido.",
    ),
    (
        "Bonus stages are worth double.",
        "Estágios bônus valem o dobro.",
    ),
    (
        "Born from Splitters and Carriers. Small, fast and stubborn.",
        "Nasce dos Divisores e das Portadoras. Pequeno, rápido e teimoso.",
    ),
    (
        "Born in the dark: tiny hitbox, 20% stronger shots, one less life.",
        "Nascida no escuro: alvo minúsculo, tiros 20% mais fortes, uma vida a menos.",
    ),
    ("Bosses defeated", "Chefes derrotados"),
    (
        "Bouncing shots get 2 extra bounces.",
        "Tiros que quicam ganham mais 2 quiques.",
    ),
    ("Broken Heart", "Coração Partido"),
    (
        "But not every question was answered.",
        "Mas nem todas as perguntas foram respondidas.",
    ),
    (
        "C / H .............. ship ability",
        "C / H .............. habilidade da nave",
    ),
    ("C, L or V", "C, L ou V"),
    ("CADET", "CADETE"),
    ("CAMPAIGN", "CAMPANHA"),
    ("CANNON", "CANHÃO"),
    ("CAPSULE RAIN", "CHUVA DE CÁPSULAS"),
    ("CAPTAIN", "CAPITÃO"),
    ("CARD RADAR", "RADAR DE CARTAS"),
    ("CARDS", "CARTAS"),
    ("CARGO CONVOY", "COMBOIO DE CARGA"),
    ("CARRIER", "PORTADORA"),
    ("CAST", "ELENCO"),
    ("CHAMELEON", "CAMALEÃO"),
    ("CHARGED SHOT", "TIRO CARREGADO"),
    ("CHOOSE A BLESSING", "ESCOLHA UMA BÊNÇÃO"),
    ("CLASSIC", "CLÁSSICO"),
    ("CLASSIC mode", "Modo CLÁSSICO"),
    ("CLOVER", "TREVO"),
    ("CODE ACCEPTED", "CÓDIGO ACEITO"),
    ("COLLECTION", "COLEÇÃO"),
    ("COLONEL", "CORONEL"),
    ("COLOSSUS", "COLOSSO"),
    ("COLOSSUS ship", "Nave COLOSSO"),
    ("COMET", "COMETA"),
    ("COMET SERPENT", "SERPENTE COMETA"),
    ("COMMODORE", "COMODORO"),
    ("COMMON", "COMUM"),
    ("CONFIRM?", "CONFIRMA?"),
    ("CONTINUE", "CONTINUAR"),
    ("CONTINUE?", "CONTINUAR?"),
    ("CONTRACT COMPLETE", "CONTRATO CUMPRIDO"),
    ("CONTRACTS", "CONTRATOS"),
    ("CONTROLS", "CONTROLES"),
    ("CORSAIR", "CORSÁRIA"),
    ("CORSAIR ship", "Nave CORSÁRIA"),
    ("COSMIC COW", "VACA CÓSMICA"),
    ("COSMIC DUST", "POEIRA ESTELAR"),
    ("COW", "VACA"),
    ("COW CANNON", "VACA-CANHÃO"),
    ("CREATED BY", "CRIAÇÃO"),
    ("CRIMSON NEBULA", "NEBULOSA CARMESIM"),
    ("CRT FILTER", "FILTRO CRT"),
    ("CRYO", "CRIO"),
    ("CRYSTAL RAIN", "CHUVA DE CRISTAIS"),
    ("CRYSTALLIZER", "CRISTALIZADOR"),
    ("CRYSTALS!", "CRISTAIS!"),
    (
        "Can only be hurt with its eyelid open. Its gaze sweeps space with beams.",
        "Só pode ser ferido com a pálpebra aberta. Seu olhar varre o espaço com raios.",
    ),
    ("Cannon Shot", "Tiro de Canhão"),
    ("Capsule Magnet", "Ímã de Cápsulas"),
    (
        "Capsules and power-ups come to you.",
        "Cápsulas e poderes vêm até você.",
    ),
    (
        "Capsules and power-ups everywhere.",
        "Cápsulas e power-ups por toda parte.",
    ),
    (
        "Capsules cycle through 4 weapons and last longer.",
        "Cápsulas alternam entre 4 armas e duram mais.",
    ),
    ("Card Fisher", "Pescador de Cartas"),
    ("Card Magnet", "Ímã de Cartas"),
    ("Cards drop 50% more often.", "Cartas caem 50% mais."),
    (
        "Cards drop twice as often.",
        "Cartas caem com o dobro de frequência.",
    ),
    ("Cards fly to you.", "Cartas voam até você."),
    (
        "Carriers, Chameleons, Mimics, Thieves,",
        "Portadoras, Camaleões, Mímicos, Ladrões,",
    ),
    (
        "Carries a chitin plate in front. Break the shield before you can hurt it.",
        "Carrega uma placa de quitina na frente. Quebre o escudo antes de feri-lo.",
    ),
    (
        "Carries two biological cannons. Stops mid-screen to fire a spread.",
        "Carrega dois canhões biológicos. Para no meio da tela para disparar em leque.",
    ),
    ("Catch the falling crystals.", "Pegue os cristais que caem."),
    ("Challenge complete!", "Desafio completo!"),
    ("Change ships in the HANGAR.", "Troque de nave no HANGAR."),
    (
        "Channels stars: every 6 shots, a star fan.",
        "Canaliza estrelas: a cada 6 disparos, um leque estelar.",
    ),
    ("Charged shot", "Tiro carregado"),
    (
        "Charged shot 40% faster and stronger.",
        "Tiro carregado 40% mais rápido e forte.",
    ),
    (
        "Chiptune synthesized by the game itself,",
        "Chiptune sintetizado pelo próprio jogo,",
    ),
    (
        "Clear 100 levels and defeat the Supreme",
        "Vença os 100 níveis e derrote a Rainha",
    ),
    (
        "Clear a wave in under 7 seconds.",
        "Limpe uma onda em menos de 7 segundos.",
    ),
    (
        "Clear all 100 levels. Reach the Heart of the Hive.",
        "Vença os 100 níveis. Chegue ao Coração da Colmeia.",
    ),
    ("Clear the first wave.", "Termine a primeira onda."),
    (
        "Clearing a wave quickly earns a time bonus.",
        "Limpar uma onda rápido rende bônus de tempo.",
    ),
    ("Close Encounters", "Contatos Imediatos"),
    ("Close Shave", "Na Raspa"),
    (
        "Co-op: PLAYERS 2 on the mode screen",
        "Co-op: JOGADORES 2 na tela de modo",
    ),
    (
        "Collect all 8: extra life and tons of points.",
        "Junte as 8: vida extra e muitos pontos.",
    ),
    (
        "Collect the letters S-T-A-R-D-U-S-T.",
        "Junte as letras S-T-A-R-D-U-S-T.",
    ),
    ("Collector", "Colecionador"),
    (
        "Combo 50 = Fever: double points and a",
        "Combo 50 = Febre: pontos em dobro e um",
    ),
    ("Combo window 50% longer.", "Janela do combo 50% maior."),
    (
        "Come back tomorrow for a new rule.",
        "Volte amanhã para uma nova regra.",
    ),
    ("Comet Dust", "Poeira de Cometa"),
    ("Comet Hunter", "Caçador de Cometas"),
    (
        "Complete 3 daily challenges.",
        "Termine 3 desafios diários.",
    ),
    (
        "Complete 30 contracts in total.",
        "Complete 30 contratos no total.",
    ),
    ("Complete a contract.", "Complete um contrato."),
    (
        "Completed contracts give extra XP at the end of the run.",
        "Contratos cumpridos dão XP extra no fim da partida.",
    ),
    ("Completionist", "Completista"),
    ("Constellation", "Constelação"),
    ("Contact Signal", "Sinal de Contato"),
    (
        "Controls reversed for 6 seconds.",
        "Controles invertidos por 6 segundos.",
    ),
    ("Cosmic Calf", "Bezerro Cósmico"),
    ("Cosmic Cow", "Vaca Cósmica"),
    ("Cosmic Tourist", "Turista Cósmico"),
    ("Counterattack", "Contra-Ataque"),
    ("Creator", "Criador"),
    ("Crystal Wings", "Asas de Cristal"),
    (
        "Crystal magnet radius +60%.",
        "Raio do ímã de cristais +60%.",
    ),
    (
        "Crystals +15% and 20% bigger magnet.",
        "Cristais +15% e ímã 20% maior.",
    ),
    (
        "Crystals +30% for the rest of the expedition.",
        "Cristais +30% pelo resto da expedição.",
    ),
    ("Crystals are worth 25% more.", "Cristais valem 25% a mais."),
    (
        "Crystals are worth double this sector.",
        "Cristais valem o dobro neste setor.",
    ),
    (
        "Crystals buy better weapons. Save up for the boss.",
        "Cristais compram armas melhores. Guarde para o chefe.",
    ),
    ("Crystals collected", "Cristais coletados"),
    (
        "Crystals come to you from anywhere.",
        "Cristais vêm até você de qualquer lugar.",
    ),
    (
        "Crystals x1.5 and points x1.2.",
        "Cristais x1,5 e pontos x1,2.",
    ),
    ("Curious", "Curioso"),
    ("Curses drop 50% less often.", "Maldições caem 50% menos."),
    ("Curses exorcised / taken", "Maldições exorcizadas / pegas"),
    ("DAILY CHALLENGE", "DESAFIO DIÁRIO"),
    ("DAMAGE", "DANO"),
    ("DAMAGE CORE", "NÚCLEO DE DANO"),
    ("DASH", "RASANTE"),
    (
        "DEMO  -  PRESS ANY KEY",
        "DEMONSTRAÇÃO  -  APERTE QUALQUER TECLA",
    ),
    ("DEVICE", "DISPOSITIVO"),
    ("DIFFICULTY", "DIFICULDADE"),
    ("DISC", "DISCO"),
    (
        "DISC and FIREBALL weapons, HARD difficulty",
        "Armas DISCO e BOLA DE FOGO, dificuldade DIFÍCIL",
    ),
    (
        "DISC weapon and HARD difficulty",
        "Arma DISCO e dificuldade DIFÍCIL",
    ),
    ("DOUBLE CRYSTALS", "CRISTAIS EM DOBRO"),
    ("DOUBLE SHOTS", "TIROS DOBRADOS"),
    ("DRONE", "ZANGÃO"),
    ("DUO BEAM!", "RAIO DUPLO!"),
    ("DUO READY: ability", "DUO PRONTO: habilidade"),
    ("DYNAMIC LIGHTING", "ILUMINAÇÃO DINÂMICA"),
    ("Daily challenges won", "Desafios diários vencidos"),
    ("Dangerous Graze", "Raspão Perigoso"),
    ("Dark Duel", "Duelo no Escuro"),
    (
        "Defeat a boss in the dark without being hit.",
        "Derrote um chefe no escuro sem ser atingido.",
    ),
    (
        "Defeat a boss without getting hit.",
        "Derrote um chefe sem ser atingido.",
    ),
    ("Defeat the Abyssal Weaver.", "Derrote a Tecelã Abissal."),
    ("Defeat the Comet Serpent.", "Derrote a Serpente Cometa."),
    ("Defeat the Devourer.", "Derrote o Devorador."),
    ("Defeat the Eye of the Void.", "Derrote o Olho do Vazio."),
    ("Defeat the Hive Queen.", "Derrote a Rainha da Colmeia."),
    ("Defeat the Twins.", "Derrote as Gêmeas."),
    ("Defeat the Wild Comet.", "Derrote o Cometa Wild."),
    (
        "Defeat the Zenith Dreadnought.",
        "Derrote o Couraçado Zênite.",
    ),
    (
        "Destroy 25 curses with your shots.",
        "Destrua 25 maldições a tiros.",
    ),
    (
        "Destroy a boss with the co-op DUO BEAM.",
        "Destrua um chefe com o RAIO DUPLO do co-op.",
    ),
    (
        "Destroy every part of a boss before its body.",
        "Destrua todas as partes de um chefe antes do corpo.",
    ),
    ("Dimension Traveler", "Viajante Dimensional"),
    (
        "Dives 40% faster, points x2.",
        "Mergulhos 40% mais rápidos, pontos x2.",
    ),
    (
        "Dives while dropping mines that explode in a spread. Shoot the mines!",
        "Desce em mergulho largando minas que explodem em leque. Atire nas minas!",
    ),
    (
        "Diving enemies are worth double points.",
        "Inimigos em mergulho valem o dobro de pontos.",
    ),
    (
        "Diving enemies are worth triple.",
        "Inimigos em mergulho valem o triplo.",
    ),
    ("Dodge the rocks!", "Desvie das pedras!"),
    (
        "Doesn't shoot. Doesn't dodge. Just wants to touch you.",
        "Não atira. Não desvia. Só quer encostar em você.",
    ),
    ("Don't Blink", "Não Pisque"),
    ("Double Arsenal", "Arsenal Duplo"),
    (
        "Double contract rewards.",
        "Recompensas de contratos em dobro.",
    ),
    (
        "Double crystals, shop 50% more expensive.",
        "Cristais em dobro, loja 50% mais cara.",
    ),
    (
        "Double points and crystals on every kill!",
        "Pontos em dobro e cristais a cada abate!",
    ),
    ("Double reward and a card!", "Prêmio dobrado e uma carta!"),
    (
        "Drones, Wasps, Beetles, Gunners,",
        "Zangões, Vespas, Besouros, Atiradores,",
    ),
    (
        "Dropped by bosses. They tell the truth.",
        "Caem dos chefes. Contam a verdade.",
    ),
    (
        "Drops mines that rise slowly and explode in an area.",
        "Solta minas que sobem devagar e explodem em área.",
    ),
    ("EASY", "FÁCIL"),
    ("ELITE PILOT", "PILOTO DE ELITE"),
    ("EMERALD", "ESMERALDA"),
    ("EMERALD paint", "Pintura ESMERALDA"),
    ("EMPTY...", "VAZIA..."),
    ("END RUN", "ENCERRAR PARTIDA"),
    ("END THE RUN", "ENCERRAR A PARTIDA"),
    ("ENERGY CELL", "CÉLULA DE ENERGIA"),
    (
        "ENTER confirms  ·  BACKSPACE deletes",
        "ENTER confirma  ·  BACKSPACE apaga",
    ),
    ("ENTER continues", "ENTER continua"),
    (
        "ENTER continues (lose half your points)",
        "ENTER continua (perde metade dos pontos)",
    ),
    ("ENTER equips", "ENTER equipa"),
    ("ENTER equips the trail", "ENTER equipa o rastro"),
    ("ENTER skips", "ENTER pula"),
    ("ENTER to continue", "ENTER para continuar"),
    ("ENTER to skip", "ENTER para pular"),
    (
        "ENTER, then press the new key or button.",
        "ENTER e depois aperte a nova tecla ou botão.",
    ),
    ("EPIC", "ÉPICA"),
    ("EQUIPPED", "EQUIPADA"),
    ("EQUIPPED CARDS", "CARTAS EQUIPADAS"),
    ("ERASE PROGRESS", "APAGAR PROGRESSO"),
    ("ESC gives up", "ESC desiste"),
    ("ETERNAL SHIELD", "ESCUDO ETERNO"),
    ("EXORCISED!", "EXORCIZADA!"),
    ("EXORCIST INSIGNIA", "INSÍGNIA DO EXORCISTA"),
    ("EXPEDITION", "EXPEDIÇÃO"),
    ("EXPEDITION INSIGNIA", "INSÍGNIA DA EXPEDIÇÃO"),
    ("EXPLORER INSIGNIA", "INSÍGNIA DO EXPLORADOR"),
    ("EXTRA LIFE", "VIDA EXTRA"),
    ("EXTRA LIFE!", "VIDA EXTRA!"),
    ("EYE OF THE VOID", "OLHO DO VAZIO"),
    (
        "Each ship has its own. The bar fills",
        "Cada nave tem a sua. A barra enche",
    ),
    ("Early Fever", "Febre Precoce"),
    (
        "Earns 15% of your crystals every sector.",
        "Rende 15% dos cristais a cada setor.",
    ),
    (
        "Eclipse, meteors, ambushes, convoys...",
        "Eclipse, meteoros, emboscadas, comboios...",
    ),
    ("Elite Hunter", "Caçador de Elite"),
    ("Elite Mercenary", "Mercenário de Elite"),
    ("Elite of the Elite", "Elite da Elite"),
    (
        "Elites (gold outline): tough and rich.",
        "Elites (borda dourada): fortes e ricos.",
    ),
    (
        "Elites always drop a capsule and sometimes a card.",
        "Elites sempre soltam cápsula e às vezes uma carta.",
    ),
    (
        "Endless sectors. Every sector, pick 1 of 3 blessings. How far can you go?",
        "Setores sem fim. A cada setor, escolha 1 de 3 bênçãos. Até onde você vai?",
    ),
    (
        "Enemies drop more crystals.",
        "Inimigos soltam mais cristais.",
    ),
    ("Enemies shot down", "Inimigos abatidos"),
    (
        "Enemies stay frozen twice as long.",
        "Inimigos ficam congelados o dobro do tempo.",
    ),
    ("Enemy shots 20% slower.", "Tiros inimigos 20% mais lentos."),
    (
        "Enemy shots curve toward you.",
        "Os tiros inimigos curvam na sua direção.",
    ),
    ("Enter Fever", "Entre em Febre"),
    (
        "Enter Fever 3 times in one run.",
        "Entre em Febre 3 vezes numa partida.",
    ),
    (
        "Enter Fever with a high combo.",
        "Entre em Febre com um combo alto.",
    ),
    (
        "Enter the Konami code on the title screen.",
        "Digite o código Konami na tela de título.",
    ),
    ("Enter your name:", "Digite seu nome:"),
    (
        "Equip them in the CARDS menu. With 8, 16 and 30",
        "Equipe no menu CARTAS. Com 8, 16 e 30",
    ),
    (
        "Erases ranks, cards and achievements. Press twice.",
        "Apaga patentes, cartas e conquistas. Aperte 2x.",
    ),
    ("Esc / P ............ pause", "Esc / P ............ pausa"),
    ("Esc or P", "Esc ou P"),
    ("Escort", "Escolta"),
    (
        "Ethereal. Smaller hitbox and mastery over time.",
        "Etérea. Hitbox menor e domínio sobre o tempo.",
    ),
    ("Event Horizon", "Horizonte de Eventos"),
    (
        "Every 100 crystals collected, +1 bomb.",
        "A cada 100 cristais coletados, +1 bomba.",
    ),
    (
        "Every 6 shots, a fan of 6 stars.",
        "A cada 6 disparos, um leque de 6 estrelas.",
    ),
    (
        "Every boss has been defeated.",
        "Todos os chefes foram derrotados.",
    ),
    (
        "Every boss in a row, against the clock.",
        "Todos os chefes em sequência, contra o relógio.",
    ),
    ("Every crash costs you.", "Cada batida custa caro."),
    (
        "Every crystal you collect turns into better weapons at the mobile hangar.",
        "Cada cristal recolhido vira armas melhores no hangar móvel.",
    ),
    (
        "Every few seconds it pulses and mends the wounds and armor of everyone around it. Kill it first.",
        "A cada poucos segundos pulsa e cura as feridas e a armadura de todos ao redor. Mate-o primeiro.",
    ),
    (
        "Every run gives XP and unlocks ships,",
        "Cada partida dá XP e libera naves,",
    ),
    (
        "Every run gives XP. Ranking up",
        "Toda partida dá XP. Subir de patente",
    ),
    (
        "Everything in the shop costs 20% less.",
        "Tudo na loja custa 20% menos.",
    ),
    (
        "Everything on screen turns 2000s and takes damage.",
        "Tudo na tela vira anos 2000 e leva dano.",
    ),
    ("Exorcist", "Exorcista"),
    ("Expeditionary", "Expedicionário"),
    (
        "Explodes and hits everything around it.",
        "Explode e acerta tudo em volta.",
    ),
    ("Explosions 40% bigger.", "Explosões 40% maiores."),
    ("Extra Gunpowder", "Pólvora Extra"),
    ("Extra Pocket", "Bolso Extra"),
    ("Extra diagonal shots.", "Tiros extras em diagonal."),
    ("Eye of the Devourer", "Olho do Devorador"),
    (
        "F  Frenzy: fires twice as fast",
        "F  Frenesi: atira o dobro",
    ),
    (
        "F11 / Alt+Enter .... fullscreen",
        "F11 / Alt+Enter .... tela cheia",
    ),
    (
        "F11 fullscreen · hold fire and release: charged shot",
        "F11 tela cheia · segure e solte o tiro: tiro carregado",
    ),
    ("F11 or Alt+Enter", "F11 ou Alt+Enter"),
    ("FALCON", "FALCÃO"),
    (
        "FALCON ship and DAILY CHALLENGE",
        "Nave FALCÃO e DESAFIO DIÁRIO",
    ),
    ("FAST ENEMIES", "INIMIGOS VELOZES"),
    ("FEVER", "FEBRE"),
    ("FEVER!", "FEBRE!"),
    ("FIRE", "FOGO"),
    ("FIRE RATE", "CADÊNCIA"),
    ("FIRE trail", "Rastro FOGO"),
    ("FIREBALL", "BOLA DE FOGO"),
    ("FIREPOWER", "PODER DE FOGO"),
    ("FLAME", "CHAMA"),
    ("FLAME and PLASMA weapons", "Armas CHAMA e PLASMA"),
    ("FLAME weapon", "Arma CHAMA"),
    ("FLASHES", "CLARÕES"),
    ("FLEET ADMIRAL", "ALMIRANTE DE ESQUADRA"),
    ("FLIGHT LOGS", "REGISTROS DE BORDO"),
    ("FLIGHT REPORT", "RELATÓRIO DE VOO"),
    ("FOCUS", "FOCO"),
    ("FOG", "NEBLINA"),
    ("FREIGHTER", "CARGUEIRO"),
    ("FRENZY", "FRENESI"),
    ("FRENZY!", "FRENESI!"),
    ("FULLSCREEN", "TELA CHEIA"),
    ("Farmer", "Fazendeiro"),
    ("Farthest in Expedition", "Mais longe na Expedição"),
    (
        "Fast and greedy: +20% crystals.",
        "Rápida e gananciosa: +20% de cristais.",
    ),
    (
        "Fast and jumpy. Loves to dive in spirals before returning to formation.",
        "Rápida e nervosa. Gosta de mergulhar em espiral antes de voltar à formação.",
    ),
    (
        "Fast and small, but with one less life.",
        "Rápida e pequena, mas com uma vida a menos.",
    ),
    ("Favorite weapon", "Arma favorita"),
    (
        "Feats unlock ships and trails. See the",
        "Feitos liberam naves e rastros. Veja nos",
    ),
    ("Fever lasts 50% longer.", "A Febre dura 50% mais."),
    (
        "Fever starts at combo 35 instead of 50.",
        "A Febre começa com combo 35 em vez de 50.",
    ),
    ("Fever!", "Febre!"),
    (
        "Fewer particles make the game lighter.",
        "Menos partículas deixa o jogo mais leve.",
    ),
    (
        "Fills the ability and speeds up recharge.",
        "Enche a habilidade e acelera a recarga.",
    ),
    ("Find 20 different cards.", "Encontre 20 cartas diferentes."),
    ("Find 6 flight logs.", "Encontre 6 registros de bordo."),
    ("Find a Relic.", "Encontre uma Relíquia."),
    ("Find a Secret Room.", "Encontre uma Sala Secreta."),
    (
        "Find all 12 flight logs.",
        "Encontre todos os 12 registros de bordo.",
    ),
    (
        "Find every common, rare, epic and legendary card.",
        "Encontre todas as cartas comuns, raras, épicas e lendárias.",
    ),
    ("Find the 4 Relics.", "Encontre as 4 Relíquias."),
    ("Find the Golden Dimension.", "Encontre a Dimensão Dourada."),
    (
        "Finish a bonus stage without firing.",
        "Termine um estágio bônus sem disparar.",
    ),
    (
        "Finish a sector during an Eclipse.",
        "Termine um setor durante um Eclipse.",
    ),
    (
        "Finish a sector without bombs",
        "Termine um setor sem bombas",
    ),
    (
        "Finish a sector without getting hit.",
        "Termine um setor sem ser atingido.",
    ),
    (
        "Finish the campaign on Hard without using continues.",
        "Termine a campanha no Difícil sem usar continues.",
    ),
    (
        "Finish the campaign on Nightmare.",
        "Termine a campanha no Pesadelo.",
    ),
    ("Finish the campaign.", "Termine a campanha."),
    ("Fire", "Atirar"),
    ("Fire rate +10%.", "Cadência de tiro +10%."),
    (
        "Fire rate +12% for the rest of the expedition.",
        "Cadência +12% pelo resto da expedição.",
    ),
    ("Fires 10% faster.", "Atira 10% mais rápido."),
    (
        "Fires exploding cows. Nobody can explain it.",
        "Dispara vacas explosivas. Ninguém explica.",
    ),
    (
        "Fires slowly, but each shot deals 60% more.",
        "Atira devagar, mas cada tiro vale 60% a mais.",
    ),
    ("First Flight", "Primeiro Voo"),
    (
        "Fixed: arrows + Enter + Esc in menus · Esc/P/Start pause",
        "Fixas: setas + Enter + Esc nos menus · Esc/P/Start pausa",
    ),
    (
        "Fly into it to skip the boss and the sector!",
        "Entre nele para pular o chefe e o setor!",
    ),
    (
        "Fly into the capsule. Only once per run!",
        "Voe até a cápsula. Só uma vez por partida!",
    ),
    ("Fool's Gold", "Ouro de Tolo"),
    (
        "Freeze 150 enemies in total.",
        "Congele 150 inimigos no total.",
    ),
    (
        "Freezes: frozen enemies take double damage.",
        "Congela: congelados levam dano dobrado.",
    ),
    (
        "Freighters full of crystals cross the screen.",
        "Cargueiros lotados de cristais cruzam a tela.",
    ),
    (
        "From what? ...that's for the next cycle.",
        "Do quê? ...isso fica para o próximo ciclo.",
    ),
    (
        "Frozen enemies take double damage.",
        "Inimigos congelados levam dano em dobro.",
    ),
    ("Full Arsenal", "Arsenal Completo"),
    ("Full Fleet", "Frota Completa"),
    ("Full Hangar", "Hangar Cheio"),
    ("Fullscreen", "Tela cheia"),
    ("GALAXY HERO", "HERÓI DA GALÁXIA"),
    ("GAME", "JOGO"),
    ("GAME MODE", "MODO DE JOGO"),
    ("GAME OVER", "FIM DE JOGO"),
    ("GAMEPAD", "CONTROLE"),
    ("GAMEPAD VIBRATION", "VIBRAÇÃO DO CONTROLE"),
    ("GOLD", "OURO"),
    ("GOLD RUSH", "FEBRE DO OURO"),
    (
        "GOLD paint and BLACK HOLE weapon",
        "Pintura OURO e arma BURACO NEGRO",
    ),
    ("GOLD trail", "Rastro OURO"),
    ("GOLDEN DIMENSION", "DIMENSÃO DOURADA"),
    ("GOLDEN RECORD", "DISCO DE OURO"),
    ("GOLDEN SCARAB", "ESCARAVELHO DOURADO"),
    ("GOLDEN SCARAB!", "ESCARAVELHO DOURADO!"),
    ("GRAPHICS", "GRÁFICOS"),
    ("GRAVITY TIDE", "MARÉ GRAVITACIONAL"),
    ("GREED", "GANÂNCIA"),
    ("GRENADE", "GRANADA"),
    ("GUNNER", "ATIRADOR"),
    ("Gain 1 bomb every sector.", "Ganha 1 bomba a cada setor."),
    (
        "Gamepad: A fire · X bomb · Y ability",
        "Controle: A atira · X bomba · Y habilidade",
    ),
    (
        "Get PERFECT in 5 minigames.",
        "Consiga PERFEITO em 5 minigames.",
    ),
    (
        "Glows around shots, beams and explosions.",
        "Brilho em volta de tiros, raios e explosões.",
    ),
    ("Golden Dimension", "Dimensão Dourada"),
    ("Golden Multiplier", "Multiplicador Dourado"),
    ("Golden Record", "Disco de Ouro"),
    ("Golden ones are worth more.", "Dourados valem mais."),
    ("Good Samaritan", "Bom Samaritano"),
    ("Good luck, pilot.", "Boa sorte, piloto."),
    ("Grab it before it's lost.", "Pegue antes que ela se perca."),
    ("Grazes", "Raspões"),
    (
        "Grazes give double points and energy.",
        "Raspões valem o dobro de pontos e de energia.",
    ),
    (
        "Grazing enemy shots fills the ability bar.",
        "Raspar nos tiros inimigos enche a barra de habilidade.",
    ),
    ("HARD", "DIFÍCIL"),
    ("HATCHLING", "FILHOTE"),
    ("HEALER", "CURANDEIRO"),
    ("HEART OF THE SWARM", "CORAÇÃO DO ENXAME"),
    ("HELLO, CREATOR!", "OLÁ, CRIADOR!"),
    ("HIGH", "ALTO"),
    ("HIGH SCORE", "RECORDE"),
    ("HIGH SCORES", "RECORDES"),
    ("HIVE", "COLMEIA"),
    ("HIVE QUEEN", "RAINHA DA COLMEIA"),
    ("HONOR INSIGNIAS", "INSÍGNIAS DE HONRA"),
    ("HONORS", "HONRAS"),
    ("HOW TO PLAY", "COMO JOGAR"),
    (
        "Half of the curses can't touch you anymore.",
        "Metade das maldições não te pega mais.",
    ),
    (
        "Half of the curses can't touch you.",
        "Metade das maldições não te pega.",
    ),
    ("Heart of the Swarm", "Coração do Enxame"),
    (
        "Heavy and patient: +1 life and 10% stronger shots.",
        "Pesada e paciente: +1 vida e tiros 10% mais fortes.",
    ),
    ("Heavy ship for 8 seconds.", "Nave pesada por 8 segundos."),
    (
        "Helix shots that undulate across the screen.",
        "Tiros em hélice que ondulam pela tela.",
    ),
    (
        "High combos multiply your points. Don't let them cool down.",
        "Combos altos multiplicam os pontos. Não deixe esfriar.",
    ),
    ("Hired", "Contratado"),
    ("Historian", "Historiador"),
    (
        "Hit every enemy in a bonus stage.",
        "Acerte todos os inimigos de um estágio bônus.",
    ),
    ("Hit the targets with SPACE.", "Acerte os alvos com ESPAÇO."),
    (
        "Hold 300 crystals at once.",
        "Tenha 300 cristais ao mesmo tempo.",
    ),
    (
        "Hold SHIFT to see exactly where your ship can be hit.",
        "Segure SHIFT para ver exatamente onde a nave pode ser atingida.",
    ),
    (
        "Hold and release . charged shot",
        "Segure e solte ..... tiro carregado",
    ),
    ("Hold fire and release", "Segure o tiro e solte"),
    (
        "Hold fire and release: it pierces everything.",
        "Segure o tiro e solte: atravessa tudo.",
    ),
    (
        "Hold fire and release: the charged shot pierces the formation.",
        "Segure o tiro e solte: o tiro carregado atravessa a formação.",
    ),
    (
        "Hold fire to shoot nonstop.",
        "Segure o tiro para atirar sem parar.",
    ),
    (
        "Homing bees. The Swarm against the Swarm.",
        "Abelhas teleguiadas. O Enxame contra o Enxame.",
    ),
    ("Horizon", "Horizonte"),
    ("Hunter", "Caçador"),
    ("ICE", "GELO"),
    ("ICE BELT", "CINTURÃO DE GELO"),
    ("ICE paint", "Pintura GELO"),
    ("ICE trail", "Rastro GELO"),
    (
        "IN MEMORY OF THE STARDUST PROBE",
        "EM MEMÓRIA DA SONDA STARDUST",
    ),
    ("INSIGNIA ???", "INSÍGNIA ???"),
    (
        "INSPIRED BY THE SHAREWARE GAMES",
        "INSPIRADO PELOS SHAREWARES",
    ),
    ("INVINCIBLE", "INVENCÍVEL"),
    ("INVINCIBLE!", "INVENCÍVEL!"),
    ("ION STORM", "TEMPESTADE IÔNICA"),
    ("Ice Age", "Era do Gelo"),
    (
        "If I fall, the hive scatters and survives. If they fall, no one will hold the door anymore.",
        "Se eu cair, a colmeia se espalha e sobrevive. Se ele cair, ninguém mais vai segurar a porta.",
    ),
    (
        "In Expedition, every blessing changes your strategy.",
        "Na Expedição, cada bênção muda a sua estratégia.",
    ),
    (
        "In the Ice Belt your ship slides. Plan ahead.",
        "No Cinturão de Gelo, sua nave escorrega. Antecipe-se.",
    ),
    (
        "In the ashes of the hive, one last truth:",
        "Nas cinzas da colmeia, uma última verdade:",
    ),
    (
        "In the dust there was a signal. And the signal called the SWARM.",
        "Na poeira havia um sinal. E o sinal chamou o ENXAME.",
    ),
    ("Inheritance", "Herança"),
    (
        "Instant beam: hits the whole column and ignores shields.",
        "Feixe instantâneo: acerta a coluna inteira e ignora escudos.",
    ),
    ("Investor", "Investidor"),
    ("It looks peaceful...", "Ele parece pacífico..."),
    (
        "It wants what's falling. Shoot it down!",
        "Ele quer o que está caindo. Derrube-o!",
    ),
    (
        "It went looking for comet dust",
        "Que buscou poeira de cometa",
    ),
    (
        "Its living mirror opens and closes. While open, it throws your own shots back at you.",
        "Seu espelho vivo abre e fecha. Aberto, devolve seus próprios tiros.",
    ),
    ("JAM", "PANE"),
    ("JAM!", "PANE!"),
    ("JUMPER", "SALTADOR"),
    ("Jinxed", "Azarado"),
    ("KEYBOARD", "TECLADO"),
    ("Keen Eye", "Olho Clínico"),
    ("Keeper of Relics", "Guardião das Relíquias"),
    ("Konami Code", "Código Konami"),
    ("LANGUAGE", "IDIOMA"),
    ("LASER ONLY", "SÓ LASER"),
    ("LASER SIGHT", "MIRA LASER"),
    ("LB precision · Start pause", "LB precisão · Start pausa"),
    ("LEADER", "LÍDER"),
    ("LEFT", "ESQUERDA"),
    (
        "LEFT/RIGHT change mode  ·  ENTER back",
        "ESQ/DIR troca o modo  ·  ENTER volta",
    ),
    (
        "LEFT/RIGHT choose · ENTER confirm",
        "ESQ/DIR escolhe · ENTER confirma",
    ),
    ("LEFT/RIGHT switch tab", "ESQ/DIR troca de aba"),
    (
        "LEFT/RIGHT tabs  ·  UP/DOWN browse  ·  ESC back",
        "ESQ/DIR abas  ·  CIMA/BAIXO navega  ·  ESC volta",
    ),
    ("LEFT/RIGHT to switch.", "ESQ/DIR para trocar."),
    ("LEGENDARY", "LENDÁRIA"),
    (
        "LEGENDARY paint and a golden name in the high scores",
        "Pintura LENDÁRIA e nome dourado no ranking",
    ),
    (
        "LEGENDARY paint for every ship",
        "Pintura LENDÁRIA para todas as naves",
    ),
    ("LETTERS", "LETRAS"),
    ("LETTERS S-T-A-R-D-U-S-T", "LETRAS S-T-A-R-D-U-S-T"),
    ("LIEUTENANT", "TENENTE"),
    ("LIEUTENANT COLONEL", "TENENTE-CORONEL"),
    ("LIGHT TRIGGER", "GATILHO LEVE"),
    ("LOCK OPENED!", "TRAVA ABERTA!"),
    ("LOCKED", "BLOQUEADO"),
    ("LOG 1 - THE QUEEN", "REGISTRO 1 - A RAINHA"),
    ("LOG 10 - THE COW", "REGISTRO 10 - A VACA"),
    ("LOG 11 - THE MERCHANT", "REGISTRO 11 - O MERCADOR"),
    ("LOG 12 - AFTERWARD", "REGISTRO 12 - DEPOIS"),
    ("LOG 2 - THE DREADNOUGHT", "REGISTRO 2 - O COURAÇADO"),
    ("LOG 3 - THE SERPENT", "REGISTRO 3 - A SERPENTE"),
    ("LOG 4 - THE WEAVER", "REGISTRO 4 - A TECELÃ"),
    ("LOG 5 - THE EYE", "REGISTRO 5 - O OLHO"),
    ("LOG 6 - THE OMEGA", "REGISTRO 6 - O ÔMEGA"),
    ("LOG 7 - THE TWINS", "REGISTRO 7 - AS GÊMEAS"),
    ("LOG 8 - THE HEART", "REGISTRO 8 - O CORAÇÃO"),
    ("LOG 9 - GOLDEN DIMENSION", "REGISTRO 9 - DIMENSÃO DOURADA"),
    ("LOG ???", "REGISTRO ???"),
    ("LOGS", "REGISTROS"),
    ("LOOT", "SAQUE"),
    ("LOW", "BAIXO"),
    ("LOW ORBIT", "ÓRBITA BAIXA"),
    ("Last Breath", "Último Suspiro"),
    (
        "Left becomes right. Good luck.",
        "Esquerda vira direita. Boa sorte.",
    ),
    (
        "Let the Merchant pass in peace three times.",
        "Deixe o Mercador passar em paz três vezes.",
    ),
    (
        "Let the game play itself in demo mode.",
        "Deixe o jogo rodar sozinho na demonstração.",
    ),
    ("Light Fingers", "Mão Leve"),
    ("Light Trigger", "Gatilho Leve"),
    ("Lightning Bomb", "Bomba Relâmpago"),
    ("Lightning Wave", "Onda Relâmpago"),
    (
        "Like the arcades: only the Blaster, no shop, no cards. Endless waves.",
        "Como nos fliperamas: só a Rajada, sem loja, sem cartas. Ondas sem fim.",
    ),
    (
        "Live through all 6 sector events.",
        "Viva os 6 eventos de setor.",
    ),
    ("Living Crystal", "Cristal Vivo"),
    (
        "Logs drop from campaign bosses and from",
        "Registros caem dos chefes da campanha e de",
    ),
    ("Long Combo", "Combo Longo"),
    ("Long Fever", "Febre Longa"),
    ("Longing", "Saudade"),
    (
        "Looks like a capsule. It isn't. When you get close, it opens its mouth.",
        "Parece uma cápsula. Não é. Quando você chega perto, ele abre a boca.",
    ),
    (
        "Losing a ship doesn't lower your weapon level.",
        "Perder uma nave não reduz o nível da arma.",
    ),
    ("Lucky", "Sortudo"),
    ("Lucky Crystal", "Cristal da Sorte"),
    ("Lucrative Contract", "Contrato Lucrativo"),
    ("MADE WITH", "FEITO EM"),
    ("MAGNET", "ÍMÃ"),
    ("MAX", "MÁXIMO"),
    ("MEDIUM", "MÉDIO"),
    ("MERCENARY", "MERCENÁRIO"),
    ("MERCHANT", "MERCADOR"),
    ("MERCHANT'S GIFT", "PRESENTE DO MERCADOR"),
    ("MERRY CHRISTMAS, PILOT!", "FELIZ NATAL, PILOTO!"),
    ("METEOR", "METEORO"),
    ("METEOR SHOWER", "CHUVA DE METEOROS"),
    ("MIDSHIPMAN", "ASPIRANTE"),
    ("MIMIC", "MÍMICO"),
    ("MIMIC!", "MÍMICO!"),
    ("MINE", "MINA"),
    ("MINELAYER", "MINADOR"),
    ("MINES", "MINAS"),
    ("MIRROR", "ESPELHO"),
    ("MIRROR!", "ESPELHO!"),
    ("MISSED!", "ERROU!"),
    ("MISSILE", "MÍSSIL"),
    ("MUSIC", "MÚSICA"),
    ("MYSTERY BOX", "CAIXA MISTERIOSA"),
    ("MYSTERY CARD", "CARTA MISTERIOSA"),
    (
        "Made for two: +1 life and a star fan every 6 shots.",
        "Feita para dois: +1 vida e um leque de estrelas a cada 6 tiros.",
    ),
    (
        "Make 100 grazes in one run.",
        "Faça 100 raspões numa partida.",
    ),
    ("Make a 100 combo.", "Faça um combo de 100."),
    ("Make a 50 combo.", "Faça um combo de 50."),
    ("Marathon", "Maratona"),
    (
        "Masked and fast. Steals what falls and flees. Shoot it down to get it back doubled.",
        "Mascarado e rápido. Rouba o que cai na tela e foge. Derrube-o para recuperar em dobro.",
    ),
    (
        "Memorize the stars that light up",
        "Memorize as estrelas que acendem",
    ),
    ("Merchants spared", "Mercadores poupados"),
    ("Millionaire", "Milionário"),
    (
        "Millions of star insects surrounded Earth in attack formation.",
        "Milhões de insetos estelares cercaram a Terra em formação de ataque.",
    ),
    (
        "Mimics show up marked in red.",
        "Mímicos aparecem marcados em vermelho.",
    ),
    (
        "Mimics: capsules that bite.",
        "Mímicos: cápsulas que mordem.",
    ),
    (
        "Minelayers drop mines: shoot them.",
        "Minadores largam minas: atire nelas.",
    ),
    (
        "Minelayers, Jumpers and a tamed Comet",
        "Minadores, Saltadores e um Cometa domado",
    ),
    (
        "Mines blink faster and faster before exploding.",
        "Minas piscam cada vez mais rápido antes de explodir.",
    ),
    (
        "Minigame rifts drop 2.5x more often.",
        "Fendas de minigame caem 2,5x mais.",
    ),
    (
        "Minigame rifts drop 2x more often.",
        "Fendas de minigame caem 2x mais.",
    ),
    ("Minigames won", "Minigames vencidos"),
    ("Moo!", "Muuu!"),
    (
        "More power-ups until the end of the run.",
        "Mais power-ups até o fim da partida.",
    ),
    (
        "Mother of the whole Swarm. Her claws fire on their own and she calls minions.",
        "Mãe de todo o Enxame. Suas garras disparam sozinhas e ela chama lacaios.",
    ),
    ("Move", "Mover"),
    (
        "My daughters fight together. I told them not to hate the pilot. The pilot just doesn't know.",
        "Minhas filhas lutam juntas. Disse a elas para não odiarem o piloto. Ele só não sabe.",
    ),
    ("NAME", "NOME"),
    ("NEEDLE", "AGULHA"),
    ("NEW BESTIARY ENTRY", "NOVO REGISTRO NO BESTIÁRIO"),
    ("NEW FLIGHT LOG", "NOVO REGISTRO DE BORDO"),
    ("NEW HIGH SCORE!", "NOVO RECORDE!"),
    ("NIGHTMARE", "PESADELO"),
    ("NO", "NÃO"),
    ("NO SLOT: UNEQUIP ANOTHER", "SEM ESPAÇO: DESEQUIPE OUTRA"),
    ("NO TRAIL", "SEM RASTRO"),
    ("NORMAL", "NORMAIS"),
    ("NOT THIS TIME", "NÃO FOI DESSA VEZ"),
    (
        "Never stays in the same spot of the formation. Vanishes in a blink and reappears far away.",
        "Nunca fica no mesmo lugar da formação. Some num piscar e reaparece longe.",
    ),
    ("Next time, be faster.", "Da próxima vez, seja mais rápido."),
    ("Nightmare Awakened", "Pesadelo Desperto"),
    (
        "No ship lost... the Swarm never saw you coming.",
        "Nenhuma nave perdida... o Enxame não te viu chegar.",
    ),
    (
        "Nobody knows how it got to space. Nobody dares to ask.",
        "Ninguém sabe como ela chegou ao espaço. Ninguém tem coragem de perguntar.",
    ),
    (
        "Nobody knows what the Swarm wants. Good luck, pilot.",
        "Ninguém sabe o que o Enxame quer. Boa sorte, piloto.",
    ),
    (
        "Non-stop waves, faster and faster. How long can you last?",
        "Ondas sem pausa, cada vez mais rápidas. Quanto você aguenta?",
    ),
    ("Nostalgic", "Saudosista"),
    (
        "Not a single shot fired. Respect.",
        "Nenhum tiro disparado. Respeito.",
    ),
    (
        "Not found yet. Keep playing.",
        "Ainda não encontrada. Continue jogando.",
    ),
    (
        "Now the hive lives, and you know the truth.",
        "Agora a colmeia vive, e você sabe a verdade.",
    ),
    ("OMEGA DREADNOUGHT", "COURAÇADO ÔMEGA"),
    ("OPTIONS", "OPÇÕES"),
    ("ORBIT", "ÓRBITA"),
    ("OUTER HIVE", "COLMEIA EXTERIOR"),
    ("OVERDRIVE", "SOBRECARGA"),
    ("OWNED", "JÁ TEM"),
    (
        "Off: the background barely moves.",
        "Desligado: o fundo quase não se mexe.",
    ),
    (
        "Once per run, revives when you lose your last life.",
        "Uma vez por partida, renasce ao perder a última vida.",
    ),
    (
        "Only one life, but points x3.",
        "Uma vida só, mas pontos x3.",
    ),
    (
        "Only the Laser exists, but it starts at level 3.",
        "Só existe Laser, mas ele começa no nível 3.",
    ),
    (
        "Only the center of your ship is vulnerable.",
        "Só o centro da nave é vulnerável.",
    ),
    ("Open 5 secret reliquaries.", "Abra 5 relicários secretos."),
    ("Open Rift", "Fenda Aberta"),
    (
        "Open the reliquary before the rift closes!",
        "Abra o relicário antes que a fenda se feche!",
    ),
    ("P1", "J1"),
    (
        "P1: A D + Space · P2: arrows + , . /",
        "J1: A D + Espaço · J2: setas + , . /",
    ),
    (
        "P1: gamepad 1 · P2: gamepad 2",
        "J1: controle 1 · J2: controle 2",
    ),
    ("P1: keyboard · P2: gamepad", "J1: teclado · J2: controle"),
    ("P2", "J2"),
    ("PACIFIST", "PACIFISTA"),
    ("PARTICLES", "PARTÍCULAS"),
    ("PARTNER RESCUED!", "PARCEIRO RESGATADO!"),
    ("PAUSE", "PAUSA"),
    ("PERFECT!", "PERFEITO!"),
    ("PERFECTIONIST INSIGNIA", "INSÍGNIA DO PERFECCIONISTA"),
    ("PHANTOM", "FANTASMA"),
    ("PHANTOM ship", "Nave FANTASMA"),
    (
        "PHANTOM ship and RAILGUN weapon",
        "Nave FANTASMA e arma CANHÃO ELÉTRICO",
    ),
    ("PHOENIX!", "FÊNIX!"),
    ("PILOT", "PILOTO"),
    ("PLASMA trail", "Rastro PLASMA"),
    ("PLAY", "JOGAR"),
    ("PLAYER", "JOGADOR"),
    ("PLAYERS: 1", "JOGADORES: 1"),
    ("PLAYERS: 2 (CO-OP)", "JOGADORES: 2 (CO-OP)"),
    ("POWER-UPS (falling orbs)", "PODERES (bolinhas que caem)"),
    ("PRECISION", "PRECISÃO"),
    ("PRESS...", "APERTE..."),
    ("PRISM", "PRISMA"),
    ("PROBE", "SONDA"),
    (
        "PROBE ship and STARDUST trail",
        "Nave SONDA e rastro POEIRA ESTELAR",
    ),
    ("PROGRESS ERASED", "PROGRESSO APAGADO"),
    ("PROMOTED!", "PROMOVIDO!"),
    ("PROTOTYPE", "PROTÓTIPO"),
    (
        "PROTOTYPE ship unlocked in the Hangar. +1 secret card.",
        "Nave PROTÓTIPO liberada no Hangar. +1 carta secreta.",
    ),
    (
        "PULSAR drops from capsules and its stars are worth double.",
        "A PULSAR cai das cápsulas e as estrelas valem o dobro.",
    ),
    ("Pacifist", "Pacifista"),
    (
        "Panels with score and weapon on the sides (wide screens).",
        "Painéis com pontos e arma nas laterais (telas largas).",
    ),
    (
        "Part of the Swarm comes from below!",
        "Parte do Enxame chega por baixo!",
    ),
    ("Pause", "Pausa"),
    ("Perfection", "Perfeição"),
    ("Perfectionist", "Perfeccionista"),
    ("Perfectionist Insignia", "Insígnia do Perfeccionista"),
    ("Persistent", "Insistente"),
    ("Phoenix", "Fênix"),
    (
        "Pick up 10 curses. On purpose?",
        "Pegue 10 maldições. De propósito?",
    ),
    (
        "Picking up the same weapon again raises its level.",
        "Pegar a mesma arma de novo sobe o nível dela.",
    ),
    ("Piggy Bank", "Cofrinho"),
    (
        "Pixel art written letter by letter",
        "Pixel art escrita letra por letra",
    ),
    (
        "Plasma orbs that grow and burn through six enemies.",
        "Orbes de plasma que crescem e atravessam seis inimigos.",
    ),
    (
        "Plated from head to claw. Frontal shots only chip the armor: hit it from behind or blow it up.",
        "Blindado da cabeça às garras. Tiros de frente só lascam a armadura: acerte por trás ou exploda.",
    ),
    ("Play 25 runs.", "Jogue 25 partidas."),
    ("Play for 2 hours in total.", "Jogue por 2 horas no total."),
    ("Play time", "Tempo de jogo"),
    (
        "Points x1.3 and enemy shots 10% slower.",
        "Pontos x1,3 e tiros inimigos 10% mais lentos.",
    ),
    (
        "Points x1.5 and max combo x10.",
        "Pontos x1,5 e combo máximo x10.",
    ),
    ("Portable Magnet", "Ímã Portátil"),
    ("Portuguese or English.", "Português ou inglês."),
    ("Power-ups drop 50% more often.", "Power-ups caem 50% mais."),
    ("Precision mode", "Modo preciso"),
    (
        "Press SPACE when the needle",
        "Aperte ESPAÇO quando a agulha",
    ),
    (
        "Press a key or button · ESC cancels",
        "Aperte uma tecla ou botão · ESC cancela",
    ),
    (
        "Pricier weapons in the shop hit much harder.",
        "Armas mais caras na loja batem muito mais forte.",
    ),
    (
        "Pulls crystals from farther away.",
        "Puxa cristais de mais longe.",
    ),
    ("Pyromaniac", "Piromaníaco"),
    (
        "Q/TAB switch tab  ·  LEFT/RIGHT adjust  ·  ESC back",
        "Q/TAB troca de aba  ·  ESQ/DIR ajusta  ·  ESC volta",
    ),
    ("QUIT", "SAIR"),
    ("Queen Bee", "Rainha das Abelhas"),
    ("Queen's Crown", "Coroa da Rainha"),
    (
        "Queen. Each sector has 3 waves, a",
        "Suprema. Cada setor tem 3 ondas, um",
    ),
    ("Quick Overdrive", "Sobrecarga Rápida"),
    ("RAILGUN", "CANHÃO ELÉTRICO"),
    ("RAINBOW", "ARCO-ÍRIS"),
    ("RAINBOW trail", "Rastro ARCO-ÍRIS"),
    ("RANKS", "PATENTES"),
    ("RARE", "RARA"),
    ("REAR ADMIRAL", "CONTRA-ALMIRANTE"),
    ("RECOVERED TWICE OVER!", "RECUPERADO EM DOBRO!"),
    ("RECRUIT", "RECRUTA"),
    ("REFLECT", "REFLEXO"),
    ("RELIC", "RELÍQUIA"),
    ("RELIC DISCOVERED!", "RELÍQUIA DESCOBERTA!"),
    ("RELIC INSIGNIA", "INSÍGNIA DA RELÍQUIA"),
    (
        "RELIC. A beam of sunlight that crosses the whole screen.",
        "RELÍQUIA. Um feixe de sol que atravessa a tela inteira.",
    ),
    (
        "RELIC. A cloud of cosmic dust that hunts everything.",
        "RELÍQUIA. Uma nuvem de poeira estelar que caça tudo.",
    ),
    (
        "RELIC. Black holes in a row and shots that bend space.",
        "RELÍQUIA. Buracos negros em série e tiros que dobram o espaço.",
    ),
    (
        "RELIC. Piercing stars and ring explosions.",
        "RELÍQUIA. Estrelas perfurantes e explosões em anel.",
    ),
    ("RELICS", "RELÍQUIAS"),
    ("REPLAY THE PROLOGUE", "REVER O PRÓLOGO"),
    ("REROLL OFFERS (FREE)", "TROCAR OFERTAS (GRÁTIS)"),
    ("RESCUE YOUR PARTNER", "RESGATE SEU PARCEIRO"),
    ("RESET THIS PLAYER", "RESTAURAR ESTE JOGADOR"),
    ("RICOCHET", "RICOCHETE"),
    ("RIFT", "FENDA"),
    ("RIFTS", "FENDAS"),
    ("RIFTS (hexagon with ?)", "FENDAS (hexágono com ?)"),
    ("RIGHT", "DIREITA"),
    ("RUBY", "RUBI"),
    ("RUBY paint and ORBIT weapon", "Pintura RUBI e arma ÓRBITA"),
    ("RUST", "FERRUGEM"),
    ("RUST!", "FERRUGEM!"),
    (
        "Raises a shot-blocking barrier for 5 s.",
        "Ergue por 5 s uma barreira que bloqueia tiros.",
    ),
    (
        "Ranks that don't come from XP: they come from feats.",
        "Patentes que não vêm de XP: vêm de feitos.",
    ),
    (
        "Rare drops; bosses always drop one.",
        "Caem raramente; chefes sempre soltam.",
    ),
    (
        "Rare feats are remembered here.",
        "Feitos raros são lembrados aqui.",
    ),
    (
        "Rare shop items and the HIVE weapon",
        "Itens raros na loja e arma COLMEIA",
    ),
    (
        "Reach level 50 of the campaign.",
        "Chegue ao nível 50 da campanha.",
    ),
    (
        "Reach sector 10 in Expedition.",
        "Chegue ao setor 10 da Expedição.",
    ),
    ("Reach the rank SERGEANT.", "Alcance a patente SARGENTO."),
    (
        "Reach wave 30 in Survival mode.",
        "Chegue à onda 30 no modo Sobrevivência.",
    ),
    ("Reactive Shield", "Escudo Reativo"),
    ("Recharge", "Recarga"),
    ("Regicide", "Regicida"),
    ("Regular Customer", "Freguês"),
    ("Reinforced Hull", "Casco Reforçado"),
    ("Reliquary", "Relicário"),
    (
        "Remap keys: SETTINGS > CONTROLS",
        "Trocar teclas: CONFIGURAÇÕES > CONTROLES",
    ),
    (
        "Reward: crystals, a weapon level and a power-up.",
        "Prêmio: cristais, nível de arma e um poder.",
    ),
    ("Rock Salt", "Sal Grosso"),
    (
        "Rock peaks split your lane.",
        "Picos de rocha dividem sua pista.",
    ),
    (
        "Rocks rain down all over the sector.",
        "Pedras despencam por todo o setor.",
    ),
    ("Routine", "Rotina"),
    (
        "Rumble on explosions and when your ship is hit.",
        "Vibra em explosões e quando sua nave é atingida.",
    ),
    ("Runs", "Partidas"),
    (
        "S  Shield   B  Bomb   $  Crystals",
        "S  Escudo   B  Bomba   $  Cristais",
    ),
    ("SCORE", "PONTOS"),
    ("SCREEN SHAKE", "TREMOR DE TELA"),
    ("SECRET ROOM", "SALA SECRETA"),
    ("SECTOR", "SETOR"),
    ("SECTOR ???", "SETOR ???"),
    ("SECTOR CLEAR!", "SETOR LIMPO!"),
    ("SECTOR EVENTS", "EVENTOS DE SETOR"),
    ("SERGEANT", "SARGENTO"),
    ("SETTINGS", "CONFIGURAÇÕES"),
    ("SHADOW", "SOMBRA"),
    ("SHADOW paint", "Pintura SOMBRA"),
    ("SHIELD", "ESCUDO"),
    ("SHIELD RESTORED", "ESCUDO REGENERADO"),
    ("SHIELD!", "ESCUDO!"),
    ("SHIELDBEARER", "ESCUDEIRO"),
    ("SHIP", "NAVE"),
    ("SHIP LOCKED", "BLOQUEADA"),
    ("SHIPS", "NAVES"),
    ("SHOOT", "ATIRAR"),
    ("SHOP", "LOJA"),
    ("SHOP AND POWER", "LOJA E PODER"),
    ("SHOW FPS", "MOSTRAR FPS"),
    ("SHRAPNEL", "ESTILHAÇO"),
    (
        "SHRAPNEL weapon and SURVIVAL mode",
        "Arma ESTILHAÇO e modo SOBREVIVÊNCIA",
    ),
    ("SIDE CANNONS", "CANHÕES LATERAIS"),
    ("SIDE PANELS", "PAINÉIS LATERAIS"),
    ("SIGNAL RECEIVED", "SINAL CAPTADO"),
    ("SINGULARITY", "SINGULARIDADE"),
    ("SKULL", "CAVEIRA"),
    ("SKULL!", "CAVEIRA!"),
    (
        "SKULLS (red bubbles): dodge them!",
        "CAVEIRAS (bolhas vermelhas): desvie!",
    ),
    ("SLOTS:", "ESPAÇOS:"),
    ("SLOW", "LENTO"),
    ("SLOW MOTION!", "CÂMERA LENTA!"),
    ("SOFT", "SUAVES"),
    ("SOLAR LANCE", "LANÇA SOLAR"),
    ("SOUND FX", "EFEITOS"),
    ("SOUNDTRACK", "TRILHA SONORA"),
    ("SPACE PIRATE", "PIRATA ESPACIAL"),
    ("SPACE WARP!", "SALTO ESPACIAL!"),
    ("SPACE to lock", "ESPAÇO para travar"),
    ("SPECIAL THANKS", "AGRADECIMENTO ESPECIAL"),
    ("SPEED", "VELOCIDADE"),
    ("SPLITTER", "DIVISOR"),
    ("STAR BANK", "BANCO ESTELAR"),
    ("STAR CANNON", "CANHÃO ESTELAR"),
    ("STAR LEGEND", "LENDA ESTELAR"),
    ("STAR MARSHAL", "MARECHAL ESTELAR"),
    ("STAR PULSE", "PULSO ESTELAR"),
    ("STAR RIFT", "FENDA ESTELAR"),
    ("STAR SEQUENCE", "SEQUÊNCIA DAS ESTRELAS"),
    ("STAR VAULT", "COFRE ESTELAR"),
    (
        "STARDUST words / Motherships",
        "Palavras STARDUST / Naves-Mãe",
    ),
    ("STATS", "NÚMEROS"),
    ("STOLEN!", "ROUBADO!"),
    ("SUPER MAGNET", "SUPER ÍMÃ"),
    ("SUPER MAGNET!", "SUPER ÍMÃ!"),
    ("SUPREME INSIGNIA", "INSÍGNIA SUPREMA"),
    ("SUPREME QUEEN", "RAINHA SUPREMA"),
    ("SUPREME QUEEN OF THE SWARM", "RAINHA SUPREMA DO ENXAME"),
    ("SURVIVAL", "SOBREVIVÊNCIA"),
    (
        "Same weapon levels up · a new one replaces it",
        "Mesma arma sobe de nível · outra arma substitui a atual",
    ),
    (
        "Saws spin around you and block shots.",
        "Serras giram em volta e bloqueiam tiros.",
    ),
    (
        "Score 1,000,000 points in one run.",
        "Faça 1.000.000 de pontos numa partida.",
    ),
    (
        "Score 60,000 points in Classic mode.",
        "Faça 60.000 pontos no modo Clássico.",
    ),
    ("Scrapyard", "Ferro-Velho"),
    ("Scrooge", "Tio Patinhas"),
    ("Secret achievement.", "Conquista secreta."),
    (
        "Secret card: Cosmic Calf.",
        "Carta secreta: Bezerro Cósmico.",
    ),
    ("Sharp Ricochet", "Ricochete Afiado"),
    (
        "Shieldbearers only protect their front. Diagonal shots get through.",
        "Escudeiros só protegem a frente. Tiros em diagonal passam.",
    ),
    (
        "Shift .............. precision mode",
        "Shift .............. modo preciso",
    ),
    ("Ship 12% faster.", "Nave 12% mais rápida."),
    ("Ship 8% faster.", "Nave 8% mais rápida."),
    ("Ships lost", "Naves perdidas"),
    (
        "Shoot down 10 Mimics in total.",
        "Abata 10 Mímicos no total.",
    ),
    (
        "Shoot down 150 Kamikazes in total.",
        "Abata 150 Kamikazes no total.",
    ),
    (
        "Shoot down 300 enemies with the Flame.",
        "Abata 300 inimigos com a Chama.",
    ),
    (
        "Shoot down 5 enemies with a single charged shot.",
        "Abata 5 inimigos com um só tiro carregado.",
    ),
    (
        "Shoot down 50 elite enemies in total.",
        "Abata 50 inimigos de elite no total.",
    ),
    (
        "Shoot down a Golden Scarab.",
        "Abata um Escaravelho Dourado.",
    ),
    (
        "Shoot down a Thief carrying its loot.",
        "Abata um Ladrão carregando o que roubou.",
    ),
    (
        "Shoot down the Cosmic Cow 3 times.",
        "Abata a Vaca Cósmica 3 vezes.",
    ),
    ("Shoot down the Cosmic Cow.", "Abata a Vaca Cósmica."),
    (
        "Shoot down the mysterious Mothership.",
        "Derrube a Nave-Mãe misteriosa.",
    ),
    ("Shoot one down to record it.", "Abata um para registrar."),
    ("Shoot them or dodge them!", "Atire neles ou desvie!"),
    (
        "Shoots sticky webs that slow your ship down for a few seconds.",
        "Dispara teias pegajosas que deixam sua nave lenta por alguns segundos.",
    ),
    (
        "Short-range flamethrower.",
        "Lança-chamas de curto alcance.",
    ),
    ("Shortcut", "Atalho"),
    (
        "Shotgun: huge damage up close.",
        "Escopeta: muito dano de perto.",
    ),
    (
        "Shots that bounce off the walls.",
        "Tiros que quicam nas paredes.",
    ),
    (
        "Skulls fall slowly and follow you. Three shots finish them.",
        "Caveiras caem devagar e te seguem. Três tiros acabam com elas.",
    ),
    ("Sleepless", "Insone"),
    ("Slow Time", "Tempo Lento"),
    (
        "Slow and armored: +1 life and starts with a shield.",
        "Lenta e blindada: +1 vida e começa com escudo.",
    ),
    ("Snake Charmer", "Encantador de Serpentes"),
    (
        "Soft: explosion flashes much weaker.",
        "Suaves: clarões de explosão bem mais fracos.",
    ),
    (
        "Something happens if you type the game's name on the title screen.",
        "Algo acontece se você digitar o nome do jogo no título.",
    ),
    (
        "Something pulls the ship inside...",
        "Algo puxa a nave para dentro...",
    ),
    (
        "Space / Z .......... fire (tap)",
        "Espaço / Z ......... atirar (aperte)",
    ),
    ("Space Pirate", "Pirata Espacial"),
    ("Spare Battery", "Bateria Extra"),
    (
        "Speed +10% for the rest of the expedition.",
        "Velocidade +10% pelo resto da expedição.",
    ),
    ("Speedrunner", "Velocista"),
    ("Speller", "Soletrador"),
    (
        "Spend your crystals before the next sector",
        "Gaste seus cristais antes do próximo setor",
    ),
    (
        "Splitters, Kamikazes, Shieldbearers, Weavers,",
        "Divisores, Kamikazes, Escudeiros, Tecelãs,",
    ),
    ("Star Hunter", "Caçador de Estrelas"),
    ("Starting Shield", "Escudo Inicial"),
    (
        "Starts every sector in Frenzy.",
        "Começa cada setor em Frenesi.",
    ),
    (
        "Starts the run with the LASER at level 2.",
        "Começa a partida com o LASER no nível 2.",
    ),
    ("Starts with 40 crystals.", "Começa com 40 cristais."),
    (
        "Starts with a shield that absorbs one hit.",
        "Começa com um escudo que absorve um golpe.",
    ),
    (
        "Starts with the ability charged.",
        "Começa com a habilidade carregada.",
    ),
    ("Strong Start", "Começo Forte"),
    ("Strong homing missiles.", "Mísseis teleguiados e fortes."),
    ("Supreme Master", "Mestre Supremo"),
    ("Survive for 12 seconds.", "Sobreviva 12 segundos."),
    ("Survivor", "Sobrevivente"),
    ("Suspicious", "Desconfiado"),
    (
        "Swallow 300 shots with black holes in total.",
        "Engula 300 tiros com buracos negros no total.",
    ),
    (
        "Swallows enemy shots and grinds anything that gets close.",
        "Engole tiros inimigos e tritura quem chega perto.",
    ),
    ("Swarm", "Enxame"),
    (
        "Swarm cargo ship. Doesn't shoot and is packed with crystals.",
        "Nave de carga do Enxame. Não atira e está lotada de cristais.",
    ),
    ("TARGET PRACTICE", "TIRO AO ALVO"),
    ("TARGET SIZE", "TAMANHO DO ALVO"),
    ("TEST SOUND FX", "TESTAR EFEITOS"),
    ("THE AMULET PROTECTED YOU!", "O AMULETO PROTEGEU!"),
    ("THE DEVOURER", "O DEVORADOR"),
    ("THE GOAL", "O OBJETIVO"),
    ("THE GROUND RISES", "O CHÃO SE ERGUE"),
    ("THE LIGHTS GO OUT", "AS LUZES SE APAGAM"),
    ("THE RIFT CLOSED", "A FENDA SE FECHOU"),
    ("THE STAR OPENED", "A ESTRELA SE ABRIU"),
    ("THE SWARM REGROUPS!", "O ENXAME SE REAGRUPA!"),
    ("THE THIEF GOT AWAY", "O LADRÃO FUGIU"),
    ("THE TWINS", "AS GÊMEAS"),
    ("THE VOID BEYOND", "O VAZIO ALÉM"),
    ("THE VOID OPENS", "O VAZIO SE ABRE"),
    ("THIEF", "LADRÃO"),
    ("THIEF!", "LADRÃO!"),
    ("THIN GLASS", "VIDRO FINO"),
    ("THRUSTER", "PROPULSOR"),
    ("THRUSTERS", "PROPULSORES"),
    ("THUNDER", "TROVÃO"),
    ("THUNDER weapon", "Arma TROVÃO"),
    ("TIME WARP", "DOBRA TEMPORAL"),
    ("TIP", "DICA"),
    ("TIP OF THE DAY", "DICA DO DIA"),
    ("TIPS ON PAUSE", "DICAS NA PAUSA"),
    ("TRAIL EQUIPPED", "RASTRO EQUIPADO"),
    ("TREASURE MAP", "MAPA DO TESOURO"),
    ("TRIGGER", "GATILHO"),
    ("TWIN STAR", "ESTRELA GÊMEA"),
    (
        "Take 12 different weapons to level 5.",
        "Leve 12 armas diferentes ao nível 5.",
    ),
    (
        "Take 5 different weapons to level 5.",
        "Leve 5 armas diferentes ao nível 5.",
    ),
    ("Take 5 weapons to level 5.", "Leve 5 armas ao nível 5."),
    (
        "Take two different weapons to level 5 in the same run.",
        "Leve duas armas diferentes ao nível 5 na mesma partida.",
    ),
    ("Tap Space, Z or J", "Aperte Espaço, Z ou J"),
    (
        "Thank you, pilot. The Devourer retreated. The hive will survive. And now you know too.",
        "Obrigada, piloto. O Devorador recuou. A colmeia vai sobreviver. E agora você também sabe.",
    ),
    (
        "Thanks for making this game.",
        "Obrigado por fazer este jogo.",
    ),
    (
        "Thanks for playing to the very end, pilot.",
        "Obrigado por jogar até o fim, piloto.",
    ),
    ("Thanks for playing, pilot.", "Obrigado por jogar, piloto."),
    (
        "Thanks for your kindness, pilot!",
        "Obrigado pela gentileza, piloto!",
    ),
    (
        "The Beetles look... different today.",
        "Os Besouros estão... diferentes hoje.",
    ),
    (
        "The Cow is not ours. It showed up one day and started following us. Since then we call it luck.",
        "A Vaca não é nossa. Ela apareceu um dia e começou a nos seguir. Desde então chamamos ela de sorte.",
    ),
    (
        "The Devourer retreated into the dark between the stars.",
        "O Devorador recuou para o escuro entre as estrelas.",
    ),
    (
        "The Eye only feels pain with its eyelid open.",
        "O Olho só sente dor com a pálpebra aberta.",
    ),
    (
        "The Eye was made to watch the dark. It saw. Since then it can't quite close.",
        "O Olho foi feito para vigiar o escuro. Ele viu. Desde então não consegue mais fechar direito.",
    ),
    (
        "The Hive releases twice as many bees.",
        "A Colmeia solta o dobro de abelhas.",
    ),
    (
        "The Merchant glares at you: prices +30%",
        "O Mercador te olha torto: preços +30%",
    ),
    (
        "The Merchant sells to every side. He knows about the Devourer. He always knew.",
        "O Mercador vende para todos os lados. Ele sabe do Devorador. Ele sempre soube.",
    ),
    (
        "The Merchant will never forget this.",
        "O Mercador nunca vai esquecer isso.",
    ),
    (
        "The Mothership left something behind...",
        "A Nave-Mãe deixou algo para trás...",
    ),
    (
        "The PULSAR weapon now drops from capsules.",
        "A arma PULSAR agora cai das cápsulas.",
    ),
    (
        "The Queen was right: the Swarm only wanted to pass through.",
        "A Rainha estava certa: o Enxame só queria passar.",
    ),
    (
        "The STARDUST probe was coming back from a distant comet",
        "A sonda STARDUST voltava de um cometa distante",
    ),
    (
        "The Serpent was the guardian of the path. It heard first: a silence that swallows stars.",
        "A Serpente era a guardiã do caminho. Ela ouviu primeiro: um silêncio que engole estrelas.",
    ),
    (
        "The Supreme Queen fell without you ever retreating.",
        "A Rainha Suprema caiu sem que você recuasse.",
    ),
    ("The Void Stares Back", "O Vazio Olha de Volta"),
    (
        "The Wild Comet's tail sheds dust. Stay in front of it.",
        "A cauda do Cometa Wild solta poeira. Fique na frente dele.",
    ),
    (
        "The ability bar fills 35% faster.",
        "A barra de habilidade enche 35% mais rápido.",
    ),
    (
        "The base of the Swarm. Harmless alone; in formation, a living wall.",
        "A base do Enxame. Sozinho é inofensivo; em formação, é uma muralha viva.",
    ),
    (
        "The crowned commander. When it falls, the formation scatters into new places in a rage.",
        "O comandante coroado. Quando cai, a formação se embaralha em fúria.",
    ),
    (
        "The first ship ever made for this game. Built from nostalgia.",
        "A primeira nave que existiu neste jogo. Feita de nostalgia.",
    ),
    (
        "The first shop reroll is free.",
        "A primeira troca de ofertas na loja é grátis.",
    ),
    (
        "The fleet fell in hours. What's left: a hidden hangar, one ship and you.",
        "A frota caiu em horas. Sobrou um hangar escondido, uma nave e você.",
    ),
    (
        "The galaxy breathes again... for now.",
        "A galáxia respira aliviada... por enquanto.",
    ),
    (
        "The heart of the Swarm has been destroyed.",
        "O coração do Enxame foi destruído.",
    ),
    (
        "The heart of the Swarm. Crowned, furious and with all her sisters' tricks.",
        "O coração do Enxame. Coroada, furiosa e com todos os truques das irmãs.",
    ),
    (
        "The human probe's dust called to us. I ordered the migration. No time to explain to the natives.",
        "A poeira da sonda humana chamou por nós. Ordenei a migração. Não há tempo para explicar aos nativos.",
    ),
    (
        "The light fades. You can only see near your ship.",
        "A luz some. Só se vê perto da nave.",
    ),
    (
        "The mysterious Mothership shows up twice as often.",
        "A Nave-Mãe misteriosa aparece com o dobro de frequência.",
    ),
    (
        "The pricier the weapon, the stronger it is.",
        "Quanto mais cara a arma, mais forte.",
    ),
    (
        "The reliquary was left behind.",
        "O relicário ficou para trás.",
    ),
    (
        "The shield comes back after 25 s without taking damage.",
        "O escudo volta depois de 25 s sem levar dano.",
    ),
    (
        "The ship of the Stardust mission. 50% bigger crystal magnet.",
        "A nave da missão Stardust. Ímã de cristais 50% maior.",
    ),
    (
        "The shop's DAMAGE CORE makes every weapon stronger.",
        "O NÚCLEO DE DANO da loja deixa todas as armas mais fortes.",
    ),
    ("The weapon lost 2 levels.", "A arma perdeu 2 níveis."),
    ("Thermal Shrapnel", "Estilhaço Térmico"),
    (
        "They don't shoot. Hit them all!",
        "Eles não atiram. Acerte todos!",
    ),
    (
        "They open a quick minigame. Win to earn",
        "Abrem um minigame rápido. Vença e ganhe",
    ),
    (
        "They take your weapon, levels, or jam your ship.",
        "Tiram sua arma, níveis ou travam a nave.",
    ),
    ("Thief Catcher", "Pega Ladrão"),
    ("Thieves are 40% slower.", "Ladrões ficam 40% mais lentos."),
    (
        "Thieves run off with falling items. Shoot them before they vanish.",
        "Ladrões fogem com o que cai. Derrube-os antes que sumam.",
    ),
    (
        "Thieves: they steal falling items. Shoot them!",
        "Ladrões: roubam o que cai. Derrube-os!",
    ),
    (
        "Throw it, it cuts through the whole formation and comes back to your hand.",
        "Vai, corta a formação inteira e volta para a mão.",
    ),
    (
        "To you, who played this far.",
        "A você, que jogou até aqui.",
    ),
    ("Total Eclipse", "Eclipse Total"),
    ("Total experience", "Experiência total"),
    ("Treasurer", "Tesoureiro"),
    ("Triple Fever", "Febre Tripla"),
    ("True Ending", "Final Verdadeiro"),
    ("Turbo Engine", "Motor Turbo"),
    ("Twin Stars", "Estrelas Gêmeas"),
    (
        "Twinkling stars in a fan. Secret.",
        "Estrelas cintilantes em leque. Secreta.",
    ),
    ("Twins Apart", "Gêmeas Separadas"),
    (
        "Two drones fire at your side.",
        "Dois drones atiram ao seu lado.",
    ),
    (
        "Two hive princesses who fight together. When one falls, the other goes berserk.",
        "Duas princesas da colmeia que lutam juntas. Quando uma cai, a outra enlouquece.",
    ),
    (
        "Two probes carry a golden record. Enter their name in the high scores.",
        "Duas sondas levam um disco de ouro. Diga o nome delas no ranking.",
    ),
    (
        "U  Super magnet: pulls every crystal",
        "U  Super ímã: puxa todos os cristais",
    ),
    ("UNTOUCHABLE", "INTOCÁVEL"),
    ("UNTOUCHABLE INSIGNIA", "INSÍGNIA DO INTOCÁVEL"),
    (
        "Ultra-thin, fast needles that pierce everything.",
        "Agulhas finíssimas e velozes que perfuram tudo.",
    ),
    ("Unlock all 10 ships.", "Desbloqueie as 10 naves."),
    (
        "Unlock the 7 classic ships.",
        "Desbloqueie as 7 naves clássicas.",
    ),
    ("Untouchable", "Intocável"),
    ("Use 50 bombs in total.", "Use 50 bombas no total."),
    ("Use a Warp Gate.", "Use um Portal de Salto."),
    ("VALKYRIE", "VALQUÍRIA"),
    ("VANGUARD", "VANGUARDA"),
    ("VEGA ship and GRENADE weapon", "Nave VEGA e arma GRANADA"),
    ("VICE ADMIRAL", "VICE-ALMIRANTE"),
    ("VICTORY!", "VITÓRIA!"),
    ("VOID", "VAZIO"),
    ("VOID INSIGNIA", "INSÍGNIA DO VAZIO"),
    ("VOID trail", "Rastro VAZIO"),
    ("Valkyrie", "Valquíria"),
    (
        "Vanishes and reappears. Can only be hit while visible.",
        "Some e reaparece. Só é atingido quando está visível.",
    ),
    ("Varied Arsenal", "Arsenal Variado"),
    (
        "Very rare. Flees fast, but carries a fortune in crystals.",
        "Raríssimo. Foge rápido, mas carrega uma fortuna em cristais.",
    ),
    ("Victories", "Vitórias"),
    ("WARNING!", "ALERTA!"),
    ("WARP", "DOBRA"),
    ("WARP GATE", "PORTAL DE SALTO"),
    ("WARRANT OFFICER", "SUBOFICIAL"),
    ("WASP", "VESPA"),
    ("WATCH OUT FOR", "CUIDADO COM"),
    ("WATCH...", "OBSERVE..."),
    ("WAVE", "ONDA"),
    ("WAVE CLEAR!", "ONDA LIMPA!"),
    ("WAVE weapon", "Arma ONDA"),
    ("WEAPON", "ARMA"),
    ("WEAPON BOOSTED", "ARMA REFORÇADA"),
    ("WEAPON INSURANCE", "SEGURO DE ARMA"),
    ("WEAPON UPGRADE", "MELHORIA DE ARMA"),
    ("WEAPONS", "ARMAS"),
    ("WEAVER", "TECELÃ"),
    ("WEB", "TEIA"),
    ("WILD COMET", "COMETA WILD"),
    (
        "We hid the hive's treasure between dimensions. Only those who never fall can find the door.",
        "Escondemos o tesouro da colmeia entre dimensões. Só quem nunca cai consegue achar a porta.",
    ),
    (
        "We reinforced the Dreadnought. If the human pilot makes it this far, maybe they're strong enough.",
        "Reforçamos o Couraçado. Se o piloto humano chegar até aqui, talvez seja forte o bastante.",
    ),
    (
        "We took the humans' ship. We needed hull and cannons. Something has been following us for three cycles.",
        "Tomamos a nave dos humanos. Precisávamos de casco e de canhões. Algo nos segue há três ciclos.",
    ),
    (
        "We wove nets between the moons to slow the thing down. The nets vanished. They didn't tear: they vanished.",
        "Tecemos redes entre as luas para atrasar a coisa. As redes sumiram. Não rasgaram: sumiram.",
    ),
    ("Weapon Master", "Mestre de Armas"),
    (
        "Weapons jammed for 3 seconds.",
        "Armas travadas por 3 segundos.",
    ),
    ("Weapons mastered", "Armas dominadas"),
    (
        "Weavers don't hurt: they slow you down. Which is worse.",
        "Tecelãs não machucam: elas te deixam lento. O que é pior.",
    ),
    (
        "What the Swarm was fleeing from is here.",
        "Aquilo de que o Enxame fugia está aqui.",
    ),
    (
        "What the Swarm was fleeing from. A hunger the size of a galaxy.",
        "Aquilo de que o Enxame fugia. Uma fome do tamanho de uma galáxia.",
    ),
    (
        "When hit, drops a free bomb.",
        "Ao ser atingido, solta uma bomba de graça.",
    ),
    (
        "When hit, every enemy takes damage.",
        "Ao ser atingido, todos os inimigos levam dano.",
    ),
    (
        "When it dies, it splits into two hungry hatchlings that hunt your ship.",
        "Quando morre, se divide em dois filhotes famintos que caçam a sua nave.",
    ),
    (
        "Who knows what's inside? It's almost always good.",
        "Quem sabe o que tem dentro? Quase sempre é bom.",
    ),
    ("Who made this game?", "Quem fez este jogo?"),
    ("Wide fan of fast shots.", "Leque largo de tiros rápidos."),
    ("Win 10 Rift minigames.", "Vença 10 minigames das Fendas."),
    (
        "Win here without continues to see the true ending.",
        "Vença aqui sem continues para ver o final verdadeiro.",
    ),
    (
        "Win the campaign 3 times to unlock.",
        "Vença a campanha 3 vezes para liberar.",
    ),
    ("Without a Scratch", "Sem Arranhões"),
    ("X / G .............. bomb", "X / G .............. bomba"),
    ("X or K", "X ou K"),
    ("YEAR 2000", "ANO 2000"),
    ("YES", "SIM"),
    ("YOUR TURN!", "SUA VEZ!"),
    ("Year 2097.", "Ano 2097."),
    (
        "You and the enemies fire twice as much.",
        "Você e os inimigos atiram o dobro.",
    ),
    (
        "You can only see near your ship.",
        "Você só enxerga perto da nave.",
    ),
    ("You lost your weapon!", "Você perdeu a arma!"),
    ("You rose from the ashes.", "Você renasceu das cinzas."),
    (
        "Your next death won't cost a weapon level.",
        "A próxima morte não tira nível da arma.",
    ),
    (
        "Your shots fly 15% faster.",
        "Seus tiros voam 15% mais rápido.",
    ),
    (
        "Your shots light the way.",
        "Seus tiros iluminam o caminho.",
    ),
    (
        "Your time has been recorded.",
        "Seu tempo ficou registrado.",
    ),
    (
        "Your weapon goes up 2 levels now.",
        "Sua arma sobe 2 níveis agora.",
    ),
    (
        "Your weapon starts at level 2.",
        "Sua arma começa no nível 2.",
    ),
    (
        "Your weapon starts at level 3.",
        "Sua arma começa no nível 3.",
    ),
    (
        "Z  Slow motion for the world",
        "Z  Câmera lenta para o mundo",
    ),
    ("ZENITH DREADNOUGHT", "COURAÇADO ZÊNITE"),
    ("a formation shoot 'em up", "um shoot 'em up de formações"),
    ("and brought back a story", "e trouxe de volta uma história"),
    (
        "and crystals. See them on pause or in the panel.",
        "e cristais. Veja na pausa ou no painel.",
    ),
    ("and repeat them with the ARROWS.", "e repita com as SETAS."),
    ("ate 20 crystals", "comeu 20 cristais"),
    (
        "bonus stage and a boss. Then, the shop.",
        "estágio bônus e um chefe. Depois, loja.",
    ),
    (
        "by shooting down, grazing, and over time.",
        "abatendo, raspando e com o tempo.",
    ),
    (
        "cards you unlock more slots.",
        "cartas você ganha mais espaços.",
    ),
    (
        "carrying samples of stardust... and something more.",
        "trazendo amostras de poeira estelar... e algo mais.",
    ),
    ("composing the soundtrack...", "compondo a trilha sonora..."),
    ("crystal on every kill.", "cristal em cada abate."),
    (
        "crystals, levels, power-ups and cards.",
        "cristais, níveis, poderes e cartas.",
    ),
    ("drones and AUTOFIRE.", "drones e no TIRO AUTOMÁTICO."),
    ("embedded font", "fonte embutida"),
    ("empty", "vazio"),
    ("for free, lucky you", "de graça, que sorte"),
    ("game in progress", "jogo ativo"),
    (
        "is on the golden band. 3 locks.",
        "estiver na faixa dourada. 3 travas.",
    ),
    ("it was just stardust", "só tinha poeira estelar"),
    ("no card equipped", "nenhuma carta equipada"),
    ("note by note", "nota por nota"),
    (
        "of the truth about the Swarm.",
        "da verdade sobre o Enxame.",
    ),
    ("one hit less", "um golpe a menos"),
    (
        "rare encounters. Each one tells a piece",
        "encontros raros. Cada um conta um pedaço",
    ),
    ("straight into your collection", "direto para a coleção"),
    (
        "that we used to play on the family PC",
        "que a gente jogava no PC da família",
    ),
    (
        "the Swarm was never the enemy. It was fleeing.",
        "o Enxame nunca foi o inimigo. Ele estava fugindo.",
    ),
    (
        "unlocks ships, weapons, modes and paints.",
        "libera naves, armas, modos e pinturas.",
    ),
    (
        "weapons, Damage Cores, fire rate,",
        "melhores, Núcleos de Dano, cadência,",
    ),
    ("weapons, modes and paints.", "armas, modos e pinturas."),
];

/// Templates with placeholders: (English parts, Portuguese parts).
static TEMPLATES: &[(&[&str], &[&str])] = &[
    (
        &["lives ", "  ·  shots x", "  ·  XP x", "  ·  continues ", ""],
        &["vidas ", "  ·  tiros x", "  ·  XP x", "  ·  continues ", ""],
    ),
    (
        &["FIREPOWER x", "   (weapon ", " x", " · cores ", " x", ")"],
        &[
            "PODER DE FOGO x",
            "   (arma ",
            " x",
            " · núcleos ",
            " x",
            ")",
        ],
    ),
    (
        &["", "  ·  UP/DOWN: ship, paint, trail"],
        &["", "  ·  CIMA/BAIXO: nave, pintura, rastro"],
    ),
    (
        &["Expedition · sector ", " cleared"],
        &["Expedição · setor ", " vencido"],
    ),
    (
        &["SHIPS ", "  BOMBS ", "  ", " LV", "  DRONES ", ""],
        &["NAVES ", "  BOMBAS ", "  ", " NV", "  DRONES ", ""],
    ),
    (
        &["Shoot down ", " while diving"],
        &["Abata ", " em mergulho"],
    ),
    (
        &["< ", " / 6 >   ·   ESC back"],
        &["< ", " / 6 >   ·   ESC volta"],
    ),
    (
        &["Extra life and +", " points"],
        &["Vida extra e +", " pontos"],
    ),
    (
        &["", "  ·  +", " XP  +", " crystals"],
        &["", "  ·  +", " XP  +", " cristais"],
    ),
    (
        &["ROUND ", " of 3 · ", " stars"],
        &["RODADA ", " de 3 · ", " estrelas"],
    ),
    (
        &["Destroy ", " boss parts"],
        &["Destrua ", " partes de chefes"],
    ),
    (&["Shoot down ", " elites"], &["Abata ", " elites"]),
    (
        &["HONOR INSIGNIAS  ", "/", ""],
        &["INSÍGNIAS DE HONRA  ", "/", ""],
    ),
    (&["DEPART FOR SECTOR ", ""], &["PARTIR PARA O SETOR ", ""]),
    (&["CONTINUE - CYCLE ", ""], &["CONTINUAR - CICLO ", ""]),
    (&["Collect ", " crystals"], &["Colete ", " cristais"]),
    (&["FARTHEST: SECTOR ", ""], &["MAIS LONGE: SETOR ", ""]),
    (&["RULE OF THE DAY: ", ""], &["REGRA DO DIA: ", ""]),
    (&["", " of ", " achievements"], &["", " de ", " conquistas"]),
    (&["", " READY FOR LAUNCH"], &["", " PRONTA PARA DECOLAR"]),
    (&["BANK: +", " CRYSTALS"], &["BANCO: +", " CRISTAIS"]),
    (&["LIGHTNING WAVE +", ""], &["ONDA RELÂMPAGO +", ""]),
    (&["REROLL OFFERS (", ")"], &["TROCAR OFERTAS (", ")"]),
    (&["TRAIL: ", " (IN USE)"], &["RASTRO: ", " (EM USO)"]),
    (&["Reach the rank ", ""], &["Alcance a patente ", ""]),
    (&["Freeze ", " enemies"], &["Congele ", " inimigos"]),
    (&["ACHIEVEMENTS ", "/", ""], &["CONQUISTAS ", "/", ""]),
    (&["FACE THE BOSS ", ""], &["ENFRENTAR O CHEFE ", ""]),
    (&["Grab ", " capsules"], &["Pegue ", " cápsulas"]),
    (&["SHARPSHOOTER +", ""], &["TIRO CERTEIRO +", ""]),
    (&["ACHIEVEMENT: ", ""], &["CONQUISTA: ", ""]),
    (&["Make a ", " combo"], &["Faça um combo de ", ""]),
    (
        &["P1  x", "  B", "  ", " LV", ""],
        &["J1  x", "  B", "  ", " NV", ""],
    ),
    (
        &["P2  x", "  B", "  ", " LV", ""],
        &["J2  x", "  B", "  ", " NV", ""],
    ),
    (&["START: LEVEL ", ""], &["INÍCIO: NÍVEL ", ""]),
    (&["TARGETS: ", " / ", ""], &["ALVOS: ", " / ", ""]),
    (&["Shoot down ", " ", ""], &["Abata ", " ", ""]),
    (&["FEVER x2   ", "s"], &["FEBRE x2   ", "s"]),
    (&["Make ", " grazes"], &["Faça ", " raspões"]),
    (&["Next: ", " (", " XP)"], &["Próxima: ", " (", " XP)"]),
    (&["", " POINTS  ·  ", ""], &["", " PONTOS  ·  ", ""]),
    (&["BEST COMBO ", ""], &["MAIOR COMBO ", ""]),
    (&["BEST WAVE: ", ""], &["MELHOR ONDA: ", ""]),
    (&["BEST TIME ", ":", ""], &["MELHOR TEMPO ", ":", ""]),
    (&["PROMOTION: ", ""], &["PROMOÇÃO: ", ""]),
    (&["HIGH SCORE ", ""], &["RECORDE ", ""]),
    (&["shot down: ", ""], &["abatidos: ", ""]),
    (&["", " (", " perfect)"], &["", " (", " perfeitos)"]),
    (&["Challenge: ", ""], &["Desafio: ", ""]),
    (&["ARCHIVES ", "/", ""], &["ARQUIVOS ", "/", ""]),
    (&["BOSS ", " OF 9"], &["CHEFE ", " DE 9"]),
    (&["CRYSTALS: ", ""], &["CRISTAIS: ", ""]),
    (&["FEVER x2  ", ""], &["FEBRE x2  ", ""]),
    (&["NEW CARD: ", ""], &["NOVA CARTA: ", ""]),
    (&["ACCURACY ", "%"], &["PRECISÃO ", "%"]),
    (&["Progress: ", ""], &["Progresso: ", ""]),
    (&["", " of ", " cards"], &["", " de ", " cartas"]),
    (&["", " was lost."], &["", " foi perdida."]),
    (&["LEVEL ", "/100"], &["NÍVEL ", "/100"]),
    (&["HITS  ", " / ", ""], &["ACERTOS  ", " / ", ""]),
    (&["BOSS IN ", "s"], &["CHEFE EM ", "s"]),
    (&["COMBO ", "  x", ""], &["COMBO ", "  x", ""]),
    (&["CREDITS: ", ""], &["CRÉDITOS: ", ""]),
    (&["ABILITY: ", ""], &["HABILIDADE: ", ""]),
    (&["", " of ", " logs"], &["", " de ", " registros"]),
    (&["P2 SHIP: ", ""], &["NAVE J2: ", ""]),
    (&["MIRROR ", "s"], &["ESPELHO ", "s"]),
    (&["GROUP! +", ""], &["GRUPO! +", ""]),
    (&["REWARD: ", ""], &["PRÊMIO: ", ""]),
    (&["SECTOR ", "-", ""], &["SETOR ", "-", ""]),
    (&["copies: ", ""], &["cópias: ", ""]),
    (&["", "  (+", " XP)"], &["", "  (+", " XP)"]),
    (&["ANCHOR ", "s"], &["ÂNCORA ", "s"]),
    (&["SECTOR ", "/", ""], &["SETOR ", "/", ""]),
    (&["BONUS  ", ""], &["BÔNUS  ", ""]),
    (&["CARDS ", "/", ""], &["CARTAS ", "/", ""]),
    (&["BOSS ", "/8"], &["CHEFE ", "/8"]),
    (&["COMBO x", ""], &["COMBO x", ""]),
    (&["DRONES ", ""], &["DRONES ", ""]),
    (&["EVENT: ", ""], &["EVENTO: ", ""]),
    (&["TODAY: ", ""], &["HOJE: ", ""]),
    (&["CURSE: ", ""], &["MALDIÇÃO: ", ""]),
    (&["PAINT: ", ""], &["PINTURA: ", ""]),
    (&["GRAZES ", ""], &["RASPÕES ", ""]),
    (&["SECTOR ", ""], &["SETOR ", ""]),
    (&["sector ", ""], &["setor ", ""]),
    (&["", " LEVEL ", ""], &["", " NÍVEL ", ""]),
    (&["TRAIL: ", ""], &["RASTRO: ", ""]),
    (&["CARD: ", ""], &["CARTA: ", ""]),
    (&["CYCLE ", ""], &["CICLO ", ""]),
    (&["FEAT: ", ""], &["FEITO: ", ""]),
    (&["SHIP: ", ""], &["NAVE: ", ""]),
    (&["SCORE ", ""], &["PONTOS ", ""]),
    (&["Rank: ", ""], &["Patente: ", ""]),
    (&["PAGE ", "/", ""], &["PÁGINA ", "/", ""]),
    (&["RULE: ", ""], &["REGRA: ", ""]),
    (&["TIME ", ":", ""], &["TEMPO ", ":", ""]),
    (&["rank #", ""], &["patente nº ", ""]),
    (&["NEW: ", ""], &["NOVO: ", ""]),
    (&["WAVE ", ""], &["ONDA ", ""]),
    (&["JAM ", "s"], &["PANE ", "s"]),
    (&["Rank ", ""], &["Patente ", ""]),
    (&["WARP ", ""], &["SALTO ", ""]),
    (&["rank ", ""], &["patente ", ""]),
    (&["", " LOST"], &["", " PERDIDA"]),
    (&["", "  (", "/", ")"], &["", "  (", "/", ")"]),
    (&["> ", " <"], &["> ", " <"]),
    (&["HI ", ""], &["REC ", ""]),
];
