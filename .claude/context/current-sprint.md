# 現在の作業コンテキスト

最終更新: 2026-07-03（monob版の実機OLED動作確認が完了。`tmg1-esp32-demo`側README/コメントも更新済み）

## 今やっていること
- **`transcode` の ffmpeg pix_fmt を `monow`→`monob` に変更（白黒反転の根本原因を修正）**
  （2026-07-01、commit `ce4a9c8`、push済み。**2026-07-03に実機OLEDでの動作確認完了**）。
  - **発端**: `--invert` 実装後、ユーザーから「monow は bit=1が黒(インク)の紙媒体規約。
    発光ディスプレイなら黒=背景と考えるのが普通では、monobが正解では」と指摘。
  - **根本原因を実機コードで確認**: `tmg1-esp32-demo/src/main.cpp:92-97` のコメントに
    「`drawXBMP` はbit=1を描画色(点灯)として扱う」と明記されていた。`monow`(bit=1=黒)と
    `drawXBMP`(bit=1=点灯)の規約が逆だったのが、実機で全面反転して見えていた**根本原因**。
    これまでの対処（`howto_video_to_tmg1.md`の手動XOR、および直前に追加した`--invert`）は
    いずれも対症療法だった。
  - **実証**: ffmpeg で同一画像を`monow`/`monob`両方に変換し、全バイトが`XOR 0xFF`の関係
    （ビット単位で完全に逆）であることを実機コマンドで確認済み。
  - **変更箇所**: `src/main.rs`の`cmd_transcode`（ffmpeg引数 `-pix_fmt monow`→`monob`、
    コメント更新）。README(英/日)の該当箇所（説明文・手動ffmpegパイプ例）も`monob`に更新。
  - **`--invert`は撤去せず維持**: 既存rawの高速な極性切替・ffmpeg以外の入力への対応という
    別用途で引き続き有用なため。
  - **検証**: `cargo build --release`成功。ffmpegで同一画像をmonow/monob変換しビット反転
    関係を確認 → `tmg1 transcode`(pix_fmt monob版)→decodeの結果が、ffmpeg単体の
    `-pix_fmt monob`出力と完全一致することを確認済み。
  - **未対応**: `howto_video_to_tmg1.md`（tmg1-esp32-demo側memory）のffmpegパイプライン例は
    まだ`monow`+手動XOR前提のまま。実機で新しい`monob`版の動作を確認したら、そちらの手順・
    memoryも更新が必要。
  - **リリース方針**: ユーザー判断で「即コミット・push・v0.3.1タグ」を選択（実機検証済みの
    ビット反転関係の実証で十分と判断。実機OLEDでの最終表示確認は別途）。
    `Cargo.toml`を`0.3.0`→`0.3.1`に更新。
  - **実機OLED最終確認（2026-07-03）**: monob直接エンコードでの実機表示動作を確認済み。
    `tmg1-esp32-demo`側の`README.md`/`README.ja.md`/`src/main.cpp`のコメントも
    「ffmpeg negate前提」から「`-pix_fmt monob`直接エンコード前提」の記述へ更新済み。

## 今やっていること（過去分）
- **encode/transcode に `--invert`（エンコード前ビット反転）オプションを追加 + v0.3.0 リリース**
  （2026-07-01、**push・タグpush済み、Release公開済み**）。
  - 背景: 実機OLEDの白黒極性反転は、これまで `tmg1-esp32-demo` の
    `howto_video_to_tmg1.md` に記録された手順どおり「monow raw を Python で
    `XOR 0xFF` して再エンコード」という手動前処理で対応していた。この手動ステップを
    CLI 組込みのオプションにしてワンステップ化。
  - **過去の別件との違いに注意**: dotnet参照実装にあった `InvertBits`（デコーダ側で
    戻す前提のヘッダフラグ案）は目的不明のためオミット決定済み（別件、
    tmg1-esp32-demo側memory参照）。今回はそれとは別物で、**入力バイト列をエンコード直前に
    反転するだけの単純な前処理**。コーデック(tmg1-codec)・FFI・デコーダ・フォーマットは
    一切無変更。
  - 実装: `src/main.rs` の `EncodeArgs`/`TranscodeArgs` に `--invert`（既定false、
    `--scd`/`--vfr`と同じBoolishValueParserパターン）を追加。共有関数`encode_stream`に
    `invert: bool` 引数を追加し、フレーム読込直後・エンコード直前に全バイトを
    `!byte`（XOR 0xFF）反転。`Tmg1EncodeConfig`（FFI struct）には含めない。
  - 検証: ランダムraw(16x8/5フレーム)で (1) `--invert`なし encode→decodeが元rawと一致
    （既存動作に影響なし）、(2) `--invert true` encode→decodeの結果が元rawの手動XOR 0xFF
    反転と一致、の両方を確認。
  - README(英/日)のオプション表に `--invert` 行を追記。
  - `Cargo.toml` を `0.2.0`→`0.3.0` に更新。commit `a0e529c`、push・タグ`v0.3.0`push済み、
    release.ymlの5ジョブ（create-release + 4プラットフォームbuild）全green、
    4バイナリ（linux-x86_64/linux-aarch64/macos-aarch64/windows-x86_64.exe）添付確認済み。

## 今やっていること（過去分）
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
- crates.io公開を進める場合は vendor化/別クレート分離の方針を `tmg1-codec` 側と合意してから
  着手する。

## 参考
- 決定経緯・セッション履歴の詳細は `session-history.md`、過去のエラー解決は `errors-log.md`。
