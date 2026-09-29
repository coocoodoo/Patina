#!/usr/bin/env python3
"""Finds where a recorded music track can loop seamlessly, and how loud it is.

    hollowbloom --decode music/title.mp3 title.wav      # decode just as the game does
    tools/find_loop.py title.wav [--min 40] [--plot title.png] [--seam title_seam.wav]

It compares every moment of the track with every other (by their spectra) to find a pair of
points where the music is the same, preferring the longest loop, lines the two up to the
sample by cross-correlation, and prints the loop (in frames) and the gain that brings the
track to the same loudness as the chiptune songs. With --seam it writes a few seconds either
side of the jump, played the way the game blends it, to listen to. Needs numpy and scipy.
"""

import argparse
import wave

import numpy as np
from scipy.signal import stft

RATE = 44100
HOP = 2048
# The chiptune songs' loudness before the mixer's scaling, which tracks are matched to.
TARGET_RMS_DB = -18.0
SEAM = 1764


def load(path):
    w = wave.open(path)
    assert w.getframerate() == RATE and w.getsampwidth() == 2
    x = np.frombuffer(w.readframes(w.getnframes()), dtype=np.int16).astype(np.float32)
    return x.reshape(-1, w.getnchannels()) / 32768.0


def features(mono):
    """A log-spectrum in 48 bands for every hop, normalised for cosine similarity."""
    f, _, z = stft(mono, fs=RATE, nperseg=4096, noverlap=4096 - HOP, boundary=None, padded=False)
    mag = np.abs(z)
    edges = np.geomspace(60, 12000, 49)
    bands = np.stack([mag[(f >= lo) & (f < hi)].sum(axis=0) for lo, hi in zip(edges[:-1], edges[1:])])
    feat = np.log1p(bands * 100).T
    feat -= feat.mean(axis=1, keepdims=True)
    feat /= np.linalg.norm(feat, axis=1, keepdims=True) + 1e-9
    return feat


def loudness(mono):
    win = RATE // 4
    n = len(mono) // win
    return 20 * np.log10(np.sqrt((mono[: n * win].reshape(n, win) ** 2).mean(axis=1)) + 1e-9)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("wav")
    ap.add_argument("--min", type=float, default=40.0, help="shortest loop, in seconds")
    ap.add_argument("--window", type=float, default=4.0, help="seconds compared each side")
    ap.add_argument("--plot")
    ap.add_argument("--seam")
    a = ap.parse_args()

    x = load(a.wav)
    mono = x.mean(axis=1)
    frames = len(mono)
    loud = loudness(mono)
    body = np.median(loud[loud > loud.max() - 30])
    # Where the music proper ends: before any fade-out or final ringing chord.
    ok = np.where(loud > body - 6.0)[0]
    first_q, last_q = ok[0], ok[-1]
    music_start = first_q * RATE // 4
    music_end = (last_q + 1) * RATE // 4

    feat = features(mono)
    n = len(feat)
    w = int(a.window * RATE / HOP)
    sim = feat @ feat.T
    min_lag = int(a.min * RATE / HOP)
    # The end has room for the window after it; the start has the window before it.
    e_hi = min(n - w - 1, music_end // HOP - w)
    s_lo = max(w, music_start // HOP + w)
    best = []
    kernel = np.ones(2 * w + 1)
    for lag in range(min_lag, e_hi - s_lo + 1):
        d = np.diagonal(sim, offset=lag)
        score = np.convolve(d, kernel, mode="same") / len(kernel)
        s = np.arange(len(d))
        e = s + lag
        # The end anywhere in the second half of the music.
        mid = (s_lo + e_hi) // 2
        valid = (s >= s_lo) & (e <= e_hi) & (e >= mid)
        if valid.any():
            i = np.argmax(np.where(valid, score, -1))
            best.append((score[i], lag, s[i], e[i]))
    best.sort(reverse=True)
    top = best[0][0]
    # Of the loops nearly as good as the best, the longest.
    good = [b for b in best if b[0] >= top - 0.02]
    score, lag, s, e = max(good, key=lambda b: b[1])
    start, end = s * HOP + 2048, e * HOP + 2048

    # Line the start up with the end to the sample.
    span = 4096
    ref = mono[end - span : end + span]
    best_c, best_d = -2, 0
    for dlt in range(-2048, 2049, 4):
        seg = mono[start + dlt - span : start + dlt + span]
        c = float(np.dot(ref, seg) / (np.linalg.norm(ref) * np.linalg.norm(seg) + 1e-9))
        if c > best_c:
            best_c, best_d = c, dlt
    for dlt in range(best_d - 4, best_d + 5):
        seg = mono[start + dlt - span : start + dlt + span]
        c = float(np.dot(ref, seg) / (np.linalg.norm(ref) * np.linalg.norm(seg) + 1e-9))
        if c > best_c:
            best_c, best_d = c, dlt
    start += best_d

    rms = 20 * np.log10(np.sqrt((mono[start:end] ** 2).mean()) + 1e-9)
    gain = min(1.0, 10 ** ((TARGET_RMS_DB - rms) / 20))
    print(f"frames {frames} ({frames / RATE:.1f}s), music {music_start / RATE:.1f}s to {music_end / RATE:.1f}s")
    print(f"loop {start}..{end} ({start / RATE:.2f}s to {end / RATE:.2f}s, {(end - start) / RATE:.1f}s long)")
    print(f"match {score:.3f} (best {top:.3f}), seam correlation {best_c:.3f}")
    print(f"loudness {rms:.1f} dBFS, gain {gain:.3f}")
    if score >= 0.75:
        print(f"RUST start: {start}, end: {end}, rest: 0, gain: {gain:.3f}")
    else:
        # No part repeats closely enough: play it all the way through, rest, start over.
        rms = 20 * np.log10(np.sqrt((mono[music_start:music_end] ** 2).mean()) + 1e-9)
        gain = min(1.0, 10 ** ((TARGET_RMS_DB - rms) / 20))
        print(f"weak match: play it through instead (loudness {rms:.1f} dBFS)")
        print(f"RUST start: 0, end: {frames}, rest: {2 * RATE}, gain: {gain:.3f}")

    if a.seam:
        # Six seconds up to the loop's end, blended into the start as the game does, then on.
        pre, post = 6 * RATE, 6 * RATE
        out = x[end - pre : end].copy()
        k = np.arange(SEAM)
        wgt = (0.5 - 0.5 * np.cos(np.pi * k / SEAM))[:, None]
        out[-SEAM:] = out[-SEAM:] * (1 - wgt) + x[start - SEAM : start] * wgt
        out = np.concatenate([out, x[start : start + post]])
        ww = wave.open(a.seam, "wb")
        ww.setnchannels(x.shape[1])
        ww.setsampwidth(2)
        ww.setframerate(RATE)
        ww.writeframes((np.clip(out, -1, 1) * 32767).astype(np.int16).tobytes())
        ww.close()

    if a.plot:
        import matplotlib

        matplotlib.use("Agg")
        import matplotlib.pyplot as plt

        fig, ax = plt.subplots(2, 1, figsize=(16, 7))
        t = np.arange(len(loud)) / 4
        ax[0].plot(t, loud, lw=0.8)
        for v, c in [(start, "g"), (end, "r"), (music_start, "k"), (music_end, "k")]:
            ax[0].axvline(v / RATE, color=c, ls="--")
        ax[0].set_title(f"loudness (loop green to red) {a.wav}")
        ax[1].imshow(feat.T, aspect="auto", origin="lower", extent=[0, n * HOP / RATE, 0, 48], cmap="magma")
        for v, c in [(start, "g"), (end, "r")]:
            ax[1].axvline(v / RATE, color=c, ls="--")
        ax[1].set_title("spectrum bands")
        plt.tight_layout()
        plt.savefig(a.plot, dpi=60)


if __name__ == "__main__":
    main()
