fn main() {
    // the page itself is packed into the program
    println!("cargo:rerun-if-changed=ui");
    // the documentation pages and the API manifest are packed into the program
    println!("cargo:rerun-if-changed=../../docs");
    println!("cargo:rerun-if-changed=../../crates/openomsi-plugin-sdk/api.json");
    tauri_build::build()
}
