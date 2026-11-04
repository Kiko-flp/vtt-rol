<!-- src/routes/componentes/BarraRejilla.svelte -->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface Rejilla {
    visible?: boolean;
    tamano: number;
    color: string;
    grosor: number;
    opacidad: number;
  }

  let {
    rejilla = $bindable(),
    mapaActual = null,
    onActualizarMapa
  }: {
    rejilla: Rejilla;
    mapaActual?: any;
    onActualizarMapa?: (mapaModificado: any) => void;
  } = $props();

  const TAMANO_MIN = 5;
  const TAMANO_MAX = 300;

  let barraRef: HTMLDivElement | null = $state(null);
  let temporizadorGuardado: ReturnType<typeof setTimeout> | null = null;
  let hayCambiosPendientes = false;

  function hexToRgba(hex: string, opacidad: number): number[] {
    const limpio = hex.replace("#", "");
    const num = parseInt(limpio.length === 3 ? limpio.split("").map((x) => x + x).join("") : limpio, 16) || 0;
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    const a = Math.round(Math.min(1, Math.max(0, opacidad)) * 255);
    return [r, g, b, a];
  }

  function registrarModificacion() {
    if (!mapaActual) return;
    hayCambiosPendientes = true;
    if (temporizadorGuardado) clearTimeout(temporizadorGuardado);
    temporizadorGuardado = setTimeout(() => {
      guardarCambios(mapaActual);
    }, 5000);
  }

  export async function forzarGuardadoInmediato() {
    if (hayCambiosPendientes && mapaActual) {
      await guardarCambios(mapaActual);
    }
  }

  async function guardarCambios(mapaRef: any) {
    if (!hayCambiosPendientes || !mapaRef || !mapaRef.id) return;
    if (temporizadorGuardado) clearTimeout(temporizadorGuardado);
    hayCambiosPendientes = false;

    const mapaModificado = {
      ...mapaRef,
      rejilla: Math.round(rejilla.tamano),
      grosor: Math.round(rejilla.grosor),
      color_opacidad: hexToRgba(rejilla.color, rejilla.opacidad)
    };

    try {
      await invoke("modificar_mapa", { mapa: mapaModificado });
      if (onActualizarMapa) {
        onActualizarMapa(mapaModificado);
      }
    } catch (err) {
      console.error("Error al persistir cambios de rejilla en Rust:", err);
    }
  }

  // Quitar el foco y guardar de inmediato si se pulsa en cualquier otra parte
  onMount(() => {
    function onPointerDownGlobal(e: PointerEvent) {
      const target = e.target as Node;
      // Si el clic fue fuera de la barra de herramientas
      if (barraRef && !barraRef.contains(target)) {
        // Desenfocar explícitamente cualquier input activo
        if (document.activeElement instanceof HTMLElement) {
          document.activeElement.blur();
        }
        if (hayCambiosPendientes && mapaActual) {
          guardarCambios(mapaActual);
        }
      }
    }

    window.addEventListener("pointerdown", onPointerDownGlobal);
    return () => {
      window.removeEventListener("pointerdown", onPointerDownGlobal);
      if (hayCambiosPendientes && mapaActual) guardarCambios(mapaActual);
    };
  });

  function normalizarTamano() {
    const t = Math.round(Number(rejilla.tamano));
    const fallbackRust = mapaActual?.rejilla || 50;
    rejilla.tamano = Number.isFinite(t) && t > 0 ? Math.min(TAMANO_MAX, Math.max(TAMANO_MIN, t)) : fallbackRust;
    registrarModificacion();
  }

  const GROSOR_MIN = 0;
  const GROSOR_MAX = 10;
  const GROSOR_PASO = 1;

  function normalizarGrosor() {
    const g = Number(rejilla.grosor);
    const fallbackRust = mapaActual?.grosor ?? 1;
    rejilla.grosor = Number.isFinite(g) && g >= GROSOR_MIN 
      ? Math.min(GROSOR_MAX, Math.max(GROSOR_MIN, g)) 
      : fallbackRust;
    registrarModificacion();
  }

  function alRuedaGrosor(e: WheelEvent) {
    if (!mapaActual) return;
    e.preventDefault();
    const delta = e.deltaY < 0 ? 1 : -1;
    
    // Si es 0 u otro número válido, lo respeta en lugar de caer al fallback
    const actual = Number.isFinite(Number(rejilla.grosor))
      ? Math.round(Number(rejilla.grosor))
      : (mapaActual.grosor ?? 1);

    rejilla.grosor = Math.min(GROSOR_MAX, Math.max(GROSOR_MIN, actual + delta));
    registrarModificacion();
  }
  
  // MODIFICACIÓN CON LA RUEDA DEL RATÓN
  function alRuedaTamano(e: WheelEvent) {
    if (!mapaActual) return;
    e.preventDefault();
    const delta = e.deltaY < 0 ? 5 : -5;
    const actual = Math.round(Number(rejilla.tamano) || mapaActual.rejilla || 50);
    rejilla.tamano = Math.min(TAMANO_MAX, Math.max(TAMANO_MIN, actual + delta));
    registrarModificacion();
  }

</script>

<div class="barra-herramientas" class:desactivada={!mapaActual} bind:this={barraRef}>
  <button class="btn-lapiz" title="Modo Dibujo" disabled={!mapaActual}>✏️</button>

  <div class="control" class:apagado={!mapaActual}>
    <label for="rejilla-tamano">Rejilla:</label>
    <input
      id="rejilla-tamano"
      type="number"
      disabled={!mapaActual}
      bind:value={rejilla.tamano}
      oninput={registrarModificacion}
      onchange={normalizarTamano}
      onwheel={alRuedaTamano}
      min={TAMANO_MIN}
      max={TAMANO_MAX}
      step="5"
      title="Rueda del ratón para subir/bajar de 5 en 5"
    />
    <span>px</span>
  </div>

  <div class="control" class:apagado={!mapaActual}>
    <label for="rejilla-color">Color:</label>
    <input
      id="rejilla-color"
      type="color"
      disabled={!mapaActual}
      bind:value={rejilla.color}
      oninput={registrarModificacion}
    />
  </div>

  <div class="control" class:apagado={!mapaActual}>
    <label for="rejilla-grosor">Grosor:</label>
    <input
      id="rejilla-grosor"
      type="number"
      disabled={!mapaActual}
      bind:value={rejilla.grosor}
      oninput={registrarModificacion}
      onchange={normalizarGrosor}
      onwheel={alRuedaGrosor}
      min={GROSOR_MIN}
      max={GROSOR_MAX}
      step={GROSOR_PASO}
      title="Rueda del ratón para subir/bajar de 1 en 1"
    />
  </div>

  <div class="control" class:apagado={!mapaActual}>
    <label for="rejilla-opacidad">Opacidad:</label>
    <input
      id="rejilla-opacidad"
      type="range"
      disabled={!mapaActual}
      min="0"
      max="1"
      step="0.05"
      bind:value={rejilla.opacidad}
      oninput={registrarModificacion}
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
    user-select: none;
  }
  .barra-herramientas.desactivada {
    opacity: 0.5;
    pointer-events: none;
  }
  .control { display: flex; align-items: center; gap: 6px; }
  .control.apagado { opacity: 0.4; }
  .control input[type="number"] {
    width: 50px;
    background: #18181b;
    color: white;
    border: 1px solid #52525b;
    border-radius: 4px;
    padding: 2px 4px;
  }
  .porcentaje { min-width: 32px; text-align: right; }
  .btn-lapiz { background: none; border: none; cursor: pointer; font-size: 16px; }
</style>