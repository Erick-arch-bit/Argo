//
// Módulo estándar os — Interacción con el sistema anfitrión.
// Expone cmd (ejecutar comando shell), env (leer variable de
// entorno) y exit (terminar el proceso) en un Objeto::Diccionario
// bajo el nombre "os".
//

use std::collections::HashMap;
use std::env;
use std::process::{exit, Command};

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones
/// del sistema operativo.
pub fn crear_modulo() -> Objeto {
    // Ejecuta un comando en el shell del sistema y retorna su stdout.
    // Recibe 1 argumento: Objeto::Cadena(comando).
    fn os_cmd(args: Vec<Objeto>) -> Objeto {
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

    // Lee una variable de entorno. Recibe 1 argumento:
    // Objeto::Cadena(llave). Retorna Cadena(valor) o Nulo.
    fn os_env(args: Vec<Objeto>) -> Objeto {
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

    // Termina el proceso con un código de salida.
    // Recibe 1 argumento: Objeto::Entero(codigo).
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

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(
        LlaveHash::Cadena("cmd".to_string()),
        Objeto::Nativa(os_cmd as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("env".to_string()),
        Objeto::Nativa(os_env as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("exit".to_string()),
        Objeto::Nativa(os_exit as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa)
}
