#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--crawde-hook") {
        // Observational hooks never block Claude when Crawde is offline.
        let _ = crawde_lib::forward_hook();
        return;
    }
    crawde_lib::run()
}
