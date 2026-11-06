<script lang="ts">
  import { urlDeImagen } from "$lib/imagenes";
  import { partida, type Entidad, type TipoEntidad } from "$lib/partida.svelte";

  let busqueda = $state("");
  let error = $state("");
  let nuevaEntidad = $state({
    nombre: "",
    tipo: "Enemigo" as TipoEntidad,
    vidaMax: 10,
    armadura: 0,
    velocidad: 6,
    rangoArma: 1
  });
  let arrastre = $state<{ item: Entidad; x: number; y: number; offsetX: number; offsetY: number } | null>(null);

  const coincideNombre = (entidad: Entidad) =>
    entidad.nombre.toLocaleLowerCase().includes(busqueda.toLocaleLowerCase());
  const jugadores = $derived(partida.banco.filter((entidad) => entidad.tipo === "Jugador" && coincideNombre(entidad)));
  const npcs = $derived(
    partida.banco.filter(
      (entidad) => entidad.tipo === "NPC" && entidad.zona_id === partida.zonaId && coincideNombre(entidad)
    )
  );
  const enemigos = $derived(
    partida.banco.filter(
      (entidad) => entidad.tipo === "Enemigo" && entidad.zona_id === partida.zonaId && coincideNombre(entidad)
    )
  );

  function fuenteFoto(entidad: Entidad) {
    return urlDeImagen(entidad.foto_url);
  }

  async function crear(evento: SubmitEvent) {
    evento.preventDefault();
    error = "";
    try {
      await partida.crearEntidad({ ...nuevaEntidad, nombre: nuevaEntidad.nombre.trim() });
      nuevaEntidad.nombre = "";
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function iniciarArrastre(evento: PointerEvent, item: Entidad) {
    if (evento.button !== 0 || !(evento.currentTarget instanceof HTMLElement)) return;
    evento.preventDefault();
    const rect = evento.currentTarget.getBoundingClientRect();
    const inicioX = evento.clientX;
    const inicioY = evento.clientY;
    let iniciado = false;

    const limpiar = () => {
      window.removeEventListener("pointermove", alMover);
      window.removeEventListener("pointerup", alSoltar);
      window.removeEventListener("pointercancel", alCancelar);
      arrastre = null;
    };
    const alMover = (movimiento: PointerEvent) => {
      if (movimiento.pointerId !== evento.pointerId) return;
      if (!iniciado && Math.hypot(movimiento.clientX - inicioX, movimiento.clientY - inicioY) < 4) return;
      iniciado = true;
      arrastre = {
        item,
        x: movimiento.clientX,
        y: movimiento.clientY,
        offsetX: inicioX - rect.left,
        offsetY: inicioY - rect.top
      };
    };
    const alSoltar = (soltar: PointerEvent) => {
      if (soltar.pointerId !== evento.pointerId) return;
      const tablero = document.querySelector(".tablero");
      const limite = tablero?.getBoundingClientRect();
      const colocar = iniciado && limite !== undefined
        && soltar.clientX >= limite.left
        && soltar.clientX <= limite.right
        && soltar.clientY >= limite.top
        && soltar.clientY <= limite.bottom;
      limpiar();
      if (colocar) {
        window.dispatchEvent(
          new CustomEvent("vtt-colocar-entidad", {
            detail: { entidadId: item.id, clientX: soltar.clientX, clientY: soltar.clientY }
          })
        );
      }
    };
    const alCancelar = (cancelar: PointerEvent) => {
      if (cancelar.pointerId === evento.pointerId) limpiar();
    };

    window.addEventListener("pointermove", alMover);
    window.addEventListener("pointerup", alSoltar);
    window.addEventListener("pointercancel", alCancelar);
  }
</script>

<div class="banco">
  <div class="cabecera-banco">
    <span class="titulo-mini">FICHAS</span>
    <input type="search" placeholder="Buscar..." bind:value={busqueda} aria-label="Buscar fichas" />
  </div>

  <details class="crear-entidad">
    <summary>Crear ficha</summary>
    <form onsubmit={crear}>
      <input bind:value={nuevaEntidad.nombre} maxlength="80" placeholder="Nombre" aria-label="Nombre de ficha" />
      <select bind:value={nuevaEntidad.tipo} aria-label="Tipo de ficha">
        <option value="Jugador">Jugador</option>
        <option value="NPC">NPC</option>
        <option value="Enemigo">Enemigo</option>
      </select>
      <div class="campos">
        <label>Vida <input type="number" min="1" max="32767" bind:value={nuevaEntidad.vidaMax} /></label>
        <label>Armadura <input type="number" min="0" max="32767" bind:value={nuevaEntidad.armadura} /></label>
        <label>Velocidad <input type="number" min="0" max="32767" bind:value={nuevaEntidad.velocidad} /></label>
        <label>Rango <input type="number" min="0" max="32767" bind:value={nuevaEntidad.rangoArma} /></label>
      </div>
      {#if error}<span class="error" role="alert">{error}</span>{/if}
      <button type="submit" disabled={partida.zonaId === null}>Guardar ficha</button>
      {#if partida.zonaId === null}<span class="ayuda">Crea primero una zona.</span>{/if}
    </form>
  </details>

  <div class="listas">
    {#each [
      { titulo: "JUGADORES · TODAS LAS ZONAS", entidades: jugadores },
      { titulo: "NPC · ZONA ACTUAL", entidades: npcs },
      { titulo: "ENEMIGOS · ZONA ACTUAL", entidades: enemigos }
    ] as grupo}
      <section class="grupo">
        <h3>{grupo.titulo}</h3>
        {#each grupo.entidades as item (item.id)}
          <button
            type="button"
            class="fila-entidad {item.tipo.toLowerCase()}"
            title="Arrastra al tablero para colocar · Clic o clic derecho para editar"
            onpointerdown={(evento) => iniciarArrastre(evento, item)}
            ondragstart={(evento) => evento.preventDefault()}
            onclick={() => partida.editarEntidad(item)}
            oncontextmenu={(evento) => {
              evento.preventDefault();
              partida.editarEntidad(item);
            }}
          >
            <span class="marca-tipo {item.tipo.toLowerCase()}" aria-label={item.tipo} title={item.tipo}></span>
            <span class="avatar">
              {#if fuenteFoto(item)}
                <img src={fuenteFoto(item)} alt="" draggable="false" />
              {:else}
                {item.nombre.slice(0, 1).toUpperCase()}
              {/if}
            </span>
            <span class="nombre">{item.nombre}</span>
          </button>
        {/each}
      </section>
    {/each}
  </div>
</div>

{#if arrastre}
  <div
    class="ficha-flotante"
    style:left="{arrastre.x - arrastre.offsetX}px"
    style:top="{arrastre.y - arrastre.offsetY}px"
  >
    <span class="marca-tipo {arrastre.item.tipo.toLowerCase()}" aria-label={arrastre.item.tipo}></span>
    <span class="avatar">{#if fuenteFoto(arrastre.item)}<img src={fuenteFoto(arrastre.item)} alt="" />{:else}{arrastre.item.nombre.slice(0, 1).toUpperCase()}{/if}</span>
    <span class="nombre">{arrastre.item.nombre}</span>
  </div>
{/if}

<style>
  .banco { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 5px; overflow: hidden; }
  .cabecera-banco { display: flex; flex-direction: column; gap: 4px; }
  .titulo-mini, .grupo h3 { margin: 0; color: #71717a; font-size: 10px; font-weight: 700; }
  input, select { box-sizing: border-box; min-width: 0; border: 1px solid #3f3f46; border-radius: 4px; padding: 4px; color: #fafafa; background: #09090b; font: inherit; }
  .crear-entidad { padding: 5px; border: 1px solid #3f3f46; border-radius: 5px; background: #1b1b1f; }
  .crear-entidad summary { cursor: pointer; color: #d4d4d8; }
  .crear-entidad form { display: flex; flex-direction: column; gap: 5px; padding-top: 6px; }
  .crear-entidad button { border: 0; border-radius: 4px; padding: 5px; color: white; background: #2563eb; cursor: pointer; }
  .crear-entidad button:disabled { opacity: .5; cursor: not-allowed; }
  .campos { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .campos label { display: flex; flex-direction: column; gap: 2px; color: #a1a1aa; }
  .error { color: #fca5a5; }
  .ayuda { color: #a1a1aa; }
  .listas { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 8px; }
  .grupo { display: flex; flex-direction: column; gap: 3px; }
  .grupo h3 { padding-bottom: 2px; border-bottom: 1px solid #3f3f46; }
  .fila-entidad, .ficha-flotante { display: flex; align-items: center; gap: 6px; box-sizing: border-box; width: 100%; padding: 5px; border: 1px solid #3f3f46; border-radius: 4px; color: #fafafa; background: #27272a; font: inherit; text-align: left; }
  .fila-entidad { cursor: grab; touch-action: none; }
  .fila-entidad:active { cursor: grabbing; }
  .avatar { display: grid; width: 24px; height: 24px; flex: 0 0 24px; place-items: center; overflow: hidden; border: 1px solid #52525b; background: #18181b; font-weight: 700; }
  .avatar img { width: 100%; height: 100%; object-fit: cover; }
  .marca-tipo { width: 12px; height: 12px; flex: 0 0 12px; background: #a1a1aa; }
  .marca-tipo.jugador { border-radius: 50%; background: #22c55e; }
  .marca-tipo.npc { clip-path: polygon(50% 0, 0 100%, 100% 100%); background: #3b82f6; }
  .marca-tipo.enemigo { background: #ef4444; }
  .nombre { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .ficha-flotante { position: fixed; z-index: 1000; width: 180px; pointer-events: none; opacity: .9; box-shadow: 0 6px 16px #0009; }
</style>
