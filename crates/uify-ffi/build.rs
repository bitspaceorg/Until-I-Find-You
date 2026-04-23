use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_header: PathBuf = PathBuf::from(&crate_dir)
        .join("../../include/uify.h")
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&crate_dir).join("../../include/uify.h"));

    // Best-effort: do not fail the build if cbindgen hits a transient error —
    // header generation is a developer-time convenience. CI runs `make header`
    // explicitly to assert the header is up to date.
    if let Ok(bindings) = cbindgen::generate(&crate_dir) {
        let _ = std::fs::create_dir_all(out_header.parent().unwrap());
        bindings.write_to_file(&out_header);
    }

    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
