// ---------------------------------------------------------------------------
// lock.rs — Gestión de argo.lock (resolución determinista)
// ---------------------------------------------------------------------------
// Formato:
// # argo.lock — NO EDITAR MANUALMENTE
// version = 1
//
// [[package]]
// name = "github.com/owner/repo"
// version = "v1.0.0"
// resolved = "abc123..."
// sha256 = "e3b0c44..."

use std::collections::HashMap;
use std::path::Path;



#[derive(Debug, Clone)]
pub struct PaqueteLock {
    pub name: String,
    pub version: String,
    pub resolved: String,  // commit hash
    pub sha256: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Lockfile {
    pub version: u32,
    pub paquetes: Vec<PaqueteLock>,
}

impl Lockfile {
    pub fn nueva() -> Self {
        Lockfile { version: 1, paquetes: Vec::new() }
    }

    /// Lee argo.lock desde disco.
    pub fn leer(path: &Path) -> Result<Option<Self>, String> {
        if !path.exists() {
            return Ok(None);
        }
        let contenido = std::fs::read_to_string(path)
            .map_err(|e| format!("No se pudo leer {}: {}", path.display(), e))?;
        Self::parsear(&contenido)
    }

    /// Parsea el contenido de argo.lock.
    pub fn parsear(contenido: &str) -> Result<Option<Self>, String> {
        let mut paquetes = Vec::new();
        let mut version = 1u32;

        // Buscar version = N
        for linea in contenido.lines() {
            let linea = linea.trim();
            if linea.starts_with("version") {
                if let Some((_, val)) = linea.split_once('=') {
                    version = val.trim().parse().unwrap_or(1);
                }
            }
        }

        // Parsear bloques [[package]]
        let mut bloque_actual: Option<PaqueteLock> = None;

        for linea in contenido.lines() {
            let linea = linea.trim();
            if linea.is_empty() || linea.starts_with('#') {
                continue;
            }

            if linea == "[[package]]" {
                if let Some(bloque) = bloque_actual.take() {
                    paquetes.push(bloque);
                }
                bloque_actual = Some(PaqueteLock {
                    name: String::new(),
                    version: String::new(),
                    resolved: String::new(),
                    sha256: String::new(),
                    dependencies: Vec::new(),
                });
                continue;
            }

            if let Some(ref mut bloque) = bloque_actual {
                if let Some((key, val)) = linea.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"');
                    match key {
                        "name" => bloque.name = val.to_string(),
                        "version" => bloque.version = val.to_string(),
                        "resolved" => bloque.resolved = val.to_string(),
                        "sha256" => bloque.sha256 = val.to_string(),
                        _ => {}
                    }
                }
                // Dependencies array (multiline simplificado)
                if linea.starts_with('"') || linea.starts_with("dependencies") {
                    if let Some(start) = linea.find('[') {
                        if let Some(end) = linea.find(']') {
                            let interior = &linea[start + 1..end];
                            for dep in interior.split(',') {
                                let dep = dep.trim().trim_matches('"').trim();
                                if !dep.is_empty() {
                                    bloque.dependencies.push(dep.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some(bloque) = bloque_actual {
            paquetes.push(bloque);
        }

        Ok(Some(Lockfile { version, paquetes }))
    }

    /// Serializa a formato argo.lock.
    pub fn serializar(&self) -> String {
        let mut out = String::new();
        out.push_str("# argo.lock — NO EDITAR MANUALMENTE. Generado por Argo.\n");
        out.push_str(&format!("version = {}\n\n", self.version));

        for pkg in &self.paquetes {
            out.push_str("[[package]]\n");
            out.push_str(&format!("name = \"{}\"\n", pkg.name));
            out.push_str(&format!("version = \"{}\"\n", pkg.version));
            out.push_str(&format!("resolved = \"{}\"\n", pkg.resolved));
            out.push_str(&format!("sha256 = \"{}\"\n", pkg.sha256));
            if !pkg.dependencies.is_empty() {
                out.push_str("dependencies = [");
                for (i, dep) in pkg.dependencies.iter().enumerate() {
                    if i > 0 { out.push_str(", "); }
                    out.push_str(&format!("\"{}\"", dep));
                }
                out.push_str("]\n");
            }
            out.push('\n');
        }

        out
    }

    /// Guarda argo.lock en disco.
    pub fn guardar(&self, path: &Path) -> Result<(), String> {
        let contenido = self.serializar();
        std::fs::write(path, contenido)
            .map_err(|e| format!("No se pudo escribir {}: {}", path.display(), e))
    }

    /// Obtiene un paquete por nombre.
    pub fn obtener_paquete(&self, name: &str) -> Option<&PaqueteLock> {
        self.paquetes.iter().find(|p| p.name == name)
    }

    /// Retorna un mapa nombre -> PaqueteLock.
    pub fn como_mapa(&self) -> HashMap<&str, &PaqueteLock> {
        self.paquetes.iter().map(|p| (p.name.as_str(), p)).collect()
    }
}
