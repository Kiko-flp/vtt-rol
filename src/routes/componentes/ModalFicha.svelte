<!-- src/routes/componentes/ModalFicha.svelte -->
<script lang="ts">
  import type { Entidad, Zona } from '$lib/types';

  export let entidad: Entidad;
  export let zonas: Zona[];
  export let onCerrar: () => void;

  let pestanaActiva: 'texto' | 'pdf' = 'texto';
</script>

<div class="fondo-oscuro" on:click={onCerrar}>
  <div class="ventana-ficha" on:click|stopPropagation>
    <header class="cabecera">
      <h2>Ficha de Entidad (Rolmaster)</h2>
      <button class="btn-cerrar" on:click={onCerrar}>✕</button>
    </header>

    <div class="cuerpo-ficha">
      <!-- Columna Izquierda: Foto -->
      <div class="columna-foto">
        <div class="foto-placeholder">
          <span>Foto (Público siempre)</span>
        </div>
      </div>

      <!-- Columna Derecha: Parámetros -->
      <div class="columna-datos">
        <div class="fila">
          <input type="text" bind:value={entidad.nombre} class="input-nombre" />
          <label><input type="checkbox" bind:checked={entidad.esPublico} /> Hacer público</label>
        </div>

        <div class="fila parametros">
          <label>Vida: <input type="number" bind:value={entidad.vidaActual} /></label>
          <label>Armadura: <input type="number" bind:value={entidad.armadura} /></label>
        </div>

        <div class="fila">
          <label>Zona asociada:</label>
          <select bind:value={entidad.zonaId}>
            {#each zonas as z}
              <option value={z.id}>{z.nombre}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <!-- Pestañas inferiores: Notas o PDF -->
    <div class="pestanas">
      <button 
        class:activa={pestanaActiva === 'texto'} 
        on:click={() => pestanaActiva = 'texto'}
      >
        Texto del programa
      </button>
      <button 
        class:activa={pestanaActiva === 'pdf'} 
        on:click={() => pestanaActiva = 'pdf'}
      >
        PDF Adjunto
      </button>
    </div>

    <div class="contenido-pestana">
      {#if pestanaActiva === 'texto'}
        <textarea placeholder="Notas privadas del máster sobre este personaje..."></textarea>
      {:else}
        <div class="panel-pdf">
          <p>Visor de PDF preparado. (Privado del Rolmaster)</p>
          <button>Sacar PDF a ventana flotante</button>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .fondo-oscuro {
    position: fixed;
    top: 0; left: 0; width: 100vw; height: 100vh;
    background: rgba(0,0,0,0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 100;
  }
  .ventana-ficha {
    background: #18181b;
    border: 1px solid #3f3f46;
    border-radius: 8px;
    width: 600px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .cabecera { display: flex; justify-content: space-between; align-items: center; }
  .btn-cerrar { background: none; border: none; color: white; cursor: pointer; font-size: 18px; }
  .cuerpo-ficha { display: flex; gap: 16px; }
  .foto-placeholder {
    width: 140px;
    height: 140px;
    background: #27272a;
    border: 1px dashed #52525b;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
    font-size: 12px;
  }
  .columna-datos { flex: 1; display: flex; flex-direction: column; gap: 8px; font-size: 13px; }
  .input-nombre { font-size: 16px; font-weight: bold; background: #09090b; border: 1px solid #3f3f46; color: white; padding: 4px; }
  .fila { display: flex; gap: 12px; align-items: center; }
  .parametros input { width: 50px; background: #09090b; border: 1px solid #3f3f46; color: white; }
  
  .pestanas { display: flex; gap: 8px; border-bottom: 1px solid #3f3f46; }
  .pestanas button {
    background: none; border: none; color: #a1a1aa; padding: 6px 12px; cursor: pointer;
  }
  .pestanas button.activa { color: white; border-bottom: 2px solid #ef4444; }
  
  textarea {
    width: 100%;
    height: 120px;
    background: #09090b;
    border: 1px solid #3f3f46;
    color: white;
    padding: 8px;
    box-sizing: border-box;
  }
  .panel-pdf {
    height: 120px;
    background: #09090b;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
  }
</style>