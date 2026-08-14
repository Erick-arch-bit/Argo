#![allow(dead_code)]

use std::io::{self, Write, stdout};
use std::thread;
use std::time::Duration;

const BLUE: &str = "\x1b[38;5;39m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const DIM: &str = "\x1b[2m";
const ORANGE: &str = "\x1b[38;5;208m";
const CYAN: &str = "\x1b[38;5;117m";
const BOLD: &str = "\x1b[1m";
const R: &str = "\x1b[0m";
const CLR: &str = "\x1b[K";

const NODE_START: &str = "◇";
const PIPE: &str = "│";
const NODE_TASK: &str = "◼";
const NODE_END: &str = "└";
const PIPE_PAD: &str = "  ";

fn print_start(msg: &str) {
    println!("{}{} {}{}{}", BLUE, NODE_START, R, msg, R);
    flush();
}

fn print_pipe(_icon: &str, msg: &str, color: &str) {
    println!("{}{}{}{} {}{}{}{}", DIM, PIPE, R, PIPE_PAD, color, msg, R, CLR);
    flush();
}

fn print_subtask(msg: &str) {
    println!("{}{}{}{} {}{}{}{}", DIM, PIPE, R, PIPE_PAD, DIM, msg, R, CLR);
    flush();
}

fn print_end(msg: &str) {
    println!("{}{}{}{} {}{}✓ {}{}", GREEN, NODE_END, R, PIPE_PAD, GREEN, BOLD, msg, R);
    flush();
}

fn print_pipe_empty() {
    println!("{}{}{}", DIM, PIPE, R);
    flush();
}

fn print_colored(color: &str, msg: &str) {
    println!("{}{}{}{} {}{}{}{}", DIM, PIPE, R, PIPE_PAD, color, msg, R, CLR);
    flush();
}

fn flush() {
    let _ = stdout().flush();
}

fn spinner(msg: &str, frames: u32, delay_ms: u64) {
    let spin_chars: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

    for i in 0..frames {
        let ch = spin_chars[(i as usize) % spin_chars.len()];
        print!("\r{}{}{}{} {}{}{}{}", BLUE, ch, R, PIPE_PAD, DIM, msg, R, CLR);
        flush();
        thread::sleep(Duration::from_millis(delay_ms));
    }
    print!("\r{}\r", CLR);
    flush();
}

fn spinner_pipe(msg: &str, frames: u32, delay_ms: u64) {
    let spin_chars: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

    for i in 0..frames {
        let ch = spin_chars[(i as usize) % spin_chars.len()];
        print!("\r{}{}{}{}{}{}{} {}{}{}{}", DIM, PIPE, R, BLUE, ch, R, PIPE_PAD, DIM, msg, R, CLR);
        flush();
        thread::sleep(Duration::from_millis(delay_ms));
    }
    print!("\r{}\r", CLR);
    flush();
}

fn spinner_task(msg: &str, frames: u32, delay_ms: u64) {
    let spin_chars: [&str; 8] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧"];

    for i in 0..frames {
        let ch = spin_chars[(i as usize) % spin_chars.len()];
        print!("\r{}{}{}{} {}{}{} {}{}{} {}{}", DIM, PIPE, R, PIPE_PAD, BLUE, ch, R, PIPE_PAD, DIM, msg, R, CLR);
        flush();
        thread::sleep(Duration::from_millis(delay_ms));
    }
    print!("\r{}\r", CLR);
    flush();
}

fn print_progress(_msg: &str, percent: u32, width: usize) {
    let filled = (percent * width as u32 / 100) as usize;
    let empty = width - filled;

    let bar_filled: String = "█".repeat(filled);
    let bar_empty: String = "░".repeat(empty);

    let bar_color = if percent >= 100 { GREEN } else { BLUE };

    print!(
        "\r{}{}{}{} {}{}{}{} {}{}{}{}%",
        DIM, PIPE, R, PIPE_PAD,
        bar_color, bar_filled, R,
        DIM, bar_empty, R,
        BOLD, percent
    );
    flush();
}

fn install_argo() {
    println!();

    print_start("Iniciando instalación de Argo v2.2.0");

    spinner_pipe("Conectando a GitHub...", 12, 80);
    print_pipe(NODE_TASK, "Repositorio: Erick-arch-bit/Argo-Lang", DIM);

    spinner_task("Descargando argo-linux-amd64...", 15, 60);
    print_progress("Progreso de descarga", 100, 30);
    println!();

    spinner_task("Verificando integridad SHA-256...", 10, 70);
    print_pipe(NODE_TASK, "SHA-256: a1b2c3d4e5f6...✓", GREEN);

    spinner_task("Configurando PATH del sistema...", 8, 90);
    print_pipe(NODE_TASK, "~/.cargo/bin/argo → /usr/local/bin/argo", DIM);

    spinner_task("Estableciendo permisos de ejecución...", 6, 100);
    print_pipe(NODE_TASK, "chmod +x /usr/local/bin/argo", DIM);

    println!();

    print_end("Argo v2.2.0 instalado exitosamente");

    println!();
    print_colored(DIM, "Ejecuta `argo` para iniciar el REPL");
    print_colored(DIM, "Ejecuta `argo --help` para ver todos los comandos");

    println!();
}

fn update_compiler() {
    println!();

    print_start("Actualizando compilador de Argo");

    spinner_pipe("Verificando versión actual...", 8, 100);
    print_pipe(NODE_TASK, "Versión instalada: v1.5.3", ORANGE);

    spinner_pipe("Conectando a GitHub API...", 10, 80);
    print_pipe(NODE_TASK, "Última versión: v2.2.0", GREEN);

    spinner_task("Descargando argo-linux-amd64 (8.2 MB)...", 20, 50);
    print_progress("Descargando", 100, 30);
    println!();

    spinner_task("Verificando SHA-256...", 8, 90);
    print_pipe(NODE_TASK, "Checksum verificado ✓", GREEN);

    spinner_task("Creando backup del binario actual...", 6, 120);
    print_pipe(NODE_TASK, "Backup: /usr/local/bin/argo.bak", DIM);

    spinner_task("Reemplazando binario...", 5, 150);
    print_pipe(NODE_TASK, "/usr/local/bin/argo → v2.2.0", GREEN);

    println!();

    print_end("Compilador actualizado de v1.5.3 a v2.2.0");

    println!();
    print_colored(DIM, "Reinicia tu terminal para usar la nueva versión");

    println!();
}

fn init_repl() {
    println!();

    print_start("Iniciando Argo REPL v2.2.0");

    spinner_pipe("Cargando caché de módulos...", 10, 70);
    print_pipe(NODE_TASK, "~/.argo/cache/ → 12 módulos cargados", DIM);

    spinner_task("Cargando biblioteca estándar...", 12, 60);
    print_pipe(NODE_TASK, "math, str, arr, fs, os, net, json...", CYAN);
    print_pipe(NODE_TASK, "buffer, thread, time, gpu, ui", CYAN);

    spinner_task("Inicializando parser y AST...", 8, 80);
    print_pipe(NODE_TASK, "Parser LL(1) listo", GREEN);

    spinner_task("Configurando entorno global...", 6, 100);
    print_pipe(NODE_TASK, "print, len, tipo, assert, canal...", DIM);

    spinner_task("Cargando módulos FFI...", 5, 120);
    print_pipe(NODE_TASK, "gpu.crear_buffer, net.solicitud...", DIM);

    println!();

    print_end("REPL listo — Escribe tu código Argo");

    println!();

    print!("{}  {}{}{}Argo v2.2.0{} — Escribe '{}exit{}' para salir\n", BLUE, NODE_START, R, BOLD, R, CYAN, R);
    print!("{}{}{} ", BLUE, NODE_START, R);
    flush();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let trimmed = input.trim();
            if trimmed == "exit" || trimmed == "quit" {
                println!();
                print_colored(DIM, "¡Hasta luego! 🐙");
                println!();
            } else {
                println!();
                print_colored(GREEN, &format!("→ {}", trimmed));
                println!();
                print_colored(DIM, "(REPL interactivo no implementado en esta demo)");
                println!();
            }
        }
        Err(e) => {
            print_colored(RED, &format!("Error leyendo entrada: {}", e));
            println!();
        }
    }
}

fn print_menu() {
    println!();
    println!("{}{}  ◇ Argo-Lang CLI{} — Panel de control\n", BLUE, BOLD, R);

    println!("{}│{} Selecciona una opción:\n", DIM, R);

    println!("{}│{}   {}1{})  {}Instalar Argo{}           {}Descarga e instala el core del lenguaje{}", DIM, R, BLUE, R, BOLD, R, DIM, R);
    println!("{}│{}   {}2{})  {}Actualizar compilador{}  {}Descarga la última versión desde GitHub{}", DIM, R, BLUE, R, BOLD, R, DIM, R);
    println!("{}│{}   {}3{})  {}Iniciar REPL{}          {}Modo interactivo con caché de módulos{}", DIM, R, BLUE, R, BOLD, R, DIM, R);

    println!("{}│{}", DIM, R);
    println!("{}└{}  {}q{})  {}Salir{}\n", DIM, R, RED, R, BOLD, R);

    print!("{}  {}{}{}>Opción:{} ", BLUE, NODE_START, R, PIPE_PAD, R);
    flush();
}

pub fn ejecutar_demo() {
    print!("\x1b[2J\x1b[H");
    flush();

    println!("{}{}", BLUE, R);
    println!("{} █████╗ ██████╗  ██████╗  ██████╗     ██╗      █████╗ ██╗   ██╗ ██████╗ {}", BLUE, R);
    println!("{}██╔══██╗██╔══██╗██╔════╝ ██╔═══██╗    ██║     ██╔══██╗████╗  ██║██╔════╝ {}", BLUE, R);
    println!("{}███████║██████╔╝██║  ███╗██║   ██║    ██║     ███████║██╔██╗ ██║██║  ███╗ {}", BLUE, R);
    println!("{}██╔══██║██╔══██╗██║   ██║██║   ██║    ██║     ██╔══██║██║╚██╗██║██║   ██║ {}", BLUE, R);
    println!("{}██║  ██║██║  ██║╚██████╔╝╚██████╔╝    ███████╗██║  ██║██║ ╚████║╚██████╔╝ {}", BLUE, R);
    println!("{}╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝  ╚═════╝     ╚══════╝╚═╝  ╚═╝╚═╝  ╚═══╝ ╚═════╝  {}", BLUE, R);
    println!("{}{}{}ARGO LANG v2.2.0{} — Lenguaje de programación interpretado\n", DIM, PIPE_PAD, CYAN, R);

    loop {
        print_menu();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let trimmed = input.trim();
                match trimmed {
                    "1" => install_argo(),
                    "2" => update_compiler(),
                    "3" => init_repl(),
                    "q" | "Q" | "quit" | "exit" => {
                        println!();
                        print_colored(DIM, "¡Hasta luego! 🐙");
                        println!();
                        break;
                    }
                    _ => {
                        println!();
                        print_colored(RED, &format!("Opción no válida: '{}'", trimmed));
                        print_colored(DIM, "Opciones: 1, 2, 3, q");
                        println!();
                    }
                }
            }
            Err(e) => {
                print_colored(RED, &format!("Error de lectura: {}", e));
                break;
            }
        }
    }
}
