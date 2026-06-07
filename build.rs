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

    for src in &srcs {
        build.file(format!("{src_dir}/{src}"));
    }

    build.compile("tmg1codec");

    // サブモジュール内のファイルが変更されたら再ビルド
    println!("cargo:rerun-if-changed=tmg1-codec/src");
    println!("cargo:rerun-if-changed=tmg1-codec/c_api");
    println!("cargo:rerun-if-changed=tmg1-codec/include");
}
