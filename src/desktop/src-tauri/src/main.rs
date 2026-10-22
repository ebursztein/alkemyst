#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    if alkemyst_desktop::run().is_err() {
        eprintln!("Alkemyst could not start. Check that its local data directory is writable.");
        std::process::exit(1);
    }
}
