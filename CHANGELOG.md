# Changelog

## v1.5.3 — 2026-07-09

### Nuevo: Documentación completa
- Carpeta `docs/` con documentación de cada módulo de la stdlib
- `docs/arr.md` — Operaciones con arreglos
- `docs/buffer.md` — Buffers binarios
- `docs/fs.md` — Sistema de archivos
- `docs/gpu.md` — Framebuffer por software
- `docs/json.md` — JSON
- `docs/math.md` — Matemáticas
- `docs/net.md` — Red
- `docs/os.md` — Sistema operativo
- `docs/str.md` — Cadenas de texto
- `docs/thread.md` — Concurrencia
- `docs/time.md` — Tiempo
- `docs/ui.md` — Interfaz de terminal (TUI)
- `docs/ix.md` — Window manager
- `docs/render.md` — Motor de renderizado interno

### Actualizado
- README.md con documentación de ix y render
- Versión bump a 1.5.3 en todos los archivos

---

## v1.5.2 — 2026-07-09

### Nuevo: Render engine profesional
- `Celda` con bg/fg/char — buffer de 2400 celdas (80×24)
- `RenderEngine` con backbuffer/frontbuffer — dirty-rect diffing
- `DirtyRect` merging — merge de rectángulos adyacentes
- `AnsiBuf` — buffer de 4KB en stack, zero-alloc ANSI writing
- `write_u8`/`write_usize` — enteros a ASCII sin format!
- Batch de colores — solo escribe fg/bg cuando cambian entre celdas
- `dirty_cells` bitset — salto rápido de celdas limpias

### Nuevo: Library UI profesional
- `Theme` estilo CustomTkinter — bg #2b2b2b, primary #3b82f6, accent #10b981
- `LayoutCache` con HashMap pre-asignado — O(1) lookup por nodo
- `calcular_layout` escribe en cache en lugar de Vec de tuples
- `dibujar_arbol` busca por HashMap en lugar de iter().find()
- Widgets: `Texto`, `Boton`, `Input`, `Separador`
- Layouts: `Columna`, `Fila` con padding/border/margen

### Nuevo: Window manager (ix)
- `IXWindow` — ventana flotante con título, posición, z-index, callback
- `IXDesktop` — escritorio con z-ordering y foco
- `redibujar_ventana()` — solo limpia y redibuja 1 ventana
- Arrow keys mueven ventana enfocada, Tab cambia foco
- `ix.crear_ventana()`, `ix.mover_ventana()`, `ix.cerrar_ventana()`

### Optimizaciones de rendimiento
- **Renderer**: 0 allocations en hot path (5760→0 allocs por frame)
- **Renderer**: AnsiBuf 4KB stack buffer vs format! heap alloc
- **Renderer**: Batch colores — solo escribe cuando fg/bg cambian
- **Renderer**: dirty_cells bitset — skip O(1) por celda limpia
- **UI**: LayoutCache HashMap — O(1) lookup vs O(n) iter().find()
- **UI**: Input widget — push_str + padding manual sin format!
- **IX**: redibujar_ventana — solo 1 ventana en lugar de todas
- **IX**: Vec::with_capacity(16) para ventanas

---

## v1.3.0 — 2026-06-24

### Nuevo: Módulo GPU
- `gpu.crear_buffer(ancho, alto)` — crea framebuffer
- `gpu.pixel(fb, x, y, r, g, b)` — dibuja un píxel
- `gpu.linea(fb, x1, y1, x2, y2, r, g, b)` — dibuja línea
- `gpu.rect(fb, x, y, ancho, alto, r, g, b)` — dibuja rectángulo relleno
- `gpu.circulo(fb, cx, cy, radio, r, g, b)` — dibuja círculo relleno
- `gpu.guardar(fb, ruta)` — exporta a PPM
- `gpu.mostrar(fb)` — abre con visor del sistema
- `gpu.limpiar(fb, r, g, b)` — llena el buffer con un color
- Renderizado por software, cero dependencias externas

### Nuevas funciones en stdlib
- **`arr`**: `contiene`, `invertir`, `primero`, `ultimo`, `concatenar`, `vacio`, `indice_de`, `plano`
- **`str`**: `indice_de`, `invertir`, `repetir`, `a_arreglo`, `codigo_en`, `de_codigo`
- **`buffer`**: `longitud`, `a_cadena`, `de_cadena`, `copiar`
- **`os`**: `directorio_actual`, `directorio_temporal`, `argumentos`, `procesadores`
- **`time`**: `segundos`, `micros`
- **`math`**: `tan`, `pow`, `log`, `log10`, `floor`, `ceil`, `round`, `max`, `min`
- **`fs`**: `existe`, `es_directorio`, `es_archivo`, `eliminar`, `crear_directorio`, `listar`

### Constantes inmutables
- `const PI = 3.1416;` — ahora da error en asignación (`PI = 3;` falla en runtime)

### Optimizaciones de rendimiento
- **Parser**: `avanzar()` retorna el `Token` por ownership — elimina ~10 clones de `String` por expresión
- **Entorno**: `actualizar()` usa `get_mut` (1 lookup en HashMap vs 3)
- **Entorno**: `declarar()` evita clonar la clave cuando no es `const`
- **Display**: `Arreglo`/`Diccionario` escriben inline sin `Vec<String>` intermedio
- **Pre-asignación**: `Vec::with_capacity` en parser y evaluador para parámetros, argumentos, arrays, diccionarios
- **Evaluador**: `evaluar_bloque` retorna `Nulo` inmediato para bloques vacíos
- **Objeto**: `tomar_llave_hash()` para conversión zero-copy de claves

### Instalación
- `install.sh` ahora auto-detecta SO/arquitectura y descarga binarios precompilados
- Soporte: `linux-amd64`, `linux-arm64`, `darwin-amd64`, `darwin-arm64`
- Fallback a `cargo build --release` si falla la descarga
- Auto-agregado a `$PATH`

### CI/CD
- Release matrix ahora incluye `aarch64-unknown-linux-gnu` (ARM64 Linux)
- Todos los targets tienen dependencias explícitas para交叉-compilación

### Fixes
- `entero()` y `cadena()` ya no se filtran incorrectamente de módulos nativos
- `const` ahora se respeta en runtime (antes solo se parseaba)

---

## v1.2.0 — 2026-06-23

### Caché binaria de AST (`.argbc`)
- Serialización binaria del AST tras el primer parseo
- Carga instantánea en ejecuciones posteriores (sin lexer ni parser)
- Soporte para módulos locales y remotos
- Formato compacto: enteros como `i64`, cadenas como `String`, floats como `f64`

### HTTP completo (`net`)
- `net.solicitud(url, metodo, cuerpo, cabeceras)` — peticiones HTTP/HTTPS
- Soporte para GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- Manejo de cabeceras personalizadas
- Timeout configurable
- Manejo de errores de red

### Concurrencia (`thread`)
- `thread.spawn(codigo)` — ejecuta código Argo en un hilo separado
- Aislamiento de entornos entre hilos
- Comunicación vía variables compartidas

### Nuevas funciones
- `tipo(obj)` — retorna el nombre del tipo como string
- `buffer.alloc(tam)`, `buffer.write(buf, pos, val)`, `buffer.read(buf, pos)`
- `time.ahora()`, `time.dormir(ms)`
- `os.ejecutar(comando)`, `os.variables()`
- `net.solicitud(url, metodo, cuerpo, cabeceras)`
- `json.parsear(str)`, `json.stringificar(obj)`

### Seguridad
- Límite de profundidad en parser y evaluador (previene stack overflow)
- Path traversal bloqueado en imports
- Timeout en peticiones HTTP

---

## v1.1.1 — 2026-06-22

### Fixes
- Parche de seguridad: path traversal en imports
- Límite de profundidad en parser para evitar stack overflow
- Límite de profundidad en evaluación de prototipos

---

## v1.1.0 — 2026-06-21

### Arquitectura stdlib
- Módulos nativos como diccionarios inmutables
- Registro modular: `arr`, `str`, `math`, `fs`, `net`, `json`, `time`, `os`, `buffer`, `thread`
- Funciones como ciudadanos de primera clase dentro de módulos

### Operadores bitwise
- `&` (AND), `|` (OR), `^` (XOR), `~` (NOT), `<<` (shift left), `>>` (shift right)
- Precedencia correcta en el parser

### Prototipos
- Cadena de prototipos para diccionarios (`__proto__`)
- Herencia delegativa entre objetos

### Mejoras
- Operador `!=` implementado
- Strings multilínea con `"""`
- Coerción automática string + número

---

## v1.0.0 — 2026-06-20

### Lenguaje base
- Tipos: Enteros (`i64`), Flotantes (`f64`), Booleanos, Cadenas, Nulo
- Estructuras: Arreglos, Diccionarios, Funciones (closures)
- Operadores: aritméticos, relacionales, lógicos, concatenación
- Control de flujo: `if/else`, `while`, `for`, `break`
- Funciones: declaración `fn`, anónimas, closures con ámbito léxico
- Variables: `let`, asignación, ámbito léxico con entorno anidado
- `try/catch` para manejo de errores
- `import "ruta"` para módulos locales

### Biblioteca estándar básica
- `print()`, `len()`, `push()`
- `math.sin`, `math.cos`, `math.sqrt`, `math.abs`, `math.random`
- `fs.leer`, `fs.escribir`
- `str.longitud`, `str.mayusculas`, `str.minusculas`, `str.recortar`, `str.dividir`, `str.contiene`, `str.subcadena`, `str.reemplazar`, `str.empieza_con`, `str.termina_con`
- `arr.len`, `arr.push`, `arr.pop`

### Infraestructura
- REPL interactivo
- Sistema de proyectos: `argo init`, `argo.toml`, `argo run`
- Parser tolerante a errores
- Comas finales toleradas
- Paréntesis opcionales en `if`, `while`, `for`, `catch`
- Instalación vía script (`install.sh`)
- CI/CD con releases multiplataforma (Linux x86_64, macOS x86_64/ARM64)
