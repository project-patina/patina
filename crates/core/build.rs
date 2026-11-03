//! The integrated Fabric mod is built from `java-mod/` and stays out of Git, so the
//! launcher also builds without it. The release scripts refuse to package such a build.

use std::path::Path;

fn main() {
    let jar = Path::new("../../resources/patina-mod/patina-client-26.3.jar");
    println!("cargo::rerun-if-changed={}", jar.display());
    println!("cargo::rustc-check-cfg=cfg(patina_mod_jar)");
    if jar.is_file() {
        println!("cargo::rustc-cfg=patina_mod_jar");
    }
}
