
pub mod token;
use token::Token;
use std::str::Chars;
use std::iter::Peekable;

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

    /// Lee y agrupa caracteres entre comillas dobles. La comilla de apertura ya fue
    /// consumida por el llamante (Iterator::next), por lo que `caracter_actual` apunta
    /// al primer carácter del contenido textual.
    fn leer_cadena(&mut self) -> Token {
        let mut literal = String::new();

        loop {
            match self.caracter_actual {
                Some('"') => {
                    self.avanzar();
                    return Token::Cadena(literal);
                }
                Some('\\') => {
                    self.avanzar();
                    match self.caracter_actual {
                        Some('"')  => { literal.push('"');  self.avanzar(); }
                        Some('\\') => { literal.push('\\'); self.avanzar(); }
                        Some('n')  => { literal.push('\n'); self.avanzar(); }
                        Some('r')  => { literal.push('\r'); self.avanzar(); }
                        Some('t')  => { literal.push('\t'); self.avanzar(); }
                        Some(c)    => {
                            literal.push('\\');
                            literal.push(c);
                            self.avanzar();
                        }
                        None => return Token::Ilegal('"'),
                    }
                }
                Some(c) => {
                    literal.push(c);
                    self.avanzar();
                }
                None => {
                    return Token::Ilegal('"');
                }
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

        // Se verifica si la palabra agrupada coincide con alguna palabra sagrada del lenguaje
        match resultado.as_str() {
            "let" => Token::Let,
            "const" => Token::Const,
            "fn" => Token::Fn,
            "class" => Token::Class,
            "constructor" => Token::Constructor,
            "super" => Token::Super,
            "this" => Token::This,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "for" => Token::For,
            "break" => Token::Break,
            "continue" => Token::Continue,
            "return" => Token::Return,
            "try" => Token::Try,
            "catch" => Token::Catch,
            "import" => Token::Import,
            "struct" => Token::Struct,
            "match" => Token::Match,
            "throw" => Token::Throw,
            "enum" => Token::Enum,
            "extern" => Token::Extern,
            "async" => Token::Async,
            "await" => Token::Await,
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            _ => Token::Identificador(resultado),
        }
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
        let c = self.caracter_actual?;

        // 3. Evaluar palabras, números o textos (Lógica compleja)
        if c.is_alphabetic() || c == '_' {
            return Some(self.leer_identificador_o_palabra_clave(c));
        } else if c.is_ascii_digit() {
            return Some(self.leer_numero(c));
        } else if c == '"' {
            self.avanzar();
            return Some(self.leer_cadena());
        }

        // 4. Evaluar Símbolos y Operadores (Lógica directa)
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
                } else {
                    Token::Pipe
                }
            },
            '^' => Token::Circunflejo,
            '*' => Token::Multiplicacion,
            '/' => {
                if self.mirar_siguiente() == Some(&'/') {
                    // Comentario de línea: // ... hasta \n
                    // Consumimos caracteres hasta encontrar \n o EOF.
                    while let Some(c) = self.caracter_actual {
                        if c == '\n' {
                            break;
                        }
                        self.avanzar();
                    }
                    // No retornamos token, seguimos iterando.
                    // Llamamos recursivamente a next() para que el bucle
                    // principal del Iterator continue con el siguiente token.
                    return self.next();
                } else {
                    Token::Division
                }
            },
            '%' => Token::Modulo,
            '{' => Token::LlaveAbierta,
            '}' => Token::LlaveCerrada,
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

        // 5. Consumir el carácter actual del símbolo antes de retornar
        self.avanzar();
        Some(token)
    }
}