# Package Manager — Sistema de Paquetes de Argo

Argo incluye un package manager completo, implementado desde cero en Rust std sin dependencias externas.

## Comandos

| Comando | Descripción |
|---------|-------------|
| `argo install` | Instala dependencias desde `argo.toml` |
| `argo install <repo>` | Instala un paquete específico desde GitHub |
| `argo update` | Actualiza todas las dependencias |
| `argo update <repo>` | Actualiza un paquete específico |
| `argo publish` | Publica el paquete en GitHub |
| `argo publish --dry-run` | Solo validación, sin publicar |
| `argo list` | Lista paquetes instalados |
| `argo clean` | Limpia el caché de paquetes (>30 días) |
| `argo config set <key> <value>` | Configura tokens de autenticación |
| `argo uninstall <repo>` | Desinstala un paquete |

## Archivos del proyecto

### `argo.toml`

Define el paquete y sus dependencias:

```toml
[package]
name = "usuario/mi-proyecto"
version = "1.0.0"
description = "Descripción del proyecto"

[dependencies]
"usuario/dependencia1" = "*"
"usuario/dependencia2" = "^1.2.3"
"otro-usuario/lib" = "~2.0.0"
```

### `argo.lock`

Generado automáticamente por `argo install`. Bloquea versiones exactas:

```toml
version = 1

[[package]]
name = "usuario/dependencia1"
version = "1.3.2"
commit = "abc123"
dependencies = []
```

### `argo.sum`

Checksums SHA-256 de archivos relevantes:

```
a1b2c3...  argo.toml
d4e5f6...  LICENSE
g7h8i9...  src/main.argo
```

### `argo.mod`

Mapa de imports generado automáticamente:

```
dependencia1 = vendor/usuario/dependencia1/argo.toml
dependencia2 = vendor/usuario/dependencia2/argo.toml
```

## Rangos semánticos

| Sintaxis | Significado |
|----------|-------------|
| `*` | Cualquier versión |
| `^1.2.3` | Compatible con 1.2.3 (>=1.2.3, <2.0.0) |
| `~1.2.3` | Similar a 1.2.3 (>=1.2.3, <1.3.0) |
| `1.2.3` | Exactamente 1.2.3 |

## Multi-host (GitHub/GitLab/Bitbucket)

Soporta repositorios de múltiples plataformas:

```bash
# GitHub (por defecto)
argo install usuario/repo

# GitLab
argo install gitlab.com/usuario/repo

# Bitbucket
argo install bitbucket.org/usuario/repo
```

## Autenticación

Configura tokens para acceso a repositorios privados:

```bash
argo config set github-token ghp_xxxxxxxxxxxx
argo config set gitlab-token glpat-xxxxxxxxxxxx
argo config set bitbucket-token xxxxxxxxxxxx
```

Los tokens se almacenan en `~/.argo/config.toml`.

## Caché binario

Los paquetes se almacenan en `~/.argo/cache/` con formato binario optimizado:

- `cache.idx` — Índice con metadatos
- `cache.data` — Contenido de los archivos

El caché migra automáticamente del formato antiguo (directorios sueltos).

## Publicación

Para publicar un paquete:

1. Asegúrate de tener `argo.toml` con `[package]` completo
2. Incluye `LICENSE` y `README.md`
3. Ejecuta `argo publish`

Validaciones:
- Nombre del paquete coincide con el remote de git
- `src/` contiene archivos `.argo`
- Tests pasan (si existe `tests/`)
- Licencia presente

## Arquitectura interna

| Componente | Descripción |
|------------|-------------|
| `toml.rs` | Parser TOML completo (subset) |
| `semver.rs` | Parser SemVer con rangos |
| `sha256.rs` | SHA-256 desde cero (FIPS 180-4) |
| `http.rs` | Cliente HTTP/1.1 sobre TcpStream |
| `lock.rs` | Lectura/escritura de argo.lock |
| `cache.rs` | Caché binario con migración |
| `output.rs` | Colores y barra de progreso |
| `multihost.rs` | URLs y auth para GitHub/GitLab/Bitbucket |
| `resolve.rs` | Resolución recursiva + lockfile |
| `publish.rs` | Publicación con SHA-256 y tag |
