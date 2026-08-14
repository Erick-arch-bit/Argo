# Argo — Referencia Completa del Lenguaje

Versión 2.2.0 — Todo lo que necesita saber sobre el lenguaje Argo.

---

## Tabla de contenidos

1. [Tipos de datos](#tipos-de-datos)
2. [Variables y constantes](#variables-y-constantes)
3. [Operadores](#operadores)
4. [Control de flujo](#control-de-flujo)
5. [Funciones](#funciones)
6. [Structs](#structs)
7. [Enums](#enums)
8. [Pattern matching](#pattern-matching)
9. [Arreglos](#arreglos)
10. [Diccionarios](#diccionarios)
11. [Cadenas](#cadenas)
12. [Manejo de errores](#manejo-de-errores)
13. [Módulos e imports](#módulos-e-imports)
14. [FFI](#ffi)
15. [Concurrencia](#concurrencia)
16. [Buffers binarios](#buffers-binarios)
17. [Literales](#literales)
18. [Verdad y falsedad](#verdad-y-falsedad)
19. [Escape de cadenas](#escape-de-cadenas)
20. [Comentarios](#comentarios)

---

## Tipos de datos

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| `entero` | Entero con signo de 64 bits (`i64`) | `42`, `-7`, `0` |
| `flotante` | Flotante de 64 bits (`f64`) | `3.14`, `-0.5`, `1e10` |
| `booleano` | `true` o `false` | `true`, `false` |
| `cadena` | Texto UTF-8 | `"Hola"`, `""` |
| `nulo` | Ausencia de valor | `null` |
| `arreglo` | Colección ordenada | `[1, 2, 3]` |
| `diccionario` | Mapa clave-valor | `{"a": 1, "b": 2}` |
| `buffer` | Datos binarios | `buffer.alloc(1024)` |
| `funcion` | Función definida por el usuario | `fn(x) { x + 1 }` |

---

## Variables y constantes

### Variables mutables

```argo
let nombre = "Argo";
let edad = 25;
let precio = 9.99;

// Reasignación permitida
nombre = "Argo v2";
```

### Constantes

```argo
const PI = 3.14159;
const MAX = 1000;

// Reasignación genera error en runtime
// PI = 3.0;  // Error!
```

### Reglas de nombres

- Deben empezar con letra o `_`
- Pueden contener letras, números y `_`
- Son case-sensitive (`nombre` ≠ `Nombre`)
- Palabras reservadas no se pueden usar como nombres

---

## Operadores

### Aritméticos

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `+` | Suma / concatenación | `2 + 3` → `5`, `"a" + "b"` → `"ab"` |
| `-` | Resta / negación | `5 - 2` → `3`, `-x` |
| `*` | Multiplicación | `3 * 4` → `12` |
| `/` | División | `10 / 3` → `3.333...` |
| `%` | Módulo | `10 % 3` → `1` |

### Asignación

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `=` | Asignación | `x = 10` |
| `+=` | Suma asignación | `x += 5` (equivale a `x = x + 5`) |
| `-=` | Resta asignación | `x -= 3` (equivale a `x = x - 3`) |

### Comparación

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `==` | Igualdad | `5 == 5` → `true` |
| `!=` | Desigualdad | `5 != 3` → `true` |
| `<` | Menor que | `3 < 5` → `true` |
| `>` | Mayor que | `5 > 3` → `true` |
| `<=` | Menor o igual | `3 <= 3` → `true` |
| `>=` | Mayor o igual | `5 >= 3` → `true` |

### Lógicos

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `&&` | AND (cortocircuito) | `true && false` → `false` |
| `\|\|` | OR (cortocircuito) | `true \|\| false` → `true` |
| `!` | NOT | `!true` → `false` |

### Bitwise

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `&` | AND bit a bit | `0b1010 & 0b1100` → `0b1000` |
| `\|` | OR bit a bit | `0b1010 \| 0b1100` → `0b1110` |
| `^` | XOR bit a bit | `0b1010 ^ 0b1100` → `0b0110` |
| `<<` | Desplazamiento izquierda | `1 << 3` → `8` |
| `>>` | Desplazamiento derecha | `8 >> 2` → `2` |

### Acceso

| Operador | Descripción | Ejemplo |
|----------|-------------|---------|
| `.` | Acceso a propiedad | `punto.x` |
| `[]` | Acceso por índice | `arr[0]`, `dict["key"]` |
| `::` | Namespace de enum | `Color::Rojo` |
| `=>` | Separador de match arm | `5 => "cinco"` |

---

## Control de flujo

### if / else

```argo
let edad = 18;

if (edad >= 18) {
    print("Mayor de edad");
} else {
    print("Menor de edad");
}

// if sin else
if (llueve) {
    print("Lleva paraguas");
}
```

### while

```argo
let i = 0;
while (i < 10) {
    print(i);
    i += 1;
}
```

### for

```argo
// for (inicialización; condición; actualización)
for (let i = 0; i < 5; i += 1) {
    print(i);
}

// Recorrer arreglo
let nums = [10, 20, 30];
for (let i = 0; i < arr.len(nums); i += 1) {
    print(nums[i]);
}
```

### break y continue

```argo
for (let i = 0; i < 100; i += 1) {
    if (i == 5) continue;  // Salta el 5
    if (i == 10) break;    // Termina en 10
    print(i);
}
// Imprime: 0 1 2 3 4 6 7 8 9
```

### return

```argo
fn cuadrado(x) {
    return x * x;
}

fn saludo(nombre) {
    print("Hola " + nombre);
    // return implícito con el último valor
}
```

---

## Funciones

### Declaración

```argo
fn sumar(a, b) {
    return a + b;
}

fn factorial(n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}
```

### Funciones anónimas

```argo
let cuadrado = fn(x) {
    return x * x;
};

print(cuadrado(5));  // 25
```

### Closures

Las funciones capturan el ámbito léxico donde fueron definidas:

```argo
fn crear_contador() {
    let count = 0;
    return fn() {
        count += 1;
        return count;
    };
}

let contador = crear_contador();
print(contador());  // 1
print(contador());  // 2
```

### Parámetros

```argo
// Sin parámetros
fn hola() {
    print("Hola");
}

// Un parámetro
fn doble(x) {
    return x * 2;
}

// Múltiples parámetros
fn pow(base, exp) {
    let resultado = 1;
    for (let i = 0; i < exp; i += 1) {
        resultado *= base;
    }
    return resultado;
}
```

---

## Structs

### Definición

```argo
struct Punto {
    x,
    y
}

struct Persona {
    nombre,
    edad,
    activo
}
```

### Instanciación

```argo
let p = Punto { x: 10, y: 20 };
let persona = Persona { nombre: "Ana", edad: 25, activo: true };
```

### Acceso a campos

```argo
print(p.x);        // 10
print(persona.nombre);  // "Ana"
```

### Modificación de campos

```argo
p.x = 15;
persona.edad = 26;
```

### Structs anidados

```argo
struct Rect {
    x,
    y,
    ancho,
    alto
}

struct Sprite {
    posicion,
    rect
}

let sprite = Sprite {
    posicion: Punto { x: 0, y: 0 },
    rect: Rect { x: 0, y: 0, ancho: 32, alto: 32 }
};

print(sprite.posicion.x);  // 0
```

---

## Enums

### Definición

```argo
enum Color {
    Rojo,
    Verde,
    Azul
}

enum Forma {
    Circulo(radio),
    Rectangulo(ancho, alto),
    Punto(x, y)
}
```

### Instanciación

```argo
let c = Color::Rojo;
let forma1 = Forma::Circulo(50);
let forma2 = Forma::Rectangulo(100, 200);
let forma3 = Forma::Punto(10, 20);
```

### Acceso a campos de variante

```argo
match (forma1) {
    Circulo(radio) => print("Radio:", radio),
    Rectangulo(ancho, alto) => print(ancho, "x", alto),
    Punto(x, y) => print("Punto en", x, y)
}
```

---

## Pattern matching

### Sintaxis básica

```argo
match (valor) {
    patron1 => expresion1,
    patron2 => expresion2,
    _ => default
}
```

### Patrones soportados

#### Literales

```argo
let x = 5;
match (x) {
    1 => print("uno"),
    2 => print("dos"),
    5 => print("cinco"),
    _ => print("otro")
}
```

#### Wildcard (_)

```argo
match (color) {
    Color::Rojo => print("Rojo"),
    _ => print("Otro color")
}
```

#### Bindings

```argo
match (forma) {
    Forma::Circulo(r) => print("Círculo con radio", r),
    Forma::Rectangulo(w, h) => print("Rect de", w, "x", h),
    _ => print("Desconocida")
}
```

#### Struct destructuring

```argo
struct Persona { nombre, edad }

let p = Persona { nombre: "Ana", edad: 25 };

match (p) {
    Persona { nombre, edad } if edad >= 18 => print(nombre, "es mayor"),
    Persona { nombre, _ } => print(nombre, "es menor")
}
```

#### Array destructuring

```argo
let punto = [10, 20];

match (punto) {
    [0, 0] => print("Origen"),
    [x, 0] => print("Eje X en", x),
    [0, y] => print("Eje Y en", y),
    [x, y] => print("Punto en", x, y)
}
```

#### Enum destructuring

```argo
match (evento) {
    Evento::Clic(x, y) => print("Click en", x, y),
    Evento::Tecla(codigo) => print("Tecla", codigo),
    Evento::Nada => print("Sin evento")
}
```

---

## Arreglos

### Creación

```argo
let nums = [1, 2, 3, 4, 5];
let vacio = [];
let mixto = [1, "dos", true, null];
```

### Acceso por índice

```argo
let arr = [10, 20, 30];
print(arr[0]);   // 10
print(arr[2]);   // 30
arr[1] = 25;     // Modificar
```

### Longitud

```argo
let arr = [1, 2, 3];
print(arr.len(arr));  // 3
```

### Métodos del módulo `arr`

| Función | Descripción |
|---------|-------------|
| `arr.len(arr)` | Longitud del arreglo |
| `arr.push(arr, elem)` | Agrega elemento al final |
| `arr.pop(arr)` | Elimina y retorna el último elemento |
| `arr.contiene(arr, elem)` | Verifica si contiene el elemento |
| `arr.invertir(arr)` | Invierte el arreglo |
| `arr.primero(arr)` | Primer elemento |
| `arr.ultimo(arr)` | Último elemento |
| `arr.concatenar(arr1, arr2)` | Concatena dos arreglos |
| `arr.vacio(arr)` | Verifica si está vacío |
| `arr.indice_de(arr, elem)` | Índice del elemento (-1 si no existe) |
| `arr.plano(arr)` | Aplana un nivel de anidamiento |
| `arr.map(arr, fn)` | Aplica función a cada elemento |
| `arr.filter(arr, fn)` | Filtra por predicado |
| `arr.reduce(arr, fn, init)` | Reduce a un solo valor |
| `arr.find(arr, fn)` | Primer elemento que cumple |
| `arr.every(arr, fn)` | Todos cumplen el predicado |
| `arr.some(arr, fn)` | Al menos uno cumple |

### Ejemplos

```argo
let nums = [1, 2, 3, 4, 5];

// Map
let dobles = arr.map(nums, fn(x) { x * 2 });
// [2, 4, 6, 8, 10]

// Filter
let pares = arr.filter(nums, fn(x) { x % 2 == 0 });
// [2, 4]

// Reduce
let suma = arr.reduce(nums, fn(acc, x) { acc + x }, 0);
// 15

// Find
let mayor = arr.find(nums, fn(x) { x > 3 });
// 4

// every / some
let todos_pos = arr.every(nums, fn(x) { x > 0 });  // true
let hay_neg = arr.some(nums, fn(x) { x < 0 });     // false
```

---

## Diccionarios

### Creación

```argo
let persona = {
    "nombre": "Ana",
    "edad": 25,
    "activo": true
};

let vacio = {};
```

### Acceso

```argo
print(persona["nombre"]);  // "Ana"
print(persona.edad);       // 25 (acceso por punto)
```

### Modificación

```argo
persona["email"] = "ana@ejemplo.com";
persona.edad = 26;
```

### Prototypal inheritance

Los diccionarios soportan cadena de prototipos:

```argo
let base = { "color": "rojo" };
let objeto = { "__proto__": base, "nombre": "test" };

print(objeto.color);   // "rojo" (heredado de base)
print(objeto.nombre);  // "test"
```

---

## Cadenas

### Literales

```argo
let s1 = "Hola mundo";
let s2 = "";
let s3 = "Línea 1\nLínea 2";  // Con salto de línea
```

### Concatenación

```argo
let nombre = "Argo";
let saludo = "Hola " + nombre + "!";
print(saludo);  // "Hola Argo!"
```

### Métodos del módulo `str`

| Función | Descripción |
|---------|-------------|
| `str.longitud(s)` | Longitud de la cadena |
| `str.mayusculas(s)` | Convierte a mayúsculas |
| `str.minusculas(s)` | Convierte a minúsculas |
| `str.recortar(s)` | Elimina espacios al inicio y fin |
| `str.dividir(s, sep)` | Divide por separador |
| `str.contiene(s, sub)` | Verifica si contiene subcadena |
| `str.subcadena(s, inicio, fin)` | Extrae subcadena |
| `str.reemplazar(s, objetivo, reemplazo)` | Reemplaza todas las ocurrencias |
| `str.empieza_con(s, prefijo)` | Comienza con prefijo |
| `str.termina_con(s, sufijo)` | Termina con sufijo |
| `str.indice_de(s, sub, inicio)` | Índice de subcadena (-1 si no existe) |
| `str.invertir(s)` | Invierte la cadena |
| `str.repetir(s, veces)` | Repite N veces |
| `str.a_arreglo(s)` | Convierte a arreglo de caracteres |
| `str.codigo_en(s, indice)` | Código Unicode en índice |
| `str.de_codigo(codigo)` | Carácter desde código Unicode |

---

## Manejo de errores

### try / catch

```argo
try {
    let resultado = 10 / 0;
} catch (e) {
    print("Error:", e);
}
```

### throw

```argo
fn dividir(a, b) {
    if (b == 0) {
        throw "División por cero";
    }
    return a / b;
}

try {
    let r = dividir(10, 0);
} catch (e) {
    print(e);  // "División por cero"
}
```

### Tipos de errores

- Errores de runtime (división por cero, índice fuera de rango, etc.)
- Errores lanzados con `throw`
- Excepciones con traceback completo

---

## Módulos e imports

### Import completo

```argo
import "math";
print(math.sin(3.14));
```

### Import selectivo

```argo
import { sin, cos, PI } from "math";
print(sin(PI / 2));
```

### Imports locales

```argo
import "utils/funciones.argo";
import { helper } from "utils/helper.argo";
```

### Imports remotos (URL)

```argo
import "https://raw.githubusercontent.com/usuario/repo/main/lib.argo";
```

### Caché de módulos

- Primera ejecución: descarga y cachea
- Segunda ejecución: usa caché binario (`.argbc`)
- Actualización: `argo update` fuerza resolución

---

## FFI

### Declaración

```argo
extern fn render_clear(r, g, b);
extern fn render_pixel(x, y, r, g, b);
```

### Uso

```argo
render_clear(0.5, 0.5, 1.0);
render_pixel(100, 100, 255, 0, 0);
```

> Nota: FFI está preparado para integración con wgpu y otros backends nativos.

---

## Concurrencia

### Canales

```argo
let ch = canal();

// Enviar valores
enviar(ch, 42);
enviar(ch, "hello");

// Recibir valores (bloqueante)
let valor = recibir(ch);
print(valor);  // 42
```

### Hilos

```argo
thread.spawn('
    for (let i = 0; i < 5; i += 1) {
        print("Hilo:", i);
    }
');
```

---

## Buffers binarios

### Creación

```argo
let buf = buffer.alloc(1024);  // 1024 bytes en cero
```

### Operaciones

```argo
buffer.write(buf, 0, 65);     // Escribir byte en índice 0
let byte = buffer.read(buf, 0);  // Leer byte
let len = buffer.longitud(buf);   // Longitud en bytes
let str = buffer.a_cadena(buf);   // Convertir a cadena
let buf2 = buffer.de_cadena("Hola");  // Cadena a buffer
buffer.copiar(origen, destino, 0);  // Copiar buffers
```

---

## Literales

### Enteros

```argo
let a = 42;
let b = 0xFF;      // Hexadecimal
let c = 0b1010;    // Binario
let d = 0o77;      // Octal
```

### Flotantes

```argo
let pi = 3.14159;
let科学 = 1.5e10;  // Notación científica
```

### Cadenas

```argo
let s1 = "Hola";
let s2 = "Línea 1\nLínea 2";
let s3 = "Tab\there";
let s4 = "Comillas \"dobles\"";
let s5 = "Backslash \\";
```

### Booleanos

```argo
let verdadero = true;
let falso = false;
```

### Arreglos

```argo
let nums = [1, 2, 3];
let mixto = [1, "dos", true, null];
let vacio = [];
```

### Diccionarios

```argo
let persona = {"nombre": "Ana", "edad": 25};
let vacio = {};
```

---

## Verdad y falsedad

Son **falsos**:
- `null`
- `false`
- `0` (entero cero)
- `0.0` (flotante cero)
- `""` (cadena vacía)
- `[]` (arreglo vacío)
- `{}` (diccionario vacío)
- `buffer.alloc(0)` (buffer vacío)

Todo lo demás es **verdadero**.

---

## Escape de cadenas

| Secuencia | Significado |
|-----------|-------------|
| `\"` | Comilla doble |
| `\\` | Backslash |
| `\n` | Salto de línea |
| `\r` | Retorno de carro |
| `\t` | Tabulación |

---

## Comentarios

```argo
// Esto es un comentario de una línea

// No hay comentarios multilínea
// pero se pueden usar varias líneas //
```

---

## Resumen de características

| Característica | Estado |
|----------------|--------|
| Variables `let` / `const` | ✅ |
| Funciones declarativas | ✅ |
| Funciones anónimas | ✅ |
| Closures | ✅ |
| Structs | ✅ |
| Enums con datos | ✅ |
| Pattern matching | ✅ |
| Arreglos | ✅ |
| Diccionarios | ✅ |
| Prototypal inheritance | ✅ |
| Manejo de errores | ✅ |
| Módulos e imports | ✅ |
| FFI | ✅ |
| Concurrencia (canales + hilos) | ✅ |
| Buffers binarios | ✅ |
| Operadores bitwise | ✅ |
| Tipado dinámico | ✅ |
| Traceback completo | ✅ |
| Caché de AST | ✅ |
| Zero dependencias externas | ✅ |
