# Changelog

## v2.1.0 — 2026-07-16

### CLI Demo integrado en el core
- **NUEVO `argo cli`:** Demo CLI interactiva con menú visual, spines Unicode y animaciones ANSI
- `src/cli_demo.rs` — Módulo con 3 flujos demostrativos (instalar, actualizar, REPL simulado)
- El binario separado `argo-cli` se eliminó; ahora es parte del binario `argo`

### Correcciones
- Funciones no usadas eliminadas de `cli_demo.rs`

### Documentación
- `docs/commands.md` actualizado con comando `argo cli`

## v2.0.0 — 2026-07-14

### Sistema de Paquetes (Package Manager)
- **NUEVO `src/pkg/toml.rs`:** Parser TOML completo (subset) para argo.toml — soporta [package], [dependencies], strings, arrays, tablas, comentarios
- **NUEVO `src/pkg/semver.rs`:** Parser SemVer con rangos ^, ~, exacto, *, pre-release, build metadata
- **NUEVO `src/pkg/sha256.rs`:** SHA-256 desde cero según FIPS 180-4 — cero dependencias externas
- **NUEVO `src/pkg/http.rs`:** Cliente HTTP/1.1 sobre TcpStream — GET/HEAD, headers, auth token, timeout
- **NUEVO `src/pkg/lock.rs`:** Lectura/escritura de argo.lock — formato TOML-like con [[package]]
- **NUEVO `src/pkg/cache.rs`:** Caché binario (cache.idx + cache.data) — migración automática de directorios
- **NUEVO `src/pkg/output.rs`:** Colores, barra de progreso, detección TTY
- **NUEVO `src/pkg/multihost.rs`:** Soporte GitHub/GitLab/Bitbucket — RepoInfo, AuthTokens, URLs
- **NUEVO `src/pkg/resolve.rs`:** Resolución recursiva de dependencias — lockfile, fallback jsDelivr, retry
- **NUEVO `src/pkg/publish.rs`:** Publicación con SHA-256, tag, push, validaciones

### Comandos CLI
- `argo install` — Instala dependencias desde argo.toml (resolución + lockfile)
- `argo install <repo>` — Instala un paquete específico desde GitHub
- `argo update` — Actualiza todas las dependencias
- `argo update <repo>` — Actualiza un paquete específico
- `argo publish` — Valida y publica el paquete en GitHub
- `argo publish --dry-run` — Solo validación sin publicar
- `argo list` — Lista paquetes instalados
- `argo clean` — Limpia caché de paquetes (>30 días)
- `argo config set` — Configura tokens de autenticación
- `argo uninstall` — Desinstala un paquete

### Seguridad
- SHA-256 desde cero (FIPS 180-4) — cero dependencias externas
- Validación de integridad en descargas
- Verificación de remote de git contra nombre del paquete

### Documentación
- README.md actualizado con v2.0.0 y comandos del package manager
- docs/ actualizados

## v1.5.3 — 2026-07-09

### Reescritura completa del módulo TUI
- **Eliminados:** `ix/` (window manager), `render.rs` (motor antiguo), `layout.rs` (layout antiguo)
- **Nuevo `engine.rs`:** MotorRenderizado con doble buffer, dirty-rect diffing, AnsiBuf 4KB zero-alloc
- **Nuevo `widgets.rs`:** ArbolUI con HashMap O(1), layout Flexbox simplificado (Columna/Fila/Area)
- **Nuevo `animaciones.rs`:** Motor de animaciones con Lerp, ease-in-out, tick-based
- **Nuevo `mod.rs`:** Game loop a ~30 FPS, input no bloqueante via ioctl FIONREAD
- **API completa:** `ui.tema`, `ui.columna`, `ui.fila`, `ui.texto`, `ui.boton`, `ui.barra_progreso`, `ui.rectangulo`, `ui.input`, `ui.separador`, `ui.animar`, `ui.ejecutar`

### Características del motor
- Doble buffer con diffing de celdas individuales
- Dirty-rect tracking para flush mínimo
- AnsiBuf de 4KB en stack para escritura ANSI sin allocaciones
- Detección de terminal via stty (Unix) con fallback 80x24
- Colores RGB 24-bit por celda
- Soporte hex #RRGGBB y nombres (rojo, azul, etc.)
- Input no bloqueante via ioctl FIONREAD
- CERO dependencias externas

### Documentación
- Carpeta `docs/` con documentación de cada módulo
- `docs/ui.md` actualizado con nueva API y arquitectura
- README.md actualizado con tabla de características TUI

### Eliminados
- `docs/ix.md` — Window manager eliminado
- `docs/render.md` — Motor antiguo eliminado

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
