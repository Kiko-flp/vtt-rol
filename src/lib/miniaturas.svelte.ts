// src/lib/miniaturas.svelte.ts
//
// Miniaturas de los mapas para el panel izquierdo. Se cachean por la RUTA de la imagen (que
// es el hash de su contenido): una imagen nueva siempre tiene miniatura nueva, dos mapas con
// la misma imagen la comparten, y nunca hay que "invalidar" nada.
import { urlDeImagen } from "$lib/imagenes";

async function generar(src: string): Promise<string> {
  if (!src) return "";
  return new Promise((resolve) => {
    const img = new Image();
    img.crossOrigin = "anonymous";
    img.onload = () => {
      const canvas = document.createElement("canvas");
      canvas.width = 128;
      canvas.height = 72;
      const ctx = canvas.getContext("2d");
      if (!ctx) return resolve("");
      ctx.drawImage(img, 0, 0, 128, 72);
      resolve(canvas.toDataURL("image/png", 0.3));
    };
    img.onerror = () => resolve("");
    img.src = src;
  });
}

class Miniaturas {
  #porRuta = $state<Record<string, string>>({});
  /** Rutas ya pedidas (en curso o terminadas): evita generar dos veces la misma. */
  #pedidas = new Set<string>();

  /** Miniatura lista para mostrar, o "" si aún no está. Reactivo. */
  de(ruta: string | null | undefined): string {
    return ruta ? (this.#porRuta[ruta] ?? "") : "";
  }

  /** Genera la miniatura de esa ruta si no se ha hecho ya. */
  asegurar(ruta: string | null | undefined) {
    if (!ruta || this.#pedidas.has(ruta)) return;
    this.#pedidas.add(ruta);
    generar(urlDeImagen(ruta)).then((mini) => {
      if (mini) this.#porRuta[ruta] = mini;
    });
  }
}

export const miniaturas = new Miniaturas();
