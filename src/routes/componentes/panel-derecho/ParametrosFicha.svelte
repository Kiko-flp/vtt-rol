<!-- src/routes/componentes/panel-derecho/ParametrosFicha.svelte -->
<script lang="ts">
  import { partida, type Cuerpo } from "$lib/partida.svelte";

  const textoCuerpo = (c: Cuerpo) => (typeof c === "object" ? `Grande (${c.Grande.join("x")})` : "1x1");

  let ficha = $derived(partida.instanciaSeleccionada);
  let mapaFicha = $derived(
    ficha ? partida.mapasTodasZonas.find((mapa) => mapa.id === ficha.id_mapa) ?? null : null
  );
  let zonaFicha = $derived(
    ficha
      ? partida.zonas.find((zona) => zona.id === (mapaFicha?.id_zona ?? ficha.base.zona_id)) ?? null
      : null
  );
  let dano = $state(1);
  let errorVida = $state("");

  async function aplicarDano(evento: SubmitEvent) {
    evento.preventDefault();
    errorVida = "";
    const cantidad = Math.floor(Number(dano));
    if (!Number.isInteger(cantidad) || cantidad < 1) {
      errorVida = "Introduce un daño mayor que cero.";
      return;
    }
    if (!ficha) return;
    try {
      await partida.modificarVida(ficha.id_instancia, -cantidad);
      dano = 1;
    } catch (error) {
      errorVida = error instanceof Error ? error.message : String(error);
    }
  }

  async function curarUnPunto() {
    if (!ficha) return;
    errorVida = "";
    try {
      await partida.modificarVida(ficha.id_instancia, 1);
    } catch (error) {
      errorVida = error instanceof Error ? error.message : String(error);
    }
  }

  async function retirarDelMapa() {
    if (!ficha || !ficha.activo || ficha.id_mapa === null) return;
    errorVida = "";
    try {
      await partida.retirarInstancia(ficha.id_instancia);
    } catch (error) {
      errorVida = error instanceof Error ? error.message : String(error);
    }
  }

  let parametros = $derived(
    ficha
      ? [
          { etiqueta: "V (Velocidad)", valor: `${ficha.base.velocidad} m` },
          { etiqueta: "RA (Rango Arma)", valor: `${ficha.base.rango_arma} m` },
          { etiqueta: "Cuerpo", valor: textoCuerpo(ficha.base.cuerpo) },
          { etiqueta: "Armadura", valor: String(ficha.base.armadura) }
        ]
      : []
  );
</script>

<div class="caja">
  <span class="titulo-mini">PARÁMETROS</span>
  {#if ficha}
    <div class="caja-parametros">
      {#each parametros as p}
        <div class="param">
          <span class="label">{p.etiqueta}</span>
          <span class="valor-destacado">{p.valor}</span>
        </div>
      {/each}

      <div class="param vida">
        <span class="label">Vida actual:</span>
        <span class="valor-destacado">{ficha.vida_actual} / {ficha.base.vida_max} PV</span>
      </div>
      <div class="param ubicacion">
        <span class="label">Ubicación</span>
        <span class="valor-destacado">
          {#if ficha.activo && mapaFicha}
            {zonaFicha?.nombre ?? "Zona desconocida"} · {mapaFicha.nombre || `Mapa ${mapaFicha.id}`}
          {:else}
            Sin colocar · {zonaFicha?.nombre ?? "Zona desconocida"}
          {/if}
        </span>
      </div>
      <button
        class="retirar"
        type="button"
        onclick={retirarDelMapa}
        disabled={!ficha.activo || ficha.id_mapa === null}
      >Retirar del mapa</button>
      <form class="control-dano" onsubmit={aplicarDano}>
        <label for="dano-recibido">Daño recibido</label>
        <input id="dano-recibido" type="number" min="1" step="1" bind:value={dano} />
        <button type="submit" disabled={ficha.vida_actual <= 0}>Aplicar daño</button>
        <button type="button" class="curar" onclick={curarUnPunto} disabled={ficha.vida_actual >= ficha.base.vida_max}>
          Curar 1 PV
        </button>
      </form>
      {#if errorVida}<span class="error-vida" role="alert">{errorVida}</span>{/if}
    </div>
  {:else}
    <div class="sin-seleccion">Pulsa una ficha para ver sus parámetros</div>
  {/if}
</div>

<style>
  .caja {
    flex: 1;
    min-height: 0;
    background: #1b1b1f;
    border: 1px solid #27272a;
    border-radius: 6px;
    padding: 6px;
    display: flex;
    flex-direction: column;
  }
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }
  .caja-parametros { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .param { background: #27272a; padding: 5px; border-radius: 4px; }
  .param.vida { grid-column: span 2; display: flex; justify-content: space-between; align-items: center; }
  .param.ubicacion { grid-column: span 2; }
  .retirar { grid-column: span 2; padding: 6px; border: 1px solid #7f1d1d; border-radius: 4px; color: #fee2e2; background: #451a1a; cursor: pointer; }
  .retirar:disabled { opacity: .5; cursor: not-allowed; }
  .label { color: #a1a1aa; font-size: 10px; display: block; }
  .valor-destacado { font-size: 13px; font-weight: bold; color: #f4f4f5; }
  .control-dano { grid-column: span 2; display: grid; grid-template-columns: 1fr 1fr; gap: 5px; align-items: center; }
  .control-dano label { grid-column: 1 / -1; color: #a1a1aa; font-size: 10px; }
  .control-dano input { grid-column: 1; width: 100%; box-sizing: border-box; padding: 5px; border: 1px solid #3f3f46; border-radius: 4px; color: #f4f4f5; background: #18181b; }
  .control-dano button { grid-column: 2; padding: 6px; border: 0; border-radius: 4px; color: white; background: #b91c1c; cursor: pointer; }
  .control-dano button.curar { grid-column: 1 / -1; background: #3f3f46; }
  .control-dano button:disabled { opacity: .5; cursor: not-allowed; }
  .error-vida { grid-column: span 2; color: #fca5a5; font-size: 10px; }
  .sin-seleccion { color: #71717a; text-align: center; margin: auto 0; font-size: 11px; }
</style>
