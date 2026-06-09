fn main() {
    let src_dir = "tmg1-codec/src";
    let capi_dir = "tmg1-codec/c_api";
    let include_dir = "tmg1-codec/include";

    let srcs = [
        "range_encoder.cpp",
        "range_decoder.cpp",
        "rice_reader.cpp",
        "rice_writer.cpp",
        "prediction.cpp",
        "encoder.cpp",
        "decoder.cpp",
    ];

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++14")
        .include(include_dir)
        .file(format!("{capi_dir}/tmg1_c.cpp"));

    // MSVC は既定でソースをシステムコードページ(日本語環境では932/Shift-JIS)として読むため、
    // UTF-8 の日本語コメントが文字化けして構文エラーになる。/utf-8 で UTF-8 と明示する。
    // (GCC/Clang は既定で UTF-8 のため不要)
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        build.flag("/utf-8");
    }

    for src in &srcs {
        build.file(format!("{src_dir}/{src}"));
    }

    build.compile("tmg1codec");

    // サブモジュール内のファイルが変更されたら再ビルド
    println!("cargo:rerun-if-changed=tmg1-codec/src");
    println!("cargo:rerun-if-changed=tmg1-codec/c_api");
    println!("cargo:rerun-if-changed=tmg1-codec/include");
}
