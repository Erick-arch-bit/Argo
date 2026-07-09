use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::evaluator::{LlaveHash, Objeto};

fn enviar_peticion_tcp(
    metodo: &str,
    host_puerto: &str,
    cuerpo: Option<&str>,
) -> Result<String, String> {
    let mut stream =
        TcpStream::connect(host_puerto).map_err(|e| {
            format!("Error al conectar a {}: {}", host_puerto, e)
        })?;

    let dominio = host_puerto.split(':').next().unwrap_or(host_puerto);

    let peticion = match cuerpo {
        Some(json) => format!(
            "{} / HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            metodo, dominio, json.len(), json
        ),
        None => format!(
            "{} / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            metodo, dominio
        ),
    };

    stream.write_all(peticion.as_bytes()).map_err(|e| {
        format!(
            "Error al enviar la petición {} a {}: {}",
            metodo, host_puerto, e
        )
    })?;

    let mut respuesta = String::new();
    stream.read_to_string(&mut respuesta).map_err(|e| {
        format!(
            "Error al leer la respuesta de {}: {}",
            host_puerto, e
        )
    })?;

    Ok(respuesta)
}

fn net_solicitud(args: Vec<Objeto>) -> Objeto {
    if args.len() < 2 || args.len() > 3 {
        return Objeto::Error(
            "Se esperaban 2 o 3 argumentos (metodo, host:puerto, cuerpo?)".to_string(),
        Vec::new());
    }
    let metodo = match &args[0] {
        Objeto::Cadena(m) => m.to_uppercase(),
        _ => {
            return Objeto::Error(
                "El primer argumento debe ser una cadena (método HTTP)".to_string(),
            Vec::new());
        }
    };
    let host_puerto = match &args[1] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El segundo argumento debe ser una cadena (host:puerto)".to_string(),
            Vec::new());
        }
    };
    if metodo != "GET" && metodo != "POST" && metodo != "PUT" && metodo != "PATCH" && metodo != "DELETE" {
        return Objeto::Error(format!(
            "Método HTTP no soportado: '{}'. Use GET, POST, PUT, PATCH o DELETE", metodo
        ), Vec::new());
    }
    let cuerpo = if args.len() == 3 {
        match &args[2] {
            Objeto::Cadena(c) => Some(c.clone()),
            _ => {
                return Objeto::Error(
                    "El tercer argumento debe ser una cadena (cuerpo)".to_string(),
                Vec::new());
            }
        }
    } else {
        None
    };
    match enviar_peticion_tcp(&metodo, &host_puerto, cuerpo.as_deref()) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e, Vec::new()),
    }
}

pub fn crear_modulo() -> Objeto {
    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();
    mapa.insert(
        LlaveHash::Cadena("solicitud".to_string()),
        Objeto::Nativa(net_solicitud as fn(Vec<Objeto>) -> Objeto),
    );
    Objeto::Diccionario(mapa)
}
