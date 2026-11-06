<!-- src/routes/componentes/PanelIzquierdo.svelte -->
<!-- Solo la estructura del panel: cada parte vive en su componente y lee el estado de `partida`. -->
<script>
  import ListaZonas from "./panel-izquierdo/ListaZonas.svelte";
  import ListaMapas from "./panel-izquierdo/ListaMapas.svelte";
  import BancoEntidades from "./panel-izquierdo/BancoEntidades.svelte";
</script>

<aside class="panel-izquierdo">
  <div class="bloque-arriba">
    <div class="subseccion"><ListaZonas /></div>
    <div class="subseccion"><ListaMapas /></div>
  </div>

  <div class="bloque-abajo">
    <BancoEntidades />
  </div>
</aside>

<style>
  .panel-izquierdo {
    width: clamp(180px, 14vw, 225px);
    height: 100%;
    background: #141416;
    border-right: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    box-sizing: border-box;
    padding: 6px;
    gap: 6px;
    user-select: none;
    font-size: 11px;
  }
  .bloque-arriba { height: 44%; display: flex; gap: 4px; border-bottom: 1px solid #27272a; padding-bottom: 4px; }
  .subseccion { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .bloque-abajo { flex: 1; display: flex; flex-direction: column; overflow: hidden; gap: 5px; }
</style>
