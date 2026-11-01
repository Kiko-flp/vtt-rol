<!-- src/routes/componentes/PanelIzquierdo.svelte -->
<script lang="ts">
  import type { Zona, Entidad } from '$lib/types';
  
  export let zonas: Zona[];
  export let bancoEntidades: Entidad[];
  export let onAbrirFicha: (entidad: Entidad) => void;

  let busqueda = "";
  let filtroZona = "todas";

  $: entidadesFiltradas = bancoEntidades.filter(e => {
    const coincideNombre = e.nombre.toLowerCase().includes(busqueda.toLowerCase());
    const coincideZona = filtroZona === "todas" || e.zonaId === filtroZona;
    return coincideNombre && coincideZona;
  });
</script>

<aside class="panel-izquierdo">
  <!-- Sección superior: Banco de Zonas -->
  <div class="seccion-zonas">
    <h3>Banco de Zonas</h3>
    <div class="cuadricula-zonas">
      {#each zonas as zona}
        <button class="tarjeta-zona" title={zona.nombre}>
          {zona.id}
        </button>
      {/each}
      <button class="tarjeta-zona nueva">+ </button>
    </div>
  </div>

  <!-- Sección inferior: Banco de NPCs / Guardados -->
  <div class="seccion-npcs">
    <h3>Banco de Entidades</h3>
    
    <input 
      type="text" 
      placeholder="Buscar por nombre..." 
      bind:value={busqueda} 
      class="input-busqueda"
    />

    <div class="filtros">
      <label>
        <input type="radio" bind:group={filtroZona} value="todas" /> Todos
      </label>
      <label>
        <input type="radio" bind:group={filtroZona} value="z1" /> Zona actual
      </label>
    </div>

    <div class="lista-fichas">
      {#each entidadesFiltradas as item}
        <div 
          class="ficha-item {item.tipo}" 
          on:dblclick={() => onAbrirFicha(item)}
          title="Doble clic para ver ficha"
        >
          <span class="icono-forma"></span>
          <span class="nombre">{item.nombre}</span>
        </div>
      {/each}
    </div>
  </div>
</aside>

<style>
  .panel-izquierdo {
    width: 240px;
    background-color: #18181b;
    border-right: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    padding: 10px;
    gap: 15px;
    user-select: none;
  }
  h3 { font-size: 14px; margin: 0 0 8px 0; color: #a1a1aa; text-transform: uppercase; }
  
  .cuadricula-zonas {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .tarjeta-zona {
    aspect-ratio: 1;
    background: #27272a;
    border: 1px solid #3f3f46;
    color: white;
    font-weight: bold;
    cursor: pointer;
    border-radius: 4px;
  }
  .tarjeta-zona.nueva { border-style: dashed; }

  .seccion-npcs {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
  }
  .input-busqueda {
    background: #09090b;
    border: 1px solid #3f3f46;
    color: white;
    padding: 6px;
    border-radius: 4px;
    font-size: 12px;
  }
  .filtros {
    display: flex;
    gap: 10px;
    font-size: 12px;
    margin: 8px 0;
    color: #a1a1aa;
  }
  .lista-fichas {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .ficha-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px;
    background: #27272a;
    border-radius: 4px;
    cursor: grab;
  }
  .icono-forma {
    width: 14px;
    height: 14px;
  }
  .ficha-item.enemigo .icono-forma { background: #ef4444; } /* Cuadrado Rojo */
  .ficha-item.npc .icono-forma { background: #3b82f6; transform: rotate(45deg); } /* Rombo Azul */
  .ficha-item.jugador .icono-forma { background: #22c55e; border-radius: 50%; } /* Círculo Verde */
</style>