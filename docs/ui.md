# Módulo `ui` — Motor de Renderizado TUI de Alto Rendimiento

El módulo `ui` implementa un framework de UI para terminales con motor de renderizado de alto rendimiento: doble buffer, dirty-rect diffing, animaciones Lerp y game loop a ~30 FPS.

## Arquitectura

```
┌─────────────────────────────────────────┐
│              API Pública                │
│  ui.tema(), ui.columna(), ui.texto()    │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Árbol de Widgets (ArbolUI)       │
│  HashMap<O(1)> + Layout Flexbox         │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       Motor de Animaciones              │
│  Lerp, ease-in-out, tick-based          │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│       MotorRenderizado                  │
│  Doble buffer, dirty-rect, AnsiBuf      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Terminal (ANSI escape codes)     │
└─────────────────────────────────────────┘
```

## API Pública

### `ui.tema(config)`

Establece el tema global de colores. Soporta hex `#RRGGBB` y nombres.

**Parámetros:**
- `config`: diccionario con claves opcionales:
  - `bg`: color de fondo
  - `fg`: color de texto
  - `primary`: color primario
  - `secondary`: color secundario
  - `accent`: color de acento
  - `border`: color de bordes
  - `text_bg`: fondo de inputs
  - `error`: color de error

**Nombres soportados:** `rojo`, `verde`, `azul`, `amarillo`, `naranja`, `morado`, `rosa`, `cian`, `blanco`, `negro`, `gris`

```argo
ui.tema({
    bg: "#1a1a2e",
    fg: "#e0e0e0",
    primary: "azul",
    accent: "#10b981"
});
```

---

### `ui.columna(padre, gap?)`

Crea un layout vertical (columna). Los hijos se apilan de arriba hacia abajo.

**Parámetros:**
- `padre`: ID del nodo padre (0 para raíz)
- `gap` (opcional): espacio entre elementos (default: 1)

**Retorno:** ID del nodo creado.

```argo
let root = ui.columna(0, 1);
```

---

### `ui.fila(padre, gap?)`

Crea un layout horizontal (fila). Los hijos se colocan de izquierda a derecha.

**Parámetros:**
- `padre`: ID del nodo padre
- `gap` (opcional): espacio entre elementos (default: 1)

**Retorno:** ID del nodo creado.

```argo
let fila = ui.fila(root, 2);
```

---

### `ui.texto(padre, texto, opts?)`

Crea un widget de texto.

**Parámetros:**
- `padre`: ID del nodo padre
- `texto`: cadena a mostrar
- `opts` (opcional): diccionario con:
  - `color`: color del texto (hex o nombre)
  - `grande`: `true` para texto grande (usa color accent)

**Retorno:** ID del nodo creado.

```argo
ui.texto(root, "Hola mundo");
ui.texto(root, "Título", { grande: true });
ui.texto(root, "Rojo", { color: "rojo" });
```

---

### `ui.boton(padre, texto, opts?)`

Crea un botón con borde y fondo.

**Parámetros:**
- `padre`: ID del nodo padre
- `texto`: etiqueta del botón
- `opts` (opcional): diccionario con:
  - `callback`: valor retornado al presionar Enter

**Retorno:** ID del nodo creado.

```argo
let btn = ui.boton(root, " Click me ", { callback: 1 });
```

---

### `ui.barra_progreso(padre, valor, opts?)`

Crea una barra de progreso animable.

**Parámetros:**
- `padre`: ID del nodo padre
- `valor`: valor inicial (0-100)
- `opts` (opcional): diccionario con:
  - `color`: color de relleno (hex o nombre)
  - `ancho`: ancho en caracteres (default: 30)

**Retorno:** ID del nodo creado.

```argo
let barra = ui.barra_progreso(root, 0, {
    color: "#10b981",
    ancho: 40
});
```

---

### `ui.rectangulo(padre, ancho, alto, opts?)`

Crea un rectángulo (borde o relleno).

**Parámetros:**
- `padre`: ID del nodo padre
- `ancho`: ancho en caracteres
- `alto`: alto en líneas
- `opts` (opcional): diccionario con:
  - `color`: color del rectángulo
  - `relleno`: `true` para relleno sólido, `false` para borde

**Retorno:** ID del nodo creado.

```argo
let caja = ui.rectangulo(root, 20, 5, { color: "azul", relleno: true });
```

---

### `ui.input(padre, placeholder)`

Crea un campo de entrada de texto con borde.

**Parámetros:**
- `padre`: ID del nodo padre
- `placeholder`: texto de ejemplo

**Retorno:** ID del nodo creado.

```argo
let input = ui.input(root, "Escribe algo...");
```

---

### `ui.separador(padre)`

Crea una línea separadora horizontal.

**Parámetros:**
- `padre`: ID del nodo padre

**Retorno:** ID del nodo creado.

```argo
ui.separador(root);
```

---

### `ui.animar(nodo_id, propiedad, fin, duracion_ms)`

Lanza una animación con interpolación lineal (Lerp).

**Parámetros:**
- `nodo_id`: ID del nodo a animar
- `propiedad`: nombre de la propiedad (`"valor"`, `"fg_r"`, `"fg_g"`, `"fg_b"`)
- `fin`: valor final de la animación
- `duracion_ms`: duración en milisegundos

```argo
// Animar barra de 0 a 100 en 2 segundos
ui.animar(barra, "valor", 100, 2000);

// Animar color de texto
ui.animar(texto, "fg_r", 255, 1000);
```

---

### `ui.ejecutar()`

Inicia el game loop principal a ~30 FPS. El motor toma control de la terminal.

**Controles:**
- `Tab`: cambiar foco al siguiente widget
- `Enter`: activar el widget enfocado (retorna callback ID)
- `flechas`: navegar (futuro)
- `q`: salir del loop

**Retorno:** `id_callback` del botón presionado, o `Nulo` con `q`.

```argo
let accion = ui.ejecutar();
if (accion == 1) {
    print("Botón clickeado!");
}
```

---

### `ui.limpiar()`

Limpia toda la pantalla de la terminal.

```argo
ui.limpiar();
```

---

### `ui.reset()`

Destruye todo el árbol de UI y resetea el estado del motor.

```argo
ui.reset();
```

---

## Ejemplo Completo

```argo
// Configurar tema
ui.tema({
    bg: "#2b2b2b",
    fg: "#ffffff",
    primary: "#3b82f6",
    accent: "#10b981",
    border: "#4b5563"
});

// Construir árbol de UI
let root = ui.columna(0, 1);

ui.texto(root, "Dashboard de Argo", { grande: true });
ui.separador(root);

let fila1 = ui.fila(root, 3);
ui.texto(fila1, "Estado:");
ui.texto(fila1, "Online", { color: "#10b981" });

let barra = ui.barra_progreso(root, 0, { color: "#3b82f6", ancho: 40 });
let btn = ui.boton(root, " Iniciar ", { callback: 1 });
let input = ui.input(root, "Nombre del proyecto...");

// Animar barra de progreso
ui.animar(barra, "valor", 100, 3000);

// Ejecutar game loop
let accion = ui.ejecutar();
if (accion == 1) {
    print("Proyecto iniciado!");
}
```

## Motor de Renderizado

El motor implementa las siguientes optimizaciones:

| Optimización | Descripción |
|---|---|
| **Doble buffer** | Comparación backbuffer/frontbuffer para detectar cambios |
| **Dirty-rect** | Solo se envían celdas modificadas a la terminal |
| **AnsiBuf** | Buffer de 4KB en stack para secuencias ANSI sin allocaciones |
| **Batch de colores** | Solo se escribe fg/cuando cambian entre celdas |
| **Input no bloqueante** | ioctl FIONREAD para verificar stdin sin bloquear |
| **Game loop** | 33ms por frame (~30 FPS) controlado por tiempo |
