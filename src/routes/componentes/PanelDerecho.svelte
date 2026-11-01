<!-- src/routes/componentes/PanelDerecho.svelte -->
<script lang="ts">
  import type { Entidad, RegistroTirada } from '$lib/types';

  export let entidadesActivas: Entidad[];
  export let historialTiradas: RegistroTirada[];
  export let onAbrirFicha: (entidad: Entidad) => void;

  $: enemigos = entidadesActivas.filter(e => e.tipo === 'enemigo');
  $: npcs = entidadesActivas.filter(e => e.tipo === 'npc');
  $: jugadores = entidadesActivas.filter(e => e.tipo === 'jugador');

  function ajustarVida(entidad: Entidad, delta: number) {
    entidad.vidaActual = Math.max(0, entidad.vidaActual + delta);
    entidadesActivas = [...entidadesActivas]; // Forzar reactividad
  }
</script>

<aside class="panel-derecho">
  <div class="bloque-activos">
    <!-- Grupo: Enemigos -->
    <div class="grupo-tipo">
      <div class="cabecera-grupo rojo">ENEMIGOS <button>+</button></div>
      {#each enemigos as enemigo}
        <div class="tarjeta-mini">
          <div class="token-icono cuadrado"></div>
          <span class="nombre" on:click={() => onAbrirFicha(enemigo)}>{enemigo.nombre}</span>
          <div class="controles-vida">
            <button on:click={() => ajustarVida(enemigo, -1)}>-</button>
            <span class="vida">{enemigo.vidaActual}/{enemigo.vidaMax}</span>
            <button on:click={() => ajustarVida(enemigo, 1)}>+</button>
          </div>
          <span class="armadura" title="Armadura">🛡️{enemigo.armadura}</span>
        </div>
      {/each}
    </div>

    <!-- Grupo: NPCs -->
    <div class="grupo-tipo">
      <div class="cabecera-grupo azul">NPCs <button>+</button></div>
      {#each npcs as npc}
        <div class="tarjeta-mini">
          <div class="token-icono rombo"></div>
          <span class="nombre" on:click={() => onAbrirFicha(npc)}>{npc.nombre}</span>
          <span class="vida">{npc.vidaActual} PV</span>
        </div>
      {/each}
    </div>

    <!-- Grupo: Jugadores -->
    <div class="grupo-tipo">
      <div class="cabecera-grupo verde">JUGADORES <button>+</button></div>
      {#each jugadores as jug}
        <div class="tarjeta-mini">
          <div class="token-icono circulo"></div>
          <span class="nombre" on:click={() => onAbrirFicha(jug)}>{jug.nombre}</span>
          <span class="vida">{jug.vidaActual}/{jug.vidaMax}</span>
        </div>
      {/each}
    </div>
  </div>

  <!-- Registro de Tiradas inferior -->
  <div class="bloque-tiradas">
    <h4>TIRADAS</h4>
    <div class="lista-tiradas">
      {#each historialTiradas as t}
        <div class="item-tirada">
          <span class="formula">{t.formula}:</span>
          <span class="desglose">{t.desglose}</span>
          <strong class="total">= {t.total}</strong>
        </div>
      {/each}
    </div>
  </div>
</aside>

<style>
  .panel-derecho {
    width: 280px;
    background-color: #18181b;
    border-left: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    padding: 10px;
    gap: 10px;
  }
  .bloque-activos {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .cabecera-grupo {
    display: flex;
    justify-content: space-between;
    font-weight: bold;
    font-size: 13px;
    padding-bottom: 4px;
    border-bottom: 2px solid;
    margin-bottom: 6px;
  }
  .cabecera-grupo.rojo { border-color: #ef4444; color: #ef4444; }
  .cabecera-grupo.azul { border-color: #3b82f6; color: #3b82f6; }
  .cabecera-grupo.verde { border-color: #22c55e; color: #22c55e; }

  .tarjeta-mini {
    display: flex;
    align-items: center;
    background: #27272a;
    padding: 6px;
    border-radius: 4px;
    gap: 8px;
    font-size: 12px;
    margin-bottom: 4px;
  }
  .nombre { flex: 1; cursor: pointer; text-decoration: underline dotted; }
  
  .controles-vida {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .controles-vida button {
    background: #3f3f46;
    border: none;
    color: white;
    width: 18px;
    height: 18px;
    cursor: pointer;
    border-radius: 2px;
  }

  .token-icono { width: 14px; height: 14px; }
  .cuadrado { background: #ef4444; }
  .rombo { background: #3b82f6; transform: rotate(45deg); }
  .circulo { background: #22c55e; border-radius: 50%; }

  .bloque-tiradas {
    height: 160px;
    border-top: 1px solid #3f3f46;
    display: flex;
    flex-direction: column;
  }
  h4 { margin: 8px 0 4px 0; font-size: 12px; color: #a1a1aa; }
  .lista-tiradas {
    flex: 1;
    overflow-y: auto;
    background: #09090b;
    padding: 6px;
    font-size: 11px;
    font-family: monospace;
    border-radius: 4px;
  }
  .item-tirada { margin-bottom: 4px; }
  .total { color: #4ade80; }
</style>