use std::collections::HashMap;

use crate::evaluator::{evaluar_llamada_funcion, LlaveHash, Objeto};

pub fn crear_modulo() -> Objeto {
    fn arr_len(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.len: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(a) => Objeto::Entero(a.len() as i64),
            _ => Objeto::Error(
                "arr.len: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_push(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "arr.push: se esperaban 2 argumentos (arreglo, elemento)".to_string(),
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
                "arr.push: el primer argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_pop(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.pop: se esperaba 1 argumento (arreglo)".to_string(),
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
                "arr.pop: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_contiene(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "arr.contiene: se esperaban 2 argumentos (arreglo, elemento)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Arreglo(v), elemento) => {
                Objeto::Booleano(v.iter().any(|x| x.son_iguales(elemento)))
            }
            _ => Objeto::Error(
                "arr.contiene: el primer argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_invertir(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.invertir: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => {
                let mut rev = v.clone();
                rev.reverse();
                Objeto::Arreglo(rev)
            }
            _ => Objeto::Error(
                "arr.invertir: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_primero(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.primero: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => v.first().cloned().unwrap_or(Objeto::Nulo),
            _ => Objeto::Error(
                "arr.primero: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_ultimo(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.ultimo: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => v.last().cloned().unwrap_or(Objeto::Nulo),
            _ => Objeto::Error(
                "arr.ultimo: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_concatenar(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "arr.concatenar: se esperaban 2 argumentos (arreglo1, arreglo2)".to_string(),
            );
        }
        match (&args[0], &args[1]) {
            (Objeto::Arreglo(a), Objeto::Arreglo(b)) => {
                let mut ambos = a.clone();
                ambos.extend(b.clone());
                Objeto::Arreglo(ambos)
            }
            _ => Objeto::Error(
                "arr.concatenar: ambos argumentos deben ser arreglos".to_string(),
            ),
        }
    }

    fn arr_vacio(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.vacio: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => Objeto::Booleano(v.is_empty()),
            _ => Objeto::Error(
                "arr.vacio: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn arr_indice_de(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "arr.indice_de: se esperaban 2 argumentos (arreglo, elemento)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => {
                let idx = v.iter().position(|x| x.son_iguales(&args[1]));
                match idx {
                    Some(i) => Objeto::Entero(i as i64),
                    None => Objeto::Entero(-1),
                }
            }
            _ => Objeto::Error(
                "arr.indice_de: el primer argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    fn llamar_con(closure: &Objeto, arg: Objeto) -> Objeto {
        match closure {
            Objeto::Funcion { .. } | Objeto::Nativa(_) => {
                evaluar_llamada_funcion(closure.clone(), vec![arg])
            }
            _ => Objeto::Error("Se esperaba una función como argumento".to_string()),
        }
    }

    fn arr_map(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error("arr.map: se esperaban 2 argumentos (arreglo, función)".to_string());
        }
        let (arr, closure) = (&args[0], &args[1]);
        match arr {
            Objeto::Arreglo(v) => {
                let mut resultado = Vec::with_capacity(v.len());
                for elem in v {
                    let res = llamar_con(closure, elem.clone());
                    if let Objeto::Error(_) = &res { return res }
                    resultado.push(res);
                }
                Objeto::Arreglo(resultado)
            }
            _ => Objeto::Error("arr.map: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn arr_filter(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error("arr.filter: se esperaban 2 argumentos (arreglo, función)".to_string());
        }
        let (arr, closure) = (&args[0], &args[1]);
        match arr {
            Objeto::Arreglo(v) => {
                let mut resultado = Vec::new();
                for elem in v {
                    let res = llamar_con(closure, elem.clone());
                    if let Objeto::Error(_) = &res { return res }
                    if es_truthy(&res) {
                        resultado.push(elem.clone());
                    }
                }
                Objeto::Arreglo(resultado)
            }
            _ => Objeto::Error("arr.filter: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn arr_reduce(args: Vec<Objeto>) -> Objeto {
        if args.len() < 2 || args.len() > 3 {
            return Objeto::Error("arr.reduce: se esperaban 2-3 argumentos (arreglo, función, inicial?)".to_string());
        }
        let mut iter = args.into_iter();
        let arreglo = iter.next().unwrap();
        let closure = iter.next().unwrap();
        let inicial = iter.next();
        match arreglo {
            Objeto::Arreglo(v) => {
                let mut acc = inicial.unwrap_or(Objeto::Nulo);
                for elem in &v {
                    let res = match &closure {
                        Objeto::Funcion { .. } | Objeto::Nativa(_) => {
                            evaluar_llamada_funcion(closure.clone(), vec![acc, elem.clone()])
                        }
                        _ => return Objeto::Error("arr.reduce: se esperaba una función".to_string()),
                    };
                    if let Objeto::Error(_) = &res { return res }
                    acc = res;
                }
                acc
            }
            _ => Objeto::Error("arr.reduce: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn arr_find(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error("arr.find: se esperaban 2 argumentos (arreglo, función)".to_string());
        }
        let (arr, closure) = (&args[0], &args[1]);
        match arr {
            Objeto::Arreglo(v) => {
                for elem in v {
                    let res = llamar_con(closure, elem.clone());
                    if let Objeto::Error(_) = &res { return res }
                    if es_truthy(&res) {
                        return elem.clone();
                    }
                }
                Objeto::Nulo
            }
            _ => Objeto::Error("arr.find: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn arr_every(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error("arr.every: se esperaban 2 argumentos (arreglo, función)".to_string());
        }
        let (arr, closure) = (&args[0], &args[1]);
        match arr {
            Objeto::Arreglo(v) => {
                for elem in v {
                    let res = llamar_con(closure, elem.clone());
                    if let Objeto::Error(_) = &res { return res }
                    if !es_truthy(&res) {
                        return Objeto::Booleano(false);
                    }
                }
                Objeto::Booleano(true)
            }
            _ => Objeto::Error("arr.every: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn arr_some(args: Vec<Objeto>) -> Objeto {
        if args.len() != 2 {
            return Objeto::Error("arr.some: se esperaban 2 argumentos (arreglo, función)".to_string());
        }
        let (arr, closure) = (&args[0], &args[1]);
        match arr {
            Objeto::Arreglo(v) => {
                for elem in v {
                    let res = llamar_con(closure, elem.clone());
                    if let Objeto::Error(_) = &res { return res }
                    if es_truthy(&res) {
                        return Objeto::Booleano(true);
                    }
                }
                Objeto::Booleano(false)
            }
            _ => Objeto::Error("arr.some: el primer argumento debe ser un arreglo".to_string()),
        }
    }

    fn es_truthy(obj: &Objeto) -> bool {
        match obj {
            Objeto::Booleano(true) => true,
            Objeto::Entero(n) => *n != 0,
            _ => false,
        }
    }

    fn arr_plano(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "arr.plano: se esperaba 1 argumento (arreglo)".to_string(),
            );
        }
        match &args[0] {
            Objeto::Arreglo(v) => {
                let mut resultado = Vec::new();
                for elemento in v {
                    match elemento {
                        Objeto::Arreglo(anidado) => {
                            resultado.extend(anidado.clone());
                        }
                        other => resultado.push(other.clone()),
                    }
                }
                Objeto::Arreglo(resultado)
            }
            _ => Objeto::Error(
                "arr.plano: el argumento debe ser un arreglo".to_string(),
            ),
        }
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(LlaveHash::Cadena("len".to_string()), Objeto::Nativa(arr_len as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("push".to_string()), Objeto::Nativa(arr_push as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("pop".to_string()), Objeto::Nativa(arr_pop as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("contiene".to_string()), Objeto::Nativa(arr_contiene as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("invertir".to_string()), Objeto::Nativa(arr_invertir as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("primero".to_string()), Objeto::Nativa(arr_primero as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("ultimo".to_string()), Objeto::Nativa(arr_ultimo as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("concatenar".to_string()), Objeto::Nativa(arr_concatenar as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("vacio".to_string()), Objeto::Nativa(arr_vacio as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("indice_de".to_string()), Objeto::Nativa(arr_indice_de as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("plano".to_string()), Objeto::Nativa(arr_plano as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("map".to_string()), Objeto::Nativa(arr_map as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("filter".to_string()), Objeto::Nativa(arr_filter as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("reduce".to_string()), Objeto::Nativa(arr_reduce as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("find".to_string()), Objeto::Nativa(arr_find as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("every".to_string()), Objeto::Nativa(arr_every as fn(Vec<Objeto>) -> Objeto));
    mapa.insert(LlaveHash::Cadena("some".to_string()), Objeto::Nativa(arr_some as fn(Vec<Objeto>) -> Objeto));

    Objeto::Diccionario(mapa)
}
