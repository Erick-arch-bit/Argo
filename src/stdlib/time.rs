use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::thread;

use crate::evaluator::{LlaveHash, Objeto};

pub fn crear_modulo() -> Objeto {
    fn time_ahora(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "La función now no recibe argumentos".to_string(),
            Vec::new());
        }
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Objeto::Entero(d.as_millis() as i64),
            Err(e) => Objeto::Error(format!(
                "Error al obtener el tiempo: {}", e
            ), Vec::new()),
        }
    }

    fn time_dormir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (milisegundos)".to_string(),
            Vec::new());
        }
        let ms = match &args[0] {
            Objeto::Entero(n) => *n,
            Objeto::Flotante(n) => *n as i64,
            _ => {
                return Objeto::Error(
                    "Se esperaba un número (milisegundos)".to_string(),
                Vec::new());
            }
        };
        if ms < 0 {
            return Objeto::Error(
                "Los milisegundos no pueden ser negativos".to_string(),
            Vec::new());
        }
        thread::sleep(Duration::from_millis(ms as u64));
        Objeto::Nulo
    }

    fn time_segundos(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "time.segundos no recibe argumentos".to_string(),
            Vec::new());
        }
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Objeto::Entero(d.as_secs() as i64),
            Err(e) => Objeto::Error(format!(
                "Error al obtener el tiempo: {}", e
            ), Vec::new()),
        }
    }

    fn time_micros(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "time.micros no recibe argumentos".to_string(),
            Vec::new());
        }
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => Objeto::Entero(d.as_micros() as i64),
            Err(e) => Objeto::Error(format!(
                "Error al obtener el tiempo: {}", e
            ), Vec::new()),
        }
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(LlaveHash::Cadena("ahora".to_string()), Objeto::Nativa(time_ahora as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("dormir".to_string()), Objeto::Nativa(time_dormir as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("segundos".to_string()), Objeto::Nativa(time_segundos as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("micros".to_string()), Objeto::Nativa(time_micros as fn(Vec<Objeto>) -> Objeto));

    Objeto::Diccionario(mapa)
}
