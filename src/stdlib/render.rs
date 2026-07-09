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
    #[inline]
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
// BufWriter ANSI — Buffer para secuencias ANSI sin allocaciones
// ---------------------------------------------------------------------------
struct AnsiBuf {
    buf: [u8; 4096],
    pos: usize,
}

impl AnsiBuf {
    #[inline]
    fn new() -> Self {
        AnsiBuf {
            buf: [0u8; 4096],
            pos: 0,
        }
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        let remaining = self.buf.len() - self.pos;
        if bytes.len() <= remaining {
            self.buf[self.pos..self.pos + bytes.len()].copy_from_slice(bytes);
            self.pos += bytes.len();
        }
    }

    #[inline]
    fn write_byte(&mut self, b: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = b;
            self.pos += 1;
        }
    }

    /// Escribe un entero como ASCII (sin format!).
    #[inline]
    fn write_u8(&mut self, n: u8) {
        if n >= 100 {
            self.write_byte(b'0' + n / 100);
            self.write_byte(b'0' + (n / 10) % 10);
            self.write_byte(b'0' + n % 10);
        } else if n >= 10 {
            self.write_byte(b'0' + n / 10);
            self.write_byte(b'0' + n % 10);
        } else {
            self.write_byte(b'0' + n);
        }
    }

    /// Escribe un usize (max 4 dígitos para coordenadas de terminal).
    #[inline]
    fn write_usize(&mut self, mut n: usize) {
        if n == 0 {
            self.write_byte(b'0');
            return;
        }
        let mut digits = [0u8; 4];
        let mut i = 0;
        while n > 0 && i < 4 {
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
            i += 1;
        }
        let start = i;
        while i > 0 {
            i -= 1;
            self.write_byte(digits[i]);
        }
        let _ = start;
    }

    /// Mueve cursor a posición: \x1b[{y};{x}H
    #[inline]
    fn cursor(&mut self, x: usize, y: usize) {
        self.write_bytes(b"\x1b[");
        self.write_usize(y + 1);
        self.write_byte(b';');
        self.write_usize(x + 1);
        self.write_byte(b'H');
    }

    /// Color de fondo: \x1b[48;2;R;G;Bm
    #[inline]
    fn bg(&mut self, r: u8, g: u8, b: u8) {
        self.write_bytes(b"\x1b[48;2;");
        self.write_u8(r);
        self.write_byte(b';');
        self.write_u8(g);
        self.write_byte(b';');
        self.write_u8(b);
        self.write_byte(b'm');
    }

    /// Color de texto: \x1b[38;2;R;G;Bm
    #[inline]
    fn fg(&mut self, r: u8, g: u8, b: u8) {
        self.write_bytes(b"\x1b[38;2;");
        self.write_u8(r);
        self.write_byte(b';');
        self.write_u8(g);
        self.write_byte(b';');
        self.write_u8(b);
        self.write_byte(b'm');
    }

    /// Carácter UTF-8
    #[inline]
    fn char(&mut self, c: char) {
        let mut tmp = [0u8; 4];
        let s = c.encode_utf8(&mut tmp);
        self.write_bytes(s.as_bytes());
    }

    #[inline]
    fn flush_to(&mut self, out: &mut impl Write) {
        if self.pos > 0 {
            let _ = out.write_all(&self.buf[..self.pos]);
            self.pos = 0;
        }
    }
}

// ---------------------------------------------------------------------------
// RenderEngine — Motor de renderizado rápido con dirty rects
// ---------------------------------------------------------------------------
pub struct RenderEngine {
    pub ancho: usize,
    pub alto: usize,
    pub backbuffer: Vec<Celda>,
    pub frontbuffer: Vec<Celda>,
    /// Flag por celda: true si la celda está sucia (cambió desde último flush)
    dirty_cells: Vec<bool>,
    dirty_rects: Vec<DirtyRect>,
}

impl RenderEngine {
    /// Crea un nuevo motor de renderizado.
    pub fn new(ancho: usize, alto: usize) -> Self {
        let total = ancho * alto;
        RenderEngine {
            ancho,
            alto,
            backbuffer: vec![Celda::default(); total],
            frontbuffer: vec![Celda::default(); total],
            dirty_cells: vec![true; total], // todo sucio al inicio
            dirty_rects: Vec::with_capacity(32),
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

    /// Marca una zona como sucia en el bitset y añade dirty rect.
    #[inline]
    fn marcar_sucio(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let nx = x.min(self.ancho);
        let ny = y.min(self.alto);
        let nw = w.min(self.ancho.saturating_sub(nx));
        let nh = h.min(self.alto.saturating_sub(ny));
        if nw == 0 || nh == 0 {
            return;
        }

        // Marcar celdas individuales
        for dy in 0..nh {
            let row = (ny + dy) * self.ancho + nx;
            for dx in 0..nw {
                self.dirty_cells[row + dx] = true;
            }
        }

        // Fusionar con rect existente
        for dr in &mut self.dirty_rects {
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
    #[inline]
    pub fn limpiar_area(&mut self, x: usize, y: usize, w: usize, h: usize) {
        let nx = x.min(self.ancho);
        let ny = y.min(self.alto);
        let nw = w.min(self.ancho.saturating_sub(nx));
        let nh = h.min(self.alto.saturating_sub(ny));
        if nw == 0 || nh == 0 {
            return;
        }

        let default = Celda::default();
        for dy in 0..nh {
            let row_start = (ny + dy) * self.ancho + nx;
            for dx in 0..nw {
                self.backbuffer[row_start + dx] = default.clone();
            }
        }
        self.marcar_sucio(nx, ny, nw, nh);
    }

    /// Escribe un carácter en el backbuffer (sin marcar dirty individual).
    #[inline]
    pub fn poner_celda(&mut self, x: usize, y: usize, caracter: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        if x < self.ancho && y < self.alto {
            let i = self.idx(x, y);
            self.backbuffer[i] = Celda { caracter, fg, bg };
            self.dirty_cells[i] = true;
        }
    }

    /// Escribe texto en el backbuffer starting en (x, y).
    #[inline]
    pub fn poner_texto(&mut self, x: usize, y: usize, texto: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        if y >= self.alto {
            return;
        }
        let row_start = y * self.ancho;
        let max_x = self.ancho;
        let mut cx = x;
        for ch in texto.chars() {
            if cx >= max_x {
                break;
            }
            let i = row_start + cx;
            self.backbuffer[i] = Celda { caracter: ch, fg, bg };
            self.dirty_cells[i] = true;
            cx += 1;
        }
        if cx > x {
            self.marcar_sucio(x, y, cx - x, 1);
        }
    }

    /// Rellena un rectángulo con un carácter (bulk write al backbuffer).
    #[allow(clippy::too_many_arguments)]
    #[inline]
    pub fn rellenar(&mut self, x: usize, y: usize, w: usize, h: usize, caracter: char, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        let nx = x.min(self.ancho);
        let ny = y.min(self.alto);
        let nw = w.min(self.ancho.saturating_sub(nx));
        let nh = h.min(self.alto.saturating_sub(ny));
        if nw == 0 || nh == 0 {
            return;
        }

        let celda = Celda { caracter, fg, bg };
        for dy in 0..nh {
            let row_start = (ny + dy) * self.ancho + nx;
            for dx in 0..nw {
                self.backbuffer[row_start + dx] = celda.clone();
            }
        }
        self.marcar_sucio(nx, ny, nw, nh);
    }

    /// Dibuja un borde con esquinas redondeadas.
    pub fn poner_borde_redondeado(&mut self, x: usize, y: usize, w: usize, h: usize, color: (u8, u8, u8)) {
        if w < 2 || h < 2 {
            return;
        }
        let bg = (0, 0, 0);
        self.poner_celda(x, y, '╭', color, bg);
        self.poner_celda(x + w - 1, y, '╮', color, bg);
        self.poner_celda(x, y + h - 1, '╰', color, bg);
        self.poner_celda(x + w - 1, y + h - 1, '╯', color, bg);
        for dx in 1..w - 1 {
            self.poner_celda(x + dx, y, '─', color, bg);
            self.poner_celda(x + dx, y + h - 1, '─', color, bg);
        }
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
    #[inline]
    pub fn poner_rectangulo(&mut self, x: usize, y: usize, w: usize, h: usize, bg: (u8, u8, u8)) {
        self.rellenar(x, y, w, h, ' ', (0, 0, 0), bg);
    }

    /// Dibuja texto centrado en un rectángulo.
    pub fn poner_texto_centrado(&mut self, x: usize, y: usize, w: usize, texto: &str, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
        let text_len = texto.chars().count();
        if text_len >= w {
            let recortado: String = texto.chars().take(w.saturating_sub(1)).collect();
            self.poner_texto(x, y, &recortado, fg, bg);
        } else {
            let padding = (w - text_len) / 2;
            self.poner_texto(x + padding, y, texto, fg, bg);
        }
    }

    /// EL MÉTODO MÁGICO — Flush con buffer ANSI zero-alloc.
    ///
    /// Optimizaciones:
    /// 1. Buffer ANSI de 4KB evita heap allocations por cada secuencia
    /// 2. Solo itera dirty_rects (no toda la pantalla)
    /// 3. Usa dirty_cells para salto rápido de celdas limpias
    /// 4. Escritura secuencial por filas para minimizar movimientos de cursor
    /// 5. Batch de colores: solo escribe cuando cambian fg/bg
    pub fn flush(&mut self, stdout: &mut impl Write) {
        if self.dirty_rects.is_empty() {
            return;
        }

        let mut ansi = AnsiBuf::new();

        // Ocultar cursor
        ansi.write_bytes(b"\x1b[?25l");

        for dr in &self.dirty_rects {
            let mut last_fg = (0, 0, 0);
            let mut last_bg = (0, 0, 0);
            let mut need_cursor = true;

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

                    let i = cy * self.ancho + cx;

                    // Skip celdas no sucias (rápido con bitset)
                    if !self.dirty_cells[i] {
                        need_cursor = true;
                        continue;
                    }

                    let back = &self.backbuffer[i];

                    // Mover cursor solo cuando es necesario
                    if need_cursor {
                        ansi.cursor(cx, cy);
                    }

                    // Solo escribir colores si cambiaron
                    if back.bg != last_bg {
                        ansi.bg(back.bg.0, back.bg.1, back.bg.2);
                        last_bg = back.bg;
                    }
                    if back.fg != last_fg {
                        ansi.fg(back.fg.0, back.fg.1, back.fg.2);
                        last_fg = back.fg;
                    }

                    // Carácter
                    ansi.char(back.caracter);

                    // Flush buffer si está casi lleno
                    if ansi.pos > 3900 {
                        ansi.flush_to(stdout);
                    }

                    // Actualizar frontbuffer
                    self.frontbuffer[i] = back.clone();
                    self.dirty_cells[i] = false;

                    // La siguiente celda necesita cursor positioning
                    need_cursor = true;
                }
            }
        }

        // Reset colores
        ansi.write_bytes(b"\x1b[0m");
        // Mostrar cursor
        ansi.write_bytes(b"\x1b[?25h");

        // Flush final
        ansi.flush_to(stdout);
        let _ = stdout.flush();

        // Limpiar dirty rects
        self.dirty_rects.clear();
    }

    /// Flush completo: fuerza re-dibujar todo.
    #[allow(dead_code)]
    pub fn flush_full(&mut self, stdout: &mut impl Write) {
        for cell in &mut self.dirty_cells {
            *cell = true;
        }
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
        for dirty in &mut self.dirty_cells {
            *dirty = true;
        }
        self.marcar_sucio(0, 0, self.ancho, self.alto);
    }

    /// Retorna número de celdas sucias (para métricas).
    #[allow(dead_code)]
    pub fn dirty_count(&self) -> usize {
        self.dirty_cells.iter().filter(|&&d| d).count()
    }

    /// Retorna número de dirty rects (para métricas).
    #[allow(dead_code)]
    pub fn rect_count(&self) -> usize {
        self.dirty_rects.len()
    }
}
