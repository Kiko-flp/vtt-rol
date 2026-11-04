<!-- src/routes/componentes/Tablero.svelte -->
<script>
  import { onMount, untrack, onDestroy } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';

  let {
    mapa = { ancho: 1000, alto: 1000, url_imagen_fondo: '' },
    rejilla = { visible: true, tamano: 40, color: '#ffffff', grosor: 1, opacidad: 0.3 }
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
  let tokenCarga = 0;

  let panActivo = $state(false);
  let ultimo = { x: 0, y: 0 };

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
    untrack(() => cargarFondo(url));
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

  function cargarFondo(url) {
    // Si la URL es la misma Y la imagen ya está en memoria, la reutilizamos (0% CPU, 0 MB extra)
    if (url === urlActual && imagenFondo) {
      ajustarVista();
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

    img.src = url.startsWith('http') || url.startsWith('data:') ? url : convertFileSrc(url);
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

  // Por ahora cualquier arrastre con botón izquierdo o central mueve la vista.
  // Cuando haya fichas: solo empezar el pan si el clic no cae sobre una ficha.
  function alPresionar(e) {
    if (e.button !== 0 && e.button !== 1) return;
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
        opacidad: rejilla.opacidad
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

    const t = Number(estilo.tamano);
    if (!(t > 0)) return;
    const paso = t * cam.zoom;
    if (paso < PASO_MIN_PX) return;

    // Rectángulo del mapa en pantalla, recortado a lo que se ve.
    const izq = cam.x;
    const arriba = cam.y;
    const x0 = Math.max(izq, 0);
    const x1 = Math.min(izq + mundoW * cam.zoom, vistaW);
    const y0 = Math.max(arriba, 0);
    const y1 = Math.min(arriba + mundoH * cam.zoom, vistaH);
    if (x0 >= x1 || y0 >= y1) return;

    // Con grosor impar, la línea va al centro del píxel para que no salga borrosa.
    const medio = Math.round(grosor) % 2 === 1 ? 0.5 : 0;

    ctx.save();
    ctx.strokeStyle = estilo.color;
    ctx.lineWidth = grosor;
    ctx.globalAlpha = opacidad;
    ctx.beginPath();

    // Índices de la primera y la última línea visibles en cada eje.
    const iMin = Math.ceil((x0 - izq) / paso);
    const iMax = Math.floor((x1 - izq) / paso);
    for (let i = iMin; i <= iMax; i++) {
      const x = Math.round(izq + i * paso) + medio;
      ctx.moveTo(x, y0);
      ctx.lineTo(x, y1);
    }

    const jMin = Math.ceil((y0 - arriba) / paso);
    const jMax = Math.floor((y1 - arriba) / paso);
    for (let j = jMin; j <= jMax; j++) {
      const y = Math.round(arriba + j * paso) + medio;
      ctx.moveTo(x0, y);
      ctx.lineTo(x1, y);
    }

    ctx.stroke();
    ctx.restore();
  }

  onMount(() => {
    const observador = new ResizeObserver(([entrada]) => {
      const primera = !ancho;
      dpr = window.devicePixelRatio || 1;
      ancho = Math.floor(entrada.contentRect.width);
      alto = Math.floor(entrada.contentRect.height);
      if (primera) ajustarVista();
    });
    observador.observe(contenedor);

    return () => {
      observador.disconnect();
      cancelAnimationFrame(rafId);
    };
  });
</script>

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
  <canvas bind:this={canvasFondo}></canvas>
  <canvas bind:this={canvasRejilla}></canvas>
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
</style>