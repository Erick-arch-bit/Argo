// ---------------------------------------------------------------------------
// semver.rs — Parser de versiones semánticas propio
// ---------------------------------------------------------------------------
// Soporta: v1.0.0, 1.0.0, ^1.0.0, ~1.2.0, *, 1.0.0-beta, 1.0.0+build

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: String,   // ej: "beta", "alpha.1"
    pub build: String,  // ej: "build.123"
}

impl Version {
    pub fn nueva(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch, pre: String::new(), build: String::new() }
    }

    pub fn parsear(s: &str) -> Result<Self, String> {
        let s = s.trim().trim_start_matches('v');
        let (version_str, pre, build) = Self::separar_pre_build(s);

        let partes: Vec<&str> = version_str.split('.').collect();
        if partes.len() < 1 || partes.len() > 3 {
            return Err(format!("Versión inválida: '{}'", s));
        }

        let major = partes[0].parse::<u32>().map_err(|_| format!("Major inválido: '{}'", partes[0]))?;
        let minor = if partes.len() > 1 {
            partes[1].parse::<u32>().map_err(|_| format!("Minor inválido: '{}'", partes[1]))?
        } else { 0 };
        let patch = if partes.len() > 2 {
            partes[2].parse::<u32>().map_err(|_| format!("Patch inválido: '{}'", partes[2]))?
        } else { 0 };

        Ok(Version { major, minor, patch, pre, build })
    }

    fn separar_pre_build(s: &str) -> (String, String, String) {
        let mut version = String::new();
        let mut pre = String::new();
        let mut build = String::new();
        let mut estado = 0; // 0=version, 1=pre, 2=build

        for c in s.chars() {
            match estado {
                0 => {
                    if c == '-' { estado = 1; }
                    else if c == '+' { estado = 2; }
                    else { version.push(c); }
                }
                1 => {
                    if c == '+' { estado = 2; }
                    else { pre.push(c); }
                }
                2 => { build.push(c); }
                _ => {}
            }
        }

        (version, pre, build)
    }

    /// Compara versiones sin pre-release (major.minor.patch).
    pub fn cmp_version(&self, other: &Version) -> std::cmp::Ordering {
        self.major.cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
    }

    pub fn es_compatible_con(&self, req: &VersionReq) -> bool {
        match req {
            VersionReq::Exact(v) => self.cmp_version(v) == std::cmp::Ordering::Equal,
            VersionReq::Caret(v) => {
                // ^1.0.0 => >=1.0.0, <2.0.0
                // ^0.1.0 => >=0.1.0, <0.2.0
                // ^0.0.1 => >=0.0.1, <0.0.2
                if self.cmp_version(v) == std::cmp::Ordering::Less {
                    return false;
                }
                if v.major > 0 {
                    self.major == v.major
                } else if v.minor > 0 {
                    self.major == 0 && self.minor == v.minor
                } else {
                    self.major == 0 && self.minor == 0 && self.patch == v.patch
                }
            }
            VersionReq::Tilde(v) => {
                // ~1.2.0 => >=1.2.0, <1.3.0
                self.cmp_version(v) != std::cmp::Ordering::Less
                    && self.major == v.major
                    && self.minor == v.minor
            }
            VersionReq::Wildcard => true,
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if !self.pre.is_empty() { write!(f, "-{}", self.pre)?; }
        if !self.build.is_empty() { write!(f, "+{}", self.build)?; }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum VersionReq {
    Exact(Version),   // v1.0.0
    Caret(Version),   // ^1.0.0
    Tilde(Version),   // ~1.2.0
    Wildcard,         // *
}

impl VersionReq {
    pub fn parsear(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s == "*" {
            return Ok(VersionReq::Wildcard);
        }
        if s.starts_with('^') {
            let ver = Version::parsear(&s[1..])?;
            return Ok(VersionReq::Caret(ver));
        }
        if s.starts_with('~') {
            let ver = Version::parsear(&s[1..])?;
            return Ok(VersionReq::Tilde(ver));
        }
        let ver = Version::parsear(s)?;
        Ok(VersionReq::Exact(ver))
    }

    pub fn satisface(&self, version: &Version) -> bool {
        version.es_compatible_con(self)
    }
}

/// Encuentra la mejor versión que satisface un rango.
pub fn mejor_version(versions: &[Version], req: &VersionReq) -> Option<Version> {
    let mut candidatas: Vec<&Version> = versions.iter()
        .filter(|v| req.satisface(v))
        .collect();
    candidatas.sort_by(|a, b| b.cmp_version(a)); // descendente
    candidatas.into_iter().next().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsear_version() {
        let v = Version::parsear("v1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn parsear_con_pre() {
        let v = Version::parsear("1.0.0-beta.1").unwrap();
        assert_eq!(v.pre, "beta.1");
    }

    #[test]
    fn caret_compat() {
        let req = VersionReq::parsear("^1.0.0").unwrap();
        assert!(Version::parsear("1.0.0").unwrap().es_compatible_con(&req));
        assert!(Version::parsear("1.5.0").unwrap().es_compatible_con(&req));
        assert!(!Version::parsear("2.0.0").unwrap().es_compatible_con(&req));
    }

    #[test]
    fn tilde_compat() {
        let req = VersionReq::parsear("~1.2.0").unwrap();
        assert!(Version::parsear("1.2.0").unwrap().es_compatible_con(&req));
        assert!(Version::parsear("1.2.5").unwrap().es_compatible_con(&req));
        assert!(!Version::parsear("1.3.0").unwrap().es_compatible_con(&req));
    }
}
