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
    ZonaCreada(Zona),
    ZonaEliminada(Zona),
    ZonaModificada(Zona),
    MapaCreado(Zona, Mapa),
    MapaModificado(Mapa),
    MapaBorrado(Mapa)
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
            DeltaPartida::ZonaCreada(zona) => {
                if let Some(pos) = self.zonas.iter().position(|z| z.id == zona.id) {
                    self.zonas[pos] = zona.clone();
                } else {
                    self.zonas.push(zona.clone());
                }
            }
            DeltaPartida::ZonaEliminada(zona) => {
                // 1. Sacamos los IDs de los mapas que pertenecían a esta zona
                let mapas_eliminados: Vec<u16> = self
                    .mapas
                    .iter()
                    .filter(|m| m.id_zona == zona.id)
                    .map(|m| m.id)
                    .collect();

                // 2. Eliminamos la zona y sus mapas
                self.zonas.retain(|z| z.id != zona.id);
                self.mapas.retain(|m| m.id_zona != zona.id);

                // 3. Procesamos las instancias activas
                self.instancias_activas.retain_mut(|inst| {
                    let estaba_en_mapa_afectado = inst
                        .id_mapa
                        .map_or(false, |id_m| mapas_eliminados.contains(&id_m));
                    let pertenece_a_zona = inst.base.zona_id == zona.id;

                    // Si la instancia estaba en un mapa de la zona o pertenece a la zona
                    if estaba_en_mapa_afectado || pertenece_a_zona {
                        if inst.base.es_unico {
                            // ES ÚNICA: La conservamos viva en memoria pero sin mapa y desactivada
                            if estaba_en_mapa_afectado {
                                inst.id_mapa = None;
                                inst.coordenadas = Coordenada::NoPuesto;
                            }
                            inst.activo = false;
                            true // Conservar en el vector
                        } else {
                            // NO ES ÚNICA: Se elimina por completo
                            false // Descartar del vector
                        }
                    } else {
                        true // No afectada por esta zona
                    }
                });

                // 4. En el banco de entidades, conservamos las únicas (jugadores/NPCs clave)
                self.banco_entidades.retain(|ent| ent.es_unico || ent.zona_id != zona.id);
            }
            DeltaPartida::ZonaModificada(zona) => {
                if let Some(pos) = self.zonas.iter().position(|z| z.id == zona.id) {
                    self.zonas[pos] = zona.clone();
                }
            }
            DeltaPartida::MapaCreado(zona, mapa) => {
                let mut nuevo_mapa = mapa.clone();
                // Aseguramos que el id_zona pertenezca a la zona recibida
                nuevo_mapa.id_zona = zona.id;

                if let Some(pos) = self.mapas.iter().position(|m| m.id == nuevo_mapa.id) {
                    self.mapas[pos] = nuevo_mapa;
                } else {
                    self.mapas.push(nuevo_mapa);
                }
            }
            DeltaPartida::MapaModificado(mapa) => {
                if let Some(pos) = self.mapas.iter().position(|m| m.id == mapa.id) {
                    self.mapas[pos] = mapa.clone();
                }
            }
            DeltaPartida::MapaBorrado(mapa) => {
                // 1. Elimina el mapa
                self.mapas.retain(|m| m.id != mapa.id);

                // 2. Gestiona las instancias que estaban en ese mapa
                self.instancias_activas.retain_mut(|inst| {
                    if inst.id_mapa == Some(mapa.id) {
                        if inst.base.es_unico {
                            // ES ÚNICA: Conservar, pero retirar del mapa y desactivar
                            inst.id_mapa = None;
                            inst.activo = false;
                            inst.coordenadas = Coordenada::NoPuesto;
                            true // Conservar
                        } else {
                            // NO ES ÚNICA: Se destruye
                            false // Eliminar
                        }
                    } else {
                        true // Pertenecía a otro mapa (o ya no tenía mapa)
                    }
                });
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

#[tauri::command]
pub fn crear_zona(state: State<'_, AppState>) -> Zona {
    let (nuevo_id, nuevo_orden) = {
        let lock = state.0.lock().unwrap();
        let id = lock.zonas.iter().map(|z| z.id).max().unwrap_or(0) + 1;
        let orden = lock.zonas.iter().map(|z| z.orden).max().map(|o| o + 1).unwrap_or(0);
        (id, orden)
    };

    let nueva_zona = Zona {
        id: nuevo_id,
        nombre: format!("Zona {}", nuevo_id),
        orden: nuevo_orden,
    };

    procesar_delta(&state, DeltaPartida::ZonaCreada(nueva_zona.clone()));
    nueva_zona
}

#[tauri::command]
pub fn eliminar_zona(zona: Zona, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::ZonaEliminada(zona));
}

#[tauri::command]
pub fn modificar_zona(zona: Zona, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::ZonaModificada(zona));
}

#[tauri::command]
pub fn crear_mapa(
    id_zona: u16,
    url_imagen_fondo: Option<String>,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    let (zona, nuevo_id, nuevo_orden) = {
        let lock = state.0.lock().unwrap();
        let z = lock
            .zonas
            .iter()
            .find(|z| z.id == id_zona)
            .cloned()
            .ok_or_else(|| format!("La zona {} no existe", id_zona))?;

        // ID secuencial global para evitar cualquier colisión entre zonas
        let id = lock.mapas.iter().map(|m| m.id).max().unwrap_or(0) + 1;
        let orden = lock
            .mapas
            .iter()
            .filter(|m| m.id_zona == id_zona)
            .map(|m| m.orden)
            .max()
            .map(|o| o + 1)
            .unwrap_or(0);

        (z, id, orden)
    };

    let nuevo_mapa = Mapa {
        id: nuevo_id,
        id_zona,
        nombre: nuevo_id.to_string(), // Nombre por defecto es su propio ID
        url_imagen_fondo,
        ancho: 1000,
        alto: 1000,
        rejilla: 50,
        color_opacidad: vec![255, 255, 255, 76],
        grosor: 1,
        orden: nuevo_orden,
    };

    procesar_delta(&state, DeltaPartida::MapaCreado(zona, nuevo_mapa.clone()));
    Ok(nuevo_mapa)
}

#[tauri::command]
pub fn modificar_mapa(mapa: Mapa, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::MapaModificado(mapa));
}

#[tauri::command]
pub fn borrar_mapa(mapa: Mapa, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::MapaBorrado(mapa));
}