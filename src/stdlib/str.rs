//
// Módulo estándar str — Manipulación avanzada de cadenas.
// Expone split, replace y trim en un Objeto::Diccionario
// bajo el nombre "str".
//

use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones
/// de manipulación de cadenas.
pub fn crear_modulo() -> Objeto {
    // Divide una cadena usando un separador y retorna un arreglo.
    fn str_split(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, separador)".to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => {
                return Objeto::Error(
                    "El primer argumento debe ser una cadena".to_string(),
                );
            }
        };
        let separador = match &args[1] {
            Objeto::Cadena(s) => s.clone(),
            _ => {
                return Objeto::Error(
                    "El separador debe ser una cadena".to_string(),
                );
            }
        };
        let partes: Vec<Objeto> = cadena
            .split(&separador)
            .map(|s| Objeto::Cadena(s.to_string()))
            .collect();
        Objeto::Arreglo(partes)
    }

    // Reemplaza todas las ocurrencias de un patrón en una cadena.
    fn str_replace(args: Vec<Objeto>) -> Objeto {
        if args.len() != 3 {
            return Objeto::Error(
                "Se esperaban 3 argumentos \
                 (cadena, objetivo, reemplazo)"
                    .to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => {
                return Objeto::Error(
                    "El primer argumento debe ser una cadena".to_string(),
                );
            }
        };
        let objetivo = match &args[1] {
            Objeto::Cadena(o) => o.clone(),
            _ => {
                return Objeto::Error(
                    "El objetivo debe ser una cadena".to_string(),
                );
            }
        };
        let reemplazo = match &args[2] {
            Objeto::Cadena(r) => r.clone(),
            _ => {
                return Objeto::Error(
                    "El reemplazo debe ser una cadena".to_string(),
                );
            }
        };
        Objeto::Cadena(cadena.replace(&objetivo, &reemplazo))
    }

    // Elimina espacios en blanco al inicio y final de una cadena.
    fn str_trim(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.trim().to_string(),
            _ => {
                return Objeto::Error(
                    "El argumento debe ser una cadena".to_string(),
                );
            }
        };
        Objeto::Cadena(cadena)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(
        LlaveHash::Cadena("split".to_string()),
        Objeto::Nativa(str_split as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("replace".to_string()),
        Objeto::Nativa(str_replace as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("trim".to_string()),
        Objeto::Nativa(str_trim as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa)
}
