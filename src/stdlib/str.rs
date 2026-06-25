use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

pub fn crear_modulo() -> Objeto {
    fn str_longitud(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => Objeto::Entero(c.len() as i64),
            _ => Objeto::Error(
                "El argumento debe ser una cadena".to_string(),
            ),
        }
    }

    fn str_mayusculas(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => Objeto::Cadena(c.to_uppercase()),
            _ => Objeto::Error(
                "El argumento debe ser una cadena".to_string(),
            ),
        }
    }

    fn str_minusculas(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => Objeto::Cadena(c.to_lowercase()),
            _ => Objeto::Error(
                "El argumento debe ser una cadena".to_string(),
            ),
        }
    }

    fn str_recortar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => Objeto::Cadena(c.trim().to_string()),
            _ => Objeto::Error(
                "El argumento debe ser una cadena".to_string(),
            ),
        }
    }

    fn str_dividir(args: Vec<Objeto>) -> Objeto {
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

    fn str_contiene(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, subcadena)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Cadena(c), Objeto::Cadena(sub)) => {
                Objeto::Booleano(c.contains(sub.as_str()))
            }
            _ => Objeto::Error(
                "Ambos argumentos deben ser cadenas".to_string(),
            ),
        }
    }

    fn str_subcadena(args: Vec<Objeto>) -> Objeto {
        if args.len() < 2 || args.len() > 3 {
            return Objeto::Error(
                "Se esperaban 2 o 3 argumentos (cadena, inicio, fin?)".to_string(),
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
        let inicio = match &args[1] {
            Objeto::Entero(i) => {
                if *i < 0 {
                    return Objeto::Error(
                        "El índice de inicio no puede ser negativo".to_string(),
                    );
                }
                *i as usize
            }
            _ => {
                return Objeto::Error(
                    "El segundo argumento debe ser un entero (inicio)".to_string(),
                );
            }
        };
        let fin = if args.len() == 3 {
            match &args[2] {
                Objeto::Entero(i) => {
                    if *i < 0 {
                        return Objeto::Error(
                            "El índice de fin no puede ser negativo".to_string(),
                        );
                    }
                    Some(*i as usize)
                }
                _ => {
                    return Objeto::Error(
                        "El tercer argumento debe ser un entero (fin)".to_string(),
                    );
                }
            }
        } else {
            None
        };
        if inicio > cadena.len() {
            return Objeto::Cadena(String::new());
        }
        match fin {
            Some(f) if f <= cadena.len() && f >= inicio => {
                Objeto::Cadena(cadena[inicio..f].to_string())
            }
            Some(_) => Objeto::Cadena(String::new()),
            None => Objeto::Cadena(cadena[inicio..].to_string()),
        }
    }

    fn str_reemplazar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 3 {
            return Objeto::Error(
                "Se esperaban 3 argumentos (cadena, objetivo, reemplazo)".to_string(),
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

    fn str_empieza_con(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, prefijo)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Cadena(c), Objeto::Cadena(prefijo)) => {
                Objeto::Booleano(c.starts_with(prefijo.as_str()))
            }
            _ => Objeto::Error(
                "Ambos argumentos deben ser cadenas".to_string(),
            ),
        }
    }

    fn str_termina_con(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, sufijo)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Cadena(c), Objeto::Cadena(sufijo)) => {
                Objeto::Booleano(c.ends_with(sufijo.as_str()))
            }
            _ => Objeto::Error(
                "Ambos argumentos deben ser cadenas".to_string(),
            ),
        }
    }

    fn str_indice_de(args: Vec<Objeto>) -> Objeto {
        if args.len() < 2 || args.len() > 3 {
            return Objeto::Error(
                "Se esperaban 2 o 3 argumentos (cadena, subcadena, inicio?)".to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => return Objeto::Error("El primer argumento debe ser una cadena".to_string()),
        };
        let subcadena = match &args[1] {
            Objeto::Cadena(s) => s.clone(),
            _ => return Objeto::Error("La subcadena debe ser una cadena".to_string()),
        };
        let inicio: usize = if args.len() == 3 {
            match &args[2] {
                Objeto::Entero(i) => {
                    if *i < 0 {
                        return Objeto::Error("El inicio no puede ser negativo".to_string());
                    }
                    *i as usize
                }
                _ => return Objeto::Error("El inicio debe ser un entero".to_string()),
            }
        } else {
            0
        };
        if inicio > cadena.len() {
            return Objeto::Entero(-1);
        }
        match cadena[inicio..].find(&subcadena) {
            Some(i) => Objeto::Entero((inicio + i) as i64),
            None => Objeto::Entero(-1),
        }
    }

    fn str_invertir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => {
                Objeto::Cadena(c.chars().rev().collect::<String>())
            }
            _ => Objeto::Error("El argumento debe ser una cadena".to_string()),
        }
    }

    fn str_repetir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, veces)".to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => return Objeto::Error("El primer argumento debe ser una cadena".to_string()),
        };
        let veces = match &args[1] {
            Objeto::Entero(n) => {
                if *n < 0 {
                    return Objeto::Error("Las veces no puede ser negativo".to_string());
                }
                *n as usize
            }
            _ => return Objeto::Error("El segundo argumento debe ser un entero".to_string()),
        };
        Objeto::Cadena(cadena.repeat(veces))
    }

    fn str_a_arreglo(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (cadena)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => {
                let chars: Vec<Objeto> = c.chars()
                    .map(|ch| Objeto::Cadena(ch.to_string()))
                    .collect();
                Objeto::Arreglo(chars)
            }
            _ => Objeto::Error("El argumento debe ser una cadena".to_string()),
        }
    }

    fn str_codigo_en(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Se esperaban 2 argumentos (cadena, indice)".to_string(),
            );
        }
        let cadena = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            _ => return Objeto::Error("El primer argumento debe ser una cadena".to_string()),
        };
        let indice = match &args[1] {
            Objeto::Entero(i) => {
                if *i < 0 { return Objeto::Error("El índice no puede ser negativo".to_string()); }
                *i as usize
            }
            _ => return Objeto::Error("El segundo argumento debe ser un entero".to_string()),
        };
        match cadena.chars().nth(indice) {
            Some(ch) => Objeto::Entero(ch as i64),
            None => Objeto::Error("Índice fuera de rango".to_string()),
        }
    }

    fn str_de_codigo(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento (código)".to_string(),
            );
        }
        let codigo = match &args[0] {
            Objeto::Entero(c) => *c as u32,
            _ => return Objeto::Error("El argumento debe ser un entero (código Unicode)".to_string()),
        };
        match char::from_u32(codigo) {
            Some(ch) => Objeto::Cadena(ch.to_string()),
            None => Objeto::Error(format!("Código Unicode inválido: {}", codigo)),
        }
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(LlaveHash::Cadena("longitud".to_string()), Objeto::Nativa(str_longitud as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("mayusculas".to_string()), Objeto::Nativa(str_mayusculas as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("minusculas".to_string()), Objeto::Nativa(str_minusculas as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("recortar".to_string()), Objeto::Nativa(str_recortar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("dividir".to_string()), Objeto::Nativa(str_dividir as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("contiene".to_string()), Objeto::Nativa(str_contiene as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("subcadena".to_string()), Objeto::Nativa(str_subcadena as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("reemplazar".to_string()), Objeto::Nativa(str_reemplazar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("empieza_con".to_string()), Objeto::Nativa(str_empieza_con as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("termina_con".to_string()), Objeto::Nativa(str_termina_con as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("indice_de".to_string()), Objeto::Nativa(str_indice_de as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("invertir".to_string()), Objeto::Nativa(str_invertir as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("repetir".to_string()), Objeto::Nativa(str_repetir as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("a_arreglo".to_string()), Objeto::Nativa(str_a_arreglo as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("codigo_en".to_string()), Objeto::Nativa(str_codigo_en as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("de_codigo".to_string()), Objeto::Nativa(str_de_codigo as fn(Vec<Objeto>) -> Objeto));

    Objeto::Diccionario(mapa)
}
