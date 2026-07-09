//
// Módulo estándar thread — Ejecución de scripts Argo en segundo plano.
// Expone spawn en un Objeto::Diccionario bajo "thread".
//

use std::collections::HashMap;
use std::panic;
use std::thread;

use crate::evaluator::{configurar_entorno_global, evaluar_programa, LlaveHash, Objeto};
use crate::lexer::Lexer;
use crate::parser::Parser;

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
        Objeto::EnumDef(_) => "enum_def",
        Objeto::EnumValor { .. } => "enum_valor",
        Objeto::Excepcion(_) => "excepcion",
    }
}

pub fn crear_modulo() -> Objeto {
    // Lanza un hilo que ejecuta código Argo en un entorno aislado.
    // Recibe 1 argumento: Objeto::Cadena(codigo_fuente).
    // Retorna inmediatamente Objeto::Booleano(true) al hilo principal.
    fn thread_spawn(args: Vec<Objeto>) -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(format!(
                "thread.spawn: se esperaba 1 argumento (codigo_fuente), se recibieron {}",
                args.len()
            ), Vec::new());
        }

        let codigo = match &args[0] {
            Objeto::Cadena(c) => c.clone(),
            other => {
                return Objeto::Error(format!(
                    "thread.spawn: el argumento debe ser una cadena (código fuente), se recibió {}",
                    tipo_objeto(other)
                ), Vec::new());
            }
        };

        if codigo.is_empty() {
            return Objeto::Error(
                "thread.spawn: el código fuente no puede estar vacío".to_string(),
            Vec::new());
        }

        thread::spawn(move || {
            let resultado = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                let entorno = &mut configurar_entorno_global();
                let lexer = Lexer::nuevo(&codigo);
                let mut parser = Parser::nuevo(lexer);
                let programa = parser.parsear_programa();

                if !parser.errores.is_empty() {
                    eprintln!("[hilo] error de sintaxis:");
                    for error in &parser.errores {
                        eprintln!("[hilo]   {}", error);
                    }
                    return;
                }

                let resultado = evaluar_programa(&programa, entorno);

                if matches!(resultado, Objeto::Error(_, _) | Objeto::Excepcion(_)) {
                    eprintln!("[hilo] error en ejecución: {}", resultado);
                }
            }));

            if let Err(panic_msg) = resultado {
                let msg = if let Some(s) = panic_msg.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic_msg.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "desconocido".to_string()
                };
                eprintln!("[hilo] panic: {}", msg);
            }
        });

        Objeto::Booleano(true)
    }

    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(
        LlaveHash::Cadena("spawn".to_string()),
        Objeto::Nativa(thread_spawn as fn(Vec<Objeto>) -> Objeto),
    );
    Objeto::Diccionario(mapa)
}
