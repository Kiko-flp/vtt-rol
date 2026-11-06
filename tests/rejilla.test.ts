import { describe, expect, it } from "vitest";
import { distribuirRejilla } from "$lib/rejilla";

describe("distribución de la rejilla", () => {
  it("ajusta las celdas para cubrir exactamente el mapa sin franja parcial", () => {
    const distribucion = distribuirRejilla(1000, 850, 70);
    expect(distribucion.columnas).toBe(14);
    expect(distribucion.filas).toBe(12);
    expect(distribucion.columnas * distribucion.anchoCelda).toBe(1000);
    expect(distribucion.filas * distribucion.altoCelda).toBe(850);
    expect(distribucion.anchoCelda).toBeCloseTo(71.43, 1);
    expect(distribucion.altoCelda).toBeCloseTo(70.83, 1);
  });

  it("mantiene celdas cuadradas cuando el mapa permite una división exacta", () => {
    expect(distribuirRejilla(980, 840, 70)).toEqual({
      columnas: 14,
      filas: 12,
      anchoCelda: 70,
      altoCelda: 70
    });
  });

  it("devuelve una distribución vacía para dimensiones inválidas", () => {
    expect(distribuirRejilla(40, 60, 0)).toEqual({
      columnas: 0,
      filas: 0,
      anchoCelda: 0,
      altoCelda: 0
    });
  });
});
