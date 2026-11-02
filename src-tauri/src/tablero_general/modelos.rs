// src-tauri/src/tablero_general/modelos.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum TipoEntidad {
    Enemigo,
    NPC,
    Jugador,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Cuerpo {
    Grande(Vec<u8>), // dimensiones del cuerpo para después saber cómo representarlos
    Pequenio,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Coordenada {
    Localizacion(Vec<i16>), // coordenadas en el eje
    NoPuesto,
}

// Representa la definición de una entidad
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entidad {
    pub id: u16,
    pub nombre: String,
    pub tipo: TipoEntidad,
    pub vida_max: i16,
    pub armadura: i16,
    pub velocidad: i16,
    pub rango_arma: i16,
    pub cuerpo: Cuerpo, // tiene que ser x * y, si es 1*1 tratarlo como unitario
    pub zona_id: u16,
    pub foto_url: String,
    pub es_publico: bool,
    pub archivos_adjuntos: Vec<Option<String>>, // si es un txt tratarlo como nota del programa, y el resto pues lo que sea, para no limitar solo poder tener un pdf
}

// Representa una instancia de una entidad, si la entidad es única habrá que manejar no tener más de una copia
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Instancia {
    pub id_instancia: u16,
    pub id_mapa: u16, // <-- NUEVO: sabemos en qué mapa está instanciada
    pub base: Entidad,
    pub vida_actual: i16,
    pub coordenadas: Coordenada, // una entidad puede estar activa pero no estar puesto en el mapa, la idea es arrastrarlos
    pub activo: bool, // Este valor representa si está vivo en el panel derecho, los jugadores siempre están activos y representados a la derecha, no hace falta dibujarlos en el panel izquierdo
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Zona {
    pub id: u16,
    pub nombre: String,
}

// El mapa se relaciona con una zona, y es dentro del mapa donde se guarda la configuración de la rejilla para cuando lo pulses se dibuje.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Mapa {
    pub id: u16,
    pub id_zona: u16,
    pub url_imagen_fondo: String,
    pub ancho: u16, // <-- 1000px por defecto
    pub alto: u16,  // <-- 1000px por defecto
    pub rejilla: u16,
    pub color_opacidad: Vec<u8>, // rgba: [r, g, b, a]
    pub grosor: u16,
}

// No me interesa tener "Desglose"
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tiradas {
    pub id: u16,
    pub total: i16,
    pub comando: String, // del estilo 1d6 + 2d8 etc
    pub hora: String,    // <-- NUEVO: hora con segundos "HH:MM:SS"
}

// ==========================================
// FUNCIONES CON DATOS DE EJEMPLO
// ==========================================

#[tauri::command]
pub fn obtener_zonas() -> Vec<Zona> {
    vec![
        Zona { id: 1, nombre: "Mazmorra Nivel 1".into() },
        Zona { id: 2, nombre: "Taberna".into() },
        Zona { id: 3, nombre: "Bosque".into() },
        Zona { id: 4, nombre: "Cripta Antigua".into() },
    ]
}

#[tauri::command]
pub fn obtener_mapas_zona(zona_id: u16) -> Vec<Mapa> {
    match zona_id {
        1 => vec![
            Mapa {
                id: 101,
                id_zona: 1,
                url_imagen_fondo: r"C:\Users\Kiro\Documents\personal\proyecto\MH\mapas\Bosque\4.png".into(),
                ancho: 1000,
                alto: 1000,
                rejilla: 40,
                color_opacidad: vec![255, 255, 255, 76], // blanco ~30%
                grosor: 1,
            },
            Mapa {
                id: 102,
                id_zona: 1,
                url_imagen_fondo: "".into(),
                ancho: 1000,
                alto: 1000,
                rejilla: 50,
                color_opacidad: vec![255, 255, 255, 76],
                grosor: 1,
            },
        ],
        _ => vec![
            Mapa {
                id: 201,
                id_zona: zona_id,
                url_imagen_fondo: "".into(),
                ancho: 1000,
                alto: 1000,
                rejilla: 50,
                color_opacidad: vec![255, 255, 255, 76],
                grosor: 1,
            }
        ],
    }
}

#[tauri::command]
pub fn obtener_banco_entidades() -> Vec<Entidad> {
    vec![
        Entidad {
            id: 1,
            nombre: "Orco Guerrero".into(),
            tipo: TipoEntidad::Enemigo,
            vida_max: 18,
            armadura: 12,
            velocidad: 9,
            rango_arma: 1,
            cuerpo: Cuerpo::Pequenio,
            zona_id: 1,
            foto_url: "".into(),
            es_publico: false,
            archivos_adjuntos: vec![Some("notas_orco.txt".into())],
        },
        Entidad {
            id: 2,
            nombre: "Goblin Arquero".into(),
            tipo: TipoEntidad::Enemigo,
            vida_max: 8,
            armadura: 10,
            velocidad: 10,
            rango_arma: 15,
            cuerpo: Cuerpo::Pequenio,
            zona_id: 1,
            foto_url: "".into(),
            es_publico: false,
            archivos_adjuntos: vec![],
        },
        Entidad {
            id: 3,
            nombre: "Tabernero Gruñón".into(),
            tipo: TipoEntidad::NPC,
            vida_max: 12,
            armadura: 10,
            velocidad: 6,
            rango_arma: 1,
            cuerpo: Cuerpo::Pequenio,
            zona_id: 2,
            foto_url: "".into(),
            es_publico: true,
            archivos_adjuntos: vec![Some("historia.pdf".into())],
        },
    ]
}

#[tauri::command]
pub fn obtener_instancias_activas() -> Vec<Instancia> {
    vec![
        Instancia {
            id_instancia: 1001,
            id_mapa: 101,
            base: Entidad {
                id: 99,
                nombre: "Guerrero Valeroso".into(),
                tipo: TipoEntidad::Jugador,
                vida_max: 28,
                armadura: 16,
                velocidad: 9,
                rango_arma: 2,
                cuerpo: Cuerpo::Pequenio,
                zona_id: 1,
                foto_url: "".into(),
                es_publico: true,
                archivos_adjuntos: vec![],
            },
            vida_actual: -1,
            coordenadas: Coordenada::Localizacion(vec![5, 8]),
            activo: true,
        },
        Instancia {
            id_instancia: 1002,
            id_mapa: 101,
            base: Entidad {
                id: 1,
                nombre: "Orco Líder".into(),
                tipo: TipoEntidad::Enemigo,
                vida_max: 18,
                armadura: 12,
                velocidad: 9,
                rango_arma: 2,
                cuerpo: Cuerpo::Grande(vec![2, 2]),
                zona_id: 1,
                foto_url: "".into(),
                es_publico: false,
                archivos_adjuntos: vec![],
            },
            vida_actual: 10,
            coordenadas: Coordenada::NoPuesto,
            activo: true,
        },
    ]
}

#[tauri::command]
pub fn obtener_historial_tiradas() -> Vec<Tiradas> {
    vec![
        Tiradas { id: 1, comando: "3x1d6 + 3".into(), total: 13, hora: "02:44:10".into() },
        Tiradas { id: 2, comando: "1x1d20".into(), total: 15, hora: "02:45:02".into() },
        Tiradas { id: 3, comando: "5x1d4".into(), total: 10, hora: "02:45:48".into() },
    ]
}