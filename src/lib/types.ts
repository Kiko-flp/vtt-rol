//Esta definición es temporal, los datos deben vivir en rust, y este exporta los datos al resto del programa
/* Ejemplo después, en rust:
#[derive(serde::Serialize)]
pub struct Entidad {
    pub nombre: String,
    pub vida: i32,
}

#[tauri::command]
pub fn obtener_enemigo() -> Entidad {
    Entidad {
        nombre: "Orco".into(),
        vida: 15,
    }
}
    
En svelte
<script>
  import { invoke } from "@tauri-apps/api/core";

  // Una variable normal de toda la vida, vacía al inicio
  let enemigo = null;

  async function cargar() {
    // Rust lo envía -> Tauri lo convierte en JSON -> Svelte lo tiene
    enemigo = await invoke("obtener_enemigo");
  }
</script>

<button on:click={cargar}>Cargar Enemigo</button>

<!-- Si existe el enemigo, pintamos sus datos -->
{#if enemigo}
  <h1>{enemigo.nombre}</h1>
  <p>Vida: {enemigo.vida}</p>
{/if}

*/
// src/lib/types.ts
export type TipoEntidad = 'jugador' | 'npc' | 'enemigo';

export interface Entidad {
  id: string;
  nombre: string;
  tipo: TipoEntidad;
  vidaActual: number;
  vidaMax: number;
  armadura: number;
  cuerpo: number; //tiene que ser x * y, si es 1*1 tratarlo como unitario
  zonaId: string;
  fotoUrl?: string;
  esPublico: boolean;
  notas?: string;
  pdfAdjuntoUrl?: string;
}

export interface Zona {
  id: string;
  nombre: string;
}

export interface ConfiguracionRejilla {
  tamano: number;
  color: string;
  grosor: number;
  opacidad: number; // de 0 a 1 (ej: 0.3 = 30%)
}

export interface RegistroTirada {
  id: string;
  formula: string;
  desglose: string;
  total: number;
}