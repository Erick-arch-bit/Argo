#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum Token {
    // --- Palabras Clave (Keywords) ---
    Let,
    Const,
    Fn,
    Class,
    Constructor,
    Super,
    This,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
    Return,
    Try,
    Catch,
    Import,
    Struct,
    Match,
    Throw,
    Enum,
    Extern,
    Async,
    Await,

    // --- Tipos Primitivos (Palabras Clave de Valores) ---
    True,
    False,
    Null,

    // --- Literales (Valores Dinámicos) ---
    Identificador(String), // ej. nombre_variable, mi_funcion
    Entero(i64),           // ej. 42, 0, -100
    Flotante(f64),         // ej. 3.14, -0.001
    Cadena(String),        // ej. "Hola Mundo"

    // --- Operadores Aritméticos ---
    Suma,           // +
    Resta,          // -
    Multiplicacion, // *
    Division,       // /
    Modulo,         // %

    // --- Operadores de Asignación ---
    Asignacion,           // =
    SumaAsignacion,       // +=
    RestaAsignacion,      // -=
    MultiplicacionAsignacion, // *=
    DivisionAsignacion,   // /=

    // --- Operadores de Comparación y Lógicos ---
    Igual,          // ==
    Diferente,      // !=
    MenorQue,       // <
    MayorQue,       // >
    MenorOIgual,    // <=
    MayorOIgual,    // >=
    And,            // &&
    Or,             // ||
    Not,            // !

    // --- Operadores a Nivel de Bits (Bitwise) ---
    Ampersand,          // &
    Pipe,               // |
    Circunflejo,        // ^
    DesplazamientoIzq,  // <<
    DesplazamientoDer,  // >>

    // --- Símbolos y Estructuras ---
    Punto,               // .
    Coma,                // ,
    PuntoComa,           // ;
    DosPuntos,           // :
    DobleDosPuntos,      // ::
    LlaveAbierta,        // {
    LlaveCerrada,        // }
    CorcheteAbierto,     // [
    CorcheteCerrado,     // ]
    ParentesisAbierto,   // (
    ParentesisCerrado,   // )
    
    // --- Operadores Modernos (Estilo JS/Dart) ---
    Flecha,              // =>
    AccesoSeguro,        // ?.
    FusionNula,          // ??

    // --- Control de Flujo Interno ---
    FinDeArchivo,   // Indica que el escáner terminó de leer el script (EOF)
    Ilegal(char),   // Encapsula caracteres desconocidos para manejar errores sin "crashear"
}