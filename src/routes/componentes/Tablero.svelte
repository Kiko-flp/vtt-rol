<!-- src/routes/componentes/Tablero.svelte -->
<script>
  import { onMount, untrack, onDestroy } from 'svelte';
  import { urlDeImagen } from '$lib/imagenes';
  import { trayectoriaRectaFlecha } from '$lib/flechas';
  import { distribuirRejilla } from '$lib/rejilla';
  import FlechasTablero from './FlechasTablero.svelte';

  /**
   * @typedef {import("$lib/partida.svelte").Mapa} Mapa
   * @typedef {import("$lib/partida.svelte").Rejilla} Rejilla
   * @typedef {import("$lib/partida.svelte").Instancia} Instancia
   * @typedef {import("$lib/partida.svelte").Entidad} Entidad
   */
  /** @type {{
   * mapa: Mapa | null,
   * rejilla: Rejilla,
   * instancias?: Instancia[],
   * instanciaSeleccionada?: Instancia | null,
   * flechas?: import("$lib/partida.svelte").Flecha[],
   * onSeleccionarInstancia?: (instancia: Instancia) => void,
   * onColocarEntidad?: (entidadId: number, x: number, y: number) => void | Promise<void>,
   * onMoverInstancia?: (instanciaId: number, x: number, y: number) => void | Promise<void>,
   * onGuardarFlecha?: (instanciaId: number, puntos: import("$lib/partida.svelte").PuntoFlecha[]) => Promise<void>,
   * onQuitarFlecha?: (instanciaId: number) => Promise<void>,
   * onEditarEntidad?: (entidad: Entidad) => void,
   * onRetirarInstancia?: (instancia: Instancia) => void | Promise<void>
   * }} */
  let {
    mapa = null,
    rejilla,
    instancias = [],
    instanciaSeleccionada = null,
    flechas = [],
    onSeleccionarInstancia,
    onColocarEntidad,
    onMoverInstancia,
    onGuardarFlecha,
    onQuitarFlecha,
    onEditarEntidad,
    onRetirarInstancia
  } = $props();

  // Por debajo de este tamaño en pantalla (px) la rejilla no se dibuja: no se ve y solo molesta.
  const PASO_MIN_PX = 5;
  const ZOOM_MIN = 0.1;
  const ZOOM_MAX = 8;

  // Elementos del DOM (solo se usan desde funciones, no hace falta que sean reactivos)
  let contenedor;
  let canvasFondo;
  let canvasRejilla;

  // Tamaño visible del tablero (px CSS) y densidad de píxeles de la pantalla.
  let ancho = $state(0);
  let alto = $state(0);
  let dpr = $state(1);

  // Cámara: pantalla = mundo * zoom + (x, y). El "mundo" es el mapa en sus píxeles originales.
  // $state.raw: no se convierte en proxy; se actualiza sustituyendo el objeto entero.
  let cam = $state.raw({ x: 0, y: 0, zoom: 1 });

  // Un <img> del DOM no debe volverse proxy: $state.raw.
  let imagenFondo = $state.raw(null);
  let urlActual = null;
  /** @type {number | null} */
  let idActual = null; // id del mapa mostrado
  let tokenCarga = 0;

  let panActivo = $state(false);
  let errorFicha = $state("");
  let flechaBorrador = $state(null);
  let ultimo = { x: 0, y: 0 };
  /** @type {null | { id: number, x: number, y: number, inicioX: number, inicioY: number, offsetX: number, offsetY: number, movida: boolean, pointerId: number }} */
  let arrastreFicha = $state(null);
  let menuFicha = $state(null);
  let fichasEnMapa = $derived(
    instancias.filter((instancia) =>
      instancia.activo &&
      instancia.id_mapa === mapa?.id &&
      Array.isArray(instancia.coordenadas?.Localizacion)
    )
  );

  const clamp = (v, min, max) => Math.min(max, Math.max(min, v));

  // Tamaño del mapa en unidades de mundo: el de la imagen si la hay, si no el del propio mapa.
  function dimsMundo() {
    if (imagenFondo) return { w: imagenFondo.naturalWidth, h: imagenFondo.naturalHeight };
    return { w: mapa?.ancho || 1000, h: mapa?.alto || 1000 };
  }
  // ---------------------------------------------------------------------------
  // Carga del fondo: se repite cuando cambia el mapa
  // ---------------------------------------------------------------------------
  $effect(() => {
    const url = mapa?.url_imagen_fondo || '';
    const id = mapa?.id ?? null;
    untrack(() => cargarFondo(url, id));
  });

  // La memoria solo se destruye al cerrar el tablero, no en cada cambio de mapa
  onDestroy(() => {
    liberarMemoriaImagen();
  });

  function liberarMemoriaImagen() {
    if (imagenFondo) {
      // Desvincular eventos primero para evitar bucle infinito de onerror en WebView2
      imagenFondo.onload = null;
      imagenFondo.onerror = null;
      imagenFondo.src = '';
      imagenFondo = null;
    }
  }

  /**
   * @param {string} url
   * @param {number | null} id
   */
  function cargarFondo(url, id) {
    const cambioDeMapa = id !== idActual;
    idActual = id;

    // Mismo fondo (ya cargado, o sin imagen): no se recarga nada. La cámara solo se recentra si es
    // OTRO mapa; si no, guardar la rejilla o renombrar mapas la devolvería al centro cada vez.
    if (url === urlActual && (imagenFondo || !url)) {
      if (cambioDeMapa) ajustarVista();
      return;
    }

    urlActual = url;
    const mio = ++tokenCarga;

    // Liberamos la imagen anterior limpiando sus listeners
    liberarMemoriaImagen();

    if (!url) {
      ajustarVista();
      return;
    }

    const img = new Image();
    img.decoding = 'async';

    img.onload = async () => {
      if (mio !== tokenCarga) {
        img.onload = null;
        img.onerror = null;
        img.src = '';
        return;
      }
      try {
        await img.decode();
      } catch {
        // Ignorar si decode() ya terminó
      }
      if (mio !== tokenCarga) {
        img.onload = null;
        img.onerror = null;
        img.src = '';
        return;
      }
      imagenFondo = img;
      ajustarVista();
    };

    img.onerror = () => {
      if (mio !== tokenCarga) return;
      img.onload = null;
      img.onerror = null;
      imagenFondo = null;
      ajustarVista();
    };

    img.src = urlDeImagen(url); // `url` es la ruta relativa guardada en el mapa
  }

  // ---------------------------------------------------------------------------
  // Cámara
  // ---------------------------------------------------------------------------
  function ajustarVista() {
    if (!ancho || !alto) return;
    const { w, h } = dimsMundo();
    const zoom = clamp(Math.min(ancho / w, alto / h) * 0.95, ZOOM_MIN, 1);
    cam = { zoom, x: (ancho - w * zoom) / 2, y: (alto - h * zoom) / 2 };
  }

  function alRueda(e) {
    e.preventDefault(); // en Svelte 5 onwheel no es pasivo, así que esto funciona
    const r = contenedor.getBoundingClientRect();
    const px = e.clientX - r.left;
    const py = e.clientY - r.top;
    const zoom = clamp(cam.zoom * Math.exp(-e.deltaY * 0.0015), ZOOM_MIN, ZOOM_MAX);
    const k = zoom / cam.zoom;
    // El punto del mundo bajo el cursor se queda bajo el cursor.
    cam = { zoom, x: px - (px - cam.x) * k, y: py - (py - cam.y) * k };
  }

  // Arrastrar una ficha la mueve; arrastrar el fondo desplaza la cámara.
  function alPresionar(e) {
    if (e.button !== 0 && e.button !== 1) return;
    if (e.target instanceof Element && e.target.closest('[data-ficha="true"]')) return;
    e.preventDefault();
    panActivo = true;
    ultimo = { x: e.clientX, y: e.clientY };
    contenedor.setPointerCapture(e.pointerId);
  }

  function alMover(e) {
    if (!panActivo) return;
    cam = { ...cam, x: cam.x + e.clientX - ultimo.x, y: cam.y + e.clientY - ultimo.y };
    ultimo = { x: e.clientX, y: e.clientY };
  }

  function alSoltar(e) {
    panActivo = false;
    if (contenedor?.hasPointerCapture?.(e.pointerId)) contenedor.releasePointerCapture(e.pointerId);
  }

  function distribucionRejilla() {
    const { w, h } = dimsMundo();
    return distribuirRejilla(
      w,
      h,
      Math.max(1, Number(rejilla.tamano) || Number(mapa?.rejilla) || 50)
    );
  }

  function dimensionesFicha(instancia) {
    const grande = instancia.base?.cuerpo?.Grande;
    return Array.isArray(grande) && grande.length >= 2
      ? { columnas: Math.max(1, grande[0]), filas: Math.max(1, grande[1]) }
      : { columnas: 1, filas: 1 };
  }

  function limitarCelda(x, y, instancia = null) {
    const distribucion = distribucionRejilla();
    const { columnas, filas } = instancia ? dimensionesFicha(instancia) : { columnas: 1, filas: 1 };
    return {
      x: clamp(Math.floor(x), 0, Math.max(0, distribucion.columnas - columnas)),
      y: clamp(Math.floor(y), 0, Math.max(0, distribucion.filas - filas))
    };
  }

  function limitarPosicion(x, y, instancia) {
    const distribucion = distribucionRejilla();
    const { columnas, filas } = dimensionesFicha(instancia);
    return {
      x: clamp(Math.round(x), 0, Math.max(0, distribucion.columnas - columnas)),
      y: clamp(Math.round(y), 0, Math.max(0, distribucion.filas - filas))
    };
  }

  function coordenadaDesdePantalla(clientX, clientY) {
    const rect = contenedor.getBoundingClientRect();
    return {
      x: (clientX - rect.left - cam.x) / cam.zoom,
      y: (clientY - rect.top - cam.y) / cam.zoom
    };
  }

  function fuenteImagen(url) {
    return urlDeImagen(url);
  }

  function editarFichaDesdeMenu() {
    const instancia = menuFicha?.instancia;
    menuFicha = null;
    if (instancia) onEditarEntidad?.(instancia.base);
  }

  function cerrarMenuFicha() {
    menuFicha = null;
  }

  function cerrarMenuConEscape(evento) {
    if (evento.key === "Escape") cerrarMenuFicha();
  }

  /**
   * @param {{ entidadId: number, clientX: number, clientY: number }} detalle
   */
  async function colocarEntidadEnPuntero({ entidadId, clientX, clientY }) {
    if (!Number.isInteger(entidadId) || entidadId <= 0) return;
    if (!mapa?.id) {
      errorFicha = "Selecciona un mapa antes de colocar una ficha.";
      return;
    }

    const punto = coordenadaDesdePantalla(clientX, clientY);
    const distribucion = distribucionRejilla();
    const posicion = limitarCelda(punto.x / distribucion.anchoCelda, punto.y / distribucion.altoCelda);
    try {
      errorFicha = "";
      await onColocarEntidad?.(entidadId, posicion.x, posicion.y);
    } catch (error) {
      errorFicha = error instanceof Error ? error.message : String(error);
    }
  }

  /** @param {CustomEvent<{ entidadId: number, clientX: number, clientY: number }>} evento */
  async function alRecibirColocacionEntidad(evento) {
    await colocarEntidadEnPuntero(evento.detail);
  }

  /**
   * @param {PointerEvent & { currentTarget: HTMLButtonElement }} e
   * @param {{ id_instancia: number, coordenadas: { Localizacion: number[] }, base: { cuerpo?: { Grande?: number[] } } }} instancia
   */
  function iniciarArrastreFicha(e, instancia) {
    if (e.button === 2) {
      iniciarApuntarFicha(e, instancia);
      return;
    }
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    onSeleccionarInstancia?.(instancia);

    const inicioX = e.clientX;
    const inicioY = e.clientY;
    const coordenadas = instancia.coordenadas.Localizacion;
    const punto = coordenadaDesdePantalla(inicioX, inicioY);
    const distribucion = distribucionRejilla();
    const x = Math.round(Number(coordenadas[0]) || 0);
    const y = Math.round(Number(coordenadas[1]) || 0);
    const arrastreInicial = {
      id: instancia.id_instancia,
      x,
      y,
      inicioX,
      inicioY,
      offsetX: punto.x - x * distribucion.anchoCelda,
      offsetY: punto.y - y * distribucion.altoCelda,
      movida: false,
      pointerId: e.pointerId
    };

    function quitarEscuchas() {
      window.removeEventListener("pointermove", alMoverFicha);
      window.removeEventListener("pointerup", alSoltarFicha);
      window.removeEventListener("pointercancel", alCancelarFicha);
    }

    /** @param {PointerEvent} evento */
    function alMoverFicha(evento) {
      if (evento.pointerId !== arrastreInicial.pointerId) return;
      if (
        !arrastreInicial.movida &&
        Math.hypot(evento.clientX - inicioX, evento.clientY - inicioY) < 4
      ) {
        return;
      }

      arrastreInicial.movida = true;
      const ficha = fichasEnMapa.find((actual) => actual.id_instancia === arrastreInicial.id);
      if (!ficha) return;
      const posicion = posicionDesdeArrastre(evento, arrastreInicial, ficha);
      arrastreFicha = { ...arrastreInicial, ...posicion };
    }

    /** @param {PointerEvent} evento */
    async function alSoltarFicha(evento) {
      if (evento.pointerId !== arrastreInicial.pointerId) return;
      quitarEscuchas();
      arrastreFicha = null;
      if (!arrastreInicial.movida) {
        if (flechas.some((flecha) => flecha.id_instancia === arrastreInicial.id)) {
          await quitarFlechaDeFicha(arrastreInicial.id);
        }
        return;
      }

      const ficha = fichasEnMapa.find((actual) => actual.id_instancia === arrastreInicial.id);
      if (!ficha) return;
      const posicionLibre = posicionDesdeArrastre(evento, arrastreInicial, ficha);
      const posicionFinal = limitarPosicion(
        posicionLibre.x,
        posicionLibre.y,
        ficha
      );
      try {
        errorFicha = "";
        await onMoverInstancia?.(arrastreInicial.id, posicionFinal.x, posicionFinal.y);
      } catch (error) {
        errorFicha = error instanceof Error ? error.message : String(error);
      }
    }

    /** @param {PointerEvent} evento */
    function alCancelarFicha(evento) {
      if (evento.pointerId !== arrastreInicial.pointerId) return;
      quitarEscuchas();
      arrastreFicha = null;
    }

    window.addEventListener("pointermove", alMoverFicha);
    window.addEventListener("pointerup", alSoltarFicha);
    window.addEventListener("pointercancel", alCancelarFicha);
  }

  async function quitarFlechaDeFicha(instanciaId) {
    try {
      errorFicha = "";
      await onQuitarFlecha?.(instanciaId);
    } catch (error) {
      errorFicha = error instanceof Error ? error.message : String(error);
    }
  }

  function iniciarApuntarFicha(evento, instancia) {
    evento.preventDefault();
    evento.stopPropagation();
    onSeleccionarInstancia?.(instancia);

    const inicioX = evento.clientX;
    const inicioY = evento.clientY;
    const [origenX = 0, origenY = 0] = instancia.coordenadas.Localizacion;
    const dimensiones = dimensionesFicha(instancia);
    const origen = {
      x: origenX + dimensiones.columnas / 2,
      y: origenY + dimensiones.filas / 2
    };
    let puntos = [origen, origen];
    let movida = false;

    function quitarEscuchas() {
      window.removeEventListener("pointermove", alMoverFlecha);
      window.removeEventListener("pointerup", alSoltarFlecha);
      window.removeEventListener("pointercancel", alCancelarFlecha);
    }

    function alMoverFlecha(movimiento) {
      if (movimiento.pointerId !== evento.pointerId) return;
      if (!movida && Math.hypot(movimiento.clientX - inicioX, movimiento.clientY - inicioY) < 4) return;
      movida = true;

      puntos = trayectoriaRectaFlecha(
        origen,
        movimiento.clientX,
        movimiento.clientY,
        contenedor.getBoundingClientRect(),
        cam,
        distribucionRejilla(),
        dimensiones
      );
      flechaBorrador = { id_instancia: instancia.id_instancia, puntos: [...puntos] };
    }

    async function alSoltarFlecha(suelta) {
      if (suelta.pointerId !== evento.pointerId) return;
      suelta.preventDefault();
      quitarEscuchas();
      if (!movida) {
        menuFicha = { x: suelta.clientX, y: suelta.clientY, instancia };
        return;
      }

      puntos = trayectoriaRectaFlecha(
        origen,
        suelta.clientX,
        suelta.clientY,
        contenedor.getBoundingClientRect(),
        cam,
        distribucionRejilla(),
        dimensiones
      );
      if (puntos[0].x === puntos[1].x && puntos[0].y === puntos[1].y) {
        flechaBorrador = null;
        return;
      }
      flechaBorrador = { id_instancia: instancia.id_instancia, puntos: [...puntos] };
      try {
        errorFicha = "";
        await onGuardarFlecha?.(instancia.id_instancia, puntos);
      } catch (error) {
        errorFicha = error instanceof Error ? error.message : String(error);
      } finally {
        flechaBorrador = null;
      }
    }

    function alCancelarFlecha(cancelar) {
      if (cancelar.pointerId !== evento.pointerId) return;
      quitarEscuchas();
      flechaBorrador = null;
    }

    window.addEventListener("pointermove", alMoverFlecha);
    window.addEventListener("pointerup", alSoltarFlecha);
    window.addEventListener("pointercancel", alCancelarFlecha);
  }

  /**
   * @param {PointerEvent} e
   * @param {{ offsetX: number, offsetY: number }} arrastre
   * @param {{ base?: { cuerpo?: { Grande?: number[] } } }} instancia
   */
  function posicionDesdeArrastre(e, arrastre, instancia) {
    const punto = coordenadaDesdePantalla(e.clientX, e.clientY);
    const distribucion = distribucionRejilla();
    return limitarPosicion(
      Math.floor((punto.x - arrastre.offsetX) / distribucion.anchoCelda),
      Math.floor((punto.y - arrastre.offsetY) / distribucion.altoCelda),
      instancia
    );
  }

  function posicionFicha(instancia) {
    const [x = 0, y = 0] = instancia.coordenadas.Localizacion;
    const posicion = arrastreFicha?.id === instancia.id_instancia ? arrastreFicha : { x, y };
    const distribucion = distribucionRejilla();
    const { columnas, filas } = dimensionesFicha(instancia);
    return {
      left: cam.x + Math.round(posicion.x) * distribucion.anchoCelda * cam.zoom,
      top: cam.y + Math.round(posicion.y) * distribucion.altoCelda * cam.zoom,
      width: columnas * distribucion.anchoCelda * cam.zoom,
      height: filas * distribucion.altoCelda * cam.zoom
    };
  }

  // ---------------------------------------------------------------------------
  // Pintado: cada capa se redibuja solo cuando cambia lo suyo, y como mucho 1 vez por frame.
  // Los efectos solo anotan "qué hay que pintar"; el pintado real ocurre en requestAnimationFrame.
  // ---------------------------------------------------------------------------
  const pendiente = { fondo: null, rejilla: null };
  let rafId = 0;

  function programar() {
    if (!rafId) rafId = requestAnimationFrame(pintar);
  }

  // Depende de: imagen, mapa, cámara y tamaño del tablero.
  $effect(() => {
    const { w, h } = dimsMundo();
    pendiente.fondo = { img: imagenFondo, w, h, cam, ancho, alto, dpr };
    programar();
  });

  // Depende de lo anterior (salvo la imagen en sí) y de los ajustes de la rejilla.
  $effect(() => {
    const { w, h } = dimsMundo();
    pendiente.rejilla = {
      w, h, cam, ancho, alto, dpr,
      estilo: {
        visible: rejilla.visible,
        tamano: rejilla.tamano,
        color: rejilla.color,
        grosor: rejilla.grosor,
        opacidad: rejilla.opacidad,
        dpr
      }
    };
    programar();
  });

  function pintar() {
    rafId = 0;
    if (!canvasFondo || !canvasRejilla) return;

    const f = pendiente.fondo;
    if (f && f.ancho && f.alto) {
      pendiente.fondo = null;
      pintarFondo(f);
    }

    const r = pendiente.rejilla;
    if (r && r.ancho && r.alto) {
      pendiente.rejilla = null;
      const ctx = prepararCanvas(canvasRejilla, r.ancho, r.alto, r.dpr);
      ctx.setTransform(r.dpr, 0, 0, r.dpr, 0, 0); // la rejilla se dibuja en px de pantalla
      dibujarRejilla(ctx, r.cam, r.w, r.h, r.ancho, r.alto, r.estilo);
    }
  }

  // Ajusta el tamaño real del canvas a la pantalla (nítido en HiDPI) y lo limpia.
  function prepararCanvas(canvas, anchoCss, altoCss, densidad) {
    const w = Math.round(anchoCss * densidad);
    const h = Math.round(altoCss * densidad);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    const ctx = canvas.getContext('2d');
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    return ctx;
  }

  function pintarFondo(f) {
    const ctx = prepararCanvas(canvasFondo, f.ancho, f.alto, f.dpr);
    ctx.setTransform(f.dpr * f.cam.zoom, 0, 0, f.dpr * f.cam.zoom, f.dpr * f.cam.x, f.dpr * f.cam.y);
    if (f.img) {
      ctx.imageSmoothingQuality = 'high';
      ctx.drawImage(f.img, 0, 0, f.w, f.h);
    } else {
      ctx.fillStyle = '#1e1e1e';
      ctx.fillRect(0, 0, f.w, f.h);
    }
  }

  /**
   * Dibuja la rejilla en coordenadas de pantalla, solo dentro del mapa y solo las líneas visibles.
   * Coste: nº de líneas en pantalla (nunca más de ~ancho/PASO_MIN_PX por eje), da igual el tamaño del mapa.
   */
  function dibujarRejilla(ctx, cam, mundoW, mundoH, vistaW, vistaH, estilo) {
    if (!estilo || estilo.visible === false) return;

    // Si el grosor es 0 o menor, no dibuja la rejilla
    const grosor = Number(estilo.grosor);
    if (!(grosor > 0)) return;

    const opacidad = clamp(Number(estilo.opacidad) || 0, 0, 1);
    if (opacidad <= 0) return;

    const distribucion = distribuirRejilla(mundoW, mundoH, Number(estilo.tamano));
    const pasoX = distribucion.anchoCelda * cam.zoom;
    const pasoY = distribucion.altoCelda * cam.zoom;
    if (
      distribucion.columnas <= 0 ||
      distribucion.filas <= 0 ||
      pasoX < PASO_MIN_PX ||
      pasoY < PASO_MIN_PX
    ) return;

    // Ajusta la separación de las líneas para que la rejilla cierre en el borde del mapa.
    const izq = cam.x;
    const arriba = cam.y;
    const x0 = Math.max(izq, 0);
    const x1 = Math.min(izq + mundoW * cam.zoom, vistaW);
    const y0 = Math.max(arriba, 0);
    const y1 = Math.min(arriba + mundoH * cam.zoom, vistaH);
    if (x0 >= x1 || y0 >= y1) return;

    // Con grosor impar, la línea va al centro del píxel para que no salga borrosa.
    const densidad = Number(estilo.dpr) || 1;
    const medio = Math.round(grosor * densidad) % 2 === 1 ? 0.5 / densidad : 0;

    ctx.save();
    ctx.beginPath();
    ctx.rect(x0, y0, x1 - x0, y1 - y0);
    ctx.clip();
    ctx.strokeStyle = estilo.color;
    ctx.lineWidth = grosor;
    ctx.globalAlpha = opacidad;
    ctx.beginPath();

    // Índices de la primera y la última línea visibles en cada eje.
    const iMin = Math.max(0, Math.ceil((x0 - izq) / pasoX));
    const iMax = Math.min(distribucion.columnas, Math.floor((x1 - izq) / pasoX));
    for (let i = iMin; i <= iMax; i++) {
      const x = izq + i * pasoX + medio;
      ctx.moveTo(x, y0);
      ctx.lineTo(x, y1);
    }

    const jMin = Math.max(0, Math.ceil((y0 - arriba) / pasoY));
    const jMax = Math.min(distribucion.filas, Math.floor((y1 - arriba) / pasoY));
    for (let j = jMin; j <= jMax; j++) {
      const y = arriba + j * pasoY + medio;
      ctx.moveTo(x0, y);
      ctx.lineTo(x1, y);
    }

    ctx.stroke();
    ctx.restore();
  }

  onMount(() => {
    window.addEventListener("vtt-colocar-entidad", alRecibirColocacionEntidad);
    const observador = new ResizeObserver(([entrada]) => {
      const primera = !ancho;
      dpr = window.devicePixelRatio || 1;
      ancho = Math.floor(entrada.contentRect.width);
      alto = Math.floor(entrada.contentRect.height);
      if (primera) ajustarVista();
    });
    observador.observe(contenedor);

    return () => {
      window.removeEventListener("vtt-colocar-entidad", alRecibirColocacionEntidad);
      observador.disconnect();
      cancelAnimationFrame(rafId);
    };
  });
</script>

<svelte:window onpointerdown={cerrarMenuFicha} onkeydown={cerrarMenuConEscape} />

<div
  class="tablero"
  class:arrastrando={panActivo}
  bind:this={contenedor}
  role="application"
  aria-label="Tablero de juego"
  title="Rueda: zoom · Arrastrar: mover · Doble clic: centrar"
  onwheel={alRueda}
  onpointerdown={alPresionar}
  onpointermove={alMover}
  onpointerup={alSoltar}
  onpointercancel={alSoltar}
  ondblclick={() => ajustarVista()}
>
  {#if errorFicha}
    <div class="error-ficha" role="alert">{errorFicha}</div>
  {/if}
  <canvas bind:this={canvasFondo}></canvas>
  <canvas bind:this={canvasRejilla}></canvas>
  {#each fichasEnMapa as instancia (instancia.id_instancia)}
    {@const posicion = posicionFicha(instancia)}
    <button
      type="button"
      class="ficha-tablero {instancia.base.tipo.toLowerCase()}"
      class:seleccionada={instanciaSeleccionada?.id_instancia === instancia.id_instancia}
      data-ficha="true"
      data-instancia-id={instancia.id_instancia}
      title="{instancia.base.nombre} · {instancia.vida_actual}/{instancia.base.vida_max} PV"
      style:left="{posicion.left}px"
      style:top="{posicion.top}px"
      style:width="{posicion.width}px"
      style:height="{posicion.height}px"
      onpointerdown={(e) => iniciarArrastreFicha(e, instancia)}
      oncontextmenu={(e) => {
        e.preventDefault();
        e.stopPropagation();
      }}
      ondblclick={(e) => {
        e.stopPropagation();
        onEditarEntidad?.(instancia.base);
      }}
    >
      <span class="forma-ficha">
        {#if instancia.base.foto_url}
          <img class="foto-ficha" src={fuenteImagen(instancia.base.foto_url)} alt="" draggable="false" />
        {:else}
          <span class="inicial-ficha">{instancia.base.nombre.slice(0, 1).toUpperCase()}</span>
        {/if}
      </span>
      <small>{instancia.vida_actual}</small>
    </button>
  {/each}
  <FlechasTablero
    {flechas}
    borrador={flechaBorrador}
    instancias={fichasEnMapa}
    distribucion={distribucionRejilla()}
    {cam}
  />
  {#if menuFicha}
    <div
      class="menu-ficha"
      style:left="{menuFicha.x}px"
      style:top="{menuFicha.y}px"
      onclick={(evento) => evento.stopPropagation()}
      onpointerdown={(evento) => evento.stopPropagation()}
    >
      <button type="button" onclick={editarFichaDesdeMenu}>Editar datos</button>
      <button type="button" class="peligro" onclick={() => {
        const instancia = menuFicha.instancia;
        menuFicha = null;
        onRetirarInstancia?.(instancia);
      }}>Retirar del mapa</button>
    </div>
  {/if}
</div>

<style>
  .tablero {
    position: relative;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: #121212;
    cursor: grab;
    touch-action: none;
  }
  .error-ficha {
    position: absolute;
    z-index: 20;
    top: 8px;
    left: 8px;
    max-width: min(360px, calc(100% - 32px));
    padding: 8px 10px;
    border: 1px solid #ef4444;
    border-radius: 5px;
    color: #fee2e2;
    background: #7f1d1d;
    pointer-events: none;
  }
  .tablero.arrastrando {
    cursor: grabbing;
  }
  canvas {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none; /* los eventos los gestiona el contenedor */
  }
  .ficha-tablero {
    position: absolute;
    z-index: 2;
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: visible;
    border: 0;
    border-radius: 0;
    padding: 0;
    color: white;
    font: inherit;
    font-size: 11px;
    font-weight: 700;
    text-shadow: 0 1px 2px #000;
    cursor: grab;
    touch-action: none;
    user-select: none;
  }
  .ficha-tablero:active { cursor: grabbing; }
  .forma-ficha {
    position: absolute;
    inset: 0;
    z-index: 0;
    display: grid;
    place-items: center;
    overflow: hidden;
    border: 2px solid #fafafa;
    background: #27272a;
    box-shadow: 0 2px 6px #0009;
  }
  .ficha-tablero.seleccionada { outline: 3px solid #facc15; outline-offset: 2px; }
  .foto-ficha { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
  .inicial-ficha { z-index: 1; font-size: clamp(12px, 2vw, 22px); }
  .ficha-tablero small { position: absolute; z-index: 1; bottom: 1px; min-width: 16px; padding: 1px 3px; border-radius: 8px; background: #09090bd9; font-size: 9px; font-weight: 600; }
  .menu-ficha {
    position: fixed; z-index: 20; display: flex; flex-direction: column; gap: 2px;
    min-width: 150px; padding: 4px; background: #1c1c1f; border: 1px solid #3f3f46;
    border-radius: 5px; box-shadow: 0 6px 18px #0009;
  }
  .menu-ficha button { padding: 6px 8px; border: 0; border-radius: 3px; color: #f4f4f5; background: transparent; text-align: left; cursor: pointer; font: inherit; }
  .menu-ficha button:hover { background: #34343a; }
  .menu-ficha button.peligro:hover { color: white; background: #b91c1c; }
  .menu-ficha button:focus-visible { outline: 2px solid #60a5fa; outline-offset: -2px; }
</style>