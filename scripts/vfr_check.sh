#!/usr/bin/env bash
# VFR検証: 重複フレーム素材で encode(vfr on/off)→decode を行い、
#   1) vfr on でラウンドトリップが元データとバイナリ一致するか (展開復元の正しさ)
#   2) vfr on のファイルが off より小さいか (スキップが実際に効いているか)
#   3) decode 後のフレーム数が入力と一致するか (タイムライン復元)
# を確認する。
set -u
B=/mnt/d/workspace/TsuMuGi/tmg1-cli/target/release/tmg1
GEN=/mnt/d/workspace/TsuMuGi/tmg1-cli/scripts/gen_raw.py
W=64; H=32; N=40
D=$(mktemp -d)
rc=0
for seed in 1 2 3; do
  python3 "$GEN" "$D/in.raw" "$W" "$H" "$N" "$seed" dup >/dev/null
  insz=$(stat -c%s "$D/in.raw")
  for C in rice range; do
    "$B" encode -i "$D/in.raw" -o "$D/on.tmg1"  --size ${W}x${H} --fps 30 --coder "$C" --vfr true  >/dev/null 2>&1
    "$B" encode -i "$D/in.raw" -o "$D/off.tmg1" --size ${W}x${H} --fps 30 --coder "$C" --vfr false >/dev/null 2>&1
    "$B" decode -i "$D/on.tmg1" -o "$D/on.raw" >/dev/null 2>&1
    onsz=$(stat -c%s "$D/on.tmg1"); offsz=$(stat -c%s "$D/off.tmg1"); ondec=$(stat -c%s "$D/on.raw")
    rt=$(cmp -s "$D/in.raw" "$D/on.raw" && echo OK || echo MISMATCH)
    smaller=$([ "$onsz" -lt "$offsz" ] && echo yes || echo no)
    fmatch=$([ "$ondec" -eq "$insz" ] && echo yes || echo no)
    echo "seed=$seed [$C] roundtrip=$rt  vfr<off=$smaller (on=${onsz}B off=${offsz}B)  size_match=$fmatch (in=${insz} dec=${ondec})"
    [ "$rt" = OK ] && [ "$smaller" = yes ] && [ "$fmatch" = yes ] || rc=1
  done
done
rm -rf "$D"
[ $rc -eq 0 ] && echo "ALL VFR CHECKS PASSED" || echo "SOME VFR CHECKS FAILED"
exit $rc
