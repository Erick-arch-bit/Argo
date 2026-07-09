use std::fmt;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArgoError {
    pub mensaje: String,
    pub linea: usize,
    pub archivo: String,
    pub pila: Vec<String>,
}

#[allow(dead_code)]
impl ArgoError {
    pub fn nuevo(mensaje: String, linea: usize, archivo: String) -> Self {
        ArgoError {
            mensaje,
            linea,
            archivo,
            pila: Vec::new(),
        }
    }

    pub fn push_pila(&mut self, nombre: &str) {
        self.pila.push(nombre.to_string());
    }

    pub fn pop_pila(&mut self) {
        self.pila.pop();
    }
}

impl fmt::Display for ArgoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error en [{}:{}]: {}", self.archivo, self.linea, self.mensaje)?;
        for frame in self.pila.iter().rev() {
            writeln!(f, "  en {}()", frame)?;
        }
        Ok(())
    }
}