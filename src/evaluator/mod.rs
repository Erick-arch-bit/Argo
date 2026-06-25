//
// Módulo principal del evaluador (árbol de interpretación). Contiene el
// sistema de tipos en tiempo de ejecución (Objeto) y la memoria local
// con alcance anidado (Entorno), más las funciones que recorren el AST
// y producen resultados.

pub mod bytecode;
pub mod modulo;
pub mod object;
pub mod environment;

pub use modulo::importar;
pub use object::LlaveHash;
pub use object::Objeto;
pub use environment::Entorno;

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Importaciones del AST
// ---------------------------------------------------------------------------
// Programa, Statement y Expression son los nodos del árbol sintáctico.
// Se importan desde crate::ast (módulo definido en src/ast/mod.rs).
// Todas son referencias (&) en las funciones del evaluador: el AST se
// construye una vez y se recorre sin tomar ownership. Esto evita clonar
// o mover el árbol, que puede ser grande.
use crate::ast::{Expression, Programa, Statement};
use crate::lexer::token::Token;

// ---------------------------------------------------------------------------
// configurar_entorno_global — Prepara el entorno con funciones nativas
// ---------------------------------------------------------------------------
// Crea un entorno global vacío y le inyecta las funciones built-in del
// lenguaje (print, len, push, tipo). Cada nativa se asigna como
// Objeto::Nativa con un puntero a función Rust.
//
// Se define AQUÍ (y no en main.rs) para que el evaluador pueda crear
// entornos aislados para módulos importados (import) sin depender del
// punto de entrada del binario. Cada módulo recibe su propia copia
// fresca de las nativas, garantizando aislamiento.
//
// Retorna: Entorno — el entorno global listo para usar.
pub fn configurar_entorno_global() -> Entorno {
    let mut entorno = Entorno::nuevo();

    let print_nativa = |args: Vec<Objeto>| -> Objeto {
        let mut partes: Vec<String> = Vec::new();
        for arg in args {
            partes.push(format!("{}", arg));
        }
        println!("{}", partes.join(" "));
        Objeto::Nulo
    };

    let len_nativa = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Número incorrecto de argumentos para len: se esperaba 1"
                    .to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(c) => Objeto::Entero(c.len() as i64),
            Objeto::Arreglo(a) => Objeto::Entero(a.len() as i64),
            Objeto::Diccionario(d) => Objeto::Entero(d.len() as i64),
            _ => Objeto::Error(
                "El tipo no soporta la función len".to_string(),
            ),
        }
    };

    let push_nativa = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 2 {
            return Objeto::Error(
                "Número incorrecto de argumentos para push: se esperaban 2"
                    .to_string(),
            );
        }
        let mut args_iter = args.into_iter();
        let primer_arg = args_iter.next().unwrap();
        let elemento = args_iter.next().unwrap();
        match primer_arg {
            Objeto::Arreglo(lista) => {
                let mut nuevo = lista;
                nuevo.push(elemento);
                Objeto::Arreglo(nuevo)
            }
            _ => Objeto::Error(
                "El primer argumento de push debe ser un arreglo".to_string(),
            ),
        }
    };

    let cast_entero = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento para entero".to_string(),
            );
        }
        match &args[0] {
            Objeto::Cadena(s) => match s.parse::<i64>() {
                Ok(n) => Objeto::Entero(n),
                Err(_) => Objeto::Error(format!(
                    "No se pudo convertir '{}' a entero", s
                )),
            },
            Objeto::Entero(_) => args[0].clone(),
            Objeto::Flotante(f) => Objeto::Entero(*f as i64),
            _ => Objeto::Error(
                "El argumento debe ser una cadena o número".to_string(),
            ),
        }
    };

    let cast_cadena = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Se esperaba 1 argumento para cadena".to_string(),
            );
        }
        Objeto::Cadena(format!("{}", args[0]))
    };

    let tipo_nativa = |args: Vec<Objeto>| -> Objeto {
        if args.len() != 1 {
            return Objeto::Error(
                "Número incorrecto de argumentos para tipo: se esperaba 1"
                    .to_string(),
            );
        }
        let nombre = match &args[0] {
            Objeto::Entero(_) => "entero",
            Objeto::Flotante(_) => "flotante",
            Objeto::Booleano(_) => "booleano",
            Objeto::Cadena(_) => "cadena",
            Objeto::Nulo => "nulo",
            Objeto::Retorno(_) => "retorno",
            Objeto::Error(_) => "error",
            Objeto::Nativa(_) => "nativa",
            Objeto::Funcion { .. } => "funcion",
            Objeto::Arreglo(_) => "arreglo",
            Objeto::Diccionario(_) => "diccionario",
            Objeto::Buffer(_) => "buffer",
            Objeto::Break => "break",
            Objeto::Continue => "continue",
        };
        Objeto::Cadena(nombre.to_string())
    };

    entorno.asignar("print".to_string(), Objeto::Nativa(print_nativa));
    entorno.asignar("len".to_string(), Objeto::Nativa(len_nativa));
    entorno.asignar("push".to_string(), Objeto::Nativa(push_nativa));
    entorno.asignar("tipo".to_string(), Objeto::Nativa(tipo_nativa));
    entorno.asignar("entero".to_string(), Objeto::Nativa(cast_entero));
    entorno.asignar("cadena".to_string(), Objeto::Nativa(cast_cadena));

    // Inyectar módulos de la biblioteca estándar (math, fs, net, json).
    crate::stdlib::inyectar_stdlib(&mut entorno);

    entorno
}

// ---------------------------------------------------------------------------
// evaluar_programa — Punto de entrada del evaluador
// ---------------------------------------------------------------------------
// Itera sobre todas las sentencias del programa ejecutándolas en orden.
// Cada sentencia produce un Objeto; si alguna retorna Error o Retorno,
// el bucle se detiene inmediatamente (short-circuit).
//
// Parámetros:
//   programa: &Programa — referencia inmutable al AST completo.
//     No tomamos ownership del Programa; solo lo prestamos para leerlo.
//     El AST permanece vivo durante toda la evaluación.
//   entorno: &mut Entorno — referencia mutable al entorno de ejecución.
//     Necesitamos &mut porque las sentencias pueden modificar el almacén
//     (asignar variables). El prestamo mutable permite exactamente un
//     escritor a la vez, garantizado por el borrow checker.
//
// Retorno: Objeto — el resultado de la última sentencia evaluada, o
//   el valor de retorno si hubo un return, o el error si algo falló.
//   La ownership del Objeto se transfiere al llamante (se mueve, no se clona).
pub fn evaluar_programa(programa: &Programa, entorno: &mut Entorno) -> Objeto {
    // `resultado` acumula el valor de cada sentencia. Se inicializa como
    // Nulo (equivalente a `undefined` en JS). Su ownership pertenece a
    // esta función; se mueve al retornar o se sobrescribe en cada iteración.
    let mut resultado = Objeto::Nulo;

    // Itera sobre cada sentencia del programa. `&programa.sentencias` es
    // un &Vec<Statement>; el bucle for presta cada elemento como &Statement.
    // No se mueve ninguna sentencia del Vec — solo se referencian.
    for sentencia in &programa.sentencias {
        // evaluar_sentencia recibe &Statement y &mut Entorno.
        // Devuelve un Objeto por ownership (move).
        resultado = evaluar_sentencia(sentencia, entorno);

        // Short-circuit en caso de error o return.
        //
        // Si la sentencia produjo un Error, detenemos la ejecución para
        // evitar errores en cadena. Si produjo un Retorno (return explícito),
        // también detenemos el bucle porque el valor debe propagarse hacia
        // arriba (a la función que llamó, o al programa principal).
        //
        // match toma `&resultado` (prestamo inmutable) para inspeccionar
        // las variantes sin mover el Objeto. Si hiciera `match resultado`,
        // la ownership se transferiría al match y no podríamos retornarlo.
        match &resultado {
            // Objeto::Error(_) y Objeto::Retorno(_) interrumpen el flujo.
            Objeto::Error(_) | Objeto::Retorno(_) => break,
            // Break y Continue se consumen (no se propagan al programa).
            Objeto::Break | Objeto::Continue => break,
            // Cualquier otro valor es un resultado normal.
            _ => {}
        }
    }

    // Retorna el último resultado acumulado.
    // Si el bucle terminó por break (error/return), resultado contiene
    // el Objeto de interrupción. Si terminó normalmente, contiene el
    // resultado de la última sentencia (o Nulo si no hubo sentencias).
    // La ownership de resultado se mueve al llamante.
    resultado
}

// ---------------------------------------------------------------------------
// evaluar_sentencia — Evalúa una sola sentencia del AST
// ---------------------------------------------------------------------------
// Toma una referencia a cualquier nodo Statement y produce un Objeto.
// Las sentencias son las unidades ejecutables del lenguaje: expresiones,
// declaraciones, control de flujo, etc.
//
// Parámetros:
//   sentencia: &Statement — préstamo inmutable del nodo AST.
//     No tomamos ownership; solo leemos la variante y los datos.
//   entorno: &mut Entorno — préstamo mutable para modificar variables.
//
// Retorno: Objeto — el valor resultante de ejecutar la sentencia.
pub fn evaluar_sentencia(sentencia: &Statement, entorno: &mut Entorno) -> Objeto {
    // match por referencia (&sentencia) para no mover el Statement.
    // Si hiciera `match sentencia` (por valor), la ownership se
    // transferiría al match y el Statement no podría reutilizarse.
    match sentencia {
        // Statement::Expresion(expr) — Una expresión usada como sentencia
        // (ej: `foo();`, `1 + 2;`). Delega en evaluar_expresion y retorna
        // su resultado directamente. `expr` es &Expression (referencia al
        // interior del enum Statement, extraída por el pattern matching).
        Statement::Expresion(expr) => evaluar_expresion(expr, entorno),

        // Statement::Bloque(sentencias) — Bloque de código { ... }.
        // Crea un nuevo ámbito léxico. Las variables declaradas dentro
        // del bloque se aíslan del exterior (a menos que se use un
        // mecanismo explícito de mutación). Delega en evaluar_bloque.
        // `sentencias` es &Vec<Statement>; Rust aplica deref coercion
        // a &[Statement] automáticamente al pasarlo a evaluar_bloque.
        Statement::Bloque(sentencias) => evaluar_bloque(sentencias, entorno),

        // Statement::If { condicion, consecuencia, alternativa } —
        // Sentencia condicional if-else. Delega en evaluar_sentencia_if.
        // condicion: &Expression (la expresión booleana a evaluar).
        // consecuencia: &Box<Statement> (rama verdadera, típicamente un
        //   Bloque, pero el parser garantiza que sea una sola sentencia).
        // alternativa: &Option<Box<Statement>> (rama falsa opcional,
        //   puede ser otro If anidado o un Bloque).
        Statement::If {
            condicion,
            consecuencia,
            alternativa,
        } => evaluar_sentencia_if(condicion, consecuencia, alternativa, entorno),

        // Statement::While { condicion, cuerpo } —
        // Bucle while: ejecuta `cuerpo` mientras `condicion` sea true.
        // condicion: &Expression (referencia a la expresión condicional).
        // cuerpo: &Statement (referencia al cuerpo, normalmente un
        //   Bloque { ... }). Delega en evaluar_sentencia_while.
        Statement::While {
            condicion,
            cuerpo,
        } => evaluar_sentencia_while(condicion, cuerpo, entorno),

        // Statement::For { inicializacion, condicion, actualizacion, cuerpo } —
        // Bucle for estilo C: `for (init; cond; update) { cuerpo }`.
        // NO clona el entorno: la inicialización y las mutaciones que
        // ocurren dentro del for se aplican directamente al entorno actual.
        // Esto es consistente con evaluar_bloque (que tampoco clona) y
        // permite que actualizar() modifique variables del ámbito exterior
        // (necesario para `s = s + 1` dentro de un for anidado).
        Statement::For {
            inicializacion,
            condicion,
            actualizacion,
            cuerpo,
        } => {
            // 1. Evaluar la inicialización en el entorno actual.
            //    No se clona el entorno: las declaraciones `let i = 0`
            //    se inyectan directamente en el ámbito actual
            //    (consistente con evaluar_bloque). Esto permite que
            //    actualizar() modifique variables del ámbito exterior.
            let resultado_init = evaluar_sentencia(inicializacion, entorno);
            if let Objeto::Error(_) = &resultado_init { return resultado_init }

            // 2. Bucle principal.
            loop {
                // 2a. Evaluar la condición.
                let resultado_cond = evaluar_expresion(condicion, entorno);
                if let Objeto::Error(_) = &resultado_cond { return resultado_cond }

                // 2b. Si la condición es falsy, salir del bucle.
                if !es_truthy(&resultado_cond) {
                    break;
                }

                // 2c. Evaluar el cuerpo.
                let resultado_cuerpo = evaluar_sentencia(cuerpo, entorno);
                match &resultado_cuerpo {
                    Objeto::Retorno(_) | Objeto::Error(_) => {
                        return resultado_cuerpo;
                    }
                    // Break: detener el bucle sin propagar la señal.
                    Objeto::Break => break,
                    // Continue: saltar a la actualización y siguiente iteración.
                    Objeto::Continue => {}
                    _ => {}
                }

                // 2d. Evaluar la actualización (normalmente i = i + 1).
                let resultado_act = evaluar_sentencia(actualizacion, entorno);
                if let Objeto::Error(_) = &resultado_act { return resultado_act }
            }

            // 3. El bucle terminó naturalmente. Nulo es el valor de toda
            //    sentencia que no produce un resultado significativo.
            Objeto::Nulo
        }

        // Statement::Return(valor) — Sentencia return.
        // `valor` es &Option<Expression> (referencia a la expresión
        // opcional de retorno). Delega en evaluar_sentencia_return.
        Statement::Return(valor) => evaluar_sentencia_return(valor, entorno),

        // Statement::Break — Señal de salida de bucle.
        // Retorna Objeto::Break que el bucle for/while intercepta
        // y consume al salir.
        Statement::Break => Objeto::Break,

        // Statement::Continue — Señal de salto a siguiente iteración.
        // Retorna Objeto::Continue que el bucle for/while intercepta.
        Statement::Continue => Objeto::Continue,

        // Statement::TryCatch { bloque_try, parametro_catch, bloque_catch } —
        // Manejo de excepciones: `try { ... } catch (e) { ... }`.
        //
        // # Mecanismo de intercepción
        // 1. Se ejecuta `bloque_try` en el entorno actual.
        // 2. Si el resultado es Objeto::Error(mensaje):
        //    a. Se crea un entorno hijo (ámbito local para catch).
        //    b. Se asigna el mensaje de error como Objeto::Cadena a la
        //       variable `parametro_catch` dentro de ese entorno.
        //    c. Se ejecuta `bloque_catch` en ese entorno hijo.
        //    d. El resultado de catch se retorna (reemplaza al error).
        // 3. Si el resultado NO es Error (Entero, Cadena, Nulo, Retorno,
        //    etc.), se retorna directamente sin ejecutar el catch.
        //
        // # Propagación controlada
        // Normalmente, cuando evaluar_sentencia encuentra un Error, lo
        // propaga hacia arriba (short-circuit). Aquí ROMPEMOS esa cadena:
        // capturamos el error y lo convertimos en un valor manejable
        // (Objeto::Cadena) dentro del bloque catch. Si el catch mismo
        // produce un error (por ejemplo, una variable no definida),
        // ese error SÍ se propaga al exterior (es un error del catch,
        // no del try).
        Statement::TryCatch {
            bloque_try,
            parametro_catch,
            bloque_catch,
        } => {
            // 1. Ejecutar el bloque protegido.
            let resultado_try = evaluar_sentencia(bloque_try, entorno);

            // 2. Evaluar el resultado: ¿es un error capturable?
            match resultado_try {
                // 2a. Sí: interceptar el error.
                Objeto::Error(mensaje) => {
                    // Crear un ámbito local aislado para el catch.
                    // Esto evita que la variable del error (ej. `e`)
                    // contamine el entorno exterior.
                    let mut entorno_catch =
                        Entorno::nuevo_local(entorno.clone());

                    // Convertir el mensaje de Rust a una cadena de Argo
                    // y asignarla al parámetro del catch.
                    let objeto_error = Objeto::Cadena(mensaje);
                    entorno_catch.asignar(
                        parametro_catch.clone(),
                        objeto_error,
                    );

                    // Ejecutar el bloque de rescate en el entorno local.
                    // Si el catch mismo produce un error, SE PROPAGA
                    // (es un error del bloque catch, no del try).
                    evaluar_sentencia(bloque_catch, &mut entorno_catch)
                }
                // 2b. No es error: retornar el valor original sin tocar.
                //     Esto incluye Objeto::Retorno y Objeto::Nulo.
                //     El catch no se ejecuta.
                otro => otro,
            }
        }

        // Statement::DeclaracionVariable { nombre, valor, constante } —
        // Declaración de variable: `let x = <expr>;` o `const x = <expr>;`.
        // 1. Evalúa la expresión de inicialización.
        // 2. Si el resultado es Error, lo propaga inmediatamente
        //    (short-circuit: no se asigna nada al entorno).
        // 3. Si es válido, asigna la variable en el entorno actual
        //    mediante `entorno.asignar()`.
        // 4. Las declaraciones `let`/`const` retornan Nulo.
        Statement::DeclaracionVariable { nombre, valor, constante } => {
            let resultado = evaluar_expresion(valor, entorno);
            if let Objeto::Error(_) = &resultado { return resultado }
            entorno.declarar(nombre.clone(), resultado, *constante);
            Objeto::Nulo
        }

        // Statement::AsignacionVariable { nombre, valor } —
        // Reasignación de variable existente: `identificador = expr;`.
        // A diferencia de DeclaracionVariable (que crea o sobrescribe en
        // el ámbito local), esta sentencia recorre la cadena de ámbitos
        // buscando la variable para actualizarla. Si no existe en ningún
        // ámbito, retorna un error ("Variable no definida").
        Statement::AsignacionVariable { nombre, valor } => {
            // 1. Evaluar la expresión del valor.
            let resultado = evaluar_expresion(valor, entorno);

            // 2. Short-circuit: si la evaluación falló, propagar error.
            if let Objeto::Error(_) = &resultado { return resultado }

            // 3. Actualizar la variable en la cadena de ámbitos.
            //    entorno.actualizar() busca recursivamente (o itera con
            //    bucle) desde el ámbito actual hasta el global. Si no
            //    encuentra la variable, retorna Err(String).
            match entorno.actualizar(nombre, resultado) {
                Ok(()) => {
                    // La reasignación no produce un valor evaluable.
                    Objeto::Nulo
                }
                Err(mensaje) => {
                    // Variable no encontrada en ningún ámbito.
                    Objeto::Error(mensaje)
                }
            }
        }

        // Statement::DeclaracionFuncion { nombre, parametros, cuerpo } —
        // Declaración de función: `fn foo(a, b) { ... }`.
        // 1. Clona el entorno actual para capturarlo en el closure.
        // 2. Crea un Objeto::Funcion con los parámetros, el cuerpo y
        //    el entorno capturado (cierre léxico).
        // 3. Almacena la función en el entorno actual como una variable
        //    más, permitiendo que otras expresiones la referencien por
        //    nombre y la invoquen con llamadas foo().
        // 4. Retorna Nulo (las declaraciones no producen valor).
        //
        // # Gestión de memoria (captura del entorno)
        //   entorno.clone() copia el HashMap actual y su cadena de
        //   padres. Esta copia queda "congelada" dentro del Objeto::Funcion
        //   y se usará como ámbito padre cuando la función se invoque.
        //   Así se implementan los closures: la función recuerda las
        //   variables de su contexto de definición.
        Statement::DeclaracionFuncion {
            nombre,
            parametros,
            cuerpo,
        } => {
            // 1. Capturar el entorno actual.
            //    `entorno.clone()` copia profundamente todo el ámbito
            //    visible en el momento de la definición. Esta copia se
            //    mueve al Objeto::Funcion y no se modifica después.
            //    Es una captura por valor (no por referencia), lo que
            //    evita complejidades de lifetimes con referencias.
            let entorno_capturado = entorno.clone();

            // 2. Construir el objeto función.
            //    parametros.clone(): clona el Vec<String> del AST.
            //      necesario porque el AST presta &Vec<String> y
            //      Objeto::Funcion necesita ownership.
            //    cuerpo.clone(): clona el Box<Statement> (árbol AST
            //      completo del cuerpo). clone() aloca un nuevo
            //      Box y copia recursivamente todas las sentencias
            //      internas. Es O(n_sentencias) en heap.
            //    entorno: entorno_capturado se mueve al enum.
            let funcion = Objeto::Funcion {
                parametros: parametros.clone(),
                cuerpo: cuerpo.clone(),
                entorno: entorno_capturado,
            };

            // 3. Almacenar la función en el entorno actual.
            //    nombre.clone() clona el String del AST porque
            //    HashMap::insert requiere ownership de la clave.
            entorno.asignar(nombre.clone(), funcion);

            // 4. Las declaraciones de función no producen valor.
            Objeto::Nulo
        }
    }
}

// ---------------------------------------------------------------------------
// evaluar_bloque — Evalúa un bloque de código delimitado por { }
// ---------------------------------------------------------------------------
// Crea un sub-ámbito para las declaraciones `let`/`const` dentro del
// bloque. Las variables declaradas dentro del bloque se aíslan del
// ámbito exterior (scope léxico estándar). La reasignación (`actualizar`)
// recorre la cadena de ámbitos hacia arriba, por lo que modificar una
// variable del padre desde dentro del bloque funciona correctamente.
//
// Parámetros:
//   sentencias: &[Statement] — slice de sentencias del bloque.
//   entorno: &mut Entorno — entorno del ámbito padre.
//
// Retorno: Objeto — el resultado de la última sentencia, o Nulo si el
//   bloque está vacío, o Error/Retorno si se propagan.
fn evaluar_bloque(sentencias: &[Statement], entorno: &mut Entorno) -> Objeto {
    if sentencias.is_empty() {
        return Objeto::Nulo;
    }
    let mut entorno_bloque = Entorno::nuevo_local(entorno.clone());
    let mut resultado = Objeto::Nulo;

    for sentencia in sentencias {
        resultado = evaluar_sentencia(sentencia, &mut entorno_bloque);

        match &resultado {
            Objeto::Error(_) | Objeto::Retorno(_) | Objeto::Break | Objeto::Continue => break,
            _ => {}
        }
    }

    resultado
}

// ---------------------------------------------------------------------------
// evaluar_sentencia_if — Evalúa una sentencia if/else
// ---------------------------------------------------------------------------
// Evalúa la condición y, según su valor truthy, ejecuta la rama
// consecuencia (if) o la rama alternativa (else/else if).
//
// Parámetros:
//   condicion: &Expression — expresión a evaluar como condición.
//   consecuencia: &Statement — rama verdadera (típicamente un Bloque).
//     Viene dentro de Box<Statement> en el AST; aquí ya es &Statement
//     (deref coercion del pattern match).
//   alternativa: &Option<Box<Statement>> — rama falsa opcional.
//     Option<Box<Statement>>; se usa .as_ref() para obtener &Statement
//     sin mover el Box.
//   entorno: &mut Entorno — entorno actual. NO se clona aquí porque
//     la consecuencia y alternativa ya contienen Bloques que crearán
//     sus propios sub-ámbitos via evaluar_bloque.
//
// Retorno: Objeto — el resultado de la rama ejecutada, o Nulo si no
//   hay alternativa, o Error si la condición falla.
fn evaluar_sentencia_if(
    condicion: &Expression,
    consecuencia: &Statement,
    alternativa: &Option<Box<Statement>>,
    entorno: &mut Entorno,
) -> Objeto {
    // 1. Evaluar la condición.
    //    evaluar_expresion recibe &Expression y &mut Entorno. Retorna
    //    un Objeto por ownership. No necesitamos &mut para la condición
    //    (es de solo lectura), pero la firma de evaluar_expresion exige
    //    &mut Entorno por consistencia con otras expresiones.
    let resultado_condicion = evaluar_expresion(condicion, entorno);

    // 2. Short-circuit: si la condición produjo Error, propagarlo.
    //    match por referencia para no mover el Objeto.
    if let Objeto::Error(_) = &resultado_condicion { return resultado_condicion }

    // 3. Determinar la truthyness de la condición.
    //    es_truthy toma &Objeto (préstamo inmutable). No consume
    //    ownership de resultado_condicion, que se dropea al salir
    //    de esta función (innecesario después de la decisión).
    if es_truthy(&resultado_condicion) {
        // Rama verdadera: evaluar la consecuencia.
        // consecuencia es &Statement (normalmente un Bloque { ... }).
        // evaluar_sentencia recibe &Statement y &mut Entorno.
        // Como consecuencia suele ser Bloque, evaluar_bloque creará
        // un sub-ámbito automáticamente. No necesitamos clonar
        // entorno aquí.
        evaluar_sentencia(consecuencia, entorno)
    } else {
        // Rama falsa: evaluar la alternativa si existe.
        // alternativa es &Option<Box<Statement>>.
        // .as_ref() convierte &Option<Box<T>> en Option<&Box<T>>.
        // Luego .and_then() aplica la función si es Some, o retorna
        // None si no hay alternativa. Si hay alternativa, el Box se
        // desempaqueta por deref coercion (&Box<Statement> → &Statement).
        //
        // map_or_else es equivalente: si Some, llama al closure con
        // el valor; si None, retorna el default (Objeto::Nulo).
        match alternativa.as_ref() {
            Some(alt) => evaluar_sentencia(alt, entorno),
            None => Objeto::Nulo,
        }
    }
}

// ---------------------------------------------------------------------------
// evaluar_sentencia_return — Evalúa una sentencia return
// ---------------------------------------------------------------------------
// Evalúa la expresión de retorno (si existe) y envuelve el resultado
// en Objeto::Retorno(Box<Objeto>). La envoltura Retorno permite que
// el valor se propague a través de las llamadas anidadas de funciones
// y bloques sin perder la semántica de "retorno temprano".
//
// Parámetros:
//   expresion: &Option<Expression> — referencia a la expresión opcional
//     de retorno. None significa `return;` (retorna Nulo).
//   entorno: &mut Entorno — entorno actual para evaluar la expresión.
//
// Retorno: Objeto::Retorno(Box<Objeto>) con el valor evaluado, o
//   Objeto::Error si la evaluación falla.
//
// # Gestión de memoria (Box)
//   Box<Objeto> asigna el Objeto en el heap. Esto es necesario porque
//   Objeto::Retorno(Objeto) crearía recursión infinita en el enum
//   (Objeto contiene Retorno que contiene Objeto...). Box rompe la
//   recursión: el enum solo almacena un puntero de 8 bytes al heap.
fn evaluar_sentencia_return(
    expresion: &Option<Expression>,
    entorno: &mut Entorno,
) -> Objeto {
    // Evaluar la expresión si existe, o usar Nulo como valor por defecto.
    // `expresion` es &Option<Expression>; el match por referencia evita
    // mover la Option del AST. En el brazo Some(expr), `expr` es
    // &Expression (referencia a la expresión dentro del AST).
    let valor = match expresion {
        // Hay expresión de retorno: evaluarla.
        Some(expr) => {
            let resultado = evaluar_expresion(expr, entorno);

            // Short-circuit: si la evaluación produjo Error,
            // propagarlo inmediatamente en lugar de retornar.
            if let Objeto::Error(_) = &resultado { return resultado }

            resultado
        }
        // Return sin expresión: valor por defecto es Nulo.
        None => Objeto::Nulo,
    };

    // Envolver el valor en Objeto::Retorno con Box.
    // Box::new(valor) mueve `valor` al heap (allocación de 16-24 bytes
    // según la variante del Objeto). La ownership del heap se transfiere
    // al Box; cuando el Box se dropee, el Objeto interno también se dropea.
    // El llamante (evaluar_programa, evaluar_bloque, etc.) reconoce
    // Objeto::Retorno en el match de short-circuit y lo propaga hacia
    // arriba, deteniendo la ejecución de más sentencias.
    Objeto::Retorno(Box::new(valor))
}

// ---------------------------------------------------------------------------
// evaluar_sentencia_while — Evalúa un bucle while
// ---------------------------------------------------------------------------
// Itera mientras la condición sea truthy. En cada iteración:
//   1. Evalúa la condición.
//   2. Si es Error, propaga (short-circuit).
//   3. Si es falsy, rompe el bucle (break).
//   4. Si es truthy, evalúa el cuerpo.
//   5. Si el cuerpo produce Error o Retorno, propaga inmediatamente
//      (escape estricto: el bucle no continúa después de un return).
//
// Parámetros:
//   condicion: &Expression — expresión que se reevalúa en cada ciclo.
//   cuerpo: &Statement — sentencia a ejecutar en cada iteración
//     (normalmente un Bloque { ... }).
//   entorno: &mut Entorno — entorno mutable para evaluar expresiones.
//
// Retorno: Objeto::Nulo si el bucle termina naturalmente, o el
//   Objeto::Retorno/Objeto::Error que se propagó desde el cuerpo.
fn evaluar_sentencia_while(
    condicion: &Expression,
    cuerpo: &Statement,
    entorno: &mut Entorno,
) -> Objeto {
    // loop { ... } es el bucle infinito de Rust. No hay límite de
    // iteraciones; Argo confía en que el programador escriba condiciones
    // que eventualmente sean falsas. Un while(true) sin break causaría
    // un bucle infinito real (el intérprete no terminaría), igual que
    // en cualquier otro lenguaje sin protección runtime.
    loop {
        // 1. Evaluar la condición en cada iteración.
        //    La condición se reevalúa siempre, permitiendo que
        //    efectos secundarios (como variables mutadas en el cuerpo)
        //    afecten al resultado de la próxima evaluación.
        let evaluacion_condicion = evaluar_expresion(condicion, entorno);

        // 2. Short-circuit: si la condición es Error, propagar.
        //    Esto evita que el bucle continúe con un estado inválido.
        if let Objeto::Error(_) = &evaluacion_condicion { return evaluacion_condicion }

        // 3. Decidir si continuar o romper el bucle.
        //    es_truthy evalúa el valor booleano implícito del Objeto.
        //    Si es false (0, null, cadena vacía, etc.), break.
        if !es_truthy(&evaluacion_condicion) {
            // La condición es falsy: salir del bucle.
            // `break` termina el loop { } y la función retorna
            // Objeto::Nulo (abajo, después del loop).
            break;
        }

        // 4. Evaluar el cuerpo del bucle.
        //    Normalmente cuerpo es un Bloque { ... }, que a su vez
        //    llama a evaluar_bloque, el cual crea un sub-entorno.
        //    Las variables declaradas dentro del cuerpo se aíslan
        //    en ese sub-entorno y se dropean al final de cada
        //    iteración (no persisten entre ciclos).
        let resultado_cuerpo = evaluar_sentencia(cuerpo, entorno);

        // 5. Escape estricto: si el cuerpo produjo Retorno o Error,
        //    propagarlo inmediatamente al llamante. Esto asegura que
        //    un `return` dentro de un while detiene TODO el bucle
        //    y la función circundante, no solo la iteración actual.
        //
        //    Sin esta verificación, el bucle continuaría iterando
        //    ignorando el retorno, lo que sería incorrecto.
        match &resultado_cuerpo {
            Objeto::Retorno(_) | Objeto::Error(_) => return resultado_cuerpo,
            // Break: señala que el cuerpo ejecutó break; detener el
            // bucle y retornar Nulo (no propagar el Break al exterior).
            Objeto::Break => break,
            // Continue: continuar con la siguiente iteración
            // (re-evaluar la condición).
            Objeto::Continue => {}
            _ => {}
        }

        // 6. La iteración termina. Si no hubo break, retorno ni error,
        //    el loop vuelve al paso 1 para reevaluar la condición.
        //    No hay sleep ni yield — el bucle es tight (ocupado).
    }

    // El bucle terminó porque la condición se volvió falsa o porque
    // se ejecutó break. Retornar Nulo: en Argo, un bucle no produce valor.
    Objeto::Nulo
}

// ---------------------------------------------------------------------------
// evaluar_llamada_funcion — Ejecuta una función del usuario
// ---------------------------------------------------------------------------
// Toma ownership de la función (Objeto::Funcion) y los argumentos
// evaluados (Vec<Objeto>). Extrae los parámetros, cuerpo y entorno
// capturado, verifica aridad, crea un nuevo ámbito de ejecución con
// los argumentos asignados, evalúa el cuerpo y desenrolla el retorno.
//
// Parámetros:
//   funcion: Objeto — debe ser Objeto::Funcion; se consume (move)
//     para extraer sus campos internos sin clonar.
//   argumentos: Vec<Objeto> — argumentos ya evaluados, ownership
//     transferido. Se consumen al asignarlos al nuevo entorno.
//
// Retorno: Objeto — el resultado de la función, o Objeto::Error si
//   la aridad no coincide o la ejecución falla.
//
// # Regla de Desenrollado de Retorno
//   Si el cuerpo retorna Objeto::Retorno(valor), se desempaqueta
//   (*valor) para obtener el Objeto interno. Esto evita que el
//   Retorno se propague más allá de la llamada a función: la
//   envoltura Retorno es un mecanismo interno de propagación a
//   través de sentencias anidadas; la llamada a función lo absorbe.
fn evaluar_llamada_funcion(funcion: Objeto, argumentos: Vec<Objeto>) -> Objeto {
    // `funcion` se mueve al match; el brazo determina el tipo de
    // invocación: Funcion (definida por el usuario, necesita entorno)
    // o Nativa (built-in, llamada directa a fn pointer).
    match funcion {
        // ===================================================================
        // Función definida por el usuario (closure con entorno capturado)
        // ===================================================================
        Objeto::Funcion {
            parametros,
            cuerpo,
            entorno: entorno_capturado,
        } => {
            // 1. Verificar aridad.
            if parametros.len() != argumentos.len() {
                return Objeto::Error(format!(
                    "Número incorrecto de argumentos: se esperaban {}, se recibieron {}",
                    parametros.len(),
                    argumentos.len()
                ));
            }

            // 2. Crear entorno de llamada con el entorno capturado como padre.
            //    Esto implementa el cierre léxico: la función ve las
            //    variables que existían cuando fue definida.
            let mut entorno_llamada = Entorno::nuevo_local(entorno_capturado);

            // 3. Asignar argumentos a parámetros.
            //    parametros.into_iter() consume el Vec (no clonación).
            //    argumentos.into_iter() consume el Vec de argumentos.
            //    zip() empareja cada uno. asignar() mueve ambos al HashMap.
            for (param, arg) in parametros.into_iter().zip(argumentos) {
                entorno_llamada.asignar(param, arg);
            }

            // 4. Evaluar el cuerpo de la función.
            let resultado = evaluar_sentencia(&cuerpo, &mut entorno_llamada);

            // 5. Desenrollar el retorno.
            match resultado {
                Objeto::Retorno(valor) => *valor,
                _ => resultado,
            }
        }

        // ===================================================================
        // Función nativa del sistema (built-in en Rust)
        // ===================================================================
        // Las funciones nativas reciben ownership completo del Vec de
        // argumentos y retornan un Objeto. No hay entorno capturado ni
        // desenrollado de retorno: la lógica se ejecuta directamente y
        // el resultado se retorna tal cual.
        //
        // `func_rust` es fn(Vec<Objeto>) -> Objeto, un puntero a función
        // de Rust. Se invoca con (func_rust)(argumentos). Los paréntesis
        // alrededor de func_rust son necesarios para desambiguar la
        // sintaxis de llamada de función en Rust.
        Objeto::Nativa(func_rust) => (func_rust)(argumentos),

        // ===================================================================
        // No invocable (seguridad: no debería llegar aquí por el check
        // en evaluar_expresion, pero se cubre por completitud)
        // ===================================================================
        _ => Objeto::Error("No es una función invocable".to_string()),
    }
}

// ---------------------------------------------------------------------------
// es_truthy — Determina el valor booleano implícito de un Objeto
// ---------------------------------------------------------------------------
// Reglas de "truthyness" similares a las de otros lenguajes dinámicos:
//   - Nulo → false
//   - Booleano(v) → v
//   - Entero(v) → v != 0
//   - Flotante(v) → v != 0.0
//   - Cadena(v) → !v.is_empty()
//   - Retorno(v) → delegar en el valor interno (recursivo)
//   - Error(_) → false (un error no es verdadero)
//
// Parámetros:
//   objeto: &Objeto — préstamo inmutable. Solo leemos el valor,
//     no tomamos ownership. Esto permite usar es_truthy sin
//     consumir el Objeto (el llamante retiene su ownership).
fn es_truthy(objeto: &Objeto) -> bool {
    match objeto {
        // Nulo siempre es falso (como null en JS/Java).
        Objeto::Nulo => false,

        // Booleano: usar el valor directamente (Copy, desreferencia).
        Objeto::Booleano(v) => *v,

        // Entero: 0 es falso, cualquier otro valor es verdadero.
        Objeto::Entero(v) => *v != 0,

        // Flotante: 0.0 es falso (incluyendo -0.0), cualquier otro
        // valor es verdadero. Se usa `!=` que compara bitwise.
        Objeto::Flotante(v) => *v != 0.0,

        // Cadena: vacía es falsa, no vacía es verdadera.
        Objeto::Cadena(v) => !v.is_empty(),

        // Retorno: extraer el valor interno con deref (Box<Objeto>
        // implementa Deref<Target=Objeto>) y evaluar recursivamente.
        // El `&**objeto` no es necesario por deref coercion.
        Objeto::Retorno(v) => es_truthy(v),

        // Arreglo: vacío es falso, no vacío es verdadero.
        // Similar al comportamiento de arrays en JavaScript.
        Objeto::Arreglo(v) => !v.is_empty(),

        // Diccionario: vacío es falso, no vacío es verdadero.
        Objeto::Diccionario(m) => !m.is_empty(),

        // Buffer: vacío es falso, no vacío es verdadero.
        Objeto::Buffer(v) => !v.is_empty(),

        // Break: señal de control, no es un valor real. Se considera
        // falso (nunca debería llegar a es_truthy en condiciones normales,
        // porque el bucle lo intercepta antes).
        Objeto::Break => false,

        // Continue: señal de control de salto de iteración. No es un
        // valor real, se intercepta en el bucle antes de es_truthy.
        Objeto::Continue => false,

        // Error: cualquier error se considera falso (no se puede
        // "negar" un error de forma significativa).
        Objeto::Error(_) => false,

        // Funcion: una función definida por el usuario es un valor
        // que se considera verdadero (como en JavaScript, donde una
        // función es truthy). Siempre que exista una función, puede
        // ser llamada, por lo que no es un valor vacío.
        Objeto::Funcion { .. } => true,

        // Nativa: una función nativa del sistema también es truthy
        // (es una función, al igual que Funcion). Independientemente
        // de qué función sea, su presencia es un valor verdadero.
        Objeto::Nativa(_) => true,
    }
}

// ---------------------------------------------------------------------------
// evaluar_unario — Aplica un operador unario sobre un Objeto
// ---------------------------------------------------------------------------
// Toma ownership del Objeto derecho (`derecha` se mueve aquí) para
// evitar clonar. Si la operación es válida, se retorna un nuevo Objeto;
// si no, se retorna Objeto::Error con un mensaje descriptivo.
//
// Parámetros:
//   operador: &str — el operador como cadena ("-", "!", "+").
//   derecha: Objeto — el valor sobre el cual aplicar el operador.
//     Se recibe por ownership (move) para evitar clone().
fn evaluar_unario(operador: &str, derecha: Objeto) -> Objeto {
    match operador {
        // -------------------------------------------------------------------
        // Negación aritmética: "-"
        // -------------------------------------------------------------------
        // Solo tiene sentido semántico sobre Entero y Flotante.
        // Para Entero: -v. Rust maneja overflow en debug mode, pero
        // en release usa wrapping semantics (two's complement).
        // En Argo no hay panic! por overflow aritmético.
        "-" => match derecha {
            // i64 es Copy; `-v` produce un nuevo i64 en el stack.
            // Se envuelve en Objeto::Entero (ownership al llamante).
            Objeto::Entero(v) => Objeto::Entero(-v),

            // f64 es Copy; `-v` produce un nuevo f64 en el stack.
            Objeto::Flotante(v) => Objeto::Flotante(-v),

            // Cualquier otro tipo no es negable. Se retorna Error
            // con el Display del Objeto para dar contexto.
            _ => Objeto::Error(format!(
                "Negación aritmética no soportada para: {}",
                derecha
            )),
        },

        // -------------------------------------------------------------------
        // Negación lógica: "!"
        // -------------------------------------------------------------------
        // Invierte el valor booleano implícito (truthy/falsy).
        // Similar a ! en JavaScript o not en Python.
        "!" => {
            // es_truthy toma &Objeto (préstamo), no consume ownership.
            let booleano = es_truthy(&derecha);
            // Retornar el booleano negado. `derecha` se dropea aquí
            // porque su ownership se movió a esta función y no se
            // usó después del préstamo a es_truthy.
            Objeto::Booleano(!booleano)
        }

        // -------------------------------------------------------------------
        // Operador unario "+" (no-op)
        // -------------------------------------------------------------------
        // En la mayoría de los lenguajes, +expr es un no-op que solo
        // fuerza coerción numérica. En Argo, retornamos el valor tal
        // cual. La ownership de derecha se transfiere al retorno.
        "+" => derecha,

        // -------------------------------------------------------------------
        // Operador desconocido (bug del parser o error interno)
        // -------------------------------------------------------------------
        _ => Objeto::Error(format!(
            "Operador unario desconocido: {}",
            operador
        )),
    }
}

// ---------------------------------------------------------------------------
// evaluar_binario — Aplica un operador binario sobre dos Objetos
// ---------------------------------------------------------------------------
// Recibe ownership de ambos operandos (izquierda, derecha se mueven aquí)
// para evitar clonaciones innecesarias. Usa un match sobre tuplas para
// implementar tipado estricto por combinación de tipos:
//
//   (Entero, Entero)  → operaciones aritméticas y relacionales
//   (Flotante, Flotante) → aritméticas y relacionales
//   (Cadena, Cadena)   → concatenación (+) y comparación (==, !=)
//   (Booleano, Booleano) → comparación (==, !=)
//   Tipos distintos    → igualdad universal (== false, != true)
//   Otra combinación   → Objeto::Error ("Discrepancia de tipos")
//
// Parámetros:
//   operador: &str  — representación textual del operador ("+", "*", ...)
//   izquierda: Objeto — operando izquierdo (ownership transferido)
//   derecha: Objeto   — operando derecho (ownership transferido)
//
// Retorno: Objeto — el resultado de la operación, o Error si falla.
fn evaluar_binario(operador: &str, izquierda: Objeto, derecha: Objeto) -> Objeto {
    // Match por valor sobre la tupla (izquierda, derecha).
    // Al pasar ownership, cada brazo recibe los datos internos sin
    // necesidad de desreferenciar. Los tipos Copy (i64, f64, bool)
    // se copian implícitamente; los tipos heap (String) se mueven.
    //
    // NOTA: Si usáramos match por referencia (&izquierda, &derecha),
    // los patrones internos serían &Objeto::Entero(&i64) y habría
    // que desreferenciar cada operación. El match por valor es más
    // limpio y consume ownership, que era el destino de estos valores.
    match (izquierda, derecha) {
        // ===================================================================
        // (Entero, Entero) — Aritmética y relacionales
        // ===================================================================
        // Entero mantiene su semántica de complemento a dos sin overflow
        // check. Rust en release wrapping, en debug panic. Argo nunca
        // usa panic! incluso en debug.
        (Objeto::Entero(i), Objeto::Entero(d)) => match operador {
            // Aritméticos básicos
            "+" => Objeto::Entero(i + d),
            "-" => Objeto::Entero(i - d),
            "*" => Objeto::Entero(i * d),

            // División entera: si d == 0, error. Rust trunca hacia
            // cero (como C99/C++11). i / d para i>0,d>0 da cociente
            // esperado. No hay panic! incluso en debug porque
            // prevenimos d == 0 explícitamente.
            "/" => {
                if d == 0 {
                    Objeto::Error("Error: división por cero".to_string())
                } else {
                    Objeto::Entero(i / d)
                }
            }

            // Módulo: igual, d == 0 produce error. i % d con i
            // negativo retorna resto con signo del dividendo
            // (estilo C99, no Python).
            "%" => {
                if d == 0 {
                    Objeto::Error("Error: módulo por cero".to_string())
                } else {
                    Objeto::Entero(i % d)
                }
            }

            // Relacionales — retornan Objeto::Booleano (nuevo enum,
            // no hay clonación). Todos los operadores usan los
            // traits nativos de Rust sobre i64.
            "<"  => Objeto::Booleano(i < d),
            ">"  => Objeto::Booleano(i > d),
            "<=" => Objeto::Booleano(i <= d),
            ">=" => Objeto::Booleano(i >= d),

            // Igualdad — i64 implementa PartialEq, comparación
            // bitwise sin heap allocation.
            "==" => Objeto::Booleano(i == d),
            "!=" => Objeto::Booleano(i != d),

            // Bitwise — operan sobre la representación binaria
            // del entero con complemento a dos.
            "&" => Objeto::Entero(i & d),
            "|" => Objeto::Entero(i | d),
            "^" => Objeto::Entero(i ^ d),
            "<<" => Objeto::Entero(i << d),
            ">>" => Objeto::Entero(i >> d),

            // Operador no soportado para Entero (ej. concatenar
            // enteros con "+" no, pero eso ya lo cubrimos arriba).
            // Cae aquí cualquier operador que no tenga sentido
            // sobre enteros, como "&&", "||", etc.
            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre enteros", operador
            )),
        },

        // ===================================================================
        // (Flotante, Flotante) — Aritmética y relacionales
        // ===================================================================
        // f64 implementa todos los operadores aritméticos y
        // relacionales. La división por cero produce infinity
        // o NaN (IEEE 754), no panic!.
        (Objeto::Flotante(i), Objeto::Flotante(d)) => match operador {
            "+" => Objeto::Flotante(i + d),
            "-" => Objeto::Flotante(i - d),
            "*" => Objeto::Flotante(i * d),
            "/" => Objeto::Flotante(i / d),
            "%" => Objeto::Flotante(i % d),

            "<"  => Objeto::Booleano(i < d),
            ">"  => Objeto::Booleano(i > d),
            "<=" => Objeto::Booleano(i <= d),
            ">=" => Objeto::Booleano(i >= d),

            "==" => Objeto::Booleano(i == d),
            "!=" => Objeto::Booleano(i != d),

            // Bitwise: error explícito, los flotantes no tienen
            // representación binaria para operaciones a nivel de bits.
            "&" | "|" | "^" | "<<" | ">>" => Objeto::Error(
                "Las operaciones a nivel de bits solo soportan números enteros"
                    .to_string(),
            ),

            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre flotantes", operador
            )),
        },

        // ===================================================================
        // (Cadena, Cadena) — Concatenación y comparación
        // ===================================================================
        (Objeto::Cadena(i), Objeto::Cadena(d)) => match operador {
            // Concatenación: i y d son Strings (ownership movido aquí).
            // Creamos un nuevo String vacío (heap allocation),
            // volcamos i con push_str (reutiliza el buffer de i),
            // luego volcamos d. El String original i se vacía
            // parcialmente (push_str mueve los bytes al nuevo
            // buffer). Al final, retornamos la cadena concatenada
            // envuelta en Objeto::Cadena.
            //
            // Clone de i no es necesario porque tenemos ownership
            // y podemos mutarlo. push_str sobrevive porque
            // String no requiere Clone para mutación interna.
            "+" => {
                // i y d son String (ownership en este scope).
                // let mut concat = i.clone() crearía una copia
                // heap O(n+m). En su lugar, movemos i a concat
                // (i se transfiere, sin copia) y luego extendemos.
                let mut concat = i;
                // push_str toma &str (préstamo de d). d no se
                // mueve, solo se presta. Después de push_str,
                // concat contiene "i" + "d" en un solo buffer heap.
                // d se dropea al final del match arm.
                concat.push_str(&d);
                Objeto::Cadena(concat)
            }

            // Comparación: String implementa PartialEq (lexicográfica).
            "==" => Objeto::Booleano(i == d),
            "!=" => Objeto::Booleano(i != d),

            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre cadenas", operador
            )),
        },

        // ===================================================================
        // (Cadena, Entero) y (Cadena, Flotante) — Concatenación con número
        // ===================================================================
        // Convierte el número a su representación textual y lo concatena
        // con la cadena. Similar a JavaScript: "T: " + 42 → "T: 42".
        (Objeto::Cadena(i), Objeto::Entero(d)) => match operador {
            "+" => Objeto::Cadena(format!("{}{}", i, d)),
            "==" => Objeto::Booleano(false),
            "!=" => Objeto::Booleano(true),
            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre cadena y entero", operador
            )),
        },
        (Objeto::Cadena(i), Objeto::Flotante(d)) => match operador {
            "+" => Objeto::Cadena(format!("{}{}", i, d)),
            "==" => Objeto::Booleano(false),
            "!=" => Objeto::Booleano(true),
            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre cadena y flotante", operador
            )),
        },
        (Objeto::Entero(i), Objeto::Cadena(d)) => match operador {
            "+" => Objeto::Cadena(format!("{}{}", i, d)),
            "==" => Objeto::Booleano(false),
            "!=" => Objeto::Booleano(true),
            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre entero y cadena", operador
            )),
        },
        (Objeto::Flotante(i), Objeto::Cadena(d)) => match operador {
            "+" => Objeto::Cadena(format!("{}{}", i, d)),
            "==" => Objeto::Booleano(false),
            "!=" => Objeto::Booleano(true),
            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre flotante y cadena", operador
            )),
        },

        // ===================================================================
        // (Booleano, Booleano) — Comparación lógica
        // ===================================================================
        (Objeto::Booleano(i), Objeto::Booleano(d)) => match operador {
            "==" => Objeto::Booleano(i == d),
            "!=" => Objeto::Booleano(i != d),

            _ => Objeto::Error(format!(
                "Operación '{}' no soportada entre booleanos", operador
            )),
        },

        // ===================================================================
        // Igualdad Universal — Tipos diferentes
        // ===================================================================
        // Cuando los tipos no coinciden (Entero vs Cadena, Flotante
        // vs Booleano, etc.), la comparación de igualdad siempre
        // retorna false, y la desigualdad retorna true. Esto replica
        // el comportamiento de lenguajes como Python o JavaScript
        // (sin coerción de tipos).
        //
        // El patrón `_` captura todas las combinaciones no cubiertas
        // por los brazos anteriores. `izquierda` y `derecha` ya fueron
        // consumidas por el match, así que el patrón `_` no las
        // vincula (no podemos acceder a sus valores).
        _ => match operador {
            // Tipos distintos nunca son iguales.
            "==" => Objeto::Booleano(false),
            // Tipos distintos siempre son diferentes.
            "!=" => Objeto::Booleano(true),

            // Cualquier otro operador entre tipos dispares es
            // un error de tipos en tiempo de ejecución.
            _ => Objeto::Error(format!(
                "Discrepancia de tipos: no se puede aplicar \
                 '{}' entre tipos distintos", operador
            )),
        },
    }
}

// ---------------------------------------------------------------------------
// evaluar_acceso_indice — Accede a un elemento por índice en una estructura
// ---------------------------------------------------------------------------
// Recibe ownership de ambos operandos. Soporta actualmente:
//   - Objeto::Arreglo  +  Objeto::Entero(i)  → clona y retorna elementos[i]
//   - Objeto::Diccionario + LlaveHash(k)      → clona y retorna mapa[&k]
//
// # Gestión de memoria (clone del elemento)
// Para retornar el elemento sin removerlo de la colección, necesitamos
// clonarlo. clone() es una copia profunda del Objeto (O(n) para tipos
// heap como Cadena, Arreglo anidado, etc.). Esto es necesario porque la
// colección retiene ownership de sus elementos; no podemos transferir la
// ownership sin mutar la colección. Una alternativa futura sería usar
// Rc<Objeto> para acceso compartido sin clonación.
//
// Parámetros:
//   estructura: Objeto — el contenedor (Arreglo o Diccionario).
//   indice: Objeto — el índice (Entero para Arreglo, cualquier LlaveHash
//     válido para Diccionario).
//
// Retorno: Objeto — el elemento clonado, Nulo si no existe, o Error.
fn evaluar_acceso_indice(estructura: Objeto, indice: Objeto) -> Objeto {
    evaluar_acceso_indice_con_profundidad(estructura, indice, 0)
}

/// Implementación interna con guardián de profundidad para prevenir
/// Stack Overflow por cadenas de prototipos cíclicas o demasiado largas.
fn evaluar_acceso_indice_con_profundidad(
    estructura: Objeto,
    indice: Objeto,
    profundidad: usize,
) -> Objeto {
    if profundidad > 64 {
        return Objeto::Error(
            "Stack Overflow preventivo: Cadena de prototipos \
             demasiado profunda o cíclica"
                .to_string(),
        );
    }
    match (estructura, indice) {
        // Acceso válido: Arreglo indexado por un entero.
        (Objeto::Arreglo(elementos), Objeto::Entero(i)) => {
            let len = elementos.len();
            // Verificar límites: índice negativo o >= longitud son inválidos.
            if i < 0 || (i as usize) >= len {
                Objeto::Error(format!(
                    "Índice fuera de rango: {} (longitud: {})", i, len
                ))
            } else {
                // Clonar el elemento en la posición i.
                // elementos[i as usize] es &Objeto; .clone() crea una
                // copia profunda independiente en heap. La ownership
                // de la copia se transfiere al llamante.
                elementos[i as usize].clone()
            }
        }
        // El índice no es un entero para arreglo.
        (Objeto::Arreglo(_), _) => {
            Objeto::Error(
                "El índice debe ser un entero".to_string()
            )
        }

        // Acceso válido: Diccionario indexado por cualquier LlaveHash.
        (Objeto::Diccionario(mapa), indice) => {
            // Convertir el objeto índice a LlaveHash. Si la conversión
            // falla (tipo no válido como clave), propagar el error.
            let llave = match indice.obtener_llave_hash() {
                Ok(k) => k,
                Err(msg) => return Objeto::Error(msg),
            };
            // Buscar la clave en el HashMap. get() retorna Option<&Objeto>.
            // Si la clave existe, clonamos el valor y lo retornamos.
            match mapa.get(&llave) {
                Some(valor) => valor.clone(),
                None => {
                    // Delegación prototípica (Prototypal Inheritance):
                    // si la clave no existe en este diccionario,
                    // buscar __proto__ y recorrer la cadena de
                    // prototipos recursivamente.
                    let clave_proto =
                        LlaveHash::Cadena("__proto__".to_string());
                    match mapa.get(&clave_proto) {
                        Some(Objeto::Diccionario(padre)) => {
                            evaluar_acceso_indice_con_profundidad(
                                Objeto::Diccionario(padre.clone()),
                                indice,
                                profundidad + 1,
                            )
                        }
                        _ => Objeto::Nulo,
                    }
                }
            }
        }

        // Acceso válido: Buffer indexado por un entero (solo lectura).
        (Objeto::Buffer(bytes), Objeto::Entero(i)) => {
            let len = bytes.len();
            if i < 0 || (i as usize) >= len {
                Objeto::Error(format!(
                    "Índice fuera de rango: {} (longitud: {})", i, len
                ))
            } else {
                Objeto::Entero(bytes[i as usize] as i64)
            }
        }
        (Objeto::Buffer(_), _) => {
            Objeto::Error(
                "El índice debe ser un entero".to_string()
            )
        }

        // La estructura no soporta acceso por índice.
        (_, _) => {
            Objeto::Error(
                "El tipo no soporta acceso por índice".to_string()
            )
        }
    }
}

// ---------------------------------------------------------------------------
// evaluar_expresion — Evalúa una expresión del AST
// ---------------------------------------------------------------------------
// Convierte un nodo Expression en un Objeto en tiempo de ejecución.
// Las expresiones literales se traducen directamente; las compuestas
// (binarias, unarias, identificadores, llamadas) se implementarán
// en fases posteriores.
//
// Parámetros:
//   expresion: &Expression — referencia inmutable al nodo del AST.
//   entorno: &mut Entorno — préstamo mutable. Aunque las expresiones
//     literales no necesitan mutar el entorno, recibimos &mut para
//     mantener una firma uniforme; expresiones futuras (identificadores,
//     asignaciones) sí requerirán mutabilidad.
//
// Retorno: Objeto — el valor evaluado. La ownership se transfiere al
//   llamante (move). Para tipos que alocan heap (Cadena), se usa clone()
//   para copiar el dato desde el AST, porque el AST retiene la ownership
//   original de los Strings que contiene.
pub fn evaluar_expresion(expresion: &Expression, entorno: &mut Entorno) -> Objeto {
    // match por referencia (&expresion). Expression es un enum que puede
    // contener Strings (heap) y Boxes (heap) en sus variantes. Tomar por
    // referencia evita mover estos datos: solo los prestamos para leerlos.
    match expresion {
        // Expression::Entero(valor) → Objeto::Entero(*valor)
        // `valor` es &i64 (referencia al i64 dentro del AST). i64 es Copy,
        // así que `*valor` desreferencia y copia el entero del stack.
        // La copia es trivial (8 bytes, sin heap). Retornamos el Objeto
        // por valor (se mueve al llamante).
        Expression::Entero(valor) => Objeto::Entero(*valor),

        // Expression::Flotante(valor) → Objeto::Flotante(*valor)
        // f64 es Copy. Misma mecánica que Entero: desreferencia (copia
        // del stack) y envuelve en Objeto.
        Expression::Flotante(valor) => Objeto::Flotante(*valor),

        // Expression::Booleano(valor) → Objeto::Booleano(*valor)
        // bool es Copy. `*valor` produce una copia bitwise (1 byte).
        Expression::Booleano(valor) => Objeto::Booleano(*valor),

        // Expression::Cadena(valor) → Objeto::Cadena(valor.clone())
        // `valor` es &String (referencia al String dentro del AST).
        // No podemos mover el String fuera del AST (está prestado como
        // &Expression). Clone() crea una copia independiente en heap:
        // aloca nuevo buffer, copia el contenido O(n). El AST retiene
        // su String original; el nuevo Objeto posee la copia.
        Expression::Cadena(valor) => Objeto::Cadena(valor.clone()),

        // Expression::Identificador(nombre) — Búsqueda de variable.
        // `nombre` es &String (referencia al nombre dentro del AST).
        // Se llama a entorno.obtener() con &str (Rust aplica deref
        // coercion de &String a &str automáticamente). obtener()
        // retorna Option<Objeto>:
        //   - Some(objeto): la variable existe, se retorna el Objeto
        //     (clonado por obtener() para no tomar ownership del
        //     almacén interno del Entorno).
        //   - None: variable no definida, se retorna Objeto::Error.
        Expression::Identificador(nombre) => {
            match entorno.obtener(nombre) {
                // Variable encontrada: retornar el valor clonado.
                // La ownership del clon se transfiere al llamante.
                Some(objeto) => objeto,
                // Variable no encontrada: construir error en heap
                // (String) y retornar Objeto::Error. Esto evita
                // panic! — el programa continúa y el error se
                // propaga hacia arriba por el short-circuit.
                None => Objeto::Error(format!(
                    "Variable no encontrada: {}",
                    nombre
                )),
            }
        }

        // Expression::OperacionBinaria { izquierda, operador, derecha } —
        // Operador binario (infijo): `1 + 2`, `a < b`, `"ho" + "la"`.
        // izquierda/derecha: &Box<Expression> (deref coercion automática).
        // operador: &Token (referencia al token del operador en el AST).
        Expression::OperacionBinaria {
            izquierda,
            operador,
            derecha,
        } => {
            // 1. Evaluar el lado izquierdo. Si es Error, propagar.
            let eval_izquierda = evaluar_expresion(izquierda, entorno);
            if let Objeto::Error(_) = &eval_izquierda { return eval_izquierda }

            // Short-circuit para && y ||: evaluar derecho solo si es necesario.
            if *operador == Token::And {
                if !es_truthy(&eval_izquierda) {
                    return eval_izquierda;
                }
                let eval_derecha = evaluar_expresion(derecha, entorno);
                if let Objeto::Error(_) = &eval_derecha { return eval_derecha }
                return eval_derecha;
            }
            if *operador == Token::Or {
                if es_truthy(&eval_izquierda) {
                    return eval_izquierda;
                }
                let eval_derecha = evaluar_expresion(derecha, entorno);
                if let Objeto::Error(_) = &eval_derecha { return eval_derecha }
                return eval_derecha;
            }

            // 2. Evaluar el lado derecho. Si es Error, propagar.
            let eval_derecha = evaluar_expresion(derecha, entorno);
            if let Objeto::Error(_) = &eval_derecha { return eval_derecha }

            // 3. Convertir el Token del operador a &str para el
            //    dispatcher. Token no implementa Display, por lo que
            //    mapeamos manualmente cada variante de operador binario.
            //    Si el operador no es ninguno de los conocidos, se
            //    retorna un Objeto::Error.
            let operador_str = match operador {
                Token::Suma => "+",
                Token::Resta => "-",
                Token::Multiplicacion => "*",
                Token::Division => "/",
                Token::Modulo => "%",
                Token::Igual => "==",
                Token::Diferente => "!=",
                Token::MenorQue => "<",
                Token::MayorQue => ">",
                Token::MenorOIgual => "<=",
                Token::MayorOIgual => ">=",
                // And/Or no se parsean actualmente, pero se incluyen
                // como previsión para expansión futura.
                Token::And => "&&",
                Token::Or => "||",
                // Bitwise
                Token::Ampersand => "&",
                Token::Pipe => "|",
                Token::Circunflejo => "^",
                Token::DesplazamientoIzq => "<<",
                Token::DesplazamientoDer => ">>",
                _ => return Objeto::Error(format!(
                    "Operador binario desconocido: {:?}", operador
                )),
            };

            // 4. Delegar en evaluar_binario, que consume ownership
            //    de ambos operandos (Objeto se mueven a la función).
            evaluar_binario(operador_str, eval_izquierda, eval_derecha)
        }

        // Expression::OperacionUnaria { operador, derecha } —
        // Operador unario (prefijo): `-expr`, `!expr`, `+expr`.
        // operador: &Token (referencia al operador dentro del AST).
        // derecha: &Box<Expression> (Rust aplica deref coercion
        // automática, se pasa como &Expression a evaluar_expresion).
        Expression::OperacionUnaria { operador, derecha } => {
            // 1. Evaluar el operando derecho recursivamente.
            //    `derecha` es &Box<Expression>; gracias a Deref,
            //    Rust lo trata como &Expression automáticamente,
            //    sin necesidad de &**derecha ni derecha.as_ref().
            let eval_derecha = evaluar_expresion(derecha, entorno);

            // 2. Short-circuit: si el operando produjo Error,
            //    propagarlo inmediatamente sin aplicar el operador.
            if let Objeto::Error(_) = &eval_derecha { return eval_derecha }

            // 3. Convertir el Token a un &str legible para el
            //    dispatcher. Token deriva Clone, pero no Display;
            //    mapeamos manualmente las tres variantes unarias.
            //    Si el parser generó un operador no unario aquí,
            //    es un bug interno — lo capturamos como error.
            let operador_str = match operador {
                Token::Suma => "+",
                Token::Resta => "-",
                Token::Not => "!",
                _ => return Objeto::Error(format!(
                    "Operador unario desconocido: {:?}", operador
                )),
            };

            // 4. Delegar en evaluar_unario, que consume ownership
            //    de eval_derecha (Objeto se mueve a la función).
            //    evaluar_unario retorna un nuevo Objeto por ownership.
            evaluar_unario(operador_str, eval_derecha)
        }

        // Expression::Llamada { funcion, argumentos } —
        // Llamada a función: `foo(1, 2)` o `identificador(args)`.
        // funcion: &Box<Expression> — expresión que se evalúa para
        //   obtener el Objeto invocable (normalmente un Identificador
        //   que resuelve a Objeto::Funcion, pero podría ser cualquier
        //   expresión que retorne una función).
        // argumentos: &Vec<Expression> — expresiones de los argumentos.
        Expression::Llamada {
            funcion,
            argumentos,
        } => {
            // 1. Evaluar la expresión de la función.
            //    `funcion` es &Box<Expression>; deref coercion lo
            //    convierte automáticamente a &Expression. La evaluación
            //    retorna un Objeto (normalmente Objeto::Funcion si la
            //    función está definida, o Error si no se encuentra).
            let eval_funcion = evaluar_expresion(funcion, entorno);

            // 2. Short-circuit: si evaluar la función falló, propagar.
            if let Objeto::Error(_) = &eval_funcion { return eval_funcion }

            // 3. Verificar que el resultado sea invocable.
            //    Puede ser Funcion (definida por el usuario) o Nativa
            //    (built-in del sistema). Cualquier otro tipo (Entero,
            //    Cadena, Nulo) no es invocable → error.
            if !matches!(eval_funcion, Objeto::Funcion { .. } | Objeto::Nativa(_)) {
                return Objeto::Error(
                    "No es una función invocable".to_string(),
                );
            }

            // 4. Evaluar cada argumento en orden.
            //    Se itera sobre &Vec<Expression>; cada elemento es
            //    &Expression. Los resultados se acumulan en un Vec<Objeto>
            //    que crece en heap dinámicamente. Si algún argumento
            //    falla, se propaga inmediatamente.
            let mut argumentos_evaluados: Vec<Objeto> = Vec::with_capacity(argumentos.len());
            for arg in argumentos {
                let eval_arg = evaluar_expresion(arg, entorno);

                // Short-circuit por argumento: si un solo argumento
                // produce Error, toda la llamada falla.
                if let Objeto::Error(_) = &eval_arg { return eval_arg }

                argumentos_evaluados.push(eval_arg);
            }

            // 5. Delegar la ejecución en evaluar_llamada_funcion.
            //    eval_funcion (Objeto::Funcion) y argumentos_evaluados
            //    se mueven a la función (ownership transferido).
            //    evaluar_llamada_funcion retorna el Objeto resultante.
            evaluar_llamada_funcion(eval_funcion, argumentos_evaluados)
        }

        // Expression::Arreglo(elementos) — Literal de arreglo [a, b, c]
        // Evalúa cada expresión elemento en orden y recolecta los
        // resultados en un Vec<Objeto>. Si algún elemento falla (Error),
        // se propaga inmediatamente (short-circuit).
        Expression::Arreglo(elementos) => {
            // Vec<Objeto> que crece en heap. Cada elemento evaluado se
            // mueve (ownership) dentro del vector.
            let mut elementos_evaluados: Vec<Objeto> = Vec::with_capacity(elementos.len());
            for expr in elementos {
                let eval = evaluar_expresion(expr, entorno);
                // Short-circuit: si un elemento es Error, toda la
                // evaluación del arreglo falla.
                if let Objeto::Error(_) = &eval { return eval }
                elementos_evaluados.push(eval);
            }
            // Envolver el Vec en Objeto::Arreglo. La ownership del Vec
            // (y de todos sus Objetos internos) se transfiere al llamante.
            Objeto::Arreglo(elementos_evaluados)
        }

        // Expression::Diccionario(pares) — Literal de diccionario {k: v, ...}
        // Evalúa cada par (clave, valor) en orden. Para cada par:
        //   1. Evalúa la expresión de la clave.
        //   2. Convierte el resultado a LlaveHash vía obtener_llave_hash().
        //   3. Evalúa la expresión del valor.
        //   4. Inserta en el HashMap.
        // Si algún paso falla (Error), se propaga inmediatamente.
        Expression::Diccionario(pares) => {
            // HashMap<LlaveHash, Objeto> que crece dinámicamente en heap.
            // La capacidad inicial se deja por defecto (HashMap::new()).
            let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::with_capacity(pares.len());
            for (clave_expr, valor_expr) in pares {
                // 1. Evaluar la expresión de la clave.
                let clave_eval = evaluar_expresion(clave_expr, entorno);
                if let Objeto::Error(_) = &clave_eval { return clave_eval }

                // 2. Convertir a LlaveHash. Si falla, propagar error.
                let llave = match clave_eval.tomar_llave_hash() {
                    Ok(k) => k,
                    Err(msg) => return Objeto::Error(msg),
                };

                // 3. Evaluar la expresión del valor.
                let valor = evaluar_expresion(valor_expr, entorno);
                if let Objeto::Error(_) = &valor { return valor }

                // 4. Insertar en el HashMap. Si la clave ya existe, se
                //    sobrescribe (comportamiento estándar en lenguajes
                //    dinámicos como JS/ Python).
                mapa.insert(llave, valor);
            }
            Objeto::Diccionario(mapa)
        }

        // Expression::AccesoIndice { izquierda, indice } —
        // Acceso por índice: `arreglo[expr]`
        // Evalúa la expresión izquierda (el arreglo) y la expresión del
        // índice, luego delega en evaluar_acceso_indice para la lógica
        // de acceso (verificación de límites, clonación del elemento).
        Expression::AccesoIndice { izquierda, indice } => {
            // 1. Evaluar la expresión izquierda (el contenedor).
            let eval_izquierda = evaluar_expresion(izquierda, entorno);
            if let Objeto::Error(_) = &eval_izquierda { return eval_izquierda }

            // 2. Evaluar la expresión del índice.
            let eval_indice = evaluar_expresion(indice, entorno);
            if let Objeto::Error(_) = &eval_indice { return eval_indice }

            // 3. Delegar en evaluar_acceso_indice, que consume la
            //    ownership de ambos objetos y retorna el resultado.
            evaluar_acceso_indice(eval_izquierda, eval_indice)
        }

        // Expression::Funcion { parametros, cuerpo } — Función anónima.
        // Evalúa `fn(params) { cuerpo }` como expresión, retornando
        // un Objeto::Funcion con el entorno actual capturado (closure).
        Expression::Funcion {
            parametros,
            cuerpo,
        } => {
            // Capturar el entorno actual (cierre léxico). La clonación
            // copia el HashMap y la cadena de padres. Esta copia queda
            // "congelada" dentro del Objeto::Funcion y se usará como
            // ámbito padre cuando la función se invoque.
            let entorno_capturado = entorno.clone();
            Objeto::Funcion {
                parametros: parametros.clone(),
                cuerpo: cuerpo.clone(),
                entorno: entorno_capturado,
            }
        }

        // Expression::Import(ruta) — Importación de módulo.
        // Evalúa `import "archivo.argo"`: resuelve el módulo (vía
        // sistema de caché de dos niveles: .argbc binario → .argo
        // textual → descarga para URLs), retornando el AST ya
        // parseado/deserializado listo para ejecutar.
        Expression::Import(ruta) => {
            // a. Resolver y compilar el módulo. importar() maneja
            //    todo el ciclo: caché binaria (.argbc), caché textual
            //    (.argo), descarga (URLs), y parseo. Retorna
            //    Vec<Statement> sin necesidad de Lexer ni Parser.
            let sentencias = match importar(ruta) {
                Ok(s) => s,
                Err(e) => return Objeto::Error(format!(
                    "No se pudo importar: {}", e
                )),
            };
            let programa_modulo = Programa { sentencias };

            // d. Crear un entorno global NUEVO y AISLADO para el
            //    módulo. El módulo NO hereda las variables del
            //    llamante: solo tiene acceso a las funciones
            //    nativas (print, len, push, tipo) y a sus propias
            //    declaraciones. Esto evita contaminación cruzada
            //    entre módulos (principio de aislamiento).
            let mut entorno_modulo = configurar_entorno_global();

            // e. Evaluar el programa del módulo en el entorno
            //    aislado. Si produce Error, propagarlo.
            let resultado_modulo = evaluar_programa(
                &programa_modulo, &mut entorno_modulo,
            );
            if let Objeto::Error(_) = &resultado_modulo { return resultado_modulo }

            // f. Extraer el HashMap interno del entorno del módulo
            //    para construir un diccionario de Argo. Recorremos
            //    el almacén del entorno y convertimos cada par
            //    (String, Objeto) → (LlaveHash, Objeto). La
            //    conversión usa LlaveHash::Cadena para las claves.
            //
            //    Esto permite al llamante acceder a las variables
            //    del módulo con sintaxis de punto:
            //      let mates = import "math.argo";
            //      mates.sumar(1, 2);
            //
            //    Se filtran las funciones nativas (print, len, push, tipo)
            //    para que no contaminen el espacio de nombres del módulo.
            //    Solo las variables explícitamente declaradas con `let`
            //    en el módulo se exportan al llamante.
            let nativas: [&str; 6] = ["print", "len", "push", "tipo", "entero", "cadena"];
            let mapa_modulo: HashMap<LlaveHash, Objeto> = entorno_modulo
                .exportar()
                .into_iter()
                .filter(|(k, _)| !nativas.contains(&k.as_str()))
                .map(|(k, v)| (LlaveHash::Cadena(k), v))
                .collect();

            // g. Retornar el diccionario del módulo. El llamante
            //    puede indexarlo con notación de punto o corchetes.
            Objeto::Diccionario(mapa_modulo)
        }
    }
}
