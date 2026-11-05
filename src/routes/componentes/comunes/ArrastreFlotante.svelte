<!-- src/routes/componentes/comunes/ArrastreFlotante.svelte -->
<!-- Clon que sigue al cursor mientras se arrastra un elemento de una ListaOrdenable. -->
<script lang="ts">
  import type { Snippet } from "svelte";

  interface Arrastre {
    activo: boolean;
    item: unknown;
    x: number;
    y: number;
    offsetX: number;
    offsetY: number;
    ancho: number;
    alto: number;
  }

  let { lista, children }: { lista: Arrastre; children: Snippet } = $props();
</script>

{#if lista.activo && lista.item}
  <div
    class="elemento-flotante"
    style="
      left: {lista.x - lista.offsetX}px;
      top: {lista.y - lista.offsetY}px;
      width: {lista.ancho}px;
      height: {lista.alto}px;
    "
  >
    {@render children()}
  </div>
{/if}

<style>
  .elemento-flotante {
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    transform: rotate(2deg) scale(1.05);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.7);
    opacity: 0.92;
    transition: transform 0.05s ease;
  }
</style>
