<script lang="ts">
  let mensaje: string = "¡Lienzo listo para el VTT!";
</script>

<main class="contenedor-principal">
  <h1>{mensaje}</h1>
</main>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    background-color: #1a1a1a;
    color: #ffffff;
    font-family: sans-serif;
  }

  .contenedor-principal {
    display: flex;
    width: 100vw;
    height: 100vh;
    justify-content: center;
    align-items: center;
  }
</style>