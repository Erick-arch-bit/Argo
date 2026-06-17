//
// Biblioteca Estándar de Argo — Punto de orquestación.
// Cada submódulo expone crear_modulo() -> Objeto::Diccionario.
// inyectar_stdlib() los asigna en el entorno global bajo su
// nombre de espacio (math, fs, net, json).
//

pub mod fs;
pub mod json;
pub mod math;
pub mod net;

use crate::evaluator::Entorno;

/// Inyecta todos los módulos de la biblioteca estándar en el
/// entorno global proporcionado.
pub fn inyectar_stdlib(entorno: &mut Entorno) {
    entorno.asignar("math".to_string(), math::crear_modulo());
    entorno.asignar("fs".to_string(), fs::crear_modulo());
    entorno.asignar("net".to_string(), net::crear_modulo());
    entorno.asignar("json".to_string(), json::crear_modulo());
}
