<!-- src/routes/+page.svelte -->
<script lang="ts">
  import type { ConfiguracionRejilla, Zona, Entidad, RegistroTirada } from '$lib/types';
  
  import PanelIzquierdo from './componentes/PanelIzquierdo.svelte';
  import PanelDerecho from './componentes/PanelDerecho.svelte';
  import Tablero from './componentes/Tablero.svelte';
  import BarraRejilla from './componentes/BarraRejilla.svelte';
  import BarraDados from './componentes/BarraDados.svelte';
  import ModalFicha from './componentes/ModalFicha.svelte';

  // Configuración de la rejilla (reactiva con la barra superior y el canvas)
  let rejilla: ConfiguracionRejilla = {
    tamano: 40,
    color: "#ffffff",
    grosor: 1,
    opacidad: 0.3 // 30% como en tu dibujo
  };

  // Datos de prueba para el prototipo
  let zonas: Zona[] = [
    { id: "1", nombre: "Mazmorra Nivel 1" },
    { id: "2", nombre: "Taberna del Jabalí" },
    { id: "3", nombre: "Bosque Sombrío" }
  ];

  let entidades: Entidad[] = [
    { id: "e1", nombre: "Orco Guerrero", tipo: "enemigo", vidaActual: 15, vidaMax: 15, armadura: 12, cuerpo: 3, zonaId: "1", esPublico: false },
    { id: "n1", nombre: "Tabernero", tipo: "npc", vidaActual: 10, vidaMax: 10, armadura: 10, cuerpo: 1, zonaId: "2", esPublico: true },
    { id: "j1", nombre: "Valeros (Guerrero)", tipo: "jugador", vidaActual: 24, vidaMax: 28, armadura: 16, cuerpo: 4, zonaId: "1", esPublico: true }
  ];

  let historialTiradas: RegistroTirada[] = [
    { id: "t1", formula: "3x1d6 + 3", desglose: "(3, 5, 2)", total: 13 },
    { id: "t2", formula: "1x1d20", desglose: "(15)", total: 15 }
  ];

  // Control del modal de ficha
  let entidadSeleccionada: Entidad | null = null;

  function abrirFicha(entidad: Entidad) {
    entidadSeleccionada = entidad;
  }

  function realizarTirada(dado: string, bonif: number) {
    // Prototipo de tirada rápido en local
    const caras = parseInt(dado.replace('d', ''));
    const resultado = Math.floor(Math.random() * caras) + 1;
    historialTiradas = [
      {
        id: Date.now().toString(),
        formula: `1x${dado} ${bonif >= 0 ? '+' : ''}${bonif}`,
        desglose: `(${resultado})`,
        total: resultado + bonif
      },
      ...historialTiradas
    ];
  }
</script>

<div class="contenedor-app">
  <!-- 1. Panel Izquierdo: Zonas y Banco -->
  <PanelIzquierdo 
    {zonas} 
    bancoEntidades={entidades} 
    onAbrirFicha={abrirFicha} 
  />

  <!-- 2. Área Central: Barra Rejilla + Lienzo + Barra Dados -->
  <main class="area-central">
    <header class="cabecera-tablero">
      <BarraRejilla bind:rejilla />
    </header>

    <section class="espacio-lienzo">
      <Tablero {rejilla} />
    </section>

    <footer class="pie-dados">
      <BarraDados onLanzarTirada={realizarTirada} />
    </footer>
  </main>

  <!-- 3. Panel Derecho: Activos y Tiradas -->
  <PanelDerecho 
    entidadesActivas={entidades} 
    {historialTiradas} 
    onAbrirFicha={abrirFicha} 
  />

  <!-- 4. Ventana Modal Ficha (si hay una seleccionada) -->
  {#if entidadSeleccionada}
    <ModalFicha 
      entidad={entidadSeleccionada} 
      {zonas} 
      onCerrar={() => entidadSeleccionada = null} 
    />
  {/if}
</div>

<style>
  :global(body, html) {
    margin: 0;
    padding: 0;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    background-color: #09090b;
    color: #f4f4f5;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .contenedor-app {
    display: flex;
    width: 100vw;
    height: 100vh;
  }

  .area-central {
    flex: 1;
    display: flex;
    flex-direction: column;
    position: relative;
    overflow: hidden;
  }

  .cabecera-tablero {
    position: absolute;
    top: 10px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
  }

  .espacio-lienzo {
    flex: 1;
    width: 100%;
    height: 100%;
  }

  .pie-dados {
    position: absolute;
    bottom: 12px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 10;
  }
</style>