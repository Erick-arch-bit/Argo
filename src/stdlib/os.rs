use std::collections::HashMap;
use std::env;
use std::process::{exit, Command};

use crate::evaluator::{LlaveHash, Objeto};

pub fn crear_modulo() -> Objeto {
    fn os_ejecutar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (comando)".to_string(),
            );
        }
        let comando = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => {
                return Objeto::Error(
                    "El argumento debe ser una cadena (comando)".to_string(),
                );
            }
        };

        match Command::new("sh").arg("-c").arg(&comando).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout)
                    .to_string();
                Objeto::Cadena(stdout)
            }
            Err(e) => Objeto::Error(format!(
                "Error al ejecutar el comando '{}': {}", comando, e
            )),
        }
    }

    fn os_variables(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (llave)".to_string(),
            );
        }
        let llave = match &args[0] {
            Objeto::Cadena(k) => k.clone(),
            _ => {
                return Objeto::Error(
                    "El argumento debe ser una cadena (llave)".to_string(),
                );
            }
        };

        match env::var(&llave) {
            Ok(valor) => Objeto::Cadena(valor),
            Err(_) => Objeto::Nulo,
        }
    }

    fn os_exit(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (código de salida)".to_string(),
            );
        }
        let codigo = match &args[0] {
            Objeto::Entero(c) => *c as i32,
            _ => {
                return Objeto::Error(
                    "El argumento debe ser un entero (código de salida)"
                        .to_string(),
                );
            }
        };
        exit(codigo);
    }

    fn os_directorio_actual(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "os.directorio_actual no recibe argumentos".to_string(),
            );
        }
        match env::current_dir() {
            Ok(path) => Objeto::Cadena(path.to_string_lossy().to_string()),
            Err(e) => Objeto::Error(format!("Error al obtener directorio actual: {}", e)),
        }
    }

    fn os_directorio_temporal(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "os.directorio_temporal no recibe argumentos".to_string(),
            );
        }
        Objeto::Cadena(env::temp_dir().to_string_lossy().to_string())
    }

    fn os_argumentos(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "os.argumentos no recibe argumentos".to_string(),
            );
        }
        let args: Vec<Objeto> = env::args()
            .map(Objeto::Cadena)
            .collect();
        Objeto::Arreglo(args)
    }

    fn os_procesadores(args: Vec<Objeto>) -> Objeto {
        if !args.is_empty() {
            return Objeto::Error(
                "os.procesadores no recibe argumentos".to_string(),
            );
        }
        let count = std::thread::available_parallelism()
            .map(|n| n.get() as i64)
            .unwrap_or(1);
        Objeto::Entero(count)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(LlaveHash::Cadena("ejecutar".to_string()), Objeto::Nativa(os_ejecutar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("variables".to_string()), Objeto::Nativa(os_variables as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("exit".to_string()), Objeto::Nativa(os_exit as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("directorio_actual".to_string()), Objeto::Nativa(os_directorio_actual as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("directorio_temporal".to_string()), Objeto::Nativa(os_directorio_temporal as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("argumentos".to_string()), Objeto::Nativa(os_argumentos as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("procesadores".to_string()), Objeto::Nativa(os_procesadores as fn(Vec<Objeto>) -> Objeto));

    Objeto::Diccionario(mapa)
}
