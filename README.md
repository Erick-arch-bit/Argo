# Argo v1.3.0

Argo es un lenguaje de programación interpretado, minimalista y extensible, escrito en Rust **sin dependencias externas**.

```bash
curl -sSL https://github.com/Erick-arch-bit/Argo-Lang/releases/download/v1.3.0/install.sh | bash
```

## Características

| Área | Detalle |
|------|---------|
| **Tipos** | Entero (`i64`), Flotante (`f64`), Booleano, Cadena, Nulo, Arreglo, Diccionario, Buffer, Función |
| **Control** | `if/else`, `while`, `for`, `break`, `try/catch` |
| **Variables** | `let` (mutable), `const` (inmutable, verificado en runtime) |
| **Funciones** | Declarativas `fn nombre(){}`, anónimas `fn(){}`, closures con ámbito léxico |
| **Módulos** | `import "ruta"` (local/remoto), `import "https://..."` con caché automática |
| **Const** | `const PI = 3.1416;` — error si se reasigna |
| **Concurrencia** | `thread.spawn("código")` — hilos aislados |
| **GPU** | Framebuffer por software: `gpu.crear_buffer`, `.pixel`, `.linea`, `.rect`, `.circulo`, `.guardar`, `.mostrar` |
| **Caché** | AST serializado a `.argbc` — segunda ejecución instantánea |

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
| Built-in | `print`, `len`, `push`, `tipo` |

## Uso

```
argo                  REPL interactivo
argo <archivo.argo>   Ejecuta un script
argo init             Crea un nuevo proyecto
argo run              Ejecuta el proyecto actual
```

```rust
print("Hola desde Argo v1.3.0");

// Constante inmutable
const PI = 3.1416;
let radio = 5;
print("Área:", PI * radio * radio);

// Fibonnaci recursivo
fn fib(n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}
print("fib(10):", fib(10));

// GPU — framebuffer por software
let fb = gpu.crear_buffer(200, 100);
fb = gpu.limpiar(fb, 20, 20, 80);
fb = gpu.circulo(fb, 100, 50, 40, 255, 200, 0);
gpu.guardar(fb, "output.ppm");

// Módulos
let lib = import "lib.argo";
let remoto = import "https://ejemplo.com/mod.argo";

// Manejo de errores
try {
    let x = 10 / 0;
} catch (e) {
    print("Error:", e);
}
```

## Instalación

### Script automático (recomendado)
```bash
curl -sSL https://github.com/Erick-arch-bit/Argo-Lang/releases/download/v1.3.0/install.sh | bash
```
Detecta SO/arquitectura (linux-amd64, linux-arm64, darwin-amd64, darwin-arm64) y descarga el binario precompilado. Si falla, compila desde fuente.

### Compilación manual
```bash
git clone <repo-url>
cd argo
cargo build --release
cp target/release/argo /usr/local/bin/
```

## Licencia

MIT — ver [LICENSE](LICENSE).

## Cambios por versión

Ver [CHANGELOG.md](CHANGELOG.md).
