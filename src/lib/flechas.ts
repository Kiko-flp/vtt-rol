import type { DistribucionRejilla } from "$lib/rejilla";
import type { PuntoFlecha } from "$lib/partida.svelte";

interface Camara {
  x: number;
  y: number;
  zoom: number;
}

interface Limites {
  left: number;
  top: number;
}

interface DimensionesFicha {
  columnas: number;
  filas: number;
}

export function trayectoriaRectaFlecha(
  origen: PuntoFlecha,
  clientX: number,
  clientY: number,
  limites: Limites,
  camara: Camara,
  rejilla: DistribucionRejilla,
  ficha: DimensionesFicha = { columnas: 1, filas: 1 }
): PuntoFlecha[] {
  const zoom = Math.max(Number.MIN_VALUE, camara.zoom);
  const x = (clientX - limites.left - camara.x) / zoom / rejilla.anchoCelda;
  const y = (clientY - limites.top - camara.y) / zoom / rejilla.altoCelda;
  const celdaX = Math.min(
    Math.max(0, rejilla.columnas - ficha.columnas),
    Math.max(0, Math.floor(x))
  );
  const celdaY = Math.min(
    Math.max(0, rejilla.filas - ficha.filas),
    Math.max(0, Math.floor(y))
  );

  return [
    origen,
    {
      x: celdaX + ficha.columnas / 2,
      y: celdaY + ficha.filas / 2
    }
  ];
}

export function rutaFlechaConGiroCercano(
  inicio: PuntoFlecha,
  fin: PuntoFlecha
): PuntoFlecha[] {
  const deltaX = fin.x - inicio.x;
  const deltaY = fin.y - inicio.y;
  if (deltaX === 0 || deltaY === 0) return [inicio, fin];

  const giro = Math.abs(deltaX) < Math.abs(deltaY)
    ? { x: fin.x, y: inicio.y }
    : { x: inicio.x, y: fin.y };
  return [inicio, giro, fin];
}
