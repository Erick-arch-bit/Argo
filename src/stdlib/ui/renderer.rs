use std::io::{self, Write};

pub struct TerminalRenderer;

impl Default for TerminalRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalRenderer {
    pub fn new() -> Self {
        TerminalRenderer
    }

    pub fn limpiar_pantalla(&self) {
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().unwrap();
    }

    pub fn mover_cursor(&self, x: usize, y: usize) {
        print!("\x1b[{};{}H", y + 1, x + 1);
        io::stdout().flush().unwrap();
    }

    pub fn color_fondo(&self, r: u8, g: u8, b: u8) {
        print!("\x1b[48;2;{};{};{}m", r, g, b);
        io::stdout().flush().unwrap();
    }

    pub fn color_texto(&self, r: u8, g: u8, b: u8) {
        print!("\x1b[38;2;{};{};{}m", r, g, b);
        io::stdout().flush().unwrap();
    }

    pub fn resetear_colores(&self) {
        print!("\x1b[0m");
        io::stdout().flush().unwrap();
    }

    pub fn dibujar_texto(
        &self,
        x: usize,
        y: usize,
        texto: &str,
        color_r: u8,
        color_g: u8,
        color_b: u8,
    ) {
        self.mover_cursor(x, y);
        self.color_texto(color_r, color_g, color_b);
        print!("{}", texto);
        self.resetear_colores();
        io::stdout().flush().unwrap();
    }

    pub fn dibujar_borde(&self, x: usize, y: usize, ancho: usize, alto: usize) {
        if ancho < 2 || alto < 2 {
            return;
        }

        // Esquina superior izquierda
        self.mover_cursor(x, y);
        print!("┌");

        // Borde superior
        for _ in 0..ancho - 2 {
            print!("─");
        }

        // Esquina superior derecha
        println!("┐");

        // Lados izquierdo y derecho
        for fila in 1..alto - 1 {
            self.mover_cursor(x, y + fila);
            print!("│");
            for _ in 0..ancho - 2 {
                print!(" ");
            }
            println!("│");
        }

        // Esquina inferior izquierda
        self.mover_cursor(x, y + alto - 1);
        print!("└");

        // Borde inferior
        for _ in 0..ancho - 2 {
            print!("─");
        }

        // Esquina inferior derecha
        println!("┘");
        io::stdout().flush().unwrap();
    }
}
