use std::io::Write;

// ---------------------------------------------------------------------------
// Celda — Un carácter en la pantalla con colores fg/bg
// ---------------------------------------------------------------------------
#[derive(Clone, PartialEq)]
pub struct Celda {
    pub caracter: char,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
}

impl Default for Celda {
    fn default() -> Self {
        Celda {
            caracter: ' ',
            fg: (255, 255, 255),
            bg: (0, 0, 0),
        }
    }
}

// ---------------------------------------------------------------------------
// DirtyRect — Rectángulo sucio que necesita re-dibujarse
// ---------------------------------------------------------------------------
#[derive(Clone, Copy)]
pub struct DirtyRect {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

// ---------------------------------------------------------------------------
// RenderEngine — Motor de renderizado rápido con dirty rects
// ---------------------------------------------------------------------------
pub struct RenderEngine {
    pub ancho: usize,
    pub alto: usize,
    pub backbuffer: Vec<Celda>,
    pub frontbuffer: Vec<Celda>,
    dirty_rects: Vec<DirtyRect>,
}

impl RenderEngine {
    /// Crea un nuevo motor de renderizado.
    pub fn new(ancho: usize, alto: usize) -> Self {
        let total = ancho * alto;
        let celda_default = Celda::default();
        RenderEngine {
            ancho,
            alto,
            backbuffer: vec![celda_default.clone(); total],
            frontbuffer: vec![celda_default; total],
            dirty_rects: Vec::new(),
        }
    }

    /// Detecta el tamaño de la terminal.
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

    /// Índice lineal en la grilla.
    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.ancho + x
    }

    /// Añade un dirty rect, fusionando con existentes si es posible.
    fn marcar_sucio(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let nx = x.min(self.ancho);
        let ny = y.min(self.alto);
        let nw = w.min(self.ancho.saturating_sub(nx));
        let nh = h.min(self.alto.saturating_sub(ny));

        if nw == 0 || nh == 0 {
            return;
        }

        // Intentar fusionar con un rect existente
        for dr in &mut self.dirty_rects {
            // ¿Se superponen o son adyacentes?
            if nx <= dr.x + dr.w
                && nx + nw >= dr.x
                && ny <= dr.y + dr.h
                && ny + nh >= dr.y
            {
                let min_x = nx.min(dr.x);
                let min_y = ny.min(dr.y);
                let max_x = (nx + nw).max(dr.x + dr.w);
                let max_y = (ny + nh).max(dr.y + dr.h);
                dr.x = min_x;
                dr.y = min_y;
                dr.w = max_x - min_x;
                dr.h = max_y - min_y;
                return;
            }
        }

        self.dirty_rects.push(DirtyRect {
            x: nx,
            y: ny,
            w: nw,
            h: nh,
        });
    }

    /// Limpia una zona del backbuffer y la marca como sucia.
    pub fn limpiar_area(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let celda = Celda::default();
        for dy in 0..h {
            let cy = y + dy;
            if cy >= self.alto {
                break;
            }
            for dx in 0..w {
                let cx = x + dx;
                if cx >= self.ancho {
                    break;
                }
                let i = self.idx(cx, cy);
                self.backbuffer[i] = celda.clone();
            }
        }
        self.marcar_sucio(x, y, w, h);
    }

    /// Escribe un carácter en el backbuffer.
    #[inline]
    pub fn poner_celda(&mut self, x: usize, y: usize, caracter: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        if x < self.ancho && y < self.alto {
            let i = self.idx(x, y);
            self.backbuffer[i] = Celda { caracter, fg, bg };
            self.marcar_sucio(x, y, 1, 1);
        }
    }

    /// Escribe texto en el backbuffer starting en (x, y).
    pub fn poner_texto(&mut self, x: usize, y: usize, texto: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        for (i, ch) in texto.chars().enumerate() {
            let cx = x + i;
            if cx >= self.ancho {
                break;
            }
            self.poner_celda(cx, y, ch, fg, bg);
        }
    }

    /// Rellena un rectángulo con un carácter.
    #[allow(clippy::too_many_arguments)]
    pub fn rellenar(&mut self, x: usize, y: usize, w: usize, h: usize, caracter: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        for dy in 0..h {
            let cy = y + dy;
            if cy >= self.alto {
                break;
            }
            for dx in 0..w {
                let cx = x + dx;
                if cx >= self.ancho {
                    break;
                }
                self.poner_celda(cx, cy, caracter, fg, bg);
            }
        }
    }

    /// Dibuja un borde con esquinas redondeadas.
    pub fn poner_borde_redondeado(&mut self, x: usize, y: usize, w: usize, h: usize, color: (u8, u8, u8)) {
        if w < 2 || h < 2 {
            return;
        }

        let bg = (0, 0, 0);

        // Esquinas redondeadas
        self.poner_celda(x, y, '╭', color, bg);
        self.poner_celda(x + w - 1, y, '╮', color, bg);
        self.poner_celda(x, y + h - 1, '╰', color, bg);
        self.poner_celda(x + w - 1, y + h - 1, '╯', color, bg);

        // Borde horizontal superior e inferior
        for dx in 1..w - 1 {
            self.poner_celda(x + dx, y, '─', color, bg);
            self.poner_celda(x + dx, y + h - 1, '─', color, bg);
        }

        // Borde vertical izquierdo y derecho
        for dy in 1..h - 1 {
            self.poner_celda(x, y + dy, '│', color, bg);
            self.poner_celda(x + w - 1, y + dy, '│', color, bg);
        }
    }

    /// Dibuja un borde con esquinas rectas.
    pub fn poner_borde(&mut self, x: usize, y: usize, w: usize, h: usize, color: (u8, u8, u8)) {
        if w < 2 || h < 2 {
            return;
        }

        let bg = (0, 0, 0);

        self.poner_celda(x, y, '┌', color, bg);
        self.poner_celda(x + w - 1, y, '┐', color, bg);
        self.poner_celda(x, y + h - 1, '└', color, bg);
        self.poner_celda(x + w - 1, y + h - 1, '┘', color, bg);

        for dx in 1..w - 1 {
            self.poner_celda(x + dx, y, '─', color, bg);
            self.poner_celda(x + dx, y + h - 1, '─', color, bg);
        }

        for dy in 1..h - 1 {
            self.poner_celda(x, y + dy, '│', color, bg);
            self.poner_celda(x + w - 1, y + dy, '│', color, bg);
        }
    }

    /// Dibuja un rectángulo sólido (sin borde).
    pub fn poner_rectangulo(&mut self, x: usize, y: usize, w: usize, h: usize, bg: (u8, u8, u8)) {
        self.rellenar(x, y, w, h, ' ', (0, 0, 0), bg);
    }

    /// Dibuja texto centrado en un rectángulo.
    pub fn poner_texto_centrado(&mut self, x: usize, y: usize, w: usize, texto: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        let text_len = texto.chars().count();
        if text_len >= w {
            // Texto más ancho que el espacio, recortar
            let recortado: String = texto.chars().take(w.saturating_sub(1)).collect();
            self.poner_texto(x, y, &recortado, fg, bg);
        } else {
            let padding = (w - text_len) / 2;
            self.poner_texto(x + padding, y, texto, fg, bg);
        }
    }

    /// EL MÉTODO MÁGICO — Flush solo los rectángulos sucios a la terminal.
    pub fn flush(&mut self, stdout: &mut impl Write) {
        if self.dirty_rects.is_empty() {
            return;
        }

        // Ocultar cursor
        let _ = stdout.write_all(b"\x1b[?25l");

        for dr in &self.dirty_rects {
            for dy in 0..dr.h {
                let cy = dr.y + dy;
                if cy >= self.alto {
                    break;
                }
                for dx in 0..dr.w {
                    let cx = dr.x + dx;
                    if cx >= self.ancho {
                        break;
                    }

                    let i = self.idx(cx, cy);
                    let back = &self.backbuffer[i];
                    let front = &self.frontbuffer[i];

                    if back == front {
                        continue;
                    }

                    // Mover cursor
                    let pos = format!("\x1b[{};{}H", cy + 1, cx + 1);
                    let _ = stdout.write_all(pos.as_bytes());

                    // Color de fondo
                    let bg = format!("\x1b[48;2;{};{};{}m", back.bg.0, back.bg.1, back.bg.2);
                    let _ = stdout.write_all(bg.as_bytes());

                    // Color de texto
                    let fg = format!("\x1b[38;2;{};{};{}m", back.fg.0, back.fg.1, back.fg.2);
                    let _ = stdout.write_all(fg.as_bytes());

                    // Carácter
                    let mut buf = [0u8; 4];
                    let s = back.caracter.encode_utf8(&mut buf);
                    let _ = stdout.write_all(s.as_bytes());

                    // Actualizar frontbuffer
                    self.frontbuffer[i] = back.clone();
                }
            }
        }

        // Reset colores
        let _ = stdout.write_all(b"\x1b[0m");
        // Mostrar cursor
        let _ = stdout.write_all(b"\x1b[?25h");
        let _ = stdout.flush();

        // Limpiar dirty rects
        self.dirty_rects.clear();
    }

    /// Flush completo: fuerza re-dibujar todo.
    #[allow(dead_code)]
    pub fn flush_full(&mut self, stdout: &mut impl Write) {
        // Marcar todo como sucio
        self.dirty_rects.push(DirtyRect {
            x: 0,
            y: 0,
            w: self.ancho,
            h: self.alto,
        });
        self.flush(stdout);
    }

    /// Limpia todo el backbuffer.
    pub fn limpiar_todo(&mut self) {
        let celda = Celda::default();
        for celda_slot in &mut self.backbuffer {
            *celda_slot = celda.clone();
        }
        self.marcar_sucio(0, 0, self.ancho, self.alto);
    }
}
