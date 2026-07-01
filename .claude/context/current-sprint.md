# 現在の作業コンテキスト

最終更新: 2026-07-01（`cargo install`対応 + `release.yml`新規 + v0.2.0リリース公開済み）

## 今やっていること
- **`tmg1-cli` を cargo でインストール可能にする対応 + v0.2.0 リリース**（2026-07-01。
  **push・タグpush済み、Release公開済み**）。
  - 背景: 従来の README 手順（`cargo build --release`）は `target/release/tmg1` を作るだけで、
    PATH配置や配布バイナリの提供手段が無かった。
  - README(英/日): 「`cargo install --path .` で `~/.cargo/bin` へ配置できる」旨を追記
    （commit `989de7e`）。ダウンロード方法（Release経由）の追記はユーザー判断で**今回見送り**。
  - **crates.io への `cargo publish` は不可と判明**（未着手・保留、詳細known-issues.md）。
  - **リリースワークフロー新規作成**: `.github/workflows/release.yml`（commit `7344b96`）。
    タグ push(`v*`) → `create-release` ジョブで `gh release create --generate-notes` により
    **即時公開**のリリースを作成 → 4ターゲット（`linux-x86_64` / `linux-aarch64` /
    `macos-aarch64` / `windows-x86_64`）を各OSネイティブランナーでビルドし
    `gh release upload --clobber` で添付。クロスコンパイル不要、サードパーティのrelease系
    アクションも不使用（`gh` CLI標準搭載のみ）。
    - `linux-aarch64` は `ubuntu-24.04-arm`（GitHub提供のネイティブARM64ホストランナー）を採用
      （詳細は known-issues.md）。
  - `Cargo.toml` を `0.1.0`→`0.2.0` に更新（commit `b8f6df6`）。
  - **検証**: タグ `v0.2.0` を push しワークフロー実走、4ジョブとも一発 green。
    4バイナリが GitHub Release に添付済み。

## 今やっていること（過去分）
- **CI を GitLab CI → GitHub Actions へ移行**（2026-07-01、`.github/workflows/ci.yml`新規、
  `.gitlab-ci.yml`削除）。
  - `build_test` 1ジョブに統合（GitLab は build/test 別ジョブだったが、両者とも submodule+g++
    必要で test は build 成果物依存のため統合）。`actions/checkout`(submodules: recursive) →
    `Swatinem/rust-cache@v2` → `cargo build --release` → `cargo test` → リリースバイナリ `tmg1` を
    `actions/upload-artifact`(7日)。`build.rs` が `cc` で codec C++ をコンパイルするため
    submodule必須（相対URL submodule `../tmg1-codec.git` は GitHub上で `tmg1-labs/tmg1-codec.git`
    に解決される前提。よって公開順序は codec → cli/esp32-demo、詳細は `tmg1-labs.github` 参照）。
  - **GitHub Actions Node20警告対応**: `upload-artifact` v4→**v7**に更新（commit `dc77383`、
    単一ファイルupload の挙動不変）。`rust-cache@v2`(=v2.9.1)・`checkout@v5` は既にNode24。
- **dotnet版CLIとの機能パリティ**（ほぼ完了）: prediction / rice-mode・rice-k / scd / vfr /
  index / info+transcode を実装済み。各機能追加の経緯は session-history.md 参照。
- `transcode` は ffmpeg 未導入のため e2e 未検証（実装は完了）。

## 一時的な制約・注意事項
- crates.io 公開は未対応（cargo install --path . のみサポート）。

## 次にやること
- 特になし（v0.2.0リリースまでの作業は完了）。crates.io公開を進める場合は
  vendor化/別クレート分離の方針を `tmg1-codec` 側と合意してから着手する。

## 参考
- 決定経緯・セッション履歴の詳細は `session-history.md`、過去のエラー解決は `errors-log.md`。
