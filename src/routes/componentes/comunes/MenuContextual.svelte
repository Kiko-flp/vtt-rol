<!-- src/routes/componentes/comunes/MenuContextual.svelte -->
<script lang="ts">
  interface Opcion {
    texto: string;
    accion: () => void;
    peligro?: boolean;
  }

  let { x, y, opciones, onCerrar }: { x: number; y: number; opciones: Opcion[]; onCerrar: () => void } = $props();
</script>

<!-- Cualquier clic fuera (o sobre una opción, tras ejecutarla) cierra el menú -->
<svelte:window onclick={onCerrar} />

<div class="menu-contextual" style="top: {y}px; left: {x}px;">
  {#each opciones as o}
    <button class:peligro={o.peligro} onclick={o.accion}>{o.texto}</button>
  {/each}
</div>

<style>
  .menu-contextual {
    position: fixed; background: #1c1c1f; border: 1px solid #3f3f46; border-radius: 4px;
    padding: 3px; display: flex; flex-direction: column; gap: 2px; z-index: 1000; box-shadow: 0 4px 12px rgba(0,0,0,0.5);
  }
  button {
    background: none; border: none; color: #f4f4f5; font-size: 11px; padding: 4px 8px;
    text-align: left; cursor: pointer; border-radius: 2px;
  }
  button:hover { background: #27272a; }
  button.peligro:hover { background: #dc2626; color: white; }
</style>
