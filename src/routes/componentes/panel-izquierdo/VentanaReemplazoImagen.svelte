<!-- src/routes/componentes/panel-izquierdo/VentanaReemplazoImagen.svelte -->
<!-- Ventanita flotante (movible) que pide confirmar antes de reemplazar la imagen de un mapa. -->
<script lang="ts">
  import type { Mapa } from "$lib/partida.svelte";

  let { mapa, onConfirmar, onCancelar }: { mapa: Mapa; onConfirmar: () => void; onCancelar: () => void } = $props();

  let pos = $state({ x: 240, y: 90 });

  function iniciarArrastre(e: PointerEvent) {
    if (e.button !== 0) return;
    const inicioX = e.clientX;
    const inicioY = e.clientY;
    const origen = { ...pos };

    const alMover = (ev: PointerEvent) => {
      pos.x = Math.max(10, Math.min(window.innerWidth - 260, origen.x + (ev.clientX - inicioX)));
      pos.y = Math.max(10, Math.min(window.innerHeight - 150, origen.y + (ev.clientY - inicioY)));
    };
    const alSoltar = () => {
      window.removeEventListener("pointermove", alMover);
      window.removeEventListener("pointerup", alSoltar);
    };

    window.addEventListener("pointermove", alMover);
    window.addEventListener("pointerup", alSoltar);
  }
</script>

<div class="ventana-flotante" style="left: {pos.x}px; top: {pos.y}px;">
  <div class="cabecera-flotante" role="presentation" onpointerdown={iniciarArrastre}>
    <span class="icono-arrastre">⋮⋮</span>
    <span class="titulo-flotante">Reemplazar imagen</span>
  </div>

  <div class="cuerpo-flotante">
    <p class="texto-modal">
      El mapa "<strong>{mapa.nombre}</strong>" ya tiene una imagen.
      <span>¿Deseas reemplazarla?</span>
    </p>

    <div class="acciones-modal">
      <button type="button" class="btn-cancelar" onclick={onCancelar}>Cancelar</button>
      <button type="button" class="btn-confirmar" onclick={onConfirmar}>Reemplazar</button>
    </div>
  </div>
</div>

<style>
  .ventana-flotante {
    position: fixed;
    width: 250px;
    background: #18181b;
    border: 1px solid #3f3f46;
    border-radius: 6px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.7);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    user-select: none;
  }
  .cabecera-flotante {
    background: #27272a;
    padding: 6px 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: grab;
    border-bottom: 1px solid #3f3f46;
  }
  .cabecera-flotante:active { cursor: grabbing; }
  .icono-arrastre { color: #71717a; font-size: 11px; letter-spacing: -1px; }
  .titulo-flotante { font-size: 11px; font-weight: 600; color: #e4e4e7; }
  .cuerpo-flotante { padding: 10px 12px; display: flex; flex-direction: column; gap: 12px; }
  .texto-modal { margin: 0; color: #e4e4e7; font-size: 11px; line-height: 1.4; }
  .texto-modal span { display: block; margin-top: 3px; color: #a1a1aa; font-size: 10px; }
  .acciones-modal { display: flex; gap: 8px; justify-content: flex-end; }
  .btn-cancelar, .btn-confirmar {
    border: none; padding: 5px 10px; border-radius: 4px; font-size: 11px; font-weight: 500; cursor: pointer;
  }
  .btn-cancelar { background: #27272a; color: #d4d4d8; }
  .btn-cancelar:hover { background: #3f3f46; }
  .btn-confirmar { background: #2563eb; color: #ffffff; }
  .btn-confirmar:hover { background: #1d4ed8; }
</style>
