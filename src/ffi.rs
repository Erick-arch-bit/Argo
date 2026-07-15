use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ast::Programa;
use crate::evaluator::{configurar_entorno_global, evaluar_programa, Objeto};
use crate::lexer::Lexer;
use crate::parser::Parser;

/// Retorna la versión de Argo como cadena C.
///
/// # Safety
/// El puntero retornado debe liberarse con `argo_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn argo_version() -> *mut c_char {
    CString::new("2.0.0").unwrap().into_raw()
}

/// Evalúa código Argo y retorna el resultado como cadena C.
///
/// # Safety
/// `code` debe ser un puntero válido a una cadena C terminada en nulo.
/// El puntero retornado debe liberarse con `argo_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn argo_evaluar(code: *const c_char) -> *mut c_char {
    let code_str = match unsafe { CStr::from_ptr(code) }.to_str() {
        Ok(s) => s,
        Err(_) => return CString::new("Error: entrada inválida").unwrap().into_raw(),
    };

    let mut entorno = configurar_entorno_global();
    let lexer = Lexer::nuevo(code_str);
    let mut parser = Parser::nuevo(lexer);
    let programa: Programa = parser.parsear_programa();

    if !parser.errores.is_empty() {
        let msg = parser.errores.join("; ");
        return CString::new(format!("Error: {}", msg)).unwrap().into_raw();
    }

    let resultado = evaluar_programa(&programa, &mut entorno);

    match resultado {
        Objeto::Error(m, _) => {
            CString::new(format!("Error: {}", m)).unwrap().into_raw()
        }
        Objeto::Excepcion(v) => {
            CString::new(format!("Excepción: {}", v)).unwrap().into_raw()
        }
        _ => {
            CString::new(format!("{}", resultado)).unwrap().into_raw()
        }
    }
}

/// Libera una cadena C previamente asignada por Argo.
///
/// # Safety
/// `s` debe ser un puntero válido previamente retornado por `argo_version`
/// o `argo_evaluar`, o null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn argo_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
