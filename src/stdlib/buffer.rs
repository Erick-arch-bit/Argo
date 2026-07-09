use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

const MAX_BUFFER_SIZE: usize = 1 << 30;

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
        Objeto::StructDef(_) => "struct_def",
        Objeto::Instancia { .. } => "instancia",
        Objeto::Break => "break",
        Objeto::Continue => "continue",
        Objeto::Error(_, _) => "error",
        Objeto::Canal(_) => "canal",
        Objeto::Excepcion(_) => "excepcion",
    }
}

fn extraer_entero(args: &[Objeto], idx: usize, nombre: &str) -> Result<i64, Objeto> {
    match args.get(idx) {
        Some(Objeto::Entero(v)) => Ok(*v),
        Some(other) => Err(Objeto::Error(format!(
            "buffer: '{}' debe ser un entero, se recibió {}",
            nombre,
            tipo_objeto(other)
        ), Vec::new())),
        None => Err(Objeto::Error(format!(
            "buffer: falta el argumento '{}' (índice {})",
            nombre, idx
        ), Vec::new())),
    }
}

fn extraer_buffer(args: &[Objeto], idx: usize) -> Result<&Vec<u8>, Objeto> {
    match args.get(idx) {
        Some(Objeto::Buffer(v)) => Ok(v),
        Some(other) => Err(Objeto::Error(format!(
            "buffer: se esperaba un buffer, se recibió {}",
            tipo_objeto(other)
        ), Vec::new())),
        None => Err(Objeto::Error(format!(
            "buffer: falta el argumento buffer (índice {})",
            idx
        ), Vec::new())),
    }
}

pub fn crear_modulo() -> Objeto {
    fn buffer_alloc(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "buffer.alloc: se esperaba 1 argumento (tamano), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let tamano = match extraer_entero(&args, 0, "tamano") {
            Ok(v) => v,
            Err(e) => return e,
        };
        if tamano < 0 {
            return Objeto::Error(format!(
                "buffer.alloc: el tamaño no puede ser negativo, se recibió {}",
                tamano
            ), Vec::new());
        }
        let tamano_usize = tamano as usize;
        if tamano_usize > MAX_BUFFER_SIZE {
            return Objeto::Error(format!(
                "buffer.alloc: tamaño {} bytes excede el máximo de {} bytes",
                tamano_usize, MAX_BUFFER_SIZE
            ), Vec::new());
        }
        Objeto::Buffer(vec![0; tamano_usize])
    }

    fn buffer_write(args: Vec<Objeto>) -> Objeto {
        if args.len() != 3 {
            return Objeto::Error(format!(
                "buffer.write: se esperaban 3 argumentos (buffer, indice, valor), se recibieron {}",
                args.len()
            ), Vec::new());
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
            ), Vec::new());
        }
        if !(0..=255).contains(&valor) {
            return Objeto::Error(format!(
                "buffer.write: el valor debe estar entre 0 y 255, se recibió {}",
                valor
            ), Vec::new());
        }
        let mut nuevo = bytes.clone();
        nuevo[indice as usize] = valor as u8;
        Objeto::Buffer(nuevo)
    }

    fn buffer_read(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(format!(
                "buffer.read: se esperaban 2 argumentos (buffer, indice), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let bytes = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let indice = match extraer_entero(&args, 1, "indice") {
            Ok(v) => v,
            Err(e) => return e,
        };

        if indice < 0 || (indice as usize) >= bytes.len() {
            return Objeto::Error(format!(
                "buffer.read: índice {} fuera de rango (longitud del buffer: {})",
                indice, bytes.len()
            ), Vec::new());
        }
        Objeto::Entero(bytes[indice as usize] as i64)
    }

    fn buffer_longitud(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "buffer.longitud: se esperaba 1 argumento (buffer), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let bytes = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        Objeto::Entero(bytes.len() as i64)
    }

    fn buffer_a_cadena(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "buffer.a_cadena: se esperaba 1 argumento (buffer), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let bytes = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        Objeto::Cadena(String::from_utf8_lossy(bytes).to_string())
    }

    fn buffer_de_cadena(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "buffer.de_cadena: se esperaba 1 argumento (cadena), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c,
            other => return Objeto::Error(format!(
                "buffer.de_cadena: se esperaba una cadena, se recibió {}",
                tipo_objeto(other)
            ), Vec::new()),
        };
        Objeto::Buffer(cadena.as_bytes().to_vec())
    }

    fn buffer_copiar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 3 {
            return Objeto::Error(format!(
                "buffer.copiar: se esperaban 3 argumentos (origen, destino, posicion), se recibieron {}",
                args.len()
            ), Vec::new());
        }
        let origen = match extraer_buffer(&args, 0) {
            Ok(v) => v,
            Err(e) => return e,
        };
        let destino = match args.get(1) {
            Some(Objeto::Buffer(v)) => v.clone(),
            Some(other) => return Objeto::Error(format!(
                "buffer.copiar: el segundo argumento debe ser un buffer, se recibió {}",
                tipo_objeto(other)
            ), Vec::new()),
            None => return Objeto::Error("buffer.copiar: falta el argumento destino".to_string(), Vec::new()),
        };
        let posicion = match extraer_entero(&args, 2, "posicion") {
            Ok(v) => v,
            Err(e) => return e,
        };
        if posicion < 0 || (posicion as usize) > destino.len() {
            return Objeto::Error(format!(
                "buffer.copiar: posición {} fuera de rango (longitud del destino: {})",
                posicion, destino.len()
            ), Vec::new());
        }
        let espacio = destino.len() - posicion as usize;
        let a_copiar = origen.len().min(espacio);
        let mut nuevo = destino;
        nuevo[posicion as usize..posicion as usize + a_copiar]
            .copy_from_slice(&origen[..a_copiar]);
        Objeto::Buffer(nuevo)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(LlaveHash::Cadena("alloc".to_string()), Objeto::Nativa(buffer_alloc as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("write".to_string()), Objeto::Nativa(buffer_write as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("read".to_string()), Objeto::Nativa(buffer_read as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("longitud".to_string()), Objeto::Nativa(buffer_longitud as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("a_cadena".to_string()), Objeto::Nativa(buffer_a_cadena as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("de_cadena".to_string()), Objeto::Nativa(buffer_de_cadena as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("copiar".to_string()), Objeto::Nativa(buffer_copiar as fn(Vec<Objeto>) -> Objeto));
    Objeto::Diccionario(mapa)
}
