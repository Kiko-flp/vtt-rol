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
    mapaActual = null
  }: {
    rejilla: Rejilla;
    mapaActual?: any;
  } = $props();

  const TAMANO_MIN = 5;
  const TAMANO_MAX = 300;
  const TAMANO_DEFECTO = 40;

  let barraRef: HTMLDivElement | null = $state(null);
  let temporizadorGuardado: ReturnType<typeof setTimeout> | null = null;
  let hayCambiosPendientes = false;
  let mapaActivoCopia: any = null;

  function hexToRgba(hex: string, opacidad: number): number[] {
    const limpio = hex.replace("#", "");
    const num = parseInt(limpio.length === 3 ? limpio.split("").map((x) => x + x).join("") : limpio, 16) || 0;
    const r = (num >> 16) & 255;
    const g = (num >> 8) & 255;
    const b = num & 255;
    const a = Math.round(Math.min(1, Math.max(0, opacidad)) * 255);
    return [r, g, b, a];
  }

  // Guardado al cambiar de mapa
  $effect(() => {
    const idActual = mapaActual?.id;
    if (mapaActivoCopia && mapaActivoCopia.id !== idActual) {
      if (hayCambiosPendientes) {
        guardarCambios(mapaActivoCopia);
      }
    }
    mapaActivoCopia = mapaActual ? { ...mapaActual } : null;
  });

  function registrarModificacion() {
    hayCambiosPendientes = true;
    if (temporizadorGuardado) clearTimeout(temporizadorGuardado);
    // Solo espera 5s si el usuario NO hace nada más
    temporizadorGuardado = setTimeout(() => {
      guardarCambios(mapaActual);
    }, 5000);
  }

  export async function forzarGuardadoInmediato() {
    if (hayCambiosPendientes) {
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
    } catch (err) {
      console.error("Error al persistir cambios de rejilla en Rust:", err);
    }
  }

  // Guardado inmediato al pulsar en cualquier otra parte de la ventana
  onMount(() => {
    function onPointerDownGlobal(e: PointerEvent) {
      if (barraRef && barraRef.contains(e.target as Node)) {
        return;
      }
      if (hayCambiosPendientes) {
        guardarCambios(mapaActual);
      }
    }

    window.addEventListener("pointerdown", onPointerDownGlobal);
    return () => {
      window.removeEventListener("pointerdown", onPointerDownGlobal);
      if (hayCambiosPendientes) guardarCambios(mapaActual);
    };
  });

  function normalizarTamano() {
    const t = Math.round(Number(rejilla.tamano));
    rejilla.tamano = Number.isFinite(t) && t > 0 ? Math.min(TAMANO_MAX, Math.max(TAMANO_MIN, t)) : TAMANO_DEFECTO;
    registrarModificacion();
  }

  function normalizarGrosor() {
    const g = Number(rejilla.grosor);
    rejilla.grosor = Number.isFinite(g) && g > 0 ? Math.min(5, Math.max(0.5, g)) : 1;
    registrarModificacion();
  }
</script>

<div class="barra-herramientas" bind:this={barraRef}>
  <button class="btn-lapiz" title="Modo Dibujo">✏️</button>

  <label class="control" title="Mostrar u ocultar la rejilla">
    <input
      type="checkbox"
      checked={rejilla.visible !== false}
      onchange={(e) => {
        rejilla.visible = e.currentTarget.checked;
        registrarModificacion();
      }}
    />
    <span>Rejilla</span>
  </label>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-tamano">Celda:</label>
    <input
      id="rejilla-tamano"
      type="number"
      bind:value={rejilla.tamano}
      oninput={registrarModificacion}
      onchange={normalizarTamano}
      min={TAMANO_MIN}
      max={TAMANO_MAX}
      step="5"
    />
    <span>px</span>
  </div>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-color">Color:</label>
    <input
      id="rejilla-color"
      type="color"
      bind:value={rejilla.color}
      oninput={registrarModificacion}
    />
  </div>

  <div class="control" class:apagado={rejilla.visible === false}>
    <label for="rejilla-grosor">Grosor:</label>
    <input
      id="rejilla-grosor"
      type="number"
      bind:value={rejilla.grosor}
      oninput={registrarModificacion}
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