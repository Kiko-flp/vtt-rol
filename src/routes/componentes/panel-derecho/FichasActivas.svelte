<!-- src/routes/componentes/panel-derecho/FichasActivas.svelte -->
<script lang="ts">
  import { partida } from "$lib/partida.svelte";

  // La geometría distingue tipos aunque el color no se perciba.
  const GRUPOS = [
    { tipo: "Enemigo", titulo: "ENEMIGOS", color: "rojo", icono: "cuadrado" },
    { tipo: "NPC", titulo: "NPCs", color: "azul", icono: "triangulo" },
    { tipo: "Jugador", titulo: "JUGADORES", color: "verde", icono: "circulo" }
  ] as const;

  let grupos = $derived(
    GRUPOS.map((g) => ({ ...g, fichas: partida.instancias.filter((i) => i.base.tipo === g.tipo) }))
  );
</script>

<span class="titulo-mini">FICHAS ACTIVAS</span>
<div class="scroll-fichas">
  {#each grupos as g}
    <div class="grupo {g.color}">{g.titulo}</div>
    {#each g.fichas as ficha}
      {@const mapa = partida.mapasTodasZonas.find((m) => m.id === ficha.id_mapa)}
      {@const zona = partida.zonas.find((z) => z.id === (mapa?.id_zona ?? ficha.base.zona_id))}
      <button
        type="button"
        class="fila-ficha"
        class:muerto={ficha.vida_actual <= 0}
        class:seleccionado={partida.idInstanciaSeleccionada === ficha.id_instancia}
        onclick={() => partida.seleccionarInstancia(ficha)}
      >
        <span class="icono {g.icono}"></span>
        <span class="datos-ficha">
          <span class="nombre">{ficha.base.nombre}</span>
          <small class="ubicacion">
            {#if ficha.activo && mapa}
              {zona?.nombre ?? "Zona desconocida"} · {mapa.nombre || `Mapa ${mapa.id}`}
            {:else}
              Sin colocar · {zona?.nombre ?? "Zona desconocida"}
            {/if}
          </small>
        </span>
        <span class="pv">{ficha.vida_actual}/{ficha.base.vida_max}</span>
      </button>
    {/each}
  {/each}
</div>

<style>
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }
  .scroll-fichas { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
  .grupo { font-size: 10px; font-weight: bold; margin-top: 4px; border-bottom: 1px solid; }
  .grupo.rojo { color: #ef4444; border-color: #ef4444; }
  .grupo.azul { color: #3b82f6; border-color: #3b82f6; }
  .grupo.verde { color: #22c55e; border-color: #22c55e; }
  .fila-ficha {
    display: flex; align-items: center; gap: 6px; background: #1f1f23; padding: 4px 6px; border-radius: 4px; cursor: pointer;
    /* reset de botón para que se vea igual que el div anterior */
    border: 1px solid transparent; width: 100%; color: inherit; font: inherit; text-align: left;
  }
  .fila-ficha.seleccionado { border-color: #3b82f6; }
  .fila-ficha.muerto { opacity: 0.35; filter: grayscale(100%); }
  .icono { width: 12px; height: 12px; flex: 0 0 12px; }
  .cuadrado { background: #ef4444; }
  .triangulo { background: #3b82f6; clip-path: polygon(50% 0, 0 100%, 100% 100%); }
  .circulo { background: #22c55e; border-radius: 50%; }
  .datos-ficha { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 2px; }
  .nombre { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }
  .ubicacion { overflow: hidden; color: #a1a1aa; text-overflow: ellipsis; white-space: nowrap; font-size: 9px; }
  .pv { flex: 0 0 auto; color: #d4d4d8; font-size: 10px; }
</style>
