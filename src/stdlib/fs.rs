use std::collections::HashMap;
use std::path::Path;

use crate::evaluator::{LlaveHash, Objeto};

fn ruta_es_segura(ruta: &str) -> bool {
    if ruta.contains("..") || ruta.contains('~') {
        return false;
    }
    if Path::new(ruta).is_absolute() {
        return false;
    }
    if ruta.starts_with('/') {
        return false;
    }
    true
}

fn validar_ruta(args: &[Objeto], idx: usize) -> Result<String, Objeto> {
    match args.get(idx) {
        Some(Objeto::Cadena(r)) => {
            if !ruta_es_segura(r) {
                Err(Objeto::Error(
                    "Error de seguridad: Acceso denegado. \
                     La ruta intenta escapar del directorio \
                     de trabajo o es absoluta.".to_string(),
                ))
            } else {
                Ok(r.clone())
            }
        }
        Some(other) => Err(Objeto::Error(format!(
            "fs: se esperaba una cadena (ruta), se recibió {}",
            match other {
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
                Objeto::Error(_) => "error",
                Objeto::Excepcion(_) => "excepcion",
            }
        ))),
        None => Err(Objeto::Error(format!(
            "fs: falta el argumento ruta en posición {}", idx
        ))),
    }
}

pub fn crear_modulo() -> Objeto {
    fn fs_leer(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(e) => return e,
        };
        match std::fs::read_to_string(&ruta) {
            Ok(contenido) => Objeto::Cadena(contenido),
            Err(e) => Objeto::Error(format!(
                "Error al leer el archivo {}: {}", ruta, e
            )),
        }
    }

    fn fs_escribir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (ruta, contenido)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(e) => return e,
        };
        let contenido = match &args[1] {
            Objeto::Cadena(c) => c.clone(),
            _ => return Objeto::Error(
                "El segundo argumento debe ser una cadena (contenido)".to_string(),
            ),
        };
        match std::fs::write(&ruta, &contenido) {
            Ok(_) => Objeto::Booleano(true),
            Err(e) => Objeto::Error(format!(
                "Error al escribir el archivo {}: {}", ruta, e
            )),
        }
    }

    fn fs_existe(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(_) => return Objeto::Booleano(false),
        };
        Objeto::Booleano(Path::new(&ruta).exists())
    }

    fn fs_es_directorio(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(_) => return Objeto::Booleano(false),
        };
        Objeto::Booleano(Path::new(&ruta).is_dir())
    }

    fn fs_es_archivo(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(_) => return Objeto::Booleano(false),
        };
        Objeto::Booleano(Path::new(&ruta).is_file())
    }

    fn fs_eliminar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(e) => return e,
        };
        match std::fs::remove_file(&ruta) {
            Ok(_) => Objeto::Booleano(true),
            Err(e) => Objeto::Error(format!(
                "Error al eliminar {}: {}", ruta, e
            )),
        }
    }

    fn fs_crear_directorio(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(e) => return e,
        };
        match std::fs::create_dir_all(&ruta) {
            Ok(_) => Objeto::Booleano(true),
            Err(e) => Objeto::Error(format!(
                "Error al crear directorio {}: {}", ruta, e
            )),
        }
    }

    fn fs_listar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (ruta)".to_string(),
            );
        }
        let ruta = match validar_ruta(&args, 0) {
            Ok(r) => r,
            Err(e) => return e,
        };
        match std::fs::read_dir(&ruta) {
            Ok(entries) => {
                let mut lista = Vec::new();
                for entry in entries {
                    match entry {
                        Ok(e) => lista.push(Objeto::Cadena(
                            e.file_name().to_string_lossy().to_string()
                        )),
                        Err(_) => {}
                    }
                }
                Objeto::Arreglo(lista)
            }
            Err(e) => Objeto::Error(format!(
                "Error al listar {}: {}", ruta, e
            )),
        }
    }

    let mut mapa_fs: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa_fs.insert(LlaveHash::Cadena("leer".to_string()), Objeto::Nativa(fs_leer as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("escribir".to_string()), Objeto::Nativa(fs_escribir as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("existe".to_string()), Objeto::Nativa(fs_existe as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("es_directorio".to_string()), Objeto::Nativa(fs_es_directorio as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("es_archivo".to_string()), Objeto::Nativa(fs_es_archivo as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("eliminar".to_string()), Objeto::Nativa(fs_eliminar as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("crear_directorio".to_string()), Objeto::Nativa(fs_crear_directorio as fn(Vec<Objeto>) -> Objeto));
    mapa_fs.insert(LlaveHash::Cadena("listar".to_string()), Objeto::Nativa(fs_listar as fn(Vec<Objeto>) -> Objeto));

    Objeto::Diccionario(mapa_fs)
}
