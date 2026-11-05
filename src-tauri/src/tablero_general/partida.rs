use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tokio::io::AsyncWriteExt;
use crate::tablero_general::modelos::*;

// ========================================================
// 1. DEFINICIÓN DE LOS DELTAS (Eventos de la partida)
// ========================================================
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DeltaPartida {
    TiradaRealizada(Tiradas),
    VidaModificada { id_instancia: u16, delta: i16 },
    EntidadCreada(Entidad),
    EntidadModificada(Entidad),
    InstanciaCreada(Instancia),
    InstanciaMovida {
        id_instancia: u16,
        id_mapa: u16,
        coordenadas: Coordenada,
    },
    InstanciaRetirada { id_instancia: u16 },
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
                    inst.vida_actual = (i32::from(inst.vida_actual) + i32::from(*delta))
                        .clamp(0, i32::from(inst.base.vida_max).max(0)) as i16;
                }
            }
            DeltaPartida::EntidadCreada(entidad) => {
                if let Some(pos) = self.banco_entidades.iter().position(|e| e.id == entidad.id) {
                    self.banco_entidades[pos] = entidad.clone();
                } else {
                    self.banco_entidades.push(entidad.clone());
                }
            }
            DeltaPartida::EntidadModificada(entidad) => {
                if let Some(pos) = self.banco_entidades.iter().position(|e| e.id == entidad.id) {
                    self.banco_entidades[pos] = entidad.clone();
                    for instancia in self
                        .instancias_activas
                        .iter_mut()
                        .filter(|i| i.base.id == entidad.id)
                    {
                        instancia.base = entidad.clone();
                        instancia.vida_actual = instancia
                            .vida_actual
                            .clamp(0, entidad.vida_max.max(0));
                    }
                }
            }
            DeltaPartida::InstanciaCreada(instancia) => {
                if let Some(pos) = self
                    .instancias_activas
                    .iter()
                    .position(|i| i.id_instancia == instancia.id_instancia)
                {
                    self.instancias_activas[pos] = instancia.clone();
                } else {
                    self.instancias_activas.push(instancia.clone());
                }
            }
            DeltaPartida::InstanciaMovida {
                id_instancia,
                id_mapa,
                coordenadas,
            } => {
                if let Some(inst) = self
                    .instancias_activas
                    .iter_mut()
                    .find(|i| i.id_instancia == *id_instancia)
                {
                    inst.id_mapa = Some(*id_mapa);
                    inst.coordenadas = coordenadas.clone();
                    inst.activo = true;
                }
            }
            DeltaPartida::InstanciaRetirada { id_instancia } => {
                if let Some(pos) = self
                    .instancias_activas
                    .iter()
                    .position(|i| i.id_instancia == *id_instancia)
                {
                    if self.instancias_activas[pos].base.es_unico {
                        let inst = &mut self.instancias_activas[pos];
                        inst.id_mapa = None;
                        inst.coordenadas = Coordenada::NoPuesto;
                        inst.activo = false;
                    } else {
                        self.instancias_activas.remove(pos);
                    }
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
    let mut linea = match serde_json::to_string(&delta) {
        Ok(linea) => linea,
        Err(err) => {
            eprintln!("No se pudo serializar un cambio de partida: {err}");
            return;
        }
    };
    linea.push('\n'); // Formato JSONL (un JSON por cada línea)

    let mut archivo = match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("deltas.jsonl")
        .await
    {
        Ok(archivo) => archivo,
        Err(err) => {
            eprintln!("No se pudo abrir deltas.jsonl para guardar la partida: {err}");
            return;
        }
    };
    if let Err(err) = archivo.write_all(linea.as_bytes()).await {
        eprintln!("No se pudo guardar un cambio en deltas.jsonl: {err}");
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
    persistir_delta_diferido(delta);
}

pub(super) fn persistir_delta_diferido(delta: DeltaPartida) {
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
pub fn crear_zona(state: State<'_, AppState>) -> Result<Zona, String> {
    let (nueva_zona, delta) = {
        let mut estado = state
            .0
            .lock()
            .map_err(|_| "No se pudo acceder al estado de la partida")?;
        let id = estado
            .zonas
            .iter()
            .map(|z| z.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Se alcanzó el límite de identificadores de zonas")?;
        let orden = estado
            .zonas
            .iter()
            .map(|z| z.orden)
            .max()
            .map(|orden| orden.checked_add(1).ok_or("Se alcanzó el límite de orden de zonas"))
            .transpose()?
            .unwrap_or(0);
        let zona = Zona {
            id,
            nombre: format!("Zona {id}"),
            orden,
        };
        let delta = DeltaPartida::ZonaCreada(zona.clone());
        estado.aplicar_delta(&delta);
        (zona, delta)
    };
    persistir_delta_diferido(delta);
    Ok(nueva_zona)
}

#[tauri::command]
pub fn eliminar_zona(zona: Zona, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::ZonaEliminada(zona));
}

fn actualizar_zona_en_estado(
    state: &State<'_, AppState>,
    zona_id: u16,
    cambio: impl FnOnce(&mut Zona) -> Result<(), String>,
) -> Result<Zona, String> {
    let (zona, delta) = {
        let mut estado = state
            .0
            .lock()
            .map_err(|_| "No se pudo acceder al estado de la partida")?;
        let zona = {
            let actual = estado
                .zonas
                .iter_mut()
                .find(|zona| zona.id == zona_id)
                .ok_or_else(|| format!("La zona {zona_id} ya no existe"))?;
            cambio(actual)?;
            actual.clone()
        };
        let delta = DeltaPartida::ZonaModificada(zona.clone());
        estado.aplicar_delta(&delta);
        (zona, delta)
    };
    persistir_delta_diferido(delta);
    Ok(zona)
}

fn actualizar_mapa_en_estado(
    state: &State<'_, AppState>,
    mapa_id: u16,
    cambio: impl FnOnce(&mut Mapa) -> Result<(), String>,
) -> Result<Mapa, String> {
    let (mapa, delta) = {
        let mut estado = state
            .0
            .lock()
            .map_err(|_| "No se pudo acceder al estado de la partida")?;
        let mapa = {
            let actual = estado
                .mapas
                .iter_mut()
                .find(|mapa| mapa.id == mapa_id)
                .ok_or_else(|| format!("El mapa {mapa_id} ya no existe"))?;
            cambio(actual)?;
            actual.clone()
        };
        let delta = DeltaPartida::MapaModificado(mapa.clone());
        estado.aplicar_delta(&delta);
        (mapa, delta)
    };
    persistir_delta_diferido(delta);
    Ok(mapa)
}

#[tauri::command]
pub fn renombrar_zona(
    zona_id: u16,
    nombre: String,
    state: State<'_, AppState>,
) -> Result<Zona, String> {
    let nombre = nombre.trim().to_string();
    if nombre.is_empty() || nombre.chars().count() > 80 {
        return Err("El nombre de la zona debe tener entre 1 y 80 caracteres".into());
    }
    actualizar_zona_en_estado(&state, zona_id, move |zona| {
        zona.nombre = nombre;
        Ok(())
    })
}

#[tauri::command]
pub fn reordenar_zona(
    zona_id: u16,
    orden: u16,
    state: State<'_, AppState>,
) -> Result<Zona, String> {
    actualizar_zona_en_estado(&state, zona_id, |zona| {
        zona.orden = orden;
        Ok(())
    })
}

#[tauri::command]
pub fn renombrar_mapa(
    mapa_id: u16,
    nombre: String,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    let nombre = nombre.trim().to_string();
    if nombre.is_empty() || nombre.chars().count() > 80 {
        return Err("El nombre del mapa debe tener entre 1 y 80 caracteres".into());
    }
    actualizar_mapa_en_estado(&state, mapa_id, move |mapa| {
        mapa.nombre = nombre;
        Ok(())
    })
}

#[tauri::command]
pub fn reordenar_mapa(
    mapa_id: u16,
    orden: u16,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    actualizar_mapa_en_estado(&state, mapa_id, |mapa| {
        mapa.orden = orden;
        Ok(())
    })
}

#[tauri::command]
pub fn actualizar_imagen_mapa(
    mapa_id: u16,
    url_imagen_fondo: Option<String>,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    actualizar_mapa_en_estado(&state, mapa_id, move |mapa| {
        mapa.url_imagen_fondo = url_imagen_fondo;
        Ok(())
    })
}

#[tauri::command]
pub fn actualizar_rejilla_mapa(
    mapa_id: u16,
    rejilla: u16,
    rejilla_visible: bool,
    grosor: u16,
    color_opacidad: Vec<u8>,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    if !(5..=300).contains(&rejilla) {
        return Err("El tamaño de la rejilla debe estar entre 5 y 300".into());
    }
    if !(1..=5).contains(&grosor) {
        return Err("El grosor de la rejilla debe estar entre 1 y 5".into());
    }
    if color_opacidad.len() != 4 {
        return Err("El color de la rejilla debe tener cuatro componentes RGBA".into());
    }
    actualizar_mapa_en_estado(&state, mapa_id, move |mapa| {
        mapa.rejilla = rejilla;
        mapa.rejilla_visible = rejilla_visible;
        mapa.grosor = grosor;
        mapa.color_opacidad = color_opacidad;
        Ok(())
    })
}

#[tauri::command]
pub fn crear_mapa(
    id_zona: u16,
    url_imagen_fondo: Option<String>,
    state: State<'_, AppState>,
) -> Result<Mapa, String> {
    let (nuevo_mapa, delta) = {
        let mut estado = state
            .0
            .lock()
            .map_err(|_| "No se pudo acceder al estado de la partida")?;
        if !estado.zonas.iter().any(|zona| zona.id == id_zona) {
            return Err(format!("La zona {id_zona} no existe"));
        }

        let id = estado
            .mapas
            .iter()
            .map(|mapa| mapa.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Se alcanzó el límite de identificadores de mapas")?;
        let orden = estado
            .mapas
            .iter()
            .filter(|mapa| mapa.id_zona == id_zona)
            .map(|mapa| mapa.orden)
            .max()
            .map(|orden| orden.checked_add(1).ok_or("Se alcanzó el límite de orden de mapas"))
            .transpose()?
            .unwrap_or(0);

        let mapa = Mapa {
            id,
            id_zona,
            nombre: id.to_string(),
            url_imagen_fondo,
            ancho: 1000,
            alto: 1000,
            rejilla: 50,
            rejilla_visible: true,
            color_opacidad: vec![255, 255, 255, 76],
            grosor: 1,
            orden,
        };
        let zona = estado
            .zonas
            .iter()
            .find(|z| z.id == id_zona)
            .cloned()
            .ok_or_else(|| format!("La zona {id_zona} ya no existe"))?;
        let delta = DeltaPartida::MapaCreado(zona, mapa.clone());
        estado.aplicar_delta(&delta);
        (mapa, delta)
    };
    persistir_delta_diferido(delta);
    Ok(nuevo_mapa)
}

#[tauri::command]
pub fn borrar_mapa(mapa: Mapa, state: State<'_, AppState>) {
    procesar_delta(&state, DeltaPartida::MapaBorrado(mapa));
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::modelos::{Cuerpo, TipoEntidad};

    #[test]
    fn los_mapas_anteriores_se_cargan_con_la_rejilla_visible() {
        let mut mapa: Mapa = serde_json::from_str(
            r#"{
                "id": 1,
                "id_zona": 1,
                "nombre": "Mapa",
                "url_imagen_fondo": null,
                "ancho": 1000,
                "alto": 1000,
                "rejilla": 50,
                "color_opacidad": [255, 255, 255, 76],
                "grosor": 1,
                "orden": 0
            }"#,
        )
        .expect("El formato anterior del mapa debe seguir siendo válido");

        assert!(mapa.rejilla_visible);
        mapa.rejilla_visible = false;
        let serializado = serde_json::to_string(&mapa).expect("El mapa debe serializarse");
        let recargado: Mapa =
            serde_json::from_str(&serializado).expect("El mapa serializado debe cargarse");
        assert!(!recargado.rejilla_visible);
    }

    fn entidad(es_unico: bool) -> Entidad {
        Entidad {
            id: 1,
            nombre: "Ficha de prueba".into(),
            tipo: TipoEntidad::Enemigo,
            vida_max: 10,
            armadura: 0,
            velocidad: 6,
            rango_arma: 1,
            cuerpo: Cuerpo::Pequenio,
            zona_id: 1,
            foto_url: None,
            es_publico: false,
            archivos_adjuntos: Vec::new(),
            es_unico,
        }
    }

    fn instancia(es_unico: bool) -> Instancia {
        Instancia {
            id_instancia: 1,
            id_mapa: Some(1),
            base: entidad(es_unico),
            vida_actual: 10,
            coordenadas: Coordenada::Localizacion(vec![0.0, 0.0]),
            activo: true,
        }
    }

    #[test]
    fn instancia_guarda_movimiento_y_vida_y_puede_retirarse() {
        let mut estado = EstadoPartida::default();
        estado
            .instancias_activas
            .push(instancia(false));

        estado.aplicar_delta(&DeltaPartida::InstanciaMovida {
            id_instancia: 1,
            id_mapa: 2,
            coordenadas: Coordenada::Localizacion(vec![3.5, 4.25]),
        });
        estado.aplicar_delta(&DeltaPartida::VidaModificada {
            id_instancia: 1,
            delta: -20,
        });

        let colocada = &estado.instancias_activas[0];
        assert_eq!(colocada.id_mapa, Some(2));
        assert!(matches!(
            &colocada.coordenadas,
            Coordenada::Localizacion(posicion) if posicion == &vec![3.5, 4.25]
        ));
        assert_eq!(colocada.vida_actual, 0);

        estado.aplicar_delta(&DeltaPartida::InstanciaRetirada { id_instancia: 1 });
        assert!(estado.instancias_activas.is_empty());
    }

    #[test]
    fn retirar_ficha_unica_la_conserva_inactiva() {
        let mut estado = EstadoPartida::default();
        estado.instancias_activas.push(instancia(true));

        estado.aplicar_delta(&DeltaPartida::InstanciaRetirada { id_instancia: 1 });

        let retirada = &estado.instancias_activas[0];
        assert!(!retirada.activo);
        assert_eq!(retirada.id_mapa, None);
        assert!(matches!(&retirada.coordenadas, Coordenada::NoPuesto));
    }

    #[test]
    fn editar_entidad_actualiza_sus_instancias_y_limita_la_vida() {
        let mut estado = EstadoPartida::default();
        estado.banco_entidades.push(entidad(false));
        estado.instancias_activas.push(instancia(false));
        let mut editada = entidad(false);
        editada.nombre = "Ficha editada".into();
        editada.vida_max = 4;

        estado.aplicar_delta(&DeltaPartida::EntidadModificada(editada));

        assert_eq!(estado.banco_entidades[0].nombre, "Ficha editada");
        assert_eq!(estado.instancias_activas[0].base.nombre, "Ficha editada");
        assert_eq!(estado.instancias_activas[0].vida_actual, 4);
    }
}