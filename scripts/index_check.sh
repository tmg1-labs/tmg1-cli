#!/usr/bin/env bash
# 索引チャンク(TMGX)の総合検証:
#  - encode --index → decode が raw とバイナリ一致 (デコーダが TMGX で停止)
#  - 索引内容 (Count / 各オフセットが実フレーム先頭) が逐次パースと一致 (index_check.py)
# rice/range × 各素材(delta/dup/unique) × vfr on/off を網羅する。
set -u
W=128; H=64; N=40
B=/mnt/d/workspace/TsuMuGi/tmg1-cli/target/release/tmg1
GEN=/mnt/d/workspace/TsuMuGi/tmg1-cli/scripts/gen_raw.py
CHK=/mnt/d/workspace/TsuMuGi/tmg1-cli/scripts/index_check.py
DIR=$(mktemp -d)
rc=0

for MODE in delta dup unique; do
  python3 "$GEN" "$DIR/in.raw" "$W" "$H" "$N" 42 "$MODE" >/dev/null
  for C in rice range; do
    for VFR in on off; do
      OUT="$DIR/out.tmg1"; DEC="$DIR/dec.raw"
      "$B" encode -i "$DIR/in.raw" -o "$OUT" --size ${W}x${H} --fps 30 \
        --coder "$C" --vfr "$VFR" --index >/dev/null 2>&1
      "$B" decode -i "$OUT" -o "$DEC" >/dev/null 2>&1
      tag="mode=$MODE coder=$C vfr=$VFR"
      if ! cmp -s "$DIR/in.raw" "$DEC"; then
        echo "[FAIL roundtrip] $tag"; rc=1; continue
      fi
      if out=$(python3 "$CHK" "$OUT" 2>&1); then
        echo "[OK] $tag :: $out"
      else
        echo "[FAIL index] $tag :: $out"; rc=1
      fi
    done
  done
done

# エッジ: 全フレーム同一 → I + finish()テールP の2チャンクのみ。
# finish() のテールフレームが索引に記録される経路を明示的に確認する。
python3 -c "open('$DIR/in.raw','wb').write(bytes($W*$H//8*$N))"
"$B" encode -i "$DIR/in.raw" -o "$DIR/out.tmg1" --size ${W}x${H} --fps 30 \
  --coder rice --vfr on --index >/dev/null 2>&1
"$B" decode -i "$DIR/out.tmg1" -o "$DIR/dec.raw" >/dev/null 2>&1
if cmp -s "$DIR/in.raw" "$DIR/dec.raw" && out=$(python3 "$CHK" "$DIR/out.tmg1" 2>&1); then
  echo "[OK] all-same (vfr tail) :: $out"
else
  echo "[FAIL] all-same (vfr tail)"; rc=1
fi

rm -rf "$DIR"
[ $rc -eq 0 ] && echo "ALL INDEX CHECKS PASSED"
exit $rc
