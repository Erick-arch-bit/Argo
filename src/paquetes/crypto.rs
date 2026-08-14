// ---------------------------------------------------------------------------
// crypto.rs — Módulo criptográfico de Argo-Lang (sistema de paquetes)
// ---------------------------------------------------------------------------
// Cero dependencias externas. Toda la criptografía se delega en herramientas
// nativas del sistema operativo vía std::process::Command:
//   - sha256sum → hash de integridad del paquete
//   - openssl   → generación, firma y verificación Ed25519
//
// IMPORTANTE (OpenSSL 3.x): las llaves Ed25519 NO aceptan un digest explícito
// ("Explicit digest not allowed with EdDSA operations"), por lo que `dgst
// -sha256 -sign` no funciona con ellas. La invocación nativa correcta es
// `openssl pkeyutl -sign/-verify -rawin`, que firma/verifica los bytes tal
// cual (Ed25519 ya hashea internamente).
//
// Si la herramienta no está instalada, se aborta con un error claro en lugar
// de compilar librerías gigantes.

use std::process::Command;

/// Calcula el hash SHA-256 de un archivo usando la herramienta `sha256sum`.
///
/// El formato de salida de `sha256sum -b` es:
///   `<64 caracteres hex> *<nombre del archivo>\n`
/// Se extraen solo los 64 caracteres hexadecimales del hash.
pub fn calcular_hash(ruta_archivo: &str) -> Result<String, String> {
    let salida = Command::new("sha256sum")
        .arg("-b")
        .arg(ruta_archivo)
        .output()
        .map_err(|_| "Error: ¿Está instalado 'sha256sum' en el sistema?".to_string())?;

    if !salida.status.success() {
        let stderr = String::from_utf8_lossy(&salida.stderr);
        return Err(format!(
            "Error: sha256sum no pudo procesar '{}': {}",
            ruta_archivo,
            stderr.trim()
        ));
    }

    let texto = String::from_utf8_lossy(&salida.stdout).to_string();
    let hash = texto.split_whitespace().next().unwrap_or("").to_string();

    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "Error: salida inesperada de sha256sum para '{}'",
            ruta_archivo
        ));
    }

    Ok(hash)
}

/// Firma un archivo con una llave privada Ed25519 usando OpenSSL.
///
/// Deja la firma binaria en `ruta_salida_firma` y devuelve esa ruta.
pub fn firmar_archivo(
    ruta_archivo: &str,
    ruta_llave_privada: &str,
    ruta_salida_firma: &str,
) -> Result<String, String> {
    let salida = Command::new("openssl")
        .args(["pkeyutl", "-sign", "-inkey"])
        .arg(ruta_llave_privada)
        .args(["-rawin", "-in"])
        .arg(ruta_archivo)
        .args(["-out"])
        .arg(ruta_salida_firma)
        .output()
        .map_err(|_| "Error: ¿Está instalado 'openssl' en el sistema?".to_string())?;

    if !salida.status.success() {
        let stderr = String::from_utf8_lossy(&salida.stderr);
        return Err(format!(
            "Error: No se pudo firmar '{}' con la llave '{}': {}",
            ruta_archivo,
            ruta_llave_privada,
            stderr.trim()
        ));
    }

    Ok(ruta_salida_firma.to_string())
}

/// Verifica la firma de un archivo contra una llave pública Ed25519.
///
/// - `Ok(true)`  → la firma es válida y el paquete no fue alterado.
/// - `Err(...)`  → la firma no coincide (paquete alterado o malicioso).
pub fn verificar_firma(
    ruta_archivo: &str,
    ruta_firma: &str,
    ruta_llave_publica: &str,
) -> Result<bool, String> {
    let status = Command::new("openssl")
        .args(["pkeyutl", "-verify", "-pubin", "-inkey"])
        .arg(ruta_llave_publica)
        .args(["-rawin", "-in"])
        .arg(ruta_archivo)
        .args(["-sigfile"])
        .arg(ruta_firma)
        .status()
        .map_err(|_| "Error: ¿Está instalado 'openssl' en el sistema?".to_string())?;

    if status.success() {
        Ok(true)
    } else {
        Err(
            "ERROR CRÍTICO: La firma criptográfica no coincide. El paquete fue alterado \
             o es malicioso. Instalación cancelada."
                .to_string(),
        )
    }
}

// ---------------------------------------------------------------------------
// Base64 (implementado a mano, sin dependencias externas)
// ---------------------------------------------------------------------------

const ALFABETO_BASE64: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Codifica bytes en Base64 estándar con padding ('=').
pub fn codificar_base64(datos: &[u8]) -> String {
    let mut salida = String::with_capacity((datos.len() + 2) / 3 * 4);
    let mut i = 0;

    while i + 3 <= datos.len() {
        let n = ((datos[i] as u32) << 16) | ((datos[i + 1] as u32) << 8) | (datos[i + 2] as u32);
        salida.push(ALFABETO_BASE64[(n >> 18) as usize & 0x3F] as char);
        salida.push(ALFABETO_BASE64[(n >> 12) as usize & 0x3F] as char);
        salida.push(ALFABETO_BASE64[(n >> 6) as usize & 0x3F] as char);
        salida.push(ALFABETO_BASE64[n as usize & 0x3F] as char);
        i += 3;
    }

    match datos.len() - i {
        1 => {
            let n = (datos[i] as u32) << 16;
            salida.push(ALFABETO_BASE64[(n >> 18) as usize & 0x3F] as char);
            salida.push(ALFABETO_BASE64[(n >> 12) as usize & 0x3F] as char);
            salida.push('=');
            salida.push('=');
        }
        2 => {
            let n = ((datos[i] as u32) << 16) | ((datos[i + 1] as u32) << 8);
            salida.push(ALFABETO_BASE64[(n >> 18) as usize & 0x3F] as char);
            salida.push(ALFABETO_BASE64[(n >> 12) as usize & 0x3F] as char);
            salida.push(ALFABETO_BASE64[(n >> 6) as usize & 0x3F] as char);
            salida.push('=');
        }
        _ => {}
    }

    salida
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn base64_casos_conocidos() {
        assert_eq!(codificar_base64(b""), "");
        assert_eq!(codificar_base64(b"f"), "Zg==");
        assert_eq!(codificar_base64(b"fo"), "Zm8=");
        assert_eq!(codificar_base64(b"foo"), "Zm9v");
        assert_eq!(codificar_base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn flujo_completo_firma_ed25519() {
        // Requiere sha256sum y openssl instalados en el sistema.
        let dir = std::env::temp_dir().join(format!("argo_crypto_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let archivo = dir.join("test.txt");
        {
            let mut f = std::fs::File::create(&archivo).unwrap();
            writeln!(f, "contenido de prueba").unwrap();
        }

        let hash = calcular_hash(archivo.to_str().unwrap()).unwrap();
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

        let privada = dir.join("id.pem");
        let publica = dir.join("id_pub.pem");
        let genpkey = Command::new("openssl")
            .args(["genpkey", "-algorithm", "Ed25519", "-out"])
            .arg(&privada)
            .output()
            .unwrap();
        assert!(genpkey.status.success(), "openssl genpkey falló");

        let pubo = Command::new("openssl")
            .args(["pkey", "-in"])
            .arg(&privada)
            .args(["-pubout", "-out"])
            .arg(&publica)
            .output()
            .unwrap();
        assert!(pubo.status.success(), "openssl pkey -pubout falló");

        let firma = dir.join("firma.bin");
        firmar_archivo(
            archivo.to_str().unwrap(),
            privada.to_str().unwrap(),
            firma.to_str().unwrap(),
        )
        .unwrap();

        let verificado = verificar_firma(
            archivo.to_str().unwrap(),
            firma.to_str().unwrap(),
            publica.to_str().unwrap(),
        )
        .unwrap();
        assert!(verificado, "La firma válida debe verificarse");

        // Alterar el archivo: la verificación debe fallar con el error crítico.
        std::fs::write(&archivo, "contenido alterado").unwrap();
        let error = verificar_firma(
            archivo.to_str().unwrap(),
            firma.to_str().unwrap(),
            publica.to_str().unwrap(),
        )
        .unwrap_err();
        assert!(error.contains("firma criptográfica no coincide"), "error: {}", error);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
