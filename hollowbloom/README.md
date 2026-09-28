# Hollowbloom

**A cozy top-down action RPG. Farm on the surface, delve an endless dungeon below it, and
help a little town come back to life.**

Your little farm sits right on top of *the Hollow*, a cave that goes down forever. Fight and
dig your way through procedurally generated floors, bring home the strange seeds, ores and
treasures you find, and grow them into a farm that helps you go deeper. Every tenth floor a
guardian watches over a waystone that can take you home. Down the road, a bus runs to
**Bramblewick**, where twenty villagers have shops to keep, stories to tell and more than a
hundred things they would love a hand with. Learn **spells** from the town's spellwright,
**brew potions** in three sizes, and watch the grass ripple in the breeze. Cast a line for
**41 kinds of fish**, cook them on the stove in your **farmhouse**, and fill it with furniture:
a charming home gets you better prices, warmer friends and some very special quests.

Everything you see is low-poly 3D with pixel-art textures, drawn by a small software renderer
that **can only ever produce the 32 colours of the
[Resurrect 32](https://lospec.com/palette-list/resurrect-32) palette** by Kerrie Lake:
textures, lighting, shadows, dithering and UI are all palette indices, and the only RGB
in the program is the palette itself, applied at the final blit. Even the see-through jelly of
the slimes, the soap-bubble of a Ward spell and the glow of a spark are palette lookups.

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
| ![Jelly slimes of every biome, and a ghost](docs/jelly-slimes.png) | ![Grass in the breeze, weeds and wild bushes](docs/breezy-farm.png) |
| ![Sparks fly off stone](docs/sparks.png) | ![Brewing potions](docs/potions.png) |

**Spells and potions**

| | |
| --- | --- |
| ![The Starfall Spellery](docs/spellery.png) | ![Hazel, the spellwright](docs/hazel.png) |
| ![Learning spells](docs/spells.png) | ![The attuning circle: two spells at a time](docs/attuning.png) |
| ![Firebolt](docs/firebolt.png) | ![Chain Spark](docs/chain-spark.png) |
| ![Starfall](docs/starfall.png) | ![Frost Nova](docs/frost-nova.png) |
| ![Ward](docs/ward.png) | ![Bloom waters and quickens the crops](docs/bloom.png) |

**Fishing, your farmhouse and the furniture shop**

| | |
| --- | --- |
| ![A line in the farm pond](docs/fishing.png) | ![Reeling in: keep the fish in the green bar](docs/reeling.png) |
| ![A perfect catch](docs/catch.png) | ![Frogs, jellies and puffers round a pond in the Hollow](docs/water-folk.png) |
| ![Your farmhouse on the first morning](docs/home.png) | ![A charming home, months later](docs/home-charming.png) |
| ![Cozy at night](docs/home-night.png) | ![Placing an armchair](docs/placing.png) |
| ![The fish tank](docs/fish-tank.png) | ![Cooking at the copper range](docs/cooking.png) |
| ![The Fishdex](docs/fishdex.png) | ![Wren's Cozy Nook](docs/cozy-nook.png) |
| ![Furniture for sale](docs/nook-shop.png) | ![Plain rods at the smithy](docs/rods.png) |

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
| Use tool, attack, shoot | J or Z | left click (aims at the cursor) |
| Fish: wind up and cast, hook, reel | hold J and let go; J on a bite; hold J to reel | left click |
| Cast your two spells | Q / R | (aims at the cursor) |
| Interact, talk, harvest, eat, place, wear | E or K | right click |
| At home: turn a piece before placing it / pick it back up | T / J | left click picks up |
| Dodge roll | Space or Shift | |
| Hotbar | 1-9, 0, [ / ] | wheel |
| Bag, worn gear, stats / crafting | Tab or I / C | right click wears armour |
| Quest journal, friends, records | L | |
| Minimap (in the Hollow and in town) | M | |
| Pause, settings, save & quit | Esc | |
| Fullscreen / screenshot | F11 or Alt+Enter / F12 | |

### The farm (Stardew-style)

* **Till** any open ground on your property with the hoe (lawn, bare soil, the paths, the
  sand by the pond), **plant** seeds, **water** every day with the can (refill it at the
  pond), and **sleep** in your house to end the day. Crops grow one stage per watered day;
  many (strawberries, tomatoes, corn, blueberries, Crystal Berries, Ember Peppers...) keep
  fruiting after the first harvest. **Sweep ripe crops with a sickle** to harvest a whole patch.
* **38 crops**: turnips, potatoes, radishes, cabbages, tomatoes, strawberries, wheat, corn,
  blueberries, eggplants, garlic, sunflowers, roses and sweet peas from Burrowby's stall, and
  strange seeds from every biome of the Hollow: Mossberries and Bunnyroots, Prism Pears and
  Geode Gourds, Puffballs, Jelly Shrooms and Truffles, Flame Tulips, Lava Lemons and Magma
  Melons, Snow Peas, Ice Plums and Frost Mint, Starfruit, Ghost Peppers and Ancient Grain.
* **Cook** them into 41 dishes on the **stove** in your house (or the campfire at any
  waystone). The good ones give a **buff** for a few minutes: Garlic Bread for damage,
  Strawberry Shortcake for luck, Frost Mint Tea for mana, Lava Lemonade to set things on
  fire...
* **Ship** anything in the bin by the house and the money arrives overnight, or trade with
  Burrowby the mole at his stall. His stock grows the deeper you have been.
* Tools cost **energy**; food restores health and energy. Stay up past 2am and you will
  collapse where you stand. Rainy days water everything for you.
* Craft **sprinklers** from dungeon ores to automate watering (4, 8 or 24 tiles).
* **The wild creeps back**: every night fresh weeds, wildflowers and **wild bushes** spring
  up on empty ground. Pull weeds for fiber and the odd seed; cut bushes back with an axe,
  sickle or sword for fiber, sticks, **heartleaf** (a potion herb) and, from berry bushes,
  blueberries.
* **3D grass** covers the lawns in tufts and tall meadow patches. Grass, weeds, flowers,
  crops, bushes and trees all **sway in the wind**: gusts roll across the land in waves,
  it blows harder in the rain, and the grass parts round your feet as you walk through it.
* Glowcaps and Moonblooms glow at night; lamps, lit windows and fireflies keep the evenings
  cozy. There is a cat. You can pet the cat.

### Bramblewick (the town down the road)

* A country road now runs along the farm's east edge. **Wave the bus down** at the shelter and
  it drives you to **Bramblewick**: streets and a plaza, nineteen storybook buildings, a
  stream with a park beyond it, and a bus back home whenever you like. Road corners are
  rounded, on the farm and in town.
* **Twelve buildings you can walk into**, each a little diorama: the Town Hall, the Lantern
  Guild, and **ten specialist shops**, each with its own shelves, **specials rolled fresh
  every morning**, and better prices for the things it deals in:
  * **Petalplate Armory** (Hilde): helmets, chest armour, leg armour, boots and shields.
  * **Sprig & Steel** (Garrick): swords, wands, staffs, and plain fishing rods and bait.
  * **Tinkerbolt Tools** (Nix): hoes, cans, sickles, axes, pickaxes and sprinklers.
  * **Moonquill Scriptorium** (Elder Quill): weapon, armour and tool scrolls, tonics, wish
    stars, and an enchanting table.
  * **Sproutling Seeds** (Posy): every seed in the valley, and rarer ones the deeper you go.
  * **Honeycrumb Bakery** (Mabel), **The Sleepy Snail** tavern (Barley): food with buffs.
  * **Glimmer & Gold** (Opal): gems and curios, and 50% more for any you sell her.
  * **Cozy Nook** (Wren): furniture for your house, rugs, pictures, wallpaper and floors
    (her grandest pieces only for charming homes), plus lamps, benches, chests and paths.
  * **The Lantern Guild** (Captain Rowan): delving supplies and guild spoils.
  * **Starfall Spellery** (Hazel): spells, the attuning circle, potions and potion herbs.
* **Twenty villagers** with their own looks, outfits, voices and **daily schedules**: they
  open their shops, stroll their favourite corners of town, gather at the tavern in the
  evening and go home at night. Talk to them (they chatter in little blips, with their
  portrait in the talk box), **give gifts** (each has things they love, like and hate) and
  your **friendship** grows to ten hearts, with presents at three, six and nine.
* **150 hand-written story quests**, five to ten from everyone in town, unlocking as you
  make friends, go deeper and make your home more charming: find **keepsakes** lost in the Hollow (Albert's locket on floors
  2-5, Toby's mailbag, Quill's spectacles...), gather **quest drops** from creatures (slime
  hearts, bat fangs, glow oil from wisps, wishing leaves from the deep), slay, reach floors,
  beat guardians, grow and harvest, cook, enchant, ship, fish, fill your Fishdex, furnish
  your house, and **deliver** letters and parcels.
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
  **collections** to complete (every crop, every curio, every dish, every creature, every
  fish). The journal (L) keeps track of it all.

### Fishing

* Buy a **Bamboo Rod** (or a Willow or Sturdy Oak one) and **bait** from Garrick at Sprig &
  Steel, stand by water and **hold J**: a power bar swings up and down while you wind up, and
  letting go throws the line. When the bobber is pulled under and a **!** pops up, press J to
  hook the fish, then **hold J to raise the green bar and let go to sink it**, keeping the
  fish inside until the catch meter fills. Stay with it the whole way for a **perfect** catch
  (and a bigger fish). Walk off, or get hit, and it's gone.
* **41 fish** in eight waters, each with its own shape, colours and markings: the farm pond,
  Bramblewick's stream, and pools in every biome of the Hollow (ponds turn up in bigger cave
  rooms now), even the **lava** of the Ember Depths if your rod can take the heat. Some only
  bite by day, at night or in the rain; rarer fish fight harder, darting, sinking, floating or
  thrashing about. Now and then you'll hook junk, or a **sunken treasure chest**.
* **Rods are gear**, with Reel Power and their own stats: **Bite Speed**, **Line Strength**,
  **Treasure Find** and **Rare Fish**, besides haste, luck and coin find, and tool scrolls
  enchant them. The smithy only sells plain ones. The other nine, up to the Leviathan Rod,
  come from the Hollow with **better rolls and brighter colours**: dropped by the **water
  folk** (bog frogs, drift jellies and puffers that live round underground ponds), found in
  **treasure chests**, and fished up in sunken treasure.
* The journal's **Fishdex** shows every fish you've caught, shadows of the rest (with where
  and when they bite) and your biggest of each. Cook your catch into **14 fish dishes**, from
  Fish & Chips and Seaweed Sushi to the Emperor's Platter, or keep them in a tank at home.

### Your farmhouse, furniture and charisma

* Walk up to your front door to **go inside**. The house starts with a bed, a little stove, a
  counter, a table for tea and a rug, and everything in it can be moved.
* **Place furniture anywhere inside**: hold a piece and press E. **T turns it** and a
  see-through preview shows where it will go; J picks it back up. Rugs go on the floor,
  pictures on the back wall, and **wallpaper** and **flooring** redo the whole room.
* **Wren's Cozy Nook** sells it all: beds and a canopy bed, a stove and a copper range, counters,
  an icebox, tables and chairs, an armchair and a sofa, bookshelves, a wardrobe, a dresser,
  lamps, a candelabra, a fireplace, plants, a grandfather clock, a piano, a globe, a telescope,
  a plush bunny, a fish bowl and a **fish tank**, plus rugs, pictures, wallpapers and floors.
  A few simple pieces can be crafted too.
* Furniture does things: sleep in the bed, **cook at the stove** (a copper range sometimes
  makes a second helping), sit down, play the piano, spin the globe, look at the stars through
  the telescope (a little luck for the night), warm up by the fire (regeneration), and the
  clock chimes the hours. Put your fish in a **tank** (six) or a **bowl** (two) and watch
  them swim.
* **Charisma**: every piece makes your home more charming (the first two of a kind count most,
  so variety beats fifty chairs), fish in tanks add more, and so do new wallpaper and floors.
  Your charisma (Plain, Cozy, Charming, Delightful, Dazzling) shows in the corner at home, and:
  * shopkeepers treat you well: **up to 20% off** everything, and **up to 10% more** when
    you sell;
  * **friendships grow faster** (up to 60%), and villagers bring you little **gifts** and
    gossip about your lovely home;
  * Wren saves her **grandest pieces** (the fireplace, the copper range, the canopy bed, the
    piano) for homes that will do them justice;
  * and it opens **home quests** with the best rewards in town.

### The Hollow (endless and procedural)

* Every floor is generated from your world seed and its depth: rooms, winding corridors,
  ore veins, pots, crates, treasure chests and torches. The stairs down are always as far from
  where you land as the floor allows.
* The biome changes every ten floors (Mossy Burrows, Crystal Grotto, Fungal Hollow, Ember
  Depths, Frost Caverns, Sunken Ruins) and then cycles, tougher each time. Each biome has its
  own creatures (slimes, bats, shroomlings, crystal crabs, wisps, beetles, imps, skeletons,
  golems, ghosts, and bog frogs, drift jellies and puffers round the ponds), its own ores and
  its own **seeds**. Slimes are wobbly **see-through jelly**
  with a nucleus and bubbles floating inside; ghosts are translucent too.
* Steel on stone throws **sparks** that glow and light up the cave around them: mining rock
  and ore, blades glancing off walls, hitting crabs, beetles, skeletons and golems, and
  blocking with a shield.
* **Every tenth floor** a guardian (King Slime, Crystal Matriarch, Old Capwood, Ember Lord,
  Frost Colossus, the Bone Warden) blocks the stairs and a **waystone**. Defeat it, touch the
  waystone, and you can go home. The Hollow's entrance on your farm then lets you start from
  any waystone you have attuned.
* Fainting in the Hollow sends you home: you wake in your own bed the next morning, a tenth
  of your money lighter. A
  Homeward Feather gets you out early.

### Magic: spells and potions

* **Hazel** keeps the **Starfall Spellery** at the south end of town. Meet her and she hands
  you your first spell, **Firebolt**, for free; the others she teaches for coins once you've
  been deep enough (or gives them away as quest rewards):
  * **Firebolt**: a ball of fire that sets things burning. Splits in three at level 4, five
    at level 8.
  * **Mend**: heals you, more at every level, plus a share of your max HP from level 3.
  * **Bloom**: waters every dug tile around you and gives the crops a chance of a growth
    spurt. In the Hollow it bursts into vines that tangle and slow foes.
  * **Frost Nova**: a ring of ice that hurts and slows everything near you; it freezes
    foes solid from level 5.
  * **Blink**: step through the air to a spot ahead, untouchable for a moment.
  * **Chain Spark**: lightning that leaps from foe to foe, one more foe every two levels.
  * **Ward**: a see-through bubble of light that soaks up damage, and throws some of it back
    from level 6.
  * **Starfall**: stars rain down on the foes around you.
* Spells cost **mana** (it comes back on its own; Spirit gear speeds it up, Wisdom raises the
  cap, Focus makes spells cheaper) and each has a short cooldown.
* **Two spells at a time**, on **Q** and **R**. Which two is up to you, but you can only
  change them at Hazel's **attuning circle**: plan your loadout before you head down.
* **Spells grow with use.** Every cast is practice, and landing it on foes counts extra. Each
  level (up to 10) makes a spell stronger and **5% cheaper to cast**, and some gain new tricks.
  The HUD shows each spell's level and cooldown next to the hotbar.
* **Potions** in three sizes, for **health**, **mana** and **energy**, brewed from the new
  **Potions** page of the crafting book: glass vials plus heartleaf, blueberries or cave
  carrots make two small potions; two smalls brew into a medium, two mediums into a large.
  Potions work instantly (even mid-fight), turn up in pots and chests in the Hollow, and are
  sold at the Spellery, the guild, the tavern and Burrowby's stall.
* Hazel has **ten quests** of her own: cast spells, gather heartleaf, brew potions, train a
  spell to level 4 and then to 10, calm the ghosts... with spells, big potions, wish stars
  and her own wand as rewards.

### Loot, gear and enchanting

* **135 weapons, pieces of armour, tools and fishing rods**, every one of them cute: swords (from a Twig
  Sword, a Carrot Blade and the Hero's Baguette to the Starlight Sword), **wands** that shoot
  magic bolts and **staffs** that set off blasts (both use mana), **shields** (a Pot Lid, a
  Turtle Shell, a Mushroom Shield...), **headgear** (Straw Hat, Flower Crown, Cat-Ear, Frog and
  Bunny Hoods, a Wizard Hat, a Miner's Hat with a lamp...), **chest armour**, **leg armour**,
  **boots** (Rain Boots, Frog and Bunny Slippers, Feather Boots...) and farming **tools**
  (hoes, watering cans shaped like ducks, teapots and frogs, sickles, axes, pickaxes), and
  **fishing rods** from bamboo to coral, jellyfish, starlight and leviathan.
  Everything you wear shows up on your little farmer.
* Every piece has an **item level** from where it was found, a rolled base value, an innate
  bonus and up to four **random stats** out of 32 (damage, crit, speed, heartsip, burn, chill
  and shock, defense, max HP, energy and mana, regeneration, move speed, dodge, block, thorns,
  luck, coin find, tool power, reach, water, bounty, growth, forage, bite speed, rare fish...). Its **rarity**
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
hollowbloom --magic-shots DIR     # jelly slimes, sparks, the breeze, potions, spells
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
  lantern, enemy magic, sparks, falling stars and lightning) accumulated on a grid of tile
  corners with line-of-sight against walls.
* **Translucency with 32 colours**: two more precomputed tables, one for *glass* (a colour
  seen through tinted jelly: the light behind is filtered by the jelly's colour, then mixed
  with it) and one for *glow* (light added on top), each at eight opacities. A translucent
  surface dithers between opacities, thickening towards its silhouette (a Fresnel rim) and
  catching a specular glint, so slimes look like gelatine and a Ward like a soap bubble.
  Translucent things are drawn last, far to near. Soft glows round sparks and bolts are
  dithered discs through the glow table.
* **Wind**: grass is meshed per 8x8-tile block and bent per vertex by a wind field (a steady
  breeze, gust fronts rolling downwind and a flutter), which also leans every plant and tree
  and pushes grass aside round the hero's feet.
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
