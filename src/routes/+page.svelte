<script lang="ts">
  // 1. Importamos la función de enlace de Tauri
  import { invoke } from "@tauri-apps/api/core";

  let resultadoDado = "Esperando tirada...";

  // 2. Función asíncrona que llama a Rust
  async function pulsarBoton() {
    // El nombre aquí DEBE ser idéntico al nombre de la función en Rust
    resultadoDado = await invoke("tirar_dado");
  }
</script>

<main class="contenedor-principal">
  <h1>Lienzo listo para el VTT</h1>
  
  <!-- 3. Vinculamos el clic del botón con on:click -->
  <button class="mi-boton" on:click={pulsarBoton}>
    🎲 TIRAR DADOS
  </button>

  <p class="resultado">{resultadoDado}</p>
</main>

<style>
  /* Tu CSS tradicional y separado, como te gusta */
  :global(body, html) {
    margin: 0;
    padding: 0;
    background-color: #1a1a1a;
    color: #ffffff;
    font-family: sans-serif;
  }

  .contenedor-principal {
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    height: 100vh;
    gap: 15px;
  }

  .mi-boton {
    background-color: #b91c1c;
    color: white;
    border: none;
    padding: 12px 24px;
    font-size: 16px;
    font-weight: bold;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.2s;
  }

  .mi-boton:hover {
    background-color: #dc2626;
  }

  .resultado {
    font-size: 18px;
    color: #4ade80;
  }
</style>