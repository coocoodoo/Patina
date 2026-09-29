#!/usr/bin/env python3
"""Encodes the recorded tracks the game builds in: music/NAME.mp3 -> music/ogg/NAME.ogg.

    tools/encode_tracks.py [NAME ...]      # from the hollowbloom folder, after a release build

Each MP3 is decoded by the game itself (`hollowbloom --decode`), so the Ogg Vorbis file holds
exactly the frames the loop points in src/audio/track.rs were measured on, then encoded at
about 100 kbps: small enough that the game, all its music inside, stays under 30 MB. Needs the
soundfile package (with libsndfile's Vorbis encoder).
"""

import os
import subprocess
import sys
import tempfile

import soundfile as sf

GAME = os.path.join("..", "target", "release", "hollowbloom")
# 0 is the best quality, 1 the smallest; 0.7 averages about 100 kbps on these tracks.
LEVEL = 0.7


def main():
    names = sys.argv[1:] or sorted(f[:-4] for f in os.listdir("music") if f.endswith(".mp3"))
    os.makedirs(os.path.join("music", "ogg"), exist_ok=True)
    for name in names:
        with tempfile.TemporaryDirectory() as tmp:
            wav = os.path.join(tmp, name + ".wav")
            subprocess.run([GAME, "--decode", os.path.join("music", name + ".mp3"), wav], check=True)
            x, rate = sf.read(wav, dtype="float32")
        out = os.path.join("music", "ogg", name + ".ogg")
        # Written a second at a time: libsndfile's Vorbis encoder can fall over on one big block.
        with sf.SoundFile(out, "w", rate, x.shape[1], format="OGG", subtype="VORBIS", compression_level=LEVEL) as f:
            for i in range(0, len(x), rate):
                f.write(x[i : i + rate])
        kbps = os.path.getsize(out) * 8 / (len(x) / rate) / 1000
        print(f"{out}: {kbps:.0f} kbps")


if __name__ == "__main__":
    main()
