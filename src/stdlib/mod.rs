//
// Biblioteca Estándar de Argo — Punto de orquestación.
// Cada submódulo expone crear_modulo() -> Objeto::Diccionario.
// inyectar_stdlib() los asigna en el entorno global bajo su
// nombre de espacio (math, fs, net, json).
//

pub mod arr;
pub mod buffer;
pub mod fs;
pub mod gpu;
pub mod json;
pub mod math;
pub mod net;
pub mod os;
pub mod str;
pub mod thread;
pub mod time;
pub mod ui;

use crate::evaluator::Entorno;

/// Inyecta todos los módulos de la biblioteca estándar en el
/// entorno global proporcionado.
pub fn inyectar_stdlib(entorno: &mut Entorno) {
    entorno.asignar("math".to_string(), math::crear_modulo());
    entorno.asignar("fs".to_string(), fs::crear_modulo());
    entorno.asignar("net".to_string(), net::crear_modulo());
    entorno.asignar("json".to_string(), json::crear_modulo());
    entorno.asignar("time".to_string(), time::crear_modulo());
    entorno.asignar("os".to_string(), os::crear_modulo());
    entorno.asignar("str".to_string(), str::crear_modulo());
    entorno.asignar("arr".to_string(), arr::crear_modulo());
    entorno.asignar("buffer".to_string(), buffer::crear_modulo());
    entorno.asignar("thread".to_string(), thread::crear_modulo());
    entorno.asignar("gpu".to_string(), gpu::crear_modulo());
    entorno.asignar("ui".to_string(), ui::crear_modulo());
}
