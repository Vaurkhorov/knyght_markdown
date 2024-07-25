// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command(rename_all = "snake_case")]
fn get_html_from_markdown(input: &str) -> String {
    let mut output = String::new();
    for line in input.split('\n') {
        if line.chars().nth(0) == Some('#') {
            let mut heading_type=5;
            for i in 0..5 {
                if line.chars().nth(i) != Some('#') {
                    heading_type = i-1;
                    break;
                }
            }
            heading_type += 1;


            output.push_str(format!(
                "<h{}>{}</h{}>",
                heading_type,
                line[heading_type..].to_string(),
                heading_type
            ).as_str());
            continue;
        } else {
            output.push_str(format!("<p>{}</p>", line).as_str())
        }
    }

    output
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_html_from_markdown])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
