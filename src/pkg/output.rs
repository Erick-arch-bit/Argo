// ---------------------------------------------------------------------------
// output.rs — Output informativo, colores ANSI, barra de progreso
// ---------------------------------------------------------------------------
// Sin dependencias externas. Detecta TTY para no usar colores en pipes.

use std::io::{self, Write};

#[derive(Clone)]
pub struct Output {
    verbose: bool,
    quiet: bool,
    is_tty: bool,
}

impl Output {
    pub fn new() -> Self {
        Output {
            verbose: false,
            quiet: false,
            is_tty: io::IsTerminal::is_terminal(&io::stdout()),
        }
    }

    pub fn verbose(mut self) -> Self { self.verbose = true; self }
    pub fn quiet(mut self) -> Self { self.quiet = true; self }

    pub fn verde(&self, msg: &str) {
        if !self.quiet {
            if self.is_tty {
                print!("\x1b[32m{}\x1b[0m", msg);
            } else {
                print!("{}", msg);
            }
            io::stdout().flush().ok();
        }
    }

    pub fn rojo(&self, msg: &str) {
        if self.is_tty {
            eprint!("\x1b[31m{}\x1b[0m", msg);
        } else {
            eprint!("{}", msg);
        }
        io::stderr().flush().ok();
    }

    pub fn amarillo(&self, msg: &str) {
        if !self.quiet {
            if self.is_tty {
                print!("\x1b[33m{}\x1b[0m", msg);
            } else {
                print!("{}", msg);
            }
            io::stdout().flush().ok();
        }
    }

    pub fn azul(&self, msg: &str) {
        if !self.quiet {
            if self.is_tty {
                print!("\x1b[34m{}\x1b[0m", msg);
            } else {
                print!("{}", msg);
            }
            io::stdout().flush().ok();
        }
    }

    pub fn dim(&self, msg: &str) {
        if !self.quiet {
            if self.is_tty {
                print!("\x1b[2m{}\x1b[0m", msg);
            } else {
                print!("{}", msg);
            }
            io::stdout().flush().ok();
        }
    }

    pub fn bold(&self, msg: &str) {
        if !self.quiet {
            if self.is_tty {
                print!("\x1b[1m{}\x1b[0m", msg);
            } else {
                print!("{}", msg);
            }
            io::stdout().flush().ok();
        }
    }

    pub fn linea(&self, msg: &str) {
        if !self.quiet {
            println!("{}", msg);
        }
    }

    pub fn linea_stderr(&self, msg: &str) {
        eprintln!("{}", msg);
    }

    pub fn verbose_log(&self, msg: &str) {
        if self.verbose && !self.quiet {
            self.dim(msg);
            println!();
        }
    }

    /// Imprime una barra de progreso: "Descargando [=====>    ] 5/12 archivos"
    pub fn barra_progreso(&self, actual: usize, total: usize, etiqueta: &str) {
        if !self.is_tty || self.quiet {
            return;
        }
        let ancho = 30;
        let llenos = if total > 0 { (actual as f64 / total as f64 * ancho as f64) as usize } else { 0 };
        let vacios = ancho - llenos;

        print!("\r\x1b[2m  {} [", etiqueta);
        for _ in 0..llenos { print!("="); }
        if llenos < ancho { print!(">"); }
        for _ in 1..vacios { print!(" "); }
        print!("] {}/{} archivos\x1b[0m", actual, total);
        io::stdout().flush().ok();
    }

    /// Imprime OK verde en la misma línea.
    pub fn ok(&self) {
        if !self.quiet {
            self.verde(" OK");
            println!();
        }
    }

    /// Imprime ERROR rojo.
    pub fn error(&self, msg: &str) {
        self.rojo("Error: ");
        self.rojo(msg);
        eprintln!();
    }

    /// Imprime resultado final.
    pub fn resultado(&self, paquete: &str, version: &str) {
        self.verde("✅ ");
        self.bold(&format!("Paquete '{}' {} publicado exitosamente.", paquete, version));
        println!();
    }
}

impl Default for Output {
    fn default() -> Self { Self::new() }
}
