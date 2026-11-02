<!-- src/routes/componentes/ModalFicha.svelte -->
<script>
  export let entidad; // Instancia o Entidad directa de Rust
  export let zonas = [];
  export let onCerrar;

  let pestanaActiva = 'texto';

  // Obtener datos independientemente de si nos pasan la Entidad o la Instancia
  $: datosBase = entidad?.base ? entidad.base : entidad;
  $: vidaActual = entidad?.vida_actual !== undefined ? entidad.vida_actual : datosBase?.vida_max;
</script>

<div class="fondo-oscuro" on:click={onCerrar}>
  <div class="ventana-ficha" on:click|stopPropagation>
    <header class="cabecera">
      <h2>Ficha de Entidad (Rolmaster)</h2>
      <button class="btn-cerrar" on:click={onCerrar}>✕</button>
    </header>

    <div class="cuerpo-ficha">
      <div class="columna-foto">
        <div class="foto-placeholder">
          {#if datosBase?.foto_url}
            <img src={datosBase.foto_url} alt={datosBase.nombre} />
          {:else}
            <span>Foto (Público)</span>
          {/if}
        </div>
      </div>

      <div class="columna-datos">
        <div class="fila">
          <input type="text" bind:value={datosBase.nombre} class="input-nombre" />
          <label><input type="checkbox" bind:checked={datosBase.es_publico} /> Hacer público</label>
        </div>

        <div class="fila parametros">
          <label>Vida Máx: <input type="number" bind:value={datosBase.vida_max} /></label>
          <label>Armadura: <input type="number" bind:value={datosBase.armadura} /></label>
          <label>Velocidad: <input type="number" bind:value={datosBase.velocidad} /></label>
          <label>Rango: <input type="number" bind:value={datosBase.rango_arma} /></label>
        </div>

        <div class="fila">
          <label>Zona asociada:</label>
          <select bind:value={datosBase.zona_id}>
            {#each zonas as z}
              <option value={z.id}>{z.nombre}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <div class="pestanas">
      <button class:activa={pestanaActiva === 'texto'} on:click={() => pestanaActiva = 'texto'}>Notas</button>
      <button class:activa={pestanaActiva === 'pdf'} on:click={() => pestanaActiva = 'pdf'}>Archivos ({datosBase?.archivos_adjuntos?.length || 0})</button>
    </div>

    <div class="contenido-pestana">
      {#if pestanaActiva === 'texto'}
        <textarea placeholder="Notas privadas del máster..."></textarea>
      {:else}
        <div class="panel-archivos">
          {#each datosBase?.archivos_adjuntos || [] as archivo}
            {#if archivo}
              <div class="item-archivo">📄 {archivo}</div>
            {/if}
          {/each}
          {#if !datosBase?.archivos_adjuntos?.length}
            <span>Sin archivos adjuntos</span>
          {/if}
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .fondo-oscuro {
    position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
    background: rgba(0,0,0,0.7); display: flex; justify-content: center; align-items: center; z-index: 100;
  }
  .ventana-ficha {
    background: #18181b; border: 1px solid #3f3f46; border-radius: 8px; width: 560px; padding: 14px; display: flex; flex-direction: column; gap: 10px;
  }
  .cabecera { display: flex; justify-content: space-between; align-items: center; }
  .btn-cerrar { background: none; border: none; color: white; cursor: pointer; font-size: 16px; }
  .cuerpo-ficha { display: flex; gap: 14px; }
  .foto-placeholder { width: 120px; height: 120px; background: #27272a; border: 1px dashed #52525b; display: flex; align-items: center; justify-content: center; font-size: 11px; }
  .columna-datos { flex: 1; display: flex; flex-direction: column; gap: 6px; font-size: 12px; }
  .input-nombre { font-size: 15px; font-weight: bold; background: #09090b; border: 1px solid #3f3f46; color: white; padding: 4px; }
  .fila { display: flex; gap: 10px; align-items: center; }
  .parametros input { width: 45px; background: #09090b; border: 1px solid #3f3f46; color: white; }
  .pestanas { display: flex; gap: 6px; border-bottom: 1px solid #3f3f46; }
  .pestanas button { background: none; border: none; color: #a1a1aa; padding: 4px 10px; cursor: pointer; }
  .pestanas button.activa { color: white; border-bottom: 2px solid #ef4444; }
  textarea { width: 100%; height: 100px; background: #09090b; border: 1px solid #3f3f46; color: white; padding: 6px; box-sizing: border-box; }
  .panel-archivos { height: 100px; background: #09090b; display: flex; flex-direction: column; gap: 4px; padding: 6px; }
</style>