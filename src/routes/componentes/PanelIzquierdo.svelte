<!-- src/routes/componentes/PanelIzquierdo.svelte -->
<script>
  export let zonas = [];
  export let mapas = [];
  export let banco = [];
  export let zonaSeleccionadaId = 1;
  export let onSeleccionarZona;
  export let onSeleccionarMapa;

  let busqueda = "";
  let filtroZona = "todos";

  $: entidadesFiltradas = banco.filter(e => {
    if (e.tipo === "Jugador") return false;
    const coincideNombre = e.nombre.toLowerCase().includes(busqueda.toLowerCase());
    const coincideZona = filtroZona === "todos" || e.zona_id === zonaSeleccionadaId;
    return coincideNombre && coincideZona;
  });
</script>

<aside class="panel-izquierdo">
  <div class="bloque-arriba">
    <div class="subseccion sub-zonas">
      <span class="titulo-mini">ZONAS</span>
      <div class="scroll-zonas">
        {#each zonas as z}
          <button 
            class="item-zona" 
            class:activo={zonaSeleccionadaId === z.id}
            on:click={() => onSeleccionarZona(z.id)}
            title={z.nombre}
          >
            {z.id}
          </button>
        {/each}
        <button class="item-zona nueva">+</button>
      </div>
    </div>

    <div class="subseccion sub-mapas">
      <span class="titulo-mini">MAPAS</span>
      <div class="scroll-mapas">
        {#each mapas as m}
          <div class="tarjeta-mapa" on:click={() => onSeleccionarMapa(m)}>
            <div class="previa-mapa">🗺️</div>
            <span class="nombre-mapa">Mapa #{m.id}</span>
          </div>
        {/each}
        {#if mapas.length === 0}
          <span class="vacio">Sin mapas</span>
        {/if}
      </div>
    </div>
  </div>

  <div class="bloque-abajo">
    <div class="sub-busqueda">
      <input 
        type="text" 
        placeholder="Buscar..." 
        bind:value={busqueda}
        class="input-buscar"
      />
      <div class="radios-filtro">
        <label><input type="radio" bind:group={filtroZona} value="todos" /> Todos</label>
        <label><input type="radio" bind:group={filtroZona} value="actual" /> Zona</label>
      </div>
    </div>

    <div class="sub-entidades">
      {#each entidadesFiltradas as item (item.id)}
        <div class="fila-entidad" draggable="true" title="Arrastra al mapa">
          <div class="forma {item.tipo.toLowerCase()}"></div>
          <span class="texto-entidad">{item.nombre}</span>
        </div>
      {/each}
    </div>
  </div>
</aside>

<style>
  .panel-izquierdo {
    width: clamp(180px, 14vw, 225px); /* Más estrecho */
    height: 100%;
    background: #141416;
    border-right: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 6px;
    gap: 6px;
    user-select: none;
    font-size: 11px;
  }
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }
  .bloque-arriba { height: 38%; display: flex; gap: 4px; border-bottom: 1px solid #27272a; padding-bottom: 4px; }
  .subseccion { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .scroll-zonas, .scroll-mapas { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
  .item-zona { background: #27272a; border: 1px solid #3f3f46; color: white; font-weight: bold; padding: 6px; border-radius: 4px; cursor: pointer; }
  .item-zona.activo { background: #3b82f6; border-color: #60a5fa; }
  .item-zona.nueva { border-style: dashed; }
  .tarjeta-mapa { background: #1f1f23; border: 1px solid #3f3f46; border-radius: 4px; padding: 4px; text-align: center; cursor: pointer; }
  .previa-mapa { font-size: 16px; }
  .nombre-mapa { font-size: 9px; color: #d4d4d8; display: block; }
  .bloque-abajo { flex: 1; display: flex; flex-direction: column; overflow: hidden; gap: 5px; }
  .sub-busqueda { display: flex; flex-direction: column; gap: 3px; }
  .input-buscar { background: #09090b; border: 1px solid #3f3f46; color: white; padding: 3px 5px; border-radius: 4px; font-size: 11px; }
  .radios-filtro { display: flex; gap: 6px; font-size: 10px; color: #a1a1aa; }
  .sub-entidades { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
  .fila-entidad { display: flex; align-items: center; gap: 6px; background: #27272a; padding: 5px; border-radius: 4px; cursor: grab; }
  .forma { width: 12px; height: 12px; flex-shrink: 0; }
  .forma.enemigo { background: #ef4444; }
  .forma.npc { background: #3b82f6; transform: rotate(45deg); }
  .texto-entidad { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .vacio { color: #52525b; font-style: italic; text-align: center; margin-top: 10px; }
</style>