# Comandos CLI de Argo

Referencia completa de todos los comandos disponibles.

## Inicialización

### `argo init`

Crea un nuevo proyecto Argo con la estructura básica:

```
mi-proyecto/
├── argo.toml
├── src/
│   └── main.argo
└── tests/
```

---

## Ejecución

### `argo run`

Ejecuta el proyecto actual (lee `argo.toml` para encontrar el entry point).

### `argo <archivo.argo>`

Ejecuta un script directamente.

### `argo repl`

Inicia el REPL interactivo.

---

## Testing

### `argo test`

Ejecuta todos los archivos `*.test.argo` en el directorio actual.

```argo
// tests/matematicas.test.argo
assert(1 + 1 == 2, "suma basica");
assert(2 * 3 == 6, "multiplicacion");
```

---

## Package Manager

### `argo install`

Instala todas las dependencias definidas en `argo.toml`.

```bash
argo install
```

### `argo install <repo>`

Instala un paquete específico.

```bash
argo install usuario/repo
argo install gitlab.com/usuario/repo
```

### `argo update`

Actualiza todas las dependencias (elimina `argo.lock` y resuelve de nuevo).

```bash
argo update
```

### `argo update <repo>`

Actualiza un paquete específico.

```bash
argo update usuario/repo
```

### `argo list`

Lista todos los paquetes instalados en el caché.

```bash
argo list
# usuario/repo 1.3.2 (245.3 KB, 12 archivos)
# otro-usuario/lib 2.0.1 (89.1 KB, 5 archivos)
```

### `argo clean`

Limpia paquetes del caché con más de 30 días sin usar.

```bash
argo clean
# 3 paquetes eliminados del caché.
```

### `argo uninstall <repo>`

Desinstala un paquete y lo elimina del caché.

```bash
argo uninstall usuario/repo
```

---

## Publicación

### `argo publish`

Valida y publica el paquete en GitHub.

```bash
argo publish
```

Validaciones:
- `[package]` completo en `argo.toml`
- Nombre coincide con remote de git
- `src/` contiene archivos `.argo`
- `LICENSE` presente
- Tests pasan

### `argo publish --dry-run`

Solo ejecuta las validaciones sin publicar.

```bash
argo publish --dry-run
```

---

## Configuración

### `argo config set <key> <value>`

Configura tokens de autenticación.

```bash
argo config set github-token ghp_xxxxxxxxxxxx
argo config set gitlab-token glpat-xxxxxxxxxxxx
argo config set bitbucket-token xxxxxxxxxxxx
```

Tokens almacenados en `~/.argo/config.toml`.

---

## Utilidades

### `argo --version`

Muestra la versión actual.

```bash
argo --version
# argo 2.0.0
```
