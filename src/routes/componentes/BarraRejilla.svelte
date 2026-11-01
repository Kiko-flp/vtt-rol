<!-- src/routes/componentes/BarraRejilla.svelte -->
<script lang="ts">
  import type { ConfiguracionRejilla } from '$lib/types';
  export let rejilla: ConfiguracionRejilla;
</script>

<div class="barra-herramientas">
  <button class="btn-lapiz" title="Modo Dibujo">✏️</button>
  
  <div class="control">
    <label>Rejilla:</label>
    <input type="number" bind:value={rejilla.tamano} min="20" max="150" step="5" />
    <span>px</span>
  </div>

  <div class="control">
    <label>Color:</label>
    <input type="color" bind:value={rejilla.color} />
  </div>

  <div class="control">
    <label>Grosor:</label>
    <input type="number" bind:value={rejilla.grosor} min="0.5" max="5" step="0.5" />
  </div>

  <div class="control">
    <label>Opacidad:</label>
    <input 
      type="range" 
      min="0" 
      max="1" 
      step="0.05" 
      bind:value={rejilla.opacidad} 
    />
    <span>{Math.round(rejilla.opacidad * 100)}%</span>
  </div>
</div>

<style>
  .barra-herramientas {
    display: flex;
    align-items: center;
    gap: 15px;
    background-color: #262626;
    padding: 6px 16px;
    border-radius: 6px;
    font-size: 13px;
    border: 1px solid #3f3f46;
  }
  .control {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .control input[type="number"] {
    width: 50px;
    background: #18181b;
    color: white;
    border: 1px solid #52525b;
    border-radius: 4px;
    padding: 2px 4px;
  }
  .btn-lapiz {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
  }
</style>