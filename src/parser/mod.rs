#![allow(
    clippy::doc_lazy_continuation,
    clippy::doc_overindented_list_items
)]
// Analizador sintáctico descendente recursivo con núcleo Pratt
// para el parseo de expresiones. El parser consume tokens del
// Lexer secuencialmente y produce un AST (Programa → Statement → Expression).

use crate::ast::{Expression, Patron, Programa, Statement};
use crate::lexer::token::Token;
use crate::lexer::Lexer;

// ---------------------------------------------------------------------------
// Precedencia — niveles del algoritmo Pratt
// ---------------------------------------------------------------------------
// Se ordenan de menor a mayor prioridad de ligadura. El orden de definición
// de las variantes coincide con la precedencia ascendente gracias a los
// derives PartialOrd/Ord, que comparan por posición del discriminante.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
#[allow(dead_code)]
pub enum Precedencia {
    Menor,           // Límite base: entrada del bucle Pratt
    BitOr,           // |
    BitXor,          // ^
    BitAnd,          // &
    Igualdad,        // ==  !=
    Comparacion,     // <  >  <=  >=
    Shift,           // <<  >>
    Suma,            // +  -
    Multiplicacion,  // *  /  %
    Prefijo,         // !  - (unario) — reservado para futuros operadores
    Llamada,         // ( ) — llamada a función
    Indice,          // [ ] — acceso por índice (máxima precedencia)
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------
// Mantiene dos tokens de lookahead (token_actual, token_siguiente) para
// poder decidir sobre producciones LL(1) sin retroceder. Los errores
// sintácticos se acumulan en `errores` sin interrumpir el parseo.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    // Token que se está evaluando actualmente. Se actualiza llamando a
    // `self.avanzar()`. Su ownership pertenece al Parser; se clona cuando
    // es necesario retener información (ej. el String de un Identificador).
    pub token_actual: Token,
    // Token de un paso de adelanto (lookahead). Permite decidir qué
    // producción usar sin consumir el token. Se convierte en token_actual
    // tras cada llamada a avanzar().
    pub token_siguiente: Token,
    // Posición (línea, columna) del token_actual en el código fuente.
    // Se actualiza en cada llamada a avanzar() para que los errores
    // sintácticos reporten la ubicación exacta del fallo.
    pub token_linea: usize,
    pub token_columna: usize,
    // Posición del token_siguiente (lookahead). Se usa en
    // error_token_esperado para reportar la ubicación del token
    // que causó la falla de predicción anticipada.
    siguiente_linea: usize,
    siguiente_columna: usize,
    // Vector de errores sintácticos. Cada error es un String descriptivo.
    // Nunca se usa panic!; los errores se recolectan aquí y el parseo
    // continúa en modo tolerante (error recovery básico).
    pub errores: Vec<String>,
}

impl<'a> Parser<'a> {
    // -----------------------------------------------------------------------
    // Constructor y avance
    // -----------------------------------------------------------------------

    /// Inicializa el parser consumiendo los dos primeros tokens del Lexer.
    ///
    /// # Gestión de memoria
    /// - `lexer` se mueve (ownership transfer) al Parser; no se clona.
    /// - `token_actual` y `token_siguiente` se inicializan con el primer y
    ///   segundo token respectivamente. `unwrap_or(Token::FinDeArchivo)`
    ///   protege contra fuentes vacías: si el Lexer no produce tokens, ambos
    ///   campos quedan en FinDeArchivo y el bucle principal no itera.
    /// - `errores` se inicializa como Vec vacío; crece dinámicamente en el
    ///   heap según se encuentren errores.
    pub fn nuevo(lexer: Lexer<'a>) -> Self {
        // Se construye el parser con valores placeholder (FinDeArchivo) para
        // todos los campos. Luego se sobrescriben token_actual y token_siguiente
        // con los tokens reales del Lexer.
        let mut parser = Parser {
            lexer,                                          // move del Lexer
            token_actual: Token::FinDeArchivo,               // placeholder
            token_siguiente: Token::FinDeArchivo,            // placeholder
            token_linea: 1,                                  // placeholder
            token_columna: 1,                                // placeholder
            siguiente_linea: 1,                              // placeholder
            siguiente_columna: 1,                            // placeholder
            errores: Vec::new(),                             // sin capacidad inicial
        };
        // next() retorna Option<Token>. Si el Lexer está vacío (None), se
        // asigna FinDeArchivo para que el bucle principal termine de inmediato.
        // Cada vez que se lee un token, se captura la posición de inicio
        // registrada por el Lexer (token_linea/token_columna del Lexer).
        parser.token_actual = parser.lexer.next().unwrap_or(Token::FinDeArchivo);
        parser.token_linea = parser.lexer.token_linea;
        parser.token_columna = parser.lexer.token_columna;
        parser.token_siguiente = parser.lexer.next().unwrap_or(Token::FinDeArchivo);
        parser.siguiente_linea = parser.lexer.token_linea;
        parser.siguiente_columna = parser.lexer.token_columna;
        parser
    }

    /// Consume el token_siguiente como nuevo token_actual y lee el siguiente
    /// token del Lexer en token_siguiente.
    ///
    /// # Gestión de memoria
    /// - `std::mem::replace` intercambia los valores sin clonar: extrae el
    ///   valor de `token_siguiente` (moviéndolo a `token_actual`) y lo
    ///   reemplaza con el nuevo token del Lexer. Esto evita una clonación
    ///   innecesaria de Token, que podría implicar copiar Strings internos.
    /// - `unwrap_or(Token::FinDeArchivo)` maneja el caso donde el Lexer
    ///   ha terminado (next() devuelve None), asegurando que el parser
    ///   nunca se encuentre con un estado inválido.
    ///
    /// # Rastreo de posición
    /// Las coordenadas de token_siguiente se trasladan a token_actual, y
    /// luego se lee un nuevo token del Lexer, capturando su posición de
    /// inicio desde lexer.token_linea / lexer.token_columna. Esto asegura
    /// que token_linea/token_columna siempre reflejen la posición del
    /// token_actual que el parser está procesando.
    fn avanzar(&mut self) -> Token {
        self.token_linea = self.siguiente_linea;
        self.token_columna = self.siguiente_columna;
        let anterior = std::mem::replace(
            &mut self.token_actual,
            std::mem::replace(
                &mut self.token_siguiente,
                self.lexer.next().unwrap_or(Token::FinDeArchivo),
            ),
        );
        self.siguiente_linea = self.lexer.token_linea;
        self.siguiente_columna = self.lexer.token_columna;
        anterior
    }

    // -----------------------------------------------------------------------
    // Programa y sentencias
    // -----------------------------------------------------------------------

    /// Bucle principal del análisis sintáctico.
    ///
    /// Itera consumiendo tokens hasta encontrar FinDeArchivo. Cada sentencia
    /// válida se agrega al vector `sentencias`. Los tokens que no corresponden
    /// a ninguna producción conocida se consumen silenciosamente (retorno None
    /// de parsear_sentencia + avanzar() al final del bucle).
    ///
    /// # Control de flujo
    /// El bucle `while self.token_actual != Token::FinDeArchivo` garantiza
    /// que al encontrar el fin del archivo se sale sin consumir nada más.
    /// `self.avanzar()` al final de cada iteración mueve el parser al siguiente
    /// token. Si parsear_sentencia retorna None, el token actual se descarta
    /// y se continúa con el siguiente.
    ///
    /// # Retorno
    /// Devuelve un `Programa` con ownership transferido al llamante. El vector
    /// `sentencias` contiene solo las sentencias parseadas exitosamente.
    /// Los errores acumulados en `self.errores` deben ser consultados por el
    /// llamante para decidir si el programa es válido.
    pub fn parsear_programa(&mut self) -> Programa {
        // Vec con capacidad dinámica. Crece en el heap a medida que se
        // parsean sentencias. No se pre-asigna capacidad porque no sabemos
        // cuántas sentencias vienen.
        let mut sentencias = Vec::new();

        // El bucle principal itera mientras haya tokens por consumir.
        // token_actual se actualiza en cada iteración via self.avanzar().
        while self.token_actual != Token::FinDeArchivo {
            // parsear_sentencia() retorna Option<Statement>:
            //   Some(stmt) → sentencia válida, se agrega al vector
            //   None       → token no reconocido como inicio de sentencia,
            //                se ignora y se avanza al siguiente
            if let Some(sentencia) = self.parsear_sentencia() {
                // Se empuja la sentencia al vector. La ownership de la
                // sentencia se transfiere al Vec. No hay clonación.
                sentencias.push(sentencia);
                // Después de una sentencia válida, solo se avanza si el
                // token actual es ';' (punto y coma). Las sentencias como
                // `let` dejan el ';' sin consumir para que el bucle
                // principal lo salte. Los bloques ({}, if, while) ya
                // consumen su '}' internamente via parsear_bloque, por lo
                // que NO deben avanzar aquí (el token siguiente ya está
                // en token_actual).
                if self.token_actual == Token::PuntoComa {
                    self.avanzar();
                }
            } else {
                // Token no reconocido: avanzar para evitar bucle infinito.
                self.avanzar();
            }
        }

        // Se construye Programa con el vector de sentencias.
        // La ownership del Vec se transfiere al Programa.
        Programa { sentencias }
    }

    /// Enruta al parseador específico según el token actual.
    ///
    /// Cada variante de `Token` que puede iniciar una sentencia se mapea a
    /// su función de parseo correspondiente. Por omisión, retorna None para
    /// que el bucle principal consuma el token sin generar sentencia.
    ///
    /// # Control de flujo
    /// Se usa `match &self.token_actual` (referencia) para evitar mover el
    /// token. Solo se consume (via avanzar) dentro de las funciones delegadas.
    fn parsear_sentencia(&mut self) -> Option<Statement> {
        // -------------------------------------------------------------------
        // Detección de asignación: `identificador = expresion;`
        // -------------------------------------------------------------------
        // Antes de delegar en el match genérico, verificamos si el patrón
        // es `Token::Identificador` seguido de `Token::Asignacion` (=).
        // Esto es necesario porque `Identificador` también inicia expresiones
        // (llamadas a función, etc.), pero si el lookahead es `=`, debe
        // tratarse como reasignación en lugar de expresión.
        //
        // # Préstamo conflictivo
        // Tomamos `&self.token_siguiente` mientras `self.token_actual` está
        // prestado por el match. Ambos son campos independientes del struct
        // Parser, por lo que el borrow checker los acepta como prestamos
        // disjuntos (cada uno es un campo diferente de self).
        if self.token_siguiente == Token::Asignacion {
            let nombre_var = match self.avanzar() {
                Token::Identificador(n) => n,
                _ => {
                    self.error_token_esperado("identificador para la asignación");
                    return None;
                }
            };
            self.avanzar();
            let valor = self.parsear_expresion(Precedencia::Menor)?;
            return Some(Statement::AsignacionVariable {
                nombre: nombre_var,
                valor,
            });
        }

        // += y -=: Azucar sintáctica para `x = x + expr` y `x = x - expr`
        if self.token_siguiente == Token::SumaAsignacion {
            let nombre_var = match self.avanzar() {
                Token::Identificador(n) => n,
                _ => {
                    self.error_token_esperado("identificador para += ");
                    return None;
                }
            };
            self.avanzar();
            let valor = self.parsear_expresion(Precedencia::Menor)?;
            let valor_binario = Expression::OperacionBinaria {
                operador: Token::Suma,
                izquierda: Box::new(Expression::Identificador(nombre_var.clone())),
                derecha: Box::new(valor),
            };
            return Some(Statement::AsignacionVariable {
                nombre: nombre_var,
                valor: valor_binario,
            });
        }

        if self.token_siguiente == Token::RestaAsignacion {
            let nombre_var = match self.avanzar() {
                Token::Identificador(n) => n,
                _ => {
                    self.error_token_esperado("identificador para -= ");
                    return None;
                }
            };
            self.avanzar();
            let valor = self.parsear_expresion(Precedencia::Menor)?;
            let valor_binario = Expression::OperacionBinaria {
                operador: Token::Resta,
                izquierda: Box::new(Expression::Identificador(nombre_var.clone())),
                derecha: Box::new(valor),
            };
            return Some(Statement::AsignacionVariable {
                nombre: nombre_var,
                valor: valor_binario,
            });
        }

        // Si no es asignación, proceder con el match normal.
        // Se evalúa el token actual por referencia para no consumirlo.
        // El match cubre explícitamente solo los tokens que inician sentencia;
        // el brazo `_` captura todo lo demás.
        match &self.token_actual {
            // let <identificador> = <expresion>;
            Token::Let => self.parsear_declaracion_let(false),
            // const <identificador> = <expresion>;
            Token::Const => self.parsear_declaracion_let(true),
            Token::LlaveAbierta => self.parsear_bloque(),
            Token::If => self.parsear_sentencia_if(),
            Token::While => self.parsear_sentencia_while(),
            Token::Return => self.parsear_sentencia_return(),
            Token::For => self.parsear_sentencia_for(),
            Token::Try => self.parsear_sentencia_try(),
            Token::Break => self.parsear_sentencia_break(),
            Token::Continue => self.parsear_sentencia_continue(),
            Token::Fn => self.parsear_declaracion_funcion(),
            Token::Struct => self.parsear_declaracion_struct(),
            Token::Enum => self.parsear_declaracion_enum(),
            Token::Extern => self.parsear_declaracion_extern(),
            // Expresiones como sentencias (ej. llamadas a función foo())
            Token::Identificador(_)
            | Token::Entero(_)
            | Token::Flotante(_)
            | Token::True
            | Token::False
            | Token::Cadena(_)
            | Token::ParentesisAbierto
            | Token::CorcheteAbierto
            | Token::Suma
            | Token::Resta
            | Token::Not
            | Token::Throw
            | Token::Match
            | Token::Import => self.parsear_expresion_como_sentencia(),
            // Expansión futura: etc.
            // Cualquier token que no inicie sentencia se ignora.
            _ => None,
        }
    }

    /// Construye un nodo `DeclaracionVariable` del AST.
    ///
    /// Gramática: `let <identificador> = <expresion>;`
    ///
    /// Flujo:
    ///   1. token_actual es `Let` (confirmado por parsear_sentencia).
    ///   2. Se espera `Identificador` en token_siguiente.
    ///   3. Se espera `Asignacion` (`=`) después del identificador.
    ///   4. Se parsea la expresión con `parsear_expresion(Precedencia::Menor)`.
    ///   5. El `;` final es opcional y se consume si está presente.
    ///
    /// # Gestión de memoria
    /// - El nombre de variable se extrae mediante `n.clone()` porque
    ///   token_siguiente contiene una `&String` dentro de `Token::Identificador`.
    ///   Clone crea una copia heap-allocada independiente. Esto es necesario
    ///   porque al llamar a `self.avanzar()` se muta el parser, invalidando
    ///   la referencia prestada `n`.
    /// - La expresión parseada se mueve directamente al Statement sin clonar.
    fn parsear_declaracion_let(&mut self, constante: bool) -> Option<Statement> {
        // -- Paso 1: Extraer el nombre de la variable --
        // Se toma prestado (&) token_siguiente para examinar su variante
        // sin consumirlo. Si es Identificador, se clona el String interno
        // ANTES de mutar self con avanzar(), para no violar las reglas del
        // borrow checker (referencia inmutable a self.token_siguiente vs.
        // referencia mutable en self.avanzar()).
        let nombre = match &self.token_siguiente {
            Token::Identificador(n) => {
                // Clone del String: operación O(n) sobre el heap. Es
                // inevitable porque el Token actual será reemplazado.
                let nombre = n.clone();
                // Avanzar consume el token Identificador: token_actual
                // pasa a ser Identificador("x"), token_siguiente es el
                // token después del identificador (normalmente Asignacion).
                self.avanzar();
                nombre
            }
            // Si el token siguiente no es un identificador, la declaración
            // let está mal formada. Se registra el error y se retorna None.
            _ => {
                // error_token_esperado empuja un String al Vec de errores.
                self.error_token_esperado("identificador para la variable");
                return None;
            }
        };

        // -- Paso 2: Consumir el signo '=' --
        // Se compara token_siguiente directamente con Token::Asignacion.
        // Ambos implementan PartialEq, por lo que la comparación es por
        // variante y valor encapsulado (aunque Asignacion no lleva valor).
        if self.token_siguiente != Token::Asignacion {
            self.error_token_esperado("'='");
            return None;
        }
        // Avanzar consume '=': token_actual = Asignacion, token_siguiente
        // apunta al primer token de la expresión (ej. Entero(42)).
        self.avanzar();

        // -- Paso 3: Parsear la expresión --
        // Se avanza para que token_actual sea el primer token de la
        // expresión (ya que token_actual actualmente es '=' y fue el
        // último token consumido en el paso anterior). Luego se delega
        // en el núcleo Pratt con precedencia base Menor.
        self.avanzar();

        // parsear_expresion con Precedencia::Menor indica que solo se
        // detendrá cuando encuentre un token con precedencia menor a
        // Menor (es decir, ninguno, porque Menor es el mínimo). Por lo
        // tanto, parsea toda la expresión hasta el primer token que no
        // sea parte de ella (; , EOF, etc.).
        let valor = self.parsear_expresion(Precedencia::Menor)?;

        // -- Paso 4: ';' opcional —
        // El punto y coma es un terminador de sentencia opcional.
        // NO se consume aquí: el bucle principal en parsear_programa
        // llama a self.avanzar() al final de cada iteración, y como
        // PuntoComa no es inicio de sentencia, parsear_sentencia
        // retorna None y el bucle lo consume limpiamente.
        // Esto evita un off-by-one en el flujo de tokens.

        // Se construye y retorna el nodo Statement.
        // nombre (String) se mueve al Statement, valor (Expression) también.
        Some(Statement::DeclaracionVariable { nombre, valor, constante })
    }

    /// Parsea un bloque de código delimitado por llaves: `{ <sentencias> }`
    ///
    /// Gramática: `{ <sentencia>* }`
    ///
    /// # Flujo
    ///   1. token_actual es `LlaveAbierta` (confirmado por parsear_sentencia).
    ///   2. Se consume la `{` con `self.avanzar()`.
    ///   3. Bucle: mientras token_actual no sea `}` ni `FinDeArchivo`:
    ///      a. Llama a `self.parsear_sentencia()`. Si retorna Some, se agrega al
    ///        vector de sentencias internas.
    ///      b. Llama a `self.avanzar()` para avanzar al siguiente token (esto
    ///        consume tanto punto y coma como tokens desconocidos).
    ///   4. Al salir del bucle:
    ///      - Si token_actual es `}`: lo consume con `avanzar()` y retorna
    ///        `Statement::Bloque(sentencias)`.
    ///      - Si es `FinDeArchivo`: error, el bloque quedó abierto, retorna None.
    ///
    /// # Gestión de memoria
    /// - `sentencias: Vec<Statement>` crece dinámicamente en heap. Cada
    ///   Statement se mueve (ownership) al vector sin clonar.
    /// - Al retornar `Statement::Bloque(sentencias)`, el Vec completo se
    ///   mueve al enum. No hay copia de las sentencias internas.
    fn parsear_bloque(&mut self) -> Option<Statement> {
        // Consumir la llave de apertura `{`.
        self.avanzar();

        // Vec para las sentencias internas. Crece en heap dinámicamente.
        let mut sentencias: Vec<Statement> = Vec::new();

        // Bucle: itera hasta encontrar `}` o `FinDeArchivo`.
        while self.token_actual != Token::LlaveCerrada
            && self.token_actual != Token::FinDeArchivo
        {
            // Intentar parsear una sentencia con el token actual.
            if let Some(sentencia) = self.parsear_sentencia() {
                sentencias.push(sentencia);
            }
            // Avanzar al siguiente token solo si es un punto y coma,
            // que es el terminador de sentencias dentro de bloques.
            // No avanzamos si el token es `}` (significa que una sentencia
            // como `while`, `for` o `if` ya consumió su bloque interno dejando
            // token_actual apuntando a la llave de cierre del bloque actual),
            // o si es otro token inesperado.
            if self.token_actual == Token::PuntoComa {
                self.avanzar();
            }
        }

        // Verificar por qué terminó el bucle.
        if self.token_actual == Token::LlaveCerrada {
            // Consumir la llave de cierre `}` y retornar el bloque.
            self.avanzar();
            Some(Statement::Bloque(sentencias))
        } else {
            // Llegó a FinDeArchivo sin encontrar `}` — bloque sin cerrar.
            let mensaje =
                "Error sintáctico: se esperaba '}' al cerrar el bloque".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            None
        }
    }

    /// Parsea una sentencia `if` con cláusula `else` opcional.
    ///
    /// Gramática: `if [<condicion>] <consecuencia> [else <alternativa>]`
    ///
    /// # Flujo
    ///   1. token_actual es `If` (confirmado por parsear_sentencia).
    ///   2. Se consume el `if` con `self.avanzar()`.
    ///   3. La condición puede llevar paréntesis opcionales (estilo Lua).
    ///      Si token_actual es `ParentesisAbierto`, se consumen los paréntesis
    ///      alrededor de la expresión; si no, se parsea directamente.
    ///   4. Se parsea la expresión de la condición.
    ///   5. El cuerpo (consecuencia) debe ser un bloque `{ ... }`.
    ///      Se delega en `self.parsear_sentencia()`, que invoca `parsear_bloque`.
    ///   6. Opcionalmente se parsea `else`:
    ///      - `else if <condicion> <bloque>` → recursión en parsear_sentencia_if
    ///      - `else <bloque>` → un bloque simple
    ///   7. Se retorna `Statement::If { condicion, consecuencia, alternativa }`.
    ///
    /// # Gestión de memoria (Box)
    /// - `Statement::If` contiene `consecuencia: Box<Statement>` y
    ///   `alternativa: Option<Box<Statement>>`. El Box es un puntero inteligente
    ///   de 8 bytes (en 64 bits) que apunta al heap. Sin Box, el enum
    ///   `Statement` tendría tamaño infinito porque `If` contendría
    ///   recursivamente otros `Statement`. Box rompe la recursión al colocar
    ///   el contenido en el heap; el enum solo almacena el puntero.
    /// - La `condicion: Expression` también podría ser grande (un árbol de
    ///   binop/unario), pero Expression ya usa Box internamente para sus
    ///   hijos, por lo que Expression tiene tamaño fijo.
    fn parsear_sentencia_if(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir el token `if` --
        self.avanzar();

        // -- Paso 2: Parsear la condicion, con paréntesis opcionales --
        // Si hay un paréntesis de apertura, se consume y se espera el cierre.
        let parentesis_opcional = self.token_actual == Token::ParentesisAbierto;
        if parentesis_opcional {
            self.avanzar();
        }
        let condicion = self.parsear_expresion(Precedencia::Menor)?;
        if parentesis_opcional {
            if self.token_actual == Token::ParentesisCerrado {
                self.avanzar();
            } else {
                let mensaje =
                    "Error de sintaxis: se esperaba ')' después de la condición del if"
                        .to_string();
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        }

        // -- Paso 3: Parsear el cuerpo (consecuencia) --
        // Debe ser un bloque delimitado por llaves.
        if self.token_actual != Token::LlaveAbierta {
            let mensaje =
                "Error de sintaxis: se esperaba '{' para iniciar el cuerpo del if"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        // parsear_sentencia() reconoce LlaveAbierta y delega en parsear_bloque().
        let consecuencia = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 4: Parsear la cláusula `else` opcional --
        // NOTA: token_actual apunta al token que siguió al bloque (}
        // fue consumido por parsear_bloque, que llamó a avanzar() internamente).
        // token_siguiente tiene el token posterior.
        // Sin embargo, miramos token_actual porque podría ser `Else`.
        let alternativa = if self.token_actual == Token::Else {
            // Consumir el token `else`.
            self.avanzar();
            // Determinar si es `else if` (recursión) o `else <bloque>`.
            if self.token_actual == Token::If {
                // `else if`: llamada recursiva. parsear_sentencia_if() consume
                // el `if`, su condición y su bloque internamente.
                Some(Box::new(self.parsear_sentencia_if()?))
            } else if self.token_actual == Token::LlaveAbierta {
                // `else <bloque>`: parsear_sentencia reconoce LlaveAbierta.
                match self.parsear_sentencia() {
                    Some(stmt) => Some(Box::new(stmt)),
                    None => return None,
                }
            } else {
                // `else` sin cuerpo válido.
                let mensaje =
                    "Error de sintaxis: se esperaba 'if' o '{' después de 'else'"
                        .to_string();
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        } else {
            // No hay cláusula else.
            None
        };

        // -- Paso 5: Retornar el nodo AST --
        Some(Statement::If {
            condicion,
            consecuencia,
            alternativa,
        })
    }

    /// Parsea un bucle `while`.
    ///
    /// Gramática: `while [<condicion>] <cuerpo>`
    ///
    /// # Flujo
    ///   1. token_actual es `While` (confirmado por parsear_sentencia).
    ///   2. Se consume el `while` con `self.avanzar()`.
    ///   3. La condición puede llevar paréntesis opcionales (estilo Lua).
    ///      Si token_actual es `ParentesisAbierto`, se consumen los paréntesis
    ///      alrededor de la expresión; si no, se parsea directamente.
    ///   4. Se parsea la expresión de la condición.
    ///   5. El cuerpo debe ser un bloque `{ ... }`. Se delega en
    ///      `self.parsear_sentencia()`, que invoca `parsear_bloque`.
    ///   6. Se retorna `Statement::While { condicion, cuerpo }`.
    ///
    /// # Gestión de memoria (Box)
    /// - `Statement::While` contiene `cuerpo: Box<Statement>`. El Box asigna
    ///   el bloque en el heap (8 bytes en 64 bits). Sin Box, el enum
    ///   `Statement` tendría tamaño infinito porque `While` contendría
    ///   recursivamente otros `Statement` dentro del bloque.
    fn parsear_sentencia_while(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir el token `while` --
        self.avanzar();

        // -- Paso 2: Parsear la condición, con paréntesis opcionales --
        let parentesis_opcional = self.token_actual == Token::ParentesisAbierto;
        if parentesis_opcional {
            self.avanzar();
        }
        let condicion = self.parsear_expresion(Precedencia::Menor)?;
        if parentesis_opcional {
            if self.token_actual == Token::ParentesisCerrado {
                self.avanzar();
            } else {
                let mensaje =
                    "Error de sintaxis: se esperaba ')' después de la condición del while"
                        .to_string();
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        }

        // -- Paso 3: Parsear el cuerpo del bucle --
        if self.token_actual != Token::LlaveAbierta {
            let mensaje =
                "Error de sintaxis: se esperaba '{' para iniciar el cuerpo del while"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let cuerpo = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 4: Retornar el nodo AST --
        Some(Statement::While {
            condicion,
            cuerpo,
        })
    }

    /// Parsea un bucle `for` al estilo C: `for (init; cond; update) { cuerpo }`
    ///
    /// Gramática: `for ( <inicializacion> ; <condicion> ; <actualizacion> ) <cuerpo>`
    ///
    /// # Flujo
    ///   1. token_actual es `For` (confirmado por parsear_sentencia).
    ///   2. Se consume el `for` con `self.avanzar()`.
    ///   3. Opcionalmente se consume `(`.
    ///   4. Se parsea la inicialización como una sentencia (normalmente `let i = 0;`).
    ///      Esta sentencia consumirá su propio `;` interno si es necesario.
    ///   5. Se parsea la condición como expresión.
    ///   6. Se consume obligatoriamente un `;`.
    ///   7. Se parsea la actualización como una sentencia (normalmente `i = i + 1`).
    ///      Como parsear_sentencia no consume el `;` final, dejamos que el bucle
    ///      principal o el bloque lo manejen.
    ///   8. Opcionalmente se consume `)`.
    ///   9. Se parsea el cuerpo (debe ser un bloque `{ ... }`).
    ///
    /// # Gestión de memoria (Box)
    /// - inicializacion, actualizacion y cuerpo se envuelven en Box para que
    ///   el enum Statement tenga tamaño fijo.
    fn parsear_sentencia_for(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir la palabra clave `for` --
        self.avanzar();

        // -- Paso 2: Paréntesis de apertura opcional --
        let parentesis_opcional = self.token_actual == Token::ParentesisAbierto;
        if parentesis_opcional {
            self.avanzar();
        }

        // -- Paso 3: Parsear la inicialización (sentencia) --
        // Normalmente `let i = 0;` o `i = 0;`. Las sentencias NO consumen
        // el `;` final (véase parsear_declaracion_let), así que después de
        // parsear la inicialización token_actual apunta al `;`. Debemos
        // consumirlo aquí antes de pasar a la condición.
        let inicializacion = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };
        // Consumir el `;` que terminó la inicialización.
        if self.token_actual == Token::PuntoComa {
            self.avanzar();
        }

        // -- Paso 4: Parsear la condición (expresión) --
        let condicion = self.parsear_expresion(Precedencia::Menor)?;

        // -- Paso 5: Consumir el `;` entre condición y actualización --
        if self.token_actual != Token::PuntoComa {
            let mensaje =
                "Error de sintaxis: se esperaba ';' después de la condición del for"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // -- Paso 6: Parsear la actualización (sentencia) --
        // Normalmente `i = i + 1`. Al igual que la inicialización, la
        // sentencia de actualización deja token_actual apuntando al `;`.
        // Consumimos ese `;` opcional antes de verificar el paréntesis.
        let actualizacion = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };
        if self.token_actual == Token::PuntoComa {
            self.avanzar();
        }

        // -- Paso 7: Paréntesis de cierre opcional --
        if parentesis_opcional {
            if self.token_actual == Token::ParentesisCerrado {
                self.avanzar();
            } else {
                let mensaje =
                    "Error de sintaxis: se esperaba ')' después de la actualización del for"
                        .to_string();
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        }

        // -- Paso 8: Parsear el cuerpo del bucle --
        // Debe ser un bloque delimitado por llaves.
        if self.token_actual != Token::LlaveAbierta {
            let mensaje =
                "Error de sintaxis: se esperaba '{' para iniciar el cuerpo del for"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let cuerpo = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 9: Retornar el nodo AST --
        Some(Statement::For {
            inicializacion,
            condicion,
            actualizacion,
            cuerpo,
        })
    }

    /// Parsea una sentencia `try / catch` para manejo de excepciones.
    ///
    /// Gramática: `try <bloque> catch ( <identificador> ) <bloque>`
    ///
    /// # Flujo
    ///   1. token_actual es `Try`.
    ///   2. Se consume el `try`.
    ///   3. Se parsea el bloque protegido: debe ser `{ ... }`.
    ///   4. Se verifica que el siguiente token sea `catch`.
    ///   5. Opcionalmente se consume `(`.
    ///   6. Se espera un identificador como nombre del parámetro de error.
    ///   7. Opcionalmente se consume `)`.
    ///   8. Se parsea el bloque de rescate: debe ser `{ ... }`.
    ///   9. Retorna `Statement::TryCatch`.
    fn parsear_sentencia_try(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir la palabra clave `try` --
        self.avanzar();

        // -- Paso 2: Parsear el bloque protegido --
        if self.token_actual != Token::LlaveAbierta {
            let mensaje =
                "Error de sintaxis: se esperaba '{' después de 'try'".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let bloque_try = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 3: Verificar la palabra clave `catch` --
        if self.token_actual != Token::Catch {
            let mensaje =
                "Error de sintaxis: se esperaba 'catch' después del bloque try"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // -- Paso 4: Paréntesis de apertura opcional --
        let parentesis_opcional = self.token_actual == Token::ParentesisAbierto;
        if parentesis_opcional {
            self.avanzar();
        }

        // -- Paso 5: Extraer el nombre del parámetro del error --
        let parametro_catch = match self.avanzar() {
            Token::Identificador(nombre) => nombre,
            _ => {
                let mensaje = format!(
                    "Error de sintaxis: se esperaba un identificador \
                     después de 'catch', pero se encontró {:?}",
                    self.token_actual
                );
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        };

        // -- Paso 6: Paréntesis de cierre opcional --
        if parentesis_opcional {
            if self.token_actual == Token::ParentesisCerrado {
                self.avanzar();
            } else {
                let mensaje =
                    "Error de sintaxis: se esperaba ')' después \
                     del parámetro de catch"
                        .to_string();
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        }

        // -- Paso 7: Parsear el bloque de rescate --
        if self.token_actual != Token::LlaveAbierta {
            let mensaje =
                "Error de sintaxis: se esperaba '{' para iniciar \
                 el bloque de catch"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let bloque_catch = match self.parsear_sentencia() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 8: Retornar el nodo AST --
        Some(Statement::TryCatch {
            bloque_try,
            parametro_catch,
            bloque_catch,
        })
    }

    /// Parsea una sentencia `return` con valor opcional.
    ///
    /// Gramática: `return [<expresion>]`
    ///
    /// # Flujo
    ///   1. token_actual es `Return` (confirmado por parsear_sentencia).
    ///   2. Se consume el `return` con `self.avanzar()`.
    ///   3. Si el token actual puede iniciar una expresión (Identificador,
    ///      Entero, etc.), se parsea como valor de retorno.
    ///   4. Si no (es `;`, `}`, `FinDeArchivo`, etc.), es un return sin valor.
    ///   5. El `;` final NO se consume — lo maneja el bucle principal
    ///      igual que en las declaraciones `let`.
    ///   6. Retorna `Statement::Return(valor)`.
    fn parsear_sentencia_return(&mut self) -> Option<Statement> {
        // Consumir la palabra clave `return`.
        self.avanzar();

        // Determinar si hay una expresión de retorno. Los tokens que pueden
        // iniciar una expresión están cubiertos por el match prefix en
        // parsear_expresion. Si token_actual es uno de los terminadores
        // conocidos (;, }, EOF), es un return sin valor.
        let valor = match self.token_actual {
            Token::PuntoComa | Token::LlaveCerrada | Token::FinDeArchivo => None,
            // Cualquier otro token se interpreta como inicio de expresión.
            _ => Some(self.parsear_expresion(Precedencia::Menor)?),
        };

        Some(Statement::Return(valor))
    }

    /// Parsea una expresión usada como sentencia (ej. `foo(1, 2);`).
    ///
    /// # Flujo
    ///   1. Delega en `parsear_expresion(Menor)` para obtener la expresión.
    ///   2. El `;` final NO se consume — lo maneja el bucle principal.
    ///   3. Retorna `Statement::Expresion(expr)`.
    fn parsear_expresion_como_sentencia(&mut self) -> Option<Statement> {
        let expr = self.parsear_expresion(Precedencia::Menor)?;
        Some(Statement::Expresion(expr))
    }

    /// Parsea una declaración de función: `fn <nombre>(<parametros>) <cuerpo>`
    ///
    /// Gramática: `fn identificador ( <identificador>, ... ) { <sentencias> }`
    ///
    /// # Flujo
    ///   1. token_actual es `Fn` (confirmado por parsear_sentencia).
    ///   2. Se consume el `fn` con `self.avanzar()`.
    ///   3. Se espera un identificador como nombre de la función.
    ///   4. Se espera `(` para la lista de parámetros.
    ///   5. Se itera sobre identificadores separados por comas (vacío si `)`).
    ///   6. Se espera `)` al final de la lista.
    ///   7. Se espera `{` para el cuerpo: se delega en `parsear_bloque()`.
    ///   8. Se retorna `Statement::DeclaracionFuncion { nombre, parametros, cuerpo }`.
    ///
    /// # Gestión de memoria
    /// - `nombre: String` se clona del token. Es necesario porque al avanzar
    ///   se muta token_actual y se pierde la referencia al String original.
    /// - `parametros: Vec<String>` crece en heap dinámicamente. Cada nombre
    ///   se clona individualmente al vector.
    /// - `cuerpo: Box<Statement>` coloca el bloque en el heap. Sin Box, el
    ///   enum `Statement` tendría tamaño infinito al contener recursivamente
    ///   otros `Statement` dentro del bloque.
    fn parsear_sentencia_break(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir la palabra clave `break` --
        self.avanzar();

        // -- Paso 2: Consumir el `;` opcional --
        if self.token_actual == Token::PuntoComa {
            self.avanzar();
        }

        // -- Paso 3: Retornar el nodo AST --
        Some(Statement::Break)
    }

    /// Parsea una sentencia `continue` para saltar a la siguiente iteración.
    fn parsear_sentencia_continue(&mut self) -> Option<Statement> {
        self.avanzar();
        if self.token_actual == Token::PuntoComa {
            self.avanzar();
        }
        Some(Statement::Continue)
    }

    fn parsear_declaracion_funcion(&mut self) -> Option<Statement> {
        // -- Paso 1: Consumir la palabra clave `fn` --
        self.avanzar();

        // -- Paso 2: Extraer el nombre de la función --
        let nombre = match self.avanzar() {
            Token::Identificador(nombre) => nombre,
            _ => {
                let mensaje = format!(
                    "Error de sintaxis: se esperaba el nombre de la función, pero se encontró {:?}",
                    self.token_actual
                );
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        };

        // -- Paso 3: Consumir el paréntesis de apertura --
        if self.token_actual != Token::ParentesisAbierto {
            let mensaje =
                "Error de sintaxis: se esperaba '(' después del nombre de la función"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // -- Paso 4: Parsear la lista de parámetros (con tipos opcionales) --
        let mut parametros: Vec<String> = Vec::with_capacity(4);
        let mut tipos_parametros: Vec<Option<String>> = Vec::with_capacity(4);
        if self.token_actual != Token::ParentesisCerrado {
            loop {
                match self.avanzar() {
                    Token::Identificador(param) => {
                        let tipo = if self.token_actual == Token::DosPuntos {
                            self.avanzar();
                            match self.avanzar() {
                                Token::Identificador(t) => Some(t),
                                _ => {
                                    let mensaje = "Error de sintaxis: se esperaba un tipo después de ':'".to_string();
                                    self.errores.push(self.error_ubicacion(mensaje));
                                    return None;
                                }
                            }
                        } else {
                            None
                        };
                        parametros.push(param);
                        tipos_parametros.push(tipo);
                    }
                    _ => {
                        let mensaje = format!(
                            "Error de sintaxis: se esperaba un parámetro, pero se encontró {:?}",
                            self.token_actual
                        );
                        self.errores.push(self.error_ubicacion(mensaje));
                        return None;
                    }
                }
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::ParentesisCerrado {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // -- Paso 5: Consumir el paréntesis de cierre --
        if self.token_actual != Token::ParentesisCerrado {
            let mensaje = format!(
                "Error de sintaxis: se esperaba ')' después de los parámetros, pero se encontró {:?}",
                self.token_actual
            );
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // -- Paso 5b: Parsear tipo de retorno opcional `: Tipo` --
        let tipo_retorno = if self.token_actual == Token::DosPuntos {
            self.avanzar();
            match self.avanzar() {
                Token::Identificador(t) => Some(t),
                _ => {
                    let mensaje = "Error de sintaxis: se esperaba un tipo después de ':'".to_string();
                    self.errores.push(self.error_ubicacion(mensaje));
                    return None;
                }
            }
        } else {
            None
        };

        // -- Paso 6: Parsear el cuerpo de la función --
        if self.token_actual != Token::LlaveAbierta {
            let mensaje = format!(
                "Error de sintaxis: se esperaba '{{' para el cuerpo de la función, pero se encontró {:?}",
                self.token_actual
            );
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let cuerpo = match self.parsear_bloque() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // -- Paso 7: Retornar el nodo AST --
        Some(Statement::DeclaracionFuncion {
            nombre,
            parametros,
            cuerpo,
            tipos_parametros,
            tipo_retorno,
        })
    }

    fn parsear_declaracion_struct(&mut self) -> Option<Statement> {
        self.avanzar();
        let nombre = match self.avanzar() {
            Token::Identificador(n) => n,
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba el nombre del struct".to_string(),
                ));
                return None;
            }
        };
        if self.token_actual != Token::LlaveAbierta {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '{' después del nombre del struct".to_string(),
            ));
            return None;
        }
        self.avanzar();

        let mut campos = Vec::new();
        if self.token_actual != Token::LlaveCerrada {
            loop {
                match self.avanzar() {
                    Token::Identificador(c) => campos.push(c),
                    _ => {
                        self.errores.push(self.error_ubicacion(
                            "Error de sintaxis: se esperaba un nombre de campo".to_string(),
                        ));
                        return None;
                    }
                }
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::LlaveCerrada {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' para cerrar la lista de campos".to_string(),
            ));
            return None;
        }
        self.avanzar();

        Some(Statement::DeclaracionStruct { nombre, campos })
    }

    // -----------------------------------------------------------------------
    // Núcleo del Algoritmo de Pratt (Top-Down Operator Precedence)
    // -----------------------------------------------------------------------

    /// Parsea una expresión usando el algoritmo de Pratt.
    ///
    /// # Argumentos
    /// - `precedencia`: nivel mínimo de precedencia. El parser sigue
    ///   consumiendo tokens infijos mientras la precedencia del operador
    ///   actual sea mayor que `precedencia`. Esto permite que llamadas
    ///   recursivas bindeen más fuerte (p. ej., en `1 + 2 * 3`, la llamada
    ///   para parsear después de `+` pasa Suma como mínima, por lo que
    ///   `*` (Multiplicacion > Suma) se parsea antes de retornar).
    ///
    /// # Flujo Pratt en dos fases
    ///
    /// ## Fase 1: Parseo de prefijo (núcleo o "led" izquierdo)
    /// Se identifica el token actual como inicio de una expresión primaria
    /// (identificador, literal, etc.) y se invoca su función de parseo
    /// "prefix". Cada prefix parser consume el token actual y avanza.
    ///
    /// ## Fase 2: Parseo de infijo (bucle Pratt)
    /// Mientras el token actual tenga una precedencia infija mayor que la
    /// precedencia recibida, se trata como operador binario. Esto permite
    /// que operadores con alta precedencia (como `*`) se agrupen antes que
    /// los de baja precedencia (como `+`) de forma natural, sin necesidad
    /// de una gramática jerárquica explícita.
    ///
    /// # Gestión de memoria
    /// - `izquierda` acumula la expresión mediante assignment: en cada
    ///   iteración del bucle infijo, se reemplaza con una nueva expresión
    ///   binaria que envuelve la anterior. Esto causa una asignación en
    ///   heap por cada operador (Box::new). Es un costo aceptable para un
    ///   compilador de scripting.
    /// - El operador se clona con `.clone()` porque se necesita ownership
    ///   para almacenarlo en el AST. Esto es inevitable ya que token_actual
    ///   es una referencia prestada.
    fn parsear_expresion(&mut self, precedencia: Precedencia) -> Option<Expression> {
        // ===================================================================
        // FASE 1: Prefijo — determinar el "núcleo" izquierdo
        // ===================================================================
        // Se examina token_actual para determinar qué tipo de expresión
        // primaria inicia. Cada variante tokenizable como expresión tiene
        // su propia función de parseo prefix.
        //
        // El operador `?` (try operator) propaga el None si la función
        // prefix falla, retornando None inmediatamente. Esto hace que
        // parsear_expresion retorne None y el llamante maneje el error.
        let mut izquierda = match &self.token_actual {
            Token::Identificador(_) => self.parsear_prefijo_identificador()?,
            Token::Entero(_) => self.parsear_entero()?,
            Token::Flotante(_) => self.parsear_flotante()?,
            Token::True | Token::False => self.parsear_booleano()?,
            Token::Cadena(_)          => self.parsear_cadena()?,
            Token::ParentesisAbierto => self.parsear_agrupacion()?,
            Token::CorcheteAbierto => self.parsear_arreglo()?,
            Token::LlaveAbierta => self.parsear_diccionario()?,
            Token::Suma | Token::Resta | Token::Not => self.parsear_unario()?,
            // `fn`: expresión de función anónima (fn(params) { cuerpo })
            Token::Fn => self.parsear_fn_expr()?,
            // `match`: expresión de pattern matching.
            Token::Match => self.parsear_match()?,
            // `throw`: lanzar una excepción con un valor arbitrario.
            Token::Throw => self.parsear_throw()?,
            // `import`: expresión de importación de módulo.
            Token::Import => self.parsear_import()?,
            // Si el token actual no puede iniciar una expresión, se
            // registra un error sintáctico y se retorna None.
            // Se usa format! manual en lugar de error_token_esperado porque
            // esa función reporta token_siguiente, pero aquí el token
            // problemático es token_actual (el prefix no reconocido).
            _ => {
                let mensaje = format!(
                    "Error de sintaxis: se esperaba expresión, pero se encontró {:?}",
                    self.token_actual
                );
                self.errores.push(self.error_ubicacion(mensaje));
                return None;
            }
        };

        // ===================================================================
        // FASE 2: Infijo — bucle de precedencia de Pratt
        // ===================================================================
        // Cada prefix parser llama internamente a `self.avanzar()` al
        // consumir el token, por lo que al salir del prefix, `token_actual`
        // ya apunta al primer token posterior a la expresión primaria.
        //
        // Ejemplo para `1 + 2 * 3`:
        //   Después de parsear_entero → token_actual = Suma (+),
        //   token_siguiente = Entero(2) (primer token del lado derecho).
        //
        // El bucle evalúa `token_actual` como potencial operador infijo.
        // Condiciones de continuación:
        //   1. token_actual NO es PuntoComa (;)
        //   2. token_actual NO es FinDeArchivo
        //   3. La precedencia de token_actual es ESTRICTAMENTE MAYOR
        //      que la precedencia recibida como argumento.
        //
        // Cuando se cumple la condición, se delega en
        // `parsear_expresion_infija(izquierda)`, que recibe la ownership
        // de `izquierda`, extrae el operador de `token_actual`, avanza
        // al lado derecho, parsea recursivamente y retorna un nuevo nodo
        // `OperacionBinaria`.
        //
        // El resultado reemplaza `izquierda` y el bucle vuelve a evaluar
        // el nuevo `token_actual` (que será el token después de la
        // expresión recién parseada).
        while self.token_actual != Token::PuntoComa
            && self.token_actual != Token::FinDeArchivo
            && precedencia < self.precedencia_actual()
        {
            izquierda = self.parsear_expresion_infija(izquierda)?;
        }

        // Retorna la expresión completa. La ownership se transfiere al
        // llamante. Si no hubo operadores infijos, izquierda es el
        // resultado del prefix parser.
        Some(izquierda)
    }

    // -----------------------------------------------------------------------
    // Funciones de parseo prefix (cada una consume token_actual)
    // -----------------------------------------------------------------------

    /// Parsea un identificador: `Token::Identificador(s)` → Expression::Identificador(s)
    ///
    /// # Gestión de memoria
    /// - Se usa `if let` para extraer el String del Token por referencia.
    /// - `n.clone()` crea una copia heap del nombre. Es necesario porque
    ///   `self.avanzar()` muta el parser tras extraer el valor.
    fn parsear_match(&mut self) -> Option<Expression> {
        self.avanzar();
        if self.token_actual != Token::ParentesisAbierto {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '(' después de 'match'".to_string(),
            ));
            return None;
        }
        self.avanzar();
        let expr = self.parsear_expresion(Precedencia::Menor)?;
        if self.token_actual != Token::ParentesisCerrado {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba ')' después de la expresión en match".to_string(),
            ));
            return None;
        }
        self.avanzar();
        if self.token_actual != Token::LlaveAbierta {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '{' para los brazos del match".to_string(),
            ));
            return None;
        }
        self.avanzar();

        let mut brazos = Vec::new();
        if self.token_actual != Token::LlaveCerrada {
            loop {
                let patron = self.parsear_patron()?;
                if self.token_actual != Token::Flecha {
                    self.errores.push(self.error_ubicacion(
                        "Error de sintaxis: se esperaba '=>' después del patrón".to_string(),
                    ));
                    return None;
                }
                self.avanzar();
                let valor = self.parsear_expresion(Precedencia::Menor)?;
                brazos.push((patron, valor));
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::LlaveCerrada {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' para cerrar el match".to_string(),
            ));
            return None;
        }
        self.avanzar();
        Some(Expression::Match {
            expr: Box::new(expr),
            brazos,
        })
    }

    fn parsear_patron(&mut self) -> Option<Patron> {
        match &self.token_actual {
            Token::Identificador(n) if n == "_" => {
                self.avanzar();
                Some(Patron::Wildcard)
            }
            Token::Identificador(_) => {
                let nombre = match self.avanzar() {
                    Token::Identificador(n) => n,
                    _ => return None,
                };
                if self.token_actual == Token::ParentesisAbierto {
                    // Variante(Campo1, Campo2) — patrón de enum
                    self.avanzar();
                    let mut bindings = Vec::new();
                    if self.token_actual != Token::ParentesisCerrado {
                        while let Token::Identificador(b) = self.avanzar() {
                            bindings.push(b);
                            if self.token_actual == Token::Coma {
                                self.avanzar();
                            } else {
                                break;
                            }
                        }
                    }
                    if self.token_actual == Token::ParentesisCerrado {
                        self.avanzar();
                    }
                    Some(Patron::EnumPatron { nombre, bindings })
                } else if self.token_actual == Token::DobleDosPuntos {
                    self.avanzar();
                    let variante = match self.avanzar() {
                        Token::Identificador(v) => v,
                        _ => return None,
                    };
                    let mut bindings = Vec::new();
                    if self.token_actual == Token::ParentesisAbierto {
                        self.avanzar();
                        while let Token::Identificador(b) = self.avanzar() {
                            bindings.push(b);
                            if self.token_actual == Token::Coma {
                                self.avanzar();
                            } else {
                                break;
                            }
                        }
                        if self.token_actual == Token::ParentesisCerrado {
                            self.avanzar();
                        }
                    }
                    Some(Patron::EnumPatron { nombre: variante, bindings })
                } else if self.token_actual == Token::LlaveAbierta {
                    self.parsear_patron_struct(nombre)
                } else {
                    Some(Patron::Binding(nombre))
                }
            }
            Token::CorcheteAbierto => {
                self.avanzar();
                let mut elementos = Vec::new();
                if self.token_actual != Token::CorcheteCerrado {
                    loop {
                        elementos.push(self.parsear_patron()?);
                        if self.token_actual == Token::Coma {
                            self.avanzar();
                            if self.token_actual == Token::CorcheteCerrado {
                                break;
                            }
                        } else {
                            break;
                        }
                    }
                }
                if self.token_actual != Token::CorcheteCerrado {
                    self.errores.push(self.error_ubicacion(
                        "Error de sintaxis: se esperaba ']' en patrón de arreglo".to_string(),
                    ));
                    return None;
                }
                self.avanzar();
                Some(Patron::Arreglo(elementos))
            }
            Token::Entero(v) => {
                let val = *v;
                self.avanzar();
                Some(Patron::Literal(Expression::Entero(val)))
            }
            Token::Flotante(v) => {
                let val = *v;
                self.avanzar();
                Some(Patron::Literal(Expression::Flotante(val)))
            }
            Token::True => {
                self.avanzar();
                Some(Patron::Literal(Expression::Booleano(true)))
            }
            Token::False => {
                self.avanzar();
                Some(Patron::Literal(Expression::Booleano(false)))
            }
            Token::Cadena(_) => {
                match self.avanzar() {
                    Token::Cadena(s) => Some(Patron::Literal(Expression::Cadena(s))),
                    _ => unreachable!(),
                }
            }
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: patrón inválido".to_string(),
                ));
                None
            }
        }
    }

    fn parsear_patron_struct(&mut self, nombre: String) -> Option<Patron> {
        self.avanzar();
        let mut campos = Vec::new();
        if self.token_actual != Token::LlaveCerrada {
            loop {
                let campo = match self.avanzar() {
                    Token::Identificador(c) => c,
                    _ => {
                        self.errores.push(self.error_ubicacion(
                            "Error de sintaxis: se esperaba nombre de campo en patrón".to_string(),
                        ));
                        return None;
                    }
                };
                if self.token_actual != Token::DosPuntos {
                    self.errores.push(self.error_ubicacion(
                        "Error de sintaxis: se esperaba ':' después del campo".to_string(),
                    ));
                    return None;
                }
                self.avanzar();
                let subpatron = self.parsear_patron()?;
                campos.push((campo, Box::new(subpatron)));
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::LlaveCerrada {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' en patrón de struct".to_string(),
            ));
            return None;
        }
        self.avanzar();
        Some(Patron::Struct(nombre, campos))
    }

    fn parsear_prefijo_identificador(&mut self) -> Option<Expression> {
        let nombre = match self.avanzar() {
            Token::Identificador(n) => n,
            _ => return None,
        };
        if self.token_actual == Token::LlaveAbierta {
            self.parsear_instancia_struct(nombre)
        } else if self.token_actual == Token::DobleDosPuntos {
            self.parsear_instancia_enum(nombre)
        } else {
            Some(Expression::Identificador(nombre))
        }
    }

    fn parsear_instancia_struct(&mut self, nombre: String) -> Option<Expression> {
        self.avanzar();
        let mut valores = Vec::new();
        if self.token_actual != Token::LlaveCerrada {
            loop {
                let campo = match self.avanzar() {
                    Token::Identificador(c) => c,
                    _ => {
                        self.errores.push(self.error_ubicacion(
                            "Error de sintaxis: se esperaba un nombre de campo"
                                .to_string(),
                        ));
                        return None;
                    }
                };
                if self.token_actual != Token::DosPuntos {
                    self.errores.push(self.error_ubicacion(
                        "Error de sintaxis: se esperaba ':' después del nombre del campo"
                            .to_string(),
                    ));
                    return None;
                }
                self.avanzar();
                let valor = self.parsear_expresion(Precedencia::Menor)?;
                valores.push((campo, valor));
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::LlaveCerrada {
                        break;
                    }
                } else {
                    break;
                }
            }
        }
        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' para cerrar la instancia del struct"
                    .to_string(),
            ));
            return None;
        }
        self.avanzar();
        Some(Expression::StructInstancia { nombre, valores })
    }

    /// Parsea `Nombre::Variante(args...)` → Expression::EnumInstancia
    fn parsear_instancia_enum(&mut self, enum_nombre: String) -> Option<Expression> {
        self.avanzar(); // consumir ::
        let variante = match self.avanzar() {
            Token::Identificador(v) => v,
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba el nombre de la variante después de '::'"
                        .to_string(),
                ));
                return None;
            }
        };
        let argumentos = if self.token_actual == Token::ParentesisAbierto {
            self.avanzar();
            let mut args = Vec::new();
            if self.token_actual != Token::ParentesisCerrado {
                loop {
                    let expr = self.parsear_expresion(Precedencia::Menor)?;
                    args.push(expr);
                    if self.token_actual == Token::Coma {
                        self.avanzar();
                    } else {
                        break;
                    }
                }
            }
            if self.token_actual != Token::ParentesisCerrado {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba ')' después de los argumentos".to_string(),
                ));
                return None;
            }
            self.avanzar();
            args
        } else {
            Vec::new()
        };
        Some(Expression::EnumInstancia { enum_nombre, variante, argumentos })
    }

    /// Parsea `enum Nombre { Variante1, Variente2(campo: Tipo) }`
    fn parsear_declaracion_enum(&mut self) -> Option<Statement> {
        self.avanzar();
        let nombre = match self.avanzar() {
            Token::Identificador(n) => n,
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba el nombre del enum".to_string(),
                ));
                return None;
            }
        };
        if self.token_actual != Token::LlaveAbierta {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '{' después del nombre del enum".to_string(),
            ));
            return None;
        }
        self.avanzar();
        let mut variantes = Vec::new();
        if self.token_actual != Token::LlaveCerrada {
            while let Token::Identificador(v_nombre) = self.avanzar() {
                let mut campos = Vec::new();
                if self.token_actual == Token::ParentesisAbierto {
                    self.avanzar();
                    while let Token::Identificador(c) = self.avanzar() {
                        let tipo = if self.token_actual == Token::DosPuntos {
                            self.avanzar();
                            match self.avanzar() {
                                Token::Identificador(t) => t,
                                _ => String::new(),
                            }
                        } else {
                            String::new()
                        };
                        campos.push((c, tipo));
                        if self.token_actual == Token::Coma {
                            self.avanzar();
                        } else {
                            break;
                        }
                    }
                    if self.token_actual == Token::ParentesisCerrado {
                        self.avanzar();
                    }
                }
                variantes.push(crate::ast::VarianteEnum { nombre: v_nombre, campos });
                if self.token_actual == Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' para cerrar el enum".to_string(),
            ));
            return None;
        }
        self.avanzar();
        Some(Statement::EnumDefinicion { nombre, variantes })
    }

    /// Parsea `extern fn nombre(args...);`
    fn parsear_declaracion_extern(&mut self) -> Option<Statement> {
        self.avanzar(); // consumir `extern`
        if self.token_actual != Token::Fn {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba 'fn' después de 'extern'".to_string(),
            ));
            return None;
        }
        self.avanzar();
        let nombre = match self.avanzar() {
            Token::Identificador(n) => n,
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba el nombre de la función externa".to_string(),
                ));
                return None;
            }
        };
        if self.token_actual != Token::ParentesisAbierto {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '(' después del nombre".to_string(),
            ));
            return None;
        }
        self.avanzar();
        if self.token_actual != Token::ParentesisCerrado {
            while let Token::Identificador(_) = self.avanzar() {
                if self.token_actual == Token::Coma {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        if self.token_actual != Token::ParentesisCerrado {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba ')' después de los parámetros".to_string(),
            ));
            return None;
        }
        self.avanzar();
        if self.token_actual == Token::PuntoComa {
            self.avanzar();
        }
        Some(Statement::ExternFn { nombre })
    }

    /// Parsea un entero: `Token::Entero(v)` → Expression::Entero(v)
    ///
    /// i64 es Copy, se extrae por desestructuración directa.
    fn parsear_entero(&mut self) -> Option<Expression> {
        if let Token::Entero(valor) = self.token_actual {
            self.avanzar();
            Some(Expression::Entero(valor))
        } else {
            None
        }
    }

    /// Parsea un flotante: `Token::Flotante(v)` → Expression::Flotante(v)
    ///
    /// f64 es Copy.
    fn parsear_flotante(&mut self) -> Option<Expression> {
        if let Token::Flotante(valor) = self.token_actual {
            self.avanzar();
            Some(Expression::Flotante(valor))
        } else {
            None
        }
    }

    /// Parsea un booleano: `Token::True`/`Token::False` → Expression::Booleano(bool)
    fn parsear_booleano(&mut self) -> Option<Expression> {
        let valor = match self.token_actual {
            Token::True => true,
            Token::False => false,
            _ => return None,
        };
        self.avanzar();
        Some(Expression::Booleano(valor))
    }

    /// Parsea un literal de cadena: `Token::Cadena(s)` → Expression::Cadena(s)
    ///
    /// # Gestión de memoria
    /// - `c.clone()` clona el String interno del Token. La clonación aloca
    ///   memoria heap (copia O(n) del contenido). Es necesaria porque
    ///   `self.avanzar()` muta el parser y movería el valor original.
    fn parsear_cadena(&mut self) -> Option<Expression> {
        match self.avanzar() {
            Token::Cadena(valor) => Some(Expression::Cadena(valor)),
            _ => None,
        }
    }

    /// Parsea una expresión agrupada entre paréntesis: `(<expr>)`
    ///
    /// # Flujo
    ///   1. Consume el `(` (token_actual ya fue validado como ParentesisAbierto).
    ///   2. Parsea la expresión interior con `parsear_expresion(Menor)`.
    ///   3. Verifica que el siguiente token sea `)`.
    ///      - Si lo es, lo consume y retorna la expresión interior.
    ///      - Si no, registra error y retorna None.
    ///
    /// # Control de flujo
    /// - `self.parsear_expresion(Precedencia::Menor)?` —
    ///   El operador `?` propaga el None si falla el parseo interno.
    fn parsear_agrupacion(&mut self) -> Option<Expression> {
        self.avanzar();
        let expr = self.parsear_expresion(Precedencia::Menor)?;
        if self.token_actual == Token::ParentesisCerrado {
            self.avanzar();
            Some(expr)
        } else {
            let mensaje =
                "Error de sintaxis: se esperaba ')' para cerrar la expresión agrupada"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            None
        }
    }

    /// Parsea un operador unario: `-expr`, `+expr`, `!expr`
    ///
    /// # Flujo
    ///   1. Clona el operador desde `token_actual` (necesita ownership).
    ///   2. Consume el operador con `avanzar()`.
    ///   3. Parsea el operando con `Precedencia::Prefijo`, la precedencia
    ///      más alta después de Llamada. Esto asegura que `-1 + 2` se
    ///      interprete como `(-1) + 2` y no como `-(1 + 2)`.
    ///   4. Construye `Expression::OperacionUnaria` con `Box::new(derecha)`.
    ///
    /// # Gestión de memoria (Box)
    /// - `Box::new(derecha)` mueve la expresión al heap. El Box es un puntero
    ///   inteligente de tamaño fijo (8 bytes en 64 bits), lo que permite que
    ///   el enum `Expression` tenga tamaño constante independientemente del
    ///   tamaño real de la expresión anidada.
    /// - `operador` (Token clonado) se almacena directamente en el enum;
    ///   Token sin datos internos (Suma, Resta, Not) son Copy.
    fn parsear_unario(&mut self) -> Option<Expression> {
        let operador = self.avanzar();
        let derecha = self.parsear_expresion(Precedencia::Prefijo)?;
        Some(Expression::OperacionUnaria {
            operador,
            derecha: Box::new(derecha),
        })
    }

    // -----------------------------------------------------------------------
    // Parseo infijo (operadores binarios)
    // -----------------------------------------------------------------------

    /// Parsea el lado derecho de un operador binario y construye el nodo
    /// `Expression::OperacionBinaria` correspondiente.
    ///
    /// # Precondición
    /// Al entrar, `token_actual` debe ser el operador infijo (ya consumido
    /// por el bucle Pratt mediante `self.avanzar()`).
    ///
    /// # Flujo
    ///   1. Se clona el operador desde `token_actual` (necesitamos ownership
    ///      para almacenarlo en el AST; `token_actual` se sobrescribirá en
    ///      la siguiente llamada a `avanzar()`).
    ///   2. Se obtiene la precedencia del operador con `precedencia_actual()`.
    ///   3. Se avanza al siguiente token (el primer token de la expresión
    ///      derecha).
    ///   4. Se llama recursivamente a `parsear_expresion(prec)` con la
    ///      precedencia del operador como mínimo. Esto asegura que
    ///      operadores de mayor precedencia en el lado derecho se agrupen
    ///      antes de retornar.
    ///
    /// # Gestión de memoria (Box)
    /// - `izquierda` se recibe por ownership (move). Esto evita clonar la
    ///   expresión ya parseada.
    /// - `Box::new(izquierda)` mueve la expresión a una asignación en heap.
    ///   El Box es un puntero inteligente de 8 bytes (en 64 bits) que
    ///   ownership del heap allocation.
    /// - Lo mismo aplica para `Box::new(derecha)`.
    /// - `operador` (Token clonado) también se mueve al nodo.
    /// - El nodo `OperacionBinaria` completo se retorna por valor (se
    ///   mueve al llamante).
    fn parsear_expresion_infija(&mut self, izquierda: Expression) -> Option<Expression> {
        // Si el token actual es `(`, es una llamada a función.
        // Se delega inmediatamente sin pasar por el flujo de operadores binarios.
        if self.token_actual == Token::ParentesisAbierto {
            return self.parsear_llamada(izquierda);
        }

        // Si el token actual es `[`, es un acceso por índice.
        if self.token_actual == Token::CorcheteAbierto {
            return self.parsear_acceso_indice(izquierda);
        }

        // Si el token actual es `.`, es un acceso por punto (azúcar
        // sintáctico para diccionarios: obj.prop ≡ obj["prop"]).
        if self.token_actual == Token::Punto {
            return self.parsear_acceso_punto(izquierda);
        }

        let prec = self.precedencia_actual();
        let operador = self.avanzar();
        // Parsear la expresión derecha con la precedencia del operador
        // como mínimo. El `?` propaga None si falla.
        let derecha = self.parsear_expresion(prec)?;
        // Construir el AST. Cada Box::new asigna en heap (allocación única
        // de 16/24 bytes según la expresión). La ownership de izquierda
        // y derecha se mueve dentro de los Box.
        Some(Expression::OperacionBinaria {
            izquierda: Box::new(izquierda),
            operador,
            derecha: Box::new(derecha),
        })
    }

    /// Parsea una llamada a función: `funcion(<arg>, <arg>, ...)`
    ///
    /// # Precondición
    /// Al entrar, `funcion` es la expresión izquierda (normalmente un
    /// Identificador) y `token_actual` es `ParentesisAbierto`.
    ///
    /// # Flujo
    ///   1. Consume el `(` con `self.avanzar()`.
    ///   2. Si el token actual no es `)`, itera parseando argumentos
    ///      separados por comas:
    ///      a. Parsea una expresión con `parsear_expresion(Menor)`.
    ///      b. Si hay una coma, la consume y continúa.
    ///      c. Si no hay coma pero tampoco `)`, error.
    ///   3. Verifica que el token actual sea `)`. Si no, error.
    ///   4. Consume el `)`.
    ///   5. Retorna `Expression::Llamada { funcion, argumentos }`.
    ///
    /// # Gestión de memoria
    /// - `funcion` se recibe por ownership (move) y se almacena en
    ///   `Box::new(funcion)`. El Box coloca la expresión en heap, evitando
    ///   que el enum Expression tenga recursión infinita de tamaño.
    /// - `argumentos: Vec<Expression>` crece dinámicamente en heap. Cada
    ///   expresión se mueve al vector sin clonación.
    fn parsear_llamada(&mut self, funcion: Expression) -> Option<Expression> {
        // Consumir el `(`
        self.avanzar();

        let mut argumentos = Vec::with_capacity(4);

        if self.token_actual != Token::ParentesisCerrado {
            loop {
                // Parsear un argumento. El `?` propaga None si falla.
                let arg = self.parsear_expresion(Precedencia::Menor)?;
                argumentos.push(arg);

                // Si sigue una coma, consumirla y continuar.
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::ParentesisCerrado {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Verificar el cierre del paréntesis.
        if self.token_actual != Token::ParentesisCerrado {
            let mensaje =
                "Error de sintaxis: se esperaba ')' o ',' en la lista de argumentos"
                    .to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        Some(Expression::Llamada {
            funcion: Box::new(funcion),
            argumentos,
        })
    }

    // -----------------------------------------------------------------------
    // parsear_arreglo — Parsea un literal de arreglo: `[<expr>, <expr>, ...]`
    // -----------------------------------------------------------------------
    //
    // # Precondición
    // Al entrar, `token_actual` es `CorcheteAbierto`.
    //
    // # Flujo
    //   1. Consume el `[`.
    //   2. Si el token actual no es `]`, itera parseando expresiones
    //      separadas por comas (trailing comma soportado).
    //   3. Exige el cierre `]`.
    //   4. Retorna `Expression::Arreglo(elementos)`.
    //
    // # Gestión de memoria
    // `elementos: Vec<Expression>` crece dinámicamente en heap. Cada
    // expresión se mueve al vector sin clonación. Al retornar, el Vec
    // completo se mueve al enum Expression.
    fn parsear_arreglo(&mut self) -> Option<Expression> {
        // Consumir el `[`.
        self.avanzar();

        let mut elementos = Vec::with_capacity(4);

        if self.token_actual != Token::CorcheteCerrado {
            loop {
                // Parsear un elemento. `?` propaga None si falla.
                let expr = self.parsear_expresion(Precedencia::Menor)?;
                elementos.push(expr);

                // Si sigue una coma, consumirla y continuar.
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    // Soporte para trailing comma: si después de la coma
                    // viene `]`, salimos del bucle.
                    if self.token_actual == Token::CorcheteCerrado {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Verificar el cierre del corchete.
        if self.token_actual == Token::CorcheteCerrado {
            self.avanzar();
            Some(Expression::Arreglo(elementos))
        } else {
            let mensaje =
                "Error de sintaxis: se esperaba ']' para cerrar el arreglo".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            None
        }
    }

    // -----------------------------------------------------------------------
    // parsear_acceso_indice — Parsea acceso por índice: `expr[<indice>]`
    // -----------------------------------------------------------------------
    //
    // # Precondición
    // Al entrar, `token_actual` es `CorcheteAbierto` e `izquierda` es la
    // expresión que se indexa (ej. un identificador de arreglo).
    //
    // # Flujo
    //   1. Consume el `[`.
    //   2. Parsea la expresión del índice con `parsear_expresion(Menor)`.
    //   3. Exige el cierre `]`.
    //   4. Retorna `Expression::AccesoIndice { izquierda, indice }`.
    //
    // # Gestión de memoria (Box)
    // - `izquierda` se recibe por ownership (move) y se almacena en
    //   `Box::new(izquierda)` en el heap.
    // - `indice` se parsea y se envuelve en `Box::new(indice)`.
    fn parsear_acceso_indice(&mut self, izquierda: Expression) -> Option<Expression> {
        // Consumir el `[`.
        self.avanzar();

        // Parsear la expresión del índice con precedencia mínima para
        // permitir expresiones completas dentro de los corchetes.
        let indice = self.parsear_expresion(Precedencia::Menor)?;

        // Verificar el cierre del corchete.
        if self.token_actual == Token::CorcheteCerrado {
            self.avanzar();
            Some(Expression::AccesoIndice {
                izquierda: Box::new(izquierda),
                indice: Box::new(indice),
            })
        } else {
            let mensaje =
                "Error de sintaxis: se esperaba ']' para cerrar el índice".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            None
        }
    }

    // -----------------------------------------------------------------------
    // parsear_acceso_punto — Parsea acceso por punto: `expr.propiedad`
    // -----------------------------------------------------------------------
    //
    // Azúcar sintáctico que transforma `expr.identificador` en el nodo de
    // acceso por índice ya existente: `expr["identificador"]`. Esto delega
    // toda la responsabilidad de evaluación (incluyendo búsqueda en
    // diccionarios, verificación de tipos, y error handling) al bloque
    // lógico preexistente de AccesoIndice, ahorrando ciclos de desarrollo
    // y reduciendo el tamaño del binario al no duplicar lógica.
    //
    // # Precondición
    // Al entrar, `token_actual` es `Punto` e `izquierda` es la expresión
    // que se está indexando (ej. un identificador de diccionario).
    //
    // # Flujo
    //   1. Consume el `.` con `self.avanzar()`.
    //   2. El siguiente token DEBE ser un Identificador. Si no lo es,
    //      registra un error y retorna None.
    //   3. Extrae el nombre del identificador (String), lo clona, y
    //      consume el token con `self.avanzar()`.
    //   4. Construye y retorna `Expression::AccesoIndice` con la izquierda
    //      original como contenedor y `Expression::Cadena(nombre)` como
    //      índice. La Cadena se convierte en `Objeto::Cadena` durante la
    //      evaluación, y luego en `LlaveHash::Cadena` para la búsqueda
    //      en el HashMap del diccionario.
    //
    // # Gestión de memoria
    // `nombre.clone()` crea una copia heap del String del identificador.
    // Es necesaria porque `self.avanzar()` muta el parser, invalidando
    // la referencia prestada al interior del Token. El String clonado se
    // mueve primero a `Expression::Cadena` y luego a `Box::new(...)`
    // dentro del `AccesoIndice`.
    fn parsear_acceso_punto(&mut self, izquierda: Expression) -> Option<Expression> {
        // Consumir el `.`.
        self.avanzar();

        // El token siguiente debe ser un identificador (nombre de propiedad).
        // Si no lo es, se registra un error y se aborta.
        match self.avanzar() {
            Token::Identificador(nombre_propiedad) => {
                Some(Expression::AccesoIndice {
                    izquierda: Box::new(izquierda),
                    indice: Box::new(Expression::Cadena(nombre_propiedad)),
                })
            }
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba un nombre de propiedad \
                     después del '.'"
                        .to_string(),
                ));
                None
            }
        }
    }

    // -----------------------------------------------------------------------
    // parsear_diccionario — Parsea un literal de diccionario: `{k: v, ...}`
    // -----------------------------------------------------------------------
    //
    // # Precondición
    // Al entrar, `token_actual` es `LlaveAbierta`.
    //
    // # Flujo
    //   1. Consume el `{`.
    //   2. Si el token actual no es `}`, itera parseando pares clave:valor
    //      separados por comas (trailing comma soportado).
    //      a. Parsea la expresión de la clave.
    //      b. Espera y consume `:` (DosPuntos).
    //      c. Parsea la expresión del valor.
    //      d. Si hay una coma, la consume; si hay `}`, sale del bucle.
    //   3. Exige el cierre `}`.
    //   4. Retorna `Expression::Diccionario(pares)`.
    //
    // # Gestión de memoria
    // `pares: Vec<(Expression, Expression)>` crece dinámicamente en heap.
    // Cada par (clave, valor) se mueve al vector sin clonación. Al retornar,
    // el Vec completo se mueve al enum Expression.
    fn parsear_diccionario(&mut self) -> Option<Expression> {
        // Consumir el `{`.
        self.avanzar();

        let mut pares = Vec::with_capacity(4);

        if self.token_actual != Token::LlaveCerrada {
            loop {
                // Parsear la expresión de la clave.
                let clave = self.parsear_expresion(Precedencia::Menor)?;

                // Esperar y consumir `:` entre clave y valor.
                if self.token_actual != Token::DosPuntos {
                    let mensaje = "Error de sintaxis: se esperaba ':' después de la clave \
                                   del diccionario".to_string();
                    self.errores.push(self.error_ubicacion(mensaje));
                    return None;
                }
                self.avanzar();

                // Parsear la expresión del valor.
                let valor = self.parsear_expresion(Precedencia::Menor)?;

                pares.push((clave, valor));

                // Si sigue una coma, consumirla y decidir si continuar.
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    // Soporte para trailing comma: si después de la coma
                    // viene `}`, salimos del bucle.
                    if self.token_actual == Token::LlaveCerrada {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Verificar el cierre de la llave.
        if self.token_actual == Token::LlaveCerrada {
            self.avanzar();
            Some(Expression::Diccionario(pares))
        } else {
            let mensaje =
                "Error de sintaxis: se esperaba '}' para cerrar el diccionario".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            None
        }
    }

    // -----------------------------------------------------------------------
    // parsear_throw — Parsea una expresión throw: `throw <expr>`
    // -----------------------------------------------------------------------
    fn parsear_throw(&mut self) -> Option<Expression> {
        self.avanzar();
        let expr = self.parsear_expresion(Precedencia::Menor)?;
        Some(Expression::Throw(Box::new(expr)))
    }

    // parsear_import — Parsea una expresión de importación: `import "ruta"`
    // -----------------------------------------------------------------------
    //
    // Sintaxis: `import <cadena>`
    //
    // # Precondición
    // token_actual es Token::Import.
    //
    // # Flujo
    //   1. Consume el token `import`.
    //   2. Verifica que el token actual sea Token::Cadena(ruta).
    //      - Si no lo es, registra error y retorna None.
    //   3. Extrae y clona la ruta, consume el token.
    //   4. Retorna Expression::Import(ruta).
    //
    // # Gestión de memoria
    // La ruta String se clona del Token (el AST debe ser autónomo;
    // no puede retener referencias al Token, que se sobrescribe en
    // cada avanzar()). La clonación aloca heap (copia O(n)).
    fn parsear_import(&mut self) -> Option<Expression> {
        self.avanzar();
        // Detectar `import { foo, bar } from "mod"`
        if self.token_actual == Token::LlaveAbierta {
            return self.parsear_import_selectivo();
        }
        match self.avanzar() {
            Token::Cadena(ruta) => Some(Expression::Import(ruta)),
            _ => {
                let mensaje = format!(
                    "Error de sintaxis: se esperaba una ruta de archivo como cadena \
                     después de 'import', pero se encontró {:?}",
                    self.token_actual
                );
                self.errores.push(self.error_ubicacion(mensaje));
                None
            }
        }
    }

    /// Parsea `import { foo, bar } from "modulo"`
    fn parsear_import_selectivo(&mut self) -> Option<Expression> {
        // Ya consumimos `import`, token_actual es `{`
        self.avanzar(); // consumir `{`
        let mut nombres = Vec::new();
        while let Token::Identificador(n) = &self.token_actual {
            nombres.push(n.clone());
            self.avanzar();
            if self.token_actual == Token::Coma {
                self.avanzar();
            } else {
                break;
            }
        }
        if self.token_actual != Token::LlaveCerrada {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba '}' después de la lista de importaciones".to_string(),
            ));
            return None;
        }
        self.avanzar(); // consumir `}`
        // Esperar `from`
        if self.token_actual != Token::Identificador("from".to_string()) {
            self.errores.push(self.error_ubicacion(
                "Error de sintaxis: se esperaba 'from' después de '}'".to_string(),
            ));
            return None;
        }
        self.avanzar(); // consumir `from`
        // Esperar cadena con el módulo
        match self.avanzar() {
            Token::Cadena(modulo) => Some(Expression::ImportSelectivo {
                nombres,
                modulo,
            }),
            _ => {
                self.errores.push(self.error_ubicacion(
                    "Error de sintaxis: se esperaba una ruta de módulo como cadena después de 'from'".to_string(),
                ));
                None
            }
        }
    }

    // -----------------------------------------------------------------------
    // parsear_fn_expr — Parsea una expresión de función anónima: `fn(...) {...}`
    // -----------------------------------------------------------------------
    //
    // Sintaxis: `fn ( <parametros> ) { <cuerpo> }`
    //
    // # Precondición
    // token_actual es Token::Fn.
    //
    // # Flujo
    //   1. Consume el token `fn`.
    //   2. NO espera un nombre (a diferencia de parsear_declaracion_funcion).
    //   3. Espera `(` para la lista de parámetros.
    //   4. Itera sobre identificadores separados por comas (vacío si `)`).
    //   5. Espera `)` al final de la lista.
    //   6. Espera `{` para el cuerpo: delega en `parsear_bloque()`.
    //   7. Retorna Expression::Funcion { parametros, cuerpo }.
    //
    // # Gestión de memoria
    // parametros: Vec<String> crece en heap. Cada nombre se clona del token.
    // cuerpo: Box<Statement> envuelve el Bloque en el heap para dar tamaño
    // fijo a Expression.
    fn parsear_fn_expr(&mut self) -> Option<Expression> {
        // Paso 1: Consumir la palabra clave `fn`.
        self.avanzar();

        // Paso 2: Consumir `(` para la lista de parámetros.
        if self.token_actual != Token::ParentesisAbierto {
            let mensaje =
                "Error de sintaxis: se esperaba '(' después de 'fn'".to_string();
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // Paso 3: Parsear la lista de parámetros (con tipos opcionales).
        let mut parametros: Vec<String> = Vec::with_capacity(4);
        let mut tipos_parametros: Vec<Option<String>> = Vec::with_capacity(4);
        if self.token_actual != Token::ParentesisCerrado {
            loop {
                match self.avanzar() {
                    Token::Identificador(param) => {
                        let tipo = if self.token_actual == Token::DosPuntos {
                            self.avanzar();
                            match self.avanzar() {
                                Token::Identificador(t) => Some(t),
                                _ => {
                                    let mensaje = "Error de sintaxis: se esperaba un tipo después de ':'".to_string();
                                    self.errores.push(self.error_ubicacion(mensaje));
                                    return None;
                                }
                            }
                        } else {
                            None
                        };
                        parametros.push(param);
                        tipos_parametros.push(tipo);
                    }
                    _ => {
                        let mensaje = format!(
                            "Error de sintaxis: se esperaba un parámetro, \
                             pero se encontró {:?}",
                            self.token_actual
                        );
                        self.errores.push(self.error_ubicacion(mensaje));
                        return None;
                    }
                }
                if self.token_actual == Token::Coma {
                    self.avanzar();
                    if self.token_actual == Token::ParentesisCerrado {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Paso 4: Consumir `)`.
        if self.token_actual != Token::ParentesisCerrado {
            let mensaje = format!(
                "Error de sintaxis: se esperaba ')' después de los parámetros, \
                 pero se encontró {:?}",
                self.token_actual
            );
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        self.avanzar();

        // Paso 4b: Parsear tipo de retorno opcional `: Tipo`.
        let tipo_retorno = if self.token_actual == Token::DosPuntos {
            self.avanzar();
            match self.avanzar() {
                Token::Identificador(t) => Some(t),
                _ => {
                    let mensaje = "Error de sintaxis: se esperaba un tipo después de ':'".to_string();
                    self.errores.push(self.error_ubicacion(mensaje));
                    return None;
                }
            }
        } else {
            None
        };

        // Paso 5: Esperar `{` para el cuerpo.
        if self.token_actual != Token::LlaveAbierta {
            let mensaje = format!(
                "Error de sintaxis: se esperaba '{{' para el cuerpo \
                 de la función, pero se encontró {:?}",
                self.token_actual
            );
            self.errores.push(self.error_ubicacion(mensaje));
            return None;
        }
        let cuerpo = match self.parsear_bloque() {
            Some(stmt) => Box::new(stmt),
            None => return None,
        };

        // Paso 6: Retornar el nodo AST.
        Some(Expression::Funcion {
            parametros,
            cuerpo,
            tipos_parametros,
            tipo_retorno,
        })
    }

    // -----------------------------------------------------------------------
    // Tablas de precedencia (actual y siguiente)
    // -----------------------------------------------------------------------

    /// Retorna la precedencia del token actual como operador infijo.
    ///
    /// Se usa dentro de `parsear_expresion_infija` para determinar con
    /// qué nivel de precedencia parsear el lado derecho.
    ///
    /// El match evalúa `self.token_actual` por referencia para no
    /// consumirlo. Los operadores se agrupan por nivel de precedencia
    /// ascendente (definido en el enum `Precedencia`).
    fn precedencia_actual(&self) -> Precedencia {
        match &self.token_actual {
            // Bitwise OR — precedencia más baja entre los operadores
            Token::Pipe => Precedencia::BitOr,
            // Bitwise XOR
            Token::Circunflejo => Precedencia::BitXor,
            // Bitwise AND
            Token::Ampersand => Precedencia::BitAnd,
            // Igualdad
            Token::Igual | Token::Diferente => Precedencia::Igualdad,
            // Comparación relacional
            Token::MenorQue
            | Token::MayorQue
            | Token::MenorOIgual
            | Token::MayorOIgual => Precedencia::Comparacion,
            // Desplazamiento de bits
            Token::DesplazamientoIzq | Token::DesplazamientoDer => {
                Precedencia::Shift
            }
            // Suma y resta
            Token::Suma | Token::Resta => Precedencia::Suma,
            // Multiplicación, división y módulo
            Token::Multiplicacion | Token::Division | Token::Modulo => {
                Precedencia::Multiplicacion
            }
            // Llamada a función — `(` como operador infijo
            Token::ParentesisAbierto => Precedencia::Llamada,
            // Acceso por índice — `[` como operador infijo (máxima precedencia)
            Token::CorcheteAbierto => Precedencia::Indice,
            // Acceso por punto — `.` como operador infijo (misma precedencia)
            Token::Punto => Precedencia::Indice,
            // Cualquier otro token no es operador infijo.
            _ => Precedencia::Menor,
        }
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Prefija un mensaje con la ubicación actual (token_actual) para errores
    /// sintácticos. Formato: `[Línea X, Columna Y] <mensaje>`
    /// Columna se convierte a 1-based para el usuario: internamente el lexer
    /// usa columna=0 tras un salto de línea (el siguiente carácter la
    /// incrementa a 1), así que mostramos max(col, 1).
    fn error_ubicacion(&self, mensaje: String) -> String {
        let col = if self.token_columna == 0 { 1 } else { self.token_columna };
        format!(
            "[Línea {}, Columna {}] {}",
            self.token_linea, col, mensaje
        )
    }

    /// Registra un error sintáctico cuando `token_siguiente` no coincide
    /// con lo esperado (lookahead). Reporta el token encontrado como
    /// `token_siguiente` porque es el que causó la falla en la predicción.
    ///
    /// # Parámetros
    /// - `esperado`: descripción textual del token que se esperaba.
    ///
    /// # Gestión de memoria
    /// - `format!` crea un String en heap con el mensaje formateado.
    /// - `self.errores.push(mensaje)` mueve el String al Vec.
    fn error_token_esperado(&mut self, esperado: &str) {
        let col = if self.siguiente_columna == 0 { 1 } else { self.siguiente_columna };
        self.errores.push(format!(
            "[Línea {}, Columna {}] Error de sintaxis: se esperaba {}, pero se encontró {:?}",
            self.siguiente_linea, col,
            esperado, self.token_siguiente
        ));
    }
}
