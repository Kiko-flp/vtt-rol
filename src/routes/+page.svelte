<!-- src/routes/+page.svelte -->
<script>
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import PanelIzquierdo from "./componentes/PanelIzquierdo.svelte";
  import PanelDerecho from "./componentes/PanelDerecho.svelte";
  import Tablero from "./componentes/Tablero.svelte";
  import BarraRejilla from "./componentes/BarraRejilla.svelte";
  import BarraDados from "./componentes/BarraDados.svelte";
  import ModalFicha from "./componentes/ModalFicha.svelte";

  let zonas = [];
  let mapas = [];
  let bancoEntidades = [];
  let instanciasActivas = [];
  let historialTiradas = [];

  let zonaSeleccionadaId = 1;
  let mapaActual = { ancho: 1000, alto: 1000, rejilla: 40, grosor: 1 };
  let instanciaSeleccionada = null;
  let fichaParaEditar = null;

  let comandoChat = "";
  let panelDerechoRef;

  let rejilla = {
    tamano: 40,
    color: "#ffffff",
    grosor: 1,
    opacidad: 0.3
  };

  onMount(async () => {
    try {
      zonas = await invoke("obtener_zonas");
      mapas = await invoke("obtener_mapas_zona", { zonaId: 1 });
      if (mapas.length > 0) aplicarMapa(mapas[0]);
      bancoEntidades = await invoke("obtener_banco_entidades");
      instanciasActivas = await invoke("obtener_instancias_activas");
      historialTiradas = await invoke("obtener_historial_tiradas");
    } catch (err) {
      console.error("Error al obtener datos de Rust:", err);
    }
  });

  async function cambiarZona(id) {
    zonaSeleccionadaId = id;
    mapas = await invoke("obtener_mapas_zona", { zonaId: id });
    if (mapas.length > 0) aplicarMapa(mapas[0]);
  }

  function aplicarMapa(m) {
    mapaActual = m;
    rejilla.tamano = m.rejilla;
    rejilla.grosor = m.grosor;
    rejilla.opacidad = m.color_opacidad[3] / 255;
    rejilla = { ...rejilla };
  }

  function sumarAlChat(fragmento) {
    comandoChat = comandoChat ? `${comandoChat} + ${fragmento}` : fragmento;
    panelDerechoRef?.enfocarChat();
  }

  function restarDelChat(fragmento) {
    comandoChat = comandoChat ? `${comandoChat} - ${fragmento}` : `-${fragmento}`;
    panelDerechoRef?.enfocarChat();
  }

  function ejecutarTiradaDesdeBoton(bonif) {
    if (!comandoChat.trim()) return;
    let final = comandoChat;
    if (bonif !== 0) {
      final += bonif > 0 ? ` + ${bonif}` : ` - ${Math.abs(bonif)}`;
    }
    procesarTirada(final);
    comandoChat = "";
  }

  function procesarTirada(cmd) {
    const ahora = new Date();
    const horaStr = ahora.toTimeString().split(' ')[0]; // HH:MM:SS
    historialTiradas = [
      { id: Date.now(), comando: cmd, total: Math.floor(Math.random() * 20) + 1, hora: horaStr },
      ...historialTiradas
    ];
  }

  function modificarVida(inst, delta) {
    inst.vida_actual += delta;
    instanciasActivas = [...instanciasActivas];
  }
</script>

<div class="contenedor-principal">
  <PanelIzquierdo 
    {zonas}
    {mapas}
    banco={bancoEntidades}
    {zonaSeleccionadaId}
    onSeleccionarZona={cambiarZona}
    onSeleccionarMapa={aplicarMapa}
  />

  <main class="area-central">
    <div class="barra-superior">
      <BarraRejilla bind:rejilla />
    </div>

    <div class="contenedor-lienzo">
      <Tablero mapa={mapaActual} {rejilla} />
    </div>

    <div class="barra-inferior">
      <BarraDados 
        onSumarAlChat={sumarAlChat}
        onRestarDelChat={restarDelChat}
        onLanzarTirada={ejecutarTiradaDesdeBoton}
      />
    </div>
  </main>

  <PanelDerecho 
    bind:this={panelDerechoRef}
    instancias={instanciasActivas}
    tiradas={historialTiradas}
    {instanciaSeleccionada}
    bind:comandoChat
    onSeleccionarInstancia={(i) => instanciaSeleccionada = i}
    onModificarVida={modificarVida}
    onEnviarTirada={procesarTirada}
  />

  {#if fichaParaEditar}
    <ModalFicha 
      entidad={fichaParaEditar} 
      {zonas} 
      onCerrar={() => fichaParaEditar = null} 
    />
  {/if}
</div>

<style>
  :global(body, html) {
    margin: 0; padding: 0; width: 100vw; height: 100vh;
    overflow: hidden; background: #09090b; color: #fafafa;
    font-family: -apple-system, BlinkMacSystemFont, sans-serif;
  }
  .contenedor-principal { display: flex; width: 100vw; height: 100vh; }
  .area-central { flex: 1; position: relative; display: flex; flex-direction: column; overflow: hidden; }
  .contenedor-lienzo { flex: 1; width: 100%; height: 100%; }
  .barra-superior { position: absolute; top: 8px; left: 50%; transform: translateX(-50%); z-index: 10; }
  .barra-inferior { position: absolute; bottom: 8px; left: 50%; transform: translateX(-50%); z-index: 10; }
</style>