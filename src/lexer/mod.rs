
pub mod token;
use token::Token;
use std::str::Chars;
use std::iter::Peekable;

/// Rastrea una cadena interpolada en curso para poder retomar su lectura
/// después de haber emitido los tokens de la expresión interior.
struct InterpoloCadena {
    /// Profundidad de llaves `{`/`}` dentro de la expresión interpolada.
    /// En 0, el próximo `}` cierra la interpolación y reanuda la cadena.
    profundidad: u32,
}

/// El Lexer se encarga de escanear el código fuente y agrupar los caracteres en Tokens lógicos.
pub struct Lexer<'a> {
    fuente: Peekable<Chars<'a>>, // Iterador eficiente sobre los caracteres
    caracter_actual: Option<char>,
    pub linea: usize,
    pub columna: usize,
    // Posición de inicio del último token producido por next().
    // Se captura después de saltar_espacios() y antes de leer el token,
    // para que el Parser pueda reportar coordenadas exactas en errores.
    pub token_linea: usize,
    pub token_columna: usize,
    // Estado de una cadena interpolada en curso ("..." con expresiones {expr}).
    // `None` indica que el lexer no está a medio camino de una interpolación.
    interpolo: Option<InterpoloCadena>,
}

impl<'a> Lexer<'a> {
    /// Inicializa un nuevo Lexer con el código fuente proporcionado.
    pub fn nuevo(texto: &'a str) -> Self {
        let mut lexer = Lexer {
            fuente: texto.chars().peekable(),
            caracter_actual: None,
            linea: 1,
            columna: 0,
            token_linea: 1,
            token_columna: 1,
            interpolo: None,
        };
        lexer.avanzar(); // Cargar el primer carácter
        lexer
    }

    /// Avanza al siguiente carácter en el código fuente de forma segura.
    fn avanzar(&mut self) {
        self.caracter_actual = self.fuente.next();
        
        if let Some(c) = self.caracter_actual {
            if c == '\n' {
                self.linea += 1;
                self.columna = 0;
            } else {
                self.columna += 1;
            }
        }
    }

    /// Echa un vistazo al siguiente carácter sin consumirlo (útil para operadores de dos caracteres como `==` o `!=`).
    fn mirar_siguiente(&mut self) -> Option<&char> {
        self.fuente.peek()
    }

    /// Ignora los espacios en blanco, tabulaciones y saltos de línea.
    fn saltar_espacios(&mut self) {
        while let Some(c) = self.caracter_actual {
            if c.is_whitespace() {
                self.avanzar();
            } else {
                break;
            }
        }
    }

    /// Agrupa dígitos consecutivos y determina si el literal es entero (i64) o flotante (f64).
    /// `primer_digito` ya está en `caracter_actual` (consumido del iterador por una llamada
    /// anterior a `avanzar()`). Al finalizar, `caracter_actual` queda apuntando al primer
    /// carácter después del número, listo para la siguiente iteración del Iterator.
    fn leer_numero(&mut self, primer_digito: char) -> Token {
        let mut literal = String::new();
        literal.push(primer_digito);

        // Consumir dígitos enteros adicionales
        while let Some(&c) = self.mirar_siguiente() {
            if c.is_ascii_digit() {
                self.avanzar();
                literal.push(c);
            } else {
                break;
            }
        }

        if let Some(&'.') = self.mirar_siguiente() {
            self.avanzar();
            literal.push('.');

            // Consumir dígitos fraccionarios
            while let Some(&c) = self.mirar_siguiente() {
                if c.is_ascii_digit() {
                    self.avanzar();
                    literal.push(c);
                } else {
                    break;
                }
            }

            // Avanzar más allá del último dígito consumido para que
            // caracter_actual apunte al siguiente carácter a lexear.
            self.avanzar();
            Token::Flotante(literal.parse::<f64>().unwrap_or(0.0))
        } else {
            // Avanzar más allá del último dígito de la parte entera.
            self.avanzar();
            Token::Entero(literal.parse::<i64>().unwrap_or(0))
        }
    }

    /// Lee una cadena de comillas dobles, cortándola en cada interpolación `{expr}`.
    /// Se asume que `caracter_actual` apunta al primer carácter del contenido (la
    /// comilla de apertura ya fue consumida por el llamante) y que la posición del
    /// token ya fue capturada.
    ///
    /// Devuelve `Some(Token::Cadena(..))` con el segmento literal leído, o `None`
    /// cuando el segmento es vacío y pertenece a una cadena interpolada (ej: los
    /// bordes de `"{x}"`), para que el llamante siga lexando sin emitir `Cadena("")`.
    ///
    /// `hay_interpolacion_anterior` indica si esta misma cadena ya contuvo una
    /// interpolación; en ese caso un segmento final vacío también se omite.
    fn leer_cadena(&mut self, hay_interpolacion_anterior: bool) -> Option<Token> {
        let mut literal = String::new();

        loop {
            match self.caracter_actual {
                Some('"') => {
                    self.avanzar();
                    if hay_interpolacion_anterior && literal.is_empty() {
                        return None;
                    }
                    return Some(Token::Cadena(literal));
                }
                Some('{') => {
                    // Inicio de interpolación: emitir el texto previo (si existe)
                    // y activar el modo expresión hasta el `}` que balancee.
                    self.avanzar();
                    self.interpolo = Some(InterpoloCadena { profundidad: 0 });
                    if literal.is_empty() {
                        return None;
                    }
                    return Some(Token::Cadena(literal));
                }
                Some('\\') => {
                    self.avanzar();
                    match self.caracter_actual {
                        Some('"')  => { literal.push('"');  self.avanzar(); }
                        Some('\\') => { literal.push('\\'); self.avanzar(); }
                        Some('n')  => { literal.push('\n'); self.avanzar(); }
                        Some('r')  => { literal.push('\r'); self.avanzar(); }
                        Some('t')  => { literal.push('\t'); self.avanzar(); }
                        // Llaves escapadas: se incluyen literalmente y NO inician
                        // interpolación (ej: "\{no interpolado\}").
                        Some('{')  => { literal.push('{');  self.avanzar(); }
                        Some('}')  => { literal.push('}');  self.avanzar(); }
                        Some(c)    => {
                            literal.push('\\');
                            literal.push(c);
                            self.avanzar();
                        }
                        None => return Some(Token::Ilegal('"')),
                    }
                }
                Some(c) => {
                    literal.push(c);
                    self.avanzar();
                }
                None => {
                    return Some(Token::Ilegal('"'));
                }
            }
        }
    }

    /// Emite el siguiente token de una cadena (posiblemente interpolada). Si la
    /// cadena empieza con interpolación y no tiene texto literal inicial, continúa
    /// lexando la expresión en lugar de emitir un `Cadena("")` intermedio.
    fn siguiente_token_cadena(&mut self, hay_interpolacion_anterior: bool) -> Option<Token> {
        match self.leer_cadena(hay_interpolacion_anterior) {
            Some(token) => Some(token),
            None => self.next(),
        }
    }

    /// Consume el `}` que cierra una interpolación (profundidad 0) y reanuda la
    /// lectura de la cadena que la contenía. El `}` de cierre NO se emite como
    /// token: solo delimita la expresión.
    fn cerrar_interpolacion(&mut self) -> Option<Token> {
        self.avanzar(); // consumir el '}'
        self.interpolo = None;
        self.siguiente_token_cadena(true)
    }

    /// Consume un comentario multilínea `/* ... */` de forma plana (sin anidar:
    /// el primer `*/` lo cierra). Se asume que `caracter_actual` es el `*` que
    /// sigue a la `/` de apertura.
    ///
    /// Devuelve `false` si se alcanza el EOF sin encontrar el cierre `*/`.
    fn saltar_comentario_multilinea(&mut self) -> bool {
        self.avanzar(); // consumir el '*' de apertura
        loop {
            // `Option<char>` es Copy: se copia para poder pedir el peek mutable
            // del siguiente carácter sin conflicto de borrow en el guard.
            let actual = self.caracter_actual;
            let cierra = actual == Some('*') && self.mirar_siguiente() == Some(&'/');
            match actual {
                Some(_) if cierra => {
                    self.avanzar(); // consumir '*'
                    self.avanzar(); // consumir '/'
                    return true;
                }
                Some(_) => {
                    self.avanzar();
                }
                None => return false,
            }
        }
    }

    /// Lee una secuencia de caracteres alfanuméricos (o guiones bajos)
    /// y determina si es una palabra clave de Argo o un identificador de variable/función.
    fn leer_identificador_o_palabra_clave(&mut self, primer_caracter: char) -> Token {
        let mut resultado = String::new();
        resultado.push(primer_caracter);

        // Mientras el siguiente carácter sea alfanumérico o guion bajo, se sigue agrupando
        while let Some(&c) = self.mirar_siguiente() {
            if c.is_alphanumeric() || c == '_' {
                resultado.push(c);
                self.avanzar();
            } else {
                break;
            }
        }

        // Avanzar más allá del último carácter consumido para que
        // caracter_actual apunte al siguiente carácter a lexear.
        self.avanzar();

        // Se verifica si la palabra agrupada coincide con alguna palabra sagrada del lenguaje.
        // Las keywords en inglés son canónicas; sus alias en español producen el MISMO token,
        // así el parser, AST y evaluator no requieren cambios.
        /* Keywords duales ESP/ENG */
        match resultado.as_str() {
            "let" | "sea" => Token::Let,
            "const" | "constante" => Token::Const,
            "fn" | "funcion" | "func" => Token::Fn,
            "class" => Token::Class,
            "constructor" => Token::Constructor,
            "super" => Token::Super,
            "this" => Token::This,
            "if" | "si" => Token::If,
            "else" | "sino" => Token::Else,
            "while" | "mientras" => Token::While,
            "for" | "para" => Token::For,
            "break" | "rompe" => Token::Break,
            "continue" | "continua" => Token::Continue,
            "return" | "retorna" | "devuelve" => Token::Return,
            "try" | "intenta" => Token::Try,
            "catch" | "captura" => Token::Catch,
            "import" | "importa" => Token::Import,
            "struct" | "estructura" => Token::Struct,
            "match" | "coincide" => Token::Match,
            "throw" | "lanza" => Token::Throw,
            "enum" | "enumeracion" => Token::Enum,
            "extern" | "externa" => Token::Extern,
            "async" | "asincrono" => Token::Async,
            "await" | "espera" => Token::Await,
            "true" | "verdadero" => Token::True,
            "false" | "falso" => Token::False,
            "null" | "nulo" | "nada" => Token::Null,
            _ => Token::Identificador(resultado),
        }
    }

    /// Empareja un símbolo u operador con su Token, sin consumir el carácter
    /// actual (el llamante hace `self.avanzar()` tras retornar).
    ///
    /// Devuelve `None` cuando se consumió un comentario (`//` o `/* */`) y el
    /// llamante debe re-lexear desde la nueva posición del iterador.
    fn leer_simbolo(&mut self, c: char) -> Option<Token> {
        let token = match c {
            // Operadores de dos caracteres (Peek)
            '=' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::Igual
                } else if self.mirar_siguiente() == Some(&'>') {
                    self.avanzar();
                    Token::Flecha
                } else {
                    Token::Asignacion
                }
            },
            '!' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::Diferente
                } else {
                    Token::Not
                }
            },
            '+' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::SumaAsignacion
                } else {
                    Token::Suma
                }
            },
            '-' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::RestaAsignacion
                } else {
                    Token::Resta
                }
            },
            '*' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::MultiplicacionAsignacion
                } else {
                    Token::Multiplicacion
                }
            },
            '/' => {
                if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::DivisionAsignacion
                } else if self.mirar_siguiente() == Some(&'/') {
                    // Comentario de línea: // ... hasta \n
                    while let Some(c) = self.caracter_actual {
                        if c == '\n' {
                            break;
                        }
                        self.avanzar();
                    }
                    return None;
                } else if self.mirar_siguiente() == Some(&'*') {
                    // Comentario multilínea: /* ... */ (plano, sin anidar).
                    // Si falta el cierre */ se emite Ilegal('*') como error léxico
                    // descriptivo en lugar de crashear.
                    self.avanzar(); // consumir la '/' de apertura
                    if !self.saltar_comentario_multilinea() {
                        return Some(Token::Ilegal('*'));
                    }
                    return None;
                } else {
                    Token::Division
                }
            },
            '<' => {
                if self.mirar_siguiente() == Some(&'<') {
                    self.avanzar();
                    Token::DesplazamientoIzq
                } else if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::MenorOIgual
                } else {
                    Token::MenorQue
                }
            },
            '>' => {
                if self.mirar_siguiente() == Some(&'>') {
                    self.avanzar();
                    Token::DesplazamientoDer
                } else if self.mirar_siguiente() == Some(&'=') {
                    self.avanzar();
                    Token::MayorOIgual
                } else {
                    Token::MayorQue
                }
            },
            
            // Símbolos de un solo carácter
            '&' => {
                if self.mirar_siguiente() == Some(&'&') {
                    self.avanzar();
                    Token::And
                } else {
                    Token::Ampersand
                }
            },
            '|' => {
                if self.mirar_siguiente() == Some(&'|') {
                    self.avanzar();
                    Token::Or
                } else if self.mirar_siguiente() == Some(&'>') {
                    // Operador pipe `|>`: tubería que reordena argumentos.
                    // Nunca debe confundirse con `|` + `>` separados.
                    self.avanzar();
                    Token::PipeDoble
                } else {
                    Token::Pipe
                }
            },
            '^' => Token::Circunflejo,
            '%' => Token::Modulo,
            // Dentro de una interpolación se rastrea el balanceo de llaves para
            // saber cuál '}' cierra la expresión (ver `cerrar_interpolacion`).
            // Fuera de interpolación el estado es `None` y el token es normal.
            '{' => {
                if let Some(estado) = self.interpolo.as_mut() {
                    estado.profundidad += 1;
                }
                Token::LlaveAbierta
            },
            '}' => {
                if let Some(estado) = self.interpolo.as_mut() {
                    estado.profundidad -= 1;
                }
                Token::LlaveCerrada
            },
            '[' => Token::CorcheteAbierto,
            ']' => Token::CorcheteCerrado,
            '(' => Token::ParentesisAbierto,
            ')' => Token::ParentesisCerrado,
            ',' => Token::Coma,
            ';' => Token::PuntoComa,
            '.' => Token::Punto,
            ':' => {
                if self.mirar_siguiente() == Some(&':') {
                    self.avanzar();
                    Token::DobleDosPuntos
                } else {
                    Token::DosPuntos
                }
            },

            // Si el carácter no coincide con nada, no hacemos crash, emitimos error léxico
            _ => Token::Ilegal(c),
        };

        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // 1. Ignorar espacios muertos antes de leer el siguiente símbolo
        self.saltar_espacios();

        // 2. Capturar la posición de inicio del token antes de leerlo.
        //    En este punto, caracter_actual es el primer carácter del token
        //    (o None si se acabó la entrada), y linea/columna reflejan su
        //    posición gracias a que saltar_espacios() llamó a avanzar().
        self.token_linea = self.linea;
        self.token_columna = self.columna;

        // 3. Obtener el carácter actual, si es None, terminamos el iterador
        let c = match self.caracter_actual {
            Some(c) => c,
            None => {
                // EOF dentro de una interpolación abierta = cadena sin comilla
                // de cierre: se reporta el mismo error léxico que una cadena
                // normal sin terminar, en lugar de terminar silenciosamente.
                if self.interpolo.is_some() {
                    return Some(Token::Ilegal('"'));
                }
                return None;
            }
        };

        // 3.1 Interpolación: dentro de una cadena interpolada, el `}` en
        //     profundidad 0 cierra la expresión y reanuda la cadena. El resto
        //     de los tokens de la expresión se lexan con el mismo despacho.
        if let Some(estado) = self.interpolo.as_ref() {
            if c == '}' && estado.profundidad == 0 {
                return self.cerrar_interpolacion();
            }
        }

        // 3. Evaluar palabras, números o textos (Lógica compleja)
        if c.is_alphabetic() || c == '_' {
            return Some(self.leer_identificador_o_palabra_clave(c));
        } else if c.is_ascii_digit() {
            return Some(self.leer_numero(c));
        } else if c == '"' {
            self.avanzar();
            return self.siguiente_token_cadena(false);
        }

        // 4. Evaluar Símbolos y Operadores (Lógica directa)
        let token = match self.leer_simbolo(c) {
            Some(token) => token,
            // Se consumió un comentario (// o /* */) sin generar token: re-lexear
            None => return self.next(),
        };

        // 5. Consumir el carácter actual del símbolo antes de retornar
        self.avanzar();
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenizar(texto: &str) -> Vec<Token> {
        Lexer::nuevo(texto).collect()
    }

    #[test]
    fn comentario_multilinea_se_omite() {
        let tokens = tokenizar("sea a /* comentario\n de varias\n lineas */ = 1;");
        assert_eq!(
            tokens,
            vec![Token::Let, Token::Identificador("a".into()), Token::Asignacion, Token::Entero(1), Token::PuntoComa]
        );
    }

    #[test]
    fn comentario_multilinea_pegado_sin_espacios() {
        let tokens = tokenizar("1+/* comentario */2");
        assert_eq!(tokens, vec![Token::Entero(1), Token::Suma, Token::Entero(2)]);
    }

    #[test]
    fn comentario_multilinea_sin_cierre_emite_ilegal() {
        let tokens = tokenizar("sea a = 1; /* sin cerrar");
        assert_eq!(
            tokens,
            vec![Token::Let, Token::Identificador("a".into()), Token::Asignacion, Token::Entero(1), Token::PuntoComa, Token::Ilegal('*')]
        );
    }

    #[test]
    fn cadena_simple_sin_interpolacion() {
        assert_eq!(tokenizar("\"Hola Mundo\""), vec![Token::Cadena("Hola Mundo".into())]);
    }

    #[test]
    fn interpolacion_basica() {
        let tokens = tokenizar("\"Hola {mundo}\"");
        assert_eq!(
            tokens,
            vec![Token::Cadena("Hola ".into()), Token::Identificador("mundo".into())]
        );
    }

    #[test]
    fn interpolacion_multiple_segun_especificacion() {
        let tokens = tokenizar("\"Hola {nombre} v{version}\"");
        assert_eq!(
            tokens,
            vec![
                Token::Cadena("Hola ".into()),
                Token::Identificador("nombre".into()),
                Token::Cadena(" v".into()),
                Token::Identificador("version".into()),
            ]
        );
    }

    #[test]
    fn llaves_escapadas_no_interpolan() {
        let tokens = tokenizar(r#""\{no interpolado\}""#);
        assert_eq!(tokens, vec![Token::Cadena("{no interpolado}".into())]);
    }

    #[test]
    fn interpolacion_con_expresion_anidada() {
        let tokens = tokenizar("\"resultado: {objeto.calcular()}\"");
        assert_eq!(
            tokens,
            vec![
                Token::Cadena("resultado: ".into()),
                Token::Identificador("objeto".into()),
                Token::Punto,
                Token::Identificador("calcular".into()),
                Token::ParentesisAbierto,
                Token::ParentesisCerrado,
            ]
        );
    }

    #[test]
    fn interpolacion_balancea_llaves_internas() {
        let tokens = tokenizar("\"x{foo({a: 1})}\"");
        assert_eq!(
            tokens,
            vec![
                Token::Cadena("x".into()),
                Token::Identificador("foo".into()),
                Token::ParentesisAbierto,
                Token::LlaveAbierta,
                Token::Identificador("a".into()),
                Token::DosPuntos,
                Token::Entero(1),
                Token::LlaveCerrada,
                Token::ParentesisCerrado,
            ]
        );
    }

    #[test]
    fn interpolacion_al_inicio_y_sin_texto_final() {
        let tokens = tokenizar("\"{x}\"");
        assert_eq!(tokens, vec![Token::Identificador("x".into())]);
    }

    #[test]
    fn cadena_sin_cerrar_emite_ilegal() {
        assert_eq!(tokenizar("\"abc"), vec![Token::Ilegal('"')]);
    }

    #[test]
    fn pipe_doble_no_confunde_or_ni_pipe_simple() {
        assert_eq!(tokenizar("|>"), vec![Token::PipeDoble]);
        assert_eq!(tokenizar("||"), vec![Token::Or]);
        assert_eq!(tokenizar("|"), vec![Token::Pipe]);
        // `|>` nunca se parte en `|` + `>` separados
        assert_eq!(tokenizar("| >"), vec![Token::Pipe, Token::MayorQue]);
        assert_eq!(
            tokenizar("a |> b | c || d"),
            vec![
                Token::Identificador("a".into()),
                Token::PipeDoble,
                Token::Identificador("b".into()),
                Token::Pipe,
                Token::Identificador("c".into()),
                Token::Or,
                Token::Identificador("d".into()),
            ]
        );
    }
}