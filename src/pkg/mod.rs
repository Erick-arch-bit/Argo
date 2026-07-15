// ---------------------------------------------------------------------------
// pkg/ — Sistema de paquetes de Argo (gestión de dependencias)
// ---------------------------------------------------------------------------
// Cero dependencias externas. Todo implementado en Rust std.

#![allow(dead_code, clippy::collapsible_if, clippy::trim_split_whitespace,
    clippy::new_without_default, clippy::len_zero, clippy::manual_strip,
    clippy::manual_is_multiple_of, clippy::manual_contains)]

pub mod cache;
pub mod http;
pub mod lock;
pub mod multihost;
pub mod output;
pub mod publish;
pub mod resolve;
pub mod semver;
pub mod sha256;
pub mod toml;

/// Versión del formato del lockfile.
pub const LOCK_VERSION: u32 = 1;

/// Versión del formato del caché binario.
pub const CACHE_VERSION: u16 = 1;
