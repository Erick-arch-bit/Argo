//
// Módulo estándar net — Conexiones de red vía TCP crudo.
// Soporta los verbos HTTP GET, POST, PUT, PATCH, DELETE
// mediante una función helper interna que construye y
// envía la petición, luego lee la respuesta completa.
//

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::evaluator::{LlaveHash, Objeto};

// ---------------------------------------------------------------------------
// Helper interno — Envía una petición HTTP y retorna la respuesta cruda.
// ---------------------------------------------------------------------------
// Conecta al host:puerto, construye el mensaje HTTP con el método y
// cuerpo opcional, escribe al stream y lee la respuesta completa.
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

// ---------------------------------------------------------------------------
// Funciones nativas expuestas al interprete Argo
// ---------------------------------------------------------------------------

fn net_get(args: Vec<Objeto>) -> Objeto {
    if args.len() != 1 {
        return Objeto::Error(
            "Se esperaba 1 argumento (host:puerto)".to_string(),
        );
    }
    let host_puerto = match &args[0] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El argumento debe ser una cadena (host:puerto)".to_string(),
            );
        }
    };
    match enviar_peticion_tcp("GET", &host_puerto, None) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e),
    }
}

fn net_delete(args: Vec<Objeto>) -> Objeto {
    if args.len() != 1 {
        return Objeto::Error(
            "Se esperaba 1 argumento (host:puerto)".to_string(),
        );
    }
    let host_puerto = match &args[0] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El argumento debe ser una cadena (host:puerto)".to_string(),
            );
        }
    };
    match enviar_peticion_tcp("DELETE", &host_puerto, None) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e),
    }
}

fn net_post(args: Vec<Objeto>) -> Objeto {
    if args.len() != 2 {
        return Objeto::Error(
            "Se esperaban 2 argumentos (host:puerto, cuerpo)".to_string(),
        );
    }
    let host_puerto = match &args[0] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El primer argumento debe ser una cadena (host:puerto)"
                    .to_string(),
            );
        }
    };
    let cuerpo = match &args[1] {
        Objeto::Cadena(c) => c.clone(),
        _ => {
            return Objeto::Error(
                "El segundo argumento debe ser una cadena (cuerpo)".to_string(),
            );
        }
    };
    match enviar_peticion_tcp("POST", &host_puerto, Some(&cuerpo)) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e),
    }
}

fn net_put(args: Vec<Objeto>) -> Objeto {
    if args.len() != 2 {
        return Objeto::Error(
            "Se esperaban 2 argumentos (host:puerto, cuerpo)".to_string(),
        );
    }
    let host_puerto = match &args[0] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El primer argumento debe ser una cadena (host:puerto)"
                    .to_string(),
            );
        }
    };
    let cuerpo = match &args[1] {
        Objeto::Cadena(c) => c.clone(),
        _ => {
            return Objeto::Error(
                "El segundo argumento debe ser una cadena (cuerpo)".to_string(),
            );
        }
    };
    match enviar_peticion_tcp("PUT", &host_puerto, Some(&cuerpo)) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e),
    }
}

fn net_patch(args: Vec<Objeto>) -> Objeto {
    if args.len() != 2 {
        return Objeto::Error(
            "Se esperaban 2 argumentos (host:puerto, cuerpo)".to_string(),
        );
    }
    let host_puerto = match &args[0] {
        Objeto::Cadena(hp) => hp.clone(),
        _ => {
            return Objeto::Error(
                "El primer argumento debe ser una cadena (host:puerto)"
                    .to_string(),
            );
        }
    };
    let cuerpo = match &args[1] {
        Objeto::Cadena(c) => c.clone(),
        _ => {
            return Objeto::Error(
                "El segundo argumento debe ser una cadena (cuerpo)".to_string(),
            );
        }
    };
    match enviar_peticion_tcp("PATCH", &host_puerto, Some(&cuerpo)) {
        Ok(r) => Objeto::Cadena(r),
        Err(e) => Objeto::Error(e),
    }
}

/// Ensambla y retorna un Objeto::Diccionario con las funciones de red.
pub fn crear_modulo() -> Objeto {
    let mut mapa: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa.insert(
        LlaveHash::Cadena("get".to_string()),
        Objeto::Nativa(net_get as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("post".to_string()),
        Objeto::Nativa(net_post as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("put".to_string()),
        Objeto::Nativa(net_put as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("patch".to_string()),
        Objeto::Nativa(net_patch as fn(Vec<Objeto>) -> Objeto),
    );
    mapa.insert(
        LlaveHash::Cadena("delete".to_string()),
        Objeto::Nativa(net_delete as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa)
}
