<script>
  import { urlDeImagen } from "$lib/imagenes";

  /**
   * @typedef {import("$lib/partida.svelte").Entidad} Entidad
   * @typedef {import("$lib/partida.svelte").Zona} Zona
   */
  /** @type {{
   * entidad: Entidad,
   * zonas: Zona[],
   * onGuardar: (entidad: Entidad, fotoData: string | null, quitarFoto: boolean) => Promise<void>,
   * onCerrar: () => void
   * }} */
  let {
    entidad,
    zonas = [],
    onGuardar,
    onCerrar
  } = $props();

  let borrador = $state({ ...entidad });
  /** @type {HTMLInputElement | null} */
  let inputFotoRef = $state(null);
  /** @type {string | null} */
  let fotoPendiente = $state(null);
  let quitarFoto = $state(false);
  let error = $state("");
  let guardando = $state(false);

  const fuenteFoto = $derived(fotoPendiente || urlDeImagen(borrador.foto_url));

  /** @param {File} archivo */
  function procesarImagen(archivo) {
    return new Promise((resolve, reject) => {
      if (!archivo.type.startsWith("image/")) {
        reject(new Error("Selecciona un archivo de imagen."));
        return;
      }
      if (archivo.size > 10 * 1024 * 1024) {
        reject(new Error("La imagen original debe ocupar menos de 10 MB."));
        return;
      }

      const imagen = new Image();
      const lector = new FileReader();
      lector.onerror = () => reject(new Error("No se pudo leer el archivo de imagen."));
      lector.onload = () => {
        imagen.onerror = () => reject(new Error("El archivo seleccionado no es una imagen válida."));
        imagen.onload = () => {
          const escala = Math.min(1, 512 / Math.max(imagen.naturalWidth, imagen.naturalHeight));
          const canvas = document.createElement("canvas");
          canvas.width = Math.max(1, Math.round(imagen.naturalWidth * escala));
          canvas.height = Math.max(1, Math.round(imagen.naturalHeight * escala));
          const contexto = canvas.getContext("2d");
          if (!contexto) {
            reject(new Error("No se pudo procesar la imagen."));
            return;
          }
          contexto.drawImage(imagen, 0, 0, canvas.width, canvas.height);
          resolve(canvas.toDataURL("image/webp", 0.82));
        };
        imagen.src = String(lector.result);
      };
      lector.readAsDataURL(archivo);
    });
  }

  /** @param {Event & { currentTarget: HTMLInputElement }} evento */
  async function alSeleccionarFoto(evento) {
    const archivo = evento.currentTarget.files?.[0];
    evento.currentTarget.value = "";
    if (!archivo) return;
    try {
      fotoPendiente = await procesarImagen(archivo);
      quitarFoto = false;
      error = "";
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    }
  }

  function quitarImagen() {
    fotoPendiente = null;
    quitarFoto = true;
    borrador.foto_url = null;
  }

  async function guardar() {
    if (!borrador.nombre.trim()) {
      error = "El nombre de la ficha no puede estar vacío.";
      return;
    }
    if (Number(borrador.vida_max) < 1) {
      error = "La vida máxima debe ser mayor que cero.";
      return;
    }

    guardando = true;
    error = "";
    try {
      await onGuardar(
        {
          ...borrador,
          nombre: borrador.nombre.trim(),
          vida_max: Number(borrador.vida_max),
          armadura: Number(borrador.armadura),
          velocidad: Number(borrador.velocidad),
          rango_arma: Number(borrador.rango_arma),
          zona_id: Number(borrador.zona_id)
        },
        fotoPendiente,
        quitarFoto
      );
      onCerrar();
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      guardando = false;
    }
  }

  /** @param {KeyboardEvent} evento */
  function alTeclear(evento) {
    if (evento.key === "Escape" && !guardando) onCerrar();
  }
</script>

<svelte:window onkeydown={alTeclear} />

<input
  bind:this={inputFotoRef}
  class="entrada-foto"
  type="file"
  accept="image/png,image/jpeg,image/webp,image/gif"
  onchange={alSeleccionarFoto}
/>

<div class="fondo-oscuro" role="presentation" onclick={onCerrar}>
  <section
    class="ventana-ficha"
    role="dialog"
    aria-modal="true"
    aria-labelledby="titulo-ficha"
    onclick={(evento) => evento.stopPropagation()}
    onkeydown={(evento) => evento.stopPropagation()}
  >
    <header class="cabecera">
      <div>
        <h2 id="titulo-ficha">Editar ficha</h2>
        <p>Los cambios se aplican al guardar.</p>
      </div>
      <button class="btn-icono" type="button" aria-label="Cerrar" onclick={onCerrar}>×</button>
    </header>

    <div class="editor-principal">
      <div class="foto-columna">
        <button
          type="button"
          class="previsualizacion {borrador.tipo.toLowerCase()}"
          onclick={() => inputFotoRef?.click()}
          title="Elegir una imagen"
        >
          {#if fuenteFoto}
            <img src={fuenteFoto} alt="Vista previa de {borrador.nombre}" />
          {:else}
            <span class="sin-foto">{borrador.nombre.slice(0, 2).toUpperCase() || "?"}</span>
          {/if}
          <span class="insignia-camara" aria-hidden="true">+</span>
        </button>
        {#if fuenteFoto}
          <button class="btn-quitar-foto" type="button" onclick={quitarImagen}>Quitar imagen</button>
        {/if}
        <small>PNG, JPG, WebP o GIF. Se optimiza a un máximo de 512 px.</small>
      </div>

      <div class="campos">
        <label class="campo">
          <span>Nombre</span>
          <input bind:value={borrador.nombre} maxlength="80" autocomplete="off" />
        </label>

        <label class="campo">
          <span>Tipo de ficha</span>
          <select bind:value={borrador.tipo}>
            <option value="Jugador">Jugador</option>
            <option value="NPC">NPC</option>
            <option value="Enemigo">Enemigo</option>
          </select>
        </label>

        <div class="rejilla-campos">
          <label class="campo">
            <span>Vida máxima</span>
            <input type="number" min="1" max="32767" bind:value={borrador.vida_max} />
          </label>
          <label class="campo">
            <span>Armadura</span>
            <input type="number" min="0" max="32767" bind:value={borrador.armadura} />
          </label>
          <label class="campo">
            <span>Velocidad</span>
            <input type="number" min="0" max="32767" bind:value={borrador.velocidad} />
          </label>
          <label class="campo">
            <span>Rango</span>
            <input type="number" min="0" max="32767" bind:value={borrador.rango_arma} />
          </label>
        </div>

        <label class="campo">
          <span>Zona</span>
          <select bind:value={borrador.zona_id}>
            {#each zonas as zona (zona.id)}
              <option value={zona.id}>{zona.nombre}</option>
            {/each}
          </select>
        </label>

        <label class="opcion">
          <input type="checkbox" bind:checked={borrador.es_unico} />
          <span>Ficha única <small>No se puede colocar más de una vez a la vez.</small></span>
        </label>
        <label class="opcion">
          <input type="checkbox" bind:checked={borrador.vida_es_publica} />
          <span>Vida pública</span>
        </label>
        <label class="opcion">
          <input type="checkbox" bind:checked={borrador.armadura_es_publica} />
          <span>Armadura pública</span>
        </label>
      </div>
    </div>

    {#if error}
      <p class="error" role="alert">{error}</p>
    {/if}

    <footer class="acciones">
      <button class="btn-secundario" type="button" disabled={guardando} onclick={onCerrar}>Cancelar</button>
      <button class="btn-guardar" type="button" disabled={guardando} onclick={guardar}>
        {guardando ? "Guardando…" : "Guardar cambios"}
      </button>
    </footer>
  </section>
</div>

<style>
  .entrada-foto { display: none; }
  .fondo-oscuro {
    position: fixed; inset: 0; z-index: 1000; display: flex; align-items: center; justify-content: center;
    padding: 16px; background: #000b; box-sizing: border-box;
  }
  .ventana-ficha {
    width: min(620px, 100%); max-height: min(90vh, 760px); overflow: auto; box-sizing: border-box;
    display: flex; flex-direction: column; gap: 16px; padding: 20px;
    color: #f4f4f5; background: #18181b; border: 1px solid #3f3f46; border-radius: 12px;
    box-shadow: 0 20px 60px #000b;
  }
  .cabecera, .acciones { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
  .cabecera h2 { margin: 0; font-size: 19px; }
  .cabecera p { margin: 4px 0 0; color: #a1a1aa; font-size: 12px; }
  .btn-icono { border: 0; color: #d4d4d8; background: transparent; font-size: 24px; cursor: pointer; }
  .editor-principal { display: grid; grid-template-columns: 170px minmax(0, 1fr); gap: 22px; }
  .foto-columna { display: flex; flex-direction: column; align-items: center; gap: 8px; }
  .previsualizacion {
    position: relative; width: 142px; height: 142px; overflow: hidden; border: 4px solid #fafafa;
    border-radius: 0; padding: 0; color: white; background: #27272a; cursor: pointer;
  }
  .previsualizacion img { width: 100%; height: 100%; object-fit: cover; }
  .sin-foto { display: grid; width: 100%; height: 100%; place-items: center; font-size: 34px; font-weight: 700; }
  .insignia-camara { position: absolute; left: calc(50% - 13px); bottom: 8px; width: 26px; height: 26px; display: grid; place-items: center; border-radius: 50%; background: #09090b; font-size: 20px; }
  .foto-columna > small { color: #a1a1aa; font-size: 10px; text-align: center; line-height: 1.4; }
  .campos { display: flex; flex-direction: column; gap: 10px; min-width: 0; }
  .campo { display: flex; flex-direction: column; gap: 5px; color: #d4d4d8; font-size: 12px; }
  .campo input, .campo select {
    width: 100%; min-width: 0; box-sizing: border-box; padding: 8px 9px;
    color: #fafafa; background: #09090b; border: 1px solid #3f3f46; border-radius: 6px; font: inherit;
  }
  .campo input:focus, .campo select:focus { outline: 2px solid #3b82f6; outline-offset: 1px; }
  .rejilla-campos { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
  .opcion { display: flex; align-items: center; gap: 8px; color: #e4e4e7; font-size: 12px; cursor: pointer; }
  .opcion input { accent-color: #3b82f6; }
  .opcion small { display: block; margin-top: 2px; color: #a1a1aa; }
  .btn-secundario, .btn-quitar-foto, .btn-guardar { padding: 8px 12px; border-radius: 6px; font: inherit; cursor: pointer; }
  .btn-secundario { color: #e4e4e7; background: #27272a; border: 1px solid #52525b; }
  .btn-quitar-foto { color: #fecaca; background: transparent; border: 1px solid #7f1d1d; font-size: 11px; }
  .btn-guardar { color: white; background: #2563eb; border: 1px solid #3b82f6; font-weight: 600; }
  button:disabled { opacity: 0.55; cursor: wait; }
  .error { margin: 0; color: #fecaca; background: #7f1d1d55; border: 1px solid #991b1b; border-radius: 6px; padding: 8px 10px; font-size: 12px; }
  @media (max-width: 560px) {
    .ventana-ficha { padding: 14px; }
    .editor-principal { grid-template-columns: 1fr; }
    .foto-columna { display: grid; grid-template-columns: 100px 1fr; justify-items: start; }
    .previsualizacion { grid-row: span 3; width: 88px; height: 88px; }
    .foto-columna > small { text-align: left; }
  }
</style>
