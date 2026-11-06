<script>
  import { rutaFlechaConGiroCercano } from "$lib/flechas";

  /** @type {{
   * flechas?: import("$lib/partida.svelte").Flecha[],
   * borrador?: { id_instancia: number, puntos: import("$lib/partida.svelte").PuntoFlecha[] } | null,
   * instancias?: import("$lib/partida.svelte").Instancia[],
   * distribucion: import("$lib/rejilla").DistribucionRejilla,
   * cam: { x: number, y: number, zoom: number }
   * }} */
  let {
    flechas = [],
    borrador = null,
    instancias = [],
    distribucion,
    cam
  } = $props();

  const flechasVisibles = $derived([
    ...flechas.filter((flecha) => flecha.id_instancia !== borrador?.id_instancia),
    ...(borrador ? [borrador] : [])
  ]);
</script>

{#each flechasVisibles as flecha (flecha.id_instancia)}
  {@const instancia = instancias.find((ficha) => ficha.id_instancia === flecha.id_instancia)}
  {#if instancia && flecha.puntos.length > 1}
    {@const tipo = instancia.base.tipo.toLowerCase()}
    {@const tamanoCelda = Math.min(distribucion.anchoCelda, distribucion.altoCelda) * cam.zoom}
    {@const grosor = Math.max(1, tamanoCelda * 0.08)}
    {@const tamanoPunta = Math.max(4, tamanoCelda * 0.45)}
    {@const origen = flecha.puntos[0]}
    {@const destino = flecha.puntos[flecha.puntos.length - 1]}
    {@const inicioX = cam.x + origen.x * distribucion.anchoCelda * cam.zoom}
    {@const inicioY = cam.y + origen.y * distribucion.altoCelda * cam.zoom}
    {@const finX = cam.x + destino.x * distribucion.anchoCelda * cam.zoom}
    {@const finY = cam.y + destino.y * distribucion.altoCelda * cam.zoom}
    {@const ruta = rutaFlechaConGiroCercano(
      { x: inicioX, y: inicioY },
      { x: finX, y: finY }
    )}
    {@const recorrido = ruta.map((punto, indice) =>
      `${indice === 0 ? "M" : "L"} ${punto.x} ${punto.y}`
    ).join(" ")}
    <svg class="flecha-capa" aria-hidden="true">
      <defs>
        <marker
          id="punta-flecha-{flecha.id_instancia}"
          class="punta-flecha {tipo}"
          viewBox="0 0 12 12"
          markerWidth={tamanoPunta}
          markerHeight={tamanoPunta}
          markerUnits="userSpaceOnUse"
          refX="10"
          refY="6"
          orient="auto"
        >
          <path d="M 0 0 L 10 5 L 0 10 z" />
        </marker>
      </defs>
      <path
        class="flecha-movimiento {tipo}"
        d={recorrido}
        stroke-width={grosor}
        marker-end="url(#punta-flecha-{flecha.id_instancia})"
      />
    </svg>
  {/if}
{/each}

<style>
  .flecha-capa {
    position: absolute;
    z-index: 4;
    inset: 0;
    width: 100%;
    height: 100%;
    overflow: visible;
    pointer-events: none;
  }
  .flecha-movimiento { fill: none; }
  .flecha-movimiento.jugador { stroke: #22c55e; }
  .punta-flecha.jugador path { stroke: #22c55e; fill: #22c55e; }
  .flecha-movimiento.npc { stroke: #3b82f6; }
  .punta-flecha.npc path { stroke: #3b82f6; fill: #3b82f6; }
  .flecha-movimiento.enemigo { stroke: #ef4444; }
  .punta-flecha.enemigo path { stroke: #ef4444; fill: #ef4444; }
</style>
