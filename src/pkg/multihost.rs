// ---------------------------------------------------------------------------
// multihost.rs — Soporte multi-host (GitHub, GitLab, Bitbucket)
// ---------------------------------------------------------------------------
// Parsea URLs de repositorio y construye URLs de API/raw para cada host.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Host {
    GitHub,
    GitLab,
    Bitbucket,
}

impl Host {
    pub fn from_domain(domain: &str) -> Option<Self> {
        match domain {
            "github.com" => Some(Host::GitHub),
            "gitlab.com" => Some(Host::GitLab),
            "bitbucket.org" => Some(Host::Bitbucket),
            _ => None,
        }
    }

    pub fn nombre(&self) -> &str {
        match self {
            Host::GitHub => "github.com",
            Host::GitLab => "gitlab.com",
            Host::Bitbucket => "bitbucket.org",
        }
    }

    pub fn hosts_soportados() -> &'static [&'static str] {
        &["github.com", "gitlab.com", "bitbucket.org"]
    }
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub host: Host,
    pub owner: String,
    pub repo: String,
    pub full_name: String, // owner/repo
}

impl RepoInfo {
    /// Parsea una cadena como "github.com/owner/repo" o "owner/repo" (default github).
    pub fn parsear(input: &str) -> Result<Self, String> {
        let input = input.trim().trim_end_matches(".git");

        // Con host explícito
        if let Some(pos) = input.find('/') {
            let dominio = &input[..pos];
            if let Some(host) = Host::from_domain(dominio) {
                let resto = &input[pos + 1..];
                let partes: Vec<&str> = resto.splitn(2, '/').collect();
                if partes.len() < 2 || partes[0].is_empty() || partes[1].is_empty() {
                    return Err(format!("Formato inválido: '{}'. Se esperaba host/owner/repo", input));
                }
                let owner = partes[0].to_string();
                let repo = partes[1].to_string();
                let full_name = format!("{}/{}", owner, repo);
                return Ok(RepoInfo { host, owner, repo, full_name });
            }
        }

        // Sin host → default github.com
        let partes: Vec<&str> = input.splitn(2, '/').collect();
        if partes.len() < 2 || partes[0].is_empty() || partes[1].is_empty() {
            return Err(format!("Formato inválido: '{}'. Se esperaba owner/repo o host/owner/repo", input));
        }
        let owner = partes[0].to_string();
        let repo = partes[1].to_string();
        let full_name = format!("{}/{}", owner, repo);
        Ok(RepoInfo { host: Host::GitHub, owner, repo, full_name })
    }

    /// URL de API para listar tags.
    pub fn url_api_tags(&self) -> String {
        match self.host {
            Host::GitHub => {
                format!("https://api.github.com/repos/{}/{}", self.owner, self.repo)
            }
            Host::GitLab => {
                let encoded = format!("{}/{}", self.owner, self.repo).replace('/', "%2F");
                format!("https://gitlab.com/api/v4/projects/{}/repository/tags", encoded)
            }
            Host::Bitbucket => {
                format!("https://api.bitbucket.org/2.0/repositories/{}/{}", self.owner, self.repo)
            }
        }
    }

    /// URL raw para descargar un archivo en una versión específica.
    pub fn url_raw_file(&self, version: &str, path: &str) -> String {
        match self.host {
            Host::GitHub => {
                format!("https://raw.githubusercontent.com/{}/{}/{}/{}", self.owner, self.repo, version, path)
            }
            Host::GitLab => {
                let encoded = format!("{}/{}", self.owner, self.repo).replace('/', "%2F");
                format!("https://gitlab.com/api/v4/projects/{}/repository/files/{}/raw?ref={}", encoded, path.replace('/', "%2F"), version)
            }
            Host::Bitbucket => {
                format!("https://bitbucket.org/{}/{}/raw/{}/{}", self.owner, self.repo, version, path)
            }
        }
    }

    /// URL de CDN jsDelivr (solo GitHub).
    pub fn url_jsdelivr(&self, version: &str, path: &str) -> Option<String> {
        if self.host == Host::GitHub {
            Some(format!("https://cdn.jsdelivr.net/gh/{}-{}@{}/{}", self.owner, self.repo, version, path))
        } else {
            None
        }
    }

    /// URL API para obtener trees (listar archivos).
    pub fn url_api_trees(&self, version: &str) -> String {
        match self.host {
            Host::GitHub => {
                format!("https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1", self.owner, self.repo, version)
            }
            Host::GitLab => {
                let encoded = format!("{}/{}", self.owner, self.repo).replace('/', "%2F");
                format!("https://gitlab.com/api/v4/projects/{}/repository/tree?ref={}&per_page=100", encoded, version)
            }
            Host::Bitbucket => {
                format!("https://api.bitbucket.org/2.0/repositories/{}/{}/src/{}/", self.owner, self.repo, version)
            }
        }
    }
}

/// Token de autenticación por host.
#[derive(Clone)]
pub struct AuthTokens {
    pub tokens: HashMap<String, String>,
}

impl AuthTokens {
    pub fn new() -> Self {
        AuthTokens { tokens: HashMap::new() }
    }

    /// Carga tokens desde ~/.argo/config.toml.
    pub fn cargar() -> Self {
        let mut tokens = HashMap::new();
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
        let config_path = std::path::PathBuf::from(home).join(".argo").join("config.toml");

        if let Ok(contenido) = std::fs::read_to_string(&config_path) {
            for linea in contenido.lines() {
                let linea = linea.trim();
                if linea.starts_with('#') || !linea.contains('=') { continue; }
                if let Some((key, val)) = linea.split_once('=') {
                    let key = key.trim();
                    let val = val.trim().trim_matches('"').trim_matches('\'');
                    if !val.is_empty() {
                        tokens.insert(key.to_string(), val.to_string());
                    }
                }
            }
        }

        AuthTokens { tokens }
    }

    pub fn token_para(&self, host: &Host) -> Option<&str> {
        let key = match host {
            Host::GitHub => "github-token",
            Host::GitLab => "gitlab-token",
            Host::Bitbucket => "bitbucket-token",
        };
        self.tokens.get(key).map(|s| s.as_str())
    }

    pub fn set_token(&mut self, host: &str, token: &str) {
        self.tokens.insert(format!("{}-token", host), token.to_string());
    }

    pub fn guardar(&self) -> Result<(), String> {
        let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).unwrap_or_default();
        let dir = std::path::PathBuf::from(home).join(".argo");
        std::fs::create_dir_all(&dir).ok();
        let config_path = dir.join("config.toml");

        let mut contenido = String::new();
        if let Ok(existente) = std::fs::read_to_string(&config_path) {
            contenido = existente;
        }

        // Actualizar o agregar tokens
        for (key, val) in &self.tokens {
            let key_pattern = format!("{} =", key);
            if contenido.contains(&key_pattern) {
                // Reemplazar línea existente
                let lineas: Vec<String> = contenido.lines().map(|l| {
                    if l.trim().starts_with(&key_pattern) || l.trim().starts_with(&format!("{}=", key)) {
                        format!("{} = \"{}\"", key, val)
                    } else {
                        l.to_string()
                    }
                }).collect();
                contenido = lineas.join("\n");
            } else {
                contenido.push_str(&format!("\n{} = \"{}\"\n", key, val));
            }
        }

        std::fs::write(&config_path, &contenido)
            .map_err(|e| format!("No se pudo escribir config.toml: {}", e))
    }
}
