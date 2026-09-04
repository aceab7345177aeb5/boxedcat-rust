fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/core/mod.rs");
    println!("cargo:rerun-if-changed=src/services/mod.rs");
}
