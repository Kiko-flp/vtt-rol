// src-tauri/src/lib.rs
mod tablero_general;

use std::sync::Mutex;
use tauri::Manager;

use tablero_general::dados::tirar_dado;
use tablero_general::ficha::{
    actualizar_entidad, colocar_entidad, crear_entidad, modificar_vida, mover_instancia,
    obtener_banco_entidades, obtener_instancias_activas, retirar_instancia,
};
use tablero_general::partida::{
    compactar_partida, iniciar_autoguardado, // <-- Importamos ambas
    obtener_historial_personal, obtener_historial_tiradas, obtener_mapas_zona, obtener_zonas,
    actualizar_imagen_mapa, actualizar_rejilla_mapa, borrar_mapa, crear_mapa, crear_zona,
    eliminar_zona, renombrar_mapa, renombrar_zona, reordenar_mapa, reordenar_zona,
    AppState, EstadoPartida,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let estado_inicial = EstadoPartida::cargar();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState(Mutex::new(estado_inicial)))
        // INICIAMOS EL TEMPORIZADOR DE 10 MINUTOS EN SEGUNDO PLANO
        .setup(|app| {
            iniciar_autoguardado(app.handle().clone());
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let state = window.state::<AppState>();
                compactar_partida(&state);
            }
        })
        .invoke_handler(tauri::generate_handler![
            tirar_dado,
            obtener_zonas,
            obtener_mapas_zona,
            obtener_banco_entidades,
            obtener_instancias_activas,
            obtener_historial_tiradas,
            obtener_historial_personal,
            crear_entidad,
            actualizar_entidad,
            colocar_entidad,
            mover_instancia,
            modificar_vida,
            retirar_instancia,
            crear_zona,
            eliminar_zona,
            renombrar_zona,
            reordenar_zona,
            crear_mapa,
            renombrar_mapa,
            reordenar_mapa,
            actualizar_imagen_mapa,
            actualizar_rejilla_mapa,
            borrar_mapa,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}