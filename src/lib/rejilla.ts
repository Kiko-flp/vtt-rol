export interface DistribucionRejilla {
  columnas: number;
  filas: number;
  anchoCelda: number;
  altoCelda: number;
}

export function distribuirRejilla(
  anchoMapa: number,
  altoMapa: number,
  tamanoDeseado: number
): DistribucionRejilla {
  if (
    !Number.isFinite(anchoMapa) ||
    !Number.isFinite(altoMapa) ||
    !Number.isFinite(tamanoDeseado) ||
    anchoMapa <= 0 ||
    altoMapa <= 0 ||
    tamanoDeseado <= 0
  ) {
    return { columnas: 0, filas: 0, anchoCelda: 0, altoCelda: 0 };
  }

  const columnas = Math.max(1, Math.round(anchoMapa / tamanoDeseado));
  const filas = Math.max(1, Math.round(altoMapa / tamanoDeseado));

  return {
    columnas,
    filas,
    anchoCelda: anchoMapa / columnas,
    altoCelda: altoMapa / filas
  };
}
