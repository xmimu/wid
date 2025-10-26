mod modules;

use modules::{
    validate_wwise_directory,
    validate_bank_directory,
    search_wwise_project,
    search_bank_directory,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validate_wwise_directory, 
            validate_bank_directory,
            search_wwise_project,
            search_bank_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
