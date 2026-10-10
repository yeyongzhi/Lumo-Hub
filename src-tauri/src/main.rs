// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let directory = std::path::PathBuf::from(local).join("Lumo-Hub");
            if std::fs::create_dir_all(&directory).is_ok() {
                let details = format!(
                    "{panic}\n{}",
                    std::backtrace::Backtrace::force_capture()
                );
                let _ = std::fs::write(directory.join("startup-error.log"), details);
            }
        }
        original_hook(panic);
    }));
    lumo_hub_lib::run()
}
