import { describe, expect, it } from "vitest";
import { rutaFlechaConGiroCercano, trayectoriaRectaFlecha } from "$lib/flechas";

describe("trayectoria de flechas", () => {
  it("genera una línea recta desde la ficha al centro de la casilla apuntada", () => {
    expect(
      trayectoriaRectaFlecha(
        { x: 1.5, y: 1.5 },
        130,
        120,
        { left: 20, top: 10 },
        { x: 10, y: 20, zoom: 2 },
        { columnas: 10, filas: 8, anchoCelda: 20, altoCelda: 10 }
      )
    ).toEqual([
      { x: 1.5, y: 1.5 },
      { x: 2.5, y: 4.5 }
    ]);
  });

  it("mantiene la punta dentro de la rejilla para fichas grandes", () => {
    expect(
      trayectoriaRectaFlecha(
        { x: 1, y: 1 },
        10_000,
        10_000,
        { left: 0, top: 0 },
        { x: 0, y: 0, zoom: 1 },
        { columnas: 10, filas: 8, anchoCelda: 20, altoCelda: 20 },
        { columnas: 2, filas: 3 }
      )
    ).toEqual([
      { x: 1, y: 1 },
      { x: 9, y: 6.5 }
    ]);
  });

  it("coloca el giro ortogonal junto a la punta, por el tramo final más corto", () => {
    expect(rutaFlechaConGiroCercano({ x: 1, y: 1 }, { x: 4, y: 6 })).toEqual([
      { x: 1, y: 1 },
      { x: 4, y: 1 },
      { x: 4, y: 6 }
    ]);
    expect(rutaFlechaConGiroCercano({ x: 1, y: 1 }, { x: 6, y: 4 })).toEqual([
      { x: 1, y: 1 },
      { x: 1, y: 4 },
      { x: 6, y: 4 }
    ]);
  });
});
