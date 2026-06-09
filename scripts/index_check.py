#!/usr/bin/env python3
# TMG1 索引チャンク(TMGX)の整合性を検証する。
# フレーム列を逐次パースして各フレーム開始オフセットを求め、
# 末尾の TMGX チャンク (Count + uint64 オフセット列) と突き合わせる。
# 使い方: index_check.py <file.tmg1>
import sys

def read_uleb128(buf, pos):
    val = 0
    shift = 0
    while True:
        b = buf[pos]; pos += 1
        val |= (b & 0x7F) << shift
        if (b & 0x80) == 0:
            break
        shift += 7
    return val, pos

def main():
    path = sys.argv[1]
    with open(path, "rb") as f:
        buf = f.read()

    assert buf[0:4] == b"TMG1", "bad file signature"
    pos = 16  # FileHeader は16バイト固定

    walked = []  # 逐次パースで得たフレーム開始オフセット
    while pos < len(buf):
        ftype = buf[pos]
        if ftype not in (0, 1):
            break  # TMGX マジック等 → フレーム列終端
        walked.append(pos)
        pos += 1
        _pts, pos = read_uleb128(buf, pos)
        psize, pos = read_uleb128(buf, pos)
        pos += 2  # frameFlags + predictionMethod
        pos += psize  # payload

    # ここで pos は TMGX チャンク先頭を指しているはず
    assert buf[pos:pos+4] == b"TMGX", f"TMGX magic not found at offset {pos}"
    pos += 4
    count, pos = read_uleb128(buf, pos)

    offsets = []
    for _ in range(count):
        off = int.from_bytes(buf[pos:pos+8], "little"); pos += 8
        offsets.append(off)

    # 検証
    assert pos == len(buf), f"trailing bytes after index: pos={pos} len={len(buf)}"
    assert count == len(walked), f"count mismatch: index={count} walked={len(walked)}"
    assert offsets == walked, f"offset list mismatch\nindex ={offsets}\nwalked={walked}"
    for off in offsets:
        assert buf[off] in (0, 1), f"offset {off} does not point to a frame header (byte={buf[off]})"

    print(f"INDEX OK: {count} frames, offsets verified (first={offsets[0]}, last={offsets[-1]})")

if __name__ == "__main__":
    main()
