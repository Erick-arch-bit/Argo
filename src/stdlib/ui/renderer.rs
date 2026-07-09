use std::io::{self, BufWriter, Stdout, Write};

pub struct TerminalRenderer {
    buf: BufWriter<Stdout>,
}

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRenderer {
    pub fn new() -> Self {
        TerminalRenderer {
            buf: BufWriter::new(io::stdout()),
        }
    }

    #[inline]
    fn write_raw(&mut self, bytes: &[u8]) {
        let _ = self.buf.write_all(bytes);
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        let _ = self.buf.write_all(s.as_bytes());
    }

    #[inline]
    pub fn flush(&mut self) {
        let _ = self.buf.flush();
    }

    pub fn limpiar_pantalla(&mut self) {
        self.write_raw(b"\x1b[2J\x1b[H");
        self.flush();
    }

    pub fn mover_cursor(&mut self, x: usize, y: usize) {
        let seq = format!("\x1b[{};{}H", y + 1, x + 1);
        self.write_str(&seq);
    }

    pub fn color_fondo(&mut self, r: u8, g: u8, b: u8) {
        let seq = format!("\x1b[48;2;{};{};{}m", r, g, b);
        self.write_str(&seq);
    }

    pub fn color_texto(&mut self, r: u8, g: u8, b: u8) {
        let seq = format!("\x1b[38;2;{};{};{}m", r, g, b);
        self.write_str(&seq);
    }

    #[inline]
    pub fn resetear_colores(&mut self) {
        self.write_raw(b"\x1b[0m");
    }

    pub fn dibujar_texto(
        &mut self,
        x: usize,
        y: usize,
        texto: &str,
        color_r: u8,
        color_g: u8,
        color_b: u8,
    ) {
        self.mover_cursor(x, y);
        self.color_texto(color_r, color_g, color_b);
        self.write_str(texto);
        self.resetear_colores();
    }

    pub fn dibujar_borde(&mut self, x: usize, y: usize, ancho: usize, alto: usize) {
        if ancho < 2 || alto < 2 {
            return;
        }

        let interior = ancho - 2;

        // Borde superior: ┌─────┐
        self.mover_cursor(x, y);
        self.write_raw(b"\xe2\x94\x8c");
        for _ in 0..interior {
            self.write_raw(b"\xe2\x94\x80");
        }
        self.write_raw(b"\xe2\x94\x90\n");

        // Filas interiores: │     │
        for fila in 1..alto - 1 {
            self.mover_cursor(x, y + fila);
            self.write_raw(b"\xe2\x94\x82");
            for _ in 0..interior {
                self.write_raw(b" ");
            }
            self.write_raw(b"\xe2\x94\x82\n");
        }

        // Borde inferior: └─────┘
        self.mover_cursor(x, y + alto - 1);
        self.write_raw(b"\xe2\x94\x94");
        for _ in 0..interior {
            self.write_raw(b"\xe2\x94\x80");
        }
        self.write_raw(b"\xe2\x94\x98");
        self.flush();
    }
}
