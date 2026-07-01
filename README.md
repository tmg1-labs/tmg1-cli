# tmg1-cli

**English** | [日本語](README.ja.md)

A Rust command-line tool for the **TMG1** 1-bit-per-pixel (bitplane) video
format. It can `encode`, `transcode`, `decode`, and inspect (`info`) `.tmg1`
files, and is designed to slot into ffmpeg pipelines via stdin/stdout.

`tmg1-cli` is a thin front-end over the shared C++ codec
[`tmg1-codec`](https://github.com/tmg1-labs/tmg1-codec), which it calls through
the C FFI. The codec is vendored as a submodule and compiled by `build.rs` at
build time.

## Features

- **Four subcommands** — `encode` (raw 1bpp → TMG1), `transcode` (any media →
  TMG1 via ffmpeg), `decode` (TMG1 → raw 1bpp), and `info` (print metadata).
- **Full codec surface** — Range or Rice coder, prediction filters, delta (P)
  frames, scene-change detection (SCD), variable frame rate (VFR), and an
  optional TMGX frame index, all exposed as flags.
- **Pipe-friendly** — `-i -` reads stdin and `-o -` writes stdout (both are the
  default), so commands compose with ffmpeg and shell pipelines.
- **`transcode` wraps ffmpeg** — scales any input to `monow` (1bpp, MSB-first)
  rawvideo and streams it straight into the encoder.

## Install / Build

```bash
git clone --recursive https://github.com/tmg1-labs/tmg1-cli
cd tmg1-cli
cargo build --release   # build.rs compiles the vendored tmg1-codec C++ via the cc crate
# binary: target/release/tmg1
```

Requirements:

- A Rust toolchain and a C++ compiler (GCC / Clang / MSVC).
- On MSVC, `build.rs` passes `/utf-8` automatically (the codec sources contain
  Japanese comments).
- `transcode` additionally requires `ffmpeg` on `PATH`.

If you forgot `--recursive`, fetch the codec submodule with
`git submodule update --init --recursive`.

To install `tmg1` onto your `PATH` (`~/.cargo/bin`) instead of just building it
into `target/release/`, run `cargo install --path .` from the cloned repo.

## Usage

```
tmg1 <encode|transcode|decode|info> [options]
```

For every subcommand, `-i/--input` and `-o/--output` default to `-`
(stdin / stdout).

### encode — raw 1bpp bitplanes → TMG1

Input is a stream of raw frames; each frame is `ceil(width/8) * height` bytes
of 1bpp bit-packed pixels.

```bash
tmg1 encode --size 128x64 --fps 30 -i frames.raw -o out.tmg1
```

| Flag | Default | Description |
|------|---------|-------------|
| `-i, --input` | `-` | Input file (`-` = stdin) |
| `-o, --output` | `-` | Output file (`-` = stdout) |
| `--size WxH` | *(required)* | Frame size, e.g. `128x64` |
| `--fps` | *(required)* | Frame rate (fps) |
| `--key-int` | `60` | Keyframe interval |
| `--coder` | `rice` | Entropy coder: `rice` or `range` |
| `--msb-first` | `true` | MSB-first bit order (`false` = LSB-first) |
| `--delta` | `true` | Enable delta (P) frames |
| `--prediction` | `true` | Try prediction filters (None/Left/Up), keep smallest |
| `--rice-mode` | `per-line` | Rice K mode: `fixed` / `per-line` / `per-frame` (Rice only) |
| `--rice-k` | `1` | Fixed K for `--rice-mode fixed` (0..7) |
| `--scd` | `true` | Scene-change detection: compress P as both I and P, keep smaller |
| `--vfr` | `true` | Variable frame rate: coalesce identical frames into `ptsDelta` |
| `--index` | `false` | Append a TMGX frame index chunk at EOF |
| `--invert` | `false` | Invert input bits before encoding (for black/white polarity flip) |

### transcode — any media → TMG1 (via ffmpeg)

Wraps ffmpeg to scale/convert the input to `monow` rawvideo and encode it. Takes
the same flags as `encode` except: `--delta` is always on, and there is no
`--msb-first` (ffmpeg `monow` is fixed MSB-first).

```bash
tmg1 transcode -i input.mp4 --size 128x64 --fps 30 -o out.tmg1
```

### decode — TMG1 → raw 1bpp bitplanes

Emits raw frames. VFR streams are expanded back to constant frame rate (repeated
frames are re-emitted).

```bash
tmg1 decode -i out.tmg1 -o frames.raw
```

### info — print metadata

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

### Pipeline examples

```bash
# Video → TMG1 (transcode bundles ffmpeg)
tmg1 transcode -i input.mp4 --size 128x64 --fps 30 -o out.tmg1

# Manual ffmpeg → encode pipe (equivalent to transcode)
ffmpeg -i input.mp4 -vf scale=128:64 -r 30 -f rawvideo -pix_fmt monow - \
  | tmg1 encode --size 128x64 --fps 30 -o out.tmg1

# Inspect / decode
tmg1 info -i out.tmg1
tmg1 decode -i out.tmg1 -o frames.raw
```

## How it works

- `src/ffi.rs` declares the codec's `extern "C"` surface
  (`tmg1_encoder_*` / `tmg1_decoder_*`); `src/main.rs` drives it.
- I/O is abstracted through the codec's `Tmg1Stream` callbacks, which here wrap
  files, stdin, and stdout (and ffmpeg's piped stdout for `transcode`).
- A frame is `ceil(width/8) * height` bytes of a 1bpp bitplane.

## Build & CI

CI runs on GitHub Actions (`.github/workflows/ci.yml`, `ubuntu-latest`) and runs
`cargo build --release` and `cargo test`, then uploads the `tmg1` binary as an
artifact. Submodules are fetched recursively (`build.rs` compiles the bundled
codec via `cc`).

## TMG1 Format

The authoritative byte-level layout of `.tmg1` lives in the standalone
[**TMG1 format specification**](https://github.com/tmg1-labs/.github/blob/main/docs/tmg1-format.md).
See [`tmg1-codec`](https://github.com/tmg1-labs/tmg1-codec) for the codec
internals and the C++/FFI API.

## Related projects

Part of **[TMG1 Labs](https://github.com/tmg1-labs)** — see the organization
profile for all repositories in the project.

## License

MIT
