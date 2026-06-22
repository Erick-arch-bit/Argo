//
// Módulo estándar math — Funciones y constantes matemáticas.
// Encapsula operaciones trigonométricas, raíz cuadrada, valor absoluto
// y constantes (PI, E) en un Objeto::Diccionario para inyectar en el
// entorno global bajo el nombre "math".
//

use std::collections::HashMap;
use std::f64::consts;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones y
/// constantes matemáticas. El llamante (inyectar_stdlib) lo asigna
/// al entorno global como "math".
pub fn crear_modulo() -> Objeto {
    // Helper interno: extrae un f64 de un Objeto numérico.
    // Retorna Err(Objeto::Error) si la aridad es incorrecta o el
    // tipo no es numérico, lo que permite propagar el error con ?.
    fn extraer_f64(args: Vec<Objeto>) -> Result<f64, Objeto> {
        if args.len() != 1 {
            return Err(Objeto::Error(
                "Se esperaba 1 argumento numérico".to_string(),
            ));
        }
        match &args[0] {
            Objeto::Entero(v) => Ok(*v as f64),
            Objeto::Flotante(v) => Ok(*v),
            _ => Err(Objeto::Error(
                "Se esperaba un número (entero o flotante)".to_string(),
            )),
        }
    }

    // Closures matemáticos. Cada uno delega el parseo en
    // extraer_f64 y aplica la función de std::f64.
    let math_sin = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.sin()),
            Err(e) => e,
        }
    };

    let math_cos = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.cos()),
            Err(e) => e,
        }
    };

    let math_sqrt = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.sqrt()),
            Err(e) => e,
        }
    };

    let math_abs = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.abs()),
            Err(e) => e,
        }
    };

    let math_random = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (min, max)".to_string(),
            );
        }
        let min = match &args[0] {
            Objeto::Entero(v) => *v,
            _ => {
                return Objeto::Error(
                    "El primer argumento debe ser un entero".to_string(),
                );
            }
        };
        let max = match &args[1] {
            Objeto::Entero(v) => *v,
            _ => {
                return Objeto::Error(
                    "El segundo argumento debe ser un entero".to_string(),
                );
            }
        };
        if min > max {
            return Objeto::Error(
                "min debe ser menor o igual que max".to_string(),
            );
        }
        let semilla = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64;
        let pseudo = (semilla.wrapping_mul(6364136223846793005).wrapping_add(1))
            >> 33;
        let resultado = min + (pseudo as i64 % (max - min + 1));
        Objeto::Entero(resultado)
    };

    // Construir el HashMap del diccionario math.
    let mut mapa_math: HashMap<LlaveHash, Objeto> = HashMap::new();

    // Funciones
    mapa_math.insert(
        LlaveHash::Cadena("sin".to_string()),
        Objeto::Nativa(math_sin as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_math.insert(
        LlaveHash::Cadena("cos".to_string()),
        Objeto::Nativa(math_cos as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_math.insert(
        LlaveHash::Cadena("sqrt".to_string()),
        Objeto::Nativa(math_sqrt as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_math.insert(
        LlaveHash::Cadena("abs".to_string()),
        Objeto::Nativa(math_abs as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_math.insert(
        LlaveHash::Cadena("random".to_string()),
        Objeto::Nativa(math_random as fn(Vec<Objeto>) -> Objeto),
    );

    // Constantes
    mapa_math.insert(
        LlaveHash::Cadena("PI".to_string()),
        Objeto::Flotante(consts::PI),
    );
    mapa_math.insert(
        LlaveHash::Cadena("E".to_string()),
        Objeto::Flotante(consts::E),
    );

    Objeto::Diccionario(mapa_math)
}
