//
// Sistema descentralizado de resolución de módulos para Argo.
// Permite importar código mediante URLs (con caché local de dos
// niveles: .argo textual + .argbc binario) y rutas de archivo
// tradicionales (también con caché .argbc local).
//
// En cache hit (.argbc) se deserializa el AST directamente, sin
// lexer ni parser — cero parsing en la segunda ejecución.
//
// Restricción: cero dependencias externas. Se usa curl (Unix)
// como subproceso para fetching HTTP.
//

use std::collections::{hash_map::DefaultHasher, HashMap};
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;

use crate::ast::Statement;
use crate::evaluator::bytecode;
use crate::lexer::Lexer;
use crate::parser::Parser;

// ---------------------------------------------------------------------------
// fetch_url — Descarga segura vía curl
// ---------------------------------------------------------------------------
fn fetch_url(url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .arg("-sSL")
        .arg(url)
        .output()
        .map_err(|e| format!("No se pudo ejecutar curl: {}", e))?;

    if output.status.success() {
        let body = String::from_utf8(output.stdout)
            .map_err(|e| format!("La respuesta no es UTF-8 válido: {}", e))?;
        if body.is_empty() {
            return Err(format!("El servidor retornó contenido vacío para: {}", url));
        }
        Ok(body)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "Error al descargar {} (código {}): {}",
            url,
            output.status.code().unwrap_or(-1),
            stderr.trim(),
        ))
    }
}

// ---------------------------------------------------------------------------
// obtener_dir_cache — Directorio base de la caché
// ---------------------------------------------------------------------------
fn obtener_dir_cache() -> PathBuf {
    let home = if cfg!(target_os = "windows") {
        env::var("USERPROFILE")
            .unwrap_or_else(|_| "C:\\Users\\Default".to_string())
    } else {
        env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
    };
    PathBuf::from(home).join(".argo").join("cache")
}

// ---------------------------------------------------------------------------
// hash_url — Hex hash de una URL para nombre de archivo en caché
// ---------------------------------------------------------------------------
fn hash_url(url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// ---------------------------------------------------------------------------
// parsear_texto — Convierte código fuente en AST (Vec<Statement>)
// ---------------------------------------------------------------------------
fn parsear_texto(codigo: &str, origen: &str) -> Result<Vec<Statement>, String> {
    let lexer = Lexer::nuevo(codigo);
    let mut parser = Parser::nuevo(lexer);
    let programa = parser.parsear_programa();

    if !parser.errores.is_empty() {
        let mut msg = format!("Error sintáctico al importar '{}'", origen);
        for err in &parser.errores {
            msg.push_str("\n  ");
            msg.push_str(err);
        }
        return Err(msg);
    }

    Ok(programa.sentencias)
}

// ---------------------------------------------------------------------------
// guardar_argbc — Serializa un AST a disco en formato .argbc
// ---------------------------------------------------------------------------
fn guardar_argbc(ruta: &std::path::Path, ast: &[Statement]) -> Result<(), String> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&(ast.len() as u32).to_le_bytes());
    for stmt in ast {
        buffer.extend_from_slice(&bytecode::sentencia_a_bytes(stmt));
    }
    fs::write(ruta, &buffer).map_err(|e| {
        format!(
            "No se pudo escribir la caché binaria en '{}': {}",
            ruta.display(),
            e
        )
    })
}

// ---------------------------------------------------------------------------
// cargar_argbc — Deserializa un AST desde un archivo .argbc
// ---------------------------------------------------------------------------
fn cargar_argbc(ruta: &std::path::Path) -> Result<Vec<Statement>, String> {
    let buffer =
        fs::read(ruta).map_err(|e| format!("Error al leer '{}': {}", ruta.display(), e))?;

    if buffer.len() < 4 {
        return Err(format!(
            "Archivo .argbc corrupto: '{}' (menos de 4 bytes)",
            ruta.display()
        ));
    }

    let count_arr: [u8; 4] = buffer[..4]
        .try_into()
        .map_err(|_| "bytecode: error al leer cantidad de sentencias".to_string())?;
    let count = u32::from_le_bytes(count_arr) as usize;

    let mut cursor = 4;
    let mut sentencias = Vec::with_capacity(count);
    for _ in 0..count {
        sentencias.push(bytecode::bytes_a_sentencia(&buffer, &mut cursor)?);
    }

    Ok(sentencias)
}

// ---------------------------------------------------------------------------
// leer_argo_mod — Parsea el archivo de manifiesto `argo.mod`
// ---------------------------------------------------------------------------
// Formato: clave = valor (una por línea, `#` para comentarios).
// Retorna un HashMap<String, String> vacío si no existe o hay error.
fn leer_argo_mod() -> HashMap<String, String> {
    let ruta = match env::current_dir() {
        Ok(d) => d.join("argo.mod"),
        Err(_) => return HashMap::new(),
    };

    let contenido = match fs::read_to_string(&ruta) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };

    let mut mapa = HashMap::new();

    for linea in contenido.lines() {
        let linea = linea.trim();
        if linea.is_empty() || linea.starts_with('#') {
            continue;
        }

        match linea.split_once('=') {
            Some((clave, valor)) => {
                let clave = clave.trim().to_string();
                let valor = valor.trim().to_string();
                if !clave.is_empty() && !valor.is_empty() {
                    mapa.insert(clave, valor);
                }
            }
            None => {
                eprintln!(
                    "⚠ argo.mod: línea ignorada (formato esperado: clave = valor): {}",
                    linea
                );
            }
        }
    }

    mapa
}

// ---------------------------------------------------------------------------
// importar — Punto de entrada para la resolución y compilación de módulos
// ---------------------------------------------------------------------------
// Retorna el AST listo para el evaluador (Vec<Statement>).
//
// Para URLs (http:// o https://):
//   1. Busca ~/.argo/cache/<hash>.argbc → cache hit (deserializa, retorna)
//   2. Busca ~/.argo/cache/<hash>.argo   → cache textual (parsea, guarda
//      .argbc, retorna)
//   3. Descarga con curl → guarda .argo, parsea, guarda .argbc, retorna
//
// Para rutas locales:
//   1. Busca <ruta>.argbc → cache hit (deserializa, retorna)
//   2. Lee <ruta> textual  → parsea, guarda .argbc junto al .argo, retorna
pub fn importar(ruta_o_url: &str) -> Result<Vec<Statement>, String> {
    let ruta_base = ruta_o_url.trim();

    // Resolver alias via import map (argo.mod)
    let mapa_mod = leer_argo_mod();
    let ruta = if let Some(resuelto) = mapa_mod.get(ruta_base) {
        resuelto.as_str()
    } else {
        ruta_base
    };

    if ruta.starts_with("http://") || ruta.starts_with("https://") {
        let dir_cache = obtener_dir_cache();
        let hash = hash_url(ruta);
        let ruta_argbc = dir_cache.join(format!("{}.argbc", hash));
        let ruta_argo = dir_cache.join(format!("{}.argo", hash));

        // Nivel 1: cache hit binario (.argbc)
        if ruta_argbc.exists() {
            return cargar_argbc(&ruta_argbc);
        }

        // Asegurar que el directorio de caché existe
        fs::create_dir_all(&dir_cache).map_err(|e| {
            format!(
                "No se pudo crear el directorio de caché '{}': {}",
                dir_cache.display(),
                e
            )
        })?;

        // Nivel 2: cache textual (.argo) o descarga
        let codigo = if ruta_argo.exists() {
            fs::read_to_string(&ruta_argo)
                .map_err(|e| format!("Error al leer caché '{}': {}", ruta_argo.display(), e))?
        } else {
            let texto = fetch_url(ruta)?;
            fs::write(&ruta_argo, &texto).map_err(|e| {
                format!(
                    "No se pudo escribir la caché en '{}': {}",
                    ruta_argo.display(),
                    e
                )
            })?;
            texto
        };

        let ast = parsear_texto(&codigo, ruta)?;
        guardar_argbc(&ruta_argbc, &ast)?;
        Ok(ast)
    } else {
        // Ruta local — buscar .argbc junto al archivo fuente
        let ruta_argbc = {
            let s = format!("{}.argbc", ruta);
            PathBuf::from(s)
        };

        if ruta_argbc.exists() {
            return cargar_argbc(&ruta_argbc);
        }

        let codigo = fs::read_to_string(ruta).map_err(|e| {
            format!("No se pudo leer el archivo '{}': {}", ruta, e)
        })?;

        let ast = parsear_texto(&codigo, ruta)?;
        let _ = guardar_argbc(&ruta_argbc, &ast); // best-effort
        Ok(ast)
    }
}
