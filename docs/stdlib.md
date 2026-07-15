# Argo — Biblioteca Estándar (Stdlib)

Referencia completa de todos los módulos de la biblioteca estándar.

---

## Tabla de contenidos

1. [Built-in globales](#built-in-globales)
2. [math](#math)
3. [str](#str)
4. [arr](#arr)
5. [fs](#fs)
6. [buffer](#buffer)
7. [os](#os)
8. [time](#time)
9. [net](#net)
10. [json](#json)
11. [thread](#thread)
12. [gpu](#gpu)
13. [ui](#ui)

---

## Built-in globales

Funciones disponibles en todo el programa sin import.

| Función | Descripción |
|---------|-------------|
| `print(...)` | Imprime valores separados por espacio, con salto de línea |
| `len(x)` | Longitud de cadena, arreglo o diccionario |
| `push(arr, elem)` | Agrega elemento al arreglo |
| `tipo(x)` | Retorna el tipo como cadena (`"entero"`, `"cadena"`, etc.) |
| `entero(x)` | Convierte a entero |
| `cadena(x)` | Convierte a cadena |
| `assert(cond, msg?)` | Verifica condición, error con mensaje si falla |
| `canal()` | Crea un canal de comunicación |
| `enviar(ch, val)` | Envía valor al canal |
| `recibir(ch)` | Recibe valor del canal (bloqueante) |

### Ejemplos

```argo
// print
print("Hola", "mundo", 42);  // Hola mundo 42

// len
print(len("Argo"));    // 4
print(len([1,2,3]));   // 3
print(len({"a": 1}));  // 1

// tipo
print(tipo(42));        // "entero"
print(tipo("hola"));   // "cadena"
print(tipo([1,2]));    // "arreglo"
print(tipo(null));     // "nulo"

// assert
assert(1 + 1 == 2, "matemática básica");
assert(len("hello") == 5);

// canales
let ch = canal();
enviar(ch, 42);
let val = recibir(ch);
print(val);  // 42
```

---

## math

Funciones matemáticas. Se accede con `math.funcion()`.

| Función | Descripción |
|---------|-------------|
| `math.sin(x)` | Seno |
| `math.cos(x)` | Coseno |
| `math.tan(x)` | Tangente |
| `math.sqrt(x)` | Raíz cuadrada |
| `math.abs(x)` | Valor absoluto |
| `math.pow(base, exp)` | Potencia |
| `math.log(x)` | Logaritmo natural |
| `math.log10(x)` | Logaritmo base 10 |
| `math.floor(x)` | Redondeo hacia abajo |
| `math.ceil(x)` | Redondeo hacia arriba |
| `math.round(x)` | Redondeo al entero más cercano |
| `math.max(a, b)` | Máximo de dos valores |
| `math.min(a, b)` | Mínimo de dos valores |
| `math.random(min, max)` | Entero aleatorio en [min, max] |

### Constantes

| Constante | Valor |
|-----------|-------|
| `math.PI` | 3.141592653589793 |
| `math.E` | 2.718281828459045 |

### Ejemplos

```argo
import "math";

print(math.sin(math.PI / 2));   // 1.0
print(math.sqrt(144));          // 12.0
print(math.pow(2, 10));         // 1024.0
print(math.abs(-42));           // 42.0
print(math.floor(3.7));         // 3.0
print(math.ceil(3.2));          // 4.0
print(math.round(3.5));         // 4.0
print(math.random(1, 100));     // Número aleatorio entre 1 y 100
print(math.PI);                 // 3.141592653589793
print(math.E);                  // 2.718281828459045
```

---

## str

Manipulación de cadenas. Se accede con `str.funcion()`.

| Función | Descripción |
|---------|-------------|
| `str.longitud(s)` | Longitud de la cadena |
| `str.mayusculas(s)` | Convierte a mayúsculas |
| `str.minusculas(s)` | Convierte a minúsculas |
| `str.recortar(s)` | Elimina espacios al inicio y fin |
| `str.dividir(s, sep)` | Divide por separador, retorna arreglo |
| `str.contiene(s, sub)` | Verifica si contiene subcadena |
| `str.subcadena(s, inicio, fin?)` | Extrae subcadena (fin es exclusivo) |
| `str.reemplazar(s, viejo, nuevo)` | Reemplaza todas las ocurrencias |
| `str.empieza_con(s, prefijo)` | Comienza con prefijo |
| `str.termina_con(s, sufijo)` | Termina con sufijo |
| `str.indice_de(s, sub, inicio?)` | Índice de subcadena (-1 si no existe) |
| `str.invertir(s)` | Invierte la cadena |
| `str.repetir(s, veces)` | Repite N veces |
| `str.a_arreglo(s)` | Convierte a arreglo de caracteres |
| `str.codigo_en(s, indice)` | Código Unicode en índice |
| `str.de_codigo(codigo)` | Carácter desde código Unicode |

### Ejemplos

```argo
import "str";

let s = "  Hola Mundo  ";

print(str.longitud(s));          // 14
print(str.mayusculas("hello")); // "HELLO"
print(str.minusculas("HELLO")); // "hello"
print(str.recortar(s));         // "Hola Mundo"
print(str.dividir("a,b,c", ",")); // ["a", "b", "c"]
print(str.contiene("hello", "ell")); // true
print(str.subcadena("hello", 1, 4)); // "ell"
print(str.reemplazar("aabb", "a", "x")); // "xxbb"
print(str.empieza_con("hello", "he")); // true
print(str.termina_con("hello", "lo")); // true
print(str.indice_de("hello", "ll"));   // 2
print(str.invertir("hello"));  // "olleh"
print(str.repetir("ab", 3));   // "ababab"
print(str.a_arreglo("hi"));    // ["h", "i"]
print(str.codigo_en("A", 0));  // 65
print(str.de_codigo(65));      // "A"
```

---

## arr

Operaciones con arreglos. Se accede con `arr.funcion()`.

| Función | Descripción |
|---------|-------------|
| `arr.len(arr)` | Longitud del arreglo |
| `arr.push(arr, elem)` | Agrega elemento al final |
| `arr.pop(arr)` | Elimina y retorna último elemento |
| `arr.contiene(arr, elem)` | Verifica si contiene elemento |
| `arr.invertir(arr)` | Invierte el arreglo |
| `arr.primero(arr)` | Primer elemento |
| `arr.ultimo(arr)` | Último elemento |
| `arr.concatenar(arr1, arr2)` | Concatena dos arreglos |
| `arr.vacio(arr)` | Verifica si está vacío |
| `arr.indice_de(arr, elem)` | Índice del elemento (-1 si no existe) |
| `arr.plano(arr)` | Aplana un nivel de anidamiento |
| `arr.map(arr, fn)` | Aplica función a cada elemento |
| `arr.filter(arr, fn)` | Filtra por predicado |
| `arr.reduce(arr, fn, init?)` | Reduce a un solo valor |
| `arr.find(arr, fn)` | Primer elemento que cumple |
| `arr.every(arr, fn)` | Todos cumplen el predicado |
| `arr.some(arr, fn)` | Al menos uno cumple |

### Ejemplos

```argo
import "arr";

let nums = [3, 1, 4, 1, 5, 9];

print(arr.len(nums));           // 6
print(arr.push(nums, 2));       // [3,1,4,1,5,9,2]
print(arr.contiene(nums, 5));   // true
print(arr.invertir(nums));      // [9,5,1,4,1,3]
print(arr.primero(nums));       // 3
print(arr.ultimo(nums));        // 9
print(arr.vacio([]));           // true
print(arr.indice_de(nums, 4));  // 2
print(arr.plano([[1,2],[3,4]])); // [1,2,3,4]

// Funciones de orden superior
let dobles = arr.map(nums, fn(x) { x * 2 });
let pares = arr.filter(nums, fn(x) { x % 2 == 0 });
let suma = arr.reduce(nums, fn(acc, x) { acc + x }, 0);
let mayor = arr.find(nums, fn(x) { x > 8 });
let todos = arr.every([1,2,3], fn(x) { x > 0 });  // true
let hay = arr.some([1,2,3], fn(x) { x > 2 });     // true
```

---

## fs

Sistema de archivos. Se accede con `fs.funcion()`.

| Función | Descripción |
|---------|-------------|
| `fs.leer(ruta)` | Lee contenido del archivo |
| `fs.escribir(ruta, contenido)` | Escribe contenido al archivo |
| `fs.existe(ruta)` | Verifica si la ruta existe |
| `fs.es_directorio(ruta)` | Verifica si es directorio |
| `fs.es_archivo(ruta)` | Verifica si es archivo |
| `fs.eliminar(ruta)` | Elimina archivo o directorio |
| `fs.crear_directorio(ruta)` | Crea directorios (recursivo) |
| `fs.listar(ruta)` | Lista entradas del directorio |

### Ejemplos

```argo
import "fs";

// Leer archivo
let contenido = fs.leer("datos.txt");
print(contenido);

// Escribir archivo
fs.escribir("salida.txt", "Hola mundo");

// Verificar existencia
if (fs.existe("config.json")) {
    let config = fs.leer("config.json");
}

// Crear directorios
fs.crear_directorio("src/components");

// Listar directorio
let archivos = fs.listar(".");
for (let i = 0; i < arr.len(archivos); i += 1) {
    print(archivos[i]);
}

// Eliminar
fs.eliminar("temp.txt");
```

---

## buffer

Buffers binarios. Se accede con `buffer.funcion()`.

| Función | Descripción |
|---------|-------------|
| `buffer.alloc(tamano)` | Allocar buffer de N bytes (máx 1GB) |
| `buffer.write(buf, indice, valor)` | Escribir byte en índice |
| `buffer.read(buf, indice)` | Leer byte en índice |
| `buffer.longitud(buf)` | Longitud en bytes |
| `buffer.a_cadena(buf)` | Convertir a cadena UTF-8 |
| `buffer.de_cadena(s)` | Convertir cadena a buffer |
| `buffer.copiar(origen, destino, pos)` | Copiar origen en destino |

### Ejemplos

```argo
import "buffer";

// Crear buffer
let buf = buffer.alloc(256);

// Escribir datos
buffer.write(buf, 0, 72);   // 'H'
buffer.write(buf, 1, 105);  // 'i'

// Leer datos
print(buffer.read(buf, 0));  // 72

// Longitud
print(buffer.longitud(buf)); // 256

// Convertir
let buf2 = buffer.de_cadena("Hola");
let texto = buffer.a_cadena(buf2);
print(texto);  // "Hola"

// Copiar
let destino = buffer.alloc(256);
buffer.copiar(buf2, destino, 0);
```

---

## os

Interfaz con el sistema operativo. Se accede con `os.funcion()`.

| Función | Descripción |
|---------|-------------|
| `os.ejecutar(comando)` | Ejecuta comando del sistema, retorna stdout |
| `os.variables(llave)` | Obtiene variable de entorno |
| `os.exit(codigo)` | Termina el proceso con código |
| `os.directorio_actual()` | Directorio de trabajo actual |
| `os.directorio_temporal()` | Directorio temporal del sistema |
| `os.argumentos()` | Argumentos de línea de comandos |
| `os.procesadores()` | Número de CPUs disponibles |

### Ejemplos

```argo
import "os";

// Ejecutar comando
let salida = os.ejecutar("ls -la");
print(salida);

// Variable de entorno
let home = os.variables("HOME");
print(home);

// Directorio actual
print(os.directorio_actual());

// Directorio temporal
let tmp = os.directorio_temporal();

// Argumentos CLI
let args = os.argumentos();
print(arr.len(args));

// CPUs disponibles
print(os.procesadores());

// Salir
os.exit(0);
```

---

## time

Funciones de tiempo. Se accede con `time.funcion()`.

| Función | Descripción |
|---------|-------------|
| `time.ahora()` | Tiempo actual en milisegundos desde epoch |
| `time.dormir(ms)` | Duerme N milisegundos |
| `time.segundos()` | Tiempo actual en segundos desde epoch |
| `time.micros()` | Tiempo actual en microsegundos desde epoch |

### Ejemplos

```argo
import "time";

// Tiempo actual
let ms = time.ahora();
let secs = time.segundos();
let微秒 = time.micros();

print("Milisegundos:", ms);
print("Segundos:", secs);

// Medir tiempo
let inicio = time.ahora();
// ... código ...
let fin = time.ahora();
print("Tardó:", fin - inicio, "ms");

// Dormir
print("Esperando...");
time.dormir(1000);  // 1 segundo
print("¡Despierto!");
```

---

## net

Peticiones HTTP. Se accede con `net.funcion()`.

| Función | Descripción |
|---------|-------------|
| `net.solicitud(metodo, host:puerto, cuerpo?)` | Realiza petición HTTP |

### Métodos soportados

- `GET` — Obtener recurso
- `POST` — Crear recurso
- `PUT` — Actualizar recurso
- `PATCH` — Actualizar parcialmente
- `DELETE` — Eliminar recurso

### Ejemplos

```argo
import "net";

// GET
let respuesta = net.solicitud("GET", "http://httpbin.org/get");
print(respuesta);

// POST con cuerpo
let body = '{"nombre": "Argo"}';
let resultado = net.solicitud("POST", "http://httpbin.org/post", body);
print(resultado);

// API REST
let users = net.solicitud("GET", "https://api.example.com/users");
let data = json.parsear(users);
```

---

## json

Parseo y serialización JSON. Se accede con `json.funcion()`.

| Función | Descripción |
|---------|-------------|
| `json.parsear(cadena)` | Convierte cadena JSON a objeto Argo |
| `json.stringificar(objeto)` | Convierte objeto Argo a cadena JSON |

### Ejemplos

```argo
import "json";

// Parsear JSON
let json_str = '{"nombre": "Ana", "edad": 25, "activo": true}';
let persona = json.parsear(json_str);
print(persona.nombre);  // "Ana"
print(persona.edad);    // 25

// Stringificar
let objeto = {
    "nombre": "Argo",
    "version": "2.0.0",
    "features": ["fast", "safe"]
};
let json_salida = json.stringificar(objeto);
print(json_salida);

// JSON con arreglos
let arr_json = '[1, 2, 3, "cuatro"]';
let arr = json.parsear(arr_json);
print(arr.len(arr));  // 4
```

---

## thread

Concurrencia con hilos. Se accede con `thread.funcion()`.

| Función | Descripción |
|---------|-------------|
| `thread.spawn(codigo)` | Crea hilo que ejecuta código Argo |

### Ejemplos

```argo
import "thread";

// Hilo simple
thread.spawn('
    for (let i = 0; i < 5; i += 1) {
        print("Hilo 1:", i);
    }
');

// Múltiples hilos
for (let i = 0; i < 3; i += 1) {
    thread.spawn('
        print("Hola desde hilo");
    ');
}

// Con canales
let ch = canal();
thread.spawn('
    for (let i = 0; i < 5; i += 1) {
        enviar(ch, i);
    }
');

for (let i = 0; i < 5; i += 1) {
    let val = recibir(ch);
    print("Recibido:", val);
}
```

---

## gpu

Framebuffer por software para gráficos 2D. Se accede con `gpu.funcion()`.

| Función | Descripción |
|---------|-------------|
| `gpu.crear_buffer(ancho, alto)` | Crea framebuffer RGBA |
| `gpu.pixel(fb, x, y, r, g, b)` | Dibuja un pixel |
| `gpu.linea(fb, x1, y1, x2, y2, r, g, b)` | Dibuja una línea (Bresenham) |
| `gpu.rect(fb, x, y, ancho, alto, r, g, b)` | Dibuja rectángulo relleno |
| `gpu.circulo(fb, cx, cy, radio, r, g, b)` | Dibuja círculo relleno |
| `gpu.guardar(fb, ruta)` | Guarda como imagen PPM |
| `gpu.mostrar(fb)` | Muestra en visor del sistema |
| `gpu.limpiar(fb, r, g, b)` | Rellena con color sólido |

### Ejemplos

```argo
import "gpu";

// Crear lienzo
let fb = gpu.crear_buffer(800, 600);

// Limpiar con azul oscuro
gpu.limpiar(fb, 20, 20, 40);

// Dibujar formas
gpu.rect(fb, 50, 50, 200, 100, 255, 100, 50);    // Rectángulo naranja
gpu.circulo(fb, 400, 300, 80, 50, 200, 100);      // Círculo verde
gpu.linea(fb, 0, 0, 800, 600, 255, 255, 255);     // Línea blanca
gpu.pixel(fb, 400, 300, 255, 0, 0);                // Pixel rojo

// Guardar imagen
gpu.guardar(fb, "salida.ppm");

// Mostrar
gpu.mostrar(fb);
```

---

## ui

Terminal UI de alto rendimiento (~30 FPS, doble buffer). Se accede con `ui.funcion()`.

### Widgets

| Función | Descripción |
|---------|-------------|
| `ui.tema(config)` | Configura colores globales |
| `ui.columna(padre, gap?)` | Crea布局 de columna |
| `ui.fila(padre, gap?)` | Crea layout de fila |
| `ui.texto(padre, texto, opts?)` | Crea widget de texto |
| `ui.boton(padre, texto, opts?)` | Crea botón interactivo |
| `ui.barra_progreso(padre, valor, opts?)` | Crea barra de progreso |
| `ui.input(padre, placeholder)` | Crea campo de texto |
| `ui.separador(padre)` | Crea línea separadora |
| `ui.rectangulo(padre, ancho, alto, opts?)` | Crea rectángulo |
| `ui.animar(nodo, prop, fin, duracion)` | Anima una propiedad |
| `ui.ejecutar()` | Inicia el game loop |
| `ui.limpiar()` | Limpia la pantalla |
| `ui.reset()` | Resetea UI y animaciones |

### Opciones de tema

```argo
ui.tema({
    "bg": "#2b2b2b",        // Fondo
    "fg": "#ffffff",        // Texto
    "primary": "#3b82f6",   // Color primario
    "secondary": "#8b5cf6", // Color secundario
    "accent": "#10b981",    // Color de acento
    "border": "#404040",    // Bordes
    "text_bg": "#1e1e1e",   // Fondo de texto
    "error": "#ef4444"      // Color de error
});
```

### Opciones de widget

```argo
// Texto
ui.texto(root, "Hola", {
    "color": "#3b82f6",
    "grande": true
});

// Botón
let btn = ui.boton(root, " Click ", {
    "callback": 1  // ID de retorno al presionar
});

// Barra de progreso
let barra = ui.barra_progreso(root, 0, {
    "color": "#10b981",
    "ancho": 30
});

// Rectángulo
ui.rectangulo(root, 20, 10, {
    "color": "#ff0000",
    "relleno": true
});
```

### Ejemplo completo

```argo
import "ui";

// Configurar tema
ui.tema({
    "bg": "#1a1b26",
    "fg": "#c0caf5",
    "primary": "#7aa2f7",
    "accent": "#9ece6a"
});

// Construir interfaz
let root = ui.columna(0, 1);

ui.texto(root, "Mi Aplicación TUI", { "grande": true });
ui.separador(root);
ui.texto(root, "Bienvenido a Argo v2.0.0");

let btn_iniciar = ui.boton(root, " Iniciar ", { "callback": 1 });
let btn_salir = ui.boton(root, " Salir ", { "callback": 2 });

let barra = ui.barra_progreso(root, 0, { "color": "#9ece6a", "ancho": 40 });

// Animar barra de 0 a 100 en 3 segundos
ui.animar(barra, "valor", 100, 3000);

// Ejecutar UI
let accion = ui.ejecutar();

if (accion == 1) {
    print("¡Iniciado!");
} else if (accion == 2) {
    print("Adiós");
}
```

### Arquitectura interna

| Característica | Detalle |
|---|---|
| Doble buffer | backbuffer/frontbuffer con diffing |
| Dirty-rect | Solo dibuja celdas modificadas |
| AnsiBuf | Buffer 4KB zero-alloc |
| ~30 FPS | Game loop controlado por tiempo |
| Animaciones | Lerp + ease-in-out |
| Colores | RGB 24-bit, hex #RRGGBB, nombres |
| Input | No bloqueante via ioctl FIONREAD |

---

## Resumen de la stdlib

| Módulo | Funciones | Constantes |
|--------|-----------|------------|
| Built-in | 11 | — |
| math | 14 | 2 |
| str | 16 | — |
| arr | 17 | — |
| fs | 8 | — |
| buffer | 7 | — |
| os | 7 | — |
| time | 4 | — |
| net | 1 | — |
| json | 2 | — |
| thread | 1 | — |
| gpu | 8 | — |
| ui | 13 | — |
| **Total** | **109** | **2** |
