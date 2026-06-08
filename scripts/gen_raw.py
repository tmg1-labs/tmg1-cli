#!/usr/bin/env python3
"""rawビットプレーン (1bpp) テストフレームを生成するヘルパー。
使い方: gen_raw.py <out.raw> <W> <H> <frames> [seed] [mode]
mode: delta(デフォルト, キーフレーム+小差分) / unique(全フレーム独立乱数) / dup(重複フレーム多め)
"""
import os, sys, random

def main():
    out = sys.argv[1]
    w = int(sys.argv[2]); h = int(sys.argv[3]); n = int(sys.argv[4])
    seed = int(sys.argv[5]) if len(sys.argv) > 5 else 42
    mode = sys.argv[6] if len(sys.argv) > 6 else "delta"
    random.seed(seed)
    bpl = (w + 7) // 8
    frame = bpl * h
    buf = bytearray()
    cur = bytearray(random.getrandbits(8) for _ in range(frame))
    for f in range(n):
        if f == 0:
            pass
        elif mode == "unique":
            cur = bytearray(random.getrandbits(8) for _ in range(frame))
        elif mode == "dup":
            if random.random() < 0.5:
                pass  # 直前と同一フレーム
            else:
                for _ in range(5):
                    cur[random.randrange(frame)] ^= random.randint(1, 255)
        elif mode == "band":
            # 横帯パターン: 各行が同一値 → UP予測で激減する
            cur = bytearray()
            for y in range(h):
                v = 0xFF if ((y // 4 + f) % 2) == 0 else 0x00
                cur += bytes([v]) * bpl
        else:  # delta
            for _ in range(3):
                cur[random.randrange(frame)] ^= random.randint(1, 255)
        buf += cur
    with open(out, "wb") as fp:
        fp.write(buf)
    print(f"wrote {out}: {len(buf)} bytes, {len(buf)//frame} frames ({w}x{h}, mode={mode})")

if __name__ == "__main__":
    main()
