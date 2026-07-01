# セッション履歴・決定経緯

> 各セッションの作業内容と決定事項のログ。新しいセッションは `session-record` スキルで追記する。
>
> codec本体の設計・アーキテクチャ決定（Range コーダv2化など）は `tmg1-codec` の
> `session-history.md` を参照。ここは本リポジトリ（Rust CLI）側の変更が中心。

---

### 2026-06-09 セッション（tmg1-cli パリティ: rice-mode + rice-k、本リポジトリ側）

#### 作業内容
tmg1-cli を dotnet版CLIと機能パリティにする計画の2個目。Riceパラメータモード
(Fixed/PerLine/PerFrame) と rice-k を全層 (C++ → C API → Rust) に実装。
codec本体側の変更は `tmg1-codec` の session-history.md 参照。

#### 完了したこと
| ファイル | 変更 |
|---|---|
| `src/ffi.rs` | `Tmg1EncodeConfig` に `rice_mode`/`rice_k` フィールド追加 |
| `src/main.rs` | `--rice-mode`(既定 per-line、値: fixed/per-line/per-frame) / `--rice-k`(0..7)
  を `EncodeArgs`/`TranscodeArgs` に追加 |

- 検証: 全モード × k=0..7 で encode→decode バイナリ一致 (rice/range)。
- コミット: `58b1597`（codec側は `b9e445d`、push未実施）。

#### 次回セッションで取り組む内容
パリティ3個目「scd (scene change detection)」 — encoderのみ。P-frame時にI/P両方を
圧縮し小さい方を採用する。

---

### 2026-07-01 セッション（cargo install対応 + release.yml新規 + v0.2.0リリース）

#### 作業内容
`tmg1-cli` を cargo でインストール可能にする対応と v0.2.0 リリースの公開。

#### 完了したこと
- README(英/日) に `cargo install --path .` でのローカルインストール手順を追記（commit `989de7e`）。
- `.github/workflows/release.yml` を新規作成（commit `7344b96`）:
  タグ push(`v*`) → `create-release`（`gh release create --generate-notes`で即時公開）→
  4ターゲット（linux-x86_64 / linux-aarch64 / macos-aarch64 / windows-x86_64）を
  各OSネイティブランナーでビルドし `gh release upload --clobber` で添付。
  クロスコンパイル・サードパーティreleaseアクション不使用。
- `Cargo.toml` を `0.1.0` → `0.2.0` に更新（commit `b8f6df6`）。
- タグ `v0.2.0` を push しワークフロー実走、4ジョブとも一発green。GitHub Releaseに
  4バイナリ添付済み。

#### 決定事項
| 決定 | 内容 | 理由 |
|---|---|---|
| crates.io公開 | **今回見送り（未着手）** | submodule構成のままでは `cargo package` が
  codecソースを含められない。vendor化/別クレート分離が必要で、`tmg1-codec` の
  「分岐コピーを増やさない」方針と衝突するため要合意 |
| linux-aarch64ビルド方式 | `ubuntu-24.04-arm` ネイティブランナー採用 | クロスコンパイル不要、
  `build.rs`のcc(C++)コンパイルもそのまま通る |

#### 注意点
- ダウンロード方法（Release経由）のREADME追記はユーザー判断で今回見送り。
