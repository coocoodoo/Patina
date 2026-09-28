# Hollowbloom

**A cozy top-down action RPG. Farm on the surface, delve an endless dungeon below it, and
help a little town come back to life.**

Your little farm sits right on top of *the Hollow*, a cave that goes down forever. Fight and
dig your way through procedurally generated floors, bring home the strange seeds, ores and
treasures you find, and grow them into a farm that helps you go deeper. Every tenth floor a
guardian watches over a waystone that can take you home. Down the road, a bus runs to
**Bramblewick**, where nineteen villagers have shops to keep, stories to tell and more than a
hundred things they would love a hand with.

Everything you see is low-poly 3D with pixel-art textures, drawn by a small software renderer
that **can only ever produce the 32 colours of the
[Resurrect 32](https://lospec.com/palette-list/resurrect-32) palette** by Kerrie Lake:
textures, lighting, shadows, dithering and UI are all palette indices, and the only RGB
in the program is the palette itself, applied at the final blit.

| | |
| --- | --- |
| ![Title](docs/title.png) | ![Welcome](docs/welcome.png) |
| ![Farm in the morning](docs/farm-morning.png) | ![Farm at night](docs/farm-night.png) |
| ![Every crop in the game](docs/garden.png) | ![The bag, worn gear and a legendary sword](docs/bag.png) |
| ![The enchanting table](docs/enchanting.png) | ![Burrowby's daily specials](docs/shop.png) |
| ![Mossy Burrows](docs/hollow-mossy.png) | ![Crystal Grotto](docs/hollow-crystal.png) |
| ![Fungal Hollow](docs/hollow-fungal.png) | ![Ember Depths](docs/hollow-ember.png) |
| ![Frost Caverns](docs/hollow-frost.png) | ![Sunken Ruins](docs/hollow-ruins.png) |
| ![A guardian](docs/guardian.png) | ![Guardian loot](docs/loot.png) |
| ![The waystone](docs/waystone.png) | ![Your stats](docs/stats.png) |
| ![Crafting](docs/crafting.png) | ![The bus to town](docs/bus.png) |

**Bramblewick**

![Bramblewick from above](docs/town.png)

| | |
| --- | --- |
| ![Toby the postman in the plaza](docs/bramblewick.png) | ![Bramblewick at night, lamps lit](docs/town-night.png) |
| ![Talking with Pip](docs/talk.png) | ![Pip asks for help](docs/quest-offer.png) |
| ![Quest complete!](docs/quest-complete.png) | ![The quest journal](docs/journal.png) |
| ![The request board](docs/board.png) | ![A keepsake waiting in the Hollow](docs/keepsake.png) |
| ![Honeycrumb Bakery](docs/bakery.png) | ![Petalplate Armory's daily specials](docs/armory.png) |
| ![The Sleepy Snail at night](docs/tavern.png) | ![Moonquill Scriptorium](docs/scriptorium.png) |
| ![The Wishing Tree in bloom](docs/wishing-tree.png) | |

![A dozen outfits from the Hollow's wardrobe](docs/wardrobe.png)

## Playing

Grab a build from the releases (`hollowbloom-*-windows-x86_64.zip` or
`hollowbloom-*-linux-x86_64.tar.gz`) or [build it yourself](#building). There is nothing to
install: it is a single executable.

| | Keyboard | Mouse |
| --- | --- | --- |
| Move | WASD / arrows | |
| Use tool, attack, cast | J or Z | left click (aims at the cursor) |
| Interact, talk, harvest, eat, place, wear | E or K | right click |
| Dodge roll | Space or Shift | |
| Hotbar | 1-9, 0, Q / R | wheel |
| Bag, worn gear, stats / crafting | Tab or I / C | right click wears armour |
| Quest journal, friends, records | L | |
| Minimap (in the Hollow and in town) | M | |
| Pause, settings, save & quit | Esc | |
| Fullscreen / screenshot | F11 or Alt+Enter / F12 | |

### The farm (Stardew-style)

* **Till** grass with the hoe, **plant** seeds, **water** every day with the can (refill it at
  the pond), and **sleep** in your house to end the day. Crops grow one stage per watered day;
  many (strawberries, tomatoes, corn, blueberries, Crystal Berries, Ember Peppers...) keep
  fruiting after the first harvest. **Sweep ripe crops with a sickle** to harvest a whole patch.
* **38 crops**: turnips, potatoes, radishes, cabbages, tomatoes, strawberries, wheat, corn,
  blueberries, eggplants, garlic, sunflowers, roses and sweet peas from Burrowby's stall, and
  strange seeds from every biome of the Hollow: Mossberries and Bunnyroots, Prism Pears and
  Geode Gourds, Puffballs, Jelly Shrooms and Truffles, Flame Tulips, Lava Lemons and Magma
  Melons, Snow Peas, Ice Plums and Frost Mint, Starfruit, Ghost Peppers and Ancient Grain.
* **Cook** them into two dozen dishes. The good ones give a **buff** for a few minutes:
  Garlic Bread for damage, Strawberry Shortcake for luck, Frost Mint Tea for mana, Lava
  Lemonade to set things on fire...
* **Ship** anything in the bin by the house and the money arrives overnight, or trade with
  Burrowby the mole at his stall. His stock grows the deeper you have been.
* Tools cost **energy**; food restores health and energy. Stay up past 2am and you will
  collapse where you stand. Rainy days water everything for you.
* Craft **sprinklers** from dungeon ores to automate watering (4, 8 or 24 tiles).
* Glowcaps and Moonblooms glow at night; lamps, lit windows and fireflies keep the evenings
  cozy. There is a cat. You can pet the cat.

### Bramblewick (the town down the road)

* A country road now runs along the farm's east edge. **Wave the bus down** at the shelter and
  it drives you to **Bramblewick**: streets and a plaza, nineteen storybook buildings, a
  stream with a park beyond it, and a bus back home whenever you like. Road corners are
  rounded, on the farm and in town.
* **Eleven buildings you can walk into**, each a little diorama: the Town Hall, the Lantern
  Guild, and **nine specialist shops**, each with its own shelves, **specials rolled fresh
  every morning**, and better prices for the things it deals in:
  * **Petalplate Armory** (Hilde): helmets, chest armour, leg armour, boots and shields.
  * **Sprig & Steel** (Garrick): swords, wands and staffs.
  * **Tinkerbolt Tools** (Nix): hoes, cans, sickles, axes, pickaxes and sprinklers.
  * **Moonquill Scriptorium** (Elder Quill): weapon, armour and tool scrolls, tonics, wish
    stars, and an enchanting table.
  * **Sproutling Seeds** (Posy): every seed in the valley, and rarer ones the deeper you go.
  * **Honeycrumb Bakery** (Mabel), **The Sleepy Snail** tavern (Barley): food with buffs.
  * **Glimmer & Gold** (Opal): gems and curios, and 50% more for any you sell her.
  * **Cozy Nook** (Wren): lamps, benches, chests, paths and other furniture.
  * **The Lantern Guild** (Captain Rowan): delving supplies and guild spoils.
* **Nineteen villagers** with their own looks, outfits, voices and **daily schedules**: they
  open their shops, stroll their favourite corners of town, gather at the tavern in the
  evening and go home at night. Talk to them (they chatter in little blips, with their
  portrait in the talk box), **give gifts** (each has things they love, like and hate) and
  your **friendship** grows to ten hearts, with presents at three, six and nine.
* **111 hand-written story quests**, five to eight from everyone in town, unlocking as you
  make friends and go deeper: find **keepsakes** lost in the Hollow (Albert's locket on floors
  2-5, Toby's mailbag, Quill's spectacles...), gather **quest drops** from creatures (slime
  hearts, bat fangs, glow oil from wisps, wishing leaves from the deep), slay, reach floors,
  beat guardians, grow and harvest, cook, enchant, ship, and **deliver** letters and parcels.
  Keepsakes and quest drops **only exist while the quest is open**, and a guardian **returns
  to its floor** for anyone who needs something it carries. A **!** marks someone with a
  request and a **?** someone you can hand one in to.
* **Rewards are generous**: heaps of coins, gear of a guaranteed rarity at your level
  (up to Legendary), top scrolls, heart crystals, sun stones and wish stars, sprinklers, rare
  seeds, and each finished quest ends in a little celebration.
* **Bring the town back**: the mayor's projects make the fountain flow, relight the lamps,
  replant the flower beds, string festival bunting, mend the bridges to the park, fix the
  clock (it chimes the hours), bring back market day and make the **Wishing Tree** bloom
  (it blesses you with luck once a day).
* Endless extras: **three new notices every day** on the plaza's **request board**, **guild
  bounties** that earn marks and promotions through six **Lantern Guild ranks**, and
  **collections** to complete (every crop, every curio, every dish, every creature). The
  journal (L) keeps track of it all.

### The Hollow (endless and procedural)

* Every floor is generated from your world seed and its depth: rooms, winding corridors,
  ore veins, pots, crates, treasure chests and torches. The stairs down are always as far from
  where you land as the floor allows.
* The biome changes every ten floors (Mossy Burrows, Crystal Grotto, Fungal Hollow, Ember
  Depths, Frost Caverns, Sunken Ruins) and then cycles, tougher each time. Each biome has its
  own creatures (slimes, bats, shroomlings, crystal crabs, wisps, beetles, imps, skeletons,
  golems, ghosts), its own ores and its own **seeds**.
* **Every tenth floor** a guardian (King Slime, Crystal Matriarch, Old Capwood, Ember Lord,
  Frost Colossus, the Bone Warden) blocks the stairs and a **waystone**. Defeat it, touch the
  waystone, and you can go home. The Hollow's entrance on your farm then lets you start from
  any waystone you have attuned.
* Fainting in the Hollow sends you home the next morning, a tenth of your money lighter. A
  Homeward Feather gets you out early.

### Loot, gear and enchanting

* **123 weapons, pieces of armour and tools**, every one of them cute: swords (from a Twig
  Sword, a Carrot Blade and the Hero's Baguette to the Starlight Sword), **wands** that shoot
  magic bolts and **staffs** that set off blasts (both use mana), **shields** (a Pot Lid, a
  Turtle Shell, a Mushroom Shield...), **headgear** (Straw Hat, Flower Crown, Cat-Ear, Frog and
  Bunny Hoods, a Wizard Hat, a Miner's Hat with a lamp...), **chest armour**, **leg armour**,
  **boots** (Rain Boots, Frog and Bunny Slippers, Feather Boots...) and farming **tools**
  (hoes, watering cans shaped like ducks, teapots and frogs, sickles, axes, pickaxes).
  Everything you wear shows up on your little farmer.
* Every piece has an **item level** from where it was found, a rolled base value, an innate
  bonus and up to four **random stats** out of 28 (damage, crit, speed, heartsip, burn, chill
  and shock, defense, max HP, energy and mana, regeneration, move speed, dodge, block, thorns,
  luck, coin find, tool power, reach, water, bounty, growth, forage...). Its **rarity**
  (Common, Uncommon, Rare, Epic, Legendary) comes from how good those rolls are; rare finds
  glow on the ground with a beam of light.
* **Enchanting scrolls** drop in the Hollow (or are scribed from gems). Weapon, armour and
  tool scrolls each carry one enchantment and only bind to their own kind of gear; a scroll's
  rarity comes from how strong its enchantment rolled. Bind them at the **enchanting table**
  by your house (or the one at every waystone) for a few coins; each piece has three sockets,
  and enchanting can raise its rarity.
* **Coins**: creatures, pots and chests drop **copper, silver and gold coins** (10 copper make
  a silver, 10 silver a gold). Guardians shower you in gold, gear, scrolls and gems; relics
  like rubber ducks, music boxes and tiny crowns sell well to Burrowby, who also has new
  **specials** every morning.

### Freedom (Core Keeper-style)

* The pickaxe **mines the Hollow's walls**: dig your own tunnels and shortcuts, and break
  ore veins for copper, iron, gold, glimmer shards, ember ore and frost opals.
* **Build anywhere**: stone and wood walls, paths and floors, fences, torches, lamps, chests,
  benches, flower pots, workbenches, sprinklers. Hit them with a pickaxe or an axe to pick them
  back up.
* Better gear is crafted from what you bring up: ores, crab shells, imp horns, wisp dust and
  more. Crafted gear comes out with random stats too.

Progress is saved automatically every morning (and when you quit anywhere but the Hollow):

* Windows: `%APPDATA%\Hollowbloom\save.json`
* Linux: `$XDG_DATA_HOME/hollowbloom/save.json` (usually `~/.local/share/hollowbloom/`)
* macOS: `~/Library/Application Support/Hollowbloom/save.json`

Set `HOLLOWBLOOM_DATA` to use another folder. Screenshots (F12) go into `screenshots/` there.

## Building

You need [Rust](https://rustup.rs) 1.85 or newer. From the repository root:

```bash
cargo run -p hollowbloom --release
```

The Linux build links only against glibc. X11 or Wayland (with `libxkbcommon`, present on
every desktop) and ALSA are loaded at run time, and without a sound device the game simply
runs quietly. Nothing beyond the Rust toolchain is needed to build on Windows or Linux.

### Cross-compiling

Both platforms can be built from one machine:

```bash
# Linux -> Windows with MinGW-w64 (the linker is set in .cargo/config.toml)
sudo apt install gcc-mingw-w64-x86-64        # Fedora: mingw64-gcc
rustup target add x86_64-pc-windows-gnu
cargo build -p hollowbloom --release --target x86_64-pc-windows-gnu

# Any OS -> Linux and Windows with cargo-zigbuild (pip install ziglang; cargo install cargo-zigbuild)
cargo zigbuild -p hollowbloom --release --target x86_64-unknown-linux-gnu.2.28   # runs on glibc >= 2.28
cargo zigbuild -p hollowbloom --release --target x86_64-pc-windows-gnu

# Or package both into dist/ (zip for Windows, tar.gz for Linux)
hollowbloom/package.sh
```

On Windows itself a plain `cargo build -p hollowbloom --release` produces a native MSVC build.
CI (`.github/workflows/hollowbloom.yml`) runs the tests, renders a screenshot tour, builds
both packages on Linux, builds and runs the game on Windows, and attaches the packages to a
release when a `hollowbloom-v*` tag is pushed.

### Tools

```bash
hollowbloom --shots DIR           # render a tour of the game to PNGs, no display needed
hollowbloom --town-shots DIR      # just the road to town, Bramblewick and every room
hollowbloom --folk-shots DIR      # just villagers, talking, quests, boards and shops
hollowbloom --wardrobe FILE       # the hero in a dozen outfits (and FILE_all: every piece)
hollowbloom --bench               # rendering speed (about 4 ms per frame at 480x270)
hollowbloom --palette-chart FILE  # the light maps: every colour at every light level
hollowbloom --mute                # no sound
```

## How it works

* **Palette-locked software renderer** (`src/render`). The scene is rendered at a low
  resolution (about 427x240, chosen per window size so pixels scale by a whole number) into
  a buffer of palette indices. Triangles are clipped, perspective-correctly textured with
  16x16 pixel-art textures, depth-tested, and lit through precomputed *light maps* that send
  `(light level, warmth, colour)` to another palette entry, computed once in OKLab so that
  shadows walk down a colour's own ramp. Ordered (Bayer) dithering blends neighbouring light
  levels. Blob shadows, pixel outlines around characters and see-through silhouettes behind
  walls are all index operations too. Tests check that every rendered pixel is one of the 32.
* **Lighting**: an ambient level and warmth (dawn, noon, sunset, moonlight, each biome's
  gloom) plus point lights (torches, lamps, campfires, glowing crystals and crops, the hero's
  lantern, enemy magic) accumulated on a grid of tile corners with line-of-sight against
  walls.
* **Everything is procedural or written in code**: tile textures, the low-poly models
  (boxes and lathes, including a 3D look for every hat, boot, shield, weapon and tool), about
  390 item icons, crop sprites and stat symbols drawn as ASCII art, two bitmap fonts,
  the sound effects (sfxr-style synthesis) and the chiptune soundtrack (a small four-channel
  sequencer with eight songs). The town's buildings are generated from a short description
  each (size, walls, roof, awning, sign), and villagers find their way about with a simple
  breadth-first path search over the town's tiles.
* **Audio** uses ALSA on Linux (loaded with `dlopen`) and winmm on Windows, from a mixer
  thread.

## Credits

* Palette: [Resurrect 32](https://lospec.com/palette-list/resurrect-32) by Kerrie Lake.
* Inspired by the look of *Final Fantasy: The 4 Heroes of Light*, the farming and townsfolk
  of *Stardew Valley* and the digging freedom of *Core Keeper*.
