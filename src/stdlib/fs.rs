//
// Módulo estándar fs — Operaciones del sistema de archivos.
// Expone leer y escribir archivos de texto en un
// Objeto::Diccionario bajo el nombre "fs".
//

use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones
/// del sistema de archivos.
pub fn crear_modulo() -> Objeto {
    // Lee el contenido completo de un archivo como cadena.
    // Recibe 1 argumento: Objeto::Cadena(ruta).
    fn fs_leer(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(ruta) => match std::fs::read_to_string(ruta) {
                Ok(contenido) => Objeto::Cadena(contenido),
                Err(e) => Objeto::Error(format!(
                    "Error al leer el archivo {}: {}", ruta, e
                )),
            },
            _ => Objeto::Error(
                "El argumento debe ser una cadena (ruta)".to_string(),
            ),
        }
    }

    // Escribe contenido en un archivo (crea o sobrescribe).
    // Recibe 2 argumentos: Objeto::Cadena(ruta), Objeto::Cadena(contenido).
    fn fs_escribir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (ruta, contenido)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Cadena(ruta), Objeto::Cadena(contenido)) => {
                match std::fs::write(ruta, contenido) {
                    Ok(_) => Objeto::Booleano(true),
                    Err(e) => Objeto::Error(format!(
                        "Error al escribir el archivo {}: {}", ruta, e
                    )),
                }
            }
            _ => Objeto::Error(
                "Ambos argumentos deben ser cadenas (ruta, contenido)"
                    .to_string(),
            ),
        }
    }

    let mut mapa_fs: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa_fs.insert(
        LlaveHash::Cadena("leer".to_string()),
        Objeto::Nativa(fs_leer as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_fs.insert(
        LlaveHash::Cadena("escribir".to_string()),
        Objeto::Nativa(fs_escribir as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa_fs)
}
