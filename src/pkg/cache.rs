// ---------------------------------------------------------------------------
// cache.rs — Caché local con formato binario propio
// ---------------------------------------------------------------------------
// Formato: cache.idx (índice) + cache.data (contenido)
// Magic: b"ARGO", versión 1, entradas con metadatos.

use std::path::PathBuf;

use crate::pkg::sha256::sha256_hex;

pub struct CacheEntry {
    pub name: String,
    pub version: String,
    pub commit_hash: String,
    pub resolved: String,
    pub installed_at: u64,
    pub last_used: u64,
    pub size_bytes: u64,
    pub file_count: u32,
    pub files: Vec<CacheFile>,
}

pub struct CacheFile {
    pub path: String,
    pub content_offset: u64,
    pub content_len: u64,
    pub sha256: String,
}

pub struct CacheIndex {
    pub entries: Vec<CacheEntry>,
}

impl CacheIndex {
    pub fn nuevo() -> Self {
        CacheIndex { entries: Vec::new() }
    }

    /// Obtiene la ruta del directorio de caché (~/.argo/).
    pub fn dir_cache() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home).join(".argo")
    }

    /// Ruta de cache.idx
    pub fn path_idx() -> PathBuf {
        Self::dir_cache().join("cache.idx")
    }

    /// Ruta de cache.data
    pub fn path_data() -> PathBuf {
        Self::dir_cache().join("cache.data")
    }

    /// Carga el índice desde disco.
    pub fn cargar() -> Result<Self, String> {
        let path = Self::path_idx();
        if !path.exists() {
            return Ok(CacheIndex::nuevo());
        }
        let datos = std::fs::read(&path)
            .map_err(|e| format!("No se pudo leer cache.idx: {}", e))?;
        Self::deserializar(&datos)
    }

    /// Guarda el índice a disco.
    pub fn guardar(&self) -> Result<(), String> {
        let path = Self::path_idx();
        if let Some(padre) = path.parent() {
            std::fs::create_dir_all(padre).ok();
        }
        let datos = self.serializar();
        std::fs::write(&path, &datos)
            .map_err(|e| format!("No se pudo escribir cache.idx: {}", e))
    }

    /// Serializa a formato binario.
    pub fn serializar(&self) -> Vec<u8> {
        let mut out = Vec::new();

        // Header
        out.extend_from_slice(b"ARGO");          // magic
        out.extend_from_slice(&1u16.to_le_bytes()); // version
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());

        // Entries
        for entry in &self.entries {
            Self::write_string(&mut out, &entry.name);
            Self::write_string(&mut out, &entry.version);
            Self::write_fixed(&mut out, &entry.commit_hash, 40);
            out.extend_from_slice(&entry.installed_at.to_le_bytes());
            out.extend_from_slice(&entry.last_used.to_le_bytes());
            out.extend_from_slice(&entry.size_bytes.to_le_bytes());
            out.extend_from_slice(&entry.file_count.to_le_bytes());

            // Files
            for file in &entry.files {
                Self::write_string(&mut out, &file.path);
                out.extend_from_slice(&file.content_offset.to_le_bytes());
                out.extend_from_slice(&file.content_len.to_le_bytes());
                Self::write_fixed(&mut out, &file.sha256, 64);
            }
        }

        out
    }

    /// Deserializa desde formato binario.
    pub fn deserializar(datos: &[u8]) -> Result<Self, String> {
        if datos.len() < 8 {
            return Err("cache.idx demasiado pequeño".to_string());
        }
        if &datos[0..4] != b"ARGO" {
            return Err("cache.idx magic inválido".to_string());
        }
        let version = u16::from_le_bytes([datos[4], datos[5]]);
        if version != 1 {
            return Err(format!("cache.idx versión no soportada: {}", version));
        }
        let entry_count = u32::from_le_bytes([datos[6], datos[7], datos[8], datos[9]]) as usize;
        let mut pos = 10;
        let mut entries = Vec::with_capacity(entry_count);

        for _ in 0..entry_count {
            let (name, new_pos) = Self::read_string(datos, pos)?;
            pos = new_pos;
            let (version, new_pos) = Self::read_string(datos, pos)?;
            pos = new_pos;
            let commit_hash = Self::read_fixed(datos, pos, 40)?;
            pos += 40;
            let installed_at = u64::from_le_bytes(datos[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let last_used = u64::from_le_bytes(datos[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let size_bytes = u64::from_le_bytes(datos[pos..pos + 8].try_into().unwrap());
            pos += 8;
            let file_count = u32::from_le_bytes(datos[pos..pos + 4].try_into().unwrap());
            pos += 4;

            let mut files = Vec::with_capacity(file_count as usize);
            for _ in 0..file_count {
                let (path, new_pos) = Self::read_string(datos, pos)?;
                pos = new_pos;
                let content_offset = u64::from_le_bytes(datos[pos..pos + 8].try_into().unwrap());
                pos += 8;
                let content_len = u64::from_le_bytes(datos[pos..pos + 8].try_into().unwrap());
                pos += 8;
                let sha256 = Self::read_fixed(datos, pos, 64)?;
                pos += 64;
                files.push(CacheFile { path, content_offset, content_len, sha256 });
            }

            entries.push(CacheEntry {
                name, version, commit_hash, resolved: String::new(), installed_at, last_used,
                size_bytes, file_count, files,
            });
        }

        Ok(CacheIndex { entries })
    }

    fn write_string(out: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(bytes);
    }

    fn write_fixed(out: &mut Vec<u8>, s: &str, len: usize) {
        let bytes = s.as_bytes();
        let mut buf = vec![0u8; len];
        let copy_len = bytes.len().min(len);
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        out.extend_from_slice(&buf);
    }

    fn read_string(datos: &[u8], pos: usize) -> Result<(String, usize), String> {
        if pos + 2 > datos.len() { return Err("EOF".to_string()); }
        let len = u16::from_le_bytes([datos[pos], datos[pos + 1]]) as usize;
        let start = pos + 2;
        if start + len > datos.len() { return Err("EOF".to_string()); }
        let s = String::from_utf8_lossy(&datos[start..start + len]).to_string();
        Ok((s, start + len))
    }

    fn read_fixed(datos: &[u8], pos: usize, len: usize) -> Result<String, String> {
        if pos + len > datos.len() { return Err("EOF".to_string()); }
        Ok(String::from_utf8_lossy(&datos[pos..pos + len]).to_string())
    }

    /// Agrega o actualiza una entrada.
    pub fn insertar(&mut self, entry: CacheEntry) {
        self.entries.retain(|e| e.name != entry.name);
        self.entries.push(entry);
    }

    /// Elimina una entrada por nombre.
    pub fn eliminar(&mut self, name: &str) {
        self.entries.retain(|e| e.name != name);
    }

    /// Retorna entrada por nombre.
    pub fn buscar(&self, name: &str) -> Option<&CacheEntry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// Actualiza last_used de una entrada.
    pub fn tocar(&mut self, name: &str, now: u64) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name == name) {
            entry.last_used = now;
        }
    }
}

/// Timestamp UNIX actual en segundos.
pub fn ahora_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Migración automática desde directorio plano antiguo (~/.argo/cache/).
pub fn migrar_cache_antiguo() -> Result<(), String> {
    let cache_dir = CacheIndex::dir_cache().join("cache");
    if !cache_dir.exists() {
        return Ok(());
    }

    let idx_path = CacheIndex::path_idx();
    if idx_path.exists() {
        return Ok(()); // Ya migrado
    }

    eprint!("  Migrando caché antiguo a formato binario...");
    let mut index = CacheIndex::nuevo();

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("argo") {
                let nombre = path.file_stem().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
                let datos = std::fs::read(&path).unwrap_or_default();
                let hash = sha256_hex(&datos);
                let now = ahora_secs();
                index.insertar(CacheEntry {
                    name: nombre,
                    version: "0.0.0".to_string(),
                    commit_hash: "unknown".to_string(),
                    resolved: String::new(),
                    installed_at: now,
                    last_used: now,
                    size_bytes: datos.len() as u64,
                    file_count: 1,
                    files: vec![CacheFile {
                        path: "src/main.argo".to_string(),
                        content_offset: 0,
                        content_len: datos.len() as u64,
                        sha256: hash,
                    }],
                });
            }
        }
    }

    index.guardar()?;
    eprintln!(" OK");
    Ok(())
}
