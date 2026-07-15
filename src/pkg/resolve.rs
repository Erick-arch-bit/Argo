// ---------------------------------------------------------------------------
// resolve.rs — Resolución de árbol de dependencias
// ---------------------------------------------------------------------------
// Lee argo.toml, descarga dependencias recursivamente, resuelve versiones
// según rangos semánticos, detecta conflictos, genera argo.lock.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::pkg::cache::{CacheEntry, CacheFile, CacheIndex, ahora_secs};
use crate::pkg::http::PeticionHttp;
use crate::pkg::lock::{Lockfile, PaqueteLock};
use crate::pkg::multihost::{AuthTokens, Host, RepoInfo};
use crate::pkg::output::Output;
use crate::pkg::semver::{Version, VersionReq};
use crate::pkg::sha256::sha256_hex;
use crate::pkg::toml;

#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    pub repo: RepoInfo,
    pub version: String,
    pub resolved: String, // commit hash
    pub sha256: String,
    pub archivos: Vec<(String, String)>, // (path, content)
    pub dependencies: Vec<String>,
}

/// Estado de resolución para detectar ciclos y conflictos.
struct EstadoResolucion {
    resueltos: HashMap<String, ResolvedPackage>,
    en_progreso: HashSet<String>,
    output: Output,
    auth: AuthTokens,
    offline: bool,
}

impl EstadoResolucion {
    fn new(output: Output, auth: AuthTokens, offline: bool) -> Self {
        EstadoResolucion {
            resueltos: HashMap::new(),
            en_progreso: HashSet::new(),
            output,
            auth,
            offline,
        }
    }
}

/// Resuelve todas las dependencias desde argo.toml del proyecto.
pub fn resolver_desde_proyecto(
    output: &Output,
    auth: &AuthTokens,
    offline: bool,
) -> Result<Vec<ResolvedPackage>, String> {
    let contenido = std::fs::read_to_string("argo.toml")
        .map_err(|_| "No se encontró argo.toml en el directorio actual.".to_string())?;

    let toml_data = toml::parsear_argo_toml(&contenido)?;
    let deps = toml::extraer_dependencies(&toml_data);

    let mut estado = EstadoResolucion::new(output.clone(), auth.clone(), offline);

    // Resolver cada dependencia recursivamente
    for (name, version_spec) in &deps {
        estado.resolver_recursivo(name, version_spec)?;
    }

    Ok(estado.resueltos.into_values().collect())
}

impl EstadoResolucion {
    fn resolver_recursivo(&mut self, name: &str, version_spec: &str) -> Result<(), String> {
        // Si ya está resuelto, no hacer nada
        if self.resueltos.contains_key(name) {
            return Ok(());
        }

        // Detectar ciclo
        if self.en_progreso.contains(name) {
            return Err(format!("Dependencia circular detectada: {}", name));
        }
        self.en_progreso.insert(name.to_string());

        self.output.verbose_log(&format!("  Resolviendo {} {}", name, version_spec));

        let repo = RepoInfo::parsear(name)?;
        let req = VersionReq::parsear(version_spec)?;

        // Obtener tags disponibles
        let tags = self.obtener_tags(&repo)?;

        // Señalar warning si no hay tags
        if tags.is_empty() {
            self.output.amarillo(&format!(
                "\n  Advertencia: {} no tiene tags semétricos. Usando rama main.\n", name
            ));
        }

        // Encontrar mejor versión
        let (version, resolved) = if tags.is_empty() {
            // Fallback a main
            let commit = self.obtener_commit_main(&repo)?;
            ("main".to_string(), commit)
        } else {
            let mejor = crate::pkg::semver::mejor_version(&tags, &req)
                .ok_or_else(|| format!(
                    "Ninguna versión de '{}' satisface '{}'. Versiones disponibles: {}",
                    name, version_spec,
                    tags.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")
                ))?;
            let commit = self.obtener_commit_tag(&repo, &mejor.to_string())?;
            (mejor.to_string(), commit)
        };

        // Descargar argo.toml del paquete
        let argo_toml = self.descargar_archivo(&repo, &version, "argo.toml")?;

        // Parsear sus dependencias
        let sub_toml = toml::parsear_argo_toml(&argo_toml).unwrap_or_default();
        let sub_deps = toml::extraer_dependencies(&sub_toml);

        let mut dep_names = Vec::new();
        for (dep_name, dep_spec) in &sub_deps {
            dep_names.push(dep_name.clone());
            self.resolver_recursivo(dep_name, dep_spec)?;
        }

        // Descargar archivos .argo
        let archivos = self.descargar_archivos_paquete(&repo, &version)?;

        // Calcular SHA-256 del contenido combinado
        let mut contenido_total = String::new();
        for (_, content) in &archivos {
            contenido_total.push_str(content);
        }
        let sha256 = sha256_hex(contenido_total.as_bytes());

        self.resueltos.insert(name.to_string(), ResolvedPackage {
            repo,
            version,
            resolved,
            sha256,
            archivos,
            dependencies: dep_names,
        });

        self.en_progreso.remove(name);
        Ok(())
    }

    fn obtener_tags(&self, repo: &RepoInfo) -> Result<Vec<Version>, String> {
        if self.offline {
            return Ok(Vec::new());
        }

        let url = repo.url_api_tags();
        let mut req = PeticionHttp::get(
            &Self::host_from_url(&url),
            Self::puerto_from_url(&url),
            &Self::path_from_url(&url),
        ).con_timeout(30);

        if let Some(token) = self.auth.token_para(&repo.host) {
            req = req.con_token(token);
        }

        let resp = req.enviar()
            .map_err(|e| format!("Error obteniendo tags de {}: {}", repo.full_name, e))?;

        if resp.status == 404 {
            return Err(format!("Repositorio '{}' no encontrado.", repo.full_name));
        }

        if resp.status == 403 {
            let remaining = resp.headers.iter()
                .find(|(k, _)| k == "x-ratelimit-remaining")
                .map(|(_, v)| v.as_str());
            if remaining == Some("0") {
                return Err("Límite de API de GitHub alcanzado. Configura un token con: argo config set github-token <TOKEN>".to_string());
            }
        }

        if resp.status != 200 {
            return Ok(Vec::new());
        }

        // Parsear tags del JSON (simplificado)
        self.parsear_tags_json(&resp.body, &repo.host)
    }

    fn parsear_tags_json(&self, body: &str, _host: &Host) -> Result<Vec<Version>, String> {
        let mut versions = Vec::new();

        // Parser JSON simplificado: buscar "name": "v1.0.0"
        let mut i = 0;
        let bytes = body.as_bytes();
        while i < bytes.len() {
            // Buscar "name"
            if let Some(pos) = body[i..].find("\"name\"") {
                i += pos + 6;
                // Buscar :
                while i < bytes.len() && bytes[i] == b' ' { i += 1; }
                if i < bytes.len() && bytes[i] == b':' { i += 1; }
                while i < bytes.len() && bytes[i] == b' ' { i += 1; }
                // Buscar valor string
                if i < bytes.len() && bytes[i] == b'"' {
                    i += 1;
                    let start = i;
                    while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                    let tag_name = &body[start..i];
                    if let Ok(v) = Version::parsear(tag_name) {
                        versions.push(v);
                    }
                }
            } else {
                break;
            }
        }

        Ok(versions)
    }

    fn obtener_commit_main(&self, repo: &RepoInfo) -> Result<String, String> {
        if self.offline {
            return Ok("offline".to_string());
        }
        // Para GitHub, usar la API de branches
        let url = format!("https://api.github.com/repos/{}/{}/branches/main", repo.owner, repo.repo);
        let host = Self::host_from_url(&url);
        let puerto = Self::puerto_from_url(&url);
        let path = Self::path_from_url(&url);

        let mut req = PeticionHttp::get(&host, puerto, &path).con_timeout(30);
        if let Some(token) = self.auth.token_para(&repo.host) {
            req = req.con_token(token);
        }

        let resp = req.enviar().map_err(|e| format!("Error obteniendo commit: {}", e))?;
        if resp.status != 200 {
            return Ok("unknown".to_string());
        }

        // Extraer SHA del JSON
        if let Some(pos) = resp.body.find("\"sha\"") {
            let rest = &resp.body[pos + 5..];
            if let Some(start) = rest.find('"') {
                let rest = &rest[start + 1..];
                if let Some(end) = rest.find('"') {
                    return Ok(rest[..end].to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    fn obtener_commit_tag(&self, repo: &RepoInfo, tag: &str) -> Result<String, String> {
        if self.offline {
            return Ok("offline".to_string());
        }

        let url = format!("https://api.github.com/repos/{}/{}/git/ref/tags/{}", repo.owner, repo.repo, tag);
        let host = Self::host_from_url(&url);
        let puerto = Self::puerto_from_url(&url);
        let path = Self::path_from_url(&url);

        let mut req = PeticionHttp::get(&host, puerto, &path).con_timeout(30);
        if let Some(token) = self.auth.token_para(&repo.host) {
            req = req.con_token(token);
        }

        let resp = req.enviar().map_err(|e| format!("Error obteniendo commit: {}", e))?;
        if resp.status != 200 {
            return Ok("unknown".to_string());
        }

        // Extraer SHA
        if let Some(pos) = resp.body.find("\"sha\"") {
            let rest = &resp.body[pos + 5..];
            if let Some(start) = rest.find('"') {
                let rest = &rest[start + 1..];
                if let Some(end) = rest.find('"') {
                    return Ok(rest[..end].to_string());
                }
            }
        }

        Ok("unknown".to_string())
    }

    fn descargar_archivo(&self, repo: &RepoInfo, version: &str, path: &str) -> Result<String, String> {
        // Intentar jsDelivr primero (solo GitHub)
        if let Some(url_jsd) = repo.url_jsdelivr(version, path) {
            if let Ok(contenido) = self.descargar_url_raw(&url_jsd) {
                return Ok(contenido);
            }
        }

        // Fallback a raw del host
        let url = repo.url_raw_file(version, path);
        self.descargar_url_raw(&url)
    }

    fn descargar_url_raw(&self, url: &str) -> Result<String, String> {
        if self.offline {
            return Err("Modo offline: no se puede descargar".to_string());
        }

        let host = Self::host_from_url(url);
        let puerto = Self::puerto_from_url(url);
        let path = Self::path_from_url(url);

        let req = PeticionHttp::get(&host, puerto, &path)
            .con_timeout(30)
            .con_reintentos(3);

        let resp = req.enviar()?;
        if resp.status != 200 {
            return Err(format!("HTTP {}: {}", resp.status, url));
        }

        Ok(resp.body)
    }

    fn descargar_archivos_paquete(&self, repo: &RepoInfo, version: &str) -> Result<Vec<(String, String)>, String> {
        // Intentar listar archivos via API
        let archivos = match repo.host {
            crate::pkg::multihost::Host::GitHub => self.listar_archivos_github(repo, version)?,
            _ => {
                // Para otros hosts, intentar archivos comunes
                vec!["argo.toml".to_string(), "src/main.argo".to_string()]
            }
        };

        let mut resultado = Vec::new();
        for path in &archivos {
            if path.ends_with(".argo") || path == "argo.toml" || path == "README.md" || path == "LICENSE" {
                match self.descargar_archivo(repo, version, path) {
                    Ok(content) => resultado.push((path.clone(), content)),
                    Err(_) => continue,
                }
            }
        }

        Ok(resultado)
    }

    fn listar_archivos_github(&self, repo: &RepoInfo, version: &str) -> Result<Vec<String>, String> {
        if self.offline {
            return Ok(Vec::new());
        }

        let url = repo.url_api_trees(version);
        let host = Self::host_from_url(&url);
        let puerto = Self::puerto_from_url(&url);
        let path = Self::path_from_url(&url);

        let mut req = PeticionHttp::get(&host, puerto, &path).con_timeout(30);
        if let Some(token) = self.auth.token_para(&repo.host) {
            req = req.con_token(token);
        }

        let resp = req.enviar()?;
        if resp.status != 200 {
            return Ok(Vec::new());
        }

        // Extraer paths del JSON (simplificado)
        let mut archivos = Vec::new();
        let mut i = 0;
        let bytes = resp.body.as_bytes();
        while i < bytes.len() {
            if let Some(pos) = resp.body[i..].find("\"path\"") {
                i += pos + 6;
                while i < bytes.len() && bytes[i] == b' ' { i += 1; }
                if i < bytes.len() && bytes[i] == b':' { i += 1; }
                while i < bytes.len() && bytes[i] == b' ' { i += 1; }
                if i < bytes.len() && bytes[i] == b'"' {
                    i += 1;
                    let start = i;
                    while i < bytes.len() && bytes[i] != b'"' { i += 1; }
                    let path = &resp.body[start..i];
                    archivos.push(path.to_string());
                }
            } else {
                break;
            }
        }

        Ok(archivos)
    }

    fn host_from_url(url: &str) -> String {
        let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);
        let pos = url.find('/').unwrap_or(url.len());
        let host_port = &url[..pos];
        if let Some(colon) = host_port.find(':') {
            host_port[..colon].to_string()
        } else {
            host_port.to_string()
        }
    }

    fn puerto_from_url(url: &str) -> u16 {
        let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);
        let pos = url.find('/').unwrap_or(url.len());
        let host_port = &url[..pos];
        if let Some(colon) = host_port.find(':') {
            host_port[colon + 1..].parse().unwrap_or(443)
        } else {
            443
        }
    }

    fn path_from_url(url: &str) -> String {
        let url = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")).unwrap_or(url);
        let pos = url.find('/');
        if let Some(pos) = pos {
            format!("/{}", &url[pos + 1..])
        } else {
            "/".to_string()
        }
    }
}

/// Genera argo.lock desde paquetes resueltos.
pub fn generar_lockfile(paquetes: &[ResolvedPackage]) -> Lockfile {
    let mut lock = Lockfile::nueva();
    for pkg in paquetes {
        lock.paquetes.push(PaqueteLock {
            name: pkg.repo.full_name.clone(),
            version: pkg.version.clone(),
            resolved: pkg.resolved.clone(),
            sha256: pkg.sha256.clone(),
            dependencies: pkg.dependencies.clone(),
        });
    }
    lock
}

/// Instala paquetes en vendor/ y actualiza caché.
pub fn instalar_paquetes(
    paquetes: &[ResolvedPackage],
    output: &Output,
) -> Result<(), String> {
    let vendor_dir = PathBuf::from("vendor");
    std::fs::create_dir_all(&vendor_dir)
        .map_err(|e| format!("No se pudo crear vendor/: {}", e))?;

    let mut cache = CacheIndex::cargar()?;
    let now = ahora_secs();

    for (i, pkg) in paquetes.iter().enumerate() {
        output.barra_progreso(i + 1, paquetes.len(), "Instalando");

        let pkg_dir = vendor_dir.join(&pkg.repo.full_name);
        std::fs::create_dir_all(&pkg_dir).ok();

        for (path, content) in &pkg.archivos {
            let file_path = pkg_dir.join(path);
            if let Some(padre) = file_path.parent() {
                std::fs::create_dir_all(padre).ok();
            }
            std::fs::write(&file_path, content)
                .map_err(|e| format!("Error escribiendo {}: {}", file_path.display(), e))?;
        }

        // Actualizar caché
        let files = pkg.archivos.iter().map(|(path, content)| {
            CacheFile {
                path: path.clone(),
                content_offset: 0,
                content_len: content.len() as u64,
                sha256: sha256_hex(content.as_bytes()),
            }
        }).collect();

        let size = pkg.archivos.iter().map(|(_, c)| c.len() as u64).sum();

        cache.insertar(CacheEntry {
            name: pkg.repo.full_name.clone(),
            version: pkg.version.clone(),
            commit_hash: pkg.resolved.clone(),
            resolved: pkg.resolved.clone(),
            installed_at: now,
            last_used: now,
            size_bytes: size,
            file_count: pkg.archivos.len() as u32,
            files,
        });
    }

    cache.guardar()?;
    output.ok();
    Ok(())
}

use crate::pkg::http::RespuestaHttp;

/// Wrapper con reintentos
impl PeticionHttp {
    pub fn con_reintentos(self, max_reintentos: u32) -> PeticionConReintentos {
        PeticionConReintentos { inner: self, max_reintentos }
    }
}

pub struct PeticionConReintentos {
    inner: PeticionHttp,
    max_reintentos: u32,
}

impl PeticionConReintentos {
    pub fn enviar(self) -> Result<RespuestaHttp, String> {
        let mut ultimo_error = String::new();
        for intento in 0..=self.max_reintentos {
            match self.inner.enviar() {
                Ok(resp) => {
                    if resp.status == 429 || resp.status >= 500 {
                        ultimo_error = format!("HTTP {}", resp.status);
                        if intento < self.max_reintentos {
                            let espera = 1u64 << intento;
                            std::thread::sleep(std::time::Duration::from_secs(espera));
                            continue;
                        }
                    }
                    return Ok(resp);
                }
                Err(e) => {
                    ultimo_error = e;
                    if intento < self.max_reintentos {
                        let espera = 1u64 << intento;
                        std::thread::sleep(std::time::Duration::from_secs(espera));
                    }
                }
            }
        }
        Err(ultimo_error)
    }
}
