<!-- src/routes/componentes/panel-izquierdo/BancoEntidades.svelte -->
<script lang="ts">
  import { partida } from "$lib/partida.svelte";

  let busqueda = $state("");
  let filtroZona = $state("todos");

  let entidadesFiltradas = $derived(
    partida.banco.filter((e) => {
      if (e.tipo === "Jugador") return false;
      const coincideNombre = e.nombre.toLowerCase().includes(busqueda.toLowerCase());
      const coincideZona = filtroZona === "todos" || e.zona_id === partida.zonaId;
      return coincideNombre && coincideZona;
    })
  );
</script>

<div class="banco">
  <div class="sub-busqueda">
    <input type="text" placeholder="Buscar..." bind:value={busqueda} class="input-buscar" />
    <div class="radios-filtro">
      <label><input type="radio" bind:group={filtroZona} value="todos" /> Todos</label>
      <label><input type="radio" bind:group={filtroZona} value="actual" /> Zona</label>
    </div>
  </div>

  <div class="sub-entidades">
    {#each entidadesFiltradas as item (item.id)}
      <div class="fila-entidad" draggable="true" title="Arrastra al mapa">
        <div class="forma {item.tipo.toLowerCase()}"></div>
        <span class="texto-recortado">{item.nombre}</span>
      </div>
    {/each}
  </div>
</div>

<style>
  .banco { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 5px; overflow: hidden; }
  .sub-busqueda { display: flex; flex-direction: column; gap: 3px; }
  .input-buscar { background: #09090b; border: 1px solid #3f3f46; color: white; padding: 3px 5px; border-radius: 4px; font-size: 11px; }
  .radios-filtro { display: flex; gap: 6px; font-size: 10px; color: #a1a1aa; }
  .sub-entidades { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
  .fila-entidad { display: flex; align-items: center; gap: 6px; background: #27272a; padding: 5px; border-radius: 4px; cursor: grab; }
  .forma { width: 12px; height: 12px; flex-shrink: 0; }
  .forma.enemigo { background: #ef4444; }
  .forma.npc { background: #3b82f6; transform: rotate(45deg); }
  .texto-recortado { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
