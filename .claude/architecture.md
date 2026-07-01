# アーキテクチャ方針

## 全体構成

TMG1 動画フォーマットの Rust 製 CLI。コアアルゴリズムは持たず、`tmg1-codec`（C++ライブラリ、
別リポジトリ）を FFI 経由で呼び出す薄いラッパー + ffmpeg 連携。

```
tmg1-cli (このリポジトリ)
├── src/main.rs        ... clapによるCLI定義、encode/transcode/decode/infoの各コマンド実装
├── src/ffi.rs          ... tmg1_c.h / io.h の手書きFFIバインディング
├── build.rs            ... cc クレートで tmg1-codec の C++ ソースをコンパイル・リンク
├── scripts/            ... roundtrip.sh(検証) / gen_raw.py / size_check.sh
└── tmg1-codec/          ... コーデック本体 (git submodule)
```

## FFI境界

- `src/ffi.rs` は `tmg1-codec/c_api/tmg1_c.h` に対応する `extern "C"` 宣言を手書きで維持する
  （bindgen 等の自動生成は使っていない）。
- `Tmg1Stream`（read/write コールバック）を Rust 側の `FileCtx`/`WriteCtx` から
  `unsafe extern "C" fn` でラップし、ファイル/stdin/stdoutを透過的に扱う。
- `Tmg1EncodeConfig` のフィールドは `tmg1-codec` の `EncodeConfig`（C++側）と1対1で対応させる。
  コーデック側でフィールドを追加・変更した場合は必ずこちらも更新する。

## submodule で取り込む理由・制約

- `build.rs` が `cc` クレートで C++ ソースを直接コンパイルするため、ソースツリーが
  必要（PlatformIOの`lib_deps`のようなバイナリ配布の仕組みがない）。よって submodule で
  ソースごと取り込む。
- **crates.io への `cargo publish` は現状不可**: `cargo package` は git 管理下ファイル
  （`git ls-files` 相当）のみ梱包し、submodule の中身は含まれない。回避するには
  codec ソースの vendor化 か `tmg1-codec-sys` 的な別クレートへの分離が必要（未着手）。
  `tmg1-codec` 側の「分岐コピーを増やさない」方針と衝突するため、進める場合は
  両リポジトリで方針をすり合わせること。

## 禁止パターン

- `tmg1-codec` のソースをこのリポジトリに直接コピー・分岐させない（submodule 経由のみ）。
- dotnet版CLIとの機能パリティを崩す独自仕様を無断で追加しない（オプション追加時はdotnet版の
  挙動を確認し、フォーマット仕様（`tmg1-labs.github`）と矛盾しないようにする）。
