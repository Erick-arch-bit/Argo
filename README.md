# Argo v1.5.1

Argo es un lenguaje de programación interpretado, minimalista y extensible, escrito en Rust **sin dependencias externas**.

```bash
curl -sSL https://github.com/Erick-arch-bit/Argo-Lang/releases/download/v1.5.1/install.sh | bash
```

## Características

| Área | Detalle |
|------|---------|
| **Tipos** | Entero (`i64`), Flotante (`f64`), Booleano, Cadena, Nulo, Arreglo, Diccionario, Buffer, Función |
| **Structs** | `struct Nombre { campo1, campo2 }` — tipos compuestos con acceso por `.` |
| **Enums** | `enum Nombre { Var1, Var2(campo) }` — variantes con datos opcionales |
| **Match** | `match (valor) { Patron => expr, ... }` — pattern matching con bindings |
| **FFI** | `extern fn nombre(args)` — simulación de funciones nativas (listo para wgpu) |
| **Control** | `if/else`, `while`, `for`, `break`, `try/catch`, `throw` |
| **Variables** | `let` (mutable), `const` (inmutable, verificado en runtime) |
| **Funciones** | Declarativas `fn nombre(){}`, anónimas `fn(){}`, closures con ámbito léxico |
| **Módulos** | `import "ruta"` (local/remoto), `import "https://..."` con caché automática, `import { x } from "mod"` selectivo |
| **Const** | `const PI = 3.1416;` — error si se reasigna |
| **Errores** | `throw "mensaje"` — lanzar errores manuales con traceback |
| **Testing** | `argo test` — ejecuta archivos `*.test.argo` con `assert()` |
| **Iteradores** | `arr.map(fn)`, `.filter()`, `.reduce()`, `.find()`, `.every()`, `.some()` |
| **Concurrencia** | `thread.spawn("código")`, canales `canal.nuevo()` |
| **GPU** | Framebuffer por software: `gpu.crear_buffer`, `.pixel`, `.linea`, `.rect`, `.circulo`, `.guardar`, `.mostrar` |
| **Caché** | AST serializado a `.argbc` — segunda ejecución instantánea |
| **Package manager** | `argo install` — lee `[dependencies]` de `argo.toml` |

## Biblioteca estándar

| Módulo | Funciones |
|--------|-----------|
| `math` | `sin`, `cos`, `tan`, `sqrt`, `abs`, `pow`, `log`, `log10`, `floor`, `ceil`, `round`, `max`, `min`, `random`, `PI`, `E` |
| `str` | `longitud`, `mayusculas`, `minusculas`, `recortar`, `dividir`, `contiene`, `subcadena`, `reemplazar`, `empieza_con`, `termina_con`, `indice_de`, `invertir`, `repetir`, `a_arreglo`, `codigo_en`, `de_codigo` |
| `arr` | `len`, `push`, `pop`, `contiene`, `invertir`, `primero`, `ultimo`, `concatenar`, `vacio`, `indice_de`, `plano` |
| `fs` | `leer`, `escribir`, `existe`, `es_directorio`, `es_archivo`, `eliminar`, `crear_directorio`, `listar` |
| `buffer` | `alloc`, `write`, `read`, `longitud`, `a_cadena`, `de_cadena`, `copiar` |
| `os` | `ejecutar`, `variables`, `directorio_actual`, `directorio_temporal`, `argumentos`, `procesadores` |
| `time` | `ahora`, `dormir`, `segundos`, `micros` |
| `net` | `solicitud` — peticiones HTTP/HTTPS (GET, POST, PUT, DELETE, etc.) |
| `json` | `parsear`, `stringificar` |
| `thread` | `spawn` |
| `gpu` | `crear_buffer`, `pixel`, `linea`, `rect`, `circulo`, `guardar`, `mostrar`, `limpiar` |
| Built-in | `print`, `len`, `push`, `tipo`, `assert`, `typeof` |

## Uso

```
argo                  REPL interactivo
argo <archivo.argo>   Ejecuta un script
argo init             Crea un nuevo proyecto
argo run              Ejecuta el proyecto actual
argo test             Ejecuta pruebas (*.test.argo)
argo install          Instala dependencias desde argo.toml
argo update           Auto-actualiza Argo a la última versión
argo --version        Muestra la versión actual
```

```rust
print("Hola desde Argo v1.5.1");

// Structs y Enums
struct Rect { x, y }
enum Evento { Clic(x, y), Nada }

let r = Rect { x: 10, y: 20 };
let e = Evento::Clic(15, 25);
print(r.x, r.y);

// Match con pattern matching
match (e) {
    Clic(x, y) => print("Click en", x, y),
    Nada => print("Sin evento")
}

// FFI simulado (listo para wgpu)
extern fn render_clear(r, g, b);
render_clear(0.5, 0.5, 1.0);

// Funciones y closures
fn fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
print("fib(10):", fib(10));

// Iteradores funcionales
let nums = [1, 2, 3, 4, 5];
let dobles = arr.map(nums, fn(x) { x * 2 });
let suma = arr.reduce(nums, fn(acc, x) { acc + x }, 0);

// Manejo de errores
try {
    let x = 10 / 0;
} catch (e) {
    print("Error:", e);
}

// Testing
assert(1 + 1 == 2, "matematicas basicas");
```

## Instalación

### Linux / macOS
```bash
curl -sSL https://raw.githubusercontent.com/Erick-arch-bit/Argo-Lang/dev/install.sh | bash
```

### Windows (PowerShell)
```powershell
irm https://raw.githubusercontent.com/Erick-arch-bit/Argo-Lang/dev/install.ps1 | iex
```

### Con Cargo (todas las plataformas)
```bash
cargo install --git https://github.com/Erick-arch-bit/Argo-Lang.git
```

### Descarga directa
Binarios precompilados para todas las plataformas en [Releases](https://github.com/Erick-arch-bit/Argo-Lang/releases).

| Plataforma | Binario |
|------------|---------|
| Linux x64 | `argo-linux-amd64` |
| Linux ARM64 | `argo-linux-arm64` |
| macOS Intel | `argo-darwin-amd64` |
| macOS Apple Silicon | `argo-darwin-arm64` |
| Windows x64 | `argo-windows-amd64.exe` |

## Licencia

MIT — ver [LICENSE](LICENSE).

## Cambios por versión

Ver [CHANGELOG.md](CHANGELOG.md).
