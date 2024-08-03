// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod plugin;

use plugin::PluginManager;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
fn get_html_from_markdown(
    state: State<Mutex<PluginManager>>,
    input: &str,
) -> Result<String, String> {
    let mut output = vec![String::new()];

    let mut plugin_manager = state
        .inner()
        .lock()
        // This error should also be handled by the combined error log.
        .map_err(|e| format!("Mutex error:\n{}", e))?;

    for line in input.split('\n') {
        let mut line = line.to_owned();
        plugin_manager.execute_line_functions(&mut line);
        output.push(line);
    }

    Ok(output.join("\n"))
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(PluginManager::new()))
        .invoke_handler(tauri::generate_handler![get_html_from_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
