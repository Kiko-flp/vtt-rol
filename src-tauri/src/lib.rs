// src-tauri/src/lib.rs
mod tablero_general;
use tablero_general::modelos::*;
use tablero_general::dados::tirar_dado;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            tirar_dado,
            obtener_zonas,
            obtener_mapas_zona,
            obtener_banco_entidades,
            obtener_instancias_activas,
            obtener_historial_tiradas
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}