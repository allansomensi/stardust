=====================================================================
                          S T A R D U S T
                  a formation shoot 'em up  ·  v0.1.0
=====================================================================

Year 2097. The STARDUST probe came back from a distant comet carrying
stardust... and a signal. The signal called the Swarm: millions of
star insects in attack formation. The fleet fell in hours. What's
left: a hidden hangar, one ship and you.

Clear 100 levels, take down whole formations, collect crystals, buy
stronger and stronger weapons and defeat the bosses of the Swarm.
Alone or with a friend, on the keyboard or with gamepads.

Language: English or Portuguese, in SETTINGS > GAME.


CONTROLS
--------
  Arrows or A/D ........ move (your ship only moves sideways)
  Space / Z ............ fire: every tap is a burst
  Hold and release ..... charged shot (pierces the formation)
  X / G ................ bomb
  C / H ................ ship ability
  Shift ................ precision mode (slower, shows your ship's
                         weak spot)
  Esc / P .............. pause
  F11 or Alt+Enter ..... fullscreen

  Holding the fire button only works with AUTOFIRE, which you buy
  in the shop. With it, the charged shot no longer works.

  Every game key and button can be remapped, for each player,
  in SETTINGS > CONTROLS. The menus also work with the mouse
  (right click goes back).

  GAMEPAD: stick or D-pad move, A fires, X bombs, Y uses the
  ability, LB is precision mode and Start pauses. In menus, A
  confirms and B goes back.


TWO PLAYERS (CO-OP)
-------------------
On the game mode screen, set PLAYERS to 2.
  No gamepad:   P1 = A D, Space/Z fire, X/G bomb, C/H ability,
                     left Shift precision
                P2 = arrows, comma (or Right Ctrl) fire, period
                     bomb, slash ability, right Shift precision
  One gamepad:  P1 on the keyboard, P2 on the gamepad.
  Two gamepads: one each.
Player 2 picks any ship player 1 has unlocked. Both ships share
score, crystals and the shop; each has its own lives and bombs.
Fly close together to fill the DUO bar, then press the ability
button for the DUO BEAM. A ship that runs out of lives leaves a
rescue capsule: fly into it to bring your partner back. Each ship
can be rescued only once per run. The run ends when both are out.


THE CAMPAIGN
------------
100 levels in 20 sectors. Each sector has 3 waves, a bonus stage
and a boss, then the shop. No two levels are alike: each has its
own formation, movement, entry paths and tricks. It gets harder at every level, and the
last quarter is brutal: elite enemies show up, and bosses change
the stage as they weaken (darkness, rising ground, meteors). Reach level 26, 51 or 76 and you can start
the next campaign from there.

HOW TO GET STRONGER
-------------------
Between sectors there is the SHOP. The crystals you collect buy:
* WEAPONS: the more stars (* to *****), the stronger and pricier.
* EVOLUTION: a weapon at level 5 can be evolved into a stronger
  form (a mini sun, twin beams, a seven-way fan...).
* DAMAGE CORE: +15% damage on any weapon, up to 6 times.
* Weapon upgrades, fire rate, speed, drones, side cannons, shield,
  bombs, lives, autofire, laser sight and more.
Picking up the same weapon again raises its level (up to 5). The
shop panel shows your total FIREPOWER.


WHAT'S IN THE GAME
------------------
* 6 MODES: Campaign, Classic, Daily Challenge, Survival, Boss Rush
  and Expedition (endless sectors with blessings).
* 4 DIFFICULTIES: the harder it gets, the faster the Swarm moves,
  dives and shoots.
* Ships with their own abilities, paints and engine trails.
* Dozens of weapons, each with its own icon.
* Bosses with destructible parts and rage phases.
* Collectible cards, ranks, Honor Insignias, achievements,
  bestiary, stats and flight logs.
* Sector events, contracts, Fever mode, minigames and much more.


TIPS
----
* Only the center of your ship is vulnerable.
* The capsule icon shows the weapon; the ring around it shows when
  it's about to change.
* Crystals are blue diamonds. Enemy shots are always warm colors
  (pink, orange, red or purple) with a dark outline.
* Skulls follow you slowly. Shoot them before they reach you.
* Explore. Not everything is written here.


YOUR PROGRESS
-------------
Rank, cards, achievements, high scores and settings are saved in:
  Windows:  %APPDATA%\Stardust\save.txt
  Linux:    ~/.local/share/stardust/save.txt
  macOS:    ~/Library/Application Support/Stardust/save.txt

To reset everything: SETTINGS > GAME > ERASE PROGRESS.


CUSTOM MUSIC
------------
The game composes its own chiptune soundtrack. To use your own
music, create a "music" folder next to stardust.exe (or inside the
save folder) with .ogg or .wav files named:
  title, stage, boss, shop, final, credits,
  region1 ... region8 (one per region), special

To export the original soundtrack as .wav:
  stardust.exe --export-music


WINDOWS WARNING
---------------
If you see "Windows protected your PC", click "More info" and then
"Run anyway". This happens with new programs that don't have a
digital signature yet.


CREDITS
-------
Created by Allan Somensi.
Made in Rust with the macroquad library.
All art and music are generated by the game itself.
Fonts: Pixelify Sans and Press Start 2P (SIL Open Font License).

Thanks for playing!
