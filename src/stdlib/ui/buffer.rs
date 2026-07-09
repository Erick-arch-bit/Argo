use std::io::{self, BufWriter, Write};

// ---------------------------------------------------------------------------
// Celda — Un carácter en la pantalla con colores de fondo y texto
// ---------------------------------------------------------------------------
#[derive(Clone)]
pub struct Celda {
    pub caracter: char,
    pub fondo: (u8, u8, u8),
    pub texto: (u8, u8, u8),
}

impl Default for Celda {
    fn default() -> Self {
        Celda {
            caracter: ' ',
            fondo: (0, 0, 0),
            texto: (255, 255, 255),
        }
    }
}

impl Celda {
    pub fn igual_a(&self, other: &Celda) -> bool {
        self.caracter == other.caracter && self.fondo == other.fondo && self.texto == other.texto
    }
}

// ---------------------------------------------------------------------------
// PantallaBuffer — Buffer doble de pantalla en RAM
// ---------------------------------------------------------------------------
pub struct PantallaBuffer {
    pub ancho: usize,
    pub alto: usize,
    celdas: Vec<Celda>,
    anterior: Vec<Celda>,
}

impl PantallaBuffer {
    /// Crea un nuevo buffer con el tamaño dado.
    pub fn new(ancho: usize, alto: usize) -> Self {
        let total = ancho * alto;
        PantallaBuffer {
            ancho,
            alto,
            celdas: vec![Celda::default(); total],
            anterior: vec![Celda::default(); total],
        }
    }

    /// Intenta obtener el tamaño real de la terminal.
    /// En Unix ejecuta `stty size`, en Windows usa `mode con`,
    /// si falla devuelve 80x24.
    pub fn detectar_terminal() -> (usize, usize) {
        #[cfg(unix)]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("stty").arg("size").output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let parts: Vec<&str> = stdout.split_whitespace().collect();
                if parts.len() == 2
                    && let (Ok(alto), Ok(ancho)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                    && ancho > 0
                    && alto > 0
                {
                    return (ancho, alto);
                }
            }
        }
        #[cfg(windows)]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("mode")
                .args(["con", "/status"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let line = line.trim();
                    if line.contains("Columnas:") || line.contains("Columns:") {
                        let val: String = line.chars().filter(|c| c.is_ascii_digit()).collect();
                        if let Ok(ancho) = val.parse::<usize>() {
                            if let Ok(output2) = Command::new("mode")
                                .args(["con", "/status"])
                                .output()
                            {
                                let s2 = String::from_utf8_lossy(&output2.stdout);
                                for l2 in s2.lines() {
                                    let l2 = l2.trim();
                                    if l2.contains("Lineas:") || l2.contains("Lines:") {
                                        let v2: String =
                                            l2.chars().filter(|c| c.is_ascii_digit()).collect();
                                        if let Ok(alto) = v2.parse::<usize>() {
                                            return (ancho, alto);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        (80, 24)
    }

    /// Limpia todo el buffer con celdas por defecto.
    pub fn limpiar(&mut self) {
        for celda in &mut self.celdas {
            *celda = Celda::default();
        }
    }

    /// Obtiene el índice lineal de una celda en la grilla.
    #[inline]
    fn indice(&self, x: usize, y: usize) -> usize {
        y * self.ancho + x
    }

    /// Escribe un carácter en una posición del buffer.
    #[inline]
    pub fn escribir_celda(
        &mut self,
        x: usize,
        y: usize,
        caracter: char,
        fondo: (u8, u8, u8),
        texto: (u8, u8, u8),
    ) {
        if x < self.ancho && y < self.alto {
            let idx = self.indice(x, y);
            self.celdas[idx] = Celda {
                caracter,
                fondo,
                texto,
            };
        }
    }

    /// Dibuja texto en el buffer starting en (x, y).
    pub fn dibujar_texto(
        &mut self,
        x: usize,
        y: usize,
        texto: &str,
        fondo: (u8, u8, u8),
        color_texto: (u8, u8, u8),
    ) {
        for (i, ch) in texto.chars().enumerate() {
            let cx = x + i;
            if cx >= self.ancho {
                break;
            }
            self.escribir_celda(cx, y, ch, fondo, color_texto);
        }
    }

    /// Dibuja un rectángulo relleno en el buffer.
    pub fn dibujar_rectangulo(
        &mut self,
        x: usize,
        y: usize,
        ancho: usize,
        alto: usize,
        fondo: (u8, u8, u8),
    ) {
        for dy in 0..alto {
            let cy = y + dy;
            if cy >= self.alto {
                break;
            }
            for dx in 0..ancho {
                let cx = x + dx;
                if cx >= self.ancho {
                    break;
                }
                self.escribir_celda(cx, cy, ' ', fondo, (255, 255, 255));
            }
        }
    }

    /// Dibuja un borde Unicode alrededor de un rectángulo.
    pub fn dibujar_borde(
        &mut self,
        x: usize,
        y: usize,
        ancho: usize,
        alto: usize,
        fondo: (u8, u8, u8),
        color: (u8, u8, u8),
    ) {
        if ancho < 2 || alto < 2 {
            return;
        }

        // Esquinas
        self.escribir_celda(x, y, '┌', fondo, color);
        self.escribir_celda(x + ancho - 1, y, '┐', fondo, color);
        self.escribir_celda(x, y + alto - 1, '└', fondo, color);
        self.escribir_celda(x + ancho - 1, y + alto - 1, '┘', fondo, color);

        // Borde superior e inferior
        for dx in 1..ancho - 1 {
            self.escribir_celda(x + dx, y, '─', fondo, color);
            self.escribir_celda(x + dx, y + alto - 1, '─', fondo, color);
        }

        // Lados izquierdo y derecho
        for dy in 1..alto - 1 {
            self.escribir_celda(x, y + dy, '│', fondo, color);
            self.escribir_celda(x + ancho - 1, y + dy, '│', fondo, color);
        }
    }

    /// Flush el buffer a la terminal real.
    /// Oculta el cursor, imprime todo, muestra el cursor.
    /// Compara con el buffer anterior para minimizar writes.
    pub fn flush(&mut self) {
        let mut out = BufWriter::new(io::stdout());

        // Ocultar cursor
        let _ = out.write_all(b"\x1b[?25l");

        // Mover cursor al origen
        let _ = out.write_all(b"\x1b[H");

        let total = self.ancho * self.alto;

        for i in 0..total {
            let celda = &self.celdas[i];
            let prev = &self.anterior[i];

            if celda.igual_a(prev) {
                continue;
            }

            let x = i % self.ancho;
            let y = i / self.ancho;

            // Mover cursor a la posición
            let pos = format!("\x1b[{};{}H", y + 1, x + 1);
            let _ = out.write_all(pos.as_bytes());

            // Color de fondo
            let bg = format!(
                "\x1b[48;2;{};{};{}m",
                celda.fondo.0, celda.fondo.1, celda.fondo.2
            );
            let _ = out.write_all(bg.as_bytes());

            // Color de texto
            let fg = format!(
                "\x1b[38;2;{};{};{}m",
                celda.texto.0, celda.texto.1, celda.texto.2
            );
            let _ = out.write_all(fg.as_bytes());

            // Carácter
            let mut buf = [0u8; 4];
            let s = celda.caracter.encode_utf8(&mut buf);
            let _ = out.write_all(s.as_bytes());
        }

        // Reset colores
        let _ = out.write_all(b"\x1b[0m");

        // Mostrar cursor
        let _ = out.write_all(b"\x1b[?25h");

        let _ = out.flush();

        // Copiar buffer actual al anterior
        self.anterior.clone_from_slice(&self.celdas);
    }

    /// Dibuja un rectángulo con borde (ventana).
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn dibujar_ventana(
        &mut self,
        x: usize,
        y: usize,
        ancho: usize,
        alto: usize,
        titulo: &str,
        fondo: (u8, u8, u8),
        borde: (u8, u8, u8),
    ) {
        self.dibujar_rectangulo(x, y, ancho, alto, fondo);
        self.dibujar_borde(x, y, ancho, alto, fondo, borde);

        if !titulo.is_empty() {
            let max_titulo = ancho.saturating_sub(4);
            let titulo_corto: String = titulo.chars().take(max_titulo).collect();
            self.dibujar_texto(x + 2, y, &titulo_corto, fondo, borde);
        }
    }
}
