//
// Módulo estándar arr — Operaciones sobre arreglos.
// Expone len, push y pop en un Objeto::Diccionario
// bajo el nombre "arr".
//

use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones
/// de manipulación de arreglos.
pub fn crear_modulo() -> Objeto {
    // Retorna la longitud de un arreglo.
    fn arr_len(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(a) => Objeto::Entero(a.len() as i64),
            _ => Objeto::Error(
                "El argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    // Agrega un elemento al final del arreglo.
    // Retorna el nuevo arreglo (clonado + elemento).
    fn arr_push(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (arreglo, elemento)".to_string(),
            );
        }
        let mut args_iter = args.into_iter();
        let arreglo = args_iter.next().unwrap();
        let elemento = args_iter.next().unwrap();
        match arreglo {
            Objeto::Arreglo(mut v) => {
                v.push(elemento);
                Objeto::Arreglo(v)
            }
            _ => Objeto::Error(
                "El primer argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    // Extrae el último elemento de un arreglo.
    // Retorna un diccionario con "elemento" (el valor extraído
    // o nulo si el arreglo estaba vacío) y "arreglo" (el
    // arreglo resultante sin ese elemento).
    fn arr_pop(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match args.into_iter().next().unwrap() {
            Objeto::Arreglo(mut v) => {
                let elemento = v.pop();
                let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
                mapa.insert(
                    LlaveHash::Cadena("elemento".to_string()),
                    elemento.unwrap_or(Objeto::Nulo),
                );
                mapa.insert(
                    LlaveHash::Cadena("arreglo".to_string()),
                    Objeto::Arreglo(v),
                );
                Objeto::Diccionario(mapa)
            }
            _ => Objeto::Error(
                "El argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(
        LlaveHash::Cadena("len".to_string()),
        Objeto::Nativa(arr_len as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("push".to_string()),
        Objeto::Nativa(arr_push as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("pop".to_string()),
        Objeto::Nativa(arr_pop as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa)
}
