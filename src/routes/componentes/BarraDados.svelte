<!-- src/routes/componentes/BarraDados.svelte -->
<script>
  export let onSumarAlChat;
  export let onRestarDelChat;
  export let onLanzarTirada;

  let bonificador = 0;

  let dados = [
    { tipo: "d4", cantidad: 1, color: "#22c55e" },
    { tipo: "d6", cantidad: 1, color: "#06b6d4" },
    { tipo: "d8", cantidad: 1, color: "#8b5cf6" },
    { tipo: "d10", cantidad: 1, color: "#ec4899" },
    { tipo: "d12", cantidad: 1, color: "#f97316" },
    { tipo: "d20", cantidad: 1, color: "#eab308" }
  ];

  function rueda(e, d) {
    e.preventDefault();
    if (e.deltaY < 0) d.cantidad++;
    else if (d.cantidad > 1) d.cantidad--;
    dados = [...dados];
  }
</script>

<div class="barra-dados">
  <div class="zona-dados">
    {#each dados as d}
      <button 
        class="boton-dado" 
        style="border-color: {d.color};"
        on:wheel={(e) => rueda(e, d)}
        on:click={() => onSumarAlChat(`${d.cantidad}${d.tipo}`)}
        on:contextmenu={(e) => { e.preventDefault(); onRestarDelChat(`${d.cantidad}${d.tipo}`); }}
        title="Rueda: cantidad | Clic: sumar | Clic derecho: restar"
      >
        <span class="cant">{d.cantidad}x</span>
        <span class="label">{d.tipo}</span>
      </button>
    {/each}
  </div>

  <div class="bonif">
    <span>+ Bonf:</span>
    <input type="number" bind:value={bonificador} />
  </div>

  <button class="btn-tirar" on:click={() => onLanzarTirada(bonificador)}>
    TIRAR
  </button>
</div>

<style>
  .barra-dados {
    display: flex;
    align-items: center;
    gap: 10px;
    background: #18181b;
    border: 1px solid #3f3f46;
    padding: 6px 12px;
    border-radius: 8px;
  }
  .zona-dados { display: flex; gap: 5px; }
  .boton-dado {
    background: #27272a;
    color: white;
    border: 2px solid;
    padding: 3px 6px;
    border-radius: 5px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    font-size: 11px;
    font-weight: bold;
    user-select: none;
  }
  .cant { font-size: 9px; color: #a1a1aa; }
  .bonif { display: flex; align-items: center; gap: 4px; font-size: 11px; }
  .bonif input { width: 40px; background: #09090b; border: 1px solid #3f3f46; color: white; padding: 4px; border-radius: 3px; }
  .btn-tirar {
    background: #dc2626; color: white; border: none; font-weight: bold; padding: 6px 14px; border-radius: 4px; cursor: pointer;
  }
</style>