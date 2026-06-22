//
// Módulo estándar time — Control de flujo temporal.
// Expone now (milisegundos desde UNIX Epoch) y sleep
// (pausa en milisegundos) en un Objeto::Diccionario
// bajo el nombre "time".
//

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones
/// de control temporal.
pub fn crear_modulo() -> Objeto {
    // Retorna los milisegundos transcurridos desde el UNIX Epoch.
    fn time_now(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "La función now no recibe argumentos".to_string(),
            );
        }
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Objeto::Entero(d.as_millis() as i64),
            Err(e) => Objeto::Error(format!(
                "Error al obtener el tiempo: {}", e
            )),
        }
    }

    // Pausa la ejecución durante la cantidad de milisegundos
    // especificada. Recibe 1 argumento numérico.
    fn time_sleep(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (milisegundos)".to_string(),
            );
        }
        let ms = match &args[0] {
            Objeto::Entero(n) => *n,
            Objeto::Flotante(n) => *n as i64,
            _ => {
                return Objeto::Error(
                    "Se esperaba un número (milisegundos)".to_string(),
                );
            }
        };
        if ms < 0 {
            return Objeto::Error(
                "Los milisegundos no pueden ser negativos".to_string(),
            );
        }
        thread::sleep(Duration::from_millis(ms as u64));
        Objeto::Nulo
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(
        LlaveHash::Cadena("now".to_string()),
        Objeto::Nativa(time_now as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("sleep".to_string()),
        Objeto::Nativa(time_sleep as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa)
}
