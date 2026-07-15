#![allow(dead_code)]

use std::io::Write;

// ---------------------------------------------------------------------------
// Celda — Un carácter en la pantalla con colores fg/bg separados
// ---------------------------------------------------------------------------
#[derive(Clone, PartialEq)]
pub struct Celda {
    pub caracter: char,
    pub fg_r: u8,
    pub fg_g: u8,
    pub fg_b: u8,
    pub bg_r: u8,
    pub bg_g: u8,
    pub bg_b: u8,
}

impl Default for Celda {
    #[inline]
    fn default() -> Self {
        Celda {
            caracter: ' ',
            fg_r: 200, fg_g: 200, fg_b: 200,
            bg_r: 0, bg_g: 0, bg_b: 0,
        }
    }
}

impl Celda {
    #[inline]
    fn es_igual(&self, otro: &Celda) -> bool {
        self.caracter == otro.caracter
            && self.fg_r == otro.fg_r
            && self.fg_g == otro.fg_g
            && self.fg_b == otro.fg_b
            && self.bg_r == otro.bg_r
            && self.bg_g == otro.bg_g
            && self.bg_b == otro.bg_b
    }
}

// ---------------------------------------------------------------------------
// AnsiBuf — Buffer de 4KB en stack para secuencias ANSI sin allocaciones
// ---------------------------------------------------------------------------
struct AnsiBuf {
    buf: [u8; 4096],
    pos: usize,
}

impl AnsiBuf {
    #[inline]
    fn new() -> Self {
        AnsiBuf { buf: [0u8; 4096], pos: 0 }
    }

    #[inline]
    fn write_byte(&mut self, b: u8) {
        if self.pos < self.buf.len() {
            self.buf[self.pos] = b;
            self.pos += 1;
        }
    }

    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        let remaining = self.buf.len() - self.pos;
        let len = bytes.len().min(remaining);
        if len > 0 {
            self.buf[self.pos..self.pos + len].copy_from_slice(&bytes[..len]);
            self.pos += len;
        }
    }

    #[inline]
    fn write_u16(&mut self, mut n: u16) {
        let mut digits = [0u8; 5];
        let mut i = 5;
        if n == 0 {
            self.write_byte(b'0');
            return;
        }
        while n > 0 {
            i -= 1;
            digits[i] = b'0' + (n % 10) as u8;
            n /= 10;
        }
        self.write_bytes(&digits[i..]);
    }

    #[inline]
    fn write_u8_rgb(&mut self, n: u8) {
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

    #[inline]
    fn flush_to<W: Write>(self, out: &mut W) {
        if self.pos > 0 {
            let _ = out.write_all(&self.buf[..self.pos]);
        }
    }
}

// ---------------------------------------------------------------------------
// MotorRenderizado — Doble buffer con dirty-rect diffing
// ---------------------------------------------------------------------------
pub struct MotorRenderizado {
    pub ancho: usize,
    pub alto: usize,
    backbuffer: Vec<Celda>,
    frontbuffer: Vec<Celda>,
    pub sucio: bool,
}

impl MotorRenderizado {
    /// Detecta tamaño de terminal en Unix via stty. Default 80x24.
    fn detectar_terminal() -> (usize, usize) {
        #[cfg(unix)]
        {
            if let Ok(out) = std::process::Command::new("stty")
                .arg("size")
                .output()
            {
                let texto = String::from_utf8_lossy(&out.stdout);
                let partes: Vec<&str> = texto.trim().split_whitespace().collect();
                if partes.len() == 2 {
                    if let (Ok(alto), Ok(ancho)) =
                        (partes[0].parse::<usize>(), partes[1].parse::<usize>())
                    {
                        if ancho > 0 && alto > 0 {
                            return (ancho, alto);
                        }
                    }
                }
            }
        }
        (80, 24)
    }

    pub fn new() -> Self {
        let (ancho, alto) = Self::detectar_terminal();
        let total = ancho * alto;
        MotorRenderizado {
            ancho,
            alto,
            backbuffer: vec![Celda::default(); total],
            frontbuffer: vec![Celda::default(); total],
            sucio: true,
        }
    }

    #[inline]
    pub fn limpiar(&mut self) {
        for celda in &mut self.backbuffer {
            celda.caracter = ' ';
            celda.fg_r = 200;
            celda.fg_g = 200;
            celda.fg_b = 200;
            celda.bg_r = 0;
            celda.bg_g = 0;
            celda.bg_b = 0;
        }
        self.sucio = true;
    }

    #[inline]
    pub fn limpiar_area(&mut self, x: usize, y: usize, w: usize, h: usize) {
        for dy in 0..h {
            let fila = y + dy;
            if fila >= self.alto {
                break;
            }
            for dx in 0..w {
                let col = x + dx;
                if col >= self.ancho {
                    break;
                }
                let idx = fila * self.ancho + col;
                let celda = &mut self.backbuffer[idx];
                celda.caracter = ' ';
                celda.bg_r = 0;
                celda.bg_g = 0;
                celda.bg_b = 0;
            }
        }
        self.sucio = true;
    }

    #[inline]
    pub fn poner_caracter(
        &mut self,
        x: usize,
        y: usize,
        c: char,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) {
        if x >= self.ancho || y >= self.alto {
            return;
        }
        let idx = y * self.ancho + x;
        let celda = &mut self.backbuffer[idx];
        celda.caracter = c;
        celda.fg_r = fg.0;
        celda.fg_g = fg.1;
        celda.fg_b = fg.2;
        celda.bg_r = bg.0;
        celda.bg_g = bg.1;
        celda.bg_b = bg.2;
    }

    #[inline]
    pub fn poner_texto(
        &mut self,
        x: usize,
        y: usize,
        texto: &str,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) {
        for (i, ch) in texto.chars().enumerate() {
            let col = x + i;
            if col >= self.ancho {
                break;
            }
            self.poner_caracter(col, y, ch, fg, bg);
        }
    }

    #[inline]
    pub fn rellenar(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        c: char,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) {
        for dy in 0..h {
            let fila = y + dy;
            if fila >= self.alto {
                break;
            }
            for dx in 0..w {
                let col = x + dx;
                if col >= self.ancho {
                    break;
                }
                self.poner_caracter(col, fila, c, fg, bg);
            }
        }
    }

    #[inline]
    pub fn poner_rectangulo(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        bg: (u8, u8, u8),
    ) {
        for dy in 0..h {
            let fila = y + dy;
            if fila >= self.alto {
                break;
            }
            for dx in 0..w {
                let col = x + dx;
                if col >= self.ancho {
                    break;
                }
                let idx = fila * self.ancho + col;
                let celda = &mut self.backbuffer[idx];
                celda.caracter = ' ';
                celda.bg_r = bg.0;
                celda.bg_g = bg.1;
                celda.bg_b = bg.2;
            }
        }
        self.sucio = true;
    }

    #[inline]
    pub fn poner_borde(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        h: usize,
        color: (u8, u8, u8),
    ) {
        if w == 0 || h == 0 {
            return;
        }
        let bg = (0, 0, 0);
        // Esquina superior izquierda
        self.poner_caracter(x, y, '┌', color, bg);
        // Esquina superior derecha
        if w > 1 {
            self.poner_caracter(x + w - 1, y, '┐', color, bg);
        }
        // Esquina inferior izquierda
        if h > 1 {
            self.poner_caracter(x, y + h - 1, '└', color, bg);
        }
        // Esquina inferior derecha
        if w > 1 && h > 1 {
            self.poner_caracter(x + w - 1, y + h - 1, '┘', color, bg);
        }
        // Borde horizontal superior e inferior
        if w > 2 {
            for dx in 1..w - 1 {
                self.poner_caracter(x + dx, y, '─', color, bg);
                if h > 1 {
                    self.poner_caracter(x + dx, y + h - 1, '─', color, bg);
                }
            }
        }
        // Borde vertical izquierdo y derecho
        if h > 2 {
            for dy in 1..h - 1 {
                self.poner_caracter(x, y + dy, '│', color, bg);
                if w > 1 {
                    self.poner_caracter(x + w - 1, y + dy, '│', color, bg);
                }
            }
        }
    }

    #[inline]
    pub fn poner_texto_centrado(
        &mut self,
        x: usize,
        y: usize,
        w: usize,
        texto: &str,
        fg: (u8, u8, u8),
        bg: (u8, u8, u8),
    ) {
        let tlen = texto.chars().count();
        let offset = if tlen < w { (w - tlen) / 2 } else { 0 };
        self.poner_texto(x + offset, y, texto, fg, bg);
    }

    /// Flush: solo escribe celdas que cambiaron. Zero-alloc ANSI.
    pub fn flush<W: Write>(&mut self, out: &mut W) {
        let total = self.ancho * self.alto;
        let mut ansi = AnsiBuf::new();
        let mut fg_actual = (0u8, 0u8, 0u8);
        let mut bg_actual = (0u8, 0u8, 0u8);
        let mut cursor_pos = (0usize, 0usize);
        let mut necesita_mover = true;

        for i in 0..total {
            let back = &self.backbuffer[i];
            let front = &self.frontbuffer[i];

            if !back.es_igual(front) {
                let x = i % self.ancho;
                let y = i / self.ancho;

                // Mover cursor si no es la posición esperada
                if necesita_mover || x != cursor_pos.0 || y != cursor_pos.1 {
                    ansi.write_bytes(b"\x1b[");
                    ansi.write_u16((y + 1) as u16);
                    ansi.write_byte(b';');
                    ansi.write_u16((x + 1) as u16);
                    ansi.write_byte(b'H');
                }

                // Cambiar bg si es necesario
                if bg_actual != (back.bg_r, back.bg_g, back.bg_b) {
                    ansi.write_bytes(b"\x1b[48;2;");
                    ansi.write_u8_rgb(back.bg_r);
                    ansi.write_byte(b';');
                    ansi.write_u8_rgb(back.bg_g);
                    ansi.write_byte(b';');
                    ansi.write_u8_rgb(back.bg_b);
                    ansi.write_byte(b'm');
                    bg_actual = (back.bg_r, back.bg_g, back.bg_b);
                }

                // Cambiar fg si es necesario
                if fg_actual != (back.fg_r, back.fg_g, back.fg_b) {
                    ansi.write_bytes(b"\x1b[38;2;");
                    ansi.write_u8_rgb(back.fg_r);
                    ansi.write_byte(b';');
                    ansi.write_u8_rgb(back.fg_g);
                    ansi.write_byte(b';');
                    ansi.write_u8_rgb(back.fg_b);
                    ansi.write_byte(b'm');
                    fg_actual = (back.fg_r, back.fg_g, back.fg_b);
                }

                // Escribir carácter (UTF-8)
                let mut tmp = [0u8; 4];
                let len = back.caracter.encode_utf8(&mut tmp).len();
                ansi.write_bytes(&tmp[..len]);

                cursor_pos = (x + 1, y);
                if cursor_pos.0 >= self.ancho {
                    cursor_pos.0 = 0;
                    cursor_pos.1 += 1;
                }
                necesita_mover = false;

                // Actualizar frontbuffer
                self.frontbuffer[i] = back.clone();
            } else {
                // La celda no cambió, la siguiente que cambie necesita mover
                let x = i % self.ancho;
                let y = i / self.alto;
                cursor_pos = (x + 1, y);
                if cursor_pos.0 >= self.ancho {
                    cursor_pos.0 = 0;
                    cursor_pos.1 += 1;
                }
                necesita_mover = true;
            }
        }

        // Reset colores al final
        ansi.write_bytes(b"\x1b[0m");
        ansi.flush_to(out);
        let _ = out.flush();
        self.sucio = false;
    }

    /// Flush completo: resetea todo el frontbuffer forzando redraw total.
    pub fn flush_full<W: Write>(&mut self, out: &mut W) {
        // Resetear frontbuffer para que todo parezca "cambiado"
        for celda in &mut self.frontbuffer {
            celda.caracter = '\0';
        }
        self.flush(out);
    }

    /// Oculta el cursor.
    pub fn ocultar_cursor<W: Write>(out: &mut W) {
        let _ = out.write_all(b"\x1b[?25l");
    }

    /// Muestra el cursor.
    pub fn mostrar_cursor<W: Write>(out: &mut W) {
        let _ = out.write_all(b"\x1b[?25h");
    }

    /// Restaura colores y muestra cursor.
    pub fn restaurar<W: Write>(out: &mut W) {
        let _ = out.write_all(b"\x1b[0m\x1b[?25h");
    }
}
