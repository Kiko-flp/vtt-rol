// src-tauri/src/tablero_general/imagenes.rs
//
// Importa imágenes de mapa al directorio de datos de la app.
//  * Se copian a `<datos>/imagenes/<hash>.<ext>`: el nombre sale del CONTENIDO, así la misma
//    imagen importada dos veces se guarda una sola vez (y dos jugadores tendrán el mismo
//    nombre para la misma imagen cuando haya red).
//  * En la partida solo se guarda la ruta RELATIVA ("imagenes/<hash>.<ext>"): la partida no
//    depende de dónde estuviera el archivo original ni de qué sistema operativo se use.
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const EXTENSIONES: [&str; 8] = ["png", "jpg", "jpeg", "webp", "gif", "bmp", "avif", "svg"];
const MAX_BYTES: u64 = 100 * 1024 * 1024; // 100 MB

/// Copia `origen` a `<dir_datos>/imagenes/` y devuelve la ruta relativa para guardar en el mapa.
/// Bloqueante (lee y escribe el archivo entero): llamarlo desde un hilo aparte.
pub fn importar(dir_datos: &Path, origen: &Path) -> Result<String, String> {
    let ext = origen
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .filter(|e| EXTENSIONES.contains(&e.as_str()))
        .ok_or_else(|| format!("Formato de imagen no admitido: {}", origen.display()))?;

    let meta = fs::metadata(origen).map_err(|e| format!("No se puede leer la imagen: {e}"))?;
    if !meta.is_file() {
        return Err("La ruta no es un archivo".into());
    }
    if meta.len() > MAX_BYTES {
        return Err(format!(
            "La imagen pesa {} MB; el máximo es {} MB",
            meta.len() / (1024 * 1024),
            MAX_BYTES / (1024 * 1024)
        ));
    }

    let bytes = fs::read(origen).map_err(|e| format!("No se puede leer la imagen: {e}"))?;
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let nombre = format!("{}.{}", &hash[..32], ext);

    let carpeta = dir_datos.join("imagenes");
    fs::create_dir_all(&carpeta).map_err(|e| format!("No se puede crear la carpeta de imágenes: {e}"))?;

    let destino = carpeta.join(&nombre);
    if !destino.exists() {
        // Temporal + rename: nunca queda una imagen a medias con el nombre definitivo.
        let tmp = carpeta.join(format!("{nombre}.tmp"));
        fs::write(&tmp, &bytes).map_err(|e| format!("No se pudo copiar la imagen: {e}"))?;
        fs::rename(&tmp, &destino).map_err(|e| format!("No se pudo copiar la imagen: {e}"))?;
    }

    Ok(format!("imagenes/{nombre}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tablero_general::almacen::pruebas::dir_temporal;

    #[test]
    fn la_misma_imagen_se_guarda_una_vez_y_con_ruta_relativa() {
        let datos = dir_temporal("img_datos");
        let origen = dir_temporal("img_origen");
        fs::create_dir_all(&origen).unwrap();
        fs::write(origen.join("mapa.PNG"), b"contenido-de-prueba").unwrap();
        fs::write(origen.join("copia.png"), b"contenido-de-prueba").unwrap();

        let a = importar(&datos, &origen.join("mapa.PNG")).unwrap();
        let b = importar(&datos, &origen.join("copia.png")).unwrap();

        assert_eq!(a, b, "mismo contenido => mismo nombre");
        assert!(a.starts_with("imagenes/") && a.ends_with(".png"), "{a}");
        assert!(datos.join(&a).is_file());
        assert_eq!(fs::read_dir(datos.join("imagenes")).unwrap().count(), 1);
    }

    #[test]
    fn rechaza_extensiones_y_rutas_no_validas() {
        let datos = dir_temporal("img_rechazo");
        let origen = dir_temporal("img_rechazo_o");
        fs::create_dir_all(&origen).unwrap();
        fs::write(origen.join("script.exe"), b"x").unwrap();

        assert!(importar(&datos, &origen.join("script.exe")).is_err());
        assert!(importar(&datos, &origen.join("no_existe.png")).is_err());
        assert!(importar(&datos, &origen).is_err()); // un directorio
    }
}
