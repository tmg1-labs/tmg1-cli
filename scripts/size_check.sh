#!/usr/bin/env bash
# 予測 on/off で帯パターンの圧縮サイズを比較し、ラウンドトリップも確認する。
set -u
B=/mnt/d/workspace/TsuMuGi/tmg1-cli/target/release/tmg1
GEN=/mnt/d/workspace/TsuMuGi/tmg1-cli/scripts/gen_raw.py
DIR=$(mktemp -d)
python3 "$GEN" "$DIR/band.raw" 128 64 10 1 band >/dev/null
"$B" encode -i "$DIR/band.raw" -o "$DIR/on.tmg1"  --size 128x64 --fps 30 --prediction true  >/dev/null 2>&1
"$B" encode -i "$DIR/band.raw" -o "$DIR/off.tmg1" --size 128x64 --fps 30 --prediction false >/dev/null 2>&1
"$B" decode -i "$DIR/on.tmg1" -o "$DIR/back.raw" >/dev/null 2>&1
on=$(wc -c < "$DIR/on.tmg1")
off=$(wc -c < "$DIR/off.tmg1")
echo "prediction ON : $on bytes"
echo "prediction OFF: $off bytes"
if cmp -s "$DIR/band.raw" "$DIR/back.raw"; then echo "band roundtrip OK"; else echo "band MISMATCH"; fi
rm -rf "$DIR"
