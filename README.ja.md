# tmg1-cli

[English](README.md) | **日本語**

**TMG1**（1ピクセル1ビット＝ビットプレーン）動画フォーマット向けの Rust 製
コマンドラインツールです。`.tmg1` ファイルの `encode`・`transcode`・`decode`・
情報表示（`info`）を行い、stdin/stdout 経由で ffmpeg パイプラインに組み込めます。

`tmg1-cli` は共有 C++ コーデック
[`tmg1-codec`](https://github.com/tmg1-labs/tmg1-codec) を C FFI 経由で呼び出す
薄いフロントエンドです。codec はサブモジュールとして同梱され、ビルド時に
`build.rs` がコンパイルします。

## 特長

- **4 つのサブコマンド** — `encode`（raw 1bpp → TMG1）、`transcode`（任意メディア
  → TMG1、ffmpeg 経由）、`decode`（TMG1 → raw 1bpp）、`info`（メタデータ表示）。
- **コーデック機能をフル露出** — Range / Rice コーダ、予測フィルタ、デルタ（P）
  フレーム、シーンチェンジ検出（SCD）、可変フレームレート（VFR）、任意の TMGX
  フレーム索引を、すべてフラグで制御できます。
- **パイプ親和性** — `-i -` で stdin、`-o -` で stdout（いずれも既定値）。ffmpeg や
  シェルパイプラインとそのまま連結できます。
- **`transcode` は ffmpeg をラップ** — 入力を `monow`（1bpp・MSBファースト）
  rawvideo へスケール変換し、そのままエンコーダへ流し込みます。

## インストール / ビルド

```bash
git clone --recursive https://github.com/tmg1-labs/tmg1-cli
cd tmg1-cli
cargo build --release   # build.rs が同梱 tmg1-codec の C++ を cc クレートでコンパイル
# 生成物: target/release/tmg1
```

必要なもの:

- Rust ツールチェインと C++ コンパイラ（GCC / Clang / MSVC）。
- MSVC では `build.rs` が自動で `/utf-8` を付与します（codec ソースに日本語コメントが
  含まれるため）。
- `transcode` は別途 `ffmpeg` が `PATH` に必要です。

`--recursive` を付け忘れた場合は
`git submodule update --init --recursive` で codec サブモジュールを取得します。

## 使い方

```
tmg1 <encode|transcode|decode|info> [options]
```

全サブコマンドで `-i/--input`・`-o/--output` の既定値は `-`（stdin / stdout）です。

### encode — raw 1bpp ビットプレーン → TMG1

入力は raw フレームの連続で、各フレームは `ceil(width/8) * height` バイトの
1bpp ビットパックピクセルです。

```bash
tmg1 encode --size 128x64 --fps 30 -i frames.raw -o out.tmg1
```

| フラグ | 既定値 | 説明 |
|--------|--------|------|
| `-i, --input` | `-` | 入力ファイル（`-` = stdin） |
| `-o, --output` | `-` | 出力ファイル（`-` = stdout） |
| `--size WxH` | *（必須）* | フレームサイズ（例: `128x64`） |
| `--fps` | *（必須）* | フレームレート（fps） |
| `--key-int` | `60` | キーフレーム間隔 |
| `--coder` | `rice` | エントロピー符号化器: `rice` または `range` |
| `--msb-first` | `true` | MSBファーストのビット順（`false` で LSBファースト） |
| `--delta` | `true` | デルタ（P）フレームを有効化 |
| `--prediction` | `true` | 予測フィルタ（None/Left/Up）を試行し最小を採用 |
| `--rice-mode` | `per-line` | Rice K モード: `fixed` / `per-line` / `per-frame`（Rice 時のみ） |
| `--rice-k` | `1` | `--rice-mode fixed` で使う固定 K（0..7） |
| `--scd` | `true` | シーンチェンジ検出: P を I/P 両方で圧縮し小さい方を採用 |
| `--vfr` | `true` | 可変フレームレート: 同一フレームを `ptsDelta` に集約 |
| `--index` | `false` | 末尾に TMGX フレーム索引チャンクを付加 |

### transcode — 任意メディア → TMG1（ffmpeg 経由）

ffmpeg をラップし、入力を `monow` rawvideo へ変換してエンコードします。フラグは
`encode` とほぼ同じですが、`--delta` は常時有効で、`--msb-first` はありません
（ffmpeg `monow` が MSBファースト固定のため）。

```bash
tmg1 transcode -i input.mp4 --size 128x64 --fps 30 -o out.tmg1
```

### decode — TMG1 → raw 1bpp ビットプレーン

raw フレームを出力します。VFR ストリームは定フレームレート（CFR）へ展開されます
（重複フレームを再出力）。

```bash
tmg1 decode -i out.tmg1 -o frames.raw
```

### info — メタデータ表示

```bash
tmg1 info -i out.tmg1
```

```
--- TMG1 File Info ---
  Version:     2
  Size:        128x64
  Framerate:   30/1 (30.00 fps)
  KeyInterval: 60
  MSB First:   true
  Coder:       Range
------------------------
```

### パイプライン例

```bash
# 動画 → TMG1（transcode は ffmpeg を同梱呼び出し）
tmg1 transcode -i input.mp4 --size 128x64 --fps 30 -o out.tmg1

# 手動 ffmpeg → encode のパイプ（transcode と等価）
ffmpeg -i input.mp4 -vf scale=128:64 -r 30 -f rawvideo -pix_fmt monow - \
  | tmg1 encode --size 128x64 --fps 30 -o out.tmg1

# 情報表示 / デコード
tmg1 info -i out.tmg1
tmg1 decode -i out.tmg1 -o frames.raw
```

## 仕組み

- `src/ffi.rs` が codec の `extern "C"` 面（`tmg1_encoder_*` / `tmg1_decoder_*`）を
  宣言し、`src/main.rs` がそれを駆動します。
- I/O は codec の `Tmg1Stream` コールバックで抽象化され、ここではファイル・stdin・
  stdout（`transcode` では ffmpeg のパイプ標準出力）をラップします。
- フレームは `ceil(width/8) * height` バイトの 1bpp ビットプレーンです。

## ビルドと CI

CI は GitHub Actions（`.github/workflows/ci.yml`、`ubuntu-latest`）で `cargo build
--release` と `cargo test` を実行し、`tmg1` バイナリを artifact として保存します。
サブモジュールは recursive で取得します（`build.rs` が同梱 codec を `cc` でコンパイル）。

## TMG1 フォーマット

`.tmg1` のバイト単位の正確なレイアウトは、独立した
[**TMG1 フォーマット仕様書**](https://github.com/tmg1-labs/.github/blob/main/docs/tmg1-format.ja.md)
に置きます。コーデック内部や C++/FFI API は
[`tmg1-codec`](https://github.com/tmg1-labs/tmg1-codec) を参照してください。

## 関連プロジェクト

**[TMG1 Labs](https://github.com/tmg1-labs)** の一部です。プロジェクトの全リポジトリ
一覧は組織プロフィールを参照してください。

## ライセンス

MIT
