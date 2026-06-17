//
// Módulo estándar net — Conexiones de red.
// Expone una función para realizar peticiones HTTP GET
// en un Objeto::Diccionario bajo el nombre "net".
//

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::evaluator::{LlaveHash, Objeto};

/// Ensambla y retorna un Objeto::Diccionario con las funciones de red.
pub fn crear_modulo() -> Objeto {
    // Realiza una petición HTTP GET a un host y puerto.
    // Recibe 1 argumento: Objeto::Cadena("host:puerto").
    // Retorna el cuerpo de la respuesta como cadena.
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
                    "El argumento debe ser una cadena (host:puerto)"
                        .to_string(),
                );
            }
        };

        // Conectar al servidor TCP
        let mut stream = match TcpStream::connect(&host_puerto) {
            Ok(s) => s,
            Err(e) => {
                return Objeto::Error(format!(
                    "Error al conectar a {}: {}",
                    host_puerto, e
                ));
            }
        };

        // Extraer el dominio (primera parte antes de ':')
        let dominio = match host_puerto.split(':').next() {
            Some(d) => d.to_string(),
            None => host_puerto.clone(),
        };

        // Construir y enviar la petición HTTP GET
        let peticion = format!(
            "GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            dominio
        );
        if let Err(e) = stream.write_all(peticion.as_bytes()) {
            return Objeto::Error(format!(
                "Error al enviar la petición a {}: {}",
                host_puerto, e
            ));
        }

        // Leer la respuesta completa
        let mut respuesta = String::new();
        if let Err(e) = stream.read_to_string(&mut respuesta) {
            return Objeto::Error(format!(
                "Error al leer la respuesta de {}: {}",
                host_puerto, e
            ));
        }

        Objeto::Cadena(respuesta)
    }

    let mut mapa_net: HashMap<LlaveHash, Objeto> = HashMap::new();

    mapa_net.insert(
        LlaveHash::Cadena("get".to_string()),
        Objeto::Nativa(net_get as fn(Vec<Objeto>) -> Objeto),
    );

    Objeto::Diccionario(mapa_net)
}
