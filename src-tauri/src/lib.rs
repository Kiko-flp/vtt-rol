// Declarar existencias de los módulos (uno por cada carpeta, concretamente mod.rs)
mod tablero_general;
use tablero_general::dados::tirar_dado; // <-- La importas arriba
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            tirar_dado,
            ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
