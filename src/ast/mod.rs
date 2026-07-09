
use crate::lexer::token::Token;

#[derive(Debug, Clone)]
pub struct Programa {
    pub sentencias: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    DeclaracionVariable { nombre: String, valor: Expression, constante: bool },
    Bloque(Vec<Statement>),
    If {
        condicion: Expression,
        consecuencia: Box<Statement>,
        alternativa: Option<Box<Statement>>,
    },
    While {
        condicion: Expression,
        cuerpo: Box<Statement>,
    },
    For {
        inicializacion: Box<Statement>,
        condicion: Expression,
        actualizacion: Box<Statement>,
        cuerpo: Box<Statement>,
    },
    Return(Option<Expression>),
    TryCatch {
        bloque_try: Box<Statement>,
        parametro_catch: String,
        bloque_catch: Box<Statement>,
    },
    Break,
    Continue,
    Expresion(Expression),
    AsignacionVariable { nombre: String, valor: Expression },
    DeclaracionFuncion {
        nombre: String,
        parametros: Vec<String>,
        cuerpo: Box<Statement>,
        #[allow(dead_code)]
        tipos_parametros: Vec<Option<String>>,
        #[allow(dead_code)]
        tipo_retorno: Option<String>,
    },
    DeclaracionStruct {
        nombre: String,
        campos: Vec<String>,
    },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Expression {
    Entero(i64),
    Flotante(f64),
    Booleano(bool),
    Cadena(String),
    Identificador(String),
    OperacionBinaria {
        izquierda: Box<Expression>,
        operador: Token,
        derecha: Box<Expression>,
    },
    OperacionUnaria {
        operador: Token,
        derecha: Box<Expression>,
    },
    Llamada {
        funcion: Box<Expression>,
        argumentos: Vec<Expression>,
    },
    Arreglo(Vec<Expression>),
    Diccionario(Vec<(Expression, Expression)>),
    AccesoIndice {
        izquierda: Box<Expression>,
        indice: Box<Expression>,
    },

    /// Función anónima (expresión): `fn(params) { cuerpo }`
    /// Se usa en expresiones como `let f = fn(x) { return x + 1; };`.
    /// El cuerpo es un Bloque (Statement), almacenado en Box para que
    /// Expression tenga tamaño fijo (el enum no contiene recursión infinita).
    Funcion {
        parametros: Vec<String>,
        cuerpo: Box<Statement>,
        #[allow(dead_code)]
        tipos_parametros: Vec<Option<String>>,
        #[allow(dead_code)]
        tipo_retorno: Option<String>,
    },

    /// Importación de módulo: `import "ruta/al/archivo.argo"`
    /// Evalúa el archivo en un entorno aislado y retorna un diccionario
    /// con las variables globales exportadas.
    Import(String),

    /// Importación selectiva: `import { foo, bar } from "modulo"`
    /// Extrae solo las variables nombradas del módulo importado.
    ImportSelectivo {
        nombres: Vec<String>,
        modulo: String,
    },

    /// Match expression: `match (expr) { Patron => Expr, ... }`
    Match {
        expr: Box<Expression>,
        brazos: Vec<(Patron, Expression)>,
    },

    /// Instancia de struct: `Punto { x: 1, y: 2 }`
    StructInstancia {
        nombre: String,
        valores: Vec<(String, Expression)>,
    },

    /// Throw expression: `throw <expr>`
    /// Evalúa la expresión y retorna un error (excepción) con ese valor.
    Throw(Box<Expression>),
}

/// Patrón para match: literales, wildcard `_`, binding, struct/array
#[derive(Debug, Clone)]
pub enum Patron {
    Literal(Expression),
    Wildcard,
    Binding(String),
    Struct(String, Vec<(String, Box<Patron>)>),
    Arreglo(Vec<Patron>),
}
