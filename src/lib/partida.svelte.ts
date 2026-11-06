// src/lib/partida.svelte.ts
//
// Estado compartido de la partida en el front: zonas, mapas, mapa abierto, rejilla, banco de
// entidades e instancias. Los componentes lo importan y lo leen directamente
// (`partida.zonas`, `partida.mapaActual`...) y llaman a sus métodos para cambiarlo; ya no hay
// props ni callbacks (`onRecargarZonas`...) que atravesar de padre a hijo.
//
// Rust sigue siendo la fuente de verdad: cada método escribe en Rust y vuelve a leer.
import { invoke } from "@tauri-apps/api/core";
import { iniciarImagenes, importarImagen } from "$lib/imagenes";

export interface Zona {
  id: number;
  nombre: string;
  orden: number;
}

export interface Mapa {
  id: number;
  id_zona: number;
  nombre: string;
  url_imagen_fondo: string | null; // ruta relativa devuelta por importar_imagen
  ancho: number;
  alto: number;
  rejilla: number;
  rejilla_visible?: boolean;
  color_opacidad: number[]; // [r, g, b, a] con a en 0..255
  grosor: number;
  orden: number;
}

export type TipoEntidad = "Enemigo" | "NPC" | "Jugador";
export type Cuerpo = "Pequenio" | { Grande: number[] };

export interface Entidad {
  id: number;
  nombre: string;
  tipo: TipoEntidad;
  vida_max: number;
  armadura: number;
  velocidad: number;
  rango_arma: number;
  cuerpo: Cuerpo;
  zona_id: number;
  foto_url?: string | null;
  vida_es_publica?: boolean;
  armadura_es_publica?: boolean;
  archivos_adjuntos?: (string | null)[];
  es_unico?: boolean;
}

export interface Instancia {
  id_instancia: number;
  id_mapa: number | null;
  base: Entidad;
  vida_actual: number;
  coordenadas: "NoPuesto" | { Localizacion: number[] };
  activo?: boolean;
}

export interface PuntoFlecha {
  x: number;
  y: number;
}

export interface Flecha {
  id_instancia: number;
  id_mapa: number;
  puntos: PuntoFlecha[];
}

export interface Rejilla {
  visible: boolean;
  tamano: number;
  color: string; // "#rrggbb"
  grosor: number;
  opacidad: number; // 0..1
}

export const LIMITES_REJILLA = { tamanoMin: 5, tamanoMax: 300, grosorMin: 0, grosorMax: 10 } as const;

// Los cambios de rejilla se agrupan: se guardan 5 s después del último, o antes si hace falta.
const ESPERA_GUARDADO_MS = 5000;

const porOrden = (a: { orden?: number }, b: { orden?: number }) => (a.orden ?? 0) - (b.orden ?? 0);

function hexARgba(hex: string, opacidad: number): number[] {
  const limpio = hex.replace("#", "");
  const completo = limpio.length === 3 ? limpio.split("").map((x) => x + x).join("") : limpio;
  const n = parseInt(completo, 16) || 0;
  const a = Math.round(Math.min(1, Math.max(0, opacidad)) * 255);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255, a];
}

const rgbAHex = (r: number, g: number, b: number) =>
  "#" + [r, g, b].map((v) => Number(v).toString(16).padStart(2, "0")).join("");

/** Elemento que queda al borrar `id` de una lista ordenada: el siguiente, o si no el anterior. */
function vecinoTrasBorrar<T extends { id: number }>(ordenada: T[], id: number): T | null {
  const i = ordenada.findIndex((x) => x.id === id);
  if (i === -1) return null;
  return ordenada[i + 1] ?? ordenada[i - 1] ?? null;
}

class Partida {
  zonas = $state<Zona[]>([]);
  /** Mapas de la zona seleccionada. */
  mapas = $state<Mapa[]>([]);
  /** Mapas de todas las zonas, para identificar la ubicación de cada instancia. */
  mapasTodasZonas = $state<Mapa[]>([]);
  banco = $state<Entidad[]>([]);
  instancias = $state<Instancia[]>([]);
  flechas = $state<Flecha[]>([]);
  entidadEnEdicion = $state<Entidad | null>(null);

  /** Solo se guarda el id: la ficha seleccionada se actualiza sola al recargar las instancias,
   *  y desaparece si Rust la elimina (p. ej. al borrar su zona). */
  idInstanciaSeleccionada = $state<number | null>(null);
  instanciaSeleccionada = $derived(
    this.instancias.find((i) => i.id_instancia === this.idInstanciaSeleccionada) ?? null
  );

  /** `null` = no hay ninguna zona (todavía no se ha creado, o se borró la última). */
  zonaId = $state<number | null>(null);
  mapaActual = $state<Mapa | null>(null);
  rejilla = $state<Rejilla>({ visible: true, tamano: 50, color: "#ffffff", grosor: 1, opacidad: 0.3 });

  zonasOrdenadas = $derived([...this.zonas].sort(porOrden));
  mapasOrdenados = $derived([...this.mapas].sort(porOrden));

  #temporizador: ReturnType<typeof setTimeout> | null = null;
  #pendiente = false;

  // ------------------------------------------------------------------
  // Carga
  // ------------------------------------------------------------------

  async iniciar() {
    await iniciarImagenes(); // hace falta antes de mostrar ningún mapa con imagen
    await this.recargarZonas();
    await this.seleccionarZona(this.zonasOrdenadas[0]?.id ?? null);
    await this.recargarEntidades();
  }

  async recargarZonas() {
    this.zonas = await invoke<Zona[]>("obtener_zonas");
  }

  /**
   * Vuelve a leer los mapas de la zona seleccionada y abre uno.
   * `preferido`: id del mapa a abrir (por defecto, el que ya estaba abierto). Si no existe, el primero.
   */
  async recargarMapas(preferido?: number | null) {
    await this.guardarRejilla(); // que no se pierdan cambios de rejilla en curso al refrescar
    const quiero = preferido === undefined ? (this.mapaActual?.id ?? null) : preferido;

    this.mapasTodasZonas = (
      await Promise.all(
        this.zonas.map((zona) => invoke<Mapa[]>("obtener_mapas_zona", { zonaId: zona.id }))
      )
    ).flat();

    if (this.zonaId === null) {
      this.mapas = [];
      await this.#abrirMapa(null);
      return;
    }
    this.mapas = this.mapasTodasZonas.filter((mapa) => mapa.id_zona === this.zonaId);
    const ordenados = this.mapasOrdenados;
    await this.#abrirMapa(ordenados.find((m) => m.id === quiero) ?? ordenados[0] ?? null);
  }

  /** Rust también cambia el banco y las instancias al borrar zonas o mapas: se vuelven a leer. */
  async recargarEntidades() {
    const [banco, instancias] = await Promise.all([
      invoke<Entidad[]>("obtener_banco_entidades"),
      invoke<Instancia[]>("obtener_instancias_activas")
    ]);
    this.banco = banco;
    this.instancias = instancias;
  }

  // ------------------------------------------------------------------
  // Selección
  // ------------------------------------------------------------------

  async seleccionarZona(id: number | null) {
    if (id === this.zonaId) return;
    this.#desenfocar();
    await this.guardarRejilla(); // cambios pendientes del mapa que se abandona
    this.zonaId = id;
    await this.recargarMapas(null);
  }

  async seleccionarMapa(mapa: Mapa | null) {
    if (!mapa || mapa.id === this.mapaActual?.id) return;
    this.#desenfocar();
    await this.guardarRejilla();
    await this.#abrirMapa(mapa);
  }

  // ------------------------------------------------------------------
  // Fichas (instancias activas)
  // ------------------------------------------------------------------

  seleccionarInstancia(instancia: Instancia | null) {
    this.idInstanciaSeleccionada = instancia?.id_instancia ?? null;
  }

  editarEntidad(entidad: Entidad) {
    this.entidadEnEdicion = entidad;
  }

  cerrarEditorEntidad() {
    this.entidadEnEdicion = null;
  }

  async crearEntidad(datos: {
    nombre: string;
    tipo: TipoEntidad;
    vidaMax: number;
    armadura: number;
    velocidad: number;
    rangoArma: number;
  }) {
    if (this.zonaId === null) throw new Error("Crea una zona antes de añadir fichas.");
    await invoke<Entidad>("crear_entidad", {
      ...datos,
      zonaId: this.zonaId
    });
    await this.recargarEntidades();
  }

  async actualizarEntidad(entidad: Entidad, fotoData: string | null, quitarFoto: boolean) {
    await invoke<Entidad>("actualizar_entidad", { entidad, fotoData, quitarFoto });
    await this.recargarEntidades();
    const instanciasModificadas = new Set(
      this.instancias.filter((instancia) => instancia.base.id === entidad.id).map((instancia) => instancia.id_instancia)
    );
    this.flechas = this.flechas.filter((flecha) => !instanciasModificadas.has(flecha.id_instancia));
  }

  async colocarEntidad(entidadId: number, x: number, y: number) {
    if (!this.mapaActual) throw new Error("Selecciona un mapa antes de colocar fichas.");
    const instancia = await invoke<Instancia>("colocar_entidad", {
      entidadId,
      mapaId: this.mapaActual.id,
      x,
      y
    });
    await this.recargarEntidades();
    this.seleccionarInstancia(instancia);
  }

  async moverInstancia(instanciaId: number, x: number, y: number) {
    if (!this.mapaActual) return;
    await invoke("mover_instancia", {
      instanciaId,
      mapaId: this.mapaActual.id,
      x,
      y
    });
    this.flechas = this.flechas.filter((flecha) => flecha.id_instancia !== instanciaId);
    await this.recargarEntidades();
  }

  async guardarFlecha(instanciaId: number, puntos: PuntoFlecha[]) {
    if (!this.mapaActual) throw new Error("Selecciona un mapa antes de crear una flecha.");
    const flecha = await invoke<Flecha>("guardar_flecha", {
      instanciaId,
      mapaId: this.mapaActual.id,
      puntos
    });
    this.flechas = [
      ...this.flechas.filter((actual) => actual.id_instancia !== instanciaId),
      flecha
    ];
  }

  async quitarFlecha(instanciaId: number) {
    if (!this.mapaActual) return;
    await invoke("quitar_flecha", {
      instanciaId,
      mapaId: this.mapaActual.id
    });
    this.flechas = this.flechas.filter((flecha) => flecha.id_instancia !== instanciaId);
  }

  async modificarVida(idInstancia: number, delta: number) {
    if (!this.instancias.some((instancia) => instancia.id_instancia === idInstancia)) return;
    await invoke("modificar_vida", { instanciaId: idInstancia, delta });
    await this.recargarEntidades();
  }

  async retirarInstancia(idInstancia: number) {
    await invoke("retirar_instancia", { instanciaId: idInstancia });
    if (this.idInstanciaSeleccionada === idInstancia) this.idInstanciaSeleccionada = null;
    this.flechas = this.flechas.filter((flecha) => flecha.id_instancia !== idInstancia);
    await this.recargarEntidades();
  }

  // ------------------------------------------------------------------
  // Zonas
  // ------------------------------------------------------------------

  async crearZona() {
    const zona = await invoke<Zona>("crear_zona");
    await this.recargarZonas();
    await this.seleccionarZona(zona.id);
  }

  async renombrarZona(zona: Zona, nombre: string) {
    await invoke("modificar_zona", { zona: { ...zona, nombre } });
    await this.recargarZonas();
  }

  /** `ids` en el orden nuevo. Solo se escriben las zonas cuya posición ha cambiado. */
  async reordenarZonas(ids: number[]) {
    for (let i = 0; i < ids.length; i++) {
      const zona = this.zonas.find((z) => z.id === ids[i]);
      if (zona && zona.orden !== i) {
        await invoke("modificar_zona", { zona: { ...zona, orden: i } });
      }
    }
    await this.recargarZonas();
  }

  async borrarZona(zona: Zona) {
    const eraLaActual = zona.id === this.zonaId;
    const siguiente = vecinoTrasBorrar(this.zonasOrdenadas, zona.id);

    // Los cambios de rejilla pendientes serían de un mapa que va a desaparecer: se descartan.
    if (eraLaActual) this.#descartarRejillaPendiente();

    await invoke("eliminar_zona", { zona });
    await this.recargarZonas();
    await this.recargarEntidades(); // Rust quita/desactiva fichas y entidades de esa zona

    if (eraLaActual) {
      // Pasamos a la zona vecina (o a ninguna si era la última): sin esto la pantalla seguía
      // mostrando los mapas de una zona que ya no existe.
      await this.seleccionarZona(siguiente?.id ?? null);
    }
  }

  // ------------------------------------------------------------------
  // Mapas
  // ------------------------------------------------------------------

  /** `rutaOrigen`: imagen del sistema para el fondo (opcional). Se copia a la carpeta de datos. */
  async crearMapa(rutaOrigen: string | null = null) {
    if (this.zonaId === null) return;

    let referencia: string | null = null;
    if (rutaOrigen) {
      try {
        referencia = await importarImagen(rutaOrigen);
      } catch (err) {
        console.error("No se pudo importar la imagen:", err);
        return;
      }
    }

    const nuevo = await invoke<Mapa>("crear_mapa", { idZona: this.zonaId, urlImagenFondo: referencia });
    await this.recargarMapas(nuevo.id); // abre el mapa nuevo
  }

  async renombrarMapa(mapa: Mapa, nombre: string) {
    await this.#editarMapa(mapa.id, { nombre });
  }

  async cambiarImagenMapa(mapa: Mapa, rutaOrigen: string) {
    let referencia: string;
    try {
      referencia = await importarImagen(rutaOrigen);
    } catch (err) {
      console.error("No se pudo importar la imagen:", err);
      return;
    }
    await this.#editarMapa(mapa.id, { url_imagen_fondo: referencia });
  }

  async reordenarMapas(ids: number[]) {
    await this.guardarRejilla();
    for (let i = 0; i < ids.length; i++) {
      const mapa = this.mapas.find((m) => m.id === ids[i]);
      if (mapa && mapa.orden !== i) {
        await invoke("modificar_mapa", { mapa: { ...mapa, orden: i } });
      }
    }
    await this.recargarMapas();
  }

  async borrarMapa(mapa: Mapa) {
    const eraElActual = mapa.id === this.mapaActual?.id;
    const siguiente = vecinoTrasBorrar(this.mapasOrdenados, mapa.id);

    if (eraElActual) this.#descartarRejillaPendiente();

    await invoke("borrar_mapa", { mapa });
    await this.recargarMapas(eraElActual ? (siguiente?.id ?? null) : undefined);
    await this.recargarEntidades(); // las fichas que estaban en ese mapa cambian
  }

  /**
   * Cambia campos de un mapa. Primero guarda la rejilla pendiente y parte de la copia MÁS
   * RECIENTE del mapa, para no pisar con datos viejos lo que acaba de guardarse.
   */
  async #editarMapa(id: number, cambios: Partial<Mapa>) {
    await this.guardarRejilla();
    const fresco = this.mapas.find((m) => m.id === id);
    if (!fresco) return;
    await invoke("modificar_mapa", { mapa: { ...fresco, ...cambios } });
    await this.recargarMapas();
  }

  // ------------------------------------------------------------------
  // Rejilla del mapa abierto (la barra de herramientas llama a estos dos)
  // ------------------------------------------------------------------

  marcarRejillaModificada() {
    if (!this.mapaActual) return;
    this.#pendiente = true;
    if (this.#temporizador) clearTimeout(this.#temporizador);
    this.#temporizador = setTimeout(() => void this.guardarRejilla(), ESPERA_GUARDADO_MS);
  }

  /** Guarda ya los cambios de rejilla pendientes (si los hay). */
  async guardarRejilla() {
    const base = this.mapaActual;
    if (!this.#pendiente || !base?.id) return;
    this.#descartarRejillaPendiente();

    const { tamanoMin, tamanoMax, grosorMin, grosorMax } = LIMITES_REJILLA;
    const tamano = Math.round(Number(this.rejilla.tamano));
    const grosor = Math.round(Number(this.rejilla.grosor));

    // Un campo vacío o a medias no debe llegar a Rust como 0: se conserva el valor anterior.
    const mapaModificado: Mapa = {
      ...base,
      rejilla: tamano >= tamanoMin && tamano <= tamanoMax ? tamano : base.rejilla,
      rejilla_visible: this.rejilla.visible,
      grosor: grosor >= grosorMin && grosor <= grosorMax ? grosor : base.grosor,
      color_opacidad: hexARgba(this.rejilla.color, this.rejilla.opacidad)
    };

    try {
      await invoke("modificar_mapa", { mapa: mapaModificado });
    } catch (err) {
      console.error("Error al persistir cambios de rejilla en Rust:", err);
      return;
    }

    if (this.mapaActual?.id === mapaModificado.id) this.mapaActual = mapaModificado;
    const i = this.mapas.findIndex((m) => m.id === mapaModificado.id);
    if (i !== -1) this.mapas[i] = mapaModificado;
  }

  // ------------------------------------------------------------------
  // Internos
  // ------------------------------------------------------------------

  async #abrirMapa(mapa: Mapa | null) {
    this.mapaActual = mapa;
    if (!mapa) {
      this.flechas = [];
      return;
    }

    this.rejilla.tamano = mapa.rejilla;
    this.rejilla.visible = mapa.rejilla_visible ?? true;
    this.rejilla.grosor = mapa.grosor;
    if (Array.isArray(mapa.color_opacidad) && mapa.color_opacidad.length >= 4) {
      const [r, g, b, a] = mapa.color_opacidad;
      this.rejilla.color = rgbAHex(r, g, b);
      this.rejilla.opacidad = Number(a) / 255;
    }
    await this.recargarFlechas(mapa.id);
  }

  async recargarFlechas(mapaId: number) {
    const flechas = await invoke<Flecha[]>("obtener_flechas", { idMapa: mapaId });
    if (this.mapaActual?.id === mapaId) this.flechas = flechas;
  }

  #descartarRejillaPendiente() {
    if (this.#temporizador) clearTimeout(this.#temporizador);
    this.#temporizador = null;
    this.#pendiente = false;
  }

  /** Quita el foco de un input en edición para que dispare su `change` antes de guardar. */
  #desenfocar() {
    if (typeof document !== "undefined" && document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
  }
}

export const partida = new Partida();
