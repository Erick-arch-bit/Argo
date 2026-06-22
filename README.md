# Argo v1.2.0

Argo es un lenguaje de programación interpretado, minimalista y extensible, escrito en Rust sin dependencias externas. Su sintaxis combina lo mejor de C y Rust con tipado dinámico.

## Características

### Tipos y valores
- **Enteros** (`i64`), **flotantes** (`f64`), **booleanos**, **cadenas**, **nulo**
- **Arreglos** heterogéneos: `[1, "dos", true]`
- **Diccionarios** con claves string/integer/boolean: `{nombre: "Argo", version: 1}`
- **Buffer** de bytes: `buffer.alloc(256)` — memoria binaria contigua
- **Funciones** como ciudadanos de primera clase (closures)

### Operadores
- Aritméticos: `+`, `-`, `*`, `/`, `%`
- Relacionales: `==`, `!=`, `<`, `>`, `<=`, `>=`
- Lógicos: `!` (not unario)
- Concatenación string + número (coerción automática)

### Control de flujo
- `if` / `else` / `else if`
- `while` y `for` (estilo C)
- `break` para salir de bucles
- `try` / `catch` para manejo de errores

### Funciones
- Declaración con `fn`: `fn suma(a, b) { return a + b; }`
- Anónimas (closures): `fn(x, y) { x + y }`
- Ámbito léxico con cadenas de entornos

### Variables
- Declaración: `let x = 10;`
- Asignación: `x = 20;` (busca en la cadena de ámbitos)

### Módulos e importación
- `import "ruta"` — ejecuta otro archivo y retorna un diccionario con sus variables exportadas
- `import "https://..."` — importación remota con caché automática de dos niveles (`.argbc` binario + `.argo` textual)
- Resolución descentralizada: cualquier URL pública es un módulo válido

### Biblioteca estándar
- `print(...)` — imprime argumentos separados por espacio
- `len(obj)` — longitud de string, arreglo, buffer o diccionario
- `push(arr, elem)` — retorna nuevo arreglo con el elemento añadido
- `tipo(obj)` — retorna el nombre del tipo como string

#### Módulos nativos

| Módulo | Funciones | Descripción |
|--------|-----------|-------------|
| `math` | `sin`, `cos`, `sqrt`, `abs`, `random`, `PI`, `E` | Operaciones matemáticas |
| `fs` | `leer`, `escribir` | Sistema de archivos |
| `net` | `solicitud` | Peticiones HTTP |
| `json` | `parsear`, `stringificar` | Serialización JSON |
| `time` | `ahora`, `dormir` | Temporización |
| `os` | `ejecutar`, `variables` | Sistema operativo |
| `str` | `longitud`, `mayusculas`, `minusculas`, `recortar`, `dividir`, `contiene`, `subcadena`, `reemplazar`, `empieza_con`, `termina_con` | Manipulación de cadenas |
| `arr` | `len`, `push`, `pop` | Manipulación de arreglos |
| `buffer` | `alloc`, `write`, `read` | Memoria binaria contigua |
| `thread` | `spawn` | Ejecución en segundo plano |

### Caché binaria de AST (`.argbc`)
- Tras el primer parseo, el AST se serializa a disco en formato binario
- Las ejecuciones posteriores cargan el AST directamente desde `.argbc` — **sin lexer ni parser**
- Soporte para módulos locales y remotos

### Comentarios
- Línea: `// comentario`
- Bloque: `/* comentario */`

### Otras características
- REPL interactivo
- Sistema de proyectos (`argo init` / `argo.toml` / `argo run`)
- Parser tolerante a errores (acumula errores sintácticos sin panic)
- Comas finales toleradas en arreglos, diccionarios, parámetros y argumentos
- Paréntesis opcionales en `if`, `while`, `for` y `catch`

## Instalación

### Descarga directa (recomendado)
Descarga el binario precompilado para tu plataforma desde [GitHub Releases](https://github.com/Erick-arch-bit/Argo/releases):

```bash
# Linux amd64
curl -sSL https://github.com/Erick-arch-bit/Argo/releases/download/v1.2.0/argo-linux-amd64 -o argo
chmod +x argo
sudo mv argo /usr/local/bin/
```

Cada release incluye checksums SHA256 para verificar la integridad de los binarios.

### Compilación desde fuente
```bash
git clone <url-del-repo>
cd argo
cargo build --release
sudo cp target/release/argo /usr/local/bin/
```

### Script de instalación
```bash
chmod +x install.sh
./install.sh
```

## Uso

```
argo                  REPL interactivo
argo <archivo.argo>   Ejecuta un script
argo init             Crea un nuevo proyecto
argo run              Ejecuta el proyecto actual
argo repl             Inicia el REPL explícitamente
```

### Ejemplo

```rust
// Hola mundo
print("Hola desde Argo v1.2.0");

// Variables
let radio = 5;
const PI = 3.1416;
let area = PI * radio * radio;
print("Área:", area);

// Funcion recursiva
fn fib(n) {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
}
print("fib(10):", fib(10));

// Diccionarios y arreglos
let usuario = {"nombre": "Argo", "version": 1.2};
let nums = [1, 2, 3, 4];
push(nums, 5);

// Buffer de bytes
let buf = buffer.alloc(4);
buffer.write(buf, 0, 0x41);
buffer.write(buf, 1, 0x72);
buffer.write(buf, 2, 0x67);
buffer.write(buf, 3, 0x6f);
print("Buffer:", buf);
print("Byte 0:", buffer.read(buf, 0));

// Hilo en segundo plano
thread.spawn("print(\"Ejecutándose en paralelo\")");
print("Esto se imprime inmediatamente");

// Importación remota con caché
let lib = import "https://ejemplo.com/lib.argo";
print(lib.mi_variable);

// Manejo de errores
try {
    let x = 10 / 0;
} catch (e) {
    print("Error atrapado:", e);
}

// Modulos locales
let m = import "milib.argo";
print(m.variable);

// Math
print(math.sqrt(144), math.PI);
```

## Licencia

Argo se distribuye bajo la licencia **MIT**. Ver [LICENSE](LICENSE).

## Contribuir

Lee [CONTRIBUTING.md](CONTRIBUTING.md) para conocer el flujo de trabajo.

## Seguridad

Reporta vulnerabilidades siguiendo el proceso descrito en [SECURITY.md](SECURITY.md).
