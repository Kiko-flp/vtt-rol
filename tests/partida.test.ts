// Pruebas del estado compartido del front contra un "Rust" falso en memoria.
import { beforeEach, afterEach, describe, expect, it, vi } from "vitest";

type Args = Record<string, any>;
type InstanciaBackend = {
  id_instancia: number;
  id_mapa: number | null;
  vida_actual: number;
  base: ReturnType<typeof crearEntidad>;
  coordenadas: "NoPuesto" | { Localizacion: number[] };
  activo: boolean;
};

// ---- Backend falso: imita los comandos de Tauri que usa el store ----
function crearEntidad(id: number, nombre: string, zona_id: number, tipo = "Enemigo", vida_max = 18) {
  return {
    id, nombre, tipo, vida_max, armadura: 1, velocidad: 6, rango_arma: 1, cuerpo: "Pequenio",
    zona_id, foto_url: null as string | null, vida_es_publica: false, armadura_es_publica: false,
    archivos_adjuntos: [] as string[], es_unico: false
  };
}

function crearBackend() {
  const clon = <T>(x: T): T => structuredClone(x);
  const entidad = crearEntidad;
  const mapa = (id: number, id_zona: number, orden: number) => ({
    id, id_zona, nombre: String(id), url_imagen_fondo: null as string | null,
    ancho: 1000, alto: 1000, rejilla: 50, rejilla_visible: true,
    color_opacidad: [255, 255, 255, 76], grosor: 1, orden
  });
  const b = {
    zonas: [
      { id: 1, nombre: "A", orden: 0 },
      { id: 2, nombre: "B", orden: 1 },
      { id: 3, nombre: "C", orden: 2 }
    ],
    mapas: [mapa(1, 1, 0), mapa(2, 1, 1), mapa(3, 2, 0), mapa(4, 3, 0)],
    banco: [entidad(1, "Orco", 1), entidad(2, "Lobo", 2, "Enemigo", 9)],
    instancias: [
      { id_instancia: 1, id_mapa: 1, vida_actual: 10, base: entidad(1, "Orco", 1), coordenadas: { Localizacion: [0, 0] }, activo: true },
      { id_instancia: 2, id_mapa: 3, vida_actual: 5, base: entidad(2, "Lobo", 2, "Enemigo", 9), coordenadas: { Localizacion: [0, 0] }, activo: true }
    ] as InstanciaBackend[],
    flechas: [] as { id_instancia: number; id_mapa: number; puntos: { x: number; y: number }[] }[],
    llamadas: [] as { cmd: string; args: Args }[],
    invoke: async (cmd: string, args: Args = {}): Promise<any> => {
      b.llamadas.push({ cmd, args: clon(args) });
      switch (cmd) {
        case "obtener_zonas": return clon(b.zonas);
        case "obtener_mapas_zona": return clon(b.mapas.filter((m) => m.id_zona === args.zonaId));
        case "obtener_banco_entidades": return clon(b.banco);
        case "obtener_instancias_activas": return clon(b.instancias);
        case "obtener_flechas": return clon(b.flechas.filter((f) => f.id_mapa === args.idMapa));
        case "obtener_dir_datos": return "/datos";
        case "importar_imagen": return "imagenes/" + String(args.origen).split("/").pop();
        case "crear_entidad": {
          const creada = entidad(
            Math.max(...b.banco.map((x) => x.id)) + 1,
            args.nombre,
            args.zonaId,
            args.tipo,
            args.vidaMax
          );
          creada.armadura = args.armadura;
          creada.velocidad = args.velocidad;
          creada.rango_arma = args.rangoArma;
          b.banco.push(creada);
          return clon(creada);
        }
        case "actualizar_entidad": {
          const actualizada = clon(args.entidad);
          if (args.quitarFoto) actualizada.foto_url = null;
          else if (args.fotoData) actualizada.foto_url = args.fotoData;
          b.banco = b.banco.map((e) => e.id === actualizada.id ? actualizada : e);
          b.instancias = b.instancias.map((i) => i.base.id === actualizada.id
            ? { ...i, base: clon(actualizada), vida_actual: Math.min(i.vida_actual, actualizada.vida_max) }
            : i);
          return clon(actualizada);
        }
        case "colocar_entidad": {
          const base = b.banco.find((e) => e.id === args.entidadId);
          if (!base) throw new Error("entidad inexistente");
          const creada: InstanciaBackend = {
            id_instancia: Math.max(...b.instancias.map((i) => i.id_instancia), 0) + 1,
            id_mapa: args.mapaId, coordenadas: { Localizacion: [args.x, args.y] },
            vida_actual: base.vida_max, base: clon(base), activo: true
          };
          b.instancias.push(creada);
          return clon(creada);
        }
        case "mover_instancia": {
          const instancia = b.instancias.find((i) => i.id_instancia === args.instanciaId);
          if (instancia) {
            instancia.id_mapa = args.mapaId;
            instancia.coordenadas = { Localizacion: [args.x, args.y] };
          }
          b.flechas = b.flechas.filter((f) => f.id_instancia !== args.instanciaId);
          return;
        }
        case "guardar_flecha": {
          const flecha = {
            id_instancia: args.instanciaId,
            id_mapa: args.mapaId,
            puntos: clon(args.puntos)
          };
          b.flechas = [...b.flechas.filter((f) => f.id_instancia !== args.instanciaId), flecha];
          return clon(flecha);
        }
        case "quitar_flecha":
          b.flechas = b.flechas.filter((f) => f.id_instancia !== args.instanciaId || f.id_mapa !== args.mapaId);
          return;
        case "crear_zona": {
          const z = { id: Math.max(...b.zonas.map((x) => x.id)) + 1, nombre: "nueva", orden: b.zonas.length };
          b.zonas.push(z);
          return clon(z);
        }
        case "modificar_zona": b.zonas = b.zonas.map((z) => (z.id === args.zona.id ? clon(args.zona) : z)); return;
        case "eliminar_zona": {
          const quitados = b.mapas.filter((m) => m.id_zona === args.zona.id).map((m) => m.id);
          b.zonas = b.zonas.filter((z) => z.id !== args.zona.id);
          b.mapas = b.mapas.filter((m) => m.id_zona !== args.zona.id);
          b.instancias = b.instancias.filter((i) => i.id_mapa === null || !quitados.includes(i.id_mapa));
          b.flechas = b.flechas.filter((f) => !quitados.includes(f.id_mapa));
          b.banco = b.banco.filter((e) => e.zona_id !== args.zona.id);
          return;
        }
        case "crear_mapa": {
          const m = mapa(Math.max(...b.mapas.map((x) => x.id)) + 1, args.idZona, 99);
          m.url_imagen_fondo = args.urlImagenFondo;
          b.mapas.push(m);
          return clon(m);
        }
        case "modificar_mapa": b.mapas = b.mapas.map((m) => (m.id === args.mapa.id ? clon(args.mapa) : m)); return;
        case "borrar_mapa":
          b.mapas = b.mapas.filter((m) => m.id !== args.mapa.id);
          b.flechas = b.flechas.filter((f) => f.id_mapa !== args.mapa.id);
          return;
        case "modificar_vida": {
          const instancia = b.instancias.find((i) => i.id_instancia === args.instanciaId);
          if (instancia) instancia.vida_actual = Math.max(0, Math.min(instancia.base.vida_max, instancia.vida_actual + args.delta));
          return clon(instancia);
        }
        case "retirar_instancia": {
          const instancia = b.instancias.find((i) => i.id_instancia === args.instanciaId);
          if (instancia?.base.es_unico) {
            instancia.activo = false;
            instancia.id_mapa = null;
            instancia.coordenadas = "NoPuesto";
          } else {
            b.instancias = b.instancias.filter((i) => i.id_instancia !== args.instanciaId);
          }
          b.flechas = b.flechas.filter((f) => f.id_instancia !== args.instanciaId);
          return;
        }
        default: throw new Error("comando desconocido: " + cmd);
      }
    }
  };
  return b;
}

let backend: ReturnType<typeof crearBackend>;

async function nuevoStore() {
  vi.resetModules();
  backend = crearBackend();
  vi.doMock("@tauri-apps/api/core", () => ({
    invoke: (c: string, a?: Args) => backend.invoke(c, a),
    convertFileSrc: (p: string) => "asset://" + p
  }));
  const { partida } = await import("$lib/partida.svelte");
  await partida.iniciar();
  return partida;
}

const llamadas = (cmd: string) => backend.llamadas.filter((l) => l.cmd === cmd);

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

describe("arranque", () => {
  it("resuelve las imagenes de fichas desde rutas del directorio de datos", async () => {
    const partida = await nuevoStore();
    const { urlDeImagen } = await import("$lib/imagenes");
    expect(urlDeImagen("fichas/orco.webp")).toBe("asset:///datos/fichas/orco.webp");
    expect(urlDeImagen("C:\\datos\\fichas\\orco.webp")).toBe("asset://C:\\datos\\fichas\\orco.webp");
    expect(urlDeImagen("https://ejemplo.test/orco.webp")).toBe("https://ejemplo.test/orco.webp");
    expect(partida.zonas).toHaveLength(3);
  });

  it("abre la primera zona, su primer mapa, y toma la rejilla de ese mapa", async () => {
    backend = crearBackend();
    backend.mapas[0].rejilla = 70;
    backend.mapas[0].color_opacidad = [255, 0, 0, 128];
    const partida = await (async () => {
      vi.resetModules();
      vi.doMock("@tauri-apps/api/core", () => ({ invoke: (c: string, a?: Args) => backend.invoke(c, a), convertFileSrc: (p: string) => p }));
      const { partida } = await import("$lib/partida.svelte");
      await partida.iniciar();
      return partida;
    })();
    expect(partida.zonaId).toBe(1);
    expect(partida.mapaActual?.id).toBe(1);
    expect(partida.rejilla.tamano).toBe(70);
    expect(partida.rejilla.color).toBe("#ff0000");
    expect(partida.banco).toHaveLength(2);
    expect(partida.mapasTodasZonas.map((m) => m.id).sort()).toEqual([1, 2, 3, 4]);
  });

  it("restaura si la rejilla está visible desde el mapa guardado", async () => {
    backend = crearBackend();
    backend.mapas[0].rejilla_visible = false;
    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      invoke: (c: string, a?: Args) => backend.invoke(c, a),
      convertFileSrc: (p: string) => p
    }));
    const { partida } = await import("$lib/partida.svelte");
    await partida.iniciar();
    expect(partida.rejilla.visible).toBe(false);
  });
});

describe("punto 4: borrar la zona seleccionada", () => {
  it("pasa a la zona vecina y muestra SUS mapas (antes se quedaba con los de la zona borrada)", async () => {
    const partida = await nuevoStore();
    await partida.borrarZona(partida.zonas.find((z) => z.id === 1)!);

    expect(partida.zonaId).toBe(2);
    expect(partida.mapas.map((m) => m.id)).toEqual([3]);
    expect(partida.mapaActual?.id).toBe(3);
    expect(partida.zonas.map((z) => z.id)).toEqual([2, 3]);
  });

  it("recarga las fichas y el banco, que Rust modifica al borrar la zona", async () => {
    const partida = await nuevoStore();
    await partida.borrarZona(partida.zonas.find((z) => z.id === 1)!);
    expect(partida.instancias.map((i) => i.id_instancia)).toEqual([2]); // la del mapa 1 ya no está
    expect(partida.banco.map((e) => e.nombre)).toEqual(["Lobo"]);
  });

  it("si era la última zona, se queda sin zona ni mapa, y no permite crear mapas", async () => {
    const partida = await nuevoStore();
    for (const id of [1, 2, 3]) await partida.borrarZona(partida.zonas.find((z) => z.id === id)!);

    expect(partida.zonaId).toBeNull();
    expect(partida.mapas).toEqual([]);
    expect(partida.mapaActual).toBeNull();

    await partida.crearMapa(null);
    expect(llamadas("crear_mapa")).toHaveLength(0);
  });

  it("al borrar la zona del final, pasa a la anterior", async () => {
    const partida = await nuevoStore();
    await partida.seleccionarZona(3);
    await partida.borrarZona(partida.zonas.find((z) => z.id === 3)!);
    expect(partida.zonaId).toBe(2);
  });

  it("borrar OTRA zona no mueve la selección ni el mapa abierto", async () => {
    const partida = await nuevoStore();
    await partida.borrarZona(partida.zonas.find((z) => z.id === 3)!);
    expect(partida.zonaId).toBe(1);
    expect(partida.mapaActual?.id).toBe(1);
  });

  it("descarta los cambios de rejilla pendientes del mapa que desaparece (no escribe un mapa borrado)", async () => {
    const partida = await nuevoStore();
    partida.rejilla.tamano = 120;
    partida.marcarRejillaModificada();

    await partida.borrarZona(partida.zonas.find((z) => z.id === 1)!);
    await vi.advanceTimersByTimeAsync(10_000);

    expect(llamadas("modificar_mapa")).toHaveLength(0);
  });
});

describe("rejilla y cambios de mapa", () => {
  it("al cambiar de mapa guarda primero la rejilla pendiente del que se abandona", async () => {
    const partida = await nuevoStore();
    partida.rejilla.tamano = 90;
    partida.marcarRejillaModificada();

    await partida.seleccionarMapa(partida.mapas.find((m) => m.id === 2)!);

    expect(backend.mapas.find((m) => m.id === 1)!.rejilla).toBe(90);
    expect(partida.mapaActual?.id).toBe(2);
    expect(partida.rejilla.tamano).toBe(50); // la del mapa 2, no la del 1
  });

  it("un campo vacío o fuera de rango no llega a Rust como 0: se conserva el valor anterior", async () => {
    const partida = await nuevoStore();
    (partida.rejilla as any).tamano = null;
    partida.marcarRejillaModificada();
    await partida.guardarRejilla();
    expect(backend.mapas.find((m) => m.id === 1)!.rejilla).toBe(50);
  });

  it("cambiar la imagen con rejilla pendiente no pisa ninguno de los dos cambios", async () => {
    const partida = await nuevoStore();
    partida.rejilla.tamano = 80;
    partida.marcarRejillaModificada();

    await partida.cambiarImagenMapa(partida.mapas.find((m) => m.id === 1)!, "/home/yo/mapa.png");

    const final = backend.mapas.find((m) => m.id === 1)!;
    expect(final.url_imagen_fondo).toBe("imagenes/mapa.png");
    expect(final.rejilla).toBe(80);
    expect(partida.mapaActual?.url_imagen_fondo).toBe("imagenes/mapa.png");
  });

  it("renombrar un mapa tampoco pierde la rejilla pendiente", async () => {
    const partida = await nuevoStore();
    partida.rejilla.grosor = 4;
    partida.marcarRejillaModificada();
    await partida.renombrarMapa(partida.mapas.find((m) => m.id === 1)!, "Taberna");
    const final = backend.mapas.find((m) => m.id === 1)!;
    expect([final.nombre, final.grosor]).toEqual(["Taberna", 4]);
  });

  it("la rejilla se guarda sola 5 s después del último cambio", async () => {
    const partida = await nuevoStore();
    partida.rejilla.tamano = 65;
    partida.marcarRejillaModificada();
    await vi.advanceTimersByTimeAsync(4_900);
    expect(llamadas("modificar_mapa")).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(200);
    expect(llamadas("modificar_mapa")).toHaveLength(1);
    expect(partida.mapaActual?.rejilla).toBe(65);
  });
});

describe("mapas y reordenación", () => {
  it("borrar el mapa abierto abre el siguiente; si era el último, el anterior", async () => {
    const partida = await nuevoStore();
    await partida.borrarMapa(partida.mapas.find((m) => m.id === 1)!);
    expect(partida.mapaActual?.id).toBe(2);
    await partida.borrarMapa(partida.mapas.find((m) => m.id === 2)!);
    expect(partida.mapaActual).toBeNull(); // la zona 1 se quedó sin mapas
  });

  it("borrar un mapa que no es el abierto no cambia el abierto", async () => {
    const partida = await nuevoStore();
    await partida.borrarMapa(partida.mapas.find((m) => m.id === 2)!);
    expect(partida.mapaActual?.id).toBe(1);
  });

  it("crear un mapa lo abre", async () => {
    const partida = await nuevoStore();
    await partida.crearMapa("/img/fondo.png");
    expect(partida.mapaActual?.url_imagen_fondo).toBe("imagenes/fondo.png");
    expect(partida.mapaActual?.id_zona).toBe(1);
  });

  it("reordenar zonas solo escribe las que cambian de posición", async () => {
    const partida = await nuevoStore();
    await partida.reordenarZonas([3, 1, 2]);
    expect(partida.zonasOrdenadas.map((z) => z.id)).toEqual([3, 1, 2]);
    expect(llamadas("modificar_zona")).toHaveLength(3);

    backend.llamadas.length = 0;
    await partida.reordenarZonas([3, 1, 2]); // sin cambios
    expect(llamadas("modificar_zona")).toHaveLength(0);
  });
});

describe("fichas: selección y vida", () => {
  it("seleccionar una ficha la expone como instanciaSeleccionada", async () => {
    const partida = await nuevoStore();
    partida.seleccionarInstancia(partida.instancias[1]);
    expect(partida.idInstanciaSeleccionada).toBe(2);
    expect(partida.instanciaSeleccionada?.base.nombre).toBe("Lobo");
    partida.seleccionarInstancia(null);
    expect(partida.instanciaSeleccionada).toBeNull();
  });

  it("guarda, recarga y elimina las flechas de una ficha mediante Rust", async () => {
    const partida = await nuevoStore();
    const puntos = [{ x: 0.5, y: 0.5 }, { x: 1.5, y: 2.5 }];

    await partida.guardarFlecha(1, puntos);
    expect(partida.flechas).toEqual([{ id_instancia: 1, id_mapa: 1, puntos }]);
    expect(llamadas("guardar_flecha")[0]?.args).toEqual({
      instanciaId: 1,
      mapaId: 1,
      puntos
    });

    await partida.seleccionarMapa(partida.mapas.find((mapa) => mapa.id === 2)!);
    expect(partida.flechas).toEqual([]);
    await partida.seleccionarMapa(partida.mapas.find((mapa) => mapa.id === 1)!);
    expect(partida.flechas).toEqual([{ id_instancia: 1, id_mapa: 1, puntos }]);

    await partida.quitarFlecha(1);
    expect(partida.flechas).toEqual([]);
    expect(llamadas("quitar_flecha")[0]?.args).toEqual({ instanciaId: 1, mapaId: 1 });
  });

  it("si Rust elimina la ficha seleccionada (al borrar su zona), la selección desaparece sola", async () => {
    const partida = await nuevoStore();
    partida.seleccionarInstancia(partida.instancias[0]); // ficha del mapa 1, zona 1
    await partida.borrarZona(partida.zonas.find((z) => z.id === 1)!);
    expect(partida.instanciaSeleccionada).toBeNull();
  });

  it("modificarVida persiste la vida y se refleja en la seleccionada", async () => {
    const partida = await nuevoStore();
    partida.seleccionarInstancia(partida.instancias[0]);
    await partida.modificarVida(1, -3);
    expect(partida.instanciaSeleccionada?.vida_actual).toBe(7);
    await partida.modificarVida(99, 5);
    expect(partida.instancias.map((i) => i.vida_actual)).toEqual([7, 5]);
    expect(llamadas("modificar_vida")).toHaveLength(1);
  });

  it("crea, edita, coloca, mueve y retira fichas a través de los comandos Rust", async () => {
    const partida = await nuevoStore();

    await partida.crearEntidad({
      nombre: "Goblin", tipo: "Enemigo", vidaMax: 7, armadura: 2, velocidad: 5, rangoArma: 1
    });
    const goblin = partida.banco.find((e) => e.nombre === "Goblin")!;
    expect(goblin.zona_id).toBe(1);

    await partida.actualizarEntidad({ ...goblin, nombre: "Goblin explorador" }, null, false);
    const actualizada = partida.banco.find((e) => e.id === goblin.id)!;
    expect(actualizada.nombre).toBe("Goblin explorador");

    await partida.colocarEntidad(actualizada.id, 3, 4);
    const colocada = partida.instancias.find((i) => i.base.id === goblin.id)!;
    expect(colocada.coordenadas).toEqual({ Localizacion: [3, 4] });
    expect(partida.idInstanciaSeleccionada).toBe(colocada.id_instancia);

    await partida.moverInstancia(colocada.id_instancia, 5, 6);
    expect(partida.instancias.find((i) => i.id_instancia === colocada.id_instancia)?.coordenadas)
      .toEqual({ Localizacion: [5, 6] });

    await partida.retirarInstancia(colocada.id_instancia);
    expect(partida.instancias.some((i) => i.id_instancia === colocada.id_instancia)).toBe(false);
    expect(partida.idInstanciaSeleccionada).toBeNull();
    expect(llamadas("retirar_instancia")[0]?.args.instanciaId).toBe(colocada.id_instancia);
    expect(llamadas("crear_entidad")).toHaveLength(1);
    expect(llamadas("actualizar_entidad")).toHaveLength(1);
    expect(llamadas("colocar_entidad")).toHaveLength(1);
    expect(llamadas("mover_instancia")).toHaveLength(1);
    expect(llamadas("retirar_instancia")).toHaveLength(1);
  });
});
