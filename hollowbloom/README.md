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
**51 kinds of fish**, cook them on the stove in your **farmhouse**, and fill it with furniture:
a charming home gets you better prices, warmer friends and some very special quests.
The **sun** throws real shadows that swing round through the day, the **moon** waxes and
wanes (and a full moon drives the Hollow wild), every biome has its own **zombies, goblins,
bugs, skeletons and ghosts**, and every tenth floor ends with a **giant** and a hoard of
treasure chests. Recipes are **learned by finding their ingredients**, bombs open **secret
rooms**, and it all plays on a **Steam Deck**.

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

**Light and shadow**

| | |
| --- | --- |
| ![Morning: long shadows to the west](docs/sun-morning.png) | ![Evening: shadows swing east](docs/sun-evening.png) |
| ![A full moon casts shadows too](docs/full-moon-night.png) | ![Bramblewick's buildings shading the plaza](docs/town-shadows.png) |
| ![Ambient occlusion in the Hollow: every crease and corner](docs/ambient-occlusion.png) | ![Under a full moon the Hollow glows red](docs/moonlit-monsters.png) |

**The creatures of the Hollow: a look for every biome**

| | |
| --- | --- |
| ![Mossy Burrows](docs/monsters-mossy.png) | ![Crystal Grotto](docs/monsters-crystal.png) |
| ![Fungal Hollow](docs/monsters-fungal.png) | ![Ember Depths](docs/monsters-ember.png) |
| ![Frost Caverns](docs/monsters-frost.png) | ![Sunken Ruins](docs/monsters-ruins.png) |
| ![A bug for every biome: moss spider, glass mantis, spore moth, fire ant, frost spider and scarab](docs/bugs.png) | ![Moss and frost spiders up close, one scuttling](docs/spiders.png) |
| ![King Slime, nearly three times life size](docs/guardian-giant.png) | ![Grub the Goblin King](docs/goblin-king.png) |
| ![A guardian's hoard: chests and loot](docs/guardian-hoard.png) | ![A gleaming chest](docs/gleaming-chest.png) |

**Recipes, bombs, secret rooms and the Steam Deck**

| | |
| --- | --- |
| ![A new recipe](docs/new-recipe.png) | ![The crafting book, with recipes still to find](docs/recipe-book.png) |
| ![Garrick's bombs, five a day](docs/bombs.png) | ![Boom](docs/bomb-blast.png) |
| ![The cracked floor gave way](docs/secret-hole.png) | ![A secret room](docs/secret-room.png) |
| ![The Steam Deck controls](docs/steam-deck-controls.png) | |

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

### Steam Deck (and controllers)

Hollowbloom reads the Steam Deck's controls itself, and any Xbox-style controller too, with
nothing extra to install. On the Deck, add the Linux build (or the Windows `.exe`, run with
Proton) to Steam as a **non-Steam game** and play it from **Game Mode**: Steam Input hands its
controls to the game. It opens fullscreen and fills the screen's 1280x800 edge to edge. Every
on-screen hint shows the Deck's buttons (**A**, **X**, **L2**...), and the pause menu's
**Controls** page shows the whole layout on a drawing of the Deck.

Putting the Linux build (a single file) on a Deck:

1. In Desktop Mode, copy the `hollowbloom` file into your home folder, right-click it, and
   under **Properties > Permissions** tick **Is executable**.
2. Right-click it again and choose **Add to Steam** (or in Steam, **Games > Add a Non-Steam
   Game to My Library**).
3. Back in Game Mode it's in your library under **Non-Steam**. Saves go in
   `~/.local/share/hollowbloom`.

**Steam Controllers** (the 2015 one and the 2026 one) work the same way, through Steam Input.
With Steam closed, the game reads Linux's own driver for the Deck's controls and Steam
Controllers instead: on the Deck and the 2026 controller, hold **Menu** for half a second to
switch them from mouse mode to gamepad mode.

| | Steam Deck |
| --- | --- |
| Walk (gently for a stroll) / aim without walking | left stick, D-pad / right stick |
| Talk, open, harvest, place, eat; OK in menus | A |
| Use tool or weapon (hold to keep going; fishing too) | X |
| Dodge roll; back in menus; turn furniture at home | B |
| Bag / crafting book | Y / View |
| Hotbar left / right; flip tabs in menus | L1 / R1 |
| Cast your two spells | L2 / R2 |
| Map / quest journal | click the left stick (L3) / right stick (R3) |
| Pause (and the Controls page) | Menu |
| In the bag and shops: split a stack, wear, sell just one | X |

### Linux handhelds (Anbernic RG351P and friends)

The same game runs on 64-bit ARM handhelds that run Linux without a desktop, such as the
**Anbernic RG351P**, RG351M and RG351V on ArkOS, AmberELEC or ROCKNIX. There it draws through
SDL2, which every such system ships (it's loaded at run time, so nothing is needed anywhere
else), fills the screen at its own resolution (480x320 on an RG351P, pixel for pixel), and
reads the built-in controls through SDL's mapping for the handheld (PortMaster's, when it's
installed). Hints and the Controls page name the buttons as printed, **Select** and
**Start**, the journal and minimap are in the pause menu for sticks that don't click in, and
holding **Select + Start** for a second saves and quits, as other ports do. Sun shadows and
ambient occlusion start off, as they halve the frame rate on a small processor.

`hollowbloom/package.sh handheld` builds it (with cargo-zigbuild) and lays it out for a
ports folder: copy `Hollowbloom.sh` and the `hollowbloom` folder into `roms/ports` and it
shows up under **Ports** in EmulationStation. Saves go in the `hollowbloom` folder beside the game (flushed to the card as they're written),
and if A and B come out the wrong way round, `Hollowbloom.sh` has a line to swap them. The
game picks SDL by itself when there's no desktop; `--sdl` asks for it anywhere.

### The farm (Stardew-style)

* **Till** any open ground on your property with the hoe (lawn, bare soil, the paths, the
  sand by the pond), **plant** seeds, **water** every day with the can (refill it at the
  pond), and **sleep** in your house to end the day. Crops grow one stage per watered day;
  many (strawberries, tomatoes, corn, blueberries, Crystal Berries, Ember Peppers...) keep
  fruiting after the first harvest. **Sweep ripe crops with a sickle** to harvest a whole patch.
* **78 crops**. Thirty-eight grow all year: turnips, potatoes, radishes, cabbages, tomatoes,
  strawberries, wheat, corn, blueberries, eggplants, garlic, sunflowers, roses and sweet peas
  from Burrowby's stall, and strange seeds from every biome of the Hollow: Mossberries and
  Bunnyroots, Prism Pears and Geode Gourds, Puffballs, Jelly Shrooms and Truffles, Flame
  Tulips, Lava Lemons and Magma Melons, Snow Peas, Ice Plums and Frost Mint, Starfruit, Ghost
  Peppers and Ancient Grain.
* **Forty seasonal crops**, ten to a season, that only grow in theirs (a few straddle two):
  pastel tulips, daffodils, bluebells, lavender, sweet cherries, cloudberries, rhubarb,
  spring onions, butter lettuce and baby carrots in **spring**; watermelons, honeydew,
  pineapples, peaches, raspberries, bell peppers, zucchini, hibiscus, poppies and cosmos in
  **summer**; pumpkins, butternut squash, sweet potatoes, cranberries, grapes, crabapples,
  chestnuts, beetroot, chrysanthemums and candy corn in **autumn**; and snowdrops, glowing
  snow roses, poinsettias, mistletoe, holly, kale, parsnips, sprouts, glowing snow melons
  and frostberries in **winter**. Posy sells every seed of the season and Burrowby the
  cheaper ones. Out of season they won't go in, and whatever is still in the ground when
  the season turns withers overnight (the evening before, the day's summary warns you).
* **Cook** them into 57 dishes on the **stove** in your house (or the campfire at any
  waystone). The good ones give a **buff** for a few minutes: Garlic Bread for damage,
  Strawberry Shortcake for luck, Frost Mint Tea for mana, Lava Lemonade to set things on
  fire...
* **Recipes are learned, not given.** You start knowing what can be made from wood, stone
  and fibre; everything else you **work out by carrying one of each of its ingredients**
  (however many it really takes). The moment you do, the game stops to show you the new
  recipe: its picture, its name, what it's good for (healing, energy, buffs, stats) and what
  it needs. The crafting book shows the ones still to find as **???**.
* **The sun moves.** It rises in the east, crosses the south and sets in the west, and
  everything that stands up (you, the house, trees, crops, fences, villagers, buildings)
  casts a **shadow** that swings round and shortens towards noon. By night the **moon** takes
  over, as bright as its phase. Rain softens it all. (Settings can turn the sun's shadows and
  the ambient occlusion off, for slower machines.)
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
  cozy.
* **Four seasons** of 28 days each: spring from day 1, then summer, autumn and winter, and
  round again (the clock shows which, and the morning summary says when a new one begins).
  Each looks and feels its own. In **spring** the trees blossom pink and white, the bushes
  flower, petals drift on the breeze and wildflowers are everywhere. **Summer** is lush,
  with butterflies by day and fireflies by night. In **autumn** the trees and bushes turn
  gold, orange and red, the grass goes gold at the tips and leaves twirl down over the farm
  and the town. **Winter** lays snow over the lawns, hedges, trees and bushes, frosts the
  grass, snows instead of raining (a blizzard now and then) and brings pale blue, moonlit
  nights; the wildflowers wait under the snow for spring.
* **The cat**, a ginger tabby with white socks, a white bib and a white-tipped tail, has a
  life of its own: it strolls about (off to see you, now and then), sits with its tail round
  its paws, takes catnaps and curls up asleep by the house at night. Press E beside it for a
  fuss.
* **A jumping spider.** In autumn Pip has a secret to trade (see *A Sweet Secret* below), and
  it's an egg. Set it down anywhere on the farm and it sits in a nest of fallen leaves for
  ten days: it hums and taps, its spots glow at night, it **tips over on the ninth day**,
  and on the **tenth** it hatches as soon as you come near. Out hops a fluffy little jumping
  spider with big glossy eyes, a pink heart on its back and teal fangs. It hops about in
  quick little jumps, snaps round to look at you, waves its front legs when you give it a
  fuss (E), and **plays with the cat**: the cat stalks it, wiggles and pounces, and the
  spider springs out of the way or right up onto the cat's back for a ride. At night the two
  of them sleep curled up together.
* The **moon** goes round its eight phases every eight days (the clock shows tonight's, and
  the morning summary says what it means below). A new moon lulls the Hollow's creatures; a
  **full moon** riles them up: they **glow red**, notice you from further off, move and strike
  faster, **take more beating and hit harder**, and **drop more and better loot**.

### Bramblewick (the town down the road)

* A country road now runs along the farm's east edge. **Wave the bus down** at the shelter and
  it drives you to **Bramblewick**: streets and a plaza, nineteen storybook buildings, a
  stream with a park beyond it, and a bus back home whenever you like. Road corners are
  rounded, on the farm and in town.
* **Twelve buildings you can walk into**, each a little diorama: the Town Hall, the Lantern
  Guild, and **ten specialist shops**, each with its own shelves, **specials rolled fresh
  every morning**, and better prices for the things it deals in. **Shops keep shop hours:
  9am to 5pm**, Burrowby's stall included (the guild and the tavern stay open later, but
  their counters trade nine to five too), and at five the shopkeepers see you out:
  * **Petalplate Armory** (Hilde): helmets, chest armour, leg armour, boots and shields.
  * **Sprig & Steel** (Garrick): swords, wands, staffs, plain fishing rods and bait, and
    **bombs** (only five a day; he restocks every morning).
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
* **176 hand-written story quests**, five to ten from everyone in town, unlocking as you
  make friends, go deeper and make your home more charming: find **keepsakes** lost in the Hollow (Albert's locket on floors
  2-5, Toby's mailbag, Quill's spectacles...), gather **quest drops** from creatures (slime
  hearts, bat fangs, glow oil from wisps, wishing leaves from the deep), slay, reach floors,
  beat guardians, grow and harvest, cook, enchant, ship, fish, fill your Fishdex, furnish
  your house, and **deliver** letters and parcels.
  Keepsakes and quest drops **only exist while the quest is open**, and a guardian **returns
  to its floor** for anyone who needs something it carries. A **!** marks someone with a
  request and a **?** someone you can hand one in to.
* **Seasonal requests**: thirteen villagers ask for the season's crops while they're in
  (Posy's first tulips, Mabel's cherry tarts and cranberry pies, the mayor's Harvest
  Festival pumpkins, Pip's pineapple party, Juniper's snowdrops...), the request board asks
  for **what's in season**, villagers love seasonal gifts, and Posy's last almanac wants all
  seventy-eight crops, spring to winter.
* **A Sweet Secret** (Pip, autumn only, and only ever once): when floor 10's guardian falls,
  **candy rocks** burst up out of the floor round it, one after another. Each grinds up out
  of the cracking floor in a spray of sparks, then hisses and **flashes white faster and
  faster** as it swells, like a lit fuse, and goes off with a bang, a flash and a cloud of
  smoke before setting hard: strawberry, cherry, candy corn and grape rock candy. Knock five
  loose with a pickaxe and trade them to Pip for a mysterious egg.
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
* Landed, the fish is **held up to show off** while the camera leans in on it, its name,
  size and rarity over it (and "New!", "Record!" or "Perfect!"). Press **A** (E on the
  keyboard) to **put it in your bag**; if your bag is full, you **throw it back** into the
  water instead (it still counts in your Fishdex).
* **51 fish** in nine waters, each with its own shape, colours and markings: the farm pond,
  Bramblewick's stream, and pools in every biome of the Hollow (ponds turn up in bigger cave
  rooms now), even the **lava** of the Ember Depths if your rod can take the heat. Some only
  bite by day, at night or in the rain; rarer fish fight harder, darting, sinking, floating or
  thrashing about. Now and then you'll hook junk, or a **sunken treasure chest**.
* The **old sewers' channels** have ten odd fish of their own, each drawn by hand: the
  **Sock Eel** (a striped sock with an eye on its toe), the **Boot Carp** peering out of an
  old boot, the **Googly Guppy**, the **Mustache Minnow** and its ginger handlebar, the warty
  **Pickle Pike**, the **Mop Catfish** that can't see past its whiskers, the **Tin Can
  Tetra**, the sad pink **Bubblegum Blobfish**, the **Two-Headed Goby** (one head grinning,
  one frowning) and, rarest of all, **the Sewer King** in his bottle-cap crown. There's twice
  the junk down there, too.
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
* Every ten floors are one biome (Mossy Burrows, Crystal Grotto, Fungal Hollow, Ember
  Depths, Frost Caverns, Sunken Ruins), **in an order of your save's own**: every sixty
  floors go through all six, shuffled, never the same one twice in a row, tougher each time
  round. A quest that needs a biome **holds its floors to it until it's done**: take the
  Guild's imp cull from floor 31 down and, if the Ember Depths aren't anywhere you can get
  to, floors 31 to 40 turn into them (trading places with wherever they were); send out for
  a guardian or its prize and it's on the floor the quest names. The ten floors you're in
  never change under you. Quest words name the biome a floor is in on your save. Each biome
  has its own creatures (slimes, bats, shroomlings, crystal crabs, wisps, beetles, imps,
  golems, and bog frogs, drift jellies and puffers round the ponds), its own ores and its
  own **seeds**.
  Slimes are wobbly **see-through jelly** with a nucleus and bubbles floating inside; ghosts
  are translucent too.
* Five families turn up everywhere, **dressed for each biome**: **zombies** (mossy,
  crystal-studded, sprouting mushrooms, charred, frozen, and **mummies** in the ruins) that
  shamble with their arms out and lunge; **fat goblin brutes** with clubs that slam the ground
  and send out a shockwave; **skinny goblin sneaks** that circle, dart in to stab and throw
  daggers; **bugs** (long-legged moss spiders, glass mantises, flying spore moths, glowing fire
  ants, icy frost spiders and scarabs) that scuttle and pounce; and **skeletons** and
  **ghosts** with a look for every biome, from mossy bones to a pharaoh's guard. They drop
  grave dust, goblin teeth (goblins carry extra coin) and bug chitin, and townsfolk have
  quests for all of them.
* From **floor 11** down, two more live in every biome. **Lantern snails** glide after you
  under shells of **glowing stained glass** (a spiral of leaded panes in the biome's
  colours) that light up the dark round them, leaving a trail of shining slime. Close up,
  they draw back and lunge; strike one and it **hides in its shell**, which turns most of a
  blow aside until it peeks out again, so be patient. **Book-worm bibliomancers** are
  caterpillars in round spectacles with a little book floating open beside them: they keep
  their distance, rear up as the pages glow and **spit gobs of glowing ink**, which dry into
  **runes on the floor whose arrows point the way to the stairs** (a gold one, now and then,
  to a secret room the floor still hides). Snails drop **stained glass** (make a **Snail
  Lamp** for the house), book-worms drop **glow ink** and scrolls, and the ink makes **Ink
  Maps**: read one in the Hollow and a line of runes lights the way down.
* **Drakelings** live from floor 11 down in the Ember Depths (**cinder drakes**, red and
  gold) and the Frost Caverns (**frost drakes**, blue with dark bat wings): little horned
  dragons that circle just out of reach, **rear back with their mouths aglow** and breathe a
  roaring gout of **fire or frost** that they sweep after you. Their scales turn blows into
  sparks. They drop **cinder** or **frost scales**, and now and then ember ore or a frost
  gem.
* **Leaflings** flit about the Mossy Burrows: cross little sprites of leaves with red eyes
  and glassy leaf wings, darting this way and that and throwing **fans of spinning leaves**.
  Every few seconds one **mends whoever's worst hurt** round it (a stream of green light and
  a green ring), so deal with them first. They drop heartleaf, fibre and mossberry seeds.
* **Werewolves** come out **under a full moon**, from floor 5 down (more of them the deeper
  you go, never on a guardian's floor), prowling far from where you came in. The first time
  one catches your scent it throws its head back and **howls**, and every creature in earshot
  comes running, riled up (a second howl riles them further). Then it runs you down in long
  lopes, drops into a crouch and **pounces**, claws first. They drop **werewolf fangs**.
* **The old sewers.** Now and then (from floor 3, never on a guardian's floor) a floor is a
  stretch of sewer instead of caves: murky green channels, misty and bubbling, run between
  **stone walkways two lanes wide on each side**, curbed along the water, meeting at
  junctions and ending in rounded chambers, all standing alone in the dark. The water glows
  a faint, murky **green**. Every stretch of channel has a **bridge** across it, of
  weathered **planks** or of **copper plate gone green with rust**, railed on both sides.
  Drain pipes trickle onto the walkways, grates sit in the floor, torches line the walls,
  **barrels** (some brimming with glowing goo), crates and pots stand along them to smash,
  **old bones** lie about, **broken planks** litter the walkways (chop them for wood) and
  float in the channels, and the dead ends hide chests. The old brickwork can be mined like
  rock.
* **Who lives in the sewers**, whatever the biome above: **sludge slimes**, brown swirls
  with a scowl, glowing red eyes and flies buzzing round them, and everyone else **down to
  their bones**: bone bats with skeletal wings, bone shamblers in rags, bone brutes swinging
  thigh-bone clubs, hooded bone sneaks with rusty daggers, pale bone spiders, slimy sewer
  skeletons, skull ghosts, bone frogs and puffers along the water, and from floor 11 bone
  snails in shells of bone and green glass, and bone bibliomancers spitting green ink. The
  bony ones often leave an old bone behind.
* **The glowcap caves.** Now and then (from floor 4, never on a guardian's floor, and most
  often in the Fungal Hollow) a floor is a cave lit by **glowing mushrooms** instead of
  torches. You come in at the end of a wing of **old ruins**: brick rooms floored with worn
  flagstones along a torch-lit corridor, with barrels, crates, pots, old bones and a chest.
  The corridor opens into a **winding cave** lined with **glowcaps** in blue, cyan, green
  and purple, from little clusters to tall ones you can't walk through, with **shelf fungi**
  glowing up the walls and side passages ending in quiet alcoves. At the far end a **great
  chamber** opens out, with **glowing pools** round a **giant mushroom**, and the stairs
  down. **Sealed nooks** glow in the rock beside the way (dig in with a pickaxe: now and
  then one hides a chest), and past the rock there's nothing but the dark. The light down
  there is cool and blue: the mushrooms and pools light the way, spores drift up off them,
  and the **shroomlings glow** like the caps. Take a **sickle** to the glowcaps and shelf
  fungi for glowcaps, glow spores and now and then glowcap spores to grow your own; the
  pools hold the Fungal Hollow's fish.
* **Bombs** (from Garrick, five a day): throw one and it bounces, fizzes for two seconds and
  goes off, hurting everything close (you too, if you stand too near), bursting pots and
  crates. About half of the floors hide a **cracked patch of floor**; blow it open and a rope
  leads down to a **secret room**: treasure chests along the back wall, pots and crates, and a
  handful of the biome's creatures standing guard. Climb the rope back up to the floor you
  came from; the room stays as you left it until you leave the floor.
* Everything is shaded with **ambient occlusion**: creases, corners, the feet of walls and
  furniture, and whatever lies on the floor sit in a soft darkness of their own colour, so
  things stand out from floors of the same colour.
* Steel on stone throws **sparks** that glow and light up the cave around them: mining rock
  and ore, blades glancing off walls, hitting crabs, beetles, skeletons and golems, and
  blocking with a shield.
* **Every tenth floor** a guardian of its biome (King Slime, Crystal Matriarch, Old Capwood,
  Ember Lord, Frost Colossus, the Bone Warden) waits in an arena, blocking the stairs and a
  **waystone**.
  Guardians are **giants**, nearly three times life size, and strong with it. On your next
  trip down through the biomes the goblin king, the glass queen, the rotting gardener,
  Ashfang the Quick, the frost wraith and the mummy king take their places, calling their
  kind for help. A fallen guardian leaves **a heap of loot** (gear, one piece always finely
  made, scrolls, gems, relics, potions) and **four to eight treasure chests** burst up round
  where it fell. Touch the waystone and you can go home; the Hollow's entrance on your farm
  then lets you start from any waystone you have attuned.
* **Chests twinkle**, and now and then one **gleams gold** (more often under a full moon, and
  among a guardian's hoard): inside is gear rolled about as well as it can be, a lucky scroll,
  gems, a relic and a pile of coin.
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

# 64-bit ARM handhelds (glibc 2.17 and newer), laid out for a ports folder
hollowbloom/package.sh handheld
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
hollowbloom --home-shots DIR      # fishing, the farmhouse, cooking and the furniture shop
hollowbloom --light-shots DIR     # the sun's shadows through the day, the moon, ambient occlusion
hollowbloom --monster-shots DIR   # every monster family in every biome, the moon, the giants
hollowbloom --feature-shots DIR   # recipe cards, the Steam Deck layout, bombs and secret rooms
hollowbloom --pet-shots DIR       # the cat, the jumping spider, its egg, autumn and candy rocks
hollowbloom --season-shots DIR    # the farm and town in every season, day and night
hollowbloom --deep-shots DIR      # lantern snails, book-worms, ink runes and the snail lamp
hollowbloom --sewer-shots DIR     # sewer floors, their clutter, fish, sludge slimes and bony folk
hollowbloom --glowcave-shots DIR  # the glowcap caves: ruins, glowing cave, nooks, pools, giant
hollowbloom --beast-shots DIR     # drakelings breathing fire and frost, leaflings, a werewolf
hollowbloom --music DIR           # every song as a WAV file, and its notes as CSV
hollowbloom --decode IN.mp3 OUT.wav  # a track decoded as the game does (for tools/find_loop.py)
hollowbloom --wardrobe FILE       # the hero in a dozen outfits (and FILE_all: every piece)
hollowbloom --bench               # rendering speed (about 5 ms per frame at 480x270)
hollowbloom --palette-chart FILE  # the light maps: every colour at every light level
hollowbloom --mute                # no sound
hollowbloom --sdl                 # fullscreen through SDL2, as on a handheld
tools/logo.py                     # the title's logo from art/logo-source.jpg
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
* **Sun and moon shadows**: everything that casts is drawn a second time, from the light's
  side, into an orthographic depth map (solid shapes put their far side in it, so their lit
  faces never shadow themselves; cut-out sprites cast their shape). Every pixel on screen is
  then turned back into a world position from the depth buffer and checked against the map;
  shadowed pixels step down their colour's own ramp, dithered by how strong the light is.
* **Ambient occlusion**: a screen-space pass compares each pixel's depth with its neighbours'
  at a few radii in four directions; creases and corners step down the ramp. Both passes run
  across all CPU cores.
* **Controllers** are read straight from the system: the kernel's event devices on Linux
  (Steam Input's virtual pad on the Deck) and XInput on Windows, mapped onto the same actions
  as the keyboard.
* **Wind**: grass is meshed per 8x8-tile block and bent per vertex by a wind field (a steady
  breeze, gust fronts rolling downwind and a flutter), which also leans every plant and tree
  and pushes grass aside round the hero's feet.
* **Everything is procedural or written in code**, but for the logo and the recorded music:
  tile textures, the low-poly models
  (boxes and lathes, including a 3D look for every hat, boot, shield, weapon and tool), about
  390 item icons, crop sprites and stat symbols drawn as ASCII art, two bitmap fonts,
  the sound effects (sfxr-style synthesis) and the chiptune soundtrack: eleven songs (the
  farm by morning and afternoon, night, the town's waltz, the shops, home, three Hollow
  themes, the guardians and the title), each written as sections of chords, a tune and a
  counter-tune, with the arpeggios and bass lines drawn from the chords. Tests check that
  every part fits its bars and that no held note clashes with the chord under it. The
  town's buildings are generated from a short description each (size, walls, roof, awning,
  sign), and villagers find their way about with a simple breadth-first path search over
  the town's tiles.
* **The logo** is drawn from `art/logo-source.jpg`: `tools/logo.py` keys out its green
  backdrop (flooding in from the edges, plus the pockets ringed by its cream outline, like
  the gaps in the H), scales it to the three heights the title picks between, and sets every
  pixel to the nearest palette colour, so it's still nothing but Resurrect 32.
* **Recorded music**: every song has a recorded track (eleven, made with Google's Lyria 3:
  the MP3s are in `music/`), built into the executable as Ogg Vorbis at about 100 kbps so
  the whole game stays one file under 30 MB, and decoded in the background by Symphonia, a
  pure-Rust decoder. Each plays all the way through and starts again after a short pause,
  but for the guardians', which loops seamlessly over its middle (the seam blended) so a
  fight never stops for an ending. When the guardian falls it leaves the loop within a beat,
  in step (the track keeps a steady 145 beats a minute, though its accents shift by half a
  beat in places), for the track's last three and a half seconds, and the floor's own song
  follows. Tracks are matched in loudness, and one you come back to
  within three minutes carries on where it left off. The chiptune band stands in for any
  track that's missing or won't decode. `hollowbloom --decode` and `tools/find_loop.py`
  find a new track's loop and loudness, and `tools/encode_tracks.py` makes the Ogg files.
* **Audio** uses ALSA on Linux (loaded with `dlopen`) and winmm on Windows, from a mixer
  thread.

## Credits

* Palette: [Resurrect 32](https://lospec.com/palette-list/resurrect-32) by Kerrie Lake.
* Inspired by the look of *Final Fantasy: The 4 Heroes of Light*, the farming and townsfolk
  of *Stardew Valley* and the digging freedom of *Core Keeper*.
