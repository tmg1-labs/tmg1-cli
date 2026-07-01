# TMG1 CLI (tmg1-cli)

@.claude/architecture.md
@.claude/coding-style.md
@.claude/workflows.md
@.claude/context/current-sprint.md
@.claude/context/known-issues.md

## Quick facts
- 言語: Rust (edition 2021)
- バイナリ名: `tmg1`（`Cargo.toml` の `[[bin]]`）
- コーデック: `tmg1-codec` を git submodule として取り込み、`build.rs`（`cc` クレート）で
  C++ をコンパイルし `src/ffi.rs` の手書きバインディング経由で呼び出す。
- サブコマンド: `encode` / `transcode`（ffmpegラッパー） / `decode` / `info`
- テスト: `cargo test`、加えて `scripts/roundtrip.sh` による encode→decode バイナリ一致検証
- CI: GitHub Actions（`.github/workflows/ci.yml`: `build_test`、`.github/workflows/release.yml`: リリース）
- 現状: dotnet版CLIとの機能パリティはほぼ完了（prediction/rice-mode・rice-k/scd/vfr/index/info+transcode）。

## 関連リポジトリ
- `tmg1-codec`: コーデック本体（本リポジトリはこれを submodule + FFI で利用。分岐コピーはしない）。
- `tmg1-esp32-demo`: ESP32実機デモ（`tmg1-codec` を PlatformIO `lib_deps` で取得する側）。
- `tmg1-labs.github`: 組織プロフィール・仕様書の正本リポジトリ。

## Claudeへの指示
- 方針の決定や修正に関する意図や経緯があれば記録していくこと。
- `tmg1-codec` の C API（`c_api/tmg1_c.h`）を変更した場合は `src/ffi.rs` の
  `Tmg1EncodeConfig` 等を同期させること（`tmg1-codec` 側の coding-style.md にも同ルールあり）。
- セッションの記録は `session-record` スキルを使う。
- 長期の決定経緯・セッション履歴は `.claude/context/session-history.md`、過去のエラー解決は
  `.claude/context/errors-log.md` を参照。
