use std::env;



fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();

    println!("cargo:rustc-link-search={}\\lib\\win64-dynamic\\lib", root);
}
