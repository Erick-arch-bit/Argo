// ---------------------------------------------------------------------------
// publish.rs — argo publish: empaqueta, firma y registra un paquete
// ---------------------------------------------------------------------------
// Cero dependencias externas. El flujo usa herramientas nativas del SO:
//   tar/zip    → empaquetar los archivos .argo del proyecto
//   sha256sum  → hash de integridad del paquete
//   openssl    → firma Ed25519 del paquete
// Al final actualiza el registro local (~/.argo/cache/registry.json) y
// muestra el JSON listo para enviar a la web de Argo.

use std::path::Path;
use std::process::Command;

use crate::paquetes::crypto::{calcular_hash, codificar_base64, firmar_archivo};
use crate::paquetes::{
    escapar_json, ruta_archivo_registro, ruta_carpeta_llaves, MetadatosPaquete, RegistroPaquetes,
};

pub struct PublicarOptions {
    pub verbose: bool,
    pub dry_run: bool,
}

/// Ejecuta `argo publish`.
pub fn ejecutar_publish(opts: PublicarOptions) -> Result<(), String> {
    // 0. Leer los metadatos del proyecto desde argo.toml
    let contenido = std::fs::read_to_string("argo.toml")
        .map_err(|_| "Error: No se encontró 'argo.toml' en el directorio actual.".to_string())?;
    let nombre = leer_campo_toml(&contenido, &["name", "nombre"])?;
    let version = leer_campo_toml(&contenido, &["version"])?;
    let autor = leer_campo_toml(&contenido, &["autor", "author"])
        .unwrap_or_else(|_| "desconocido".to_string());

    if opts.verbose {
        println!("  Empaquetando '{}' v{} (autor: {})...", nombre, version, autor);
    }

    // 1. Empaquetar los archivos .argo del proyecto
    let archivos = listar_archivos_argo()?;
    let nombre_paquete = format!("{}-{}.tar.gz", nombre, version);
    let archivo_paquete = crear_tarball(&archivos, &nombre_paquete)?;

    // 2. Hash SHA-256 del paquete (integridad)
    let hash = calcular_hash(&archivo_paquete)?;

    // 3. Leer la llave privada del desarrollador
    let ruta_privada = format!("{}/id_ed25519.pem", ruta_carpeta_llaves());
    if !Path::new(&ruta_privada).exists() {
        return Err(format!(
            "Error: No se encontró la llave privada en '{}'. Ejecuta 'argo login' primero.",
            ruta_privada
        ));
    }

    // 4. Firmar el paquete (autenticidad)
    let ruta_firma = "firma.bin";
    firmar_archivo(&archivo_paquete, &ruta_privada, ruta_firma)?;

    // 5. Leer la llave pública del desarrollador
    let ruta_publica = format!("{}/id_ed25519_pub.pem", ruta_carpeta_llaves());
    let llave_publica_pem = std::fs::read_to_string(&ruta_publica).map_err(|_| {
        format!(
            "Error: No se encontró la llave pública en '{}'. Ejecuta 'argo login' primero.",
            ruta_publica
        )
    })?;
    let llave_publica = pelar_pem(&llave_publica_pem);

    // 6. Codificar la firma binaria a Base64
    let firma_bytes = std::fs::read(ruta_firma)
        .map_err(|e| format!("Error: No se pudo leer la firma '{}': {}", ruta_firma, e))?;
    let firma_base64 = codificar_base64(&firma_bytes);

    // URL de descarga sugerida (release de GitHub)
    let url_descarga = match github_slug() {
        Some(slug) => format!(
            "https://github.com/{}/releases/download/v{}/{}",
            slug, version, nombre_paquete
        ),
        None => format!(
            "https://github.com/{}/releases/download/v{}/{}",
            autor, version, nombre_paquete
        ),
    };

    // 7. Construir los metadatos y el JSON de envío
    let metadatos = MetadatosPaquete {
        nombre: nombre.clone(),
        version: version.clone(),
        autor: autor.clone(),
        hash: hash.clone(),
        firma: firma_base64.clone(),
        llave_publica: llave_publica.clone(),
        url_descarga: url_descarga.clone(),
    };

    let json = format!(
        "{{\"paquetes\":{{{}}}}}",
        format!("{}:{}", escapar_json(&nombre), metadatos.a_json())
    );

    if opts.verbose || opts.dry_run {
        println!("\n  ----- JSON DE PUBLICACIÓN -----");
        println!("  {}", json);
        println!("  --------------------------------\n");
    }

    if opts.dry_run {
        println!("  Dry run: no se actualizó el registro ni se subió el paquete.");
        return Ok(());
    }

    // 8. Actualizar el registro local
    let mut registro = RegistroPaquetes::cargar();
    registro.agregar(metadatos);
    registro.guardar()?;

    println!(
        "  ✔ Paquete '{}' v{} firmado y registrado con éxito.",
        nombre, version
    );
    println!("    Hash SHA-256: {}", hash);
    println!("    Registro actualizado en: {}", ruta_archivo_registro());
    println!();
    println!("  Para publicarlo en la web, copia y pega este JSON en tu perfil de Argo:");
    println!("  {}", json);
    println!();

    Ok(())
}

// ===========================================================================
// Ayudantes
// ===========================================================================

/// Extrae el valor de una clave `clave = "valor"` de un argo.toml (mini-parser).
fn leer_campo_toml(contenido: &str, claves: &[&str]) -> Result<String, String> {
    for linea in contenido.lines() {
        let linea = linea.trim();
        if linea.starts_with('#') {
            continue;
        }
        let Some((clave, valor)) = linea.split_once('=') else {
            continue;
        };
        let clave = clave.trim();
        if !claves.contains(&clave) {
            continue;
        }
        let valor = valor.trim();
        if let Some(resto) = valor.strip_prefix('"') {
            if let Some(fin) = resto.find('"') {
                return Ok(resto[..fin].to_string());
            }
        }
        return Ok(valor.to_string());
    }
    Err(format!(
        "Error: No se encontró la clave '{}' en argo.toml.",
        claves[0]
    ))
}

/// Lista los archivos `.argo` del directorio actual.
fn listar_archivos_argo() -> Result<Vec<String>, String> {
    let mut archivos = Vec::new();
    let entradas = std::fs::read_dir(".")
        .map_err(|e| format!("Error: No se pudo leer el directorio actual: {}", e))?;

    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error: {}", e))?;
        if entrada.path().is_file() {
            if let Some(nombre) = entrada.file_name().to_str() {
                if nombre.ends_with(".argo") {
                    archivos.push(nombre.to_string());
                }
            }
        }
    }

    if archivos.is_empty() {
        return Err(
            "Error: No se encontraron archivos '.argo' en el directorio actual para empaquetar."
                .to_string(),
        );
    }

    archivos.sort();
    Ok(archivos)
}

/// Crea un tarball comprimido con los archivos indicados.
/// Si `tar` falla, intenta con `zip`; si ambos fallan, devuelve un error claro.
fn crear_tarball(archivos: &[String], nombre_archivo: &str) -> Result<String, String> {
    let _ = std::fs::remove_file(nombre_archivo);

    let mut tar = Command::new("tar");
    tar.args(["-czf", nombre_archivo]);
    for archivo in archivos {
        tar.arg(archivo);
    }

    match tar.output() {
        Ok(salida) if salida.status.success() => Ok(nombre_archivo.to_string()),
        Ok(salida) => {
            // tar falló → intentar con zip
            let nombre_zip = nombre_archivo.replace(".tar.gz", ".zip");
            let _ = std::fs::remove_file(&nombre_zip);

            let mut zip = Command::new("zip");
            zip.args(["-q", &nombre_zip]);
            for archivo in archivos {
                zip.arg(archivo);
            }

            match zip.output() {
                Ok(z) if z.status.success() => Ok(nombre_zip),
                Ok(z) => Err(format!(
                    "Error: No se pudo crear el paquete. tar: {} | zip: {}",
                    String::from_utf8_lossy(&salida.stderr).trim(),
                    String::from_utf8_lossy(&z.stderr).trim()
                )),
                Err(_) => Err(format!(
                    "Error: No se pudo crear el paquete. tar: {}",
                    String::from_utf8_lossy(&salida.stderr).trim()
                )),
            }
        }
        Err(_) => Err(
            "Error: No se pudo crear el paquete. ¿Está instalado 'tar' en el sistema?".to_string(),
        ),
    }
}

/// Quita la armadura PEM (-----BEGIN/END PUBLIC KEY-----) y deja solo el
/// cuerpo Base64, que es el formato que se guarda en el registro.
fn pelar_pem(pem: &str) -> String {
    pem.lines()
        .filter(|l| !l.trim_start().starts_with("-----BEGIN"))
        .filter(|l| !l.trim_start().starts_with("-----END"))
        .map(|l| l.trim())
        .collect()
}

/// Obtiene el slug `owner/repo` del remote 'origin' de GitHub.
fn github_slug() -> Option<String> {
    let salida = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !salida.status.success() {
        return None;
    }

    let remote = String::from_utf8_lossy(&salida.stdout).trim().to_string();
    let normalizado = remote
        .replace("git@github.com:", "github.com/")
        .replace("https://github.com/", "github.com/")
        .replace("http://github.com/", "github.com/")
        .trim_end_matches(".git")
        .to_string();

    let slug = normalizado.strip_prefix("github.com/")?.to_string();
    if slug.contains('/') {
        Some(slug)
    } else {
        None
    }
}
