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

  let zonas = $state([]);
  let mapas = $state([]);
  let bancoEntidades = $state([]);
  let instanciasActivas = $state([]);
  let historialTiradas = $state([]);

  let zonaSeleccionadaId = $state(1);
  let mapaActual = $state({ ancho: 1000, alto: 1000, rejilla: 40, grosor: 1 });
  let instanciaSeleccionada = $state(null);
  let fichaParaEditar = $state(null);

  let comandoChat = $state("");
  let panelDerechoRef; // bind:this: solo se usa desde funciones

  let rejilla = $state({
    visible: true,
    tamano: 40,
    color: "#ffffff",
    grosor: 1,
    opacidad: 0.3
  });

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
    // Con $state, basta con asignar: ya no hace falta rejilla = { ...rejilla }
    rejilla.tamano = m.rejilla;
    rejilla.grosor = m.grosor;
    rejilla.opacidad = m.color_opacidad[3] / 255;
  }

  function actualizarDadoEnChat(fragmento, esSuma) {
    // Captura cantidad, caras y los sufijos (!, adv)
    const matchFrag = fragmento.match(/^(\d*)d(\d+)([!a-z]*)$/i);
    if (!matchFrag) return;

    const delta = (matchFrag[1] ? parseInt(matchFrag[1]) : 1) * (esSuma ? 1 : -1);
    const caras = matchFrag[2];
    const sufijos = matchFrag[3] || ""; // ej: "!", "adv", "!adv" o ""

    // Escapamos los signos para el RegExp
    const sufijosEscapados = sufijos.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(`(^|[+-])\\s*(\\d*)d${caras}${sufijosEscapados}(?=\\s|[+-]|$)`, "i");
    const match = comandoChat.match(regex);

    if (match) {
      const signoActual = match[1] === "-" ? -1 : 1;
      const vecesActuales = (match[2] ? parseInt(match[2]) : 1) * signoActual;
      const nuevaCantidad = vecesActuales + delta;

      if (nuevaCantidad === 0) {
        comandoChat = comandoChat.replace(regex, "");
      } else if (nuevaCantidad > 0) {
        const prefijo = match.index === 0 ? "" : "+ ";
        comandoChat = comandoChat.replace(regex, `${prefijo}${nuevaCantidad}d${caras}${sufijos}`);
      } else {
        const prefijo = "- ";
        comandoChat = comandoChat.replace(regex, `${prefijo}${Math.abs(nuevaCantidad)}d${caras}${sufijos}`);
      }

      comandoChat = comandoChat
        .replace(/^\s*\+\s*/, "")
        .replace(/\s*[+-]\s*$/, "")
        .replace(/\s*\+\s*\+/g, " + ")
        .replace(/\s*\+\s*-\s*/g, " - ")
        .trim();
    } else {
      if (esSuma) {
        const texto = `${matchFrag[1] || 1}d${caras}${sufijos}`;
        comandoChat = comandoChat.trim() ? `${comandoChat.trim()} + ${texto}` : texto;
      } else {
        const texto = `- ${matchFrag[1] || 1}d${caras}${sufijos}`;
        comandoChat = comandoChat.trim() ? `${comandoChat.trim()} ${texto}` : texto;
      }
    }

    panelDerechoRef?.enfocarChat();
  }

  function sumarAlChat(fragmento) {
    actualizarDadoEnChat(fragmento, true);
  }

  function restarDelChat(fragmento) {
    actualizarDadoEnChat(fragmento, false);
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

  // En src/routes/+page.svelte
  async function procesarTirada(cmd) {
    if (!cmd || !cmd.trim()) return;

    // Solo invocas tirar_dado. Rust se encarga de guardar tanto en la partida como en el historial personal.
    try {
      const res = await invoke("tirar_dado", { comando: cmd.trim() });
      const [desglose, total] = res.split(" = ");

      const ahora = new Date();
      const horaStr = ahora.toTimeString().slice(0, 5);

      historialTiradas.unshift({
        id: Date.now(),
        hora: horaStr,
        comando: desglose,
        total: total
      });

      if (historialTiradas.length > 20) {
        historialTiradas.pop();
      }

      comandoChat = "";
    } catch (err) {
      historialTiradas.unshift({
        id: Date.now(),
        hora: horaStr,
        comando: "tirada no válida",
        total: ""
      });
    }
  }

  function modificarVida(inst, delta) {
    // $state es profundo: modificar el campo ya actualiza la pantalla, sin copiar el array
    inst.vida_actual += delta;
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
    onSeleccionarInstancia={(i) => (instanciaSeleccionada = i)}
    onModificarVida={modificarVida}
    onEnviarTirada={procesarTirada}
  />

  {#if fichaParaEditar}
    <ModalFicha
      bind:entidad={fichaParaEditar}
      {zonas}
      onCerrar={() => (fichaParaEditar = null)}
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
