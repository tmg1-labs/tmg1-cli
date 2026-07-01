# コーディング規約

実際の `src/main.rs` / `src/ffi.rs` から観察された現行の規約。

## 命名規則
- 関数・変数: snake_case（例: `cmd_encode`, `encode_stream`, `parse_size`）
- 型・enum: PascalCase（例: `EncodeArgs`, `RiceMode`, `Tmg1EncodeConfig`）
- CLIサブコマンド/フラグ: `clap` の kebab-case 自動変換に従う（`--rice-mode`, `--key-int` 等）。
  dotnet版CLIとのオプション名パリティを優先する。

## CLI構造（clap）
- `#[derive(Parser)]` / `#[derive(Subcommand)]` でサブコマンドを定義（`Encode`/`Transcode`/
  `Decode`/`Info`）。
- bool系オプションは `value_parser = clap::builder::BoolishValueParser::new(), num_args = 1`
  で `--flag true/false` 形式にし、既定値は `default_value = "true"` のように文字列で指定する
  （dotnet版の `--flag <bool>` 形式に合わせるための意図的な選択）。
- 数値範囲チェックは `value_parser = clap::value_parser!(u8).range(0..=7)` のように clap 側で行う。

## エラーハンドリング
- ライブラリ関数として `Result` を伝播させるのではなく、CLIコマンド関数内で
  `eprintln!("tmg1: ...")` → `std::process::exit(1)` する即時終了パターンを一貫して使う
  （`unwrap_or_else` のクロージャ内でこのパターンを使うのが定型）。
- FFI呼び出し（`unsafe { tmg1_xxx(...) }`）の戻り値チェックも同じパターン。

## FFI / unsafe
- FFI境界の関数は `unsafe extern "C" fn` として `src/ffi.rs` に定義し、ポインタ経由の
  コンテキスト（`FileCtx`/`WriteCtx`）を `Box<dyn Read>`/`Box<dyn Write>` でラップする。
- 呼び出し側（`main.rs`）は `unsafe { ... }` ブロックを個々の呼び出しに最小スコープで付ける。

## コメント
- ソースコードには**日本語のコメント**を記入する（コーデック本体と同じ方針）。
- 非自明な意図（例: バッファリング/デッドロック回避のためのスレッド分離、EOF検出の理由）は
  コメントで明示する。

## フォーマッタ
- `rustfmt` の既定設定に準拠（プロジェクト固有の `rustfmt.toml` は現状なし）。
