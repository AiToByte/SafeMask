fn main() {
    println!("cargo:rerun-if-env-changed=SAFEMASK_HMAC_KEY");
    tauri_build::build()
}
