fn main() {
    // Tell Cargo to re-run this build script if the src/ directory changes.
    // File watching for `frame build --watch` is handled at runtime via the
    // `notify` crate in src/cli, not here.
    println!("cargo:rerun-if-changed=src/");
}
