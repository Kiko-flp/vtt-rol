use serde::{Deserialize, Serialize};
use tauri::State;

use super::estado::DeltaPartida;
use super::modelos::{Coordenada, Cuerpo};
use super::partida::AppState;

const MAX_PUNTOS: usize = 4096;
const TOLERANCIA_PUNTO: f32 = 0.0001;
const MAX_COORDENADA: f32 = 32767.0;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct PuntoFlecha {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Flecha {
    pub id_instancia: u16,
    pub id_mapa: u16,
    pub puntos: Vec<PuntoFlecha>,
}

fn dimensiones_cuerpo(cuerpo: &Cuerpo) -> (f32, f32) {
    match cuerpo {
        Cuerpo::Grande(dimensiones) if dimensiones.len() >= 2 => (
            f32::from(dimensiones[0].max(1)),
            f32::from(dimensiones[1].max(1)),
        ),
        _ => (1.0, 1.0),
    }
}

fn limpiar_puntos(puntos: Vec<PuntoFlecha>) -> Result<Vec<PuntoFlecha>, String> {
    if puntos.len() < 2 || puntos.len() > MAX_PUNTOS {
        return Err(format!(
            "La flecha debe tener entre 2 y {MAX_PUNTOS} puntos"
        ));
    }

    let inicio = puntos[0];
    let fin = *puntos.last().expect("se validó que hay al menos dos puntos");
    for punto in &puntos {
        if !punto.x.is_finite()
            || !punto.y.is_finite()
            || punto.x < 0.0
            || punto.y < 0.0
            || punto.x > MAX_COORDENADA
            || punto.y > MAX_COORDENADA
        {
            return Err("La trayectoria de la flecha sale de los límites del mapa".into());
        }
    }

    let dx = fin.x - inicio.x;
    let dy = fin.y - inicio.y;
    if dx * dx + dy * dy <= TOLERANCIA_PUNTO * TOLERANCIA_PUNTO {
        return Err("La trayectoria de la flecha debe tener longitud".into());
    }
    Ok(vec![inicio, fin])
}

#[tauri::command]
pub fn obtener_flechas(id_mapa: u16, state: State<'_, AppState>) -> Vec<Flecha> {
    state
        .lock()
        .flechas
        .iter()
        .filter(|flecha| flecha.id_mapa == id_mapa)
        .cloned()
        .collect()
}

#[tauri::command]
pub fn guardar_flecha(
    instancia_id: u16,
    mapa_id: u16,
    puntos: Vec<PuntoFlecha>,
    state: State<'_, AppState>,
) -> Result<Flecha, String> {
    let flecha = {
        let partida = state.lock();
        if !partida.mapas.iter().any(|mapa| mapa.id == mapa_id) {
            return Err(format!("El mapa {mapa_id} no existe"));
        }
        let instancia = partida
            .instancias_activas
            .iter()
            .find(|instancia| {
                instancia.id_instancia == instancia_id
                    && instancia.activo
                    && instancia.id_mapa == Some(mapa_id)
            })
            .ok_or_else(|| format!("La instancia {instancia_id} no está activa en ese mapa"))?;
        let Coordenada::Localizacion(posicion) = &instancia.coordenadas else {
            return Err("La ficha no tiene una posición válida".into());
        };
        let (ancho_ficha, alto_ficha) = dimensiones_cuerpo(&instancia.base.cuerpo);
        let (x, y) = (
            posicion.first().copied().unwrap_or(0.0),
            posicion.get(1).copied().unwrap_or(0.0),
        );
        let puntos = limpiar_puntos(puntos)?;
        let inicio = puntos[0];
        if (inicio.x - (x + ancho_ficha / 2.0)).abs() > 0.1
            || (inicio.y - (y + alto_ficha / 2.0)).abs() > 0.1
        {
            return Err("La trayectoria debe comenzar en el centro de la ficha".into());
        }

        Flecha {
            id_instancia: instancia_id,
            id_mapa: mapa_id,
            puntos,
        }
    };

    state
        .lock()
        .registrar(
            &state.almacen,
            DeltaPartida::FlechaActualizada(flecha.clone()),
        )
        .map_err(|error| format!("No se pudo guardar la flecha: {error}"))?;
    Ok(flecha)
}

#[tauri::command]
pub fn quitar_flecha(
    instancia_id: u16,
    mapa_id: u16,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let existe = state
        .lock()
        .flechas
        .iter()
        .any(|flecha| flecha.id_instancia == instancia_id && flecha.id_mapa == mapa_id);
    if !existe {
        return Err(format!(
            "La ficha {instancia_id} no tiene una flecha en ese mapa"
        ));
    }

    state
        .lock()
        .registrar(
            &state.almacen,
            DeltaPartida::FlechaEliminada {
                id_instancia: instancia_id,
                id_mapa: mapa_id,
            },
        )
        .map_err(|error| format!("No se pudo quitar la flecha: {error}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tablero_general::almacen::{Almacen, pruebas::dir_temporal};
    use crate::tablero_general::estado::{EstadoPartida, cargar};

    #[test]
    fn limpiar_puntos_conserva_solo_extremos_para_linea_recta() {
        let puntos = limpiar_puntos(vec![
            PuntoFlecha { x: 1.0, y: 1.0 },
            PuntoFlecha { x: 2.0, y: 2.0 },
            PuntoFlecha { x: 3.0, y: 3.0 },
            PuntoFlecha { x: 3.0, y: 3.0 },
            PuntoFlecha { x: 3.0, y: 4.0 },
        ])
        .unwrap();

        assert_eq!(
            puntos,
            vec![
                PuntoFlecha { x: 1.0, y: 1.0 },
                PuntoFlecha { x: 3.0, y: 4.0 },
            ]
        );
    }

    #[test]
    fn limpiar_puntos_rechaza_coordenadas_fuera_del_mapa() {
        let resultado = limpiar_puntos(vec![
            PuntoFlecha { x: 1.0, y: 1.0 },
            PuntoFlecha {
                x: MAX_COORDENADA + 1.0,
                y: 1.0,
            },
        ]);
        assert!(resultado.is_err());
    }

    #[test]
    fn las_flechas_se_persisten_y_se_pueden_eliminar() {
        let dir = dir_temporal("flechas-persistencia");
        {
            let almacen = Almacen::abrir(&dir).unwrap();
            let mut estado = EstadoPartida::default();
            let flecha = Flecha {
                id_instancia: 7,
                id_mapa: 2,
                puntos: vec![
                    PuntoFlecha { x: 1.5, y: 1.5 },
                    PuntoFlecha { x: 2.0, y: 3.0 },
                ],
            };
            estado
                .registrar(&almacen, DeltaPartida::FlechaActualizada(flecha.clone()))
                .unwrap();
            let (recargado, _) = cargar(&almacen).unwrap();
            assert_eq!(recargado.flechas, vec![flecha]);

            estado
                .registrar(
                    &almacen,
                    DeltaPartida::FlechaEliminada {
                        id_instancia: 7,
                        id_mapa: 2,
                    },
                )
                .unwrap();
            let (recargado, _) = cargar(&almacen).unwrap();
            assert!(recargado.flechas.is_empty());
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
