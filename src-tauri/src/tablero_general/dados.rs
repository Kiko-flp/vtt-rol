// src-tauri/src/tablero_general/dados.rs
use std::fmt::Write;
use tauri::State;
use crate::tablero_general::partida::{registrar_comando_personal, registrar_tirada, AppState};

pub struct Tirada {
    pub signo: i32,
    pub veces: i32,
    pub d: i32,
    pub bonificador: Option<i32>,
    pub ventaja: bool,
    pub desventaja: bool,
}

// Resalta en negrita si es 1 (pifia) o el valor máximo de caras (crítico)
fn formatear_num(valor: i32, caras: i32) -> String {
    if valor == 1 || valor == caras {
        format!("<b>{valor}</b>")
    } else {
        format!("{valor}")
    }
}

fn parsear_comando(comando: &str) -> Result<Vec<Tirada>, String> {
    let mut tiradas: Vec<Tirada> = Vec::new();
    let mut signo_actual = 1;
    let mut buffer = String::new();
    let mut terminos = Vec::new();

    // 1. Separamos términos respetando '+' y '-'
    for c in comando.chars() {
        if c == '+' || c == '-' {
            let limpio = buffer.trim();
            if !limpio.is_empty() {
                terminos.push((signo_actual, limpio.to_string()));
                buffer.clear();
            }
            signo_actual = if c == '+' { 1 } else { -1 };
        } else {
            buffer.push(c);
        }
    }
    let limpio = buffer.trim();
    if !limpio.is_empty() {
        terminos.push((signo_actual, limpio.to_string()));
    }

    if terminos.is_empty() {
        return Err("Comando vacío".into());
    }

    // 2. Parseamos cada término
    for (signo, termino) in terminos {
        if let Some(pos_d) = termino.find(|c| c == 'd' || c == 'D') {
            let (veces_str, resto_str) = termino.split_at(pos_d);
            let resto = resto_str[1..].trim();

            let veces: i32 = if veces_str.trim().is_empty() {
                1
            } else {
                veces_str.trim().parse().map_err(|_| "Cantidad inválida")?
            };

            // Separamos número de caras de modificadores (adv, desv, dis)
            let pos_mods = resto.find(|c: char| !c.is_ascii_digit()).unwrap_or(resto.len());
            let (caras_str, mods_str) = resto.split_at(pos_mods);

            let caras: i32 = caras_str.trim().parse().map_err(|_| "Caras inválidas")?;
            if veces <= 0 || caras <= 0 {
                return Err("Los dados y caras deben ser mayores a 0".into());
            }

            let mods_lower = mods_str.to_lowercase();
            let ventaja = mods_lower.contains("adv");
            let desventaja = mods_lower.contains("desv") || mods_lower.contains("dis");

            let test_limpio = mods_lower.replace("adv", "").replace("desv", "").replace("dis", "");
            if !test_limpio.trim().is_empty() {
                return Err(format!("Modificador desconocido: '{mods_str}'"));
            }

            // Agrupamos dados idénticos del mismo tipo y signo
            if let Some(existente) = tiradas.iter_mut().find(|t| {
                t.bonificador.is_none()
                    && t.d == caras
                    && t.signo == signo
                    && t.ventaja == ventaja
                    && t.desventaja == desventaja
            }) {
                existente.veces += veces;
            } else {
                tiradas.push(Tirada {
                    signo,
                    veces,
                    d: caras,
                    bonificador: None,
                    ventaja,
                    desventaja,
                });
            }
        } else {
            let bonif: i32 = termino.parse().map_err(|_| "Número inválido")?;
            tiradas.push(Tirada {
                signo,
                veces: 0,
                d: 0,
                bonificador: Some(bonif),
                ventaja: false,
                desventaja: false,
            });
        }
    }

    Ok(tiradas)
}

fn ejecutar_tiradas(tiradas: &[Tirada]) -> String {
    let mut resultado = String::new();
    let mut total: i32 = 0;
    let mut primero = true;

    for t in tiradas {
        if primero {
            if t.signo < 0 {
                resultado.push_str("- ");
            }
        } else {
            if t.signo < 0 {
                resultado.push_str(" - ");
            } else {
                resultado.push_str(" + ");
            }
        }
        primero = false;

        // Bonificadores numéricos directos
        if let Some(bonificador) = t.bonificador {
            let _ = write!(resultado, "{bonificador}");
            total += t.signo * bonificador;
            continue;
        }

        // Encabezado del dado (ej: 1d20, 1d20adv, 1d20desv)
        let mod_tipo = if t.ventaja {
            "adv"
        } else if t.desventaja {
            "desv"
        } else {
            ""
        };
        let _ = write!(resultado, "{}d{}{} (", t.veces, t.d, mod_tipo);

        let mut subtotal_dados: i32 = 0;

        for roll_idx in 0..t.veces {
            if roll_idx > 0 {
                resultado.push_str(", ");
            }

            if t.ventaja || t.desventaja {
                // Tiramos 2 dados directamente sin alocar memoria
                let r1 = rand::random_range(1..=t.d);
                let r2 = rand::random_range(1..=t.d);

                let (ganador, perdedor) = if t.ventaja {
                    if r1 >= r2 { (r1, r2) } else { (r2, r1) }
                } else {
                    if r1 <= r2 { (r1, r2) } else { (r2, r1) }
                };

                subtotal_dados += ganador;

                // El ganador se muestra en negrita <b>
                if t.veces == 1 {
                    let _ = write!(resultado, "<b>{ganador}</b>, {perdedor}");
                } else {
                    let _ = write!(resultado, "[<b>{ganador}</b>, {perdedor}]");
                }
            } else {
                // Tirada normal directa
                let r = rand::random_range(1..=t.d);
                subtotal_dados += r;
                let _ = write!(resultado, "{}", formatear_num(r, t.d));
            }
        }
        resultado.push(')');

        total += t.signo * subtotal_dados;
    }

    let _ = write!(resultado, " = {total}");
    resultado
}

#[tauri::command]
pub fn tirar_dado(comando: String, state: State<'_, AppState>) -> Result<String, String> {
    let tiradas = parsear_comando(&comando)?;
    let resultado = ejecutar_tiradas(&tiradas);

    // 1. Registra en el estado de la partida / deltas
    registrar_tirada(&state, &resultado);

    // 2. Guarda el comando en el historial personal de este cliente directamente en disco
    registrar_comando_personal(&state, &comando);

    Ok(resultado)
}