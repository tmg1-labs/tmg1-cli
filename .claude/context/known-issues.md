# 既知の問題・注意事項

> 過去のエラー解決の原文は `errors-log.md`。ここはハマりやすい地雷の要点を集約する。

## パッケージング・配布

### `cargo publish`（crates.io 公開）は submodule 依存構成のままでは不可（2026-07-01 判明）
- **症状/懸念**: `cargo package`/`cargo publish` は git 管理下ファイル（`git ls-files` 相当）のみ梱包し、submodule
  （`tmg1-codec/`）の中身は含まれない。`build.rs` が参照する codec の C++ ソースが欠落しビルド不能になる。
- **回避策（未着手）**: codec ソースを `tmg1-cli` 側へ vendor コピーするか、`tmg1-codec-sys` 的な別クレートへ分離して
  そちらに vendor する。`tmg1-codec` 側の方針「codecの分岐コピーを増やさない」（architecture.md）と衝突するため、
  進めるなら方針側の判断も必要。
- **現状動く範囲**: `cargo install --path .`（ローカルクローン、submodule init 済み前提）は動作確認済み。
  `cargo install --git <url>`（cargoがgit依存のsubmoduleも取得する想定）は未検証。crates.io 公開のみが不可。

### submodule の remote が古いホストのまま残る問題
- **症状**: `tmg1-codec` submodule 配下で `git fetch origin` しても目的のコミットが取れず
  `fatal: unable to read tree <sha>` / `Not a valid object name`。
- **原因**: submodule のローカル clone の `origin` が**旧ホスト(gitlab)のまま**だった。superproject の
  `.gitmodules` を相対 URL(`../tmg1-codec.git`)に変えても、既存 submodule の `.git/config` remote は自動追随しない。
- **回避策**: `git submodule sync tmg1-codec` で `.gitmodules` の URL を submodule の remote へ反映してから fetch。
- **注**: `tmg1-esp32-demo` は既に submodule 自体を撤去し `lib_deps`(git タグ)運用に移行済みのため、この問題は
  esp32-demo側では無関係化した。本リポジトリは今も submodule を使うため引き続き注意が必要。

## CI

### GitHub Actions: Linux ARM64 ネイティブホストランナーが利用可能（2026-07-01、release.yml で採用）
- **内容**: `ubuntu-24.04-arm` / `ubuntu-22.04-arm` ラベルで、クロスコンパイル無しに aarch64 をネイティブビルドできる
  GitHub提供ホストランナーが存在する（2025-01時点でPublic Preview、GitHub公式ブログで確認）。パブリックリポジトリなら無料、
  プライベートリポジトリでは使えない。プレビュー期間中はピーク時にキュー待ちが長くなる可能性がある。
- **用途**: `.github/workflows/release.yml` で `linux-aarch64` ターゲットに採用。`build.rs` の `cc` クレートに
  よるC++コンパイルもクロスツールチェーン不要でそのまま通った（初回実走で確認済み）。

### GitHub Actions: Node20 deprecation 警告（2026-07-01 対応）
- **症状**: ワークフロー実行の annotation に「Node.js 20 is deprecated ... forced to run on Node.js 24」。
- **回避策**: `actions/upload-artifact` は **v4/v5 は node20**、**v6 以降が node24**。本リポジトリは v4→**v7** に
  更新済み（commit `dc77383`、単一ファイル upload の挙動は不変）。`Swatinem/rust-cache@v2`（現行v2.9.1）・
  `actions/checkout@v5` は既に node24。
- **他ホスト/自前ランナー注意**: upload-artifact v6+ は Actions Runner 2.327.1 以上が必要。GitHub ホストの
  `ubuntu-latest` は常に最新なので問題なし。self-hosted のときだけ要確認。
- **落とし穴（アクションごとに Node24 化のメジャーが違う）**: 「最新メジャー = Node24」とは限らない。
  `runs.using` を実確認すること（`gh api repos/<o>/<r>/contents/action.yml?ref=<tag> --jq .content | base64 -d | grep using`）。

## 開発環境

### WSL を `bash -lc '...'` でインライン実行すると変数展開が消える
- **症状**: シングルクオート内の `$VAR`/`$(...)` が空になり、`scripts/roundtrip.sh` 系の検証が
  全件 MISMATCH になる（コマンドは成立し気づきにくい）。
- **回避策**: `.sh` ファイル経由で実行する（`wsl bash -lc 'bash /mnt/d/.../roundtrip.sh'`）。
  `/mnt` パスは `-lc` 文字列の**中**に置くこと（引数に直接渡すと Git Bash がパス変換する）。
