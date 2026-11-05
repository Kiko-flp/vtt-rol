<!-- src/routes/componentes/PanelIzquierdo.svelte -->
<script>
  import { onMount, onDestroy } from "svelte";
  import { flip } from "svelte/animate";
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { getCurrentWebview } from "@tauri-apps/api/webview";

  let {
    zonas = [],
    mapas = [],
    banco = [],
    zonaSeleccionadaId = 1,
    mapaActual = null,
    onSeleccionarZona,
    onSeleccionarMapa,
    onRecargarZonas,
    onRecargarMapas,
    onCrearEntidad,
    onEditarEntidad,
    onError
  } = $props();

  let busqueda = $state("");
  let nuevaEntidad = $state({
    nombre: "",
    tipo: "Enemigo",
    vidaMax: 10,
    armadura: 0,
    velocidad: 6,
    rangoArma: 1
  });
  let errorEntidad = $state("");

  // Edición en línea
  let editandoZonaId = $state(null);
  let textoEdicionZona = $state("");
  let editandoMapaId = $state(null);
  let textoEdicionMapa = $state("");

  // Menús contextuales
  let menuZona = $state(null);
  let menuMapa = $state(null);
  let menuEntidad = $state(null);

  // Selector manual de imagen
  let inputArchivoRef = $state(null);
  let mapaParaCambiarImagen = $state(null);

  // Miniaturas ultra-bajas reactivas
  let miniaturasPorMapa = $state({});

  // Listas locales para previsualizar el movimiento en vivo durante el arrastre
  let zonasVisuales = $state([]);
  let mapasVisuales = $state([]);

  // Sincronizar listas visuales cuando cambien las props (si no estamos arrastrando)
  $effect(() => {
    if (!arrastre.activo) {
      zonasVisuales = [...zonas].sort((a, b) => (a.orden ?? 0) - (b.orden ?? 0));
    }
  });

  $effect(() => {
    if (!arrastre.activo) {
      mapasVisuales = [...mapas].sort((a, b) => (a.orden ?? 0) - (b.orden ?? 0));
    }
  });

  /**
   * @typedef {{ id: number, nombre: string, foto_url?: string | null, url_imagen_fondo?: string | null }} ElementoArrastrable
   */
  /** @type {{ activo: boolean, tipo: "zona" | "mapa" | "entidad" | null, item: ElementoArrastrable | null, x: number, y: number, ancho: number, alto: number, offsetRelX: number, offsetRelY: number, indiceOrigen: number, indiceDestino: number }} */
  let arrastre = $state({
    activo: false,
    tipo: null, // 'zona' | 'mapa'
    item: null,
    x: 0,
    y: 0,
    ancho: 0,
    alto: 0,
    offsetRelX: 0,
    offsetRelY: 0,
    indiceOrigen: -1,
    indiceDestino: -1
  });

  let unlistenDrop = null;

  let entidadesJugadores = $derived(
    banco.filter((entidad) =>
      entidad.tipo === "Jugador" &&
      entidad.nombre.toLowerCase().includes(busqueda.toLowerCase())
    )
  );
  let entidadesZona = $derived(
    banco.filter((entidad) =>
      entidad.tipo !== "Jugador" &&
      entidad.zona_id === zonaSeleccionadaId &&
      entidad.nombre.toLowerCase().includes(busqueda.toLowerCase())
    )
  );

  async function crearEntidad() {
    if (!nuevaEntidad.nombre.trim()) {
      errorEntidad = "Escribe un nombre para la ficha.";
      return;
    }

    try {
      await onCrearEntidad({
        ...nuevaEntidad,
        nombre: nuevaEntidad.nombre.trim()
      });
      nuevaEntidad.nombre = "";
      errorEntidad = "";
    } catch (err) {
      errorEntidad = String(err);
    }
  }

  /**
   * @param {PointerEvent} e
   * @param {ElementoArrastrable} entidad
   */
  function iniciarArrastreEntidad(e, entidad) {
    if (e.button !== 0) return;
    e.preventDefault();
    const rect = e.currentTarget.getBoundingClientRect();
    const inicioX = e.clientX;
    const inicioY = e.clientY;
    let inicioArrastre = false;

    function quitarEscuchas() {
      window.removeEventListener("pointermove", alMover);
      window.removeEventListener("pointerup", alSoltar);
      window.removeEventListener("pointercancel", alCancelar);
    }

    /** @param {PointerEvent} evento */
    function alMover(evento) {
      if (evento.pointerId !== e.pointerId) return;
      if (!inicioArrastre && Math.hypot(evento.clientX - inicioX, evento.clientY - inicioY) < 4) return;

      inicioArrastre = true;
      arrastre.activo = true;
      arrastre.tipo = "entidad";
      arrastre.item = entidad;
      arrastre.x = evento.clientX;
      arrastre.y = evento.clientY;
      arrastre.ancho = rect.width;
      arrastre.alto = rect.height;
      arrastre.offsetRelX = inicioX - rect.left;
      arrastre.offsetRelY = inicioY - rect.top;
    }

    /** @param {PointerEvent} evento */
    function alSoltar(evento) {
      if (evento.pointerId !== e.pointerId) return;
      quitarEscuchas();
      if (inicioArrastre) {
        const tablero = document.elementFromPoint(evento.clientX, evento.clientY)?.closest(".tablero");
        if (tablero) {
          window.dispatchEvent(new CustomEvent("vtt-colocar-entidad", {
            detail: {
              entidadId: entidad.id,
              clientX: evento.clientX,
              clientY: evento.clientY
            }
          }));
        }
      }
      arrastre.activo = false;
      arrastre.item = null;
    }

    /** @param {PointerEvent} evento */
    function alCancelar(evento) {
      if (evento.pointerId !== e.pointerId) return;
      quitarEscuchas();
      arrastre.activo = false;
      arrastre.item = null;
    }

    window.addEventListener("pointermove", alMover);
    window.addEventListener("pointerup", alSoltar);
    window.addEventListener("pointercancel", alCancelar);
  }

  async function generarMiniaturaUltraBaja(src) {
    if (!src) return "";
    return new Promise((resolve) => {
      const img = new Image();
      img.crossOrigin = "anonymous";
      img.onload = () => {
        const canvas = document.createElement("canvas");
        canvas.width = 64;
        canvas.height = 36;
        const ctx = canvas.getContext("2d");
        if (ctx) {
          ctx.drawImage(img, 0, 0, 64, 36);
          resolve(canvas.toDataURL("image/jpeg", 0.3));
        } else {
          resolve("");
        }
      };
      img.onerror = () => resolve("");
      img.src = src;
    });
  }

  $effect(() => {
    for (const m of mapas) {
      if (m.url_imagen_fondo && !miniaturasPorMapa[m.id]) {
        generarMiniaturaUltraBaja(m.url_imagen_fondo).then((mini) => {
          if (mini) miniaturasPorMapa[m.id] = mini;
        });
      }
    }
  });

  // ==========================================
  // ARRASTRE FLUIDO CON MOVIMIENTO EN VIVO
  // ==========================================
  function iniciarArrastrePuntero(tipo, item, indice, e) {
    if (e.button !== 0) return; // Solo botón principal
    if (editandoZonaId !== null || editandoMapaId !== null) return;

    const target = e.currentTarget;
    const rect = target.getBoundingClientRect();

    const startX = e.clientX;
    const startY = e.clientY;
    let haSuperadoUmbral = false;

    function onPointerMove(ev) {
      const dist = Math.hypot(ev.clientX - startX, ev.clientY - startY);
      if (!haSuperadoUmbral && dist > 4) {
        haSuperadoUmbral = true;
        arrastre.activo = true;
        arrastre.tipo = tipo;
        arrastre.item = item;
        arrastre.ancho = rect.width;
        arrastre.alto = rect.height;
        arrastre.offsetRelX = startX - rect.left;
        arrastre.offsetRelY = startY - rect.top;
        arrastre.indiceOrigen = indice;
        arrastre.indiceDestino = indice;
      }

      if (!arrastre.activo) return;

      arrastre.x = ev.clientX;
      arrastre.y = ev.clientY;

      // Detectar sobre qué tarjeta estamos pasando
      const el = document.elementFromPoint(ev.clientX, ev.clientY);
      const contenedor = el?.closest(tipo === "zona" ? ".item-zona" : ".tarjeta-mapa");

      if (contenedor && contenedor.dataset.indice !== undefined) {
        const nuevoIndice = Number(contenedor.dataset.indice);
        if (nuevoIndice !== arrastre.indiceDestino) {
          arrastre.indiceDestino = nuevoIndice;

          // Reordenar en vivo la lista visual
          if (tipo === "zona") {
            const copia = [...zonasVisuales];
            const [removido] = copia.splice(copia.findIndex((z) => z.id === item.id), 1);
            copia.splice(nuevoIndice, 0, removido);
            zonasVisuales = copia;
          } else {
            const copia = [...mapasVisuales];
            const [removido] = copia.splice(copia.findIndex((m) => m.id === item.id), 1);
            copia.splice(nuevoIndice, 0, removido);
            mapasVisuales = copia;
          }
        }
      }
    }

    async function onPointerUp() {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);

      if (arrastre.activo) {
        const tipoArrastre = arrastre.tipo;
        try {
          if (arrastre.tipo === "zona") {
            for (let i = 0; i < zonasVisuales.length; i++) {
              const zona = zonasVisuales[i];
              if (zona.orden !== i) {
                await invoke("reordenar_zona", { zonaId: zona.id, orden: i });
              }
            }
            await onRecargarZonas();
          } else {
            for (let i = 0; i < mapasVisuales.length; i++) {
              const mapa = mapasVisuales[i];
              if (mapa.orden !== i) {
                await invoke("reordenar_mapa", { mapaId: mapa.id, orden: i });
              }
            }
            await onRecargarMapas(zonaSeleccionadaId);
          }
        } catch (err) {
          onError?.(err);
          if (tipoArrastre === "zona") {
            await onRecargarZonas();
          } else {
            await onRecargarMapas(zonaSeleccionadaId);
          }
        }
      } else if (arrastre.tipo === "mapa") {
        // Si no se arrastró, fue un clic estándar de selección
        await onSeleccionarMapa(item);
      } else {
        onSeleccionarZona(item.id);
      }

      arrastre.activo = false;
      arrastre.item = null;
    }

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  // ==========================================
  // OPERACIONES BACKEND (Zonas y Mapas)
  // ==========================================
  async function anadirZona() {
    try {
      const nuevaZona = await invoke("crear_zona");
      await onRecargarZonas();
      onSeleccionarZona(nuevaZona.id);
    } catch (err) {
      onError?.(err);
    }
  }

  async function anadirMapa(urlImagen = null) {
    try {
      const nuevoMapa = await invoke("crear_mapa", {
        idZona: zonaSeleccionadaId,
        urlImagenFondo: urlImagen || null
      });

      if (nuevoMapa.url_imagen_fondo) {
        const miniatura = await generarMiniaturaUltraBaja(nuevoMapa.url_imagen_fondo);
        if (miniatura) miniaturasPorMapa[nuevoMapa.id] = miniatura;
      }

      const mapasActualizados = await onRecargarMapas(zonaSeleccionadaId);
      await onSeleccionarMapa(mapasActualizados?.find((mapa) => mapa.id === nuevoMapa.id) ?? nuevoMapa);
    } catch (err) {
      onError?.(err);
    }
  }


  let modalConfirmacion = $state(null); // { mapa, urlImagen }
  let posModal = $state({ x: 240, y: 90 });

  function iniciarArrastreModal(e) {
    if (e.button !== 0) return;
    const startX = e.clientX;
    const startY = e.clientY;
    const initX = posModal.x;
    const initY = posModal.y;

    function onPointerMove(ev) {
      posModal.x = Math.max(10, Math.min(window.innerWidth - 260, initX + (ev.clientX - startX)));
      posModal.y = Math.max(10, Math.min(window.innerHeight - 150, initY + (ev.clientY - startY)));
    }

    function onPointerUp() {
      window.removeEventListener("pointermove", onPointerMove);
      window.removeEventListener("pointerup", onPointerUp);
    }

    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  async function asignarImagenAMapa(mapa, urlImagen) {
    // Si ya tiene imagen, se abre (o se reemplaza si ya había uno abierto) la ventanita flotante
    if (mapa.url_imagen_fondo && mapa.url_imagen_fondo.trim() !== "") {
      modalConfirmacion = { mapa, urlImagen };
      return;
    }
    await aplicarNuevaImagen(mapa, urlImagen);
  }

  async function confirmarReemplazoImagen() {
    if (!modalConfirmacion) return;
    const { mapa, urlImagen } = modalConfirmacion;
    modalConfirmacion = null;
    await aplicarNuevaImagen(mapa, urlImagen);
  }

  async function aplicarNuevaImagen(mapa, urlImagen) {
    try {
      await invoke("actualizar_imagen_mapa", {
        mapaId: mapa.id,
        urlImagenFondo: urlImagen
      });
      const miniatura = await generarMiniaturaUltraBaja(urlImagen);
      if (miniatura) miniaturasPorMapa[mapa.id] = miniatura;

      const mapasActualizados = await onRecargarMapas(zonaSeleccionadaId);
      const mapaActualizado = mapasActualizados?.find((item) => item.id === mapa.id);
      if (mapaActual?.id === mapa.id && mapaActualizado) {
        await onSeleccionarMapa(mapaActualizado);
      }
    } catch (err) {
      onError?.(err);
    }
  }

  // Drop de imágenes desde el SO
  onMount(async () => {
    try {
      const webview = getCurrentWebview();
      unlistenDrop = await webview.onDragDropEvent(async (event) => {
        if (event.payload.type === "drop") {
          const paths = event.payload.paths;
          if (!paths || paths.length === 0) return;
          const ruta = paths[0];
          if (!ruta.match(/\.(png|jpe?g|webp|gif|svg|bmp|avif)$/i)) return;

          const urlFinal = convertFileSrc(ruta);
          const { x, y } = event.payload.position;
          const escala = window.devicePixelRatio || 1;
          const el = document.elementFromPoint(x / escala, y / escala);

          const tarjeta = el?.closest(".tarjeta-mapa");
          if (tarjeta && tarjeta.dataset.mapaId) {
            const id = Number(tarjeta.dataset.mapaId);
            const mapaDestino = mapas.find((m) => m.id === id);
            if (mapaDestino) {
              await asignarImagenAMapa(mapaDestino, urlFinal);
              return;
            }
          }

          if (el?.closest(".sub-mapas")) {
            await anadirMapa(urlFinal);
          }
        }
      });
    } catch (err) {
      console.warn("Tauri dragDropEvent no disponible:", err);
    }
  });

  onDestroy(() => {
    if (unlistenDrop) unlistenDrop();
  });

  function iniciarEdicionZona(z) {
    cerrarMenus();
    editandoZonaId = z.id;
    textoEdicionZona = z.nombre;
  }

  async function guardarEdicionZona(z) {
    if (editandoZonaId === null) return;
    editandoZonaId = null;
    const nuevoNombre = textoEdicionZona.trim() || z.nombre;
    if (nuevoNombre !== z.nombre) {
      try {
        await invoke("renombrar_zona", { zonaId: z.id, nombre: nuevoNombre });
        await onRecargarZonas();
      } catch (err) {
        onError?.(err);
      }
    }
  }

  async function borrarZona(z) {
    cerrarMenus();
    try {
      await invoke("eliminar_zona", { zona: z });
      await onRecargarZonas();
    } catch (err) {
      onError?.(err);
    }
  }

  function iniciarEdicionMapa(m) {
    cerrarMenus();
    editandoMapaId = m.id;
    textoEdicionMapa = m.nombre;
  }

  async function guardarEdicionMapa(m) {
    if (editandoMapaId === null) return;
    editandoMapaId = null;
    const nuevoNombre = textoEdicionMapa.trim() || m.nombre;
    if (nuevoNombre !== m.nombre) {
      try {
        await invoke("renombrar_mapa", { mapaId: m.id, nombre: nuevoNombre });
        await onRecargarMapas(zonaSeleccionadaId);
      } catch (err) {
        onError?.(err);
      }
    }
  }

  async function borrarMapa(m) {
    cerrarMenus();
    try {
      await invoke("borrar_mapa", { mapa: m });
      delete miniaturasPorMapa[m.id];
      await onRecargarMapas(zonaSeleccionadaId);
    } catch (err) {
      onError?.(err);
    }
  }

  function abrirSelectorManual(mapa) {
    mapaParaCambiarImagen = mapa;
    cerrarMenus();
    inputArchivoRef?.click();
  }

  function onArchivoSeleccionadoManualmente(e) {
    const files = e.target.files;
    if (!files || files.length === 0 || !mapaParaCambiarImagen) return;
    const archivo = files[0];
    const reader = new FileReader();
    reader.onload = async () => {
      try {
        await asignarImagenAMapa(mapaParaCambiarImagen, reader.result);
        mapaParaCambiarImagen = null;
        e.target.value = "";
      } catch (err) {
        onError?.(err);
      }
    };
    reader.onerror = () => onError?.(new Error("No se pudo leer la imagen del mapa."));
    reader.readAsDataURL(archivo);
  }

  function abrirMenuZona(e, z) {
    e.preventDefault();
    cerrarMenus();
    menuZona = { x: e.clientX, y: e.clientY, zona: z };
  }

  function abrirMenuMapa(e, m) {
    e.preventDefault();
    cerrarMenus();
    menuMapa = { x: e.clientX, y: e.clientY, mapa: m };
  }

  function abrirMenuEntidad(e, entidad) {
    e.preventDefault();
    e.stopPropagation();
    cerrarMenus();
    menuEntidad = { x: e.clientX, y: e.clientY, entidad };
  }

  function editarEntidadDesdeMenu() {
    const entidad = menuEntidad?.entidad;
    cerrarMenus();
    if (entidad) onEditarEntidad?.(entidad);
  }

  function cerrarMenus() {
    menuZona = null;
    menuMapa = null;
    menuEntidad = null;
  }
</script>

<svelte:window onclick={cerrarMenus} />

<input
  type="file"
  accept="image/*"
  bind:this={inputArchivoRef}
  style="display: none;"
  onchange={onArchivoSeleccionadoManualmente}
/>

<aside class="panel-izquierdo">
  <div class="bloque-arriba">
    <!-- SUBSECCIÓN ZONAS -->
    <div class="subseccion sub-zonas">
      <span class="titulo-mini">ZONAS</span>
      <div class="scroll-zonas">
        {#each zonasVisuales as z, i (z.id)}
          <div animate:flip={{ duration: 150 }}>
            <button
              class="item-zona"
              class:activo={zonaSeleccionadaId === z.id}
              class:fantasma-origen={arrastre.activo && arrastre.tipo === 'zona' && arrastre.item?.id === z.id}
              data-indice={i}
              onpointerdown={(e) => iniciarArrastrePuntero("zona", z, i, e)}
              ondblclick={(e) => {
                e.stopPropagation();
                iniciarEdicionZona(z);
              }}
              oncontextmenu={(e) => abrirMenuZona(e, z)}
              title={z.nombre}
            >
              {#if editandoZonaId === z.id}
                <input
                  type="text"
                  class="input-nombre"
                  bind:value={textoEdicionZona}
                  onblur={() => guardarEdicionZona(z)}
                  onkeydown={(e) => {
                    if (e.key === "Enter") guardarEdicionZona(z);
                    if (e.key === "Escape") editandoZonaId = null;
                  }}
                  onclick={(e) => e.stopPropagation()}
                  autofocus
                />
              {:else}
                <span class="texto-recortado">{z.nombre}</span>
              {/if}
            </button>
          </div>
        {/each}
        <button class="item-zona nueva" onclick={anadirZona} title="Añadir zona">+</button>
      </div>
    </div>

    <!-- SUBSECCIÓN MAPAS -->
    <div class="subseccion sub-mapas">
      <span class="titulo-mini">MAPAS</span>
      <div class="scroll-mapas">
        {#each mapasVisuales as m, i (m.id)}
          <div animate:flip={{ duration: 150 }}>
            <button
              type="button"
              class="tarjeta-mapa"
              class:seleccionado={mapaActual?.id === m.id}
              class:fantasma-origen={arrastre.activo && arrastre.tipo === 'mapa' && arrastre.item?.id === m.id}
              data-mapa-id={m.id}
              data-indice={i}
              onpointerdown={(e) => iniciarArrastrePuntero("mapa", m, i, e)}
              ondblclick={(e) => {
                e.stopPropagation();
                iniciarEdicionMapa(m);
              }}
              oncontextmenu={(e) => abrirMenuMapa(e, m)}
            >
              <!-- Contenedor de imagen blindado para no interceptar clics/arrastres -->
              {#if m.url_imagen_fondo && miniaturasPorMapa[m.id]}
                <div class="contenedor-low">
                  <img src={miniaturasPorMapa[m.id]} alt="" class="img-low" draggable="false" />
                </div>
              {:else}
                <span class="previa-mapa">🗺️</span>
              {/if}

              {#if editandoMapaId === m.id}
                <input
                  type="text"
                  class="input-nombre"
                  bind:value={textoEdicionMapa}
                  onblur={() => guardarEdicionMapa(m)}
                  onkeydown={(e) => {
                    if (e.key === "Enter") guardarEdicionMapa(m);
                    if (e.key === "Escape") editandoMapaId = null;
                  }}
                  onclick={(e) => e.stopPropagation()}
                  autofocus
                />
              {:else}
                <span class="nombre-mapa texto-recortado">{m.nombre}</span>
              {/if}
            </button>
          </div>
        {/each}
        <button class="tarjeta-mapa nueva" onclick={() => anadirMapa(null)} title="Añadir mapa">+</button>
      </div>
    </div>
  </div>

  <div class="bloque-abajo">
    <div class="sub-busqueda">
      <input type="text" placeholder="Buscar..." bind:value={busqueda} class="input-buscar" />
    </div>

    <details class="crear-entidad">
      <summary>Crear ficha</summary>
      <form onsubmit={(e) => { e.preventDefault(); crearEntidad(); }}>
        <input bind:value={nuevaEntidad.nombre} maxlength="80" placeholder="Nombre" aria-label="Nombre de la ficha" />
        <select bind:value={nuevaEntidad.tipo} aria-label="Tipo de ficha">
          <option value="Enemigo">Enemigo</option>
          <option value="NPC">NPC</option>
          <option value="Jugador">Jugador</option>
        </select>
        <div class="campos-entidad">
          <label>Vida <input type="number" min="1" max="32767" bind:value={nuevaEntidad.vidaMax} /></label>
          <label>Armadura <input type="number" min="0" max="32767" bind:value={nuevaEntidad.armadura} /></label>
          <label>Velocidad <input type="number" min="0" max="32767" bind:value={nuevaEntidad.velocidad} /></label>
          <label>Rango <input type="number" min="0" max="32767" bind:value={nuevaEntidad.rangoArma} /></label>
        </div>
        {#if errorEntidad}<span class="error-entidad" role="alert">{errorEntidad}</span>{/if}
        <button type="submit">Guardar ficha</button>
      </form>
    </details>

    <div class="sub-entidades">
      <div class="grupo-entidades">
        <span class="titulo-grupo">JUGADORES · TODAS LAS ZONAS</span>
        {#each entidadesJugadores as item (item.id)}
          <button
            type="button"
            class="fila-entidad"
            title="Arrastra al mapa para colocar una ficha"
            onpointerdown={(e) => iniciarArrastreEntidad(e, item)}
            oncontextmenu={(e) => abrirMenuEntidad(e, item)}
            ondblclick={(e) => {
              e.stopPropagation();
              onEditarEntidad?.(item);
            }}
          >
            <div class="avatar-entidad">
              {#if item.foto_url}
                <img src={item.foto_url.startsWith("data:") || item.foto_url.startsWith("http") ? item.foto_url : convertFileSrc(item.foto_url)} alt="" />
              {:else}
                <span>{item.nombre.slice(0, 1).toUpperCase()}</span>
              {/if}
            </div>
            <span class="texto-recortado">{item.nombre}</span>
          </button>
        {/each}
      </div>
      <div class="grupo-entidades">
        <span class="titulo-grupo">NPC · {zonas.find((zona) => zona.id === zonaSeleccionadaId)?.nombre ?? "ZONA"}</span>
        {#each entidadesZona.filter((entidad) => entidad.tipo === "NPC") as item (item.id)}
          <button
            type="button"
            class="fila-entidad"
            title="Arrastra al mapa para colocar una ficha"
            onpointerdown={(e) => iniciarArrastreEntidad(e, item)}
            oncontextmenu={(e) => abrirMenuEntidad(e, item)}
            ondblclick={(e) => {
              e.stopPropagation();
              onEditarEntidad?.(item);
            }}
          >
            <div class="avatar-entidad">
              {#if item.foto_url}
                <img src={item.foto_url.startsWith("data:") || item.foto_url.startsWith("http") ? item.foto_url : convertFileSrc(item.foto_url)} alt="" />
              {:else}
                <span>{item.nombre.slice(0, 1).toUpperCase()}</span>
              {/if}
            </div>
            <span class="texto-recortado">{item.nombre}</span>
          </button>
        {/each}
      </div>
      <div class="grupo-entidades">
        <span class="titulo-grupo">ENEMIGOS · {zonas.find((zona) => zona.id === zonaSeleccionadaId)?.nombre ?? "ZONA"}</span>
        {#each entidadesZona.filter((entidad) => entidad.tipo === "Enemigo") as item (item.id)}
          <button
            type="button"
            class="fila-entidad"
            title="Arrastra al mapa para colocar una ficha"
            onpointerdown={(e) => iniciarArrastreEntidad(e, item)}
            oncontextmenu={(e) => abrirMenuEntidad(e, item)}
            ondblclick={(e) => {
              e.stopPropagation();
              onEditarEntidad?.(item);
            }}
          >
            <div class="avatar-entidad">
              {#if item.foto_url}
                <img src={item.foto_url.startsWith("data:") || item.foto_url.startsWith("http") ? item.foto_url : convertFileSrc(item.foto_url)} alt="" />
              {:else}
                <span>{item.nombre.slice(0, 1).toUpperCase()}</span>
              {/if}
            </div>
            <span class="texto-recortado">{item.nombre}</span>
          </button>
        {/each}
      </div>
    </div>
  </div>
</aside>

<!-- ==========================================
     TARJETA CLON FLOTANTE (Sigue al cursor con realismo)
     ========================================== -->
{#if arrastre.activo && arrastre.item}
  <div
    class="elemento-flotante"
    style="
      left: {arrastre.x - arrastre.offsetRelX}px;
      top: {arrastre.y - arrastre.offsetRelY}px;
      width: {arrastre.ancho}px;
      height: {arrastre.alto}px;
    "
  >
    {#if arrastre.tipo === 'zona'}
      <div class="item-zona clon activo">
        <span class="texto-recortado">{arrastre.item.nombre}</span>
      </div>
    {:else if arrastre.tipo === 'mapa'}
      <div class="tarjeta-mapa clon seleccionado">
        {#if arrastre.item.url_imagen_fondo && miniaturasPorMapa[arrastre.item.id]}
          <div class="contenedor-low">
            <img src={miniaturasPorMapa[arrastre.item.id]} alt="" class="img-low" />
          </div>
        {:else}
          <span class="previa-mapa">🗺️</span>
        {/if}
        <span class="nombre-mapa texto-recortado">{arrastre.item.nombre}</span>
      </div>
    {:else}
      <div class="fila-entidad ficha-flotante">
        <div class="avatar-entidad">
          {#if arrastre.item.foto_url}
            <img src={arrastre.item.foto_url.startsWith("data:") || arrastre.item.foto_url.startsWith("http") ? arrastre.item.foto_url : convertFileSrc(arrastre.item.foto_url)} alt="" />
          {:else}
            <span>{arrastre.item.nombre.slice(0, 1).toUpperCase()}</span>
          {/if}
        </div>
        <span class="texto-recortado">{arrastre.item.nombre}</span>
      </div>
    {/if}
  </div>
{/if}

<!-- Menús contextuales -->
{#if menuZona}
  <div class="menu-contextual" style="top: {menuZona.y}px; left: {menuZona.x}px;">
    <button onclick={() => iniciarEdicionZona(menuZona.zona)}>Cambiar nombre</button>
    <button class="peligro" onclick={() => borrarZona(menuZona.zona)}>Borrar zona</button>
  </div>
{/if}

{#if menuMapa}
  <div class="menu-contextual" style="top: {menuMapa.y}px; left: {menuMapa.x}px;">
    <button onclick={() => abrirSelectorManual(menuMapa.mapa)}>Cambiar imagen</button>
    <button onclick={() => iniciarEdicionMapa(menuMapa.mapa)}>Cambiar nombre</button>
    <button class="peligro" onclick={() => borrarMapa(menuMapa.mapa)}>Borrar mapa</button>
  </div>
{/if}

{#if menuEntidad}
  <div class="menu-contextual" style="top: {menuEntidad.y}px; left: {menuEntidad.x}px;" onclick={(e) => e.stopPropagation()}>
    <button onclick={editarEntidadDesdeMenu}>Editar datos</button>
  </div>
{/if}

<!-- VENTANA FLOTANTE INDEPENDIENTE -->
{#if modalConfirmacion}
  <div
    class="ventana-flotante"
    style="left: {posModal.x}px; top: {posModal.y}px;"
  >
    <div class="cabecera-flotante" onpointerdown={iniciarArrastreModal}>
      <span class="icono-arrastre">⋮⋮</span>
      <span class="titulo-flotante">Reemplazar imagen</span>
    </div>

    <div class="cuerpo-flotante">
      <p class="texto-modal">
        El mapa "<strong>{modalConfirmacion.mapa.nombre}</strong>" ya tiene una imagen.
        <span>¿Deseas reemplazarla?</span>
      </p>

      <div class="acciones-modal">
        <button type="button" class="btn-cancelar" onclick={() => (modalConfirmacion = null)}>
          Cancelar
        </button>
        <button type="button" class="btn-confirmar" onclick={confirmarReemplazoImagen}>
          Reemplazar
        </button>
      </div>
    </div>
  </div>
{/if}

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
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }
  .bloque-arriba { height: 44%; display: flex; gap: 4px; border-bottom: 1px solid #27272a; padding-bottom: 4px; }
  .subseccion { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
  .scroll-zonas, .scroll-mapas { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }

  .item-zona {
    width: 100%; position: relative; background: #27272a; border: 1px solid #3f3f46;
    color: white; font-weight: 500; padding: 6px 4px; border-radius: 4px; cursor: grab;
    text-align: center; overflow: hidden; touch-action: none;
  }
  .item-zona:active { cursor: grabbing; }
  .item-zona.activo { background: #3b82f6; border-color: #60a5fa; }
  .item-zona.nueva { border-style: dashed; font-size: 14px; padding: 3px; cursor: pointer; }

  /* Efecto hueco punteado mientras se arrastra */
  .fantasma-origen {
    opacity: 0.25 !important;
    border: 1px dashed #60a5fa !important;
    transform: scale(0.96);
  }

  .texto-recortado { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .input-nombre {
    width: 90%; background: #09090b; border: 1px solid #60a5fa; color: white;
    font-size: 10px; padding: 2px; border-radius: 2px; text-align: center; outline: none;
  }

  .tarjeta-mapa {
    background: #1f1f23; border: 1px solid #3f3f46; border-radius: 4px; padding: 3px;
    text-align: center; cursor: grab; width: 100%; color: inherit; font: inherit; display: block;
    touch-action: none;
  }
  .tarjeta-mapa:active { cursor: grabbing; }
  .tarjeta-mapa.seleccionado { border-color: #3b82f6; background: #2a2a32; }
  .tarjeta-mapa.nueva { border-style: dashed; font-size: 14px; padding: 6px; cursor: pointer; }
  .previa-mapa { font-size: 16px; display: block; pointer-events: none; }
  .nombre-mapa { font-size: 9px; color: #d4d4d8; display: block; margin-top: 2px; pointer-events: none; }

  /* Blindaje total para que la imagen no robe el evento de arrastre */
  .contenedor-low {
    width: 100%; height: 36px; border-radius: 2px; overflow: hidden;
    background: #09090b; display: flex; align-items: center; justify-content: center;
    pointer-events: none;
  }
  .img-low {
    width: 100%; height: 100%; object-fit: cover;
    pointer-events: none; -webkit-user-drag: none; user-select: none;
  }

  /* Clon flotante que sigue al cursor */
  .elemento-flotante {
    position: fixed;
    pointer-events: none;
    z-index: 9999;
    transform: rotate(2deg) scale(1.05);
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.7);
    opacity: 0.92;
    transition: transform 0.05s ease;
  }
  .clon { cursor: grabbing !important; }

  .bloque-abajo { flex: 1; display: flex; flex-direction: column; overflow: hidden; gap: 5px; }
  .sub-busqueda { display: flex; flex-direction: column; gap: 3px; }
  .input-buscar { background: #09090b; border: 1px solid #3f3f46; color: white; padding: 3px 5px; border-radius: 4px; font-size: 11px; }
  .crear-entidad { flex-shrink: 0; background: #1b1b1f; border: 1px solid #3f3f46; border-radius: 4px; padding: 4px 6px; }
  .crear-entidad summary { cursor: pointer; color: #e4e4e7; font-size: 11px; }
  .crear-entidad form { display: flex; flex-direction: column; gap: 4px; margin-top: 5px; }
  .crear-entidad input, .crear-entidad select { min-width: 0; background: #09090b; border: 1px solid #3f3f46; color: white; border-radius: 3px; padding: 3px; font: inherit; }
  .campos-entidad { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .campos-entidad label { display: flex; align-items: center; justify-content: space-between; gap: 3px; color: #a1a1aa; font-size: 9px; }
  .campos-entidad input { width: 44px; }
  .crear-entidad button { background: #2563eb; border: 0; border-radius: 3px; color: white; padding: 4px; cursor: pointer; font: inherit; }
  .error-entidad { color: #f87171; overflow-wrap: anywhere; }
  .sub-entidades { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 7px; }
  .grupo-entidades { display: flex; flex-direction: column; gap: 3px; }
  .titulo-grupo { color: #a1a1aa; font-size: 9px; font-weight: 700; padding: 2px 1px; }
  .fila-entidad { width: 100%; display: flex; align-items: center; gap: 6px; background: #27272a; border: 0; color: inherit; text-align: left; font: inherit; padding: 5px; border-radius: 4px; cursor: grab; touch-action: none; }
  .fila-entidad:hover { background: #34343a; }
  .avatar-entidad { width: 28px; height: 28px; flex-shrink: 0; display: grid; place-items: center; overflow: hidden; border: 2px solid #fafafa; border-radius: 0; background: #09090b; color: white; font-weight: bold; }
  .avatar-entidad img { width: 100%; height: 100%; object-fit: cover; }

  .menu-contextual {
    position: fixed; background: #1c1c1f; border: 1px solid #3f3f46; border-radius: 4px;
    padding: 3px; display: flex; flex-direction: column; gap: 2px; z-index: 1000; box-shadow: 0 4px 12px rgba(0,0,0,0.5);
  }
  .menu-contextual button {
    background: none; border: none; color: #f4f4f5; font-size: 11px; padding: 4px 8px;
    text-align: left; cursor: pointer; border-radius: 2px;
  }
  .menu-contextual button:hover { background: #27272a; }
  .menu-contextual button.peligro:hover { background: #dc2626; color: white; }
/* Ventana flotante independiente */
  .ventana-flotante {
    position: fixed;
    width: 250px;
    background: #18181b;
    border: 1px solid #3f3f46;
    border-radius: 6px;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.7);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    user-select: none;
  }
  .cabecera-flotante {
    background: #27272a;
    padding: 6px 10px;
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: grab;
    border-bottom: 1px solid #3f3f46;
  }
  .cabecera-flotante:active {
    cursor: grabbing;
  }
  .icono-arrastre {
    color: #71717a;
    font-size: 11px;
    letter-spacing: -1px;
  }
  .titulo-flotante {
    font-size: 11px;
    font-weight: 600;
    color: #e4e4e7;
  }
  .cuerpo-flotante {
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .texto-modal {
    margin: 0;
    color: #e4e4e7;
    font-size: 11px;
    line-height: 1.4;
  }
  .texto-modal span {
    display: block;
    margin-top: 3px;
    color: #a1a1aa;
    font-size: 10px;
  }
  .acciones-modal {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }
  .btn-cancelar, .btn-confirmar {
    border: none;
    padding: 5px 10px;
    border-radius: 4px;
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
  }
  .btn-cancelar {
    background: #27272a;
    color: #d4d4d8;
  }
  .btn-cancelar:hover {
    background: #3f3f46;
  }
  .btn-confirmar {
    background: #2563eb;
    color: #ffffff;
  }
  .btn-confirmar:hover {
    background: #1d4ed8;
  }
</style>