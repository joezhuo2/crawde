#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|arg| arg == "--crawde-hook") {
        // Observational hooks never block Claude when Crawde is offline.
        let _ = crawde_lib::forward_hook();
        return;
    }
    if let Some(action) = std::env::args().find(|arg| {
        matches!(
            arg.as_str(),
            "--install-hooks" | "--uninstall-hooks" | "--hook-status"
        )
    }) {
        match crawde_lib::cli(&action) {
            Ok(message) => println!("{message}"),
            Err(error) => {
                eprintln!("Crawde: {error}");
                std::process::exit(1);
            }
        }
        return;
    }
    crawde_lib::run()
}
