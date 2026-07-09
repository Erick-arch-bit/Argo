use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::ast::Programa;
use crate::evaluator::{configurar_entorno_global, evaluar_programa, Objeto};
use crate::lexer::Lexer;
use crate::parser::Parser;

#[unsafe(no_mangle)]
pub extern "C" fn argo_version() -> *mut c_char {
    CString::new("1.5.0").unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn argo_evaluar(code: *const c_char) -> *mut c_char {
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

#[unsafe(no_mangle)]
pub extern "C" fn argo_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
