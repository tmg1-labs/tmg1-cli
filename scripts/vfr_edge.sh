#!/usr/bin/env bash
# VFR エッジケース: 全同一 / 単一 / 末尾連続重複 / 中間重複 を
# encode(vfr on)→decode し、フレーム数とバイナリ一致を確認する。
set -u
B=/mnt/d/workspace/TsuMuGi/tmg1-cli/target/release/tmg1
W=8; H=8; FB=8   # 8x8=64bit=8byte/frame
D=$(mktemp -d)
pass=0; fail=0

mkframe() { python3 -c "import sys; sys.stdout.buffer.write(bytes([$1])*$FB)"; }

run_case() { # name expected_frames raw_path
  local name=$1 exp=$2 raw=$3
  "$B" encode -i "$raw" -o "$D/t.tmg1" --size ${W}x${H} --fps 30 --vfr true >/dev/null 2>&1
  "$B" decode -i "$D/t.tmg1" -o "$D/t.raw" >/dev/null 2>&1
  local n=$(( $(stat -c%s "$D/t.raw") / FB ))
  local rt; cmp -s "$raw" "$D/t.raw" && rt=OK || rt=MISMATCH
  if [ "$n" -eq "$exp" ] && [ "$rt" = OK ]; then
    echo "OK   $name: frames=$n roundtrip=$rt"; pass=$((pass+1))
  else
    echo "FAIL $name: frames=$n (expected $exp) roundtrip=$rt"; fail=$((fail+1))
  fi
}

# 全同一5フレーム (末尾フラッシュのみで全期間を表現)
{ mkframe 0xAA; mkframe 0xAA; mkframe 0xAA; mkframe 0xAA; mkframe 0xAA; } > "$D/c1.raw"
run_case "all-same-5" 5 "$D/c1.raw"
# 単一フレーム (フラッシュ無し)
mkframe 0xF0 > "$D/c2.raw"
run_case "single" 1 "$D/c2.raw"
# 末尾連続重複 A B B B B
{ mkframe 0x11; mkframe 0x22; mkframe 0x22; mkframe 0x22; mkframe 0x22; } > "$D/c3.raw"
run_case "trailing-dup" 5 "$D/c3.raw"
# 中間重複 A A B C C C
{ mkframe 0x01; mkframe 0x01; mkframe 0x02; mkframe 0x03; mkframe 0x03; mkframe 0x03; } > "$D/c4.raw"
run_case "mid-dup" 6 "$D/c4.raw"

echo "pass=$pass fail=$fail"
rm -rf "$D"
[ $fail -eq 0 ] && exit 0 || exit 1
