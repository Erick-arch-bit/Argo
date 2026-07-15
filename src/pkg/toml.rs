// ---------------------------------------------------------------------------
// toml.rs — Parser TOML propio para Argo (subset: [package], [dependencies])
// ---------------------------------------------------------------------------
// Soporta: strings, integers, floats, booleans, arrays de strings,
//          tablas [section], key = value, comentarios #.

#[derive(Debug, Clone)]
pub enum TomlValor {
    Cadena(String),
    Entero(i64),
    Flotante(f64),
    Booleano(bool),
    Arreglo(Vec<TomlValor>),
    Tabla(HashMap<String, TomlValor>),
}

use std::collections::HashMap;

impl TomlValor {
    pub fn como_cadena(&self) -> Option<&str> {
        if let TomlValor::Cadena(s) = self { Some(s) } else { None }
    }
    pub fn como_entero(&self) -> Option<i64> {
        if let TomlValor::Entero(n) = self { Some(*n) } else { None }
    }
    pub fn como_booleano(&self) -> Option<bool> {
        if let TomlValor::Booleano(b) = self { Some(*b) } else { None }
    }
    pub fn como_arreglo(&self) -> Option<&[TomlValor]> {
        if let TomlValor::Arreglo(a) = self { Some(a) } else { None }
    }
    pub fn como_tabla(&self) -> Option<&HashMap<String, TomlValor>> {
        if let TomlValor::Tabla(t) = self { Some(t) } else { None }
    }
}

pub struct TomlParser<'a> {
    chars: &'a [u8],
    pos: usize,
}

impl<'a> TomlParser<'a> {
    pub fn nuevo(input: &'a str) -> Self {
        TomlParser { chars: input.as_bytes(), pos: 0 }
    }

    pub fn parsear(input: &str) -> Result<HashMap<String, TomlValor>, String> {
        let mut parser = TomlParser::nuevo(input);
        let mut resultado = HashMap::new();
        parser.saltar_espacios_y_comentarios();

        while parser.pos < parser.chars.len() {
            if parser.pos < parser.chars.len() && parser.chars[parser.pos] == b'[' {
                // Sección [table]
                let nombre = parser.parsear_seccion()?;
                let mut tabla = HashMap::new();
                parser.saltar_espacios_y_comentarios();

                while parser.pos < parser.chars.len() {
                    if parser.pos < parser.chars.len() && parser.chars[parser.pos] == b'[' {
                        break;
                    }
                    if parser.pos < parser.chars.len() && parser.chars[parser.pos] == b']' {
                        break;
                    }
                    let (key, val) = parser.parsear_key_value()?;
                    tabla.insert(key, val);
                    parser.saltar_espacios_y_comentarios();
                }
                resultado.insert(nombre, TomlValor::Tabla(tabla));
            } else if parser.pos < parser.chars.len() && parser.chars[parser.pos] == b'#' {
                parser.saltar_linea();
            } else {
                let (key, val) = parser.parsear_key_value()?;
                resultado.insert(key, val);
            }
            parser.saltar_espacios_y_comentarios();
        }

        Ok(resultado)
    }

    fn saltar_espacios_y_comentarios(&mut self) {
        while self.pos < self.chars.len() {
            match self.chars[self.pos] {
                b' ' | b'\t' | b'\r' | b'\n' => self.pos += 1,
                b'#' => self.saltar_linea(),
                _ => break,
            }
        }
    }

    fn saltar_linea(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos] != b'\n' {
            self.pos += 1;
        }
        if self.pos < self.chars.len() {
            self.pos += 1;
        }
    }

    fn parsear_seccion(&mut self) -> Result<String, String> {
        self.pos += 1; // skip [
        let inicio = self.pos;
        while self.pos < self.chars.len() && self.chars[self.pos] != b']' {
            self.pos += 1;
        }
        if self.pos >= self.chars.len() {
            return Err("TOML: sección sin cerrar ']'".to_string());
        }
        let nombre = String::from_utf8_lossy(&self.chars[inicio..self.pos]).to_string();
        self.pos += 1; // skip ]
        Ok(nombre.trim().to_string())
    }

    fn parsear_key_value(&mut self) -> Result<(String, TomlValor), String> {
        self.saltar_espacios_y_comentarios();
        let key = self.parsear_key()?;
        self.saltar_espacios_y_comentarios();

        if self.pos >= self.chars.len() || self.chars[self.pos] != b'=' {
            return Err(format!("TOML: se esperaba '=' después de '{}'", key));
        }
        self.pos += 1;
        self.saltar_espacios_y_comentarios();
        let val = self.parsear_valor()?;
        Ok((key, val))
    }

    fn parsear_key(&mut self) -> Result<String, String> {
        self.saltar_espacios_y_comentarios();
        if self.pos >= self.chars.len() {
            return Err("TOML: key vacía".to_string());
        }
        // Keys con comillas o sin comillas
        if self.chars[self.pos] == b'"' {
            self.parsear_cadena()
        } else {
            let inicio = self.pos;
            while self.pos < self.chars.len() {
                match self.chars[self.pos] {
                    b'=' | b' ' | b'\t' | b'\n' | b'\r' => break,
                    _ => self.pos += 1,
                }
            }
            Ok(String::from_utf8_lossy(&self.chars[inicio..self.pos]).trim().to_string())
        }
    }

    fn parsear_valor(&mut self) -> Result<TomlValor, String> {
        self.saltar_espacios_y_comentarios();
        if self.pos >= self.chars.len() {
            return Err("TOML: valor esperado".to_string());
        }
        match self.chars[self.pos] {
            b'"' => Ok(TomlValor::Cadena(self.parsear_cadena()?)),
            b'[' => self.parsear_array(),
            b't' | b'f' => self.parsear_booleano(),
            b'-' | b'0'..=b'9' => self.parsear_numero(),
            _ => Err(format!("TOML: carácter inesperado '{}'", self.chars[self.pos] as char)),
        }
    }

    fn parsear_cadena(&mut self) -> Result<String, String> {
        if self.pos >= self.chars.len() || self.chars[self.pos] != b'"' {
            return Err("TOML: se esperaba '\"'".to_string());
        }
        self.pos += 1;
        let mut resultado = String::new();
        while self.pos < self.chars.len() && self.chars[self.pos] != b'"' {
            if self.chars[self.pos] == b'\\' && self.pos + 1 < self.chars.len() {
                self.pos += 1;
                match self.chars[self.pos] {
                    b'n' => resultado.push('\n'),
                    b'r' => resultado.push('\r'),
                    b't' => resultado.push('\t'),
                    b'\\' => resultado.push('\\'),
                    b'"' => resultado.push('"'),
                    _ => resultado.push(self.chars[self.pos] as char),
                }
            } else {
                resultado.push(self.chars[self.pos] as char);
            }
            self.pos += 1;
        }
        if self.pos >= self.chars.len() {
            return Err("TOML: cadena sin cerrar".to_string());
        }
        self.pos += 1; // skip "
        Ok(resultado)
    }

    fn parsear_numero(&mut self) -> Result<TomlValor, String> {
        let inicio = self.pos;
        let mut es_float = false;
        if self.chars[self.pos] == b'-' {
            self.pos += 1;
        }
        while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.chars.len() && self.chars[self.pos] == b'.' {
            es_float = true;
            self.pos += 1;
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let texto = String::from_utf8_lossy(&self.chars[inicio..self.pos]);
        if es_float {
            texto.parse::<f64>().map(TomlValor::Flotante)
                .map_err(|e| format!("TOML: número float inválido: {}", e))
        } else {
            texto.parse::<i64>().map(TomlValor::Entero)
                .map_err(|e| format!("TOML: número entero inválido: {}", e))
        }
    }

    fn parsear_booleano(&mut self) -> Result<TomlValor, String> {
        if self.pos + 4 <= self.chars.len() && &self.chars[self.pos..self.pos + 4] == b"true" {
            self.pos += 4;
            Ok(TomlValor::Booleano(true))
        } else if self.pos + 5 <= self.chars.len() && &self.chars[self.pos..self.pos + 5] == b"false" {
            self.pos += 5;
            Ok(TomlValor::Booleano(false))
        } else {
            Err("TOML: booleano inválido".to_string())
        }
    }

    fn parsear_array(&mut self) -> Result<TomlValor, String> {
        self.pos += 1; // skip [
        self.saltar_espacios_y_comentarios();
        let mut elementos = Vec::new();

        if self.pos < self.chars.len() && self.chars[self.pos] == b']' {
            self.pos += 1;
            return Ok(TomlValor::Arreglo(elementos));
        }

        loop {
            self.saltar_espacios_y_comentarios();
            elementos.push(self.parsear_valor()?);
            self.saltar_espacios_y_comentarios();
            if self.pos < self.chars.len() && self.chars[self.pos] == b',' {
                self.pos += 1;
            } else {
                break;
            }
        }

        self.saltar_espacios_y_comentarios();
        if self.pos >= self.chars.len() || self.chars[self.pos] != b']' {
            return Err("TOML: array sin cerrar ']'".to_string());
        }
        self.pos += 1;
        Ok(TomlValor::Arreglo(elementos))
    }
}

/// Parsea un argo.toml y retorna un HashMap anidado.
pub fn parsear_argo_toml(contenido: &str) -> Result<HashMap<String, TomlValor>, String> {
    TomlParser::parsear(contenido)
}

/// Extrae [package] como HashMap<String, String>.
pub fn extraer_package(toml: &HashMap<String, TomlValor>) -> HashMap<String, String> {
    let mut pkg = HashMap::new();
    if let Some(TomlValor::Tabla(tabla)) = toml.get("package") {
        for (k, v) in tabla {
            if let Some(s) = v.como_cadena() {
                pkg.insert(k.clone(), s.to_string());
            }
        }
    }
    pkg
}

/// Extrae [dependencies] como HashMap<String, String> (name -> version_spec).
pub fn extraer_dependencies(toml: &HashMap<String, TomlValor>) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if let Some(TomlValor::Tabla(tabla)) = toml.get("dependencies") {
        for (k, v) in tabla {
            match v {
                TomlValor::Cadena(s) => deps.push((k.clone(), s.clone())),
                TomlValor::Tabla(inner) => {
                    if let Some(TomlValor::Cadena(ver)) = inner.get("version") {
                        deps.push((k.clone(), ver.clone()));
                    }
                }
                _ => {}
            }
        }
    }
    deps
}

/// Extrae [dev-dependencies] como Vec<(String, String)>.
pub fn extraer_dev_dependencies(toml: &HashMap<String, TomlValor>) -> Vec<(String, String)> {
    let mut deps = Vec::new();
    if let Some(TomlValor::Tabla(tabla)) = toml.get("dev-dependencies") {
        for (k, v) in tabla {
            match v {
                TomlValor::Cadena(s) => deps.push((k.clone(), s.clone())),
                TomlValor::Tabla(inner) => {
                    if let Some(TomlValor::Cadena(ver)) = inner.get("version") {
                        deps.push((k.clone(), ver.clone()));
                    }
                }
                _ => {}
            }
        }
    }
    deps
}
