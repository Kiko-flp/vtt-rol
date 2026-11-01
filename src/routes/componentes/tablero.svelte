<!-- src/routes/componentes/Tablero.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import type { ConfiguracionRejilla } from '$lib/types';

  export let rejilla: ConfiguracionRejilla;

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;

  $: if (ctx && canvas && rejilla) {
    dibujar();
  }

  function dibujar() {
    if (!ctx || !canvas) return;

    // 1. Limpiamos y pintamos el fondo oscuro siempre
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.fillStyle = '#1e1e1e';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    // 🛑 PROTECCIÓN CRÍTICA:
    // Si están borrando el texto, es 0, o es menor a 15px, 
    // NO dibujamos líneas para evitar bucles infinitos y petes de CPU.
    const t = Number(rejilla.tamano);
    if (!t || t < 15) {
      return; 
    }

    // 2. Configuración de la rejilla
    ctx.strokeStyle = rejilla.color;
    ctx.lineWidth = rejilla.grosor;
    ctx.globalAlpha = rejilla.opacidad;

    ctx.beginPath();
    // Líneas verticales
    for (let x = 0; x <= canvas.width; x += t) {
      ctx.moveTo(x, 0);
      ctx.lineTo(x, canvas.height);
    }
    // Líneas horizontales
    for (let y = 0; y <= canvas.height; y += t) {
      ctx.moveTo(0, y);
      ctx.lineTo(canvas.width, y);
    }
    ctx.stroke();

    ctx.globalAlpha = 1.0;
  }

  onMount(() => {
    ctx = canvas.getContext('2d');
    dibujar();
  });
</script>

<div class="contenedor-lienzo">
  <canvas 
    bind:this={canvas} 
    width="1200" 
    height="800" 
    class="lienzo-vtt"
  ></canvas>
</div>


<style>
  .contenedor-lienzo {
    width: 100%;
    height: 100%;
    overflow: hidden;
    display: flex;
    justify-content: center;
    align-items: center;
    background-color: #121212;
  }
  .lienzo-vtt {
    box-shadow: 0 0 20px rgba(0,0,0,0.8);
    cursor: grab;
  }
</style>