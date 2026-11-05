//! Weapon evolution. Once a weapon reaches level 5, the shop offers to evolve it. An evolved
//! weapon hits 30% harder and fires 10% faster, and several weapons change shape:
//!
//! - BLASTER: a seven-way fan.          - LASER: beams pierce many more enemies.
//! - MISSILE: two extra missiles.       - FIREBALL: a mini sun with a huge blast.
//! - PLASMA: two extra, bigger orbs.    - RAILGUN: twin beams.
//! - METEOR: a comet that pierces deep and bursts into twice the shards.
//!
//! The evolution belongs to the weapon: picking up a different weapon leaves it behind.

use super::*;

/// What evolving `w` does, shown in the shop.
pub fn evolution_desc(w: Weapon) -> &'static str {
    match w {
        Weapon::Blaster => "A seven-way fan, 30% stronger.",
        Weapon::Laser => "Beams pierce many more enemies, 30% stronger.",
        Weapon::Missile => "Two extra homing missiles per volley.",
        Weapon::Fireball => "A mini sun: a huge blast and a ring of embers.",
        Weapon::Plasma => "Two extra, bigger plasma orbs.",
        Weapon::Railgun => "Twin beams that shred the whole column.",
        Weapon::Meteor => "A comet that pierces deep and bursts into twice the shards.",
        _ => "30% stronger and 10% faster.",
    }
}

impl Game {
    /// True when the weapon in hand has been evolved.
    pub fn evolved(&self) -> bool {
        self.player.evolved_weapon == Some(self.player.weapon)
    }

    /// The shop can offer to evolve the weapon in hand.
    pub(super) fn can_evolve(&self) -> bool {
        self.player.level >= 5 && !self.evolved() && !self.player.weapon.is_relic()
    }

    pub(super) fn evolve(&mut self, au: &mut Audio) {
        self.player.evolved_weapon = Some(self.player.weapon);
        au.play(Sfx::Fanfare);
        self.fx.ring(self.player.pos, 300.0, 0.6, GOLD, 4.0);
        self.banner = Some((
            format!("{} EVOLVED!", self.player.weapon.name()),
            evolution_desc(self.player.weapon).into(),
            3.0,
        ));
    }
}
