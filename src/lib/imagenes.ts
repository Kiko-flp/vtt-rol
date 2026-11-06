// src/lib/imagenes.ts
//
// Las imágenes de mapas y fichas se guardan como rutas relativas al directorio de datos de la app.
// Para mostrar también las rutas absolutas antiguas de las fichas, se aceptan ambos formatos.
import { invoke, convertFileSrc } from "@tauri-apps/api/core";

let dirDatos = "";

/** Llamar UNA vez al arrancar, antes de pintar ningún mapa. */
export async function iniciarImagenes(): Promise<void> {
  dirDatos = await invoke<string>("obtener_dir_datos");
}

// Valores que ya son una URL completa (mapas creados con versiones anteriores de la app).
const YA_ES_URL = /^(https?:|data:|blob:|asset:)/i;

/** Copia una imagen del sistema a la carpeta de datos de la app. Devuelve la ruta relativa a guardar. */
export function importarImagen(origen: string): Promise<string> {
  return invoke<string>("importar_imagen", { origen });
}

/** Ruta guardada en el mapa o ficha -> URL que el <img>/canvas puede cargar. "" si no hay imagen. */
export function urlDeImagen(ref: string | null | undefined): string {
  if (!ref) return "";
  if (YA_ES_URL.test(ref)) return ref;
  // Las versiones anteriores guardaban las fotos de las fichas como rutas absolutas.
  if (/^(?:[a-z]:[\\/]|\\\\|\/)/i.test(ref)) return convertFileSrc(ref);

  // Usamos el mismo separador que el sistema para que la ruta sea coherente (C:\...\imagenes\x.png).
  const sep = dirDatos.includes("\\") ? "\\" : "/";
  return convertFileSrc(`${dirDatos}${sep}${ref.split("/").join(sep)}`);
}
