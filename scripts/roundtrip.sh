#!/usr/bin/env bash
# encode→decode のラウンドトリップがバイナリ一致するか検証する。
# 使い方: roundtrip.sh <W> <H> <frames> [extra encode args...]
set -u
W=$1; H=$2; N=$3; shift 3
EXTRA="$*"
DIR=$(mktemp -d)
B=/mnt/d/workspace/TsuMuGi/tmg1-cli/target/release/tmg1
GEN=/mnt/d/workspace/TsuMuGi/tmg1-cli/scripts/gen_raw.py
python3 "$GEN" "$DIR/in.raw" "$W" "$H" "$N" 42 delta >/dev/null
rc=0
for C in rice range; do
    "$B" encode -i "$DIR/in.raw" -o "$DIR/out.tmg1" --size ${W}x${H} --fps 30 --coder "$C" $EXTRA >/dev/null 2>&1
    "$B" decode -i "$DIR/out.tmg1" -o "$DIR/dec.raw" >/dev/null 2>&1
    if cmp -s "$DIR/in.raw" "$DIR/dec.raw"; then
        echo "[$C] ROUNDTRIP OK  (args: --coder $C $EXTRA)"
    else
        echo "[$C] MISMATCH      (args: --coder $C $EXTRA)"
        rc=1
    fi
done
rm -rf "$DIR"
exit $rc
