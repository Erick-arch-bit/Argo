// Punto de entrada del intérprete Argo. Implementa:
//   argo init    → Animación de inicio + creación de proyecto
//   argo run     → Ejecuta el proyecto desde argo.toml
//   argo repl    → REPL interactivo
//   argo <file>  → Ejecuta un script .argo
//   argo         → REPL (por defecto)

mod lexer;
mod ast;
mod evaluator;
mod stdlib;
mod pkg;
mod cli_demo;

use std::io::{self, Write};
use std::env;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

use lexer::Lexer;
use parser::Parser;
use ast::Programa;
use evaluator::{configurar_entorno_global, evaluar_programa, Objeto};

mod parser;

// ===========================================================================
// mostrar_logo — ASCII art del pulpo de Argo
// ===========================================================================
fn mostrar_logo() {
    let lila  = "\x1b[38;5;99m";
    let naranja = "\x1b[38;5;208m";
    let cyan  = "\x1b[38;5;117m";
    let gris  = "\x1b[38;5;240m";
    let r     = "\x1b[0m";

    println!();
    println!("{}          ++++++++*{}", lila, r);
    println!("{}                                                                    ++++*#++++{}", lila, r);
    println!("{}                                                                     *++++++++*{}", lila, r);
    println!("{}                                                                 +++++++++++* *+++++++{}", cyan, r);
    println!("{}                                                                *+++++*  *++++++++++++++{}", cyan, r);
    println!("{}                                                                ** *+++++++++++++++++++++{}", cyan, r);
    println!("{}                                                               *++++++++++++++++++++++++++{}", cyan, r);
    println!("{}                                                             ++++++++++++*  #*++++++++++++*{}", cyan, r);
    println!("{}                                                            *+++++++  *++++++++++++++++++++++++*{}", cyan, r);
    println!("{}                                                           *+++*  ++++++++++++++++++++++++++++++++*{}", cyan, r);
    println!("{}                                                           #+* *++++++++++++++++     ++    *++++++++*{}", cyan, r);
    println!("{}                                                             +++++++++++++++* ++============++ +++++++{}", cyan, r);
    println!("{}                                                           +++++++++++++++**+==================+ ++++++{}", cyan, r);
    println!("{}                                                          +++++++++++++++ +======================+*++++{}", cyan, r);
    println!("{}                                                         +++++++++++++++ +========================+ +++{}", cyan, r);
    println!("{}                                                         ++++++++++++++ +==========================**++{}", cyan, r);
    println!("{}                                                        +++++++++++++++ +======================-==== *+{}", cyan, r);
    println!("{}                                                        +++++++++++++++ ========:   :+========   .==+{}", naranja, r);
    println!("{}                                                        +++++++++++++++ =======       -=======    ==+{}", naranja, r);
    println!("{}                                                        +++++++++++++++**=====-       -===-::==..-==+++#{}", naranja, r);
    println!("{}                                                         +++++++++++++++ +=====-.   .-====--========**++{}", naranja, r);
    println!("{}                                                         ++++++++++++++++ +=======================+**+++*{}", naranja, r);
    println!("{}                                                          ++++++++++++++++* +=====================+ ++++{}", naranja, r);
    println!("{}                                                           +++++++++++++++++* +++=============+++ +++++*{}", naranja, r);
    println!("{}                                                            ++++++++++++++++++++  ++++=+=+++=  *+++++++{}", naranja, r);
    println!("{}                                                             *+++++++++++++++++++++++++++++++++++++++{}", naranja, r);
    println!("{}                                                                ++++++++++++++++++++++++++++++++++*{}", naranja, r);
    println!("{}                                                                  *+++++++++++++++++++++++++++*     +++{}", naranja, r);
    println!("{}                                                                       *+++++++++++++++++*  ++++   +=+=+{}", gris, r);
    println!("{}                                                                           +*+ =+++++ ++++=====+   +====={}", gris, r);
    println!("{}                                                                  +++++   ++++ +====+ +++ +=====+++====={}", gris, r);
    println!("{}                                                                  ++++++++++++ +===== ++++ +==========+{}", gris, r);
    println!("{}                                                                    +++++++++ +====+**+++++  +++====++{}", gris, r);
    println!("{}                                                                        +*  +======  *+++++*   **{}", gris, r);
    println!("{}                                                                       +=========+    ++++++++++++*{}", gris, r);
    println!("{}                                                                       +=====+++        +++++++++*{}", gris, r);
    println!("{}                                                                         *                  *#{}", gris, r);
    println!();
}

// ===========================================================================
// iniciar_animacion — Pantalla de carga animada con el pulpo de Argo
// ===========================================================================
// Muestra una animación en bucle con el pulpo mascota, barra de progreso
// con gradiente y mensajes de estado cambiantes. Usa escapes ANSI para
// refrescar solo el bloque de 5 líneas sin redibujar toda la pantalla.
//
// # Flujo
//   1. Imprime la línea de comando y el badge [argo] (estáticos).
//   2. Entra en un bucle de 7 estados, cada uno con un mensaje y porcentaje.
//   3. En cada estado renderiza 5 líneas (caja + texto derecho) y las
//      refresca con escapes de cursor.
//   4. Termina mostrando "Secuencia completa" y restaura el cursor.
fn iniciar_animacion() {
    // ---- Códigos de escape ANSI ----
    let hide     = "\x1b[?25l";    // Oculta el cursor durante la animación
    let show     = "\x1b[?25h";    // Restaura el cursor al final
    let arriba5  = "\x1b[5A";      // Mueve el cursor 5 líneas arriba (área animada)
    let dim      = "\x1b[2m";      // Texto atenuado (comando superior)
    let lg       = "\x1b[38;5;250m"; // Gris claro (texto general)
    let b        = "\x1b[1m";      // Bold (negrita)
    let r        = "\x1b[0m";      // Reset
    let badge    = "\x1b[48;5;255m\x1b[30m"; // [argo] con fondo gris claro
    let box_c    = "\x1b[38;5;240m"; // Borde de la caja (gris medio)
    let head     = "\x1b[38;5;99m";  // Cabeza del pulpo (lila)
    let face     = "\x1b[38;5;208m"; // Rostro (naranja)

    // Paleta de la barra de progreso (gradiente rojo → naranja → amarillo → verde)
    let rojo    = "\x1b[38;5;196m";
    let naranja = "\x1b[38;5;214m";
    let amarillo= "\x1b[38;5;226m";
    let verde   = "\x1b[38;5;46m";
    let vacio   = "\x1b[38;5;237m";

    // ---- Cabecera estática (se imprime una sola vez) ----
    print!("{}", hide);
    println!("{dim}> argo init....{r}\n");
    println!(" {badge}[argo]{r} {lg}Initialization sequence online.{r}\n");

    // ---- Estados de la animación ----
    // Cada tupla: (mensaje, porcentaje).
    let estados: [(&str, u8); 7] = [
        ("Iniciando kernel...",       10),
        ("Cargando modulos...",       25),
        ("Analizando sistema...",     40),
        ("Compilando recursos...",    55),
        ("Desplegando subsistemas...", 70),
        ("Calibrando sensores...",    85),
        ("Secuencia completa!",      100),
    ];

    for (i, (mensaje, porcentaje)) in estados.iter().enumerate() {
        // ---- Construir la barra de progreso ----
        // 10 caracteres de ancho, cada bloque = 10%.
        let llenos = (*porcentaje / 10) as usize;
        let vacios = 10 - llenos;

        let mut barra = String::new();
        for j in 0..llenos {
            // Gradiente: rojo → naranja → amarillo → verde según posición
            let color = if j < 2 { rojo } else if j < 5 { naranja } else if j < 8 { amarillo } else { verde };
            barra.push_str(&format!("{color}█{r}"));
        }
        for _ in 0..vacios {
            barra.push_str(&format!("{vacio}░{r}"));
        }

        // ---- Alternar fotogramas del pulpo ----
        // Cada fotograma mide 5 líneas exactas (incluyendo bordes).
        // Esto permite refrescar con \x1b[5A sin solapamiento.
        let (par_ojo, par_tentaculo) = if i % 2 == 0 {
            ("(O) w (O)", "\\ ~~~ /")
        } else {
            ("(-) w (-)", "/ ~~~ \\")
        };

        let octo: [String; 5] = [
            format!("{box_c}╭──────────╮{r}"),
            format!("{box_c}│{r} {head}.-~~~-.{r}  {box_c}│{r}"),
            format!("{box_c}│{r} {face}{}{r} {box_c}│{r}", par_ojo),
            format!("{box_c}│{r} {face}{}{r} {box_c}│{r}", par_tentaculo),
            format!("{box_c}╰──────────╯{r}"),
        ];

        // Texto lateral (3 líneas alineadas a la derecha de la caja)
        let lado: [String; 3] = [
            format!("  {b}Houston:{r}"),
            format!("  {lg}{}{r}", mensaje),
            format!("  [{}{}{r}] {:2}%", barra, r, porcentaje),
        ];

        // ---- Renderizar las 5 líneas del fotograma ----
        for fila in 0..5 {
            if fila < 3 {
                println!("    {}{}", octo[fila], lado[fila]);
            } else {
                println!("    {}", octo[fila]);
            }
        }

        io::stdout().flush().ok();

        // ---- Pausa entre fotogramas ----
        if *porcentaje < 100 {
            thread::sleep(Duration::from_millis(400));
        } else {
            thread::sleep(Duration::from_millis(600));
        }

        // ---- Volver al inicio del bloque animado (excepto en el último frame) ----
        if i < estados.len() - 1 {
            print!("{}", arriba5);
        }
    }

    print!("{}", show);
    println!(); // Salto final después de la animación
}

// ===========================================================================
// generar_proyecto — Crea la estructura de directorios y archivos de un
//                    proyecto Argo en el disco.
// ===========================================================================
// 1. Pide al usuario el nombre del proyecto.
// 2. Crea el directorio raíz y src/.
// 3. Escribe argo.toml con los metadatos del proyecto.
// 4. Escribe src/main.argo con un programa de prueba.
//
// # Manejo de errores
// Cada operación de E/S se maneja con match para evitar panic!.
// Los errores se notifican al usuario con un mensaje descriptivo
// y la función retorna sin abortar el proceso.
fn generar_proyecto() {
    // ---- 1. Solicitar nombre del proyecto ----
    let lg_ = "\x1b[38;5;250m";
    let b_  = "\x1b[1m";
    let r_  = "\x1b[0m";
    print!("\n  {lg_}[?]{r_} {b_}Nombre del proyecto:{r_} ");
    io::stdout().flush().ok();

    let mut nombre = String::new();
    if io::stdin().read_line(&mut nombre).is_err() {
        println!("\n  Error: No se pudo leer el nombre del proyecto.");
        return;
    }
    let nombre = nombre.trim().to_string();

    if nombre.is_empty() {
        println!("  Error: El nombre del proyecto no puede estar vacío.");
        return;
    }

    // ---- 2. Crear directorios ----
    // create_dir_all crea toda la jerarquía: "mi-proyecto/src/".
    if let Err(e) = fs::create_dir_all(&nombre) {
        println!("  Error: No se pudo crear el directorio '{}': {}", nombre, e);
        return;
    }
    if let Err(e) = fs::create_dir_all(format!("{}/src", nombre)) {
        println!("  Error: No se pudo crear '{}': {}", nombre, e);
        return;
    }

    // ---- 3. Escribir argo.toml ----
    let toml = format!(
        "[proyecto]\n\
         nombre = \"{0}\"\n\
         version = \"1.0.0\"\n\
         autor = \"Desarrollador\"\n\
         entrada = \"src/main.argo\"\n",
        nombre
    );
    if let Err(e) = fs::write(format!("{}/argo.toml", nombre), &toml) {
        println!("  Error: No se pudo escribir argo.toml: {}", e);
        return;
    }

    // ---- 4. Escribir src/main.argo ----
    let codigo = "print(\"Secuencia de despegue exitosa! Hola desde Argo.\");\n";
    if let Err(e) = fs::write(format!("{}/src/main.argo", nombre), codigo) {
        println!("  Error: No se pudo escribir src/main.argo: {}", e);
        return;
    }

    let verde = "\x1b[38;5;46m";
    println!();
    println!("  {verde}✔{r_} Proyecto '{nombre}' creado exitosamente.");
    println!("  {verde}✔{r_}   ├── argo.toml");
    println!("  {verde}✔{r_}   └── src/main.argo");
    println!();
    println!("  Usa '{b_}argo run{r_}' dentro del directorio para ejecutarlo.");
}

// ===========================================================================
// ejecutar_proyecto — Lee argo.toml, extrae la ruta de entrada y ejecuta el
//                     script principal del proyecto.
// ===========================================================================
// 1. Intenta leer ./argo.toml del directorio actual.
// 2. Busca la línea que comienza con 'entrada = ' (mini-parser TOML sin
//    dependencias externas).
// 3. Extrae el valor entre comillas dobles.
// 4. Delega en ejecutar_archivo() para correr el script.
//
// # Mini-parser TOML
// Dado que no se permite usar crates externos (TOML, serde, etc.),
// implementamos un parser de una sola línea: busca el prefijo textual
// 'entrada = ' y extrae el contenido entre las primeras comillas dobles
// después del signo '='. Esto es suficiente para nuestro formato simple.
fn ejecutar_proyecto() {
    const RUTA_MANIFIESTO: &str = "./argo.toml";

    // ---- 1. Leer argo.toml ----
    let contenido = match fs::read_to_string(RUTA_MANIFIESTO) {
        Ok(c) => c,
        Err(_) => {
            println!("\x1b[91mError:\x1b[0m No se encontró '{}' en el directorio actual.", RUTA_MANIFIESTO);
            println!("  Este comando debe ejecutarse dentro de un proyecto Argo válido.");
            return;
        }
    };

    // ---- 2. Buscar la línea 'entrada = ' ----
    let ruta_entrada: String = {
        let mut ruta = String::new();
        let mut encontrado = false;

        for linea in contenido.lines() {
            let linea = linea.trim();
            // Buscar el prefijo "entrada = " (puede tener espacios alrededor)
            if let Some(pos) = linea.find("entrada = ") {
                let despues_igual = &linea[pos + "entrada = ".len()..];
                // Extraer el valor entre comillas dobles: "src/main.argo"
                if let Some(ini) = despues_igual.find('"') {
                    let resto = &despues_igual[ini + 1..];
                    if let Some(fin) = resto.find('"') {
                        ruta = resto[..fin].to_string();
                        encontrado = true;
                        break;
                    }
                }
            }
        }

        if !encontrado {
            println!("\x1b[91mError:\x1b[0m No se encontró la clave 'entrada' en argo.toml.");
            println!("  Asegúrate de que el archivo contenga una línea como:");
            println!("  entrada = \"src/main.argo\"");
            return;
        }

        ruta
    };

    // ---- 3. Ejecutar el archivo de entrada ----
    // La ruta en argo.toml es relativa al directorio del proyecto.
    println!("\x1b[2mEjecutando {}...\x1b[0m", ruta_entrada);
    ejecutar_archivo(&ruta_entrada);
}

// ===========================================================================
// ejecutar_archivo — Lee, parsea y evalúa un script .argo desde el
// sistema de archivos.
// ===========================================================================
fn ejecutar_archivo(ruta: &str) {
    ejecutar_desde_str(ruta, &match fs::read_to_string(ruta) {
        Ok(c) => c,
        Err(_) => {
            println!("\x1b[91mError:\x1b[0m No se pudo leer el archivo '{}'", ruta);
            return;
        }
    });
}

fn ejecutar_desde_str(ruta: &str, contenido: &str) -> bool {
    let mut entorno = configurar_entorno_global();
    let lexer = Lexer::nuevo(contenido);
    let mut parser = Parser::nuevo(lexer);
    let programa: Programa = parser.parsear_programa();

    if !parser.errores.is_empty() {
        for error in &parser.errores {
            println!("\x1b[91merror de sintaxis\x1b[0m: {}", error);
        }
        return false;
    }

    let resultado = evaluar_programa(&programa, &mut entorno);

    if matches!(resultado, Objeto::Error(_, _) | Objeto::Excepcion(_)) {
        println!("  {}: {}", ruta, resultado);
        false
    } else {
        true
    }
}

// ===========================================================================
// ejecutar_pruebas — Busca y ejecuta archivos *.test.argo en tests/, reporta resumen
// ===========================================================================
fn ejecutar_pruebas() {
    let mut total_pasaron = 0usize;
    let mut total_fallaron = 0usize;

    let dir = Path::new("tests");
    let Ok(entries) = fs::read_dir(dir) else {
        println!("\x1b[91mError:\x1b[0m No se pudo leer el directorio 'tests'.");
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("argo") {
            continue;
        }
        let nombre = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !nombre.contains(".test.") {
            continue;
        }

        let contenido = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        print!("  probando {} ... ", nombre);
        io::stdout().flush().ok();

        let pasaron = ejecutar_desde_str(nombre, &contenido);
        if pasaron {
            println!("\x1b[32mOK\x1b[0m");
            total_pasaron += 1;
        } else {
            total_fallaron += 1;
        }
    }

    let total = total_pasaron + total_fallaron;
    println!();
    if total == 0 {
        println!("  No se encontraron archivos *.test.argo");
    } else {
        println!(
            "  Resultado: {} pasaron, {} fallaron, {} total",
            total_pasaron, total_fallaron, total
        );
    }
}

// ===========================================================================
// iniciar_repl — Bucle interactivo de lectura, evaluación e impresión
// ===========================================================================
fn iniciar_repl() {
    let mut entorno_global = configurar_entorno_global();

    loop {
        print!("argo>> ");
        if io::stdout().flush().is_err() {
            break;
        }

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {
                let input = input.trim().to_string();
                if input == "exit" {
                    break;
                }
                if input.is_empty() {
                    continue;
                }

                let lexer = Lexer::nuevo(&input);
                let mut parser = Parser::nuevo(lexer);
                let programa: Programa = parser.parsear_programa();

                if !parser.errores.is_empty() {
                    for error in &parser.errores {
                        println!("\x1b[91merror de sintaxis\x1b[0m: {}", error);
                    }
                    continue;
                }

                let resultado = evaluar_programa(&programa, &mut entorno_global);

                if !matches!(resultado, Objeto::Nulo) {
                    println!("{}", resultado);
                }
            }
            Err(_) => break,
        }
    }
}

// ===========================================================================
// ejecutar_install_v2 — Instalación de dependencias v2.0
// ===========================================================================
fn ejecutar_install_v2() {
    let output = pkg::output::Output::new();
    let auth = pkg::multihost::AuthTokens::cargar();

    // Migrar caché antiguo si existe
    pkg::cache::migrar_cache_antiguo().ok();

    // Verificar argo.lock existente
    let lock_path = std::path::Path::new("argo.lock");
    if lock_path.exists() {
        output.dim("  Usando argo.lock existente...");
        println!();
        // TODO: usar lockfile existente
    }

    output.bold("  Resolviendo dependencias...\n");

    match pkg::resolve::resolver_desde_proyecto(&output, &auth, false) {
        Ok(paquetes) => {
            if paquetes.is_empty() {
                output.linea("  No se encontraron dependencias en argo.toml.");
                return;
            }

            output.bold("  Descargando paquetes...\n");
            if let Err(e) = pkg::resolve::instalar_paquetes(&paquetes, &output) {
                output.error(&e);
                std::process::exit(1);
            }

            // Generar argo.lock
            let lock = pkg::resolve::generar_lockfile(&paquetes);
            lock.guardar(lock_path).ok();

            // Generar argo.mod
            let mut argo_mod = String::new();
            for pkg in &paquetes {
                let alias = pkg.repo.repo.clone();
                argo_mod.push_str(&format!("{} = vendor/{}/argo.toml\n", alias, pkg.repo.full_name));
            }
            std::fs::write("argo.mod", &argo_mod).ok();

            output.verde("✅ Instalación completada.");
            println!();
        }
        Err(e) => {
            output.error(&e);
            std::process::exit(1);
        }
    }
}

// ===========================================================================
// ejecutar_install_repo — Instalar un paquete específico
// ===========================================================================
fn ejecutar_install_repo(repo: &str) {
    let output = pkg::output::Output::new();
    let auth = pkg::multihost::AuthTokens::cargar();

    pkg::cache::migrar_cache_antiguo().ok();

    let info = match pkg::multihost::RepoInfo::parsear(repo) {
        Ok(i) => i,
        Err(e) => {
            output.error(&e);
            std::process::exit(1);
        }
    };

    output.bold(&format!("  Descargando {}...\n", info.full_name));

    // Resolver solo este paquete
    let estado = pkg::resolve::resolver_desde_proyecto(&output, &auth, false);
    match estado {
        Ok(paquetes) => {
            // Agregar este paquete si no está
            if !paquetes.iter().any(|p| p.repo.full_name == info.full_name) {
                // Resolver manualmente
                output.error(&format!("Paquete '{}' no encontrado en argo.toml.", repo));
                output.linea("  Agrega la dependencia a argo.toml primero:");
                output.linea(&format!("    [dependencies]\n    \"{}\" = \"*\"", info.full_name));
            } else {
                if let Err(e) = pkg::resolve::instalar_paquetes(&paquetes, &output) {
                    output.error(&e);
                    std::process::exit(1);
                }
                output.verde("✅ Instalación completada.");
                println!();
            }
        }
        Err(e) => {
            output.error(&e);
            std::process::exit(1);
        }
    }
}

// ===========================================================================
// ejecutar_list — Listar paquetes instalados
// ===========================================================================
fn ejecutar_list() {
    let output = pkg::output::Output::new();
    pkg::cache::migrar_cache_antiguo().ok();

    match pkg::cache::CacheIndex::cargar() {
        Ok(index) => {
            if index.entries.is_empty() {
                output.linea("  No hay paquetes instalados.");
                return;
            }
            output.bold("  Paquetes instalados:\n");
            for entry in &index.entries {
                let size_kb = entry.size_bytes as f64 / 1024.0;
                output.linea(&format!(
                    "    {} {} ({:.1} KB, {} archivos)",
                    entry.name, entry.version, size_kb, entry.file_count
                ));
            }
        }
        Err(e) => {
            output.error(&e);
        }
    }
}

// ===========================================================================
// ejecutar_clean — Limpiar caché
// ===========================================================================
fn ejecutar_clean(all: bool) {
    let output = pkg::output::Output::new();
    pkg::cache::migrar_cache_antiguo().ok();

    match pkg::cache::CacheIndex::cargar() {
        Ok(mut index) => {
            if all {
                index.entries.clear();
                output.verde("  Caché limpiada completamente.");
            } else {
                let now = pkg::cache::ahora_secs();
                let antes = index.entries.len();
                index.entries.retain(|e| {
                    let dias = (now - e.last_used) / 86400;
                    dias < 30
                });
                let eliminados = antes - index.entries.len();
                output.verde(&format!("  {} paquetes eliminados del caché.", eliminados));
            }
            index.guardar().ok();
        }
        Err(e) => {
            output.error(&e);
        }
    }
}

// ===========================================================================
// ejecutar_update_v2 — Actualizar dependencias
// ===========================================================================
fn ejecutar_update_v2(paquete: Option<&str>) {
    let output = pkg::output::Output::new();

    if let Some(nombre) = paquete {
        output.bold(&format!("  Actualizando {}...\n", nombre));
    } else {
        output.bold("  Actualizando todas las dependencias...\n");
    }

    // Eliminar argo.lock para forzar resolución
    let lock_path = std::path::Path::new("argo.lock");
    if lock_path.exists() {
        std::fs::remove_file(lock_path).ok();
    }

    ejecutar_install_v2();
}

// ===========================================================================
// ejecutar_uninstall — Desinstalar un paquete
// ===========================================================================
fn ejecutar_uninstall(repo: &str) {
    let output = pkg::output::Output::new();

    match pkg::cache::CacheIndex::cargar() {
        Ok(mut index) => {
            if index.buscar(repo).is_none() {
                output.error(&format!("Paquete '{}' no encontrado.", repo));
                return;
            }
            index.eliminar(repo);
            index.guardar().ok();

            // Eliminar directorio vendor
            let vendor_path = std::path::PathBuf::from("vendor").join(repo);
            if vendor_path.exists() {
                std::fs::remove_dir_all(&vendor_path).ok();
            }

            output.verde(&format!("✅ Paquete '{}' desinstalado.", repo));
            println!();
        }
        Err(e) => {
            output.error(&e);
        }
    }
}

// ===========================================================================
// ejecutar_config_set — Configurar un valor
// ===========================================================================
fn ejecutar_config_set(key: &str, value: &str) {
    let output = pkg::output::Output::new();
    let mut auth = pkg::multihost::AuthTokens::cargar();

    let host = match key {
        "github-token" => "github",
        "gitlab-token" => "gitlab",
        "bitbucket-token" => "bitbucket",
        _ => {
            output.error(&format!("Clave desconocida '{}'. Claves válidas: github-token, gitlab-token, bitbucket-token", key));
            std::process::exit(1);
        }
    };

    auth.set_token(host, value);
    match auth.guardar() {
        Ok(()) => {
            output.verde(&format!("✅ Token '{}' configurado.", key));
            println!();
        }
        Err(e) => {
            output.error(&e);
            std::process::exit(1);
        }
    }
}

// ===========================================================================
// main — Punto de entrada del binario
// ===========================================================================
// Enruta los comandos según el primer argumento:
//   init  → animación + generar_proyecto()
//   run   → ejecutar_proyecto()
//   repl  → iniciar_repl()
//   test  → ejecutar_pruebas()
//   cli   → cli_demo::ejecutar_demo()
//   <file> → ejecutar_archivo(<file>)
//   (ninguno) → iniciar_repl()
fn main() {
    let lg = "\x1b[38;5;250m";
    let b  = "\x1b[1m";
    let r  = "\x1b[0m";

    let args: Vec<String> = env::args().collect();

    match args.len() {
        // Sin argumentos → REPL interactivo
        1 => {
            mostrar_logo();
            println!("Argo v2.1.0 - Interprete Nativo");
            println!("Escribe 'exit' para salir.\n");
            iniciar_repl();
        }

        // Un argumento de usuario
        2 => match args[1].as_str() {
            "--version" | "-v" => {
                mostrar_logo();
                println!("argo 2.1.0");
            }
            "init" => {
                mostrar_logo();
                iniciar_animacion();
                generar_proyecto();
            }
            "run" => {
                ejecutar_proyecto();
            }
            "repl" => {
                mostrar_logo();
                println!("Argo v2.1.0 - Interprete Nativo");
                println!("Escribe 'exit' para salir.\n");
                iniciar_repl();
            }
            "test" => {
                ejecutar_pruebas();
            }
            "cli" => {
                cli_demo::ejecutar_demo();
            }
            "install" => {
                ejecutar_install_v2();
            }
            "publish" => {
                let opts = pkg::publish::PublicarOptions {
                    verbose: false,
                    dry_run: false,
                };
                if let Err(e) = pkg::publish::ejecutar_publish(opts) {
                    eprintln!("\x1b[31mError:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            "list" => {
                ejecutar_list();
            }
            "clean" => {
                ejecutar_clean(false);
            }
            "update" => {
                ejecutar_update_v2(None);
            }
            "config" => {
                eprintln!("\x1b[31mError:\x1b[0m Uso: argo config set <key> <value>");
                std::process::exit(1);
            }
            "uninstall" => {
                eprintln!("\x1b[31mError:\x1b[0m Uso: argo uninstall <repo>");
                std::process::exit(1);
            }
            // Si no es un comando reservado, tratar como ruta de archivo
            _ => {
                ejecutar_archivo(&args[1]);
            }
        },

        3 => match args[1].as_str() {
            "install" => {
                ejecutar_install_repo(&args[2]);
            }
            "update" => {
                ejecutar_update_v2(Some(&args[2]));
            }
            "uninstall" => {
                ejecutar_uninstall(&args[2]);
            }
            "config" => {
                eprintln!("\x1b[31mError:\x1b[0m Uso: argo config set <key> <value>");
                std::process::exit(1);
            }
            "publish" => {
                let flags: Vec<&str> = args[2].split_whitespace().collect();
                let verbose = flags.iter().any(|f| *f == "--verbose" || *f == "-v");
                let dry_run = flags.contains(&"--dry-run");
                let opts = pkg::publish::PublicarOptions { verbose, dry_run };
                if let Err(e) = pkg::publish::ejecutar_publish(opts) {
                    eprintln!("\x1b[31mError:\x1b[0m {}", e);
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("\x1b[31mError:\x1b[0m Comando desconocido: {} {}", args[1], args[2]);
                std::process::exit(1);
            }
        },

        // 4+ argumentos
        _ => match args[1].as_str() {
            "config" => {
                if args.len() >= 4 && args[2] == "set" {
                    ejecutar_config_set(&args[3], &args[4..].join(" "));
                } else {
                    eprintln!("\x1b[31mError:\x1b[0m Uso: argo config set <key> <value>");
                    std::process::exit(1);
                }
            }
            "install" => {
                ejecutar_install_repo(&args[2]);
            }
            _ => {
                println!("Uso: argo [comando|ruta]");
                println!();
                println!("  {b}{lg}Comandos:{r}");
                println!("    {lg}init{r}             Inicializar un nuevo proyecto Argo");
                println!("    {lg}run{r}              Ejecutar el proyecto actual");
                println!("    {lg}repl{r}             Iniciar el REPL interactivo");
                println!("    {lg}test{r}             Ejecutar pruebas (*.test.argo)");
                println!("    {lg}cli{r}              Demo CLI interactiva con animaciones");
                println!("    {lg}install{r}          Instalar dependencias desde argo.toml");
                println!("    {lg}install <repo>{r}   Instalar un paquete desde GitHub");
                println!("    {lg}uninstall <repo>{r} Desinstalar un paquete");
                println!("    {lg}update{r}           Actualizar todas las dependencias");
                println!("    {lg}update <repo>{r}    Actualizar un paquete específico");
                println!("    {lg}list{r}             Listar paquetes instalados");
                println!("    {lg}clean{r}            Limpiar caché de paquetes");
                println!("    {lg}publish{r}          Publicar paquete en GitHub");
                println!("    {lg}config set{r}       Configurar tokens de autenticación");
                println!();
                println!("  {b}{lg}Tambien:{r}");
                println!("    {lg}argo <archivo.argo>{r}  Ejecutar un script directamente");
                std::process::exit(1);
            }
        }
    }
}