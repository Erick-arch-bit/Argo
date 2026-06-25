use std::collections::HashMap;
use std::f64::consts;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::evaluator::{LlaveHash, Objeto};

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

fn extraer_dos_f64(args: Vec<Objeto>) -> Result<(f64, f64), Objeto> {
    if args.len() != 2 {
        return Err(Objeto::Error(
            "Se esperaban 2 argumentos numéricos".to_string(),
        ));
    }
    let a = match &args[0] {
        Objeto::Entero(v) => *v as f64,
        Objeto::Flotante(v) => *v,
        _ => return Err(Objeto::Error(
            "Ambos argumentos deben ser números".to_string(),
        )),
    };
    let b = match &args[1] {
        Objeto::Entero(v) => *v as f64,
        Objeto::Flotante(v) => *v,
        _ => return Err(Objeto::Error(
            "Ambos argumentos deben ser números".to_string(),
        )),
    };
    Ok((a, b))
}

pub fn crear_modulo() -> Objeto {
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

    let math_tan = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.tan()),
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

    let math_pow = |args: Vec<Objeto>| -> Objeto {
        match extraer_dos_f64(args) {
            Ok((a, b)) => Objeto::Flotante(a.powf(b)),
            Err(e) => e,
        }
    };

    let math_log = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.ln()),
            Err(e) => e,
        }
    };

    let math_log10 = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.log10()),
            Err(e) => e,
        }
    };

    let math_floor = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.floor()),
            Err(e) => e,
        }
    };

    let math_ceil = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.ceil()),
            Err(e) => e,
        }
    };

    let math_round = |args: Vec<Objeto>| -> Objeto {
        match extraer_f64(args) {
            Ok(v) => Objeto::Flotante(v.round()),
            Err(e) => e,
        }
    };

    let math_max = |args: Vec<Objeto>| -> Objeto {
        match extraer_dos_f64(args) {
            Ok((a, b)) => Objeto::Flotante(a.max(b)),
            Err(e) => e,
        }
    };

    let math_min = |args: Vec<Objeto>| -> Objeto {
        match extraer_dos_f64(args) {
            Ok((a, b)) => Objeto::Flotante(a.min(b)),
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

    let mut mapa_math: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa_math.insert(LlaveHash::Cadena("sin".to_string()), Objeto::Nativa(math_sin as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("cos".to_string()), Objeto::Nativa(math_cos as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("tan".to_string()), Objeto::Nativa(math_tan as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("sqrt".to_string()), Objeto::Nativa(math_sqrt as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("abs".to_string()), Objeto::Nativa(math_abs as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("pow".to_string()), Objeto::Nativa(math_pow as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("log".to_string()), Objeto::Nativa(math_log as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("log10".to_string()), Objeto::Nativa(math_log10 as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("floor".to_string()), Objeto::Nativa(math_floor as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("ceil".to_string()), Objeto::Nativa(math_ceil as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("round".to_string()), Objeto::Nativa(math_round as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("max".to_string()), Objeto::Nativa(math_max as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("min".to_string()), Objeto::Nativa(math_min as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("random".to_string()), Objeto::Nativa(math_random as fn(Vec<Objeto>) -> Objeto));
    mapa_math.insert(LlaveHash::Cadena("PI".to_string()), Objeto::Flotante(consts::PI));
    mapa_math.insert(LlaveHash::Cadena("E".to_string()), Objeto::Flotante(consts::E));

    Objeto::Diccionario(mapa_math)
}
