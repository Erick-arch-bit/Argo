// Sistema de tipos en tiempo de ejecución (Runtime Type System). Cada valor// "vivo" durante la ejecución de Argo se representa como una variante del
// enum `Objeto`. Las variantes que almacenan datos en heap (String, Box)
// requieren una gestión explícita de ownership para que el borrow checker
// de Rust apruebe el programa en tiempo de compilación.

use std::collections::HashMap;
use std::fmt;

// Importaciones necesarias para Objeto::Funcion:
// Statement (nodo AST del cuerpo) y Entorno (entorno capturado
// para closures). Ambas son tipos del propio proyecto Argo, sin
// dependencias externas. La ruta completa evita ambigüedades.
use crate::ast::Statement;
use crate::evaluator::Entorno;

// ---------------------------------------------------------------------------
// LlaveHash — Tipos de datos utilizables como claves de diccionario
// ---------------------------------------------------------------------------
// No todos los Objetos pueden ser claves: solo Cadena, Entero y Booleano
// son válidos (tipos inmutables con igualdad e hash estables). Flotante
// no se incluye porque NaN != NaN y f64 no implementa Hash en std.
//
// Derivamos Eq, Hash para poder usar LlaveHash como clave de HashMap.
// PartialEq y Eq son necesarios para la igualdad de claves.
// Ord no es necesario para HashMap (solo para BTreeMap).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LlaveHash {
    Cadena(String),
    Entero(i64),
    Booleano(bool),
}

impl fmt::Display for LlaveHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LlaveHash::Cadena(s) => write!(f, "\"{}\"", s),
            LlaveHash::Entero(n) => write!(f, "{}", n),
            LlaveHash::Booleano(b) => write!(f, "{}", b),
        }
    }
}

// ---------------------------------------------------------------------------
// Objeto — Representación polimórfica de todo valor en tiempo de ejecución
// ---------------------------------------------------------------------------
// El enum `Objeto` usa un discriminante (8 bytes en 64 bits) más el espacio
// de la variante más grande. La variante `Cadena(String)` almacena su
// contenido en el heap; el enum solo guarda el puntero (24 bytes: 8 ptr +
// 8 len + 8 cap). La variante `Retorno` usa `Box<Objeto>`, que es un puntero
// inteligente de 8 bytes que apunta al heap. Sin Box, el enum tendría
// tamaño infinito por recursión (`Retorno(Objeto)` → contiene otro Objeto).
//
// Clone se deriva automáticamente; Debug se implementa manualmente porque
// la variante Nativa(fn(...)) contiene un puntero a función (fn pointer)
// que no implementa Debug en Rust.
#[derive(Clone)]
pub enum Objeto {
    // Entero(i64) — i64 es un tipo "Copy" (todo en el stack, 8 bytes).
    // No requiere gestión de heap. Al asignarlo o pasarlo por valor, Rust
    // hace una copia bitwise (memcpy) sin necesidad de clone() explícito.
    Entero(i64),

    // Flotante(f64) — f64 también es Copy (stack, 8 bytes). Misma mecánica
    // que Entero: la ownership se transfiere por copia bitwise implícita.
    Flotante(f64),

    // Booleano(bool) — bool es Copy (stack, 1 byte). Al moverlo, Rust
    // copia el byte sin heap allocation.
    Booleano(bool),

    // Cadena(String) — String es un struct de 24 bytes en el stack (ptr,
    // len, cap) que posee un buffer de bytes en el heap. Cuando se mueve
    // (ownership transfer), Rust copia los 24 bytes del stack pero NO el
    // contenido del heap; el nuevo owner es responsable de liberarlo al
    // salir de scope. Clone() sí copia el heap (allocación O(n)). Drop()
    // libera el heap automáticamente cuando el owner sale de ámbito.
    Cadena(String),

    // Nulo — Representa la ausencia de valor (null/nil). Sin datos internos.
    // Es un enum sin payload; solo ocupa el espacio del discriminante.
    // No requiere heap allocation ni gestión de ownership especial.
    Nulo,

    // Retorno(Box<Objeto>) — Envuelve un valor para propagarlo a través de
    // sentencias `return` anidadas. Box<Objeto> es un puntero inteligente
    // de 8 bytes (en 64 bits) que posee un Objeto en el heap.
    //
    // Por qué Box es necesario aquí:
    //   Si definiéramos `Retorno(Objeto)`, el enum se auto-referenciaría
    //   (Objeto contiene Retorno que contiene Objeto...) y Rust lo
    //   rechazaría por tener tamaño infinito. Box rompe la recursión al
    //   colocar el Objeto interno en el heap; el enum solo almacena el
    //   puntero. La ownership del heap se transfiere al Box; cuando el Box
    //   se dropea, llama a Drop sobre el Objeto interno y libera la memoria.
    Retorno(Box<Objeto>),

    // Break — Señal de control para salir de un bucle (break).
    // Similar a Retorno pero sin valor asociado. El bucle que lo
    // recibe interrumpe su iteración y retorna Objeto::Nulo al
    // ámbito exterior (el Break se consume, no se propaga).
    Break,

    // Error(String) — Representa un error en tiempo de ejecución.
    // String aloca en el heap (misma mecánica que Cadena). El mensaje
    // de error vive en heap y se libera cuando el Objeto::Error se dropea.
    // Esta variante permite manejar errores sin usar panic! ni unwrap().
    Error(String),

    // Nativa(fn(Vec<Objeto>) -> Objeto) — Función nativa del sistema
    // (built-in) implementada directamente en Rust. Almacena un puntero
    // a función (fn pointer, 8 bytes en 64 bits). Los fn pointers son
    // Copy y Clone, pero NO implementan Debug. La ausencia de Debug
    // obliga a implementar Display manualmente (no podemos derivar
    // Debug completo; usamos un enfoque manual o el derive omitirá
    // esta variante si no implementa Debug).
    //
    // fn(Vec<Objeto>) -> Objeto recibe ownership de los argumentos y
    // retorna ownership del resultado. No hay entorno capturado: las
    // nativas son funciones Rust puras, sin estado interno.
    Nativa(fn(Vec<Objeto>) -> Objeto),

    // -----------------------------------------------------------------------
    // Arreglo(Vec<Objeto>) — Colección ordenada de valores
    // -----------------------------------------------------------------------
    // Representa un arreglo dinámico en tiempo de ejecución. Vec<Objeto>
    // aloca sus elementos en el heap; cada Objeto dentro del Vec es un
    // enum con ownership independiente. Cuando el Arreglo se dropea, todos
    // sus elementos se liberan recursivamente.
    //
    // Clone sobre Vec<Objeto> es O(n): itera sobre cada elemento, clonando
    // cada Objeto (copia profunda). Para Cadena/String dentro de un Objeto,
    // esto implica allocaciones heap para cada una. Para arreglos grandes,
    // el costo de clonación puede ser significativo.
    //
    // El acceso por índice (obtener un elemento) requiere clone() del
    // Objeto interno porque no podemos dar ownership del elemento sin
    // removerlo del Vec (el Vec debe permanecer intacto para futuros
    // accesos). Clone es la alternativa segura sin Rc/Arc.
    Arreglo(Vec<Objeto>),

    // -----------------------------------------------------------------------
    // Diccionario(HashMap<LlaveHash, Objeto>) — Tabla hash clave→valor
    // -----------------------------------------------------------------------
    // Representa un diccionario (también llamado mapa, tabla hash u objeto)
    // en tiempo de ejecución. HashMap<LlaveHash, Objeto> almacena cada
    // par en el heap; la tabla hash en sí misma es una estructura contigua
    // (el arreglo interno del HashMap) que referencia las entradas en heap.
    //
    // LlaveHash solo admite Cadena, Entero y Booleano como claves. Esto
    // evita problemas con tipos que no implementan Hash (f64) o que son
    // mutables (Arreglo, Diccionario).
    //
    // Clone sobre HashMap<K,V> es O(n_entries): itera sobre cada entrada,
    // clonando la clave (LlaveHash: String/Copy) y el valor (Objeto).
    // Para diccionarios grandes, el costo de clonación puede ser alto.
    Diccionario(HashMap<LlaveHash, Objeto>),

    // Funcion { parametros, cuerpo, entorno } — Valor de función
    // definida por el usuario (cierre léxico / closure).
    //
    // parametros: Vec<String> — nombres de los parámetros formales.
    //   Vec<String> almacena sus Strings en heap (cada String: ptr,
    //   len, cap en stack, buffer en heap). Clone copia el Vec
    //   entry por entry (O(n) allocaciones heap).
    //
    // cuerpo: Box<crate::ast::Statement> — el cuerpo de la función
    //   (típicamente un Bloque { ... }). Box<Statement> coloca el
    //   AST del cuerpo en el heap (8 bytes en el enum). Clone sobre
    //   Box<Statement> clona recursivamente el AST (árbol sintáctico
    //   completo). Esto permite reutilizar la función múltiples veces
    //   sin compartir el AST original.
    //
    // entorno: Entorno — entorno capturado en el momento de la
    //   definición. Almacena todas las variables visibles cuando se
    //   declaró la función. Cuando la función se invoca, este entorno
    //   se usa como padre del nuevo ámbito de llamada, implementando
    //   así el cierre léxico (closure): la función "recuerda" las
    //   variables de su contexto de definición.
    //
    //   Entorno contiene un HashMap<String, Objeto> (heap) y un
    //   Option<Box<Entorno>>. Clone sobre Entorno es una copia
    //   profunda de toda la cadena de ámbitos (O(n_vars * prof)).
    //   Esto es necesario porque el entorno capturado debe ser
    //   independiente del entorno de ejecución actual: si el código
    //   mutable modifica el entorno original, la closure no debe
    //   verse afectada (inmutabilidad del capture).
    Funcion {
        parametros: Vec<String>,
        cuerpo: Box<Statement>,
        entorno: Entorno,
    },
}

// ---------------------------------------------------------------------------
// obtener_llave_hash — Convierte un Objeto en una LlaveHash válida
// ---------------------------------------------------------------------------
// Retorna Ok(LlaveHash) si el Objeto es de un tipo que puede usarse como
// clave de diccionario (Cadena, Entero, Booleano). Retorna Err(String)
// con un mensaje descriptivo para tipos no válidos (Flotante, Arreglo,
// Funcion, etc.).
//
// # Gestión de memoria
// Para Objeto::Cadena, clonamos el String interno. Clone aloca en heap
// una copia del contenido textual (O(n)). Entero y Booleano son Copy,
// se convierten sin allocación.
impl Objeto {
    pub fn obtener_llave_hash(&self) -> Result<LlaveHash, String> {
        match self {
            Objeto::Cadena(s) => Ok(LlaveHash::Cadena(s.clone())),
            Objeto::Entero(n) => Ok(LlaveHash::Entero(*n)),
            Objeto::Booleano(b) => Ok(LlaveHash::Booleano(*b)),
            Objeto::Flotante(_) => Err("Las claves de diccionario no pueden ser flotantes".to_string()),
            Objeto::Arreglo(_) => Err("Las claves de diccionario no pueden ser arreglos".to_string()),
            Objeto::Diccionario(_) => Err("Las claves de diccionario no pueden ser diccionarios".to_string()),
            Objeto::Funcion { .. } | Objeto::Nativa(_) => {
                Err("Las claves de diccionario no pueden ser funciones".to_string())
            }
            Objeto::Nulo => Err("Las claves de diccionario no pueden ser nulo".to_string()),
            Objeto::Retorno(_) => Err("Las claves de diccionario no pueden ser retornos".to_string()),
            Objeto::Break => Err("Las claves de diccionario no pueden ser break".to_string()),
            Objeto::Error(_) => Err("Las claves de diccionario no pueden ser errores".to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Display — Representación textual para consola/STDOUT
// ---------------------------------------------------------------------------
// El trait std::fmt::Display controla cómo se imprime el valor con {}.
// A diferencia de Debug ({:?}), Display produce una salida "bonita"
// pensada para el usuario final del lenguaje Argo.
impl fmt::Display for Objeto {
    // `&self` es una referencia inmutable al Objeto. No toma ownership;
    // solo presta (borrow) el enum para leerlo. El borrow checker verifica
    // que no haya referencias mutables simultáneas a este Objeto mientras
    // `self` está prestado inmutablemente.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // match por referencia (&self) para no mover (move) el valor.
        // Si hiciera `match self` (por valor), la ownership del Objeto
        // se transferiría al match y no podría usarse después. Como solo
        // necesitamos leer el discriminante y los datos, el borrow es
        // suficiente y más eficiente: evita un clone() innecesario.
        match self {
            // Entero: escribe el número. i64 implementa Display, así que
            // delegamos en su implementación. No hay heap aquí.
            Objeto::Entero(valor) => write!(f, "{}", valor),

            // Flotante: igual, f64 implementa Display. Se imprime con
            // la menor cantidad de decimales necesaria (formato nativo).
            Objeto::Flotante(valor) => write!(f, "{}", valor),

            // Booleano: true o false. Display de bool es nativo.
            Objeto::Booleano(valor) => write!(f, "{}", valor),

            // Cadena: accedemos al String por referencia (&String).
            // No clonamos ni movemos el contenido. El formato {} sobre
            // String imprime el contenido textual sin comillas.
            Objeto::Cadena(valor) => write!(f, "{}", valor),

            // Nulo: literal "null", igual que en JavaScript.
            Objeto::Nulo => write!(f, "null"),

            // Retorno: desempaqueta (unbox) el Box con `**` o con
            // autoref/deref. `&objeto` es &Objeto (referencia al heap).
            // Luego delegamos en el Display del Objeto interno.
            // Box implementa Deref, por lo que `&**objeto` no es necesario;
            // Rust aplica deref coercion automática en el match.
            Objeto::Retorno(objeto) => {
                // objeto es &Box<Objeto>. Gracias a Deref, Rust lo trata
                // como &Objeto automáticamente. Pasamos la referencia al
                // write! de abajo, que espera algún tipo que implemente
                // Display (Objeto lo implementa). La sintaxis es la
                // misma que para el formato básico: write!(f, "{}", ...).
                write!(f, "{}", objeto)
            }

            // Break: no imprime nada (es una señal de control interna).
            Objeto::Break => Ok(()),

            // Error: imprime el mensaje con prefijo "error: " para
            // distinguirlo visualmente de otros valores. El String
            // interno se muestra por referencia (no se mueve ni clona).
            Objeto::Error(mensaje) => write!(f, "error: {}", mensaje),

            // Nativa: función built-in del sistema. No tiene nombre
            // asociado (es un puntero anónimo). Se imprime un texto
            // descriptivo genérico.
            Objeto::Nativa(_) => {
                write!(f, "[Función nativa del sistema]")
            }

            // Arreglo: imprime los elementos separados por coma y
            // encerrados entre corchetes, estilo JSON/JS.
            // Iteramos por referencia (&Objeto) sobre el Vec para no
            // mover ni clonar los elementos. Cada elemento se formatea
            // con su propio Display ({}), que para Objetos anidados
            // (ej. arreglos dentro de arreglos) se imprime recursivamente.
            Objeto::Arreglo(elementos) => {
                let strs: Vec<String> = elementos
                    .iter()
                    .map(|e| format!("{}", e))
                    .collect();
                write!(f, "[{}]", strs.join(", "))
            }

            // Diccionario: imprime los pares clave:valor separados por coma
            // y encerrados entre llaves, estilo JSON. Se itera sobre el
            // HashMap. No hay orden garantizado (HashMap no ordena las
            // entradas). Para un orden determinista, usaríamos BTreeMap,
            // pero eso requiere Ord en LlaveHash. La iteración devuelve
            // referencias (&LlaveHash, &Objeto); formateamos cada una con
            // su Display ({}) sin clonar.
            Objeto::Diccionario(mapa) => {
                let strs: Vec<String> = mapa
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{{}}}", strs.join(", "))
            }

            // Funcion: mostrar un resumen informativo. No se imprime
            // el contenido del cuerpo ni del entorno capturado porque
            // sería excesivamente verboso. &self.parametros presta el
            // Vec<String> sin clonarlo; {:?} usa Debug para mostrar
            // la lista de parámetros (["a", "b", ...]).
            Objeto::Funcion {
                parametros,
                cuerpo: _cuerpo,
                entorno: _entorno,
            } => {
                // Se usa Debug para imprimir la lista de parámetros.
                // El cuerpo y entorno se omiten (no son relevantes
                // para la consola del usuario). El Pattern Matching
                // con _ descarta los campos no usados; Rust no los
                // mueve porque estamos en un match por referencia (&self).
                write!(f, "[Función definida por el usuario]({})",
                    format!("{:?}", parametros)
                        .trim_start_matches('[')
                        .trim_end_matches(']'))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Debug — Implementación manual (no derivada) para Objeto
// ---------------------------------------------------------------------------
// Necesitamos Debug para el short-circuit con match en el evaluador
// (a veces se usa {:?} para depuración). Implementamos Debug
// manualmente porque la variante Nativa(fn(...)) contiene un puntero
// a función que no implementa Debug.
impl fmt::Debug for Objeto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Para todas las variantes excepto Nativa, delegamos en Display
        // ({}) que ya produce salida legible. Nativa requiere manejo
        // especial porque fn pointer no es debugeable.
        match self {
            // Nativa: el puntero a función no es debugeable directamente.
            // Mostramos un texto fijo. No podemos mostrar la dirección
            // de memoria sin Pointer (que fn pointer sí implementa, pero
            // Pointer ≠ Debug). Usar transmute para extraer la dirección
            // requeriría unsafe, que está prohibido en Argo.
            Objeto::Nativa(_) => write!(f, "Nativa([Función nativa del sistema])"),
            // El resto de variantes delega en Display (el formato {}).
            // Esto es más consistente que el Debug derivado que mostraría
            // los nombres de las variantes y sus campos internos.
            _ => fmt::Display::fmt(self, f),
        }
    }
}
