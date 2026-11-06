use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use tauri::State;

use super::estado::{DeltaPartida, EstadoPartida};
use super::modelos::{Coordenada, Cuerpo, Entidad, Instancia, TipoEntidad};
use super::partida::{procesar_delta, AppState};

#[tauri::command]
pub fn obtener_banco_entidades(state: State<'_, AppState>) -> Result<Vec<Entidad>, String> {
    Ok(state.lock().banco_entidades.clone())
}

#[tauri::command]
pub fn obtener_instancias_activas(state: State<'_, AppState>) -> Result<Vec<Instancia>, String> {
    Ok(state.lock().instancias_activas.clone())
}

#[tauri::command]
pub fn crear_entidad(
    nombre: String,
    tipo: TipoEntidad,
    vida_max: i16,
    armadura: i16,
    velocidad: i16,
    rango_arma: i16,
    zona_id: u16,
    state: State<'_, AppState>,
) -> Result<Entidad, String> {
    let nombre = nombre.trim();
    if nombre.is_empty() || nombre.chars().count() > 80 {
        return Err("El nombre de la ficha debe tener entre 1 y 80 caracteres".into());
    }
    if vida_max <= 0 {
        return Err("La vida máxima debe ser mayor que cero".into());
    }
    if armadura < 0 || velocidad < 0 || rango_arma < 0 {
        return Err("Armadura, velocidad y rango no pueden ser negativos".into());
    }

    let entidad = {
        let mut partida = state.lock();
        if !partida.zonas.iter().any(|zona| zona.id == zona_id) {
            return Err(format!("La zona {zona_id} no existe"));
        }
        let id = partida
            .banco_entidades
            .iter()
            .map(|entidad| entidad.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or("Se alcanzó el límite de identificadores de entidades")?;
        let entidad = Entidad {
            id,
            nombre: nombre.to_string(),
            tipo,
            vida_max,
            armadura,
            velocidad,
            rango_arma,
            cuerpo: Cuerpo::Pequenio,
            zona_id,
            foto_url: None,
            vida_es_publica: false,
            armadura_es_publica: false,
            archivos_adjuntos: Vec::new(),
            es_unico: false,
        };
        let delta = DeltaPartida::EntidadCreada(entidad.clone());
        partida
            .registrar(&state.almacen, delta)
            .map_err(|err| format!("No se pudo guardar la ficha: {err}"))?;
        entidad
    };

    Ok(entidad)
}

#[tauri::command]
pub fn actualizar_entidad(
    mut entidad: Entidad,
    foto_data: Option<String>,
    quitar_foto: bool,
    state: State<'_, AppState>,
) -> Result<Entidad, String> {
    entidad.nombre = entidad.nombre.trim().to_string();
    if entidad.nombre.is_empty() || entidad.nombre.chars().count() > 80 {
        return Err("El nombre de la ficha debe tener entre 1 y 80 caracteres".into());
    }
    if entidad.vida_max <= 0 {
        return Err("La vida máxima debe ser mayor que cero".into());
    }
    if entidad.armadura < 0 || entidad.velocidad < 0 || entidad.rango_arma < 0 {
        return Err("Armadura, velocidad y rango no pueden ser negativos".into());
    }
    if foto_data.is_some() && quitar_foto {
        return Err("No se puede subir y quitar una imagen en la misma operación".into());
    }

    let mut partida = state.lock();
    let actual = partida
        .banco_entidades
        .iter()
        .find(|existente| existente.id == entidad.id)
        .ok_or_else(|| format!("La ficha {} ya no existe", entidad.id))?;
    if !partida.zonas.iter().any(|zona| zona.id == entidad.zona_id) {
        return Err(format!("La zona {} no existe", entidad.zona_id));
    }
    entidad.foto_url = if quitar_foto {
        None
    } else {
        actual.foto_url.clone()
    };

    if let Some(data_url) = foto_data {
        if data_url.len() > 5 * 1024 * 1024 {
            return Err("La imagen procesada supera el límite de 5 MB".into());
        }
        let (header, contenido) = data_url
            .split_once(',')
            .ok_or("El formato de imagen recibido no es válido")?;
        let extension = match header {
            "data:image/webp;base64" => "webp",
            "data:image/png;base64" => "png",
            "data:image/jpeg;base64" => "jpg",
            _ => return Err("Usa una imagen PNG, JPEG o WebP".into()),
        };
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(contenido)
            .map_err(|_| "No se pudo leer el contenido de la imagen")?;
        if bytes.is_empty() || bytes.len() > 3 * 1024 * 1024 {
            return Err("La imagen debe ocupar menos de 3 MB después de procesarla".into());
        }

        let directorio = state.almacen.dir().join("fichas");
        std::fs::create_dir_all(&directorio)
            .map_err(|err| format!("No se pudo crear la carpeta de imágenes: {err}"))?;
        let marca_tiempo = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("No se pudo obtener la hora del sistema: {err}"))?
            .as_nanos();
        let nombre_archivo = format!("ficha-{}-{marca_tiempo}.{extension}", entidad.id);
        let ruta = directorio.join(&nombre_archivo);
        std::fs::write(&ruta, bytes)
            .map_err(|err| format!("No se pudo guardar la imagen de la ficha: {err}"))?;
        entidad.foto_url = Some(format!("fichas/{nombre_archivo}"));
    }

    let delta = DeltaPartida::EntidadModificada(entidad.clone());
    partida
        .registrar(&state.almacen, delta)
        .map_err(|err| format!("No se pudo guardar la ficha: {err}"))?;
    Ok(entidad)
}

fn validar_posicion(x: f32, y: f32) -> Result<(), String> {
    if !x.is_finite()
        || !y.is_finite()
        || x < 0.0
        || y < 0.0
        || x > 32767.0
        || y > 32767.0
        || x.fract() != 0.0
        || y.fract() != 0.0
    {
        return Err("La posición de la ficha no es válida".into());
    }
    Ok(())
}

#[tauri::command]
pub fn colocar_entidad(
    entidad_id: u16,
    mapa_id: u16,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<Instancia, String> {
    validar_posicion(x, y)?;
    let instancia = {
        let mut partida = state.lock();
        if !partida.mapas.iter().any(|mapa| mapa.id == mapa_id) {
            return Err(format!("El mapa {mapa_id} no existe"));
        }
        let entidad = partida
            .banco_entidades
            .iter()
            .find(|entidad| entidad.id == entidad_id)
            .cloned()
            .ok_or_else(|| format!("La entidad {entidad_id} no existe"))?;
        let instancia = if entidad.es_unico {
            if let Some(mut instancia) = partida
                .instancias_activas
                .iter()
                .find(|instancia| instancia.base.id == entidad_id)
                .cloned()
            {
                if instancia.activo && instancia.id_mapa.is_some() {
                    return Err(format!("La ficha «{}» ya está colocada", entidad.nombre));
                }
                instancia.id_mapa = Some(mapa_id);
                instancia.coordenadas = Coordenada::Localizacion(vec![x, y]);
                instancia.activo = true;
                instancia
            } else {
                nueva_instancia(&partida, entidad, mapa_id, x, y)?
            }
        } else {
            nueva_instancia(&partida, entidad, mapa_id, x, y)?
        };
        let delta = DeltaPartida::InstanciaCreada(instancia.clone());
        partida
            .registrar(&state.almacen, delta)
            .map_err(|err| format!("No se pudo colocar la ficha: {err}"))?;
        instancia
    };
    Ok(instancia)
}

fn nueva_instancia(
    partida: &EstadoPartida,
    entidad: Entidad,
    mapa_id: u16,
    x: f32,
    y: f32,
) -> Result<Instancia, String> {
    let id_instancia = partida
        .instancias_activas
        .iter()
        .map(|instancia| instancia.id_instancia)
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or("Se alcanzó el límite de identificadores de instancias")?;
    Ok(Instancia {
        id_instancia,
        id_mapa: Some(mapa_id),
        vida_actual: entidad.vida_max,
        base: entidad,
        coordenadas: Coordenada::Localizacion(vec![x, y]),
        activo: true,
    })
}

#[tauri::command]
pub fn mover_instancia(
    instancia_id: u16,
    mapa_id: u16,
    x: f32,
    y: f32,
    state: State<'_, AppState>,
) -> Result<(), String> {
    validar_posicion(x, y)?;
    {
        let partida = state.lock();
        if !partida.mapas.iter().any(|mapa| mapa.id == mapa_id) {
            return Err(format!("El mapa {mapa_id} no existe"));
        }
        if !partida
            .instancias_activas
            .iter()
            .any(|instancia| instancia.id_instancia == instancia_id && instancia.activo)
        {
            return Err(format!("La instancia {instancia_id} no está activa"));
        }
    }
    procesar_delta(
        &state,
        DeltaPartida::InstanciaMovida {
            id_instancia: instancia_id,
            id_mapa: mapa_id,
            coordenadas: Coordenada::Localizacion(vec![x, y]),
        },
    );
    Ok(())
}

#[tauri::command]
pub fn modificar_vida(
    instancia_id: u16,
    delta: i16,
    state: State<'_, AppState>,
) -> Result<Instancia, String> {
    let instancia = {
        let partida = state.lock();
        partida
            .instancias_activas
            .iter()
            .find(|instancia| instancia.id_instancia == instancia_id && instancia.activo)
            .cloned()
            .ok_or_else(|| format!("La instancia {instancia_id} no está activa"))?
    };
    procesar_delta(
        &state,
        DeltaPartida::VidaModificada {
            id_instancia: instancia_id,
            delta,
        },
    );
    let mut actualizada = instancia;
    actualizada.vida_actual = (i32::from(actualizada.vida_actual) + i32::from(delta))
        .clamp(0, i32::from(actualizada.base.vida_max).max(0)) as i16;
    Ok(actualizada)
}

#[tauri::command]
pub fn retirar_instancia(instancia_id: u16, state: State<'_, AppState>) -> Result<(), String> {
    let existe = state
        .lock()
        .instancias_activas
        .iter()
        .any(|instancia| instancia.id_instancia == instancia_id && instancia.activo);
    if !existe {
        return Err(format!("La instancia {instancia_id} no está activa"));
    }
    procesar_delta(&state, DeltaPartida::InstanciaRetirada { id_instancia: instancia_id });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rechaza_posiciones_invalidas() {
        assert!(validar_posicion(0.0, 1.0).is_ok());
        assert!(validar_posicion(0.5, 1.0).is_err());
        assert!(validar_posicion(0.0, 1.25).is_err());
        assert!(validar_posicion(f32::NAN, 1.0).is_err());
        assert!(validar_posicion(-0.1, 0.0).is_err());
        assert!(validar_posicion(32768.0, 0.0).is_err());
    }

    #[test]
    fn coordenadas_enteras_anteriores_siguen_cargandose() {
        let coordenadas: Coordenada =
            serde_json::from_str(r#"{"Localizacion":[3,4]}"#).expect("Las coordenadas antiguas deben cargarse");
        assert!(matches!(
            coordenadas,
            Coordenada::Localizacion(posicion) if posicion == vec![3.0, 4.0]
        ));
    }
}
