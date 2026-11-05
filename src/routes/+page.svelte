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
  let mapaActual = $state({
    id: 1,
    id_zona: 1,
    url_imagen_fondo: "",
    ancho: 1000,
    alto: 1000,
    rejilla: 50,
    rejilla_visible: true,
    color_opacidad: [255, 255, 255, 76],
    grosor: 1,
    orden: 0
  });

  let instanciaSeleccionada = $state(null);
  let fichaParaEditar = $state(null);

  let comandoChat = $state("");
  let mensajeError = $state("");
  let panelDerechoRef;

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
      }
      await recargarMapas(zonaSeleccionadaId);
      bancoEntidades = await invoke("obtener_banco_entidades");
      await recargarInstancias();
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
      // Si el mapa actual ya no existe o cambió de zona, cargamos el primero
      if (!mapas.some((m) => m.id === mapaActual?.id)) {
        aplicarMapa(mapas[0]);
      }
    } else {
      mapaActual = null;
    }
    return mapas;
  }

  async function recargarBancoEntidades() {
    bancoEntidades = await invoke("obtener_banco_entidades");
  }

  async function recargarInstancias() {
    instanciasActivas = await invoke("obtener_instancias_activas");
    if (instanciaSeleccionada) {
      instanciaSeleccionada =
        instanciasActivas.find((i) => i.id_instancia === instanciaSeleccionada.id_instancia) ?? null;
    }
  }

  function mostrarError(err) {
    mensajeError = err instanceof Error ? err.message : String(err);
  }

  async function crearEntidad(datos) {
    await invoke("crear_entidad", {
      nombre: datos.nombre,
      tipo: datos.tipo,
      vidaMax: Number(datos.vidaMax),
      armadura: Number(datos.armadura),
      velocidad: Number(datos.velocidad),
      rangoArma: Number(datos.rangoArma),
      zonaId: zonaSeleccionadaId
    });
    await recargarBancoEntidades();
  }

  function editarEntidad(entidad) {
    fichaParaEditar = { entidad };
  }

  async function guardarEntidad(entidad, fotoData, quitarFoto) {
    await invoke("actualizar_entidad", {
      entidad,
      fotoData,
      quitarFoto
    });
    await Promise.all([recargarBancoEntidades(), recargarInstancias()]);
  }

  async function colocarEntidad(entidadId, x, y) {
    if (!mapaActual?.id) {
      mostrarError("Crea o selecciona un mapa antes de colocar fichas.");
      return;
    }
    try {
      const instancia = await invoke("colocar_entidad", {
        entidadId,
        mapaId: mapaActual.id,
        x,
        y
      });
      await recargarInstancias();
      instanciaSeleccionada = instancia;
    } catch (err) {
      mostrarError(err);
    }
  }

  async function moverInstancia(instanciaId, x, y) {
    try {
      await invoke("mover_instancia", {
        instanciaId,
        mapaId: mapaActual.id,
        x,
        y
      });
      await recargarInstancias();
    } catch (err) {
      mostrarError(err);
      await recargarInstancias();
    }
  }

  async function retirarInstancia(instancia) {
    try {
      await invoke("retirar_instancia", { instanciaId: instancia.id_instancia });
      instanciaSeleccionada = null;
      await recargarInstancias();
    } catch (err) {
      mostrarError(err);
    }
  }

  async function modificarVida(inst, delta) {
    try {
      await invoke("modificar_vida", { instanciaId: inst.id_instancia, delta });
      await recargarInstancias();
    } catch (err) {
      mostrarError(err);
    }
  }

  let barraRejillaRef;

  async function cambiarZona(id) {
    // Si había cambios pendientes en la rejilla, se guardan en el acto
    const guardado = await barraRejillaRef?.forzarGuardadoInmediato();
    if (guardado === false) return;
    zonaSeleccionadaId = id;
    await recargarMapas(id);
  }

  function aplicarMapa(m) {
    mapaActual = m;
    rejilla.visible = m.rejilla_visible !== false;
    rejilla.tamano = m.rejilla;
    rejilla.grosor = m.grosor;
    if (m.color_opacidad && m.color_opacidad.length >= 4) {
      rejilla.opacidad = m.color_opacidad[3] / 255;
      rejilla.color =
        "#" +
        m.color_opacidad
          .slice(0, 3)
          .map((v) => v.toString(16).padStart(2, "0"))
          .join("");
    }
  }

  async function seleccionarMapa(mapa) {
    const guardado = await barraRejillaRef?.forzarGuardadoInmediato();
    if (guardado === false) return;
    aplicarMapa(mapa);
  }

  /** @param {{ id: number, id_zona: number, nombre: string, url_imagen_fondo: string | null, ancho: number, alto: number, rejilla: number, rejilla_visible: boolean, color_opacidad: number[], grosor: number, orden: number }} mapa */
  function actualizarMapa(mapa) {
    mapas = mapas.map((actual) => actual.id === mapa.id ? mapa : actual);
    if (mapaActual?.id === mapa.id) aplicarMapa(mapa);
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
      await invoke("tirar_dado", { comando: cmd.trim() });
      historialTiradas = await invoke("obtener_historial_tiradas");
      comandoChat = "";
    } catch (err) {
      mostrarError(err);
    }
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
    onSeleccionarMapa={seleccionarMapa}
    onRecargarZonas={recargarZonas}
    onRecargarMapas={recargarMapas}
    onCrearEntidad={crearEntidad}
    onEditarEntidad={editarEntidad}
    onError={mostrarError}
  />

  <main class="area-central">
    <div class="barra-superior">
      <BarraRejilla
        bind:this={barraRejillaRef}
        bind:rejilla
        {mapaActual}
        onMapaGuardado={actualizarMapa}
        onError={mostrarError}
      />
    </div>

    <div class="contenedor-lienzo">
      <Tablero
        mapa={mapaActual}
        {rejilla}
        instancias={instanciasActivas}
        {instanciaSeleccionada}
        onSeleccionarInstancia={(i) => (instanciaSeleccionada = i)}
        onColocarEntidad={colocarEntidad}
        onMoverInstancia={moverInstancia}
        onEditarEntidad={editarEntidad}
        onRetirarInstancia={retirarInstancia}
      />
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
    onRetirarInstancia={retirarInstancia}
    onEnviarTirada={procesarTirada}
  />

  {#if mensajeError}
    <div class="mensaje-error" role="alert">
      <span>{mensajeError}</span>
      <button type="button" aria-label="Cerrar aviso" onclick={() => (mensajeError = "")}>×</button>
    </div>
  {/if}

  {#if fichaParaEditar}
    <ModalFicha
      entidad={fichaParaEditar.entidad}
      {zonas}
      onGuardar={guardarEntidad}
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
  .mensaje-error {
    position: fixed; top: 12px; left: 50%; transform: translateX(-50%); z-index: 1000;
    display: flex; align-items: center; gap: 12px; max-width: min(80vw, 600px);
    background: #7f1d1d; border: 1px solid #ef4444; border-radius: 6px; padding: 8px 12px;
    color: #fee2e2; font-size: 13px; box-shadow: 0 4px 16px #0008;
  }
  .mensaje-error button { background: transparent; border: 0; color: inherit; font-size: 18px; cursor: pointer; }

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