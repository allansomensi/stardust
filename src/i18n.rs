//! Tradução da interface. O jogo é escrito em português; em inglês, cada texto
//! desenhado passa por `tr`, que procura a frase exata ou um modelo com variáveis.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Pt,
    En,
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    LANG.store(l as u8, Ordering::Relaxed);
    CACHE.with(|c| c.borrow_mut().clear());
}

pub fn lang() -> Lang {
    if LANG.load(Ordering::Relaxed) == 1 {
        Lang::En
    } else {
        Lang::Pt
    }
}

thread_local! {
    static CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

/// Traduz um texto já pronto para a língua atual.
pub fn tr(s: &str) -> String {
    if lang() == Lang::Pt || s.is_empty() {
        return s.to_string();
    }
    if let Some(v) = CACHE.with(|c| c.borrow().get(s).cloned()) {
        return v;
    }
    let out = translate(s);
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

fn translate(s: &str) -> String {
    translate_depth(s, 0)
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
    // Estrelas de classe no fim ("TROVÃO ***").
    let trimmed = s.trim_end_matches('*');
    if trimmed.len() < s.len() && trimmed.ends_with(' ') {
        let base = trimmed.trim_end();
        return format!("{}{}", translate_depth(base, depth + 1), &s[base.len()..]);
    }
    let mut fallback: Option<String> = None;
    for (pt, en) in TEMPLATES {
        let Some(vals) = matches(s, pt) else { continue };
        let mut clean = true;
        let mut out = String::new();
        for (i, part) in en.iter().enumerate() {
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
    // Textos montados com separadores ("RARA - descrição").
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

/// Casa `s` com um modelo dividido nas partes fixas; devolve os valores das variáveis.
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

static EXACT: &[(&str, &str)] = &[
    ("  ·  SECRETA", "  ·  SECRET"),
    (" (EM USO)", " (IN USE)"),
    (
        "*  Estrela: invencível e atropela",
        "*  Star: invincible, rams through",
    ),
    ("+1 BOMBA", "+1 BOMB"),
    ("+1 bomba (tecla X).", "+1 bomb (X key)."),
    (
        "+1 bomba no início da partida.",
        "+1 bomb at the start of the run.",
    ),
    ("+1 continue por partida.", "+1 continue per run."),
    ("+1 drone de apoio (máximo 3).", "+1 support drone (max 3)."),
    ("+1 nave", "+1 ship"),
    ("+1 nave agora.", "+1 ship now."),
    ("+1 nave reserva.", "+1 reserve ship."),
    ("+1 nível de arma", "+1 weapon level"),
    ("+1 nível na arma atual.", "+1 level on the current weapon."),
    ("+1 vida e +1 bomba. Muuu.", "+1 life and +1 bomb. Moo."),
    (
        "+1 vida e escudo novo a cada setor.",
        "+1 life and a fresh shield every sector.",
    ),
    (
        "+1 vida no início da partida.",
        "+1 life at the start of the run.",
    ),
    (
        "+15% de dano em todas as armas. Acumula até 6x.",
        "+15% damage on every weapon. Stacks up to 6x.",
    ),
    ("+2 bombas", "+2 bombs"),
    ("+2 bombas agora.", "+2 bombs now."),
    ("+25 CRISTAIS", "+25 CRYSTALS"),
    (
        "+25000 pontos e 30 cristais",
        "+25000 points and 30 crystals",
    ),
    (
        "+3 vidas. Você sabe o que fez.",
        "+3 lives. You know what you did.",
    ),
    ("+45 cristais", "+45 crystals"),
    (
        "12 para vencer · 22 sem bater = perfeito",
        "12 to win · 22 without crashing = perfect",
    ),
    ("16 cartas", "16 cards"),
    ("1UP  Vida extra (rara!)", "1UP  Extra life (rare!)"),
    (
        "3 missões por partida. Cumpra e ganhe XP",
        "3 missions per run. Complete them for XP",
    ),
    (
        "3 s devolvendo os tiros inimigos que chegam perto.",
        "3 s sending nearby enemy shots back.",
    ),
    (
        "3 setores com uma regra maluca que muda todo dia. Igual para todos.",
        "3 sectors with a crazy rule that changes daily. Same for everyone.",
    ),
    ("3 tiros destroem uma maldição.", "3 shots destroy a curse."),
    ("30 cartas", "30 cards"),
    (
        "4 s capturando os tiros por perto: cada um vira cristal.",
        "4 s catching nearby shots: each one becomes a crystal.",
    ),
    (
        "5 s atirando o dobro com a arma um nível acima.",
        "5 s firing twice as fast, one weapon level higher.",
    ),
    (
        "5 s com inimigos e tiros quase parados.",
        "5 s with enemies and shots almost frozen.",
    ),
    (
        "5 s puxando todos os itens da tela (menos maldições).",
        "5 s pulling every item on screen (except curses).",
    ),
    ("8 cartas", "8 cards"),
    (
        "8 setores, chefes, loja e cartas. A aventura completa.",
        "8 sectors, bosses, shop and cards. The full adventure.",
    ),
    (
        "A Colmeia solta o dobro de abelhas.",
        "The Hive releases twice as many bees.",
    ),
    ("A ESTRELA SE ABRIU", "THE STAR OPENED"),
    ("A FENDA SE FECHOU", "THE RIFT CLOSED"),
    (
        "A Febre começa com combo 35 em vez de 50.",
        "Fever starts at combo 35 instead of 50.",
    ),
    ("A Febre dura 50% mais.", "Fever lasts 50% longer."),
    (
        "A Nave-Mãe deixou algo para trás...",
        "The Mothership left something behind...",
    ),
    (
        "A Nave-Mãe misteriosa aparece com o dobro de frequência.",
        "The mysterious Mothership shows up twice as often.",
    ),
    (
        "A PULSAR cai das cápsulas e as estrelas valem o dobro.",
        "PULSAR drops from capsules and its stars are worth double.",
    ),
    (
        "A Rainha Suprema caiu sem que você recuasse.",
        "The Supreme Queen fell without you ever retreating.",
    ),
    (
        "A Rainha estava certa: o Enxame só queria passar.",
        "The Queen was right: the Swarm only wanted to pass through.",
    ),
    (
        "A Serpente era a guardiã do caminho. Ela ouviu primeiro: um silêncio que engole estrelas.",
        "The Serpent was the guardian of the path. It heard first: a silence that swallows stars.",
    ),
    (
        "A Vaca não é nossa. Ela apareceu um dia e começou a nos seguir. Desde então chamamos ela de sorte.",
        "The Cow is not ours. It showed up one day and started following us. Since then we call it luck.",
    ),
    (
        "A arma PULSAR agora cai das cápsulas.",
        "The PULSAR weapon now drops from capsules.",
    ),
    ("A arma perdeu 2 níveis.", "The weapon lost 2 levels."),
    (
        "A barra de habilidade enche 35% mais rápido.",
        "The ability bar fills 35% faster.",
    ),
    (
        "A base do Enxame. Sozinho é inofensivo; em formação, é uma muralha viva.",
        "The base of the Swarm. Harmless alone; in formation, a living wall.",
    ),
    (
        "A cada 100 cristais coletados, +1 bomba.",
        "Every 100 crystals collected, +1 bomb.",
    ),
    (
        "A cada 6 disparos, um leque de 6 estrelas.",
        "Every 6 shots, a fan of 6 stars.",
    ),
    (
        "A cauda do Cometa Wild solta poeira. Fique na frente dele.",
        "The Wild Comet's tail sheds dust. Stay in front of it.",
    ),
    (
        "A frota caiu em horas. Sobrou um hangar escondido, uma nave e você.",
        "The fleet fell in hours. What's left: a hidden hangar, one ship and you.",
    ),
    (
        "A galáxia respira aliviada... por enquanto.",
        "The galaxy breathes again... for now.",
    ),
    (
        "A luz some. Só se vê perto da nave.",
        "The light fades. You can only see near your ship.",
    ),
    (
        "A nave da missão Stardust. Ímã de cristais 50% maior.",
        "The ship of the Stardust mission. 50% bigger crystal magnet.",
    ),
    (
        "A poeira da sonda humana chamou por nós. Ordenei a migração. Não há tempo para explicar aos nativos.",
        "The human probe's dust called to us. I ordered the migration. No time to explain to the natives.",
    ),
    (
        "A primeira nave que existiu neste jogo. Feita de nostalgia.",
        "The first ship ever made for this game. Built from nostalgia.",
    ),
    (
        "A primeira troca de ofertas na loja é grátis.",
        "The first shop reroll is free.",
    ),
    (
        "A próxima morte não tira nível da arma.",
        "Your next death won't cost a weapon level.",
    ),
    (
        "A sonda STARDUST voltava de um cometa distante",
        "The STARDUST probe was coming back from a distant comet",
    ),
    (
        "A você, que jogou até aqui.",
        "To you, who played this far.",
    ),
    ("AGRADECIMENTO ESPECIAL", "SPECIAL THANKS"),
    ("AGULHA", "NEEDLE"),
    ("ALERTA!", "WARNING!"),
    ("ALMIRANTE", "ADMIRAL"),
    ("ALMIRANTE DE ESQUADRA", "FLEET ADMIRAL"),
    ("ALTO", "HIGH"),
    ("AMULETO", "AMULET"),
    ("ANO 2000", "YEAR 2000"),
    ("APAGAR PROGRESSO", "ERASE PROGRESS"),
    ("ARCO-ÍRIS", "RAINBOW"),
    ("ARMA", "WEAPON"),
    ("ARMA REFORÇADA", "WEAPON BOOSTED"),
    ("ARMAS", "WEAPONS"),
    ("ARQUIVOS", "ARCHIVES"),
    ("ARQUIVOS, aba HONRAS.", "ARCHIVES, HONORS tab."),
    ("ARTE", "ART"),
    ("AS GÊMEAS", "THE TWINS"),
    ("ASPIRANTE", "MIDSHIPMAN"),
    ("ATIRADOR", "GUNNER"),
    (
        "Abata 10 Mímicos no total.",
        "Shoot down 10 Mimics in total.",
    ),
    (
        "Abata 150 Kamikazes no total.",
        "Shoot down 150 Kamikazes in total.",
    ),
    (
        "Abata 300 inimigos com a Chama.",
        "Shoot down 300 enemies with the Flame.",
    ),
    (
        "Abata 5 inimigos com um só tiro carregado.",
        "Shoot down 5 enemies with a single charged shot.",
    ),
    (
        "Abata 50 inimigos de elite no total.",
        "Shoot down 50 elite enemies in total.",
    ),
    (
        "Abata a Vaca Cósmica 3 vezes.",
        "Shoot down the Cosmic Cow 3 times.",
    ),
    ("Abata a Vaca Cósmica.", "Shoot down the Cosmic Cow."),
    (
        "Abata um Escaravelho Dourado.",
        "Shoot down a Golden Scarab.",
    ),
    (
        "Abata um Ladrão carregando o que roubou.",
        "Shoot down a Thief carrying its loot.",
    ),
    ("Abata um para registrar.", "Shoot one down to record it."),
    (
        "Abelhas teleguiadas. O Enxame contra o Enxame.",
        "Homing bees. The Swarm against the Swarm.",
    ),
    ("Abra 5 relicários secretos.", "Open 5 secret reliquaries."),
    (
        "Abra o relicário antes que a fenda se feche!",
        "Open the reliquary before the rift closes!",
    ),
    (
        "Abrem um minigame rápido. Vença e ganhe",
        "They open a quick minigame. Win to earn",
    ),
    ("Absorve um golpe.", "Absorbs one hit."),
    ("Acerte os alvos com ESPAÇO.", "Hit the targets with SPACE."),
    (
        "Acerte todos os inimigos de um estágio bônus.",
        "Hit every enemy in a bonus stage.",
    ),
    (
        "Agora a colmeia vive, e você sabe a verdade.",
        "Now the hive lives, and you know the truth.",
    ),
    (
        "Agulhas finíssimas e velozes que perfuram tudo.",
        "Ultra-thin, fast needles that pierce everything.",
    ),
    (
        "Ainda não encontrada. Continue jogando.",
        "Not found yet. Keep playing.",
    ),
    ("Alcance a patente SARGENTO.", "Reach the rank SERGEANT."),
    (
        "Algo acontece se você digitar o nome do jogo no título.",
        "Something happens if you type the game's name on the title screen.",
    ),
    (
        "Algo puxa a nave para dentro...",
        "Something pulls the ship inside...",
    ),
    ("Amuleto Antigo", "Ancient Amulet"),
    ("Ano 2097.", "Year 2097."),
    (
        "Ao ser atingido, solta uma bomba de graça.",
        "When hit, drops a free bomb.",
    ),
    (
        "Ao ser atingido, todos os inimigos levam dano.",
        "When hit, every enemy takes damage.",
    ),
    (
        "Apaga patentes, cartas e conquistas. Aperte 2x.",
        "Erases ranks, cards and achievements. Press twice.",
    ),
    (
        "Aperte ESPAÇO quando a agulha",
        "Press SPACE when the needle",
    ),
    ("Aperte Espaço, Z ou J", "Tap Space, Z or J"),
    (
        "Aquilo de que o Enxame fugia está aqui.",
        "What the Swarm was fleeing from is here.",
    ),
    (
        "Aquilo de que o Enxame fugia. Uma fome do tamanho de uma galáxia.",
        "What the Swarm was fleeing from. A hunger the size of a galaxy.",
    ),
    ("Aracnofobia", "Arachnophobia"),
    (
        "Aranha do tamanho de uma lua. Suas teias atravessam a tela inteira.",
        "A spider the size of a moon. Its webs cross the entire screen.",
    ),
    ("Arma CHAMA", "FLAME weapon"),
    (
        "Arma DISCO e dificuldade DIFÍCIL",
        "DISC weapon and HARD difficulty",
    ),
    (
        "Arma ESTILHAÇO e modo SOBREVIVÊNCIA",
        "SHRAPNEL weapon and SURVIVAL mode",
    ),
    ("Arma ONDA", "WAVE weapon"),
    ("Arma TROVÃO", "THUNDER weapon"),
    ("Arma favorita", "Favorite weapon"),
    ("Armas dominadas", "Weapons mastered"),
    (
        "Armas mais caras na loja batem muito mais forte.",
        "Pricier weapons in the shop hit much harder.",
    ),
    (
        "Armas travadas por 3 segundos.",
        "Weapons jammed for 3 seconds.",
    ),
    ("Arqueólogo", "Archaeologist"),
    ("Arsenal Completo", "Full Arsenal"),
    ("Arsenal Duplo", "Double Arsenal"),
    ("Arsenal Variado", "Varied Arsenal"),
    ("Asas de Cristal", "Crystal Wings"),
    ("Atalho", "Shortcut"),
    ("Ataque o Mercador.", "Attack the Merchant."),
    ("Atira 10% mais rápido.", "Fires 10% faster."),
    (
        "Atira devagar, mas cada tiro vale 60% a mais.",
        "Fires slowly, but each shot deals 60% more.",
    ),
    ("Atirar", "Fire"),
    (
        "Atravesse 8 setores e derrote a Rainha",
        "Cross 8 sectors and defeat the Supreme",
    ),
    (
        "Atravesse os 8 setores. Chegue ao Coração da Colmeia.",
        "Cross all 8 sectors. Reach the Heart of the Hive.",
    ),
    ("Azarado", "Jinxed"),
    ("BAIXO", "LOW"),
    ("BANCO ESTELAR", "STAR BANK"),
    ("BASTIÃO", "BASTION"),
    ("BESOURO", "BEETLE"),
    ("BESTIÁRIO", "BESTIARY"),
    ("BLINDAGEM", "ARMOR"),
    ("BLOQUEADA", "LOCKED"),
    ("BLOQUEADO", "LOCKED"),
    ("BOMBA", "BOMB"),
    ("BOMBAS", "BOMBS"),
    ("BUMERANGUE", "BOOMERANG"),
    ("BURACO NEGRO", "BLACK HOLE"),
    ("Bateria Extra", "Spare Battery"),
    ("Bezerro Cósmico", "Cosmic Calf"),
    ("Boa sorte, piloto.", "Good luck, pilot."),
    ("Bolso Extra", "Extra Pocket"),
    ("Bom Samaritano", "Good Samaritan"),
    ("Bomba", "Bomb"),
    ("Bomba Relâmpago", "Lightning Bomb"),
    ("Bombardeiro", "Bomber"),
    (
        "Bombas apagam os tiros na tela. Use antes de ser atingido.",
        "Bombs wipe the shots on screen. Use one before you get hit.",
    ),
    (
        "Bombas causam o dobro de dano.",
        "Bombs deal double damage.",
    ),
    ("Bombas usadas", "Bombs used"),
    (
        "Buracos negros duram o dobro e sugam cristais.",
        "Black holes last twice as long and suck in crystals.",
    ),
    ("BÔNUS", "BONUS"),
    (
        "C / L / V .......... habilidade da nave",
        "C / L / V .......... ship ability",
    ),
    ("C, L ou V", "C, L or V"),
    ("CADETE", "CADET"),
    ("CADÊNCIA", "FIRE RATE"),
    ("CAIXA MISTERIOSA", "MYSTERY BOX"),
    ("CAMALEÃO", "CHAMELEON"),
    ("CAMPANHA", "CAMPAIGN"),
    ("CAMPO DE ASTEROIDES", "ASTEROID FIELD"),
    ("CANHÃO", "CANNON"),
    ("CANHÃO ESTELAR", "STAR CANNON"),
    ("CANHÕES LATERAIS", "SIDE CANNONS"),
    ("CAPITÃO", "CAPTAIN"),
    ("CARGUEIRO", "FREIGHTER"),
    ("CARTA MISTERIOSA", "MYSTERY CARD"),
    ("CARTAS", "CARDS"),
    ("CARTAS EQUIPADAS", "EQUIPPED CARDS"),
    ("CAVEIRA", "SKULL"),
    ("CAVEIRA!", "SKULL!"),
    (
        "CAVEIRAS (bolhas vermelhas): desvie!",
        "SKULLS (red bubbles): dodge them!",
    ),
    ("CHAMA", "FLAME"),
    ("CHEFE", "BOSS"),
    ("CHEFE ABATIDO!", "BOSS DOWN!"),
    ("CHEFE DERROTADO", "BOSS DEFEATED"),
    ("CHEFÕES", "BOSS RUSH"),
    ("CHUVA DE CRISTAIS", "CRYSTAL RAIN"),
    ("CHUVA DE CÁPSULAS", "CAPSULE RAIN"),
    ("CHUVA DE METEOROS", "METEOR SHOWER"),
    ("CINTURÃO DE GELO", "ICE BELT"),
    ("CLARÕES", "FLASHES"),
    ("CLÁSSICO", "CLASSIC"),
    ("COFRE ESTELAR", "STAR VAULT"),
    ("COLEÇÃO", "COLLECTION"),
    ("COLMEIA", "HIVE"),
    ("COLMEIA EXTERIOR", "OUTER HIVE"),
    ("COLOSSO", "COLOSSUS"),
    ("COMBOIO DE CARGA", "CARGO CONVOY"),
    ("COMETA", "COMET"),
    ("COMETA WILD", "WILD COMET"),
    ("COMETA WILD ANCESTRAL", "ANCIENT WILD COMET"),
    ("COMO JOGAR", "HOW TO PLAY"),
    ("COMODORO", "COMMODORE"),
    ("COMUM", "COMMON"),
    ("CONFIGURAÇÕES", "SETTINGS"),
    ("CONFIRMA?", "CONFIRM?"),
    ("CONQUISTAS", "ACHIEVEMENTS"),
    ("CONTINUAR", "CONTINUE"),
    ("CONTINUAR?", "CONTINUE?"),
    ("CONTRA-ALMIRANTE", "REAR ADMIRAL"),
    ("CONTRATO CUMPRIDO", "CONTRACT COMPLETE"),
    ("CONTRATOS", "CONTRACTS"),
    ("CONTROLES", "CONTROLS"),
    ("CORAÇÃO DO ENXAME", "HEART OF THE SWARM"),
    ("CORONEL", "COLONEL"),
    ("CORSÁRIA", "CORSAIR"),
    ("COURAÇADO ZÊNITE", "ZENITH DREADNOUGHT"),
    ("COURAÇADO ÔMEGA", "OMEGA DREADNOUGHT"),
    ("CRIAÇÃO", "CREATED BY"),
    ("CRIO", "CRYO"),
    ("CRISTAIS DE BÔNUS", "BONUS CRYSTALS"),
    ("CRISTAIS EM DOBRO", "DOUBLE CRYSTALS"),
    ("CRISTAIS!", "CRYSTALS!"),
    ("CRISTALIZADOR", "CRYSTALLIZER"),
    ("CUIDADO COM", "WATCH OUT FOR"),
    ("Cada batida custa caro.", "Every crash costs you."),
    (
        "Cada cristal recolhido vira armas melhores no hangar móvel.",
        "Every crystal you collect turns into better weapons at the mobile hangar.",
    ),
    (
        "Cada nave tem a sua. A barra enche",
        "Each ship has its own. The bar fills",
    ),
    (
        "Cada partida dá XP e libera naves,",
        "Every run gives XP and unlocks ships,",
    ),
    (
        "Cadência +12% pelo resto da expedição.",
        "Fire rate +12% for the rest of the expedition.",
    ),
    ("Cadência de tiro +10%.", "Fire rate +10%."),
    (
        "Caem dos chefes. Contam a verdade.",
        "Dropped by bosses. They tell the truth.",
    ),
    (
        "Caem raramente; chefes sempre soltam.",
        "Rare drops; bosses always drop one.",
    ),
    (
        "Canaliza estrelas: a cada 6 disparos, um leque estelar.",
        "Channels stars: every 6 shots, a star fan.",
    ),
    (
        "Cargueiros lotados de cristais cruzam a tela.",
        "Freighters full of crystals cross the screen.",
    ),
    (
        "Carrega dois canhões biológicos. Para no meio da tela para disparar em leque.",
        "Carries two biological cannons. Stops mid-screen to fire a spread.",
    ),
    (
        "Carrega uma placa de quitina na frente. Quebre o escudo antes de feri-lo.",
        "Carries a chitin plate in front. Break the shield before you can hurt it.",
    ),
    (
        "Carta secreta: Bezerro Cósmico.",
        "Secret card: Cosmic Calf.",
    ),
    ("Cartas caem 50% mais.", "Cards drop 50% more often."),
    (
        "Cartas caem com o dobro de frequência.",
        "Cards drop twice as often.",
    ),
    ("Cartas voam até você.", "Cards fly to you."),
    ("Casco Reforçado", "Reinforced Hull"),
    (
        "Caveiras caem devagar e te seguem. Três tiros acabam com elas.",
        "Skulls fall slowly and follow you. Three shots finish them.",
    ),
    ("Caçador", "Hunter"),
    ("Caçador de Cometas", "Comet Hunter"),
    ("Caçador de Elite", "Elite Hunter"),
    ("Caçador de Estrelas", "Star Hunter"),
    ("Chefes derrotados", "Bosses defeated"),
    (
        "Chegue ao setor 10 da Expedição.",
        "Reach sector 10 in Expedition.",
    ),
    (
        "Chegue à onda 30 no modo Sobrevivência.",
        "Reach wave 30 in Survival mode.",
    ),
    (
        "Chiptune sintetizado pelo próprio jogo,",
        "Chiptune synthesized by the game itself,",
    ),
    ("Cofrinho", "Piggy Bank"),
    ("Colecionador", "Collector"),
    (
        "Comandante blindado. Aguenta dois tiros e sempre leva escolta nos mergulhos.",
        "Armored commander. Takes two hits and always brings escorts when diving.",
    ),
    (
        "Combo 50 = Febre: pontos em dobro e um",
        "Combo 50 = Fever: double points and a",
    ),
    ("Combo Longo", "Long Combo"),
    (
        "Combos altos multiplicam os pontos. Não deixe esfriar.",
        "High combos multiply your points. Don't let them cool down.",
    ),
    (
        "Começa a partida com o LASER no nível 2.",
        "Starts the run with the LASER at level 2.",
    ),
    (
        "Começa cada setor em Frenesi.",
        "Starts every sector in Frenzy.",
    ),
    ("Começa com 40 cristais.", "Starts with 40 crystals."),
    (
        "Começa com a habilidade carregada.",
        "Starts with the ability charged.",
    ),
    (
        "Começa com um escudo que absorve um golpe.",
        "Starts with a shield that absorbs one hit.",
    ),
    ("Começo Forte", "Strong Start"),
    (
        "Como nos fliperamas: só a Rajada, sem loja, sem cartas. Ondas sem fim.",
        "Like the arcades: only the Blaster, no shop, no cards. Endless waves.",
    ),
    (
        "Complete 30 contratos no total.",
        "Complete 30 contracts in total.",
    ),
    ("Complete um contrato.", "Complete a contract."),
    ("Completista", "Completionist"),
    (
        "Congela: congelados levam dano dobrado.",
        "Freezes: frozen enemies take double damage.",
    ),
    (
        "Congele 150 inimigos no total.",
        "Freeze 150 enemies in total.",
    ),
    ("Conquista secreta.", "Secret achievement."),
    (
        "Consiga PERFEITO em 5 minigames.",
        "Get PERFECT in 5 minigames.",
    ),
    ("Constelação", "Constellation"),
    ("Contatos Imediatos", "Close Encounters"),
    ("Contra-Ataque", "Counterattack"),
    ("Contratado", "Hired"),
    ("Contrato Lucrativo", "Lucrative Contract"),
    (
        "Contratos cumpridos dão XP extra no fim da partida.",
        "Completed contracts give extra XP at the end of the run.",
    ),
    (
        "Controles invertidos por 6 segundos.",
        "Controls reversed for 6 seconds.",
    ),
    ("Coração Partido", "Broken Heart"),
    ("Coração do Enxame", "Heart of the Swarm"),
    ("Coroa da Rainha", "Queen's Crown"),
    ("Criador", "Creator"),
    (
        "Cristais +15% e ímã 20% maior.",
        "Crystals +15% and 20% bigger magnet.",
    ),
    (
        "Cristais +30% pelo resto da expedição.",
        "Crystals +30% for the rest of the expedition.",
    ),
    ("Cristais coletados", "Crystals collected"),
    (
        "Cristais compram armas melhores. Guarde para o chefe.",
        "Crystals buy better weapons. Save up for the boss.",
    ),
    (
        "Cristais em dobro, loja 50% mais cara.",
        "Double crystals, shop 50% more expensive.",
    ),
    ("Cristais valem 25% a mais.", "Crystals are worth 25% more."),
    (
        "Cristais valem o dobro neste setor.",
        "Crystals are worth double this sector.",
    ),
    (
        "Cristais vêm até você de qualquer lugar.",
        "Crystals come to you from anywhere.",
    ),
    (
        "Cristais x1,5 e pontos x1,2.",
        "Crystals x1.5 and points x1.2.",
    ),
    ("Cristal Vivo", "Living Crystal"),
    ("Cristal da Sorte", "Lucky Crystal"),
    ("Curioso", "Curious"),
    ("CÂMERA LENTA!", "SLOW MOTION!"),
    ("CÉLULA DE ENERGIA", "ENERGY CELL"),
    ("CÓDIGO ACEITO", "CODE ACCEPTED"),
    (
        "Cápsulas alternam entre 4 armas e duram mais.",
        "Capsules cycle through 4 weapons and last longer.",
    ),
    (
        "Cápsulas e poderes vêm até você.",
        "Capsules and power-ups come to you.",
    ),
    (
        "Cápsulas e power-ups por toda parte.",
        "Capsules and power-ups everywhere.",
    ),
    ("Código Konami", "Konami Code"),
    ("DANO", "DAMAGE"),
    (
        "DEMONSTRAÇÃO  -  APERTE QUALQUER TECLA",
        "DEMO  -  PRESS ANY KEY",
    ),
    ("DESAFIO DIÁRIO", "DAILY CHALLENGE"),
    ("DICA", "TIP"),
    ("DICA DO DIA", "TIP OF THE DAY"),
    ("DICAS NA PAUSA", "TIPS ON PAUSE"),
    ("DIFICULDADE", "DIFFICULTY"),
    ("DIFÍCIL", "HARD"),
    ("DIMENSÃO DOURADA", "GOLDEN DIMENSION"),
    ("DISCO", "DISC"),
    ("DISCO DE OURO", "GOLDEN RECORD"),
    ("DIVISOR", "SPLITTER"),
    ("DOBRA", "WARP"),
    ("DOBRA TEMPORAL", "TIME WARP"),
    ("DRONE DE ATAQUE", "ATTACK DRONE"),
    ("Da próxima vez, seja mais rápido.", "Next time, be faster."),
    (
        "Deixe o Mercador passar em paz três vezes.",
        "Let the Merchant pass in peace three times.",
    ),
    (
        "Deixe o jogo rodar sozinho na demonstração.",
        "Let the game play itself in demo mode.",
    ),
    ("Derrote a Rainha da Colmeia.", "Defeat the Hive Queen."),
    ("Derrote a Serpente Cometa.", "Defeat the Comet Serpent."),
    ("Derrote a Tecelã Abissal.", "Defeat the Abyssal Weaver."),
    ("Derrote as Gêmeas.", "Defeat the Twins."),
    ("Derrote o Cometa Wild.", "Defeat the Wild Comet."),
    (
        "Derrote o Couraçado Zênite.",
        "Defeat the Zenith Dreadnought.",
    ),
    ("Derrote o Devorador.", "Defeat the Devourer."),
    ("Derrote o Olho do Vazio.", "Defeat the Eye of the Void."),
    (
        "Derrote um chefe sem ser atingido.",
        "Defeat a boss without getting hit.",
    ),
    (
        "Derrube a Nave-Mãe misteriosa.",
        "Shoot down the mysterious Mothership.",
    ),
    ("Desafio completo!", "Challenge complete!"),
    ("Desafios diários vencidos", "Daily challenges won"),
    ("Desbloqueie as 10 naves.", "Unlock all 10 ships."),
    (
        "Desbloqueie as 7 naves clássicas.",
        "Unlock the 7 classic ships.",
    ),
    (
        "Desce em mergulho largando minas que explodem em leque. Atire nas minas!",
        "Dives while dropping mines that explode in a spread. Shoot the mines!",
    ),
    ("Desconfiado", "Suspicious"),
    (
        "Desligado: o fundo quase não se mexe.",
        "Off: the background barely moves.",
    ),
    (
        "Destrua 25 maldições a tiros.",
        "Destroy 25 curses with your shots.",
    ),
    (
        "Destrua todas as partes de um chefe antes do corpo.",
        "Destroy every part of a boss before its body.",
    ),
    ("Desvie das pedras!", "Dodge the rocks!"),
    (
        "Digite o código Konami na tela de título.",
        "Enter the Konami code on the title screen.",
    ),
    ("Digite seu nome:", "Enter your name:"),
    ("Dimensão Dourada", "Golden Dimension"),
    ("Disco de Ouro", "Golden Record"),
    (
        "Dispara teias pegajosas que deixam sua nave lenta por alguns segundos.",
        "Shoots sticky webs that slow your ship down for a few seconds.",
    ),
    (
        "Dispara vacas explosivas. Ninguém explica.",
        "Fires exploding cows. Nobody can explain it.",
    ),
    (
        "Divisores, Kamikazes, Escudeiros, Tecelãs,",
        "Splitters, Kamikazes, Shieldbearers, Weavers,",
    ),
    (
        "Do quê? ...isso fica para o próximo ciclo.",
        "From what? ...that's for the next cycle.",
    ),
    (
        "Dois drones atiram ao seu lado.",
        "Two drones fire at your side.",
    ),
    ("Dourados valem mais.", "Golden ones are worth more."),
    (
        "Duas princesas da colmeia que lutam juntas. Quando uma cai, a outra enlouquece.",
        "Two hive princesses who fight together. When one falls, the other goes berserk.",
    ),
    (
        "Duas sondas levam um disco de ouro. Diga o nome delas no ranking.",
        "Two probes carry a golden record. Enter their name in the high scores.",
    ),
    ("EFEITOS", "SOUND FX"),
    ("ELENCO", "CAST"),
    (
        "EM MEMÓRIA DA SONDA STARDUST",
        "IN MEMORY OF THE STARDUST PROBE",
    ),
    ("EMBOSCADA", "AMBUSH"),
    ("ENCERRAR A PARTIDA", "END THE RUN"),
    ("ENCERRAR PARTIDA", "END RUN"),
    (
        "ENTER confirma  ·  BACKSPACE apaga",
        "ENTER confirms  ·  BACKSPACE deletes",
    ),
    ("ENTER continua", "ENTER continues"),
    (
        "ENTER continua (perde metade dos pontos)",
        "ENTER continues (lose half your points)",
    ),
    ("ENTER equipa", "ENTER equips"),
    ("ENTER equipa o rastro", "ENTER equips the trail"),
    ("ENTER para continuar", "ENTER to continue"),
    ("ENTER para pular", "ENTER to skip"),
    ("ENTER pula", "ENTER skips"),
    ("EQUIPADA", "EQUIPPED"),
    ("ERROU!", "MISSED!"),
    ("ESC desiste", "ESC gives up"),
    ("ESCARAVELHO DOURADO", "GOLDEN SCARAB"),
    ("ESCARAVELHO DOURADO!", "GOLDEN SCARAB!"),
    ("ESCOLHA UMA BÊNÇÃO", "CHOOSE A BLESSING"),
    ("ESCUDEIRO", "SHIELDBEARER"),
    ("ESCUDO", "SHIELD"),
    ("ESCUDO ETERNO", "ETERNAL SHIELD"),
    ("ESCUDO REGENERADO", "SHIELD RESTORED"),
    ("ESCUDO!", "SHIELD!"),
    ("ESMERALDA", "EMERALD"),
    ("ESPAÇO para travar", "SPACE to lock"),
    ("ESPAÇOS:", "SLOTS:"),
    ("ESPELHO", "MIRROR"),
    ("ESPELHO!", "MIRROR!"),
    (
        "ESQ/DIR abas  ·  CIMA/BAIXO navega  ·  ESC volta",
        "LEFT/RIGHT tabs  ·  UP/DOWN browse  ·  ESC back",
    ),
    (
        "ESQ/DIR escolhe · ENTER confirma",
        "LEFT/RIGHT choose · ENTER confirm",
    ),
    ("ESQ/DIR troca de aba", "LEFT/RIGHT switch tab"),
    (
        "ESQ/DIR troca o modo  ·  ENTER volta",
        "LEFT/RIGHT change mode  ·  ENTER back",
    ),
    ("ESTILHAÇO", "SHRAPNEL"),
    ("ESTÁGIO BÔNUS", "BONUS STAGE"),
    ("EVENTOS DE SETOR", "SECTOR EVENTS"),
    ("EXORCIZADA!", "EXORCISED!"),
    ("EXPEDIÇÃO", "EXPEDITION"),
    ("Eclipse Total", "Total Eclipse"),
    (
        "Eclipse, meteoros, emboscadas, comboios...",
        "Eclipse, meteors, ambushes, convoys...",
    ),
    ("Ele parece pacífico...", "It looks peaceful..."),
    (
        "Ele quer o que está caindo. Derrube-o!",
        "It wants what's falling. Shoot it down!",
    ),
    (
        "Eles não atiram. Acerte todos!",
        "They don't shoot. Hit them all!",
    ),
    ("Elite da Elite", "Elite of the Elite"),
    (
        "Elites (borda dourada): fortes e ricos.",
        "Elites (gold outline): tough and rich.",
    ),
    (
        "Elites sempre soltam cápsula e às vezes uma carta.",
        "Elites always drop a capsule and sometimes a card.",
    ),
    ("Encantador de Serpentes", "Snake Charmer"),
    (
        "Enche a habilidade e acelera a recarga.",
        "Fills the ability and speeds up recharge.",
    ),
    ("Encontre 20 cartas diferentes.", "Find 20 different cards."),
    ("Encontre 6 registros de bordo.", "Find 6 flight logs."),
    ("Encontre a Dimensão Dourada.", "Find the Golden Dimension."),
    ("Encontre as 4 Relíquias.", "Find the 4 Relics."),
    (
        "Encontre todas as cartas comuns, raras, épicas e lendárias.",
        "Find every common, rare, epic and legendary card.",
    ),
    (
        "Encontre todos os 12 registros de bordo.",
        "Find all 12 flight logs.",
    ),
    ("Encontre uma Relíquia.", "Find a Relic."),
    ("Encontre uma Sala Secreta.", "Find a Secret Room."),
    (
        "Engole tiros inimigos e tritura quem chega perto.",
        "Swallows enemy shots and grinds anything that gets close.",
    ),
    (
        "Engula 300 tiros com buracos negros no total.",
        "Swallow 300 shots with black holes in total.",
    ),
    ("Entre em Febre", "Enter Fever"),
    (
        "Entre em Febre 3 vezes numa partida.",
        "Enter Fever 3 times in one run.",
    ),
    (
        "Entre em Febre com um combo alto.",
        "Enter Fever with a high combo.",
    ),
    (
        "Entre nele para pular o chefe e o setor!",
        "Fly into it to skip the boss and the sector!",
    ),
    (
        "Entre os setores, gaste cristais em armas",
        "Between sectors, spend crystals on better",
    ),
    ("Enxame", "Swarm"),
    (
        "Equilibrada e confiável. A nave de todo recruta.",
        "Balanced and reliable. Every recruit's ship.",
    ),
    (
        "Equipe no menu CARTAS. Com 8, 16 e 30",
        "Equip them in the CARDS menu. With 8, 16 and 30",
    ),
    ("Era do Gelo", "Ice Age"),
    (
        "Ergue por 5 s uma barreira que bloqueia tiros.",
        "Raises a shot-blocking barrier for 5 s.",
    ),
    ("Esc / P ............ pausa", "Esc / P ............ pause"),
    ("Esc ou P", "Esc or P"),
    ("Escolta", "Escort"),
    (
        "Escondemos o tesouro da colmeia entre dimensões. Só quem nunca cai consegue achar a porta.",
        "We hid the hive's treasure between dimensions. Only those who never fall can find the door.",
    ),
    (
        "Escopeta: muito dano de perto.",
        "Shotgun: huge damage up close.",
    ),
    (
        "Escudeiros só protegem a frente. Tiros em diagonal passam.",
        "Shieldbearers only protect their front. Diagonal shots get through.",
    ),
    ("Escudo Inicial", "Starting Shield"),
    ("Escudo Reativo", "Reactive Shield"),
    (
        "Espaço / Z / J ..... atirar (aperte)",
        "Space / Z / J ...... fire (tap)",
    ),
    (
        "Esquerda vira direita. Boa sorte.",
        "Left becomes right. Good luck.",
    ),
    ("Estilhaço Térmico", "Thermal Shrapnel"),
    (
        "Estrelas cintilantes em leque. Secreta.",
        "Twinkling stars in a fan. Secret.",
    ),
    (
        "Estágios bônus valem o dobro.",
        "Bonus stages are worth double.",
    ),
    (
        "Etérea. Hitbox menor e domínio sobre o tempo.",
        "Ethereal. Smaller hitbox and mastery over time.",
    ),
    ("Exorcista", "Exorcist"),
    ("Expedicionário", "Expeditionary"),
    ("Experiência total", "Total experience"),
    (
        "Explode e acerta tudo em volta.",
        "Explodes and hits everything around it.",
    ),
    ("Explosões 40% maiores.", "Explosions 40% bigger."),
    (
        "F  Frenesi: atira o dobro",
        "F  Frenzy: fires twice as fast",
    ),
    (
        "F11 / Alt+Enter .... tela cheia",
        "F11 / Alt+Enter .... fullscreen",
    ),
    ("F11 ou Alt+Enter", "F11 or Alt+Enter"),
    ("FALCÃO", "FALCON"),
    ("FANTASMA", "PHANTOM"),
    ("FEBRE", "FEVER"),
    ("FEBRE DO OURO", "GOLD RUSH"),
    ("FEBRE!", "FEVER!"),
    ("FEITO EM", "MADE WITH"),
    ("FELIZ NATAL, PILOTO!", "MERRY CHRISTMAS, PILOT!"),
    ("FENDA", "RIFT"),
    ("FENDA ESTELAR", "STAR RIFT"),
    ("FENDAS", "RIFTS"),
    ("FENDAS (hexágono com ?)", "RIFTS (hexagon with ?)"),
    ("FERRUGEM", "RUST"),
    ("FERRUGEM!", "RUST!"),
    ("FILHOTE", "HATCHLING"),
    ("FILTRO CRT", "CRT FILTER"),
    ("FIM DE JOGO", "GAME OVER"),
    ("FOCO", "FOCUS"),
    ("FOGO", "FIRE"),
    ("FRENESI", "FRENZY"),
    ("FRENESI!", "FRENZY!"),
    ("FUNDO ANIMADO", "ANIMATED BACKGROUND"),
    ("Fazendeiro", "Farmer"),
    (
        "Faça 1.000.000 de pontos numa partida.",
        "Score 1,000,000 points in one run.",
    ),
    (
        "Faça 100 raspões numa partida.",
        "Make 100 grazes in one run.",
    ),
    (
        "Faça 60.000 pontos no modo Clássico.",
        "Score 60,000 points in Classic mode.",
    ),
    ("Faça um combo de 100.", "Make a 100 combo."),
    ("Faça um combo de 50.", "Make a 50 combo."),
    ("Febre Longa", "Long Fever"),
    ("Febre Precoce", "Early Fever"),
    ("Febre Tripla", "Triple Fever"),
    ("Febre!", "Fever!"),
    (
        "Feitos liberam naves e rastros. Veja nos",
        "Feats unlock ships and trails. See the",
    ),
    (
        "Feitos raros são lembrados aqui.",
        "Rare feats are remembered here.",
    ),
    (
        "Feixes que atravessam inimigos.",
        "Beams that pierce through enemies.",
    ),
    ("Fenda Aberta", "Open Rift"),
    (
        "Fendas de minigame caem 2,5x mais.",
        "Minigame rifts drop 2.5x more often.",
    ),
    (
        "Fendas de minigame caem 2x mais.",
        "Minigame rifts drop 2x more often.",
    ),
    ("Ferro-Velho", "Scrapyard"),
    ("Final Verdadeiro", "True Ending"),
    ("Fliperama", "Arcade"),
    ("Fliperamista", "Arcade Master"),
    ("Freguês", "Regular Customer"),
    ("Frota Completa", "Full Fleet"),
    ("FÁCIL", "EASY"),
    ("FÊNIX!", "PHOENIX!"),
    ("Fênix", "Phoenix"),
    ("GANÂNCIA", "GREED"),
    ("GATILHO", "TRIGGER"),
    ("GATILHO LEVE", "LIGHT TRIGGER"),
    ("GELO", "ICE"),
    ("GRANADA", "GRENADE"),
    ("GRÁFICOS", "GRAPHICS"),
    ("Ganha 1 bomba a cada setor.", "Gain 1 bomb every sector."),
    (
        "Gaste seus cristais antes do próximo setor",
        "Spend your crystals before the next sector",
    ),
    ("Gatilho Leve", "Light Trigger"),
    ("Guardião das Relíquias", "Keeper of Relics"),
    ("Gêmeas Separadas", "Twins Apart"),
    ("HABILIDADE  [C]", "ABILITY  [C]"),
    ("HABILIDADES", "ABILITIES"),
    ("HERÓI DA GALÁXIA", "GALAXY HERO"),
    ("HITBOX SEMPRE VISÍVEL", "ALWAYS SHOW HITBOX"),
    ("HONRAS", "HONORS"),
    ("Habilidade", "Ability"),
    (
        "Habilidade carrega 30% mais rápido.",
        "Ability charges 30% faster.",
    ),
    ("Hangar Cheio", "Full Hangar"),
    ("Herança", "Inheritance"),
    ("Historiador", "Historian"),
    ("Horizonte", "Horizon"),
    ("Horizonte de Eventos", "Event Horizon"),
    ("INIMIGOS VELOZES", "FAST ENEMIES"),
    (
        "INSPIRADO PELOS SHAREWARES",
        "INSPIRED BY THE SHAREWARE GAMES",
    ),
    ("INSÍGNIA ???", "INSIGNIA ???"),
    ("INSÍGNIA DA EXPEDIÇÃO", "EXPEDITION INSIGNIA"),
    ("INSÍGNIA DA RELÍQUIA", "RELIC INSIGNIA"),
    ("INSÍGNIA DO ARSENAL", "ARSENAL INSIGNIA"),
    ("INSÍGNIA DO EXORCISTA", "EXORCIST INSIGNIA"),
    ("INSÍGNIA DO EXPLORADOR", "EXPLORER INSIGNIA"),
    ("INSÍGNIA DO FLIPERAMA", "ARCADE INSIGNIA"),
    ("INSÍGNIA DO INTOCÁVEL", "UNTOUCHABLE INSIGNIA"),
    ("INSÍGNIA DO PERFECCIONISTA", "PERFECTIONIST INSIGNIA"),
    ("INSÍGNIA DO VAZIO", "VOID INSIGNIA"),
    ("INSÍGNIA SUPREMA", "SUPREME INSIGNIA"),
    ("INSÍGNIAS DE HONRA", "HONOR INSIGNIAS"),
    ("INTOCÁVEL", "UNTOUCHABLE"),
    ("INVENCÍVEL", "INVINCIBLE"),
    ("INVENCÍVEL!", "INVINCIBLE!"),
    ("Inimigos abatidos", "Enemies shot down"),
    (
        "Inimigos congelados levam dano em dobro.",
        "Frozen enemies take double damage.",
    ),
    (
        "Inimigos em mergulho valem o dobro de pontos.",
        "Diving enemies are worth double points.",
    ),
    (
        "Inimigos em mergulho valem o triplo.",
        "Diving enemies are worth triple.",
    ),
    (
        "Inimigos ficam congelados o dobro do tempo.",
        "Enemies stay frozen twice as long.",
    ),
    (
        "Inimigos soltam mais cristais.",
        "Enemies drop more crystals.",
    ),
    ("Insistente", "Persistent"),
    ("Insone", "Sleepless"),
    ("Insígnia do Fliperama", "Arcade Insignia"),
    ("Insígnia do Perfeccionista", "Perfectionist Insignia"),
    ("Intocável", "Untouchable"),
    ("Investidor", "Investor"),
    (
        "Itens raros na loja e arma COLMEIA",
        "Rare shop items and the HIVE weapon",
    ),
    ("JOGAR", "PLAY"),
    ("JOGO", "GAME"),
    ("Janela do combo 50% maior.", "Combo window 50% longer."),
    ("Jogue 25 partidas.", "Play 25 runs."),
    ("Jogue por 2 horas no total.", "Play for 2 hours in total."),
    (
        "Junte as 8: vida extra e muitos pontos.",
        "Collect all 8: extra life and tons of points.",
    ),
    (
        "Junte as letras S-T-A-R-D-U-S-T.",
        "Collect the letters S-T-A-R-D-U-S-T.",
    ),
    ("JÁ TEM", "OWNED"),
    ("LADRÃO", "THIEF"),
    ("LADRÃO!", "THIEF!"),
    ("LANÇA SOLAR", "SOLAR LANCE"),
    ("LENDA ESTELAR", "STAR LEGEND"),
    ("LENDÁRIA", "LEGENDARY"),
    ("LENTO", "SLOW"),
    ("LETRAS", "LETTERS"),
    ("LETRAS S-T-A-R-D-U-S-T", "LETTERS S-T-A-R-D-U-S-T"),
    ("LOJA", "SHOP"),
    ("LOJA E PODER", "SHOP AND POWER"),
    ("Ladrões ficam 40% mais lentos.", "Thieves are 40% slower."),
    (
        "Ladrões fogem com o que cai. Derrube-os antes que sumam.",
        "Thieves run off with falling items. Shoot them before they vanish.",
    ),
    (
        "Ladrões: roubam o que cai. Derrube-os!",
        "Thieves: they steal falling items. Shoot them!",
    ),
    (
        "Lança-chamas de curto alcance.",
        "Short-range flamethrower.",
    ),
    (
        "Lenta e blindada: +1 vida e começa com escudo.",
        "Slow and armored: +1 life and starts with a shield.",
    ),
    ("Leque largo de tiros rápidos.", "Wide fan of fast shots."),
    (
        "Leve 12 armas diferentes ao nível 5.",
        "Take 12 different weapons to level 5.",
    ),
    ("Leve 5 armas ao nível 5.", "Take 5 weapons to level 5."),
    (
        "Leve 5 armas diferentes ao nível 5.",
        "Take 5 different weapons to level 5.",
    ),
    (
        "Leve duas armas diferentes ao nível 5 na mesma partida.",
        "Take two different weapons to level 5 in the same run.",
    ),
    (
        "Limpar uma onda rápido rende bônus de tempo.",
        "Clearing a wave quickly earns a time bonus.",
    ),
    (
        "Limpe uma onda em menos de 7 segundos.",
        "Clear a wave in under 7 seconds.",
    ),
    (
        "Lâminas que vão e voltam.",
        "Blades that fly out and come back.",
    ),
    ("MAPA DO TESOURO", "TREASURE MAP"),
    ("MARECHAL ESTELAR", "STAR MARSHAL"),
    ("MARÉ GRAVITACIONAL", "GRAVITY TIDE"),
    ("MELHORIA DE ARMA", "WEAPON UPGRADE"),
    ("MERCADOR", "MERCHANT"),
    ("MERCENÁRIO", "MERCENARY"),
    ("MINA", "MINE"),
    ("MINADOR", "MINELAYER"),
    ("MINAS", "MINES"),
    ("MIRA LASER", "LASER SIGHT"),
    ("MODO DE JOGO", "GAME MODE"),
    ("MOSTRAR FPS", "SHOW FPS"),
    ("MURALHA", "BULWARK"),
    ("Maior combo", "Best combo"),
    ("Mais longe na Expedição", "Farthest in Expedition"),
    (
        "Mais power-ups até o fim da partida.",
        "More power-ups until the end of the run.",
    ),
    ("Maldições caem 50% menos.", "Curses drop 50% less often."),
    ("Maldições exorcizadas / pegas", "Curses exorcised / taken"),
    ("Maratona", "Marathon"),
    (
        "Mas nem todas as perguntas foram respondidas.",
        "But not every question was answered.",
    ),
    (
        "Mascarado e rápido. Rouba o que cai na tela e foge. Derrube-o para recuperar em dobro.",
        "Masked and fast. Steals what falls and flees. Shoot it down to get it back doubled.",
    ),
    ("Melhor onda (sobrevivência)", "Best wave (survival)"),
    ("Melhor setor", "Best sector"),
    (
        "Memorize as estrelas que acendem",
        "Memorize the stars that light up",
    ),
    (
        "Menos partículas deixa o jogo mais leve.",
        "Fewer particles make the game lighter.",
    ),
    ("Mercadores poupados", "Merchants spared"),
    ("Mercenário de Elite", "Elite Mercenary"),
    (
        "Mergulhos 40% mais rápidos, pontos x2.",
        "Dives 40% faster, points x2.",
    ),
    (
        "Mesma arma sobe de nível · outra arma substitui a atual",
        "Same weapon levels up · a new one replaces it",
    ),
    ("Mestre Supremo", "Supreme Master"),
    ("Mestre de Armas", "Weapon Master"),
    (
        "Metade das maldições não te pega mais.",
        "Half of the curses can't touch you anymore.",
    ),
    (
        "Metade das maldições não te pega.",
        "Half of the curses can't touch you.",
    ),
    (
        "Milhões de insetos estelares cercaram a Terra em formação de ataque.",
        "Millions of star insects surrounded Earth in attack formation.",
    ),
    ("Milionário", "Millionaire"),
    (
        "Minadores largam minas: atire nelas.",
        "Minelayers drop mines: shoot them.",
    ),
    (
        "Minadores, Saltadores e um Cometa domado",
        "Minelayers, Jumpers and a tamed Comet",
    ),
    (
        "Minas piscam cada vez mais rápido antes de explodir.",
        "Mines blink faster and faster before exploding.",
    ),
    (
        "Minhas filhas lutam juntas. Disse a elas para não odiarem o piloto. Ele só não sabe.",
        "My daughters fight together. I told them not to hate the pilot. The pilot just doesn't know.",
    ),
    ("Minigames vencidos", "Minigames won"),
    ("Modo CHEFÕES e arma CRIO", "BOSS RUSH mode and CRYO weapon"),
    ("Modo CLÁSSICO", "CLASSIC mode"),
    ("Modo preciso", "Precision mode"),
    (
        "Mostra sempre o ponto vulnerável da nave.",
        "Always shows your ship's weak spot.",
    ),
    ("Motor Turbo", "Turbo Engine"),
    ("Mover", "Move"),
    ("Multiplicador Dourado", "Golden Multiplier"),
    ("Muuu!", "Moo!"),
    ("MÁXIMO", "MAX"),
    ("MÉDIO", "MEDIUM"),
    ("MÍMICO", "MIMIC"),
    ("MÍMICO!", "MIMIC!"),
    ("MÍSSIL", "MISSILE"),
    ("MÚSICA", "MUSIC"),
    (
        "Mãe de todo o Enxame. Suas garras disparam sozinhas e ela chama lacaios.",
        "Mother of the whole Swarm. Her claws fire on their own and she calls minions.",
    ),
    ("Mão Leve", "Light Fingers"),
    (
        "Mímicos aparecem marcados em vermelho.",
        "Mimics show up marked in red.",
    ),
    (
        "Mímicos: cápsulas que mordem.",
        "Mimics: capsules that bite.",
    ),
    ("Mísseis teleguiados e fortes.", "Strong homing missiles."),
    ("NAVE", "SHIP"),
    ("NAVES", "SHIPS"),
    ("NEBLINA", "FOG"),
    ("NEBULOSA CARMESIM", "CRIMSON NEBULA"),
    ("NOME", "NAME"),
    ("NORMAIS", "NORMAL"),
    ("NOVO RECORDE!", "NEW HIGH SCORE!"),
    ("NOVO REGISTRO DE BORDO", "NEW FLIGHT LOG"),
    ("NOVO REGISTRO NO BESTIÁRIO", "NEW BESTIARY ENTRY"),
    (
        "Na Expedição, cada bênção muda a sua estratégia.",
        "In Expedition, every blessing changes your strategy.",
    ),
    ("Na Raspa", "Close Shave"),
    (
        "Na poeira havia um sinal. E o sinal chamou o ENXAME.",
        "In the dust there was a signal. And the signal called the SWARM.",
    ),
    (
        "Nas cinzas da colmeia, uma última verdade:",
        "In the ashes of the hive, one last truth:",
    ),
    (
        "Nasce dos Divisores e das Portadoras. Pequeno, rápido e teimoso.",
        "Born from Splitters and Carriers. Small, fast and stubborn.",
    ),
    ("Nave 12% mais rápida.", "Ship 12% faster."),
    ("Nave 8% mais rápida.", "Ship 8% faster."),
    (
        "Nave BASTIÃO e arma RICOCHETE",
        "BASTION ship and RICOCHET weapon",
    ),
    ("Nave COLOSSO", "COLOSSUS ship"),
    ("Nave CORSÁRIA", "CORSAIR ship"),
    (
        "Nave FALCÃO e DESAFIO DIÁRIO",
        "FALCON ship and DAILY CHALLENGE",
    ),
    ("Nave FANTASMA", "PHANTOM ship"),
    (
        "Nave PROTÓTIPO liberada no Hangar. +1 carta secreta.",
        "PROTOTYPE ship unlocked in the Hangar. +1 secret card.",
    ),
    (
        "Nave SONDA e rastro POEIRA ESTELAR",
        "PROBE ship and STARDUST trail",
    ),
    ("Nave VEGA e arma GRANADA", "VEGA ship and GRENADE weapon"),
    (
        "Nave de carga do Enxame. Não atira e está lotada de cristais.",
        "Swarm cargo ship. Doesn't shoot and is packed with crystals.",
    ),
    ("Nave pesada por 8 segundos.", "Heavy ship for 8 seconds."),
    ("Nave ÉGIDE", "AEGIS ship"),
    ("Naves perdidas", "Ships lost"),
    (
        "Nenhum tiro disparado. Respeito.",
        "Not a single shot fired. Respect.",
    ),
    (
        "Nenhuma nave perdida... o Enxame não te viu chegar.",
        "No ship lost... the Swarm never saw you coming.",
    ),
    (
        "Ninguém sabe como ela chegou ao espaço. Ninguém tem coragem de perguntar.",
        "Nobody knows how it got to space. Nobody dares to ask.",
    ),
    (
        "Ninguém sabe o que o Enxame quer. Boa sorte, piloto.",
        "Nobody knows what the Swarm wants. Good luck, pilot.",
    ),
    (
        "No Cinturão de Gelo, sua nave escorrega. Antecipe-se.",
        "In the Ice Belt your ship slides. Plan ahead.",
    ),
    (
        "Nunca fica no mesmo lugar da formação. Some num piscar e reaparece longe.",
        "Never stays in the same spot of the formation. Vanishes in a blink and reappears far away.",
    ),
    ("NÃO", "NO"),
    ("NÃO FOI DESSA VEZ", "NOT THIS TIME"),
    ("NÚCLEO DE DANO", "DAMAGE CORE"),
    ("NÚMEROS", "STATS"),
    ("Não Pisque", "Don't Blink"),
    (
        "Não atira. Não desvia. Só quer encostar em você.",
        "Doesn't shoot. Doesn't dodge. Just wants to touch you.",
    ),
    ("O AMULETO PROTEGEU!", "THE AMULET PROTECTED YOU!"),
    ("O DEVORADOR", "THE DEVOURER"),
    (
        "O Devorador recuou para o escuro entre as estrelas.",
        "The Devourer retreated into the dark between the stars.",
    ),
    ("O LADRÃO FUGIU", "THE THIEF GOT AWAY"),
    (
        "O Mercador nunca vai esquecer isso.",
        "The Merchant will never forget this.",
    ),
    (
        "O Mercador te olha torto: preços +30%",
        "The Merchant glares at you: prices +30%",
    ),
    (
        "O Mercador vende para todos os lados. Ele sabe do Devorador. Ele sempre soube.",
        "The Merchant sells to every side. He knows about the Devourer. He always knew.",
    ),
    (
        "O NÚCLEO DE DANO da loja deixa todas as armas mais fortes.",
        "The shop's DAMAGE CORE makes every weapon stronger.",
    ),
    ("O OBJETIVO", "THE GOAL"),
    (
        "O Olho foi feito para vigiar o escuro. Ele viu. Desde então não consegue mais fechar direito.",
        "The Eye was made to watch the dark. It saw. Since then it can't quite close.",
    ),
    (
        "O Olho só sente dor com a pálpebra aberta.",
        "The Eye only feels pain with its eyelid open.",
    ),
    ("O VAZIO ALÉM", "THE VOID BEYOND"),
    ("O VAZIO SE ABRE", "THE VOID OPENS"),
    ("O Vazio Olha de Volta", "The Void Stares Back"),
    (
        "O coração do Enxame foi destruído.",
        "The heart of the Swarm has been destroyed.",
    ),
    (
        "O coração do Enxame. Coroada, furiosa e com todos os truques das irmãs.",
        "The heart of the Swarm. Crowned, furious and with all her sisters' tricks.",
    ),
    (
        "O escudo volta depois de 25 s sem levar dano.",
        "The shield comes back after 25 s without taking damage.",
    ),
    (
        "O relicário ficou para trás.",
        "The reliquary was left behind.",
    ),
    ("OBSERVE...", "WATCH..."),
    ("OLHO ANCESTRAL", "ANCIENT EYE"),
    ("OLHO DO VAZIO", "EYE OF THE VOID"),
    ("OLÁ, CRIADOR!", "HELLO, CREATOR!"),
    ("ONDA", "WAVE"),
    ("ONDA LIMPA!", "WAVE CLEAR!"),
    ("OPÇÕES", "OPTIONS"),
    ("OURO", "GOLD"),
    (
        "Obrigada, piloto. O Devorador recuou. A colmeia vai sobreviver. E agora você também sabe.",
        "Thank you, pilot. The Devourer retreated. The hive will survive. And now you know too.",
    ),
    (
        "Obrigado pela gentileza, piloto!",
        "Thanks for your kindness, pilot!",
    ),
    (
        "Obrigado por fazer este jogo.",
        "Thanks for making this game.",
    ),
    (
        "Obrigado por jogar até o fim, piloto.",
        "Thanks for playing to the very end, pilot.",
    ),
    ("Obrigado por jogar, piloto.", "Thanks for playing, pilot."),
    ("Olho Clínico", "Keen Eye"),
    ("Olho do Devorador", "Eye of the Devourer"),
    ("Onda Relâmpago", "Lightning Wave"),
    (
        "Ondas sem pausa, cada vez mais rápidas. Quanto você aguenta?",
        "Non-stop waves, faster and faster. How long can you last?",
    ),
    (
        "Os Besouros estão... diferentes hoje.",
        "The Beetles look... different today.",
    ),
    (
        "Os tiros inimigos curvam na sua direção.",
        "Enemy shots curve toward you.",
    ),
    ("Ouro de Tolo", "Fool's Gold"),
    ("PACIFISTA", "PACIFIST"),
    ("PAINÉIS LATERAIS", "SIDE PANELS"),
    ("PAIOL", "ARSENAL"),
    ("PANE", "JAM"),
    ("PANE!", "JAM!"),
    ("PARTÍCULAS", "PARTICLES"),
    ("PATENTES", "RANKS"),
    ("PAUSA", "PAUSE"),
    ("PERFEITO!", "PERFECT!"),
    ("PESADELO", "NIGHTMARE"),
    ("PILOTO", "PILOT"),
    ("PILOTO DE ELITE", "ELITE PILOT"),
    ("PIRATA ESPACIAL", "SPACE PIRATE"),
    ("PODER DE FOGO", "FIREPOWER"),
    ("PODERES (bolinhas que caem)", "POWER-UPS (falling orbs)"),
    ("POEIRA ESTELAR", "COSMIC DUST"),
    ("PONTOS", "SCORE"),
    ("PORTADORA", "CARRIER"),
    ("PORTAL DE SALTO", "WARP GATE"),
    ("PRESENTE DO MERCADOR", "MERCHANT'S GIFT"),
    ("PRISMA", "PRISM"),
    ("PROGRESSO APAGADO", "PROGRESS ERASED"),
    ("PROMOVIDO!", "PROMOTED!"),
    ("PROPULSOR", "THRUSTER"),
    ("PROPULSORES", "THRUSTERS"),
    ("PROTÓTIPO", "PROTOTYPE"),
    ("PULSO ESTELAR", "STAR PULSE"),
    ("Pacifista", "Pacifist"),
    (
        "Painéis com pontos e arma nas laterais (telas largas).",
        "Panels with score and weapon on the sides (wide screens).",
    ),
    (
        "Palavras STARDUST / Naves-Mãe",
        "STARDUST words / Motherships",
    ),
    (
        "Parece uma cápsula. Não é. Quando você chega perto, ele abre a boca.",
        "Looks like a capsule. It isn't. When you get close, it opens its mouth.",
    ),
    (
        "Parte do Enxame chega por baixo!",
        "Part of the Swarm comes from below!",
    ),
    ("Partidas", "Runs"),
    (
        "Patentes que não vêm de XP: vêm de feitos.",
        "Ranks that don't come from XP: they come from feats.",
    ),
    ("Pausa", "Pause"),
    (
        "Pedras despencam por todo o setor.",
        "Rocks rain down all over the sector.",
    ),
    ("Pega Ladrão", "Thief Catcher"),
    (
        "Pegar a mesma arma de novo sobe o nível dela.",
        "Picking up the same weapon again raises its level.",
    ),
    (
        "Pegue 10 maldições. De propósito?",
        "Pick up 10 curses. On purpose?",
    ),
    ("Pegue antes que ela se perca.", "Grab it before it's lost."),
    ("Pegue os cristais que caem.", "Catch the falling crystals."),
    (
        "Perder uma nave não reduz o nível da arma.",
        "Losing a ship doesn't lower your weapon level.",
    ),
    ("Perfeccionista", "Perfectionist"),
    ("Perfeição", "Perfection"),
    (
        "Pesada e paciente: +1 vida e tiros 10% mais fortes.",
        "Heavy and patient: +1 life and 10% stronger shots.",
    ),
    ("Pesadelo Desperto", "Nightmare Awakened"),
    ("Pescador de Cartas", "Card Fisher"),
    ("Pintura ESMERALDA", "EMERALD paint"),
    ("Pintura GELO", "ICE paint"),
    (
        "Pintura LENDÁRIA e nome dourado no ranking",
        "LEGENDARY paint and a golden name in the high scores",
    ),
    (
        "Pintura LENDÁRIA para todas as naves",
        "LEGENDARY paint for every ship",
    ),
    (
        "Pintura OURO e arma BURACO NEGRO",
        "GOLD paint and BLACK HOLE weapon",
    ),
    ("Pintura RUBI e arma ÓRBITA", "RUBY paint and ORBIT weapon"),
    ("Pintura SOMBRA", "SHADOW paint"),
    ("Pirata Espacial", "Space Pirate"),
    ("Piromaníaco", "Pyromaniac"),
    (
        "Pixel art escrita letra por letra",
        "Pixel art written letter by letter",
    ),
    ("Poeira de Cometa", "Comet Dust"),
    ("Pontaria", "Aim"),
    (
        "Pontos em dobro e cristais a cada abate!",
        "Double points and crystals on every kill!",
    ),
    (
        "Pontos x1,3 e tiros inimigos 10% mais lentos.",
        "Points x1.3 and enemy shots 10% slower.",
    ),
    (
        "Pontos x1,5 e combo máximo x10.",
        "Points x1.5 and max combo x10.",
    ),
    (
        "Portadoras, Camaleões, Mímicos, Ladrões,",
        "Carriers, Chameleons, Mimics, Thieves,",
    ),
    ("Português ou inglês.", "Portuguese or English."),
    ("Power-ups caem 50% mais.", "Power-ups drop 50% more often."),
    ("Primeiro Voo", "First Flight"),
    ("Prêmio dobrado e uma carta!", "Double reward and a card!"),
    (
        "Prêmio: cristais, nível de arma e um poder.",
        "Reward: crystals, a weapon level and a power-up.",
    ),
    (
        "Puxa cristais de mais longe.",
        "Pulls crystals from farther away.",
    ),
    ("Pólvora Extra", "Extra Gunpowder"),
    (
        "Q/TAB troca de aba  ·  ESQ/DIR ajusta  ·  ESC volta",
        "Q/TAB switch tab  ·  LEFT/RIGHT adjust  ·  ESC back",
    ),
    (
        "Quando morre, se divide em dois filhotes famintos que caçam a sua nave.",
        "When it dies, it splits into two hungry hatchlings that hunt your ship.",
    ),
    (
        "Quanto mais cara a arma, mais forte.",
        "The pricier the weapon, the stronger it is.",
    ),
    (
        "Que buscou poeira de cometa",
        "It went looking for comet dust",
    ),
    ("Quem fez este jogo?", "Who made this game?"),
    (
        "Quem sabe o que tem dentro? Quase sempre é bom.",
        "Who knows what's inside? It's almost always good.",
    ),
    ("RADAR DE CARTAS", "CARD RADAR"),
    ("RAINHA DA COLMEIA", "HIVE QUEEN"),
    ("RAINHA SUPREMA", "SUPREME QUEEN"),
    ("RAINHA SUPREMA DO ENXAME", "SUPREME QUEEN OF THE SWARM"),
    ("RAJADA", "BLASTER"),
    ("RARA", "RARE"),
    ("RASANTE", "DASH"),
    ("RASTRO EQUIPADO", "TRAIL EQUIPPED"),
    ("RECORDE", "HIGH SCORE"),
    ("RECORDES", "HIGH SCORES"),
    ("RECRUTA", "RECRUIT"),
    ("RECUPERADO EM DOBRO!", "RECOVERED TWICE OVER!"),
    ("REFLEXO", "REFLECT"),
    ("REGISTRO 1 - A RAINHA", "LOG 1 - THE QUEEN"),
    ("REGISTRO 10 - A VACA", "LOG 10 - THE COW"),
    ("REGISTRO 11 - O MERCADOR", "LOG 11 - THE MERCHANT"),
    ("REGISTRO 12 - DEPOIS", "LOG 12 - AFTERWARD"),
    ("REGISTRO 2 - O COURAÇADO", "LOG 2 - THE DREADNOUGHT"),
    ("REGISTRO 3 - A SERPENTE", "LOG 3 - THE SERPENT"),
    ("REGISTRO 4 - A TECELÃ", "LOG 4 - THE WEAVER"),
    ("REGISTRO 5 - O OLHO", "LOG 5 - THE EYE"),
    ("REGISTRO 6 - O ÔMEGA", "LOG 6 - THE OMEGA"),
    ("REGISTRO 7 - AS GÊMEAS", "LOG 7 - THE TWINS"),
    ("REGISTRO 8 - O CORAÇÃO", "LOG 8 - THE HEART"),
    ("REGISTRO 9 - DIMENSÃO DOURADA", "LOG 9 - GOLDEN DIMENSION"),
    ("REGISTRO ???", "LOG ???"),
    ("REGISTROS", "LOGS"),
    ("REGISTROS DE BORDO", "FLIGHT LOGS"),
    ("RELATÓRIO DE VOO", "FLIGHT REPORT"),
    ("RELÍQUIA", "RELIC"),
    ("RELÍQUIA DESCOBERTA!", "RELIC DISCOVERED!"),
    (
        "RELÍQUIA. Buracos negros em série e tiros que dobram o espaço.",
        "RELIC. Black holes in a row and shots that bend space.",
    ),
    (
        "RELÍQUIA. Estrelas perfurantes e explosões em anel.",
        "RELIC. Piercing stars and ring explosions.",
    ),
    (
        "RELÍQUIA. Um feixe de sol que atravessa a tela inteira.",
        "RELIC. A beam of sunlight that crosses the whole screen.",
    ),
    (
        "RELÍQUIA. Uma nuvem de poeira estelar que caça tudo.",
        "RELIC. A cloud of cosmic dust that hunts everything.",
    ),
    ("RELÍQUIAS", "RELICS"),
    ("RESULTADO DO BÔNUS", "BONUS RESULT"),
    ("REVER O PRÓLOGO", "REPLAY THE PROLOGUE"),
    ("RICOCHETE", "RICOCHET"),
    ("ROUBADO!", "STOLEN!"),
    ("RUBI", "RUBY"),
    ("RUÍNAS ANCESTRAIS", "ANCIENT RUINS"),
    ("Rainha das Abelhas", "Queen Bee"),
    (
        "Raio do ímã de cristais +60%.",
        "Crystal magnet radius +60%.",
    ),
    (
        "Raios que saltam entre inimigos.",
        "Bolts that jump between enemies.",
    ),
    (
        "Raríssimo. Foge rápido, mas carrega uma fortuna em cristais.",
        "Very rare. Flees fast, but carries a fortune in crystals.",
    ),
    (
        "Raspar nos tiros inimigos enche a barra de habilidade.",
        "Grazing enemy shots fills the ability bar.",
    ),
    ("Raspão Perigoso", "Dangerous Graze"),
    ("Raspões", "Grazes"),
    (
        "Raspões valem o dobro de pontos e de energia.",
        "Grazes give double points and energy.",
    ),
    ("Rastro ARCO-ÍRIS", "RAINBOW trail"),
    ("Rastro FOGO", "FIRE trail"),
    ("Rastro GELO", "ICE trail"),
    ("Rastro OURO", "GOLD trail"),
    ("Rastro PLASMA", "PLASMA trail"),
    ("Rastro VAZIO", "VOID trail"),
    ("Recarga", "Recharge"),
    (
        "Recompensas de contratos em dobro.",
        "Double contract rewards.",
    ),
    (
        "Reforçamos o Couraçado. Se o piloto humano chegar até aqui, talvez seja forte o bastante.",
        "We reinforced the Dreadnought. If the human pilot makes it this far, maybe they're strong enough.",
    ),
    ("Regicida", "Regicide"),
    (
        "Registros caem dos chefes da campanha e de",
        "Logs drop from campaign bosses and from",
    ),
    ("Relicário", "Reliquary"),
    (
        "Rende 15% dos cristais a cada setor.",
        "Earns 15% of your crystals every sector.",
    ),
    ("Ricochete Afiado", "Sharp Ricochet"),
    ("Rotina", "Routine"),
    (
        "Rápida e gananciosa: +20% de cristais.",
        "Fast and greedy: +20% crystals.",
    ),
    (
        "Rápida e nervosa. Gosta de mergulhar em espiral antes de voltar à formação.",
        "Fast and jumpy. Loves to dive in spirals before returning to formation.",
    ),
    (
        "Rápida e pequena, mas com uma vida a menos.",
        "Fast and small, but with one less life.",
    ),
    (
        "S  Escudo   B  Bomba   $  Cristais",
        "S  Shield   B  Bomb   $  Crystals",
    ),
    ("SAIR", "QUIT"),
    ("SALA SECRETA", "SECRET ROOM"),
    ("SALTADOR", "JUMPER"),
    ("SALTO ESPACIAL!", "SPACE WARP!"),
    ("SAQUE", "LOOT"),
    ("SARGENTO", "SERGEANT"),
    ("SEGURO DE ARMA", "WEAPON INSURANCE"),
    ("SEM ESPAÇO: DESEQUIPE OUTRA", "NO SLOT: UNEQUIP ANOTHER"),
    ("SEM RASTRO", "NO TRAIL"),
    ("SEQUÊNCIA DAS ESTRELAS", "STAR SEQUENCE"),
    ("SERPENTE ANCESTRAL", "ANCIENT SERPENT"),
    ("SERPENTE COMETA", "COMET SERPENT"),
    (
        "SETAS escolhem  ·  ENTER decola  ·  ESC volta",
        "ARROWS choose  ·  ENTER launch  ·  ESC back",
    ),
    ("SETAS · ENTER · ESC", "ARROWS · ENTER · ESC"),
    ("SETOR", "SECTOR"),
    ("SETOR ???", "SECTOR ???"),
    ("SETOR LIMPO!", "SECTOR CLEAR!"),
    ("SIM", "YES"),
    ("SINAL CAPTADO", "SIGNAL RECEIVED"),
    ("SINGULARIDADE", "SINGULARITY"),
    ("SOBRECARGA", "OVERDRIVE"),
    ("SOBREVIVÊNCIA", "SURVIVAL"),
    ("SOMBRA", "SHADOW"),
    ("SONDA", "PROBE"),
    ("SUA VEZ!", "YOUR TURN!"),
    ("SUAVES", "SOFT"),
    ("SUBOFICIAL", "WARRANT OFFICER"),
    ("SUPER ÍMÃ", "SUPER MAGNET"),
    ("SUPER ÍMÃ!", "SUPER MAGNET!"),
    ("Sal Grosso", "Rock Salt"),
    ("Saudade", "Longing"),
    ("Saudosista", "Nostalgic"),
    (
        "Se eu cair, a colmeia se espalha e sobrevive. Se ele cair, ninguém mais vai segurar a porta.",
        "If I fall, the hive scatters and survives. If they fall, no one will hold the door anymore.",
    ),
    (
        "Segure SHIFT para ver exatamente onde a nave pode ser atingida.",
        "Hold SHIFT to see exactly where your ship can be hit.",
    ),
    (
        "Segure e solte ..... tiro carregado",
        "Hold and release . charged shot",
    ),
    ("Segure o tiro e solte", "Hold fire and release"),
    (
        "Segure o tiro e solte: atravessa tudo.",
        "Hold fire and release: it pierces everything.",
    ),
    (
        "Segure o tiro e solte: o tiro carregado atravessa a formação.",
        "Hold fire and release: the charged shot pierces the formation.",
    ),
    (
        "Segure o tiro para atirar sem parar.",
        "Hold fire to shoot nonstop.",
    ),
    ("Sem Arranhões", "Without a Scratch"),
    (
        "Serras giram em volta e bloqueiam tiros.",
        "Saws spin around you and block shots.",
    ),
    (
        "Setas / A D ........ mover para os lados",
        "Arrows / A D ....... move sideways",
    ),
    ("Setas e Espaço", "Arrows and Space"),
    ("Setas ou A/D", "Arrows or A/D"),
    ("Setas ou WASD", "Arrows or WASD"),
    (
        "Setores sem fim. A cada setor, escolha 1 de 3 bênçãos. Até onde você vai?",
        "Endless sectors. Every sector, pick 1 of 3 blessings. How far can you go?",
    ),
    (
        "Seu tempo ficou registrado.",
        "Your time has been recorded.",
    ),
    (
        "Seus tiros voam 15% mais rápido.",
        "Your shots fly 15% faster.",
    ),
    (
        "Shift .............. modo preciso",
        "Shift .............. precision mode",
    ),
    ("Sinal de Contato", "Contact Signal"),
    ("Sobrecarga Rápida", "Quick Overdrive"),
    ("Sobreviva 12 segundos.", "Survive for 12 seconds."),
    ("Sobrevivente", "Survivor"),
    ("Soletrador", "Speller"),
    (
        "Solta minas que sobem devagar e explodem em área.",
        "Drops mines that rise slowly and explode in an area.",
    ),
    (
        "Some e reaparece. Só é atingido quando está visível.",
        "Vanishes and reappears. Can only be hit while visible.",
    ),
    ("Sortudo", "Lucky"),
    (
        "Sua arma começa no nível 2.",
        "Your weapon starts at level 2.",
    ),
    (
        "Sua arma começa no nível 3.",
        "Your weapon starts at level 3.",
    ),
    (
        "Sua arma sobe 2 níveis agora.",
        "Your weapon goes up 2 levels now.",
    ),
    (
        "Suaves: clarões de explosão bem mais fracos.",
        "Soft: explosion flashes much weaker.",
    ),
    (
        "Suprema. Cada setor tem 3 ondas, um",
        "Queen. Each sector has 3 waves, a",
    ),
    ("SÓ LASER", "LASER ONLY"),
    (
        "Só existe Laser, mas ele começa no nível 3.",
        "Only the Laser exists, but it starts at level 3.",
    ),
    (
        "Só o centro da nave é vulnerável.",
        "Only the center of your ship is vulnerable.",
    ),
    (
        "Só pode ser ferido com a pálpebra aberta. Seu olhar varre o espaço com raios.",
        "Can only be hurt with its eyelid open. Its gaze sweeps space with beams.",
    ),
    ("TAMANHO DO ALVO", "TARGET SIZE"),
    ("TECELÃ", "WEAVER"),
    ("TECELÃ ABISSAL", "ABYSSAL WEAVER"),
    ("TEIA", "WEB"),
    ("TELA CHEIA", "FULLSCREEN"),
    ("TEMPESTADE IÔNICA", "ION STORM"),
    ("TENENTE", "LIEUTENANT"),
    ("TENENTE-CORONEL", "LIEUTENANT COLONEL"),
    ("TESTAR EFEITOS", "TEST SOUND FX"),
    ("TIRO AO ALVO", "TARGET PRACTICE"),
    ("TIRO AUTOMÁTICO", "AUTOFIRE"),
    (
        "TIRO AUTOMÁTICO se compra na loja.",
        "AUTOFIRE is sold in the shop.",
    ),
    ("TIRO AUTOMÁTICO!", "AUTOFIRE!"),
    ("TIRO CARREGADO", "CHARGED SHOT"),
    ("TIROS DOBRADOS", "DOUBLE SHOTS"),
    ("TRAVA ABERTA!", "LOCK OPENED!"),
    ("TREMOR DE TELA", "SCREEN SHAKE"),
    ("TREVO", "CLOVER"),
    ("TRILHA SONORA", "SOUNDTRACK"),
    ("TROCAR OFERTAS (GRÁTIS)", "REROLL OFFERS (FREE)"),
    ("TROVÃO", "THUNDER"),
    (
        "Tecelãs não machucam: elas te deixam lento. O que é pior.",
        "Weavers don't hurt: they slow you down. Which is worse.",
    ),
    (
        "Tecemos redes entre as luas para atrasar a coisa. As redes sumiram. Não rasgaram: sumiram.",
        "We wove nets between the moons to slow the thing down. The nets vanished. They didn't tear: they vanished.",
    ),
    ("Tela cheia", "Fullscreen"),
    ("Tempo Lento", "Slow Time"),
    ("Tempo de jogo", "Play time"),
    (
        "Tenha 300 cristais ao mesmo tempo.",
        "Hold 300 crystals at once.",
    ),
    (
        "Termine 3 desafios diários.",
        "Complete 3 daily challenges.",
    ),
    (
        "Termine a campanha no Difícil sem usar continues.",
        "Finish the campaign on Hard without using continues.",
    ),
    (
        "Termine a campanha no Pesadelo.",
        "Finish the campaign on Nightmare.",
    ),
    ("Termine a campanha.", "Finish the campaign."),
    ("Termine a primeira onda.", "Clear the first wave."),
    (
        "Termine um estágio bônus sem disparar.",
        "Finish a bonus stage without firing.",
    ),
    (
        "Termine um setor durante um Eclipse.",
        "Finish a sector during an Eclipse.",
    ),
    (
        "Termine um setor sem bombas",
        "Finish a sector without bombs",
    ),
    (
        "Termine um setor sem ser atingido.",
        "Finish a sector without getting hit.",
    ),
    ("Tesoureiro", "Treasurer"),
    ("Tio Patinhas", "Scrooge"),
    (
        "Tiram sua arma, níveis ou travam a nave.",
        "They take your weapon, levels, or jam your ship.",
    ),
    ("Tiro carregado", "Charged shot"),
    (
        "Tiro carregado 40% mais rápido e forte.",
        "Charged shot 40% faster and stronger.",
    ),
    ("Tiro de Canhão", "Cannon Shot"),
    (
        "Tiros em hélice que ondulam pela tela.",
        "Helix shots that undulate across the screen.",
    ),
    ("Tiros extras em diagonal.", "Extra diagonal shots."),
    ("Tiros inimigos 20% mais lentos.", "Enemy shots 20% slower."),
    (
        "Tiros que quicam ganham mais 2 quiques.",
        "Bouncing shots get 2 extra bounces.",
    ),
    (
        "Tiros que quicam nas paredes.",
        "Shots that bounce off the walls.",
    ),
    (
        "Toda partida dá XP. Subir de patente",
        "Every run gives XP. Ranking up",
    ),
    (
        "Todos os chefes em sequência, contra o relógio.",
        "Every boss in a row, against the clock.",
    ),
    (
        "Todos os chefes foram derrotados.",
        "Every boss has been defeated.",
    ),
    (
        "Tomamos a nave dos humanos. Precisávamos de casco e de canhões. Algo nos segue há três ciclos.",
        "We took the humans' ship. We needed hull and cannons. Something has been following us for three cycles.",
    ),
    ("Troque de nave no HANGAR.", "Change ships in the HANGAR."),
    (
        "Tudo na loja custa 20% menos.",
        "Everything in the shop costs 20% less.",
    ),
    (
        "Tudo na tela vira anos 2000 e leva dano.",
        "Everything on screen turns 2000s and takes damage.",
    ),
    ("Turista Cósmico", "Cosmic Tourist"),
    (
        "U  Super ímã: puxa todos os cristais",
        "U  Super magnet: pulls every crystal",
    ),
    ("UM BEZERRO TE SEGUE", "A CALF FOLLOWS YOU"),
    ("UM CHEFE SE APROXIMA", "A BOSS APPROACHES"),
    ("UM MERCADOR SE APROXIMA", "A MERCHANT APPROACHES"),
    ("UM PORTAL SE ABRIU", "A PORTAL OPENED"),
    ("UMA CARTA!", "A CARD!"),
    ("UMA RELÍQUIA SURGIU!", "A RELIC APPEARED!"),
    ("UMA TRAÇA!", "A MOTH!"),
    (
        "Um Escaravelho Dourado aparece em todo setor.",
        "A Golden Scarab appears every sector.",
    ),
    (
        "Um Escaravelho Dourado no próximo setor.",
        "A Golden Scarab in the next sector.",
    ),
    (
        "Um ala atira ao seu lado por 2 setores.",
        "A wingman fires at your side for 2 sectors.",
    ),
    (
        "Um bezerro te acompanha e recolhe cristais. Muuu.",
        "A calf follows you and gathers crystals. Moo.",
    ),
    (
        "Um comerciante neutro. Retribui a gentileza... e nunca esquece uma traição.",
        "A neutral trader. Returns kindness... and never forgets a betrayal.",
    ),
    (
        "Um cometa que o Enxame domou. Sua cauda solta poeira estelar que fere.",
        "A comet the Swarm tamed. Its tail sheds cosmic dust that hurts.",
    ),
    (
        "Um drone de escolta acompanha você.",
        "An escort drone follows you.",
    ),
    (
        "Um drone que atira com você.",
        "A drone that fires with you.",
    ),
    (
        "Um escudo novo no começo de cada setor.",
        "A fresh shield at the start of every sector.",
    ),
    (
        "Um impulso instantâneo e invulnerável. Gasta 1/3 da barra.",
        "An instant, invulnerable burst. Costs 1/3 of the bar.",
    ),
    ("Um novo começo, piloto.", "A fresh start, pilot."),
    (
        "Um raio gigante e contínuo por 3 s.",
        "A giant, continuous beam for 3 s.",
    ),
    (
        "Um raio que se divide em três ao viajar.",
        "A beam that splits into three as it travels.",
    ),
    (
        "Um tesouro escondido entre as dimensões...",
        "A treasure hidden between dimensions...",
    ),
    ("Uma Insígnia de Honra", "A Honor Insignia"),
    (
        "Uma carta aleatória para a coleção.",
        "A random card for your collection.",
    ),
    (
        "Uma cauda de anéis em brasa. Cada anel destruído enfraquece a fera.",
        "A tail of burning rings. Each ring destroyed weakens the beast.",
    ),
    (
        "Uma creche voadora. Solta filhotes enquanto estiver viva.",
        "A flying nursery. Releases hatchlings while alive.",
    ),
    ("Uma fortuna em cristais!", "A fortune in crystals!"),
    (
        "Uma lembrança de consolo: 5 cristais.",
        "A consolation prize: 5 crystals.",
    ),
    (
        "Uma linha mostra onde o tiro vai passar.",
        "A line shows where your shot will go.",
    ),
    (
        "Uma nave roubada e infestada. O canhão central dispara um laser avisado.",
        "A stolen, infested ship. The central cannon fires a telegraphed laser.",
    ),
    (
        "Uma onda de luz apaga tiros por perto e fere inimigos.",
        "A wave of light wipes nearby shots and hurts enemies.",
    ),
    (
        "Uma vez por partida, renasce ao perder a última vida.",
        "Once per run, revives when you lose your last life.",
    ),
    (
        "Uma vida só, mas pontos x3.",
        "Only one life, but points x3.",
    ),
    ("Use 50 bombas no total.", "Use 50 bombs in total."),
    ("Use um Portal de Salto.", "Use a Warp Gate."),
    ("VACA", "COW"),
    ("VACA CÓSMICA", "COSMIC COW"),
    ("VACA-CANHÃO", "COW CANNON"),
    ("VANGUARDA", "VANGUARD"),
    ("VAZIA...", "EMPTY..."),
    ("VAZIO", "VOID"),
    ("VELOCIDADE", "SPEED"),
    ("VESPA", "WASP"),
    ("VICE-ALMIRANTE", "VICE ADMIRAL"),
    ("VIDA EXTRA", "EXTRA LIFE"),
    ("VIDA EXTRA!", "EXTRA LIFE!"),
    ("VIDRO FINO", "THIN GLASS"),
    ("VITÓRIA!", "VICTORY!"),
    ("VOLTAR", "BACK"),
    ("Vaca Cósmica", "Cosmic Cow"),
    (
        "Vai, corta a formação inteira e volta para a mão.",
        "Throw it, it cuts through the whole formation and comes back to your hand.",
    ),
    (
        "Velocidade +10% pelo resto da expedição.",
        "Speed +10% for the rest of the expedition.",
    ),
    ("Velocista", "Speedrunner"),
    ("Vença 10 minigames das Fendas.", "Win 10 Rift minigames."),
    (
        "Vença a campanha 3 vezes para liberar.",
        "Win the campaign 3 times to unlock.",
    ),
    (
        "Vença aqui sem continues para ver o final verdadeiro.",
        "Win here without continues to see the true ending.",
    ),
    (
        "Vença o modo Chefões em menos de 12 minutos.",
        "Beat Boss Rush mode in under 12 minutes.",
    ),
    ("Viajante Dimensional", "Dimension Traveler"),
    ("Vitórias", "Victories"),
    (
        "Viva os 6 eventos de setor.",
        "Live through all 6 sector events.",
    ),
    (
        "Você e os inimigos atiram o dobro.",
        "You and the enemies fire twice as much.",
    ),
    ("Você perdeu a arma!", "You lost your weapon!"),
    ("Você renasceu das cinzas.", "You rose from the ashes."),
    (
        "Você só enxerga perto da nave.",
        "You can only see near your ship.",
    ),
    (
        "Volte amanhã para uma nova regra.",
        "Come back tomorrow for a new rule.",
    ),
    ("X / K .............. bomba", "X / K .............. bomb"),
    ("X ou K", "X or K"),
    (
        "Z  Câmera lenta para o mundo",
        "Z  Slow motion for the world",
    ),
    ("ZANGÃO", "DRONE"),
    (
        "Zangões, Vespas, Besouros, Atiradores,",
        "Drones, Wasps, Beetles, Gunners,",
    ),
    ("Zero Absoluto", "Absolute Zero"),
    (
        "abatendo, raspando e com o tempo.",
        "by shooting down, grazing, and over time.",
    ),
    ("armas, modos e pinturas.", "weapons, modes and paints."),
    (
        "cartas você ganha mais espaços.",
        "cards you unlock more slots.",
    ),
    ("comeu 20 cristais", "ate 20 crystals"),
    ("compondo a trilha sonora...", "composing the soundtrack..."),
    (
        "cristais, níveis, poderes e cartas.",
        "crystals, levels, power-ups and cards.",
    ),
    ("cristal em cada abate.", "crystal on every kill."),
    (
        "da verdade sobre o Enxame.",
        "of the truth about the Swarm.",
    ),
    ("de graça, que sorte", "for free, lucky you"),
    ("direto para a coleção", "straight into your collection"),
    ("drones e no TIRO AUTOMÁTICO.", "drones and AUTOFIRE."),
    (
        "e cristais. Veja na pausa ou no painel.",
        "and crystals. See them on pause or in the panel.",
    ),
    ("e repita com as SETAS.", "and repeat them with the ARROWS."),
    ("e trouxe de volta uma história", "and brought back a story"),
    (
        "encontros raros. Cada um conta um pedaço",
        "rare encounters. Each one tells a piece",
    ),
    (
        "estiver na faixa dourada. 3 travas.",
        "is on the golden band. 3 locks.",
    ),
    (
        "estágio bônus e um chefe. Depois, loja.",
        "bonus stage and a boss. Then, the shop.",
    ),
    ("fonte embutida", "embedded font"),
    ("jogo ativo", "game in progress"),
    (
        "libera naves, armas, modos e pinturas.",
        "unlocks ships, weapons, modes and paints.",
    ),
    (
        "melhores, Núcleos de Dano, cadência,",
        "weapons, Damage Cores, fire rate,",
    ),
    ("nenhuma carta equipada", "no card equipped"),
    ("nota por nota", "note by note"),
    (
        "o Enxame nunca foi o inimigo. Ele estava fugindo.",
        "the Swarm was never the enemy. It was fleeing.",
    ),
    (
        "que a gente jogava no PC da família",
        "that we used to play on the family PC",
    ),
    ("só tinha poeira estelar", "it was just stardust"),
    (
        "trazendo amostras de poeira estelar... e algo mais.",
        "carrying samples of stardust... and something more.",
    ),
    ("um golpe a menos", "one hit less"),
    ("um shoot 'em up de formações", "a formation shoot 'em up"),
    ("vazio", "empty"),
    ("ÁS", "ACE"),
    ("ÁUDIO", "AUDIO"),
    ("ÂNCORA", "ANCHOR"),
    ("ÂNCORA!", "ANCHOR!"),
    ("ÉGIDE", "AEGIS"),
    ("ÉPICA", "EPIC"),
    ("ÍMÃ", "MAGNET"),
    ("Ímã Portátil", "Portable Magnet"),
    ("Ímã de Cartas", "Card Magnet"),
    ("Ímã de Cápsulas", "Capsule Magnet"),
    ("ÓRBITA", "ORBIT"),
    ("ÓRBITA BAIXA", "LOW ORBIT"),
    ("Último Suspiro", "Last Breath"),
];

static TEMPLATES: &[(&[&str], &[&str])] = &[
    (
        &["vidas ", "  ·  tiros x", "  ·  XP x", "  ·  continues ", ""],
        &["lives ", "  ·  shots x", "  ·  XP x", "  ·  continues ", ""],
    ),
    (
        &[
            "PODER DE FOGO x",
            "   (arma ",
            " x",
            " · núcleos ",
            " x",
            ")",
        ],
        &["FIREPOWER x", "   (weapon ", " x", " · cores ", " x", ")"],
    ),
    (
        &["", "  ·  CIMA/BAIXO: nave, pintura, rastro"],
        &["", "  ·  UP/DOWN: ship, paint, trail"],
    ),
    (
        &["NAVES ", "  BOMBAS ", "  ", " NV", "  DRONES ", ""],
        &["SHIPS ", "  BOMBS ", "  ", " LV", "  DRONES ", ""],
    ),
    (
        &["Expedição · setor ", " vencido"],
        &["Expedition · sector ", " cleared"],
    ),
    (
        &["Destrua ", " partes de chefes"],
        &["Destroy ", " boss parts"],
    ),
    (
        &["< ", " / 6 >   ·   ESC volta"],
        &["< ", " / 6 >   ·   ESC back"],
    ),
    (
        &["RODADA ", " de 3 · ", " estrelas"],
        &["ROUND ", " of 3 · ", " stars"],
    ),
    (
        &["INSÍGNIAS DE HONRA  ", "/", ""],
        &["HONOR INSIGNIAS  ", "/", ""],
    ),
    (
        &["Vida extra e +", " pontos"],
        &["Extra life and +", " points"],
    ),
    (
        &["", "  ·  +", " XP  +", " cristais"],
        &["", "  ·  +", " XP  +", " crystals"],
    ),
    (&["PARTIR PARA O SETOR ", ""], &["DEPART FOR SECTOR ", ""]),
    (&["", " PRONTA PARA DECOLAR"], &["", " READY FOR LAUNCH"]),
    (
        &["Abata ", " em mergulho"],
        &["Shoot down ", " while diving"],
    ),
    (&["Alcance a patente ", ""], &["Reach the rank ", ""]),
    (&["CONTINUAR - CICLO ", ""], &["CONTINUE - CYCLE ", ""]),
    (&["ENFRENTAR O CHEFE ", ""], &["FACE THE BOSS ", ""]),
    (&["MAIS LONGE: SETOR ", ""], &["FARTHEST: SECTOR ", ""]),
    (&["BANCO: +", " CRISTAIS"], &["BANK: +", " CRYSTALS"]),
    (&["Congele ", " inimigos"], &["Freeze ", " enemies"]),
    (&["Faça um combo de ", ""], &["Make a ", " combo"]),
    (&["TROCAR OFERTAS (", ")"], &["REROLL OFFERS (", ")"]),
    (&["RASTRO: ", " (EM USO)"], &["TRAIL: ", " (IN USE)"]),
    (&["Colete ", " cristais"], &["Collect ", " crystals"]),
    (&["ONDA RELÂMPAGO +", ""], &["LIGHTNING WAVE +", ""]),
    (&["Pegue ", " cápsulas"], &["Grab ", " capsules"]),
    (&["Próxima: ", " (", " XP)"], &["Next: ", " (", " XP)"]),
    (&["TIRO CERTEIRO +", ""], &["SHARPSHOOTER +", ""]),
    (&["", " de ", " conquistas"], &["", " of ", " achievements"]),
    (&["MELHOR TEMPO ", ":", ""], &["BEST TIME ", ":", ""]),
    (&["REGRA DO DIA: ", ""], &["RULE OF THE DAY: ", ""]),
    (&["", " de ", " registros"], &["", " of ", " logs"]),
    (&["Abata ", " elites"], &["Shoot down ", " elites"]),
    (&["Faça ", " raspões"], &["Make ", " grazes"]),
    (&["MELHOR ONDA: ", ""], &["BEST WAVE: ", ""]),
    (&["", " (", " perfeitos)"], &["", " (", " perfect)"]),
    (&["", " foi perdida."], &["", " was lost."]),
    (&["ACERTOS  ", " / ", ""], &["HITS  ", " / ", ""]),
    (&["CONQUISTAS ", "/", ""], &["ACHIEVEMENTS ", "/", ""]),
    (&["FEBRE x2   ", "s"], &["FEVER x2   ", "s"]),
    (&["HABILIDADE: ", ""], &["ABILITY: ", ""]),
    (&["MAIOR COMBO ", ""], &["BEST COMBO ", ""]),
    (&["NOVA CARTA: ", ""], &["NEW CARD: ", ""]),
    (&["", " PONTOS  ·  ", ""], &["", " POINTS  ·  ", ""]),
    (&["CHEFE ", " DE 9"], &["BOSS ", " OF 9"]),
    (&["CONQUISTA: ", ""], &["ACHIEVEMENT: ", ""]),
    (&["Progresso: ", ""], &["Progress: ", ""]),
    (&["patente nº ", ""], &["rank #", ""]),
    (&["", " de ", " cartas"], &["", " of ", " cards"]),
    (&["ALVOS: ", " / ", ""], &["TARGETS: ", " / ", ""]),
    (&["ARQUIVOS ", "/", ""], &["ARCHIVES ", "/", ""]),
    (&["CHEFE EM ", "s"], &["BOSS IN ", "s"]),
    (&["CRISTAIS: ", ""], &["CRYSTALS: ", ""]),
    (&["CRÉDITOS: ", ""], &["CREDITS: ", ""]),
    (&["FEBRE x2  ", ""], &["FEVER x2  ", ""]),
    (&["MALDIÇÃO: ", ""], &["CURSE: ", ""]),
    (&["PRECISÃO ", "%"], &["ACCURACY ", "%"]),
    (&["PROMOÇÃO: ", ""], &["PROMOTION: ", ""]),
    (&["abatidos: ", ""], &["shot down: ", ""]),
    (&["COMBO ", "  x", ""], &["COMBO ", "  x", ""]),
    (&["ESPELHO ", "s"], &["MIRROR ", "s"]),
    (&["PINTURA: ", ""], &["PAINT: ", ""]),
    (&["Patente: ", ""], &["Rank: ", ""]),
    (&["CARTAS ", "/", ""], &["CARDS ", "/", ""]),
    (&["CHEFE ", "/8"], &["BOSS ", "/8"]),
    (&["EVENTO: ", ""], &["EVENT: ", ""]),
    (&["GRUPO! +", ""], &["GROUP! +", ""]),
    (&["PRÊMIO: ", ""], &["REWARD: ", ""]),
    (&["Patente ", ""], &["Rank ", ""]),
    (&["PÁGINA ", "/", ""], &["PAGE ", "/", ""]),
    (&["RASPÕES ", ""], &["GRAZES ", ""]),
    (&["RECORDE ", ""], &["HIGH SCORE ", ""]),
    (&["cópias: ", ""], &["copies: ", ""]),
    (&["patente ", ""], &["rank ", ""]),
    (&["", "  (+", " XP)"], &["", "  (+", " XP)"]),
    (&["", " PERDIDA"], &["", " LOST"]),
    (&["ÂNCORA ", "s"], &["ANCHOR ", "s"]),
    (&["RASTRO: ", ""], &["TRAIL: ", ""]),
    (&["Abata ", " ", ""], &["Shoot down ", " ", ""]),
    (&["BÔNUS  ", ""], &["BONUS  ", ""]),
    (&["CARTA: ", ""], &["CARD: ", ""]),
    (&["COMBO x", ""], &["COMBO x", ""]),
    (&["DRONES ", ""], &["DRONES ", ""]),
    (&["FEITO: ", ""], &["FEAT: ", ""]),
    (&["PONTOS ", ""], &["SCORE ", ""]),
    (&["REGRA: ", ""], &["RULE: ", ""]),
    (&["SETOR ", "-", ""], &["SECTOR ", "-", ""]),
    (&["TEMPO ", ":", ""], &["TIME ", ":", ""]),
    (&["", " NÍVEL ", ""], &["", " LEVEL ", ""]),
    (&["CICLO ", ""], &["CYCLE ", ""]),
    (&["HOJE: ", ""], &["TODAY: ", ""]),
    (&["NAVE: ", ""], &["SHIP: ", ""]),
    (&["NOVO: ", ""], &["NEW: ", ""]),
    (&["PANE ", "s"], &["JAM ", "s"]),
    (&["SALTO ", ""], &["WARP ", ""]),
    (&["SETOR ", ""], &["SECTOR ", ""]),
    (&["setor ", ""], &["sector ", ""]),
    (&["ONDA ", ""], &["WAVE ", ""]),
    (&["", "  (", "/", ")"], &["", "  (", "/", ")"]),
    (&["REC ", ""], &["HI ", ""]),
    (&["> ", " <"], &["> ", " <"]),
];
