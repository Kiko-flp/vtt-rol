<!-- src/routes/+page.svelte -->
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
  let mapaActual = $state(null);

  let instanciaSeleccionada = $state(null);
  let fichaParaEditar = $state(null);

  let comandoChat = $state("");
  let panelDerechoRef;
  let barraRejillaRef;

  let rejilla = $state({
    visible: true,
    tamano: 50,
    color: "#ffffff",
    grosor: 1,
    opacidad: 0.3
  });

  onMount(async () => {
    try {
      await recargarZonas();
      if (zonas.length > 0) {
        zonaSeleccionadaId = zonas[0].id;
        await recargarMapas(zonaSeleccionadaId);
      }
      bancoEntidades = await invoke("obtener_banco_entidades");
      instanciasActivas = await invoke("obtener_instancias_activas");
      historialTiradas = await invoke("obtener_historial_tiradas");
    } catch (err) {
      console.error("Error al obtener datos de Rust:", err);
    }
  });

  async function recargarZonas() {
    zonas = await invoke("obtener_zonas");
  }

  async function recargarMapas(idZona) {
    mapas = await invoke("obtener_mapas_zona", { zonaId: idZona });
    if (mapas.length > 0) {
      const existente = mapas.find((m) => m.id === mapaActual?.id);
      aplicarMapa(existente || mapas[0]);
    } else {
      mapaActual = null;
    }
  }

  async function cambiarZona(id) {
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
    await barraRejillaRef?.forzarGuardadoInmediato();
    zonaSeleccionadaId = id;
    await recargarMapas(id);
  }

  // GUARDA EL MAPA ANTERIOR ANTES DE CAMBIAR AL NUEVO
  async function cambiarMapa(nuevoMapa) {
    if (!nuevoMapa || mapaActual?.id === nuevoMapa?.id) return;
    
    // 1. Quitar el foco activo si el usuario estaba escribiendo
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur();
    }
    // 2. Guardar en Rust los cambios del mapa saliente
    await barraRejillaRef?.forzarGuardadoInmediato();
    // 3. Aplicar los valores del nuevo mapa
    aplicarMapa(nuevoMapa);
  }

  function aplicarMapa(m) {
    if (!m) {
      mapaActual = null;
      return;
    }
    mapaActual = m;

    rejilla.tamano = m.rejilla;
    rejilla.grosor = m.grosor;

    if (Array.isArray(m.color_opacidad) && m.color_opacidad.length >= 4) {
      const [r, g, b, a] = m.color_opacidad;
      rejilla.color = `#${[r, g, b].map((v) => Number(v).toString(16).padStart(2, "0")).join("")}`;
      rejilla.opacidad = Number(a) / 255;
    }
  }

  function onMapaActualizadoPorRejilla(mapaModificado) {
    mapaActual = mapaModificado;
    const idx = mapas.findIndex((m) => m.id === mapaModificado.id);
    if (idx !== -1) {
      mapas[idx] = mapaModificado;
    }
  }

  function actualizarDadoEnChat(fragmento, esSuma) {
    const matchFrag = fragmento.match(/^(\d*)d(\d+)([!a-z]*)$/i);
    if (!matchFrag) return;

    const delta = (matchFrag[1] ? parseInt(matchFrag[1]) : 1) * (esSuma ? 1 : -1);
    const caras = matchFrag[2];
    const sufijos = matchFrag[3] || "";

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

  async function procesarTirada(cmd) {
    if (!cmd || !cmd.trim()) return;
    try {
      const res = await invoke("tirar_dado", { comando: cmd.trim() });
      const [desglose, total] = res.split(" = ");
      const ahora = new Date();
      const horaStr = ahora.toTimeString().slice(0, 5);

      historialTiradas.unshift({
        hora: horaStr,
        comando: desglose,
        total: total
      });

      if (historialTiradas.length > 20) {
        historialTiradas.pop();
      }
      comandoChat = "";
    } catch {
      historialTiradas.unshift({
        hora: new Date().toTimeString().slice(0, 5),
        comando: "tirada no válida",
        total: ""
      });
    }
  }

  function modificarVida(inst, delta) {
    inst.vida_actual += delta;
  }
</script>

<div class="contenedor-principal">
  <PanelIzquierdo
    {zonas}
    {mapas}
    banco={bancoEntidades}
    {zonaSeleccionadaId}
    {mapaActual}
    onSeleccionarZona={cambiarZona}
    onSeleccionarMapa={cambiarMapa}
    onRecargarZonas={recargarZonas}
    onRecargarMapas={recargarMapas}
  />

  <main class="area-central">
    <div class="barra-superior">
      <BarraRejilla
        bind:this={barraRejillaRef}
        bind:rejilla
        {mapaActual}
        onActualizarMapa={onMapaActualizadoPorRejilla}
      />
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

  /* Scrollbar ultrafina global */
  :global(*) {
    scrollbar-width: thin;
    scrollbar-color: #3f3f46 transparent;
  }
  :global(*::-webkit-scrollbar) {
    width: 4px;
    height: 4px;
  }
  :global(*::-webkit-scrollbar-track) {
    background: transparent;
  }
  :global(*::-webkit-scrollbar-thumb) {
    background: #3f3f46;
    border-radius: 4px;
  }
  :global(*::-webkit-scrollbar-thumb:hover) {
    background: #71717a;
  }
</style>