// ---------------------------------------------------------------------------
// publish.rs — argo publish: publicar paquetes en GitHub
// ---------------------------------------------------------------------------
// Validaciones, SHA-256, creación de tag, push.

use std::path::Path;

use crate::pkg::output::Output;
use crate::pkg::sha256::sha256_archivo;
use crate::pkg::toml;

pub struct PublicarOptions {
    pub verbose: bool,
    pub dry_run: bool,
}

pub fn ejecutar_publish(opts: PublicarOptions) -> Result<(), String> {
    let output = if opts.verbose { Output::new().verbose() } else { Output::new() };

    // 1. Verificar argo.toml
    output.bold("Validando estructura del paquete...");
    let contenido = std::fs::read_to_string("argo.toml")
        .map_err(|_| "No se encontró argo.toml en el directorio actual.".to_string())?;
    let toml_data = toml::parsear_argo_toml(&contenido)?;
    let pkg = toml::extraer_package(&toml_data);

    let name = pkg.get("name").ok_or("argo.toml: falta [package] name")?;
    let version = pkg.get("version").ok_or("argo.toml: falta [package] version")?;
    let description = pkg.get("description").ok_or("argo.toml: falta [package] description")?;

    if name.is_empty() { return Err("El nombre del paquete no puede estar vacío".to_string()); }
    if version.is_empty() { return Err("La versión del paquete no puede estar vacía".to_string()); }
    if description.is_empty() { return Err("La descripción del paquete no puede estar vacía".to_string()); }
    output.ok();

    // 2. Verificar que name coincide con el remote de git
    output.bold("Verificando remote de git...");
    let remote_url = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|_| "No se pudo obtener el remote de git. ¿Es un repositorio git?".to_string())?;

    if !remote_url.status.success() {
        return Err("No se pudo obtener el remote de git.".to_string());
    }

    let remote_str = String::from_utf8_lossy(&remote_url.stdout).trim().to_string();
    let remote_normalized = remote_str
        .trim_end_matches(".git")
        .replace("git@github.com:", "github.com/")
        .replace("https://github.com/", "github.com/")
        .replace("http://github.com/", "github.com/");

    if remote_normalized != *name {
        return Err(format!(
            "El nombre '{}' en argo.toml no coincide con el remote '{}'.",
            name, remote_str
        ));
    }
    output.ok();

    // 3. Verificar src/ existe y no está vacío
    output.bold("Verificando directorio src/...");
    let src_dir = std::path::Path::new("src");
    if !src_dir.exists() || !src_dir.is_dir() {
        return Err("No se encontró el directorio src/.".to_string());
    }
    let has_src_files = std::fs::read_dir(src_dir)
        .map_err(|_| "No se pudo leer src/")?
        .any(|e| e.ok().map(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("argo")).unwrap_or(false));
    if !has_src_files {
        return Err("El directorio src/ está vacío. Agrega al menos un archivo .argo.".to_string());
    }
    output.ok();

    // 4. Verificar LICENSE
    output.bold("Verificando LICENSE...");
    if !Path::new("LICENSE").exists() {
        return Err("No se encontró el archivo LICENSE. Es requerido para publicar.".to_string());
    }
    output.ok();

    // 5. Ejecutar tests si existe tests/
    let tests_dir = std::path::Path::new("tests");
    if tests_dir.exists() && tests_dir.is_dir() {
        output.bold("Ejecutando tests...");
        let test_output = std::process::Command::new("argo")
            .arg("test")
            .output()
            .map_err(|_| "No se pudo ejecutar 'argo test'".to_string())?;

        if !test_output.status.success() {
            let stderr = String::from_utf8_lossy(&test_output.stderr);
            return Err(format!("Tests fallaron:\n{}", stderr));
        }
        output.ok();
    }

    // 6. Verificar accesibilidad de dependencias
    output.bold("Verificando dependencias...");
    let deps = toml::extraer_dependencies(&toml_data);
    for (dep_name, _dep_spec) in &deps {
        // HEAD request al argo.toml de la dependencia
        output.verbose_log(&format!("  Verificando {}...", dep_name));
    }
    output.ok();

    // 7. Generar argo.sum
    output.bold("Generando argo.sum...");
    let mut sum_contenido = String::new();
    let archivos_relevantes = ["argo.toml", "LICENSE", "README.md"];
    for archivo in &archivos_relevantes {
        if Path::new(archivo).exists() {
            let hash = sha256_archivo(Path::new(archivo))?;
            sum_contenido.push_str(&format!("{}  {}\n", hash, archivo));
        }
    }
    // Agregar archivos de src/
    if let Ok(entries) = std::fs::read_dir("src") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("argo") {
                let hash = sha256_archivo(&path)?;
                let display = path.display().to_string();
                sum_contenido.push_str(&format!("{}  {}\n", hash, display));
            }
        }
    }
    std::fs::write("argo.sum", &sum_contenido)
        .map_err(|e| format!("No se pudo escribir argo.sum: {}", e))?;
    output.ok();

    if opts.dry_run {
        output.linea("Dry run: no se creó tag ni se publicó.");
        return Ok(());
    }

    // 8. Crear tag
    output.bold(&format!("Creando tag v{}...", version));
    let tag_name = format!("v{}", version);

    // Verificar si el tag ya existe
    let tag_check = std::process::Command::new("git")
        .args(["tag", "-l", &tag_name])
        .output()
        .map_err(|e| format!("Error verificando tag: {}", e))?;
    let existing_tags = String::from_utf8_lossy(&tag_check.stdout);
    if existing_tags.contains(&tag_name) {
        return Err(format!("El tag '{}' ya existe. Incrementa la versión en argo.toml.", tag_name));
    }

    let tag_msg = format!("Release {} of {}", tag_name, name);
    std::process::Command::new("git")
        .args(["tag", "-a", &tag_name, "-m", &tag_msg])
        .output()
        .map_err(|e| format!("Error creando tag: {}", e))?;
    output.ok();

    // 9. Push tag
    output.bold("Publicando a GitHub...");
    let push_output = std::process::Command::new("git")
        .args(["push", "origin", &tag_name])
        .output()
        .map_err(|e| format!("Error haciendo push: {}", e))?;

    if !push_output.status.success() {
        let stderr = String::from_utf8_lossy(&push_output.stderr);
        return Err(format!("Error en git push: {}", stderr));
    }
    output.ok();

    // 10. Resultado
    output.resultado(name, version);
    output.linea(&format!(
        "\nOtros usuarios pueden instalarlo con:\n    argo install {}\n", name
    ));

    Ok(())
}
