# Hollowbloom

**A cozy top-down action RPG. Farm on the surface, delve an endless dungeon below it.**

Your little farm sits right on top of *the Hollow*, a cave that goes down forever. Fight and
dig your way through procedurally generated floors, bring home the strange seeds, ores and
treasures you find, and grow them into a farm that helps you go deeper. Every tenth floor a
guardian watches over a waystone that can take you home.

Everything you see is low-poly 3D with pixel-art textures, drawn by a small software renderer
that **can only ever produce the 32 colours of the
[Resurrect 32](https://lospec.com/palette-list/resurrect-32) palette** by Kerrie Lake:
textures, lighting, shadows, dithering and UI are all palette indices, and the only RGB
in the program is the palette itself, applied at the final blit.

| | |
| --- | --- |
| ![Title](docs/title.png) | ![Welcome](docs/welcome.png) |
| ![Farm in the morning](docs/farm-morning.png) | ![Farm at night](docs/farm-night.png) |
| ![Mossy Burrows](docs/hollow-mossy.png) | ![Crystal Grotto](docs/hollow-crystal.png) |
| ![Ember Depths](docs/hollow-ember.png) | ![Frost Caverns](docs/hollow-frost.png) |
| ![A guardian](docs/guardian.png) | ![The waystone](docs/waystone.png) |
| ![Crafting](docs/crafting.png) | |

## Playing

Grab a build from the releases (`hollowbloom-*-windows-x86_64.zip` or
`hollowbloom-*-linux-x86_64.tar.gz`) or [build it yourself](#building). There is nothing to
install: it is a single executable.

| | Keyboard | Mouse |
| --- | --- | --- |
| Move | WASD / arrows | |
| Use tool, attack | J or Z | left click (aims at the cursor) |
| Interact, harvest, eat, place | E or K | right click |
| Dodge roll | Space or Shift | |
| Hotbar | 1-9, 0, Q / R | wheel |
| Bag / crafting | Tab or I / C | |
| Minimap (in the Hollow) | M | |
| Pause, settings, save & quit | Esc | |
| Fullscreen / screenshot | F11 or Alt+Enter / F12 | |

### The farm (Stardew-style)

* **Till** grass with the hoe, **plant** seeds, **water** every day with the can (refill it at
  the pond), and **sleep** in your house to end the day. Crops grow one stage per watered day;
  some (Crystal Berries, Ember Peppers) keep fruiting after the first harvest.
* **Ship** anything in the bin by the house and the money arrives overnight, or trade with
  Burrowby the mole at his stall. His stock grows the deeper you have been.
* Tools cost **energy**; food restores health and energy. Stay up past 2am and you will
  collapse where you stand. Rainy days water everything for you.
* Craft **sprinklers** from dungeon ores to automate watering (4, 8 or 24 tiles).
* Glowcaps and Moonblooms glow at night; lamps, lit windows and fireflies keep the evenings
  cozy. There is a cat. You can pet the cat.

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
* Fainting in the Hollow sends you home the next morning, a tenth of your gold lighter. A
  Homeward Feather gets you out early.

### Freedom (Core Keeper-style)

* The pickaxe **mines the Hollow's walls**: dig your own tunnels and shortcuts, and break
  ore veins for copper, iron, gold, glimmer shards, ember ore and frost opals.
* **Build anywhere**: stone and wood walls, paths and floors, fences, torches, lamps, chests,
  benches, flower pots, workbenches, sprinklers. Hit them with a pickaxe or an axe to pick them
  back up.
* Better swords, pickaxes, axes and watering cans are crafted from what you bring up.

Progress is saved automatically every morning (and when you quit from the farm):

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
hollowbloom --bench               # rendering speed (about 3 ms per frame at 480x270)
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
  (boxes and lathes), about 80 item icons and crop sprites drawn as ASCII art, two bitmap fonts,
  the sound effects (sfxr-style synthesis) and the chiptune soundtrack (a small four-channel
  sequencer with six songs).
* **Audio** uses ALSA on Linux (loaded with `dlopen`) and winmm on Windows, from a mixer
  thread.

## Credits

* Palette: [Resurrect 32](https://lospec.com/palette-list/resurrect-32) by Kerrie Lake.
* Inspired by the look of *Final Fantasy: The 4 Heroes of Light*, the farming of
  *Stardew Valley* and the digging freedom of *Core Keeper*.
