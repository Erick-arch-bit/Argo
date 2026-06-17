//
// Módulo estándar json — Serialización y deserialización JSON.
// Expone parse (texto → Objeto) y stringify (Objeto → texto)
// en un Objeto::Diccionario bajo el nombre "json".
//

use std::collections::HashMap;

use crate::evaluator::{LlaveHash, Objeto};

/// Convierte un Objeto de Argo a su representación JSON en String.
fn json_stringificar_valor(valor: &Objeto) -> String {
    match valor {
        Objeto::Nulo => "null".to_string(),
        Objeto::Booleano(b) => {
            if *b { "true".to_string() } else { "false".to_string() }
        }
        Objeto::Entero(i) => i.to_string(),
        Objeto::Flotante(f) => f.to_string(),
        Objeto::Cadena(s) => json_escapar_cadena(s),
        Objeto::Arreglo(v) => {
            let elementos: Vec<String> = v
                .iter()
                .map(json_stringificar_valor)
                .collect();
            format!("[{}]", elementos.join(","))
        }
        Objeto::Diccionario(m) => {
            let pares: Vec<String> = m
                .iter()
                .map(|(k, v)| {
                    let clave = match k {
                        LlaveHash::Cadena(s) => json_escapar_cadena(s),
                        LlaveHash::Entero(i) => i.to_string(),
                        LlaveHash::Booleano(b) => {
                            if *b { "true".to_string() } else { "false".to_string() }
                        }
                    };
                    format!("{}:{}", clave, json_stringificar_valor(v))
                })
                .collect();
            format!("{{{}}}", pares.join(","))
        }
        _ => "\"__tipo_no_soportado__\"".to_string(),
    }
}

/// Escapa una cadena para incluirla dentro de un literal JSON.
fn json_escapar_cadena(s: &str) -> String {
    let mut res = String::with_capacity(s.len() + 2);
    res.push('"');
    for c in s.chars() {
        match c {
            '"' => res.push_str("\\\""),
            '\\' => res.push_str("\\\\"),
            '\n' => res.push_str("\\n"),
            '\r' => res.push_str("\\r"),
            '\t' => res.push_str("\\t"),
            '\u{0008}' => res.push_str("\\b"),
            '\u{000C}' => res.push_str("\\f"),
            c if c.is_ascii_control() => {
                res.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => res.push(c),
        }
    }
    res.push('"');
    res
}

// ---------------------------------------------------------------------------
// Parser JSON interno (cero dependencias externas)
// ---------------------------------------------------------------------------
// Opera sobre una &str mediante un índice de posición.
struct JsonParser<'a> {
    fuente: &'a str,
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn nuevo(fuente: &'a str) -> Self {
        JsonParser { fuente, pos: 0 }
    }

    fn resto(&self) -> &'a str {
        &self.fuente[self.pos..]
    }

    fn avanzar(&mut self) {
        self.pos += 1;
    }

    fn saltar_blancos(&mut self) {
        while let Some(c) = self.resto().chars().next() {
            if c.is_ascii_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn esperar(&mut self, c: char) -> Result<(), String> {
        self.saltar_blancos();
        if self.resto().starts_with(c) {
            self.pos += c.len_utf8();
            Ok(())
        } else {
            Err(format!(
                "JSON: se esperaba '{}' pero se encontró '{:.20}'",
                c,
                self.resto()
            ))
        }
    }

    fn parsear_valor(&mut self, profundidad: usize) -> Result<Objeto, String> {
        if profundidad > 128 {
            return Err(
                "Error de seguridad: Estructura JSON demasiado profunda"
                    .to_string(),
            );
        }
        self.saltar_blancos();
        let resto = self.resto();
        if resto.is_empty() {
            return Err("JSON: entrada vacía o inesperada".to_string());
        }
        match resto.as_bytes()[0] {
            b'{' => self.parsear_objeto(profundidad + 1),
            b'[' => self.parsear_arreglo(profundidad + 1),
            b'"' => self.parsear_cadena(),
            b't' => {
                if resto.starts_with("true") {
                    self.pos += 4;
                    Ok(Objeto::Booleano(true))
                } else {
                    Err("JSON: se esperaba 'true'".to_string())
                }
            }
            b'f' => {
                if resto.starts_with("false") {
                    self.pos += 5;
                    Ok(Objeto::Booleano(false))
                } else {
                    Err("JSON: se esperaba 'false'".to_string())
                }
            }
            b'n' => {
                if resto.starts_with("null") {
                    self.pos += 4;
                    Ok(Objeto::Nulo)
                } else {
                    Err("JSON: se esperaba 'null'".to_string())
                }
            }
            b'-' | b'0'..=b'9' => self.parsear_numero(),
            _ => Err(format!(
                "JSON: token inesperado '{:.20}'",
                resto
            )),
        }
    }

    fn parsear_cadena(&mut self) -> Result<Objeto, String> {
        self.esperar('"')?;
        let mut resultado = String::new();
        loop {
            let resto = self.resto();
            if resto.is_empty() {
                return Err("JSON: cadena sin cerrar".to_string());
            }
            let c = resto.as_bytes()[0];
            if c == b'"' {
                self.avanzar();
                return Ok(Objeto::Cadena(resultado));
            } else if c == b'\\' {
                self.avanzar();
                let sig = self.resto().as_bytes().first().copied();
                match sig {
                    Some(b'"') => { resultado.push('"'); self.avanzar(); }
                    Some(b'\\') => { resultado.push('\\'); self.avanzar(); }
                    Some(b'/') => { resultado.push('/'); self.avanzar(); }
                    Some(b'n') => { resultado.push('\n'); self.avanzar(); }
                    Some(b'r') => { resultado.push('\r'); self.avanzar(); }
                    Some(b't') => { resultado.push('\t'); self.avanzar(); }
                    Some(b'b') => { resultado.push('\u{0008}'); self.avanzar(); }
                    Some(b'f') => { resultado.push('\u{000C}'); self.avanzar(); }
                    Some(b'u') => {
                        self.avanzar();
                        let hex = &self.fuente[self.pos..self.pos + 4];
                        if hex.len() < 4 {
                            return Err(
                                "JSON: escape \\u incompleto".to_string()
                            );
                        }
                        let code = u32::from_str_radix(hex, 16)
                            .map_err(|_| {
                                "JSON: \\u inválido".to_string()
                            })?;
                        if let Some(codepoint) =
                            char::from_u32(code)
                        {
                            resultado.push(codepoint);
                        }
                        self.pos += 4;
                    }
                    _ => {
                        return Err(format!(
                            "JSON: escape inválido '\\{}'",
                            self.resto()
                                .chars()
                                .next()
                                .unwrap_or('?')
                        ));
                    }
                }
            } else {
                resultado.push(c as char);
                self.avanzar();
            }
        }
    }

    fn parsear_numero(&mut self) -> Result<Objeto, String> {
        let inicio = self.pos;
        let resto = self.resto();
        if resto.starts_with('-') {
            self.avanzar();
        }
        while let Some(&b) = self.resto().as_bytes().first() {
            if b.is_ascii_digit() {
                self.avanzar();
            } else {
                break;
            }
        }
        let mut es_flotante = false;
        if self.resto().starts_with('.') {
            es_flotante = true;
            self.avanzar();
            while let Some(&b) = self.resto().as_bytes().first() {
                if b.is_ascii_digit() {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        if self.resto().starts_with('e') || self.resto().starts_with('E') {
            es_flotante = true;
            self.avanzar();
            if self.resto().starts_with('+') || self.resto().starts_with('-') {
                self.avanzar();
            }
            while let Some(&b) = self.resto().as_bytes().first() {
                if b.is_ascii_digit() {
                    self.avanzar();
                } else {
                    break;
                }
            }
        }
        let literal = &self.fuente[inicio..self.pos];
        if literal.is_empty() || literal == "-" {
            return Err("JSON: número inválido".to_string());
        }
        if es_flotante {
            let val: f64 = literal.parse().map_err(|_| {
                format!("JSON: flotante inválido '{}'", literal)
            })?;
            Ok(Objeto::Flotante(val))
        } else {
            let val: i64 = literal.parse().map_err(|_| {
                format!("JSON: entero inválido '{}'", literal)
            })?;
            Ok(Objeto::Entero(val))
        }
    }

    fn parsear_objeto(&mut self, profundidad: usize) -> Result<Objeto, String> {
        self.esperar('{')?;
        self.saltar_blancos();
        let mut mapa = HashMap::new();
        if self.resto().starts_with('}') {
            self.avanzar();
            return Ok(Objeto::Diccionario(mapa));
        }
        loop {
            self.saltar_blancos();
            let clave = self.parsear_cadena()?;
            let clave_str = match &clave {
                Objeto::Cadena(s) => s.clone(),
                _ => unreachable!(),
            };
            self.esperar(':')?;
            let valor = self.parsear_valor(profundidad)?;
            mapa.insert(LlaveHash::Cadena(clave_str), valor);
            self.saltar_blancos();
            if self.resto().starts_with('}') {
                self.avanzar();
                return Ok(Objeto::Diccionario(mapa));
            }
            self.esperar(',')?;
        }
    }

    fn parsear_arreglo(&mut self, profundidad: usize) -> Result<Objeto, String> {
        self.esperar('[')?;
        self.saltar_blancos();
        let mut vec = Vec::new();
        if self.resto().starts_with(']') {
            self.avanzar();
            return Ok(Objeto::Arreglo(vec));
        }
        loop {
            let valor = self.parsear_valor(profundidad)?;
            vec.push(valor);
            self.saltar_blancos();
            if self.resto().starts_with(']') {
                self.avanzar();
                return Ok(Objeto::Arreglo(vec));
            }
            self.esperar(',')?;
        }
    }
}

/// Función nativa: json.parse(cadena) → Objeto
fn json_parse(args: Vec<Objeto>) -> Objeto {
    if args.len() != 1 {
        return Objeto::Error(
            "Se esperaba 1 argumento (cadena JSON)".to_string(),
        );
    }
    let texto = match &args[0] {
        Objeto::Cadena(s) => s.clone(),
        _ => {
            return Objeto::Error(
                "El argumento debe ser una cadena".to_string(),
            );
        }
    };

    // Truco para HTTP: buscar el primer '{' o '[' para descartar
    // cabeceras HTTP que pudieran preceder al JSON.
    let inicio = texto
        .find(|c| c == '{' || c == '[')
        .unwrap_or(0);
    let recortado = &texto[inicio..];

    let mut parser = JsonParser::nuevo(recortado);
    match parser.parsear_valor(0) {
        Ok(valor) => valor,
        Err(e) => Objeto::Error(e),
    }
}

/// Función nativa: json.stringify(objeto) → Objeto::Cadena
fn json_stringify(args: Vec<Objeto>) -> Objeto {
    if args.len() != 1 {
        return Objeto::Error(
            "Se esperaba 1 argumento".to_string(),
        );
    }
    let json = json_stringificar_valor(&args[0]);
    Objeto::Cadena(json)
}

/// Ensambla y retorna un Objeto::Diccionario con las funciones JSON.
pub fn crear_modulo() -> Objeto {
    let mut mapa_json: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa_json.insert(
        LlaveHash::Cadena("parse".to_string()),
        Objeto::Nativa(json_parse as fn(Vec<Objeto>) -> Objeto),
    );
    mapa_json.insert(
        LlaveHash::Cadena("stringify".to_string()),
        Objeto::Nativa(json_stringify as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa_json)
}
