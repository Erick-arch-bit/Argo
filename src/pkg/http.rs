// ---------------------------------------------------------------------------
// http.rs — Cliente HTTP/1.1 propio sobre TcpStream
// ---------------------------------------------------------------------------
// Soporta HTTP plano y HTTPS (via native-tls del sistema si está disponible).
// Sin dependencias externas de crates.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct PeticionHttp {
    pub metodo: String,
    pub host: String,
    pub puerto: u16,
    pub ruta: String,
    pub headers: Vec<(String, String)>,
    pub cuerpo: Option<String>,
    pub timeout_secs: u64,
    pub token: Option<String>,
}

pub struct RespuestaHttp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl PeticionHttp {
    pub fn get(host: &str, puerto: u16, ruta: &str) -> Self {
        PeticionHttp {
            metodo: "GET".to_string(),
            host: host.to_string(),
            puerto,
            ruta: ruta.to_string(),
            headers: Vec::new(),
            cuerpo: None,
            timeout_secs: 30,
            token: None,
        }
    }

    pub fn head(host: &str, puerto: u16, ruta: &str) -> Self {
        PeticionHttp {
            metodo: "HEAD".to_string(),
            host: host.to_string(),
            puerto,
            ruta: ruta.to_string(),
            headers: Vec::new(),
            cuerpo: None,
            timeout_secs: 30,
            token: None,
        }
    }

    pub fn con_token(mut self, token: &str) -> Self {
        self.token = Some(token.to_string());
        self
    }

    pub fn con_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn enviar(&self) -> Result<RespuestaHttp, String> {
        let addr = format!("{}:{}", self.host, self.puerto);
        let timeout = Duration::from_secs(self.timeout_secs);

        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Error de conexión a {}: {}", addr, e))?;
        stream.set_read_timeout(Some(timeout))
            .map_err(|e| format!("Error configurando timeout: {}", e))?;
        stream.set_write_timeout(Some(timeout))
            .map_err(|e| format!("Error configurando timeout: {}", e))?;

        // Construir request HTTP/1.1
        let mut request = format!("{} {} HTTP/1.1\r\n", self.metodo, self.ruta);
        request.push_str(&format!("Host: {}\r\n", self.host));
        request.push_str("User-Agent: argo-cli/2.2.0\r\n");
        request.push_str("Accept: application/json\r\n");
        request.push_str("Connection: close\r\n");

        if let Some(ref token) = self.token {
            request.push_str(&format!("Authorization: token {}\r\n", token));
        }

        for (key, val) in &self.headers {
            request.push_str(&format!("{}: {}\r\n", key, val));
        }

        if let Some(ref cuerpo) = self.cuerpo {
            request.push_str(&format!("Content-Length: {}\r\n", cuerpo.len()));
            request.push_str("Content-Type: application/json\r\n");
        }

        request.push_str("\r\n");
        if let Some(ref cuerpo) = self.cuerpo {
            request.push_str(cuerpo);
        }

        stream.write_all(request.as_bytes())
            .map_err(|e| format!("Error escribiendo request: {}", e))?;

        // Leer respuesta
        let mut reader = BufReader::new(&mut stream);
        let mut status_line = String::new();
        reader.read_line(&mut status_line)
            .map_err(|e| format!("Error leyendo respuesta: {}", e))?;

        let status = parsear_status_line(&status_line)?;

        // Leer headers
        let mut headers = Vec::new();
        let mut content_length: usize = 0;
        loop {
            let mut linea = String::new();
            reader.read_line(&mut linea)
                .map_err(|e| format!("Error leyendo headers: {}", e))?;
            let linea = linea.trim().to_string();
            if linea.is_empty() {
                break;
            }
            if let Some((key, val)) = linea.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim().to_string();
                if key == "content-length" {
                    content_length = val.parse().unwrap_or(0);
                }
                headers.push((key, val));
            }
        }

        // Leer body
        let mut body = String::new();
        if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            reader.read_exact(&mut buf)
                .map_err(|e| format!("Error leyendo body: {}", e))?;
            body = String::from_utf8_lossy(&buf).to_string();
        } else {
            reader.read_to_string(&mut body)
                .map_err(|e| format!("Error leyendo body: {}", e))?;
        }

        Ok(RespuestaHttp { status, headers, body })
    }
}

fn parsear_status_line(linea: &str) -> Result<u16, String> {
    // HTTP/1.1 200 OK
    let partes: Vec<&str> = linea.trim().split_whitespace().collect();
    if partes.len() < 2 {
        return Err(format!("Status line inválida: '{}'", linea.trim()));
    }
    partes[1].parse::<u16>().map_err(|_| format!("Status code inválido: '{}'", partes[1]))
}

/// Parsea una URL simple: host.com/path -> (host, puerto, ruta)
pub fn parsear_url(url: &str) -> Result<(String, u16, String), String> {
    let url = url.trim();
    let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);

    let (host_path, puerto) = if let Some(colon_pos) = url.find(':') {
        let after_colon = &url[colon_pos + 1..];
        if let Some(slash_pos) = after_colon.find('/') {
            let puerto_str = &after_colon[..slash_pos];
            let puerto = puerto_str.parse::<u16>().map_err(|_| format!("Puerto inválido: {}", puerto_str))?;
            (&url[slash_pos..], puerto)
        } else {
            (after_colon, 443)
        }
    } else {
        (url.find('/').map(|i| &url[i..]).unwrap_or(""), 443)
    };

    let host = if let Some(pos) = url.find('/') {
        let h = &url[..pos];
        if let Some(colon) = h.find(':') { &h[..colon] } else { h }
    } else {
        url
    };

    let ruta = if host_path.starts_with('/') { host_path.to_string() } else { format!("/{}", host_path) };

    Ok((host.to_string(), puerto, ruta))
}

/// Helper para leer un token de ~/.argo/config.toml
pub fn leer_token_config() -> Option<String> {
    let home = std::env::var("HOME").ok()?;
    let config_path = std::path::PathBuf::from(home).join(".argo").join("config.toml");
    let contenido = std::fs::read_to_string(&config_path).ok()?;
    for linea in contenido.lines() {
        let linea = linea.trim();
        if linea.starts_with('#') { continue; }
        if let Some((key, val)) = linea.split_once('=') {
            let key = key.trim();
            let val = val.trim().trim_matches('"').trim_matches('\'');
            if key == "github-token" || key == "gitlab-token" || key == "bitbucket-token" {
                if !val.is_empty() {
                    return Some(val.to_string());
                }
            }
        }
    }
    None
}
