<!-- src/routes/componentes/Tablero.svelte -->
<script>
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';

  export let mapa = { ancho: 1000, alto: 1000, rejilla: 40, grosor: 1, url_imagen_fondo: "" };
  export let rejilla;

  let canvas;
  let ctx = null;
  let imagenFondo = null;
  let urlCargadaActualmente = null;

  // 🔄 REACCIÓN AL CAMBIO DE MAPA:
  $: if (mapa && canvas) {
    if (mapa.url_imagen_fondo !== urlCargadaActualmente) {
      urlCargadaActualmente = mapa.url_imagen_fondo;

      if (mapa.url_imagen_fondo) {
        // Tiene imagen nueva -> La cargamos
        cargarImagen(mapa.url_imagen_fondo);
      } else {
        // NO tiene imagen -> BORRAMOS la anterior y volvemos a 1000x1000 gris
        imagenFondo = null;
        canvas.width = mapa.ancho || 1000;
        canvas.height = mapa.alto || 1000;
        dibujar();
      }
    }
  }

  function cargarImagen(url) {
    const rutaFinal = url.startsWith("http") ? url : convertFileSrc(url);
    const img = new Image();
    img.src = rutaFinal;

    img.onload = () => {
      imagenFondo = img;
      canvas.width = img.naturalWidth;
      canvas.height = img.naturalHeight;
      dibujar();
    };

    img.onerror = () => {
      imagenFondo = null;
      canvas.width = mapa.ancho || 1000;
      canvas.height = mapa.alto || 1000;
      dibujar();
    };
  }

  $: if (ctx && canvas && rejilla?.tamano) {
    dibujar();
  }

  function dibujar() {
    if (!ctx || !canvas) return;

    const t = Number(rejilla.tamano);
    if (isNaN(t) || t < 20) return;

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    // Si hay imagen se pinta la imagen, si no, fondo gris
    if (imagenFondo) {
      ctx.drawImage(imagenFondo, 0, 0, canvas.width, canvas.height);
    } else {
      ctx.fillStyle = '#1e1e1e';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
    }

    // Rejilla
    ctx.strokeStyle = rejilla.color;
    ctx.lineWidth = rejilla.grosor;
    ctx.globalAlpha = rejilla.opacidad;

    ctx.beginPath();
    for (let x = 0; x <= canvas.width; x += t) {
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
    }
    for (let y = 0; y <= canvas.height; y += t) {
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
    }
    ctx.stroke();

    ctx.globalAlpha = 1.0;
  }

  onMount(() => {
    ctx = canvas.getContext('2d');
    if (mapa?.url_imagen_fondo) {
      cargarImagen(mapa.url_imagen_fondo);
    } else {
      dibujar();
    }
  });
</script>

<div class="contenedor-lienzo">
  <canvas 
    bind:this={canvas} 
    width={mapa?.ancho || 1000} 
    height={mapa?.alto || 1000} 
    class="lienzo-vtt"
  ></canvas>
</div>

<style>
  .contenedor-lienzo {
    width: 100%;
    height: 100%;
    overflow: auto;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #121212;
  }
  .lienzo-vtt {
    box-shadow: 0 0 25px rgba(0,0,0,0.8);
  }
</style>
