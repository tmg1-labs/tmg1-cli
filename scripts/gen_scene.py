#!/usr/bin/env python3
# シーンチェンジに富むテスト素材を生成する。
# 各「シーン」は構造的(=ラン長圧縮が効く)だが、シーン間のXOR差分は大きい。
# → SCD有効時はシーン切替フレームをIフレーム化して小さくできるはず。
import sys

path, W, H, frames = sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), int(sys.argv[4])
bpl = (W + 7) // 8
out = bytearray()
for f in range(frames):
    scene = f // 4  # 4フレームごとにシーン切替
    for y in range(H):
        row = bytearray(bpl)
        for x in range(W):
            # シーンごとに異なる単純な縞模様(構造的)
            if scene % 3 == 0:
                bit = 1 if (x // 8) % 2 == 0 else 0
            elif scene % 3 == 1:
                bit = 1 if (y // 8) % 2 == 0 else 0
            else:
                bit = 1 if ((x + y) // 8) % 2 == 0 else 0
            if bit:
                row[x // 8] |= (1 << (7 - x % 8))  # MSBファースト
        out += row
with open(path, 'wb') as fp:
    fp.write(out)
