//
// Módulo estándar buffer — Manipulación de memoria binaria contigua.
// Expone alloc, write y read en un Objeto::Diccionario bajo "buffer".
//

use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

const MAX_BUFFER_SIZE: usize = 1 << 30; // 1 GB

fn tipo_objeto(o: &Objeto) -> &'static str {
    match o {
        Objeto::Entero(_) => "entero",
        Objeto::Flotante(_) => "flotante",
        Objeto::Booleano(_) => "booleano",
        Objeto::Cadena(_) => "cadena",
        Objeto::Nulo => "nulo",
        Objeto::Buffer(_) => "buffer",
        Objeto::Arreglo(_) => "arreglo",
        Objeto::Diccionario(_) => "diccionario",
        Objeto::Funcion { .. } | Objeto::Nativa(_) => "función",
        Objeto::Retorno(_) => "retorno",
        Objeto::Break => "break",
        Objeto::Error(_) => "error",
    }
}

fn extraer_entero(args: &[Objeto], idx: usize, nombre: &str) -> Result<i64, Objeto> {
    match args.get(idx) {
        Some(Objeto::Entero(v)) => Ok(*v),
        Some(other) => Err(Objeto::Error(format!(
            "buffer: '{}' debe ser un entero, se recibió {}",
            nombre,
            tipo_objeto(other)
        ))),
        None => Err(Objeto::Error(format!(
            "buffer: falta el argumento '{}' (índice {})",
            nombre, idx
        ))),
    }
}

fn extraer_buffer(args: &[Objeto], idx: usize) -> Result<&Vec<u8>, Objeto> {
    match args.get(idx) {
        Some(Objeto::Buffer(v)) => Ok(v),
        Some(other) => Err(Objeto::Error(format!(
            "buffer: se esperaba un buffer, se recibió {}",
            tipo_objeto(other)
        ))),
        None => Err(Objeto::Error(format!(
            "buffer: falta el argumento buffer (índice {})",
            idx
        ))),
    }
}

pub fn crear_modulo() -> Objeto {
    // Asigna un buffer de `tamano` bytes inicializado en cero.
    // Recibe 1 argumento: Objeto::Entero(tamano).
    fn buffer_alloc(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "buffer.alloc: se esperaba 1 argumento (tamano), se recibieron {}",
                args.len()
            ));
        }
        let tamano = match extraer_entero(&args, 0, "tamano") {
            Ok(v) => v,
            Err(e) => return e,
        };
        if tamano < 0 {
            return Objeto::Error(format!(
                "buffer.alloc: el tamaño no puede ser negativo, se recibió {}",
                tamano
            ));
        }
        let tamano_usize = tamano as usize;
        if tamano_usize > MAX_BUFFER_SIZE {
            return Objeto::Error(format!(
                "buffer.alloc: tamaño {} bytes excede el máximo de {} bytes",
                tamano_usize, MAX_BUFFER_SIZE
            ));
        }
        Objeto::Buffer(vec![0; tamano_usize])
    }

    // Escribe un byte en una posición del buffer.
    // Retorna un NUEVO buffer (clon + mutación) para mantener
    // la seguridad funcional (inmutabilidad de datos en Argo).
    // Recibe 3 argumentos: Objeto::Buffer, Objeto::Entero(indice),
    // Objeto::Entero(valor_byte).
    fn buffer_write(args: Vec<Objeto>) -> Objeto {
        if args.len() != 3 {
            return Objeto::Error(format!(
                "buffer.write: se esperaban 3 argumentos (buffer, indice, valor), se recibieron {}",
                args.len()
            ));
        }
        let bytes = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let indice = match extraer_entero(&args, 1, "indice") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let valor = match extraer_entero(&args, 2, "valor") {
            Ok(v) => v,
            Err(e) => return e,
        };

        let len = bytes.len();
        if indice < 0 || (indice as usize) >= len {
            return Objeto::Error(format!(
                "buffer.write: índice {} fuera de rango (longitud del buffer: {})",
                indice, len
            ));
        }
        if valor < 0 || valor > 255 {
            return Objeto::Error(format!(
                "buffer.write: el valor debe estar entre 0 y 255, se recibió {}",
                valor
            ));
        }
        let mut nuevo = bytes.clone();
        nuevo[indice as usize] = valor as u8;
        Objeto::Buffer(nuevo)
    }

    // Lee un byte en una posición del buffer.
    // Recibe 2 argumentos: Objeto::Buffer, Objeto::Entero(indice).
    fn buffer_read(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(format!(
                "buffer.read: se esperaban 2 argumentos (buffer, indice), se recibieron {}",
                args.len()
            ));
        }
        let bytes = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let indice = match extraer_entero(&args, 1, "indice") {
            Ok(v) => v,
            Err(e) => return e,
        };

        let len = bytes.len();
        if indice < 0 || (indice as usize) >= len {
            return Objeto::Error(format!(
                "buffer.read: índice {} fuera de rango (longitud del buffer: {})",
                indice, len
            ));
        }
        Objeto::Entero(bytes[indice as usize] as i64)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(
        LlaveHash::Cadena("alloc".to_string()),
        Objeto::Nativa(buffer_alloc as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("write".to_string()),
        Objeto::Nativa(buffer_write as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("read".to_string()),
        Objeto::Nativa(buffer_read as fn(Vec<Objeto>) -> Objeto),
    );
    Objeto::Diccionario(mapa)
}
