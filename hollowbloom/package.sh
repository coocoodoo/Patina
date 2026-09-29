#!/usr/bin/env bash
# Builds release packages of Hollowbloom for Linux and Windows from a single machine.
#
#   hollowbloom/package.sh            # both platforms
#   hollowbloom/package.sh linux      # just one
#   hollowbloom/package.sh windows
#   hollowbloom/package.sh handheld   # 64-bit ARM Linux handhelds (RG351P and friends)
#
# Linux is built with cargo-zigbuild when it is installed (portable: glibc 2.28 and newer),
# otherwise with the host toolchain. Windows is cross-compiled with MinGW-w64 when
# `x86_64-w64-mingw32-gcc` is available, otherwise with cargo-zigbuild. The handheld build
# needs cargo-zigbuild and comes laid out for a ports folder.
# Packages land in dist/.
set -euo pipefail
cd "$(dirname "$0")/.."

version=$(grep -m1 '^version' hollowbloom/Cargo.toml | cut -d'"' -f2)
if [ "$#" -eq 0 ]; then targets=(linux windows); else targets=("$@"); fi
mkdir -p dist

have() { command -v "$1" >/dev/null 2>&1; }

zip_dir() { # zip_dir <dir>  ->  <dir>.zip
    local parent name
    parent=$(dirname "$1")
    name=$(basename "$1")
    rm -f "$1.zip"
    if have zip; then
        (cd "$parent" && zip -qr "$name.zip" "$name")
    else
        python3 - "$1" "$1.zip" <<'PY'
import os, sys, zipfile
src, out = sys.argv[1], sys.argv[2]
base = os.path.dirname(os.path.abspath(src))
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(src):
        for f in files:
            p = os.path.join(root, f)
            z.write(p, os.path.relpath(p, base))
PY
    fi
}

stage() { # stage <name> <binary>
    local dir="dist/$1"
    rm -rf "$dir" && mkdir -p "$dir"
    cp "$2" "$dir/"
    cp hollowbloom/README.md "$dir/README.md"
    cp LICENSE "$dir/LICENSE"
}

for t in "${targets[@]}"; do
    case "$t" in
    linux)
        if have cargo-zigbuild; then
            cargo zigbuild -p hollowbloom --release --target x86_64-unknown-linux-gnu.2.28
        else
            cargo build -p hollowbloom --release --target x86_64-unknown-linux-gnu
        fi
        name="hollowbloom-$version-linux-x86_64"
        stage "$name" target/x86_64-unknown-linux-gnu/release/hollowbloom
        tar -C dist -czf "dist/$name.tar.gz" "$name"
        echo "built dist/$name.tar.gz"
        ;;
    windows)
        rustup target add x86_64-pc-windows-gnu >/dev/null 2>&1 || true
        if have x86_64-w64-mingw32-gcc; then
            cargo build -p hollowbloom --release --target x86_64-pc-windows-gnu
        elif have cargo-zigbuild; then
            cargo zigbuild -p hollowbloom --release --target x86_64-pc-windows-gnu
        else
            echo "install MinGW-w64 (gcc-mingw-w64-x86-64) or cargo-zigbuild to build for Windows" >&2
            exit 1
        fi
        name="hollowbloom-$version-windows-x86_64"
        stage "$name" target/x86_64-pc-windows-gnu/release/hollowbloom.exe
        zip_dir "dist/$name"
        echo "built dist/$name.zip"
        ;;
    handheld)
        rustup target add aarch64-unknown-linux-gnu >/dev/null 2>&1 || true
        if ! have cargo-zigbuild; then
            echo "install cargo-zigbuild to build for handhelds" >&2
            exit 1
        fi
        cargo zigbuild -p hollowbloom --release --target aarch64-unknown-linux-gnu.2.17
        name="hollowbloom-$version-handheld-aarch64"
        dir="dist/$name"
        rm -rf "$dir" && mkdir -p "$dir/hollowbloom"
        cp hollowbloom/port/Hollowbloom.sh "$dir/"
        cp target/aarch64-unknown-linux-gnu/release/hollowbloom hollowbloom/port/README.txt \
            LICENSE "$dir/hollowbloom/"
        chmod +x "$dir/Hollowbloom.sh" "$dir/hollowbloom/hollowbloom"
        # Zipped from inside, so it unpacks straight into a ports folder.
        rm -f "dist/$name.zip"
        if have zip; then
            (cd "$dir" && zip -qr "../$name.zip" Hollowbloom.sh hollowbloom)
        else
            python3 - "$dir" "dist/$name.zip" <<'PY'
import os, sys, zipfile
src, out = sys.argv[1], sys.argv[2]
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(src):
        for f in files:
            p = os.path.join(root, f)
            z.write(p, os.path.relpath(p, src))
PY
        fi
        echo "built dist/$name.zip"
        ;;
    *)
        echo "unknown target: $t (use linux, windows or handheld)" >&2
        exit 1
        ;;
    esac
done
