<!-- src/routes/componentes/PanelDerecho.svelte -->
<script>
  import { invoke } from "@tauri-apps/api/core";

  let {
    instancias = [],
    tiradas = [],
    instanciaSeleccionada = null,
    comandoChat = $bindable(""),
    onSeleccionarInstancia,
    onModificarVida,
    onEnviarTirada
  } = $props();

  let inputChatRef;
  
  // Estado de navegación local estilo terminal
  let historial = [];
  let indiceHistorial = -1; // -1 significa que no estamos navegando en el historial
  let borradorTexto = "";   // Almacena lo que el usuario tenía escrito

  export function enfocarChat() {
    inputChatRef?.focus();
  }

  function colocarCursorAlFinal() {
    requestAnimationFrame(() => {
      if (inputChatRef) {
        const len = inputChatRef.value.length;
        inputChatRef.setSelectionRange(len, len);
      }
    });
  }

  async function manejarKeyDown(e) {
    if (e.key === "Enter") {
      if (comandoChat.trim()) {
        onEnviarTirada(comandoChat.trim());
        comandoChat = "";
        // Reiniciamos el estado de navegación
        indiceHistorial = -1;
        borradorTexto = "";
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();

      // Si no estábamos navegando, pedimos el historial a Rust
      if (indiceHistorial === -1) {
        historial = await invoke("obtener_historial_personal");
        if (historial.length === 0) return;
        
        borradorTexto = comandoChat; // Guardamos lo que estaba escribiendo
        indiceHistorial = historial.length;
      }

      if (indiceHistorial > 0) {
        indiceHistorial--;
        comandoChat = historial[indiceHistorial];
        colocarCursorAlFinal();
      }
    } else if (e.key === "ArrowDown") {
      if (indiceHistorial === -1) return; // Si no estábamos navegando, no hace nada
      e.preventDefault();

      if (indiceHistorial < historial.length) {
        indiceHistorial++;
        
        // Si volvemos abajo del todo, restauramos el texto original
        if (indiceHistorial === historial.length) {
          comandoChat = borradorTexto;
          indiceHistorial = -1; // Salimos del modo historial
        } else {
          comandoChat = historial[indiceHistorial];
        }
        colocarCursorAlFinal();
      }
    }
  }

  let enemigos = $derived(instancias.filter((i) => i.base.tipo === "Enemigo"));
  let npcs = $derived(instancias.filter((i) => i.base.tipo === "NPC"));
  let jugadores = $derived(instancias.filter((i) => i.base.tipo === "Jugador"));
</script>

<aside class="panel-derecho">
  <!-- Subpágina 5: Fichas Activas -->
  <div class="sub-fichas-activas">
    <span class="titulo-mini">FICHAS ACTIVAS</span>
    <div class="scroll-fichas">
      <div class="grupo rojo">ENEMIGOS</div>
      {#each enemigos as e}
        <button
          type="button"
          class="fila-ficha"
          class:muerto={e.vida_actual <= 0}
          class:seleccionado={instanciaSeleccionada?.id_instancia === e.id_instancia}
          onclick={() => onSeleccionarInstancia(e)}
        >
          <span class="icono cuadrado"></span>
          <span class="nombre">{e.base.nombre}</span>
          <span class="pv">{e.vida_actual}/{e.base.vida_max}</span>
        </button>
      {/each}

      <div class="grupo azul">NPCs</div>
      {#each npcs as n}
        <button
          type="button"
          class="fila-ficha"
          class:muerto={n.vida_actual <= 0}
          class:seleccionado={instanciaSeleccionada?.id_instancia === n.id_instancia}
          onclick={() => onSeleccionarInstancia(n)}
        >
          <span class="icono rombo"></span>
          <span class="nombre">{n.base.nombre}</span>
          <span class="pv">{n.vida_actual}</span>
        </button>
      {/each}

      <div class="grupo verde">JUGADORES</div>
      {#each jugadores as j}
        <button
          type="button"
          class="fila-ficha"
          class:muerto={j.vida_actual <= 0}
          class:seleccionado={instanciaSeleccionada?.id_instancia === j.id_instancia}
          onclick={() => onSeleccionarInstancia(j)}
        >
          <span class="icono circulo"></span>
          <span class="nombre">{j.base.nombre}</span>
          <span class="pv">{j.vida_actual}/{j.base.vida_max}</span>
        </button>
      {/each}
    </div>
  </div>

  <!-- Subpágina 6: Parámetros -->
  <div class="sub-parametros">
    <span class="titulo-mini">PARÁMETROS</span>
    {#if instanciaSeleccionada}
      <div class="caja-parametros">
        <div class="param">
          <span class="label">V (Velocidad)</span>
          <span class="valor-destacado">{instanciaSeleccionada.base.velocidad} m</span>
        </div>
        <div class="param">
          <span class="label">RA (Rango Arma)</span>
          <span class="valor-destacado">{instanciaSeleccionada.base.rango_arma} m</span>
        </div>
        <div class="param">
          <span class="label">Cuerpo</span>
          <span class="valor-destacado">
            {#if instanciaSeleccionada.base.cuerpo.Grande}
              Grande ({instanciaSeleccionada.base.cuerpo.Grande.join('x')})
            {:else}
              1x1
            {/if}
          </span>
        </div>
        <div class="param">
          <span class="label">Armadura</span>
          <span class="valor-destacado">{instanciaSeleccionada.base.armadura}</span>
        </div>
        <div class="param vida">
          <span class="label">Vida:</span>
          <div class="control-vida">
            <button onclick={() => onModificarVida(instanciaSeleccionada, -1)}>-</button>
            <span class="valor-destacado">{instanciaSeleccionada.vida_actual} / {instanciaSeleccionada.base.vida_max}</span>
            <button onclick={() => onModificarVida(instanciaSeleccionada, 1)}>+</button>
          </div>
        </div>
      </div>
    {:else}
      <div class="sin-seleccion">Pulsa una ficha para ver sus parámetros</div>
    {/if}
  </div>

  <!-- Subpágina 7: Tiradas y Chat integrado en el mismo recuadro -->
  <div class="sub-tiradas">
    <span class="titulo-mini">TIRADAS</span>
    <div class="recuadro-chat-tiradas">
      <div class="lista-tiradas">
        {#each tiradas as t}
          <div class="linea-tirada" class:error={!t.total}>
            <span class="hora">{t.hora}</span>
            <span class="cmd">{@html t.comando}</span>
            {#if t.total}
              <strong class="total">= {t.total}</strong>
            {/if}
          </div>
        {/each}
      </div>

      <div class="input-comando-contenedor">
        <input
          type="text"
          bind:this={inputChatRef}
          bind:value={comandoChat}
          placeholder="Escribir comando (ej: 2d6 + 3)..."
          onkeydown={manejarKeyDown}
          class="input-chat"
        />
      </div>
    </div>
  </div>
</aside>

<style>
  .panel-derecho {
    width: clamp(200px, 16vw, 250px); /* Borde más estrecho */
    height: 100%;
    background: #141416;
    border-left: 1px solid #27272a;
    display: flex;
    flex-direction: column;
    padding: 6px;
    gap: 6px;
    box-sizing: border-box;
    font-size: 12px;
  }
  .titulo-mini { font-size: 10px; font-weight: bold; color: #71717a; margin-bottom: 3px; display: block; }

  /* Subpágina 5 */
  .sub-fichas-activas { flex: 1.2; display: flex; flex-direction: column; overflow: hidden; }
  .scroll-fichas { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
  .grupo { font-size: 10px; font-weight: bold; margin-top: 4px; border-bottom: 1px solid; }
  .grupo.rojo { color: #ef4444; border-color: #ef4444; }
  .grupo.azul { color: #3b82f6; border-color: #3b82f6; }
  .grupo.verde { color: #22c55e; border-color: #22c55e; }
  .fila-ficha {
    display: flex; align-items: center; gap: 6px; background: #1f1f23; padding: 4px 6px; border-radius: 4px; cursor: pointer;
    /* reset de botón para que se vea igual que el div anterior */
    border: 1px solid transparent; width: 100%; color: inherit; font: inherit; text-align: left;
  }
  .fila-ficha.seleccionado { border-color: #3b82f6; }
  .fila-ficha.muerto { opacity: 0.35; filter: grayscale(100%); }
  .icono { width: 10px; height: 10px; flex-shrink: 0; }
  .cuadrado { background: #ef4444; }
  .rombo { background: #3b82f6; transform: rotate(45deg); }
  .circulo { background: #22c55e; border-radius: 50%; }
  .nombre { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }

  /* Subpágina 6: Parámetros aumentados */
  .sub-parametros {
    height: 150px;
    background: #1b1b1f;
    border: 1px solid #27272a;
    border-radius: 6px;
    padding: 6px;
    display: flex;
    flex-direction: column;
  }
  .caja-parametros { display: grid; grid-template-columns: 1fr 1fr; gap: 4px; }
  .param { background: #27272a; padding: 5px; border-radius: 4px; }
  .param.vida { grid-column: span 2; display: flex; justify-content: space-between; align-items: center; }
  .label { color: #a1a1aa; font-size: 10px; display: block; }
  .valor-destacado { font-size: 13px; font-weight: bold; color: #f4f4f5; }
  .control-vida { display: flex; align-items: center; gap: 6px; }
  .control-vida button {
    background: #3f3f46; color: white; border: none; width: 22px; height: 22px; border-radius: 3px; cursor: pointer; font-weight: bold;
  }
  .sin-seleccion { color: #71717a; text-align: center; margin: auto 0; font-size: 11px; }

  /* Subpágina 7: Tiradas + Chat ampliado */
  .sub-tiradas {
    height: 200px; /* Recuadro más grande */
    display: flex;
    flex-direction: column;
  }
  .recuadro-chat-tiradas {
    flex: 1;
    background: #09090b;
    border: 1px solid #27272a;
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .linea-tirada.error .cmd {
    color: #ef4444;
    font-style: italic;
  }

  .lista-tiradas {
    flex: 1;
    overflow-y: auto;
    padding: 6px;
    font-family: monospace;
    display: flex;
    flex-direction: column;
    gap: 4px;

    /* SCROLLBAR ULTRAFINA (Firefox y navegadores modernos) */
    scrollbar-width: thin;
    scrollbar-color: #3f3f46 transparent;
  }
  /* SCROLLBAR ULTRAFINA (Chrome, Edge, Tauri WebView) */
  .lista-tiradas::-webkit-scrollbar {
    width: 3px; /* Grosor de solo 3px (una línea discreta) */
  }
  .lista-tiradas::-webkit-scrollbar-track {
    background: transparent;
  }
  .lista-tiradas::-webkit-scrollbar-thumb {
    background: #3f3f46;
    border-radius: 4px;
  }
  .lista-tiradas::-webkit-scrollbar-thumb:hover {
    background: #71717a;
  }


 .linea-tirada {
    display: flex;
    align-items: flex-start;
    gap: 6px;
    padding: 1px 0;
  }

  .hora {
    color: #52525b;
    font-size: 9px;
    flex-shrink: 0;
    margin-top: 2px;
    letter-spacing: -0.5px;
    user-select: none;
  }

  .cmd :global(b) {
    font-weight: 900;
    color: #ffffff; /* Blanco brillante para que destaque del resto de números */
    text-shadow: 0 0 2px rgba(255, 255, 255, 0.4);
  }

  .cmd {
    flex: 1;
    color: #e4e4e7;
    font-size: 11px;
    white-space: normal;
    word-break: break-word;
    line-height: 1.25;
  }

  .total {
    color: #4ade80;
    font-size: 12px;
    flex-shrink: 0;
    white-space: nowrap;
  }

  .input-comando-contenedor {
    border-top: 1px solid #27272a; /* Línea fina separadora */
    background: #111113;
    padding: 4px 6px;
  }
  .input-chat {
    width: 100%;
    background: transparent;
    border: none;
    color: white;
    font-size: 12px;
    outline: none;
    box-sizing: border-box;
  }
</style>
