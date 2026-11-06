<!-- src/routes/componentes/panel-derecho/TiradasChat.svelte -->
<!-- Tiradas y chat integrados en el mismo recuadro: la lista arriba y la caja de comandos abajo. -->
<script>
  import ListaTiradas from "./ListaTiradas.svelte";
  import EntradaComando from "./EntradaComando.svelte";
</script>

<span class="titulo-mini">TIRADAS</span>
<div class="recuadro-chat-tiradas">
  <ListaTiradas />
  <EntradaComando />
</div>

<style>
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }
  .recuadro-chat-tiradas {
    flex: 1;
    background: #09090b;
    border: 1px solid #27272a;
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
</style>
