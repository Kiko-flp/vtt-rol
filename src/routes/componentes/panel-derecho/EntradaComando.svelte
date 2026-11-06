<!-- src/routes/componentes/panel-derecho/EntradaComando.svelte -->
<script lang="ts">
  import { tiradas } from "$lib/tiradas.svelte";

  let input: HTMLInputElement | null = null;

  // La barra de dados le pide el foco a través del store: ya no hace falta bind:this desde la página.
  $effect(() => tiradas.registrarFoco(() => input?.focus()));

  function colocarCursorAlFinal() {
    requestAnimationFrame(() => {
      if (input) input.setSelectionRange(input.value.length, input.value.length);
    });
  }

  async function alTeclear(e: KeyboardEvent) {
    if (e.key === "Enter") {
      await tiradas.enviar();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (await tiradas.historialAnterior()) colocarCursorAlFinal();
    } else if (e.key === "ArrowDown") {
      if (!tiradas.navegandoHistorial) return; // si no se estaba navegando, no hace nada
      e.preventDefault();
      if (tiradas.historialSiguiente()) colocarCursorAlFinal();
    }
  }
</script>

<div class="input-comando-contenedor">
  <input
    type="text"
    bind:this={input}
    bind:value={tiradas.comando}
    placeholder="Escribir comando (ej: 2d6 + 3)..."
    onkeydown={alTeclear}
    class="input-chat"
  />
</div>

<style>
  .input-comando-contenedor {
    border-top: 1px solid #27272a; /* Línea fina separadora */
    background: #111113;
    padding: 4px 6px;
  }
  .input-chat {
    width: 100%;
    background: transparent;
    border: none;
    color: white;
    font-size: 12px;
    outline: none;
    box-sizing: border-box;
  }
</style>
