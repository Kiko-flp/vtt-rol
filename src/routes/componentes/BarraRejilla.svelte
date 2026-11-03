<!-- src/routes/componentes/BarraRejilla.svelte -->
<script lang="ts">
  interface Rejilla {
    visible?: boolean;
    tamano: number; // lado de la celda, en px del mapa
    color: string;
    grosor: number; // px en pantalla
    opacidad: number; // 0..1
  }

  // $bindable: el padre usa <BarraRejilla bind:rejilla /> y los cambios le llegan.
  let { rejilla = $bindable() }: { rejilla: Rejilla } = $props();

  const TAMANO_MIN = 5;
  const TAMANO_MAX = 300;
  const TAMANO_DEFECTO = 40;

  // Un input numérico vacío o a medias da null/NaN: al terminar de escribir lo dejamos en un valor válido.
  function normalizarTamano() {
    const t = Math.round(Number(rejilla.tamano));
    rejilla.tamano = Number.isFinite(t) && t > 0 ? Math.min(TAMANO_MAX, Math.max(TAMANO_MIN, t)) : TAMANO_DEFECTO;
  }

  function normalizarGrosor() {
    const g = Number(rejilla.grosor);
    rejilla.grosor = Number.isFinite(g) && g > 0 ? Math.min(5, Math.max(0.5, g)) : 1;
  }
</script>

<div class="barra-herramientas">
  <button class="btn-lapiz" title="Modo Dibujo">✏️</button>

  <label class="control" title="Mostrar u ocultar la rejilla">
    <input
      type="checkbox"
      checked={rejilla.visible !== false}
      onchange={(e) => (rejilla.visible = e.currentTarget.checked)}
    />
    <span>Rejilla</span>
  </label>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-tamano">Celda:</label>
    <input
      id="rejilla-tamano"
      type="number"
      bind:value={rejilla.tamano}
      onchange={normalizarTamano}
      min={TAMANO_MIN}
      max={TAMANO_MAX}
      step="5"
      title="Si las celdas se ven más pequeñas de 5 px (al alejar el zoom) la rejilla se oculta sola"
    />
    <span>px</span>
  </div>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-color">Color:</label>
    <input id="rejilla-color" type="color" bind:value={rejilla.color} />
  </div>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-grosor">Grosor:</label>
    <input
      id="rejilla-grosor"
      type="number"
      bind:value={rejilla.grosor}
      onchange={normalizarGrosor}
      min="0.5"
      max="5"
      step="0.5"
    />
  </div>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-opacidad">Opacidad:</label>
    <input
      id="rejilla-opacidad"
      type="range"
      min="0"
      max="1"
      step="0.05"
      bind:value={rejilla.opacidad}
    />
    <span class="porcentaje">{Math.round(rejilla.opacidad * 100)}%</span>
  </div>
</div>

<style>
  .barra-herramientas {
    display: flex;
    align-items: center;
    gap: 15px;
    background-color: #262626;
    padding: 6px 16px;
    border-radius: 6px;
    font-size: 13px;
    border: 1px solid #3f3f46;
  }
  .control {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .control.apagado {
    opacity: 0.4;
  }
  .control input[type='number'] {
    width: 50px;
    background: #18181b;
    color: white;
    border: 1px solid #52525b;
    border-radius: 4px;
    padding: 2px 4px;
  }
  .porcentaje {
    min-width: 32px;
    text-align: right;
  }
  .btn-lapiz {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 16px;
  }
</style>
