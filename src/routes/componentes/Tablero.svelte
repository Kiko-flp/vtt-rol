<!-- src/routes/componentes/Tablero.svelte -->
<script>
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let mapa = { ancho: 1000, alto: 1000, url_imagen_fondo: '' };
  export let rejilla = { visible: true, tamano: 40, color: '#ffffff', grosor: 1, opacidad: 0.3 };

  // Por debajo de este tamaño en pantalla (px) la rejilla no se dibuja: no se ve y solo molesta.
  const PASO_MIN_PX = 5;
  const ZOOM_MIN = 0.1;
  const ZOOM_MAX = 8;

  let contenedor;
  let canvasFondo;
  let canvasRejilla;

  // Tamaño visible del tablero (px CSS) y densidad de píxeles de la pantalla.
  let ancho = 0;
  let alto = 0;
  let dpr = 1;

  // Cámara: pantalla = mundo * zoom + (x, y). El "mundo" es el mapa en sus píxeles originales.
  let cam = { x: 0, y: 0, zoom: 1 };

  let imagenFondo = null;
  let urlActual = null;
  let tokenCarga = 0;

  let panActivo = false;
  let ultimo = { x: 0, y: 0 };

  const clamp = (v, min, max) => Math.min(max, Math.max(min, v));

  // Tamaño del mapa en unidades de mundo: el de la imagen si la hay, si no el del propio mapa.
  function dimsMundo() {
    if (imagenFondo) return { w: imagenFondo.naturalWidth, h: imagenFondo.naturalHeight };
    return { w: mapa?.ancho || 1000, h: mapa?.alto || 1000 };
  }

  // ---------------------------------------------------------------------------
  // Carga del fondo
  // ---------------------------------------------------------------------------
  $: if (mapa) cargarFondo(mapa.url_imagen_fondo || '');

  function cargarFondo(url) {
    if (url === urlActual) {
      ajustarVista(); // mismo fondo, pero puede ser otro mapa con otras dimensiones
      return;
    }
    urlActual = url;
    const mio = ++tokenCarga; // si el usuario cambia de mapa rápido, ignoramos cargas viejas

    if (!url) {
      imagenFondo = null;
      ajustarVista();
      return;
    }

    const img = new Image();
    img.onload = () => {
      if (mio !== tokenCarga) return;
      imagenFondo = img;
      ajustarVista();
    };
    img.onerror = () => {
      if (mio !== tokenCarga) return;
      imagenFondo = null;
      ajustarVista();
    };
    img.src = url.startsWith('http') ? url : convertFileSrc(url);
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
    e.preventDefault();
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
  // Pintado: cada capa se redibuja solo cuando cambia lo suyo, y como mucho 1 vez por frame
  // ---------------------------------------------------------------------------
  const pendiente = { fondo: false, rejilla: false };
  let rafId = 0;

  // Los argumentos extra solo sirven para que Svelte sepa de qué depende cada capa.
  $: pedir('fondo', imagenFondo, mapa, cam, ancho, alto);
  $: pedir('rejilla', rejilla, imagenFondo, mapa, cam, ancho, alto);

  function pedir(capa, ..._dependencias) {
    pendiente[capa] = true;
    if (!rafId) rafId = requestAnimationFrame(pintar);
  }

  function pintar() {
    rafId = 0;
    if (!canvasFondo || !canvasRejilla || !ancho || !alto) return;

    if (pendiente.fondo) {
      pendiente.fondo = false;
      pintarFondo();
    }
    if (pendiente.rejilla) {
      pendiente.rejilla = false;
      const ctx = prepararCanvas(canvasRejilla);
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0); // la rejilla se dibuja en px de pantalla
      const { w, h } = dimsMundo();
      dibujarRejilla(ctx, cam, w, h, ancho, alto, rejilla);
    }
  }

  // Ajusta el tamaño real del canvas a la pantalla (nítido en HiDPI) y lo limpia.
  function prepararCanvas(canvas) {
    const w = Math.round(ancho * dpr);
    const h = Math.round(alto * dpr);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    const ctx = canvas.getContext('2d');
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    return ctx;
  }

  function pintarFondo() {
    const ctx = prepararCanvas(canvasFondo);
    const { w, h } = dimsMundo();
    ctx.setTransform(dpr * cam.zoom, 0, 0, dpr * cam.zoom, dpr * cam.x, dpr * cam.y);
    if (imagenFondo) {
      ctx.imageSmoothingQuality = 'high';
      ctx.drawImage(imagenFondo, 0, 0, w, h);
    } else {
      ctx.fillStyle = '#1e1e1e';
      ctx.fillRect(0, 0, w, h);
    }
  }

  /**
   * Dibuja la rejilla en coordenadas de pantalla, solo dentro del mapa y solo las líneas visibles.
   * Coste: nº de líneas en pantalla (nunca más de ~ancho/PASO_MIN_PX por eje), da igual el tamaño del mapa.
   */
  function dibujarRejilla(ctx, cam, mundoW, mundoH, vistaW, vistaH, estilo) {
    if (!estilo || estilo.visible === false) return;

    const t = Number(estilo.tamano);
    if (!(t > 0)) return; // vacío o inválido mientras el usuario escribe: no hacemos nada
    const paso = t * cam.zoom; // tamaño de celda en pantalla
    if (paso < PASO_MIN_PX) return;

    const grosor = Math.max(0.1, Number(estilo.grosor) || 1);
    const opacidad = clamp(Number(estilo.opacidad) || 0, 0, 1);

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
      dpr = window.devicePixelRatio || 1;
      const primera = !ancho;
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
  role="application"
  aria-label="Tablero de juego"
  class:arrastrando={panActivo}
  bind:this={contenedor}
  on:wheel|nonpassive={alRueda}
  on:pointerdown={alPresionar}
  on:pointermove={alMover}
  on:pointerup={alSoltar}
  on:pointercancel={alSoltar}
  on:dblclick={() => ajustarVista()}
  title="Rueda: zoom · Arrastrar: mover · Doble clic: centrar"
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
