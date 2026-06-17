# Argo v1.0.0

Argo es un lenguaje de programación interpretado, minimalista y extensible, escrito en Rust sin dependencias externas. Su sintaxis combina lo mejor de C y Rust con tipado dinámico.

## Características

### Tipos y valores
- **Enteros** (`i64`), **flotantes** (`f64`), **booleanos**, **cadenas**, **nulo**
- **Arreglos** heterogéneos: `[1, "dos", true]`
- **Diccionarios** con claves string/integer/boolean: `{nombre: "Argo", version: 1}`
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

### Módulos
- `import "ruta"` — ejecuta otro archivo y retorna un diccionario con sus variables exportadas

### Biblioteca estándar
- `print(...)` — imprime argumentos separados por espacio
- `len(obj)` — longitud de string, arreglo o diccionario
- `push(arr, elem)` — retorna nuevo arreglo con el elemento añadido
- `tipo(obj)` — retorna el nombre del tipo como string

### Módulo matemático (`math`)
- `math.sin(x)`, `math.cos(x)`, `math.sqrt(x)`, `math.abs(x)`
- `math.PI`, `math.E`

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

```bash
git clone <url-del-repo>
cd argo
cargo build --release
sudo cp target/release/argo /usr/local/bin/
```

O con el script de instalación (auto-agrega `~/.local/bin` al PATH):

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
print("Hola desde Argo v1.0.0");

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
let usuario = {"nombre": "Argo", "version": 1.0};
let nums = [1, 2, 3, 4];
push(nums, 5);

// Manejo de errores
try {
    let x = 10 / 0;
} catch (e) {
    print("Error atrapado:", e);
}

// Modulos
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
