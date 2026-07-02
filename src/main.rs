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

use std::io::{self, Write};
use std::env;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::thread;
use std::time::Duration;

use lexer::Lexer;
use parser::Parser;
use ast::Programa;
use evaluator::{configurar_entorno_global, evaluar_programa, Objeto};

mod parser;

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

    if matches!(resultado, Objeto::Error(_) | Objeto::Excepcion(_)) {
        println!("  {}: {}", ruta, resultado);
        false
    } else {
        true
    }
}

// ===========================================================================
// ejecutar_pruebas — Busca y ejecuta archivos *.test.argo, reporta resumen
// ===========================================================================
fn ejecutar_pruebas() {
    let mut total_pasaron = 0usize;
    let mut total_fallaron = 0usize;

    let dir = Path::new(".");
    let Ok(entries) = fs::read_dir(dir) else {
        println!("\x1b[91mError:\x1b[0m No se pudo leer el directorio actual.");
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
// ejecutar_install — Lee argo.toml, descarga dependencias, genera argo.mod/lock
// ===========================================================================
fn ejecutar_install() {
    let contenido = match fs::read_to_string("argo.toml") {
        Ok(c) => c,
        Err(_) => {
            println!("\x1b[91mError:\x1b[0m No se encontró 'argo.toml' en el directorio actual.");
            return;
        }
    };

    let mut dependencias = HashMap::new();
    let mut en_deps = false;

    for linea in contenido.lines() {
        let linea = linea.trim();
        if linea.starts_with("[dependencies]") {
            en_deps = true;
            continue;
        }
        if en_deps {
            if linea.starts_with('[') {
                break;
            }
            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }
            if let Some((clave, valor)) = linea.split_once('=') {
                let clave = clave.trim().to_string();
                let valor = valor.trim().trim_matches('"').to_string();
                if !clave.is_empty() && !valor.is_empty() {
                    dependencias.insert(clave, valor);
                }
            }
        }
    }

    if dependencias.is_empty() {
        println!("  No se encontraron dependencias en argo.toml.");
        return;
    }

    // Descargar cada dependencia
    let dir_cache = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
    ).join(".argo").join("cache");
    fs::create_dir_all(&dir_cache).ok();

    for (alias, url) in &dependencias {
        print!("  descargando {} de {} ... ", alias, url);
        io::stdout().flush().ok();

        match evaluator::modulo::fetch_url(url) {
            Ok(texto) => {
                // Guardar en proyecto local
                let dir_deps = Path::new("deps");
                fs::create_dir_all(dir_deps).ok();
                let ruta_destino = dir_deps.join(format!("{}.argo", alias));
                if let Err(e) = fs::write(&ruta_destino, &texto) {
                    println!("\x1b[91mError\x1b[0m al escribir {}: {}", ruta_destino.display(), e);
                    continue;
                }
                println!("\x1b[32mOK\x1b[0m");
            }
            Err(e) => {
                println!("\x1b[91mError\x1b[0m: {}", e);
            }
        }
    }

    // Generar/actualizar argo.mod (import map)
    {
        let mut lineas = String::new();
        for (alias, _url) in &dependencias {
            lineas.push_str(&format!("{} = {}.argo\n", alias, alias));
        }
        if let Err(e) = fs::write("argo.mod", &lineas) {
            println!("\x1b[91mError\x1b[0m al escribir argo.mod: {}", e);
        } else {
            println!("  argo.mod actualizado");
        }
    }

    // Generar argo.lock
    {
        let mut lineas = String::new();
        lineas.push_str("# argo.lock — generado automáticamente\n");
        for (alias, url) in &dependencias {
            let dir_deps = Path::new("deps");
            let ruta_destino = dir_deps.join(format!("{}.argo", alias));
            let checksum = fs::read(&ruta_destino)
                .ok()
                .map(|b| {
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    b.hash(&mut h);
                    format!("{:x}", h.finish())
                })
                .unwrap_or_else(|| "unknown".to_string());
            lineas.push_str(&format!("{} = {}#{}", alias, url, checksum));
            lineas.push('\n');
        }
        if let Err(e) = fs::write("argo.lock", &lineas) {
            println!("\x1b[91mError\x1b[0m al escribir argo.lock: {}", e);
        } else {
            println!("  argo.lock generado");
        }
    }

    println!("  \x1b[32m✔ Instalación completada.\x1b[0m");
}

// ===========================================================================
// main — Punto de entrada del binario
// ===========================================================================
// Enruta los comandos según el primer argumento:
//   init  → animación + generar_proyecto()
//   run   → ejecutar_proyecto()
//   repl  → iniciar_repl()
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
            println!("Argo v1.0.0 - Interprete Nativo");
            println!("Escribe 'exit' para salir.\n");
            iniciar_repl();
        }

        // Un argumento de usuario
        2 => match args[1].as_str() {
            "init" => {
                iniciar_animacion();
                generar_proyecto();
            }
            "run" => {
                ejecutar_proyecto();
            }
            "repl" => {
                println!("Argo v1.0.0 - Interprete Nativo");
                println!("Escribe 'exit' para salir.\n");
                iniciar_repl();
            }
            "test" => {
                ejecutar_pruebas();
            }
            "install" => {
                ejecutar_install();
            }
            // Si no es un comando reservado, tratar como ruta de archivo
            _ => {
                ejecutar_archivo(&args[1]);
            }
        },

        // Uso incorrecto
        _ => {
            println!("Uso: argo [comando|ruta]");
            println!();
            println!("  {b}{lg}Comandos:{r}");
            println!("    {lg}init{r}    Inicializar un nuevo proyecto Argo");
            println!("    {lg}run{r}     Ejecutar el proyecto actual");
            println!("    {lg}repl{r}    Iniciar el REPL interactivo");
            println!("    {lg}test{r}    Ejecutar pruebas (*.test.argo)");
            println!("    {lg}install{r} Instalar dependencias desde argo.toml");
            println!();
            println!("  {b}{lg}Tambien:{r}");
            println!("    {lg}argo <archivo.argo>{r}  Ejecutar un script directamente");
            std::process::exit(1);
        }
    }
}