<!-- src/routes/componentes/BarraDados.svelte -->
<script lang="ts">
  export let onLanzarTirada: (dados: string, bonif: number) => void;

  let dadoSeleccionado = "d20";
  let bonificador = 0;

  const dados = [
    { label: "d4", color: "#22c55e" },
    { label: "d6", color: "#06b6d4" },
    { label: "d8", color: "#8b5cf6" },
    { label: "d10", color: "#ec4899" },
    { label: "d12", color: "#f97316" },
    { label: "d20", color: "#eab308" }
  ];
</script>

<div class="barra-dados">
  <div class="selectores-dado">
    {#each dados as d}
      <button 
        class="btn-dado" 
        class:activo={dadoSeleccionado === d.label}
        style="--color-dado: {d.color}"
        on:click={() => dadoSeleccionado = d.label}
      >
        {d.label}
      </button>
    {/each}
  </div>

  <div class="bonificador">
    <label>+ Bonf:</label>
    <input type="number" bind:value={bonificador} />
  </div>

  <button class="btn-tirar" on:click={() => onLanzarTirada(dadoSeleccionado, bonificador)}>
    TIRAR
  </button>
</div>

<style>
  .barra-dados {
    display: flex;
    align-items: center;
    gap: 12px;
    background: #18181b;
    padding: 8px 16px;
    border-radius: 8px;
    border: 1px solid #3f3f46;
  }
  .selectores-dado { display: flex; gap: 6px; }
  .btn-dado {
    background: #27272a;
    border: 2px solid var(--color-dado);
    color: white;
    font-weight: bold;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
  }
  .btn-dado.activo {
    background: var(--color-dado);
    color: black;
  }
  .bonificador {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
  }
  .bonificador input {
    width: 45px;
    background: #09090b;
    border: 1px solid #52525b;
    color: white;
    padding: 4px;
    border-radius: 4px;
  }
  .btn-tirar {
    background: #dc2626;
    color: white;
    font-weight: bold;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
</style>