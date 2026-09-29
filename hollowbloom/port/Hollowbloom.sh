#!/bin/bash
# Hollowbloom on a Linux handheld: the Anbernic RG351P (and M and V), and the like, on ArkOS,
# AmberELEC, ROCKNIX and friends. Put this script and the hollowbloom folder beside it in your
# ports folder; it shows up under Ports.

# PortMaster, when it's installed, knows this handheld's controls best: take its mapping.
XDG_DATA_HOME=${XDG_DATA_HOME:-$HOME/.local/share}
for dir in /opt/system/Tools/PortMaster /opt/tools/PortMaster "$XDG_DATA_HOME/PortMaster" \
    /roms/ports/PortMaster /storage/roms/ports/PortMaster; do
  if [ -f "$dir/control.txt" ]; then
    controlfolder="$dir"
    break
  fi
done
if [ -n "$controlfolder" ]; then
  source "$controlfolder/control.txt"
  [ -f "$controlfolder/mod_${CFW_NAME}.txt" ] && source "$controlfolder/mod_${CFW_NAME}.txt"
  get_controls
  [ -n "$sdl_controllerconfig" ] && export SDL_GAMECONTROLLERCONFIG="$sdl_controllerconfig"
fi

GAMEDIR="$(cd "$(dirname "$0")/hollowbloom" && pwd)"
cd "$GAMEDIR" || exit 1
# Saves stay with the game, on the card.
export HOLLOWBLOOM_DATA="$GAMEDIR/save"
# If A and B (and X and Y) come out the wrong way round, remove the # from the next line.
# export HOLLOWBLOOM_SWAP_AB=1

chmod +x ./hollowbloom
./hollowbloom --sdl > "$GAMEDIR/log.txt" 2>&1
