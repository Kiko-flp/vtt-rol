use std::sync::Mutex;
use tauri::{State, AppHandle, Manager};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use crate::tablero_general::modelos::*;

// ========================================================
// 1. DEFINICIÓN DE LOS DELTAS (Eventos de la partida)
// ========================================================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DeltaPartida {
    TiradaRealizada(Tiradas),
    VidaModificada { id_instancia: u16, delta: i16 },
    // En el futuro aquí añadirás:
    // EntidadMovida { id_instancia: u16, x: i16, y: i16 },
    // ZonaCreada(Zona),
}

// ========================================================
// 2. ESTADO GENERAL
// ========================================================
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EstadoPartida {
    #[serde(default)]
    pub zonas: Vec<Zona>,
    #[serde(default)]
    pub mapas: Vec<Mapa>,
    #[serde(default)]
    pub banco_entidades: Vec<Entidad>,
    #[serde(default)]
    pub instancias_activas: Vec<Instancia>,
    #[serde(default)]
    pub historial_tiradas: Vec<Tiradas>,
}

pub struct AppState(pub Mutex<EstadoPartida>);

impl EstadoPartida {
    // Aplica un delta modificando el estado en memoria
    pub fn aplicar_delta(&mut self, delta: &DeltaPartida) {
        match delta {
            DeltaPartida::TiradaRealizada(tirada) => {
                self.historial_tiradas.insert(0, tirada.clone());
                if self.historial_tiradas.len() > 20 {
                    self.historial_tiradas.pop();
                }
            }
            DeltaPartida::VidaModificada { id_instancia, delta } => {
                if let Some(inst) = self.instancias_activas.iter_mut().find(|i| i.id_instancia == *id_instancia) {
                    inst.vida_actual += delta;
                }
            }
        }
    }

    // Carga el estado base (partida.json) y luego reproduce los deltas pendientes
    pub fn cargar() -> Self {
        let mut estado: EstadoPartida = match std::fs::read_to_string("partida.json") {
            Ok(contenido) => serde_json::from_str(&contenido).unwrap_or_default(),
            Err(_) => EstadoPartida::default(),
        };

        // Si existe un archivo de deltas acumulados, los reproducimos en memoria
        if let Ok(lineas) = std::fs::read_to_string("deltas.jsonl") {
            for linea in lineas.lines() {
                if let Ok(delta) = serde_json::from_str::<DeltaPartida>(linea) {
                    estado.aplicar_delta(&delta);
                }
            }
        }

        estado
    }
}

// ========================================================
// 3. PERSISTENCIA RÁPIDA (Solo añade 1 línea en disco)
// ========================================================
async fn persistir_delta(delta: DeltaPartida) {
    if let Ok(mut linea) = serde_json::to_string(&delta) {
        linea.push('\n'); // Formato JSONL (un JSON por cada línea)

        if let Ok(mut archivo) = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("deltas.jsonl")
            .await
        {
            let _ = archivo.write_all(linea.as_bytes()).await;
        }
    }
}

// FUNCIÓN CENTRAL PARA DESPACHAR CUALQUIER CAMBIO
pub fn procesar_delta(state: &State<'_, AppState>, delta: DeltaPartida) {
    // 1. Aplica el cambio en la memoria RAM instantáneamente
    {
        let mut lock = state.0.lock().unwrap();
        lock.aplicar_delta(&delta);
    }

    // 2. FUTURA RED: Aquí es donde enviarás el paquete por websocket/red:
    // enviar_por_red(&delta);

    // 3. Guarda únicamente ese delta al final del archivo en segundo plano
    let delta_clonado = delta.clone();
    tauri::async_runtime::spawn(async move {
        persistir_delta(delta_clonado).await;
    });
}

// Función específica para registrar tiradas (llamada por dados.rs)
pub fn registrar_tirada(state: &State<'_, AppState>, resultado: &str) {
    let (desglose, total_num) = if let Some((izq, der)) = resultado.split_once(" = ") {
        (izq.to_string(), der.trim().parse::<i16>().unwrap_or(0))
    } else {
        (resultado.to_string(), 0)
    };

    let hora_actual = chrono::Local::now().format("%H:%M").to_string();

    let tirada = Tiradas {
        total: total_num,
        comando: desglose,
        hora: hora_actual,
    };

    // Despachamos el evento
    procesar_delta(state, DeltaPartida::TiradaRealizada(tirada));
}

const ARCHIVO_HISTORIAL_PERSONAL: &str = "historial_personal.txt";

// En src-tauri/src/tablero_general/partida.rs

pub fn registrar_comando_personal(comando: &str) {
    let cmd = comando.trim().to_string();
    if cmd.is_empty() {
        return;
    }

    tauri::async_runtime::spawn(async move {
        // 1. Leemos las líneas existentes
        let mut lineas: Vec<String> = if let Ok(contenido) = tokio::fs::read_to_string(ARCHIVO_HISTORIAL_PERSONAL).await {
            contenido
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        // 2. No repetimos si el último comando es exactamente idéntico
        if lineas.last() != Some(&cmd) {
            lineas.push(cmd);
        }

        // 3. LÍMITE DE 100: Si hay más de 100, eliminamos los más viejos del principio
        if lineas.len() > 100 {
            let sobrante = lineas.len() - 100;
            lineas.drain(0..sobrante);
        }

        // 4. Guardamos las 100 líneas actualizadas
        let mut salida = lineas.join("\n");
        salida.push('\n');

        let _ = tokio::fs::write(ARCHIVO_HISTORIAL_PERSONAL, salida).await;
    });
}

// Vuelca la memoria RAM a partida.json y resetea deltas.jsonl
// En src-tauri/src/tablero_general/partida.rs

// COMPACTACIÓN BLINDADA (Usa archivo temporal para evitar corrupción si hay un corte)
pub fn compactar_partida(state: &AppState) {
    let lock = state.0.lock().unwrap();

    if let Ok(json) = serde_json::to_string_pretty(&*lock) {
        // 1. Escribimos en el archivo temporal primero
        if std::fs::write("partida.json.tmp", json).is_ok() {
            // 2. Renombrado atómico: sustituye partida.json de golpe y de forma 100% segura
            if std::fs::rename("partida.json.tmp", "partida.json").is_ok() {
                // 3. Solo cuando partida.json está confirmado a salvo, limpiamos deltas.jsonl
                let _ = std::fs::remove_file("deltas.jsonl");
            }
        }
    }
}

// BUCLE DE AUTOGUARDADO CADA 10 MINUTOS
pub fn iniciar_autoguardado(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // 10 minutos = 600 segundos
        let mut intervalo = tokio::time::interval(std::time::Duration::from_secs(600));
        
        // El primer tick se consume de inmediato al iniciar, lo descartamos
        intervalo.tick().await;

        loop {
            intervalo.tick().await; // Espera 10 minutos de forma no bloqueante
            
            // Si la aplicación sigue abierta y el estado existe, compacta
            if let Some(state) = app_handle.try_state::<AppState>() {
                compactar_partida(&state);
            }
        }
    });
}

// ========================================================
// 4. COMANDOS DE LECTURA (Leen de RAM instantáneamente)
// ========================================================

// Comando opcional por si se implementa botón de guardar partida
#[tauri::command]
pub async fn _guardar_partida(state: State<'_, AppState>) -> Result<(), String> {
    compactar_partida(&state);
    Ok(())
}

#[tauri::command]
pub fn obtener_historial_tiradas(state: State<'_, AppState>) -> Vec<Tiradas> {
    state.0.lock().unwrap().historial_tiradas.clone()
}

#[tauri::command]
pub fn obtener_zonas(state: State<'_, AppState>) -> Vec<Zona> {
    state.0.lock().unwrap().zonas.clone()
}

#[tauri::command]
pub fn obtener_mapas_zona(zona_id: u16, state: State<'_, AppState>) -> Vec<Mapa> {
    state.0.lock().unwrap().mapas.iter().filter(|m| m.id_zona == zona_id).cloned().collect()
}

#[tauri::command]
pub fn obtener_banco_entidades(state: State<'_, AppState>) -> Vec<Entidad> {
    state.0.lock().unwrap().banco_entidades.clone()
}

#[tauri::command]
pub fn obtener_instancias_activas(state: State<'_, AppState>) -> Vec<Instancia> {
    state.0.lock().unwrap().instancias_activas.clone()
}

// COMANDO TAURI: Solo lectura para cuando el frontend pulse las flechas
#[tauri::command]
pub async fn obtener_historial_personal() -> Vec<String> {
    if let Ok(contenido) = tokio::fs::read_to_string(ARCHIVO_HISTORIAL_PERSONAL).await {
        contenido
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect()
    } else {
        Vec::new()
    }
}
