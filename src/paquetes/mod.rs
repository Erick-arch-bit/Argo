// ---------------------------------------------------------------------------
// paquetes/ — Sistema de paquetes inmutables de Argo-Lang v2.2.0
// ---------------------------------------------------------------------------
// Cero dependencias externas. Todo implementado en Rust std + herramientas
// nativas del sistema operativo (openssl, sha256sum, curl, tar, git).
//
// Este módulo centraliza:
//   - crypto.rs       → hash, firma/verificación Ed25519, Base64.
//   - RegistroPaquetes → estado local de paquetes (~/.argo/cache/registry.json)
//     y descarga del registro remoto desde la web (curl + parser JSON propio).

#![allow(dead_code, clippy::collapsible_if, clippy::new_without_default,
    clippy::len_zero, clippy::manual_contains)]

pub mod crypto;

use std::collections::HashMap;
use std::path::Path;

/// Devuelve el directorio home del usuario (variable HOME del sistema).
pub fn home_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string())
}

/// Ruta de la carpeta de llaves del desarrollador: ~/.argo/keys
pub fn ruta_carpeta_llaves() -> String {
    format!("{}/.argo/keys", home_dir())
}

/// Ruta del registro local de paquetes: ~/.argo/cache/registry.json
pub fn ruta_archivo_registro() -> String {
    format!("{}/.argo/cache/registry.json", home_dir())
}

// ===========================================================================
// MetadatosPaquete
// ===========================================================================

/// Metadatos inmutables de un paquete publicado en el registro.
#[derive(Debug, Clone)]
pub struct MetadatosPaquete {
    pub nombre: String,
    pub version: String,
    pub autor: String,
    pub hash: String,
    pub firma: String,
    pub llave_publica: String,
    pub url_descarga: String,
}

impl MetadatosPaquete {
    /// Serializa los metadatos como un objeto JSON:
    /// `{"version":"...","autor":"...","hash":"...","firma":"...","llave_publica":"...","url_descarga":"..."}`
    pub fn a_json(&self) -> String {
        format!(
            "{{\"version\":{},\"autor\":{},\"hash\":{},\"firma\":{},\"llave_publica\":{},\"url_descarga\":{}}}",
            escapar_json(&self.version),
            escapar_json(&self.autor),
            escapar_json(&self.hash),
            escapar_json(&self.firma),
            escapar_json(&self.llave_publica),
            escapar_json(&self.url_descarga),
        )
    }
}

// ===========================================================================
// RegistroPaquetes — estado local instalado
// ===========================================================================

/// Registro local de paquetes instalados/registrados.
#[derive(Debug, Clone)]
pub struct RegistroPaquetes {
    pub paquetes: HashMap<String, MetadatosPaquete>,
}

impl RegistroPaquetes {
    pub fn nuevo() -> Self {
        RegistroPaquetes { paquetes: HashMap::new() }
    }

    /// Carga el registro local desde ~/.argo/cache/registry.json.
    /// Si el archivo no existe o es inválido, devuelve un registro vacío.
    pub fn cargar() -> Self {
        let mut registro = RegistroPaquetes::nuevo();
        let ruta = ruta_archivo_registro();

        if let Ok(contenido) = std::fs::read_to_string(&ruta) {
            if let Ok(json) = parsear_json(&contenido) {
                if let Some(ValorJson::Objeto(objeto)) = json.get("paquetes") {
                    for (nombre, valor) in objeto {
                        if let ValorJson::Objeto(campos) = valor {
                            if let Ok(meta) = metadatos_desde_objeto(nombre, campos) {
                                registro.paquetes.insert(nombre.clone(), meta);
                            }
                        }
                    }
                }
            }
        }

        registro
    }

    /// Guarda el registro local en ~/.argo/cache/registry.json.
    pub fn guardar(&self) -> Result<(), String> {
        let ruta = ruta_archivo_registro();
        let dir = Path::new(&ruta)
            .parent()
            .ok_or_else(|| "Error: No se pudo determinar la carpeta del registro.".to_string())?;
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Error: No se pudo crear '{}': {}", dir.display(), e))?;
        std::fs::write(&ruta, self.a_json())
            .map_err(|e| format!("Error: No se pudo escribir '{}': {}", ruta, e))
    }

    pub fn agregar(&mut self, meta: MetadatosPaquete) {
        self.paquetes.insert(meta.nombre.clone(), meta);
    }

    pub fn obtener(&self, nombre: &str) -> Option<&MetadatosPaquete> {
        self.paquetes.get(nombre)
    }

    pub fn eliminar(&mut self, nombre: &str) -> Option<MetadatosPaquete> {
        self.paquetes.remove(nombre)
    }

    pub fn contiene(&self, nombre: &str) -> bool {
        self.paquetes.contains_key(nombre)
    }

    /// Serializa el registro completo en el mismo formato del registro remoto:
    /// `{"paquetes":{ "nombre": { ... } }}`
    pub fn a_json(&self) -> String {
        let mut nombres: Vec<&String> = self.paquetes.keys().collect();
        nombres.sort();
        let mut partes = Vec::with_capacity(nombres.len());
        for nombre in nombres {
            let meta = &self.paquetes[nombre];
            partes.push(format!("{}:{}", escapar_json(nombre), meta.a_json()));
        }
        format!("{{\"paquetes\":{{{}}}}}", partes.join(","))
    }
}

// ===========================================================================
// Registro remoto — descarga desde la web con curl
// ===========================================================================

/// Descarga y parsea el registro remoto de paquetes desde una URL JSON.
///
/// Formato esperado:
/// `{"paquetes": { "nombre_paquete": { "version":"1.0.0", "hash":"...",
///   "firma":"...", "llave_publica":"...", "url_descarga":"..." } } }`
pub fn obtener_metadatos_desde_github(url_json: &str) -> Result<HashMap<String, MetadatosPaquete>, String> {
    let salida = std::process::Command::new("curl")
        .arg("-s")
        .arg(url_json)
        .output()
        .map_err(|_| "Error: ¿Está instalado 'curl' en el sistema?".to_string())?;

    let texto = String::from_utf8_lossy(&salida.stdout).to_string();
    let texto = texto.trim();
    if texto.is_empty() {
        return Err(format!(
            "Error: No se pudo descargar el registro desde '{}'.",
            url_json
        ));
    }

    parsear_registro(texto)
}

/// Parsea el texto JSON de un registro remoto a un mapa de paquetes.
fn parsear_registro(texto: &str) -> Result<HashMap<String, MetadatosPaquete>, String> {
    let json = parsear_json(texto)?;

    let paquetes_objeto = match json.get("paquetes") {
        Some(ValorJson::Objeto(o)) => o,
        _ => return Err("Error: Registro JSON inválido: falta el objeto 'paquetes'.".to_string()),
    };

    let mut resultado = HashMap::new();
    for (nombre, valor) in paquetes_objeto {
        let campos = match valor {
            ValorJson::Objeto(campos) => campos,
            _ => {
                return Err(format!(
                    "Error: El paquete '{}' no es un objeto de metadatos.",
                    nombre
                ));
            }
        };
        let meta = metadatos_desde_objeto(nombre, campos)?;
        resultado.insert(nombre.clone(), meta);
    }

    Ok(resultado)
}

// ===========================================================================
// Parser JSON propio (subset suficiente para el registro)
// ===========================================================================

#[derive(Debug, Clone)]
enum ValorJson {
    Cadena(String),
    Objeto(HashMap<String, ValorJson>),
    Texto(String),
}

struct ParserJson<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> ParserJson<'a> {
    fn nuevo(texto: &'a str) -> Self {
        ParserJson { bytes: texto.as_bytes(), pos: 0 }
    }

    fn saltar_blancos(&mut self) {
        while self.pos < self.bytes.len()
            && matches!(self.bytes[self.pos], b' ' | b'\t' | b'\n' | b'\r')
        {
            self.pos += 1;
        }
    }

    fn parsear_objeto(&mut self) -> Result<HashMap<String, ValorJson>, String> {
        let mut mapa = HashMap::new();
        if self.bytes.get(self.pos) != Some(&b'{') {
            return Err("Error: JSON inválido: se esperaba '{'".to_string());
        }
        self.pos += 1;
        self.saltar_blancos();

        if self.bytes.get(self.pos) == Some(&b'}') {
            self.pos += 1;
            return Ok(mapa);
        }

        loop {
            self.saltar_blancos();
            if self.bytes.get(self.pos) != Some(&b'"') {
                return Err("Error: JSON inválido: se esperaba una clave entre comillas.".to_string());
            }
            let clave = self.parsear_cadena()?;
            self.saltar_blancos();
            if self.bytes.get(self.pos) != Some(&b':') {
                return Err(format!("Error: JSON inválido: se esperaba ':' tras '{}'.", clave));
            }
            self.pos += 1;
            self.saltar_blancos();
            let valor = self.parsear_valor()?;
            mapa.insert(clave, valor);
            self.saltar_blancos();

            match self.bytes.get(self.pos) {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err("Error: JSON inválido: se esperaba ',' o '}'.".to_string()),
            }
        }

        Ok(mapa)
    }

    fn parsear_valor(&mut self) -> Result<ValorJson, String> {
        self.saltar_blancos();
        match self.bytes.get(self.pos) {
            Some(b'{') => Ok(ValorJson::Objeto(self.parsear_objeto()?)),
            Some(b'"') => Ok(ValorJson::Cadena(self.parsear_cadena()?)),
            Some(_) => {
                let inicio = self.pos;
                while self.pos < self.bytes.len()
                    && !matches!(self.bytes[self.pos], b',' | b'}' | b']')
                {
                    self.pos += 1;
                }
                if inicio == self.pos {
                    return Err("Error: JSON inválido: valor vacío.".to_string());
                }
                Ok(ValorJson::Texto(
                    String::from_utf8_lossy(&self.bytes[inicio..self.pos])
                        .trim()
                        .to_string(),
                ))
            }
            None => Err("Error: JSON inválido: se esperaba un valor.".to_string()),
        }
    }

    fn parsear_cadena(&mut self) -> Result<String, String> {
        if self.bytes.get(self.pos) != Some(&b'"') {
            return Err("Error: JSON inválido: se esperaba '\"'.".to_string());
        }
        self.pos += 1;

        let mut resultado = String::new();
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'"' => {
                    self.pos += 1;
                    return Ok(resultado);
                }
                b'\\' => {
                    self.pos += 1;
                    if self.pos >= self.bytes.len() {
                        break;
                    }
                    match self.bytes[self.pos] {
                        b'n' => resultado.push('\n'),
                        b't' => resultado.push('\t'),
                        b'r' => resultado.push('\r'),
                        b'b' => resultado.push('\u{0008}'),
                        b'f' => resultado.push('\u{000C}'),
                        b'/' => resultado.push('/'),
                        b'"' => resultado.push('"'),
                        b'\\' => resultado.push('\\'),
                        b'u' => {
                            if self.pos + 4 < self.bytes.len() {
                                let hex = String::from_utf8_lossy(&self.bytes[self.pos + 1..self.pos + 5]);
                                if let Ok(codepoint) = u16::from_str_radix(&hex, 16) {
                                    if let Some(c) = char::from_u32(codepoint as u32) {
                                        resultado.push(c);
                                        self.pos += 4;
                                    }
                                }
                            }
                        }
                        otro => resultado.push(otro as char),
                    }
                    self.pos += 1;
                }
                byte => resultado.push(byte as char),
            }
            self.pos += 1;
        }

        Err("Error: JSON inválido: cadena sin cerrar.".to_string())
    }
}

/// Parsea un documento JSON completo y devuelve su objeto raíz.
fn parsear_json(texto: &str) -> Result<HashMap<String, ValorJson>, String> {
    let mut parser = ParserJson::nuevo(texto);
    parser.saltar_blancos();
    if parser.bytes.is_empty() {
        return Err("Error: Registro JSON vacío.".to_string());
    }
    parser.parsear_objeto()
}

/// Convierte un objeto JSON de campos en `MetadatosPaquete`.
fn metadatos_desde_objeto(
    nombre: &str,
    campos: &HashMap<String, ValorJson>,
) -> Result<MetadatosPaquete, String> {
    let obtener = |clave: &str| -> Result<String, String> {
        match campos.get(clave) {
            Some(ValorJson::Cadena(s)) => Ok(s.clone()),
            Some(ValorJson::Texto(t)) => Ok(t.clone()),
            _ => Err(format!(
                "Error: El paquete '{}' no tiene el campo '{}'.",
                nombre, clave
            )),
        }
    };

    Ok(MetadatosPaquete {
        nombre: nombre.to_string(),
        version: obtener("version")?,
        autor: obtener("autor").unwrap_or_default(),
        hash: obtener("hash")?,
        firma: obtener("firma")?,
        llave_publica: obtener("llave_publica")?,
        url_descarga: obtener("url_descarga")?,
    })
}

/// Escapa una cadena para incluirla dentro de un literal JSON.
pub fn escapar_json(s: &str) -> String {
    let mut resultado = String::with_capacity(s.len() + 2);
    resultado.push('"');
    for c in s.chars() {
        match c {
            '"' => resultado.push_str("\\\""),
            '\\' => resultado.push_str("\\\\"),
            '\n' => resultado.push_str("\\n"),
            '\r' => resultado.push_str("\\r"),
            '\t' => resultado.push_str("\\t"),
            c if (c as u32) < 0x20 => resultado.push_str(&format!("\\u{:04x}", c as u32)),
            c => resultado.push(c),
        }
    }
    resultado.push('"');
    resultado
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGISTRO_EJEMPLO: &str = r#"{
        "paquetes": {
            "mi_libreria_tui": {
                "version": "1.0.0",
                "autor": "erick901",
                "hash": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6",
                "firma": "ZXhhYW1wbGVfYmFzZTY0",
                "llave_publica": "MCowBQYDK2VwAyEAabc123",
                "url_descarga": "https://github.com/erick-arch-bit/mi_libreria/releases/download/v1.0.0/mi_libreria.tar.gz"
            }
        }
    }"#;

    #[test]
    fn parsea_registro_remoto() {
        let paquetes = parsear_registro(REGISTRO_EJEMPLO).unwrap();
        let meta = paquetes.get("mi_libreria_tui").unwrap();
        assert_eq!(meta.version, "1.0.0");
        assert_eq!(meta.autor, "erick901");
        assert_eq!(meta.hash.len(), 64);
        assert_eq!(meta.firma, "ZXhhYW1wbGVfYmFzZTY0");
        assert_eq!(meta.llave_publica, "MCowBQYDK2VwAyEAabc123");
        assert_eq!(
            meta.url_descarga,
            "https://github.com/erick-arch-bit/mi_libreria/releases/download/v1.0.0/mi_libreria.tar.gz"
        );
    }

    #[test]
    fn roundtrip_registro_serializacion() {
        let mut registro = RegistroPaquetes::nuevo();
        registro.agregar(MetadatosPaquete {
            nombre: "lib".to_string(),
            version: "2.2.0".to_string(),
            autor: "dev".to_string(),
            hash: "ab".repeat(32),
            firma: "c2ln".to_string(),
            llave_publica: "cHVibGlj".to_string(),
            url_descarga: "https://example.com/lib.tar.gz".to_string(),
        });

        let json = registro.a_json();
        let reparsed = parsear_json(&json).unwrap();
        let objeto = match reparsed.get("paquetes") {
            Some(ValorJson::Objeto(o)) => o,
            _ => panic!("falta paquetes"),
        };
        assert!(objeto.contains_key("lib"));

        match &objeto["lib"] {
            ValorJson::Objeto(campos) => {
                let meta = metadatos_desde_objeto("lib", campos).unwrap();
                assert_eq!(meta.version, "2.2.0");
                assert_eq!(meta.url_descarga, "https://example.com/lib.tar.gz");
            }
            _ => panic!("el paquete 'lib' no es un objeto"),
        }
    }
}
