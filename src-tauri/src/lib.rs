// src-tauri/src/lib.rs
mod tablero_general;

use tablero_general::partida::{
    obtener_banco_entidades, obtener_historial_tiradas, obtener_instancias_activas,
    obtener_mapas_zona, obtener_zonas, AppState, EstadoPartida,
    obtener_historial_personal // <-- Solo necesitamos este
};
use tablero_general::dados::tirar_dado;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let estado_inicial = EstadoPartida::cargar();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(estado_inicial)))
        .invoke_handler(tauri::generate_handler![
            tirar_dado,
            obtener_zonas,
            obtener_mapas_zona,
            obtener_banco_entidades,
            obtener_instancias_activas,
            obtener_historial_tiradas,
            obtener_historial_personal // <-- Solo lectura
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}