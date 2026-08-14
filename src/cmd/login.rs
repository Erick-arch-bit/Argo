// ---------------------------------------------------------------------------
// login.rs — argo login: genera la identidad del desarrollador (Ed25519)
// ---------------------------------------------------------------------------
// Cero dependencias externas. El par de llaves se genera con OpenSSL del
// sistema operativo y se guarda en ~/.argo/keys/:
//   - id_ed25519.pem      → llave privada (NUNCA se comparte)
//   - id_ed25519_pub.pem  → llave pública (se pega en el perfil web de Argo)

use std::process::Command;

use crate::paquetes::ruta_carpeta_llaves;

/// Ejecuta `argo login`.
///
/// 1. Crea ~/.argo/keys si no existe.
/// 2. Genera una llave Ed25519 con OpenSSL.
/// 3. Guarda la llave privada en id_ed25519.pem.
/// 4. Extrae y guarda la llave pública en id_ed25519_pub.pem.
/// 5. Imprime la llave pública con instrucciones para el perfil web.
pub fn ejecutar_login() -> Result<(), String> {
    let ruta_carpeta = ruta_carpeta_llaves();

    // 1. Crear el directorio de llaves
    std::fs::create_dir_all(&ruta_carpeta)
        .map_err(|e| format!("Error: No se pudo crear '{}': {}", ruta_carpeta, e))?;

    // 2. Generar la llave privada Ed25519 (genpkey escribe la llave en stdout)
    let genpkey = Command::new("openssl")
        .args(["genpkey", "-algorithm", "Ed25519"])
        .output()
        .map_err(|_| "Error: ¿Está instalado 'openssl' en el sistema?".to_string())?;

    if !genpkey.status.success() {
        let stderr = String::from_utf8_lossy(&genpkey.stderr);
        return Err(format!(
            "Error: No se pudo generar la llave Ed25519 con OpenSSL: {}",
            stderr.trim()
        ));
    }

    // 3. Guardar la llave privada
    let ruta_privada = format!("{}/id_ed25519.pem", ruta_carpeta);
    std::fs::write(&ruta_privada, &genpkey.stdout)
        .map_err(|e| format!("Error: No se pudo escribir la llave privada: {}", e))?;

    // 4. Extraer la llave pública
    let pubkey = Command::new("openssl")
        .args(["pkey", "-in"])
        .arg(&ruta_privada)
        .arg("-pubout")
        .output()
        .map_err(|_| "Error: ¿Está instalado 'openssl' en el sistema?".to_string())?;

    if !pubkey.status.success() {
        let stderr = String::from_utf8_lossy(&pubkey.stderr);
        return Err(format!(
            "Error: No se pudo extraer la llave pública de '{}': {}",
            ruta_privada,
            stderr.trim()
        ));
    }

    let ruta_publica = format!("{}/id_ed25519_pub.pem", ruta_carpeta);
    std::fs::write(&ruta_publica, &pubkey.stdout)
        .map_err(|e| format!("Error: No se pudo escribir la llave pública: {}", e))?;

    // 5. Imprimir la llave pública con formato claro
    let llave_publica = String::from_utf8_lossy(&pubkey.stdout).trim().to_string();

    println!();
    println!("  ----- LLAVE PÚBLICA PARA TU PERFIL EN LA WEB -----");
    println!("  {}", llave_publica);
    println!("  --------------------------------------------------");
    println!();
    println!("  1. Copia la llave pública de arriba (incluyendo las líneas");
    println!("     '-----BEGIN PUBLIC KEY-----' y '-----END PUBLIC KEY-----').");
    println!("  2. Pégala en tu perfil de desarrollador en la web de Argo.");
    println!("     Así el registro sabrá reconocer las firmas de tus paquetes.");
    println!();
    println!("  Tu llave privada se guardó en: {}", ruta_privada);
    println!("  NUNCA la compartas: con ella se firman tus paquetes.");
    println!("  La llave pública también se guardó en: {}", ruta_publica);
    println!();

    Ok(())
}
