use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=data/io.github.pomato.Pomato.gschema.xml");

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("schemas");
    std::fs::create_dir_all(&out).expect("create schema out dir");
    std::fs::copy(
        "data/io.github.pomato.Pomato.gschema.xml",
        out.join("io.github.pomato.Pomato.gschema.xml"),
    )
    .expect("copy gschema");

    let status = Command::new("glib-compile-schemas")
        .arg(&out)
        .status()
        .expect("glib-compile-schemas must be available");
    assert!(status.success(), "glib-compile-schemas failed");

    println!("cargo:rustc-env=POMATO_SCHEMA_DIR={}", out.display());
}
