// build.rs

fn main() {
    let src_path = "bcg729/src";
    let include_path = "bcg729/include";

    cc::Build::new()
        .files(
            glob::glob(&format!("{}/*.c", src_path))
                .expect("Failed to read glob pattern")
                .filter_map(Result::ok)
        )
        .include(include_path)
        
        // BU TEK SATIR, Lİnker HATASINI ÖNLEYECEKTİR.
        .define("BCG729_STATIC", None)

        .compile("g729");

    println!("cargo:rerun-if-changed={}", src_path);
    println!("cargo:rerun-if-changed={}", include_path);
}