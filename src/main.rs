mod ffi;

use clap::{Parser, Subcommand, ValueEnum};
use std::fs::File;
use std::io::{self, Read, Write};
use std::os::raw::{c_int, c_long, c_uchar, c_void};
use std::path::PathBuf;

use ffi::*;

// ---------------------------------------------------------------------------
// CLIの定義
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "tmg1", about = "TMG1 video codec CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// rawビットプレーンデータをTMG1にエンコード
    Encode(EncodeArgs),
    /// TMG1ファイルをrawビットプレーンデータにデコード
    Decode(DecodeArgs),
    /// TMG1ファイルのメタデータを表示
    Info(InfoArgs),
}

#[derive(ValueEnum, Clone)]
enum Coder {
    Rice,
    Range,
}

#[derive(ValueEnum, Clone)]
enum RiceMode {
    /// 固定K (--rice-k の値を使用。内部的には per-frame 機構で表現)
    Fixed,
    /// 行ごとに最適K (3bit/行)
    PerLine,
    /// フレームごとに最適K (3bit/フレーム)
    PerFrame,
}

impl RiceMode {
    fn as_u8(&self) -> u8 {
        match self {
            RiceMode::Fixed    => 0,
            RiceMode::PerLine  => 1,
            RiceMode::PerFrame => 2,
        }
    }
}

#[derive(Parser)]
struct EncodeArgs {
    /// 入力ファイル（- で stdin）
    #[arg(short, long, default_value = "-")]
    input: String,

    /// 出力ファイル（- で stdout）
    #[arg(short, long, default_value = "-")]
    output: String,

    /// フレームサイズ（例: 128x64）
    #[arg(long)]
    size: String,

    /// フレームレート（fps）
    #[arg(long)]
    fps: u16,

    /// キーフレーム間隔
    #[arg(long, default_value_t = 60)]
    key_int: u16,

    /// エントロピー符号化器
    #[arg(long, value_enum, default_value = "rice")]
    coder: Coder,

    /// MSBファースト（false でLSBファースト）
    #[arg(long, default_value = "true", value_parser = clap::builder::BoolishValueParser::new(), num_args = 1)]
    msb_first: bool,

    /// 差分圧縮（Pフレーム）を有効化
    #[arg(long, default_value = "true", value_parser = clap::builder::BoolishValueParser::new(), num_args = 1)]
    delta: bool,

    /// 予測フィルタ（None/Left/Up を試行し最小を選択）を有効化
    #[arg(long, default_value = "true", value_parser = clap::builder::BoolishValueParser::new(), num_args = 1)]
    prediction: bool,

    /// Riceパラメータの決定モード（Riceコーダ使用時のみ）
    #[arg(long, value_enum, default_value = "per-line")]
    rice_mode: RiceMode,

    /// Fixedモードで使用するRice-k値（0..7）
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(0..=7))]
    rice_k: u8,

    /// シーンチェンジ検出（Pフレームを I/P 両方で圧縮し小さい方を採用）を有効化
    #[arg(long, default_value = "true", value_parser = clap::builder::BoolishValueParser::new(), num_args = 1)]
    scd: bool,

    /// 可変フレームレート（前フレームと同一フレームを書かず ptsDelta に集約）を有効化
    #[arg(long, default_value = "true", value_parser = clap::builder::BoolishValueParser::new(), num_args = 1)]
    vfr: bool,

    /// ファイル末尾にフレーム索引チャンク（TMGX）を付加する
    #[arg(long, default_value_t = false)]
    index: bool,
}

#[derive(Parser)]
struct DecodeArgs {
    /// 入力ファイル（- で stdin）
    #[arg(short, long, default_value = "-")]
    input: String,

    /// 出力ファイル（- で stdout）
    #[arg(short, long, default_value = "-")]
    output: String,
}

#[derive(Parser)]
struct InfoArgs {
    /// 入力ファイル（- で stdin）
    #[arg(short, long, default_value = "-")]
    input: String,
}

// ---------------------------------------------------------------------------
// ファイル/stdinをTmg1Streamとして扱うためのコンテキスト
// ---------------------------------------------------------------------------

struct FileCtx {
    file: Box<dyn Read>,
}

unsafe extern "C" fn file_read(ctx: *mut c_void, buf: *mut c_uchar, len: usize) -> c_int {
    let fc = &mut *(ctx as *mut FileCtx);
    let slice = std::slice::from_raw_parts_mut(buf, len);
    match fc.file.read(slice) {
        Ok(n) => n as c_int,
        Err(_) => -1,
    }
}

struct WriteCtx {
    file: Box<dyn Write>,
}

unsafe extern "C" fn file_write(ctx: *mut c_void, buf: *const c_uchar, len: usize) -> c_int {
    let wc = &mut *(ctx as *mut WriteCtx);
    let slice = std::slice::from_raw_parts(buf, len);
    match wc.file.write_all(slice) {
        Ok(_) => len as c_int,
        Err(_) => -1,
    }
}

fn open_read(path: &str) -> Box<dyn Read> {
    if path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(path).unwrap_or_else(|e| {
            eprintln!("tmg1: 入力ファイルを開けませんでした: {path}: {e}");
            std::process::exit(1);
        }))
    }
}

fn open_write(path: &str) -> Box<dyn Write> {
    if path == "-" {
        Box::new(io::stdout())
    } else {
        Box::new(File::create(path).unwrap_or_else(|e| {
            eprintln!("tmg1: 出力ファイルを開けませんでした: {path}: {e}");
            std::process::exit(1);
        }))
    }
}

fn parse_size(size: &str) -> (u16, u16) {
    let parts: Vec<&str> = size.split('x').collect();
    if parts.len() != 2 {
        eprintln!("tmg1: --size の形式が正しくありません（例: 128x64）");
        std::process::exit(1);
    }
    let w: u16 = parts[0].parse().unwrap_or_else(|_| {
        eprintln!("tmg1: 幅の値が不正です: {}", parts[0]);
        std::process::exit(1);
    });
    let h: u16 = parts[1].parse().unwrap_or_else(|_| {
        eprintln!("tmg1: 高さの値が不正です: {}", parts[1]);
        std::process::exit(1);
    });
    (w, h)
}

// ---------------------------------------------------------------------------
// encode
// ---------------------------------------------------------------------------

fn cmd_encode(args: EncodeArgs) {
    let (width, height) = parse_size(&args.size);
    // rawビットプレーン: 1ピクセル1ビット、1行 = width/8 バイト
    let frame_bytes = ((width as usize + 7) / 8) * height as usize;

    let mut read_ctx = FileCtx { file: open_read(&args.input) };
    let mut write_ctx = WriteCtx { file: open_write(&args.output) };

    let mut in_stream = Tmg1Stream {
        ctx:   &mut read_ctx as *mut FileCtx as *mut c_void,
        read:  Some(file_read),
        write: None,
        tell:  None,
        seek:  None,
    };
    let mut out_stream = Tmg1Stream {
        ctx:   &mut write_ctx as *mut WriteCtx as *mut c_void,
        read:  None,
        write: Some(file_write),
        tell:  None,
        seek:  None,
    };

    let config = Tmg1EncodeConfig {
        width,
        height,
        timebase_num:    1,
        timebase_den:    args.fps,
        key_interval:    args.key_int,
        msb_first:       args.msb_first as u8,
        use_range_coder: matches!(args.coder, Coder::Range) as u8,
        delta_enabled:   args.delta as u8,
        prediction_enabled: args.prediction as u8,
        rice_mode:       args.rice_mode.as_u8(),
        rice_k:          args.rice_k,
        scene_change_enabled: args.scd as u8,
        vfr_enabled:     args.vfr as u8,
        index_enabled:   args.index as u8,
    };

    let enc = unsafe { tmg1_encoder_create(&mut out_stream, &config) };
    if enc.is_null() {
        eprintln!("tmg1: エンコーダの初期化に失敗しました");
        std::process::exit(1);
    }

    let mut buf = vec![0u8; frame_bytes];
    let mut total_frames = 0usize;
    let mut total_bytes = 0usize;

    loop {
        // フレームを1枚分読み込む
        let n = unsafe {
            let ctx = &mut read_ctx;
            let slice = std::slice::from_raw_parts_mut(buf.as_mut_ptr(), frame_bytes);
            read_exact_or_eof(&mut *ctx.file, slice)
        };
        match n {
            Ok(0) => break, // EOF
            Ok(_) => {}
            Err(e) => {
                eprintln!("tmg1: 読み込みエラー: {e}");
                unsafe { tmg1_encoder_destroy(enc) };
                std::process::exit(1);
            }
        }

        let ret = unsafe { tmg1_encoder_encode_frame(enc, buf.as_ptr(), frame_bytes) };
        if ret != 0 {
            eprintln!("tmg1: エンコードエラー: {ret}");
            unsafe { tmg1_encoder_destroy(enc) };
            std::process::exit(1);
        }
        total_frames += 1;
        total_bytes += frame_bytes;
    }

    let ret = unsafe { tmg1_encoder_finish(enc) };
    unsafe { tmg1_encoder_destroy(enc) };

    if ret != 0 {
        eprintln!("tmg1: finish エラー: {ret}");
        std::process::exit(1);
    }

    eprintln!("エンコード完了: {total_frames} フレーム, 入力 {total_bytes} バイト");
}

// フレーム分ちょうど読むか EOF を検出する
fn read_exact_or_eof(r: &mut dyn Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut total = 0;
    while total < buf.len() {
        match r.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => total += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
    Ok(total)
}

// ---------------------------------------------------------------------------
// decode
// ---------------------------------------------------------------------------

fn cmd_decode(args: DecodeArgs) {
    let mut read_ctx = FileCtx { file: open_read(&args.input) };
    let mut write_ctx = WriteCtx { file: open_write(&args.output) };

    let mut in_stream = Tmg1Stream {
        ctx:   &mut read_ctx as *mut FileCtx as *mut c_void,
        read:  Some(file_read),
        write: None,
        tell:  None,
        seek:  None,
    };

    let dec = unsafe { tmg1_decoder_create(&mut in_stream) };
    if dec.is_null() {
        eprintln!("tmg1: デコーダの初期化に失敗しました（ファイルが壊れているか形式が不正）");
        std::process::exit(1);
    }

    let width  = unsafe { tmg1_decoder_width(dec) } as usize;
    let height = unsafe { tmg1_decoder_height(dec) } as usize;
    let num    = unsafe { tmg1_decoder_timebase_num(dec) } as u32;
    let den    = unsafe { tmg1_decoder_timebase_den(dec) } as u32;
    let frame_bytes = ((width + 7) / 8) * height;

    eprintln!("TMG1: {width}x{height} @ {den}/{num} fps");

    let mut buf = vec![0u8; frame_bytes];
    let mut prev = vec![0u8; frame_bytes];
    let mut has_prev = false;
    let mut total_frames = 0usize;

    // 書き込みヘルパ（エラー時は終了）
    let write_frame = |wc: &mut WriteCtx, data: &[u8]| {
        wc.file.write_all(data).unwrap_or_else(|e| {
            eprintln!("tmg1: 書き込みエラー: {e}");
            std::process::exit(1);
        });
    };

    loop {
        let ret = unsafe { tmg1_decoder_decode_frame(dec, buf.as_mut_ptr(), frame_bytes) };
        if ret != 0 {
            // エラーコード -1 は通常 EOF
            break;
        }

        // VFR(可変フレームレート)の復元: ptsDelta が 1 より大きい場合、
        // 前フレームが (ptsDelta - 1) 回続いたことを意味するため、その分だけ繰り返してから
        // 現フレームを書き込む (dotnet版デコーダと同じ挙動。CFR では全て ptsDelta=1 で無変化)。
        let pts = unsafe { tmg1_decoder_last_pts_delta(dec) };
        if has_prev && pts > 1 {
            for _ in 0..(pts - 1) {
                write_frame(&mut write_ctx, &prev);
                total_frames += 1;
            }
        }

        write_frame(&mut write_ctx, &buf);
        total_frames += 1;

        prev.copy_from_slice(&buf);
        has_prev = true;
    }

    unsafe { tmg1_decoder_destroy(dec) };
    eprintln!("デコード完了: {total_frames} フレーム");
}

// ---------------------------------------------------------------------------
// info
// ---------------------------------------------------------------------------

fn cmd_info(args: InfoArgs) {
    let mut read_ctx = FileCtx { file: open_read(&args.input) };

    let mut in_stream = Tmg1Stream {
        ctx:   &mut read_ctx as *mut FileCtx as *mut c_void,
        read:  Some(file_read),
        write: None,
        tell:  None,
        seek:  None,
    };

    let dec = unsafe { tmg1_decoder_create(&mut in_stream) };
    if dec.is_null() {
        eprintln!("tmg1: ファイルの読み込みに失敗しました");
        std::process::exit(1);
    }

    let width  = unsafe { tmg1_decoder_width(dec) };
    let height = unsafe { tmg1_decoder_height(dec) };
    let num    = unsafe { tmg1_decoder_timebase_num(dec) };
    let den    = unsafe { tmg1_decoder_timebase_den(dec) };

    println!("Size:      {width}x{height}");
    println!("Framerate: {den}/{num}");

    unsafe { tmg1_decoder_destroy(dec) };
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Encode(args) => cmd_encode(args),
        Commands::Decode(args) => cmd_decode(args),
        Commands::Info(args)   => cmd_info(args),
    }
}
