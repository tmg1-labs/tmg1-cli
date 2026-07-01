# よく使うコマンド・手順

## セットアップ
```bash
git submodule update --init tmg1-codec
```

## ビルド・テスト
```bash
cargo build --release          # target/release/tmg1
cargo test
```

## インストール
```bash
cargo install --path .         # ~/.cargo/bin/tmg1 に配置
```
- `cargo install --git <url>` は未検証。**crates.io への `cargo publish` は現状不可**
  （known-issues.md参照）。

## ラウンドトリップ検証
```bash
bash scripts/roundtrip.sh      # encode→decode バイナリ一致を全モード×パラメータで検証
```
- WSL越しに実行する場合はインライン `bash -lc '...'` ではなく `.sh` ファイル経由で実行すること
  （known-issues.md参照）。

## リリース
- タグ push（`v*`）で `.github/workflows/release.yml` が起動。
- `create-release` ジョブが `gh release create --generate-notes` で即時公開のリリースを作成 →
  4ターゲット（`linux-x86_64` / `linux-aarch64` / `macos-aarch64` / `windows-x86_64`）を
  各OSネイティブランナーでビルドし `gh release upload --clobber` で添付。
  クロスコンパイル不要、サードパーティのrelease系アクションも不使用（`gh` CLI標準搭載のみ）。
- `linux-aarch64` は `ubuntu-24.04-arm`（GitHub提供ネイティブARM64ホストランナー）を使用。
- リリース前に `Cargo.toml` の `version` を上げてコミットしておくこと。

## codec の更新を取り込む
```bash
git submodule update --remote tmg1-codec   # 最新コミットへ更新（またはタグ指定でcheckout）
git add tmg1-codec
git commit -m "chore: bump tmg1-codec submodule"
```
- `tmg1-codec` の C API（`c_api/tmg1_c.h`）が変わった場合は `src/ffi.rs` も同時に更新する。

## CI
- `.github/workflows/ci.yml`（`build_test`）: `actions/checkout`(submodules: recursive) →
  `Swatinem/rust-cache@v2` → `cargo build --release` → `cargo test` → リリースバイナリを
  `actions/upload-artifact`(7日) で保存。
- 実行条件: `push`（`main` / `feature/**`）と `pull_request`。
