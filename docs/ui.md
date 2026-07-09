# Módulo `ui` — Interfaz de Terminal (TUI)

El módulo `ui` proporciona un framework de UI inmediata para terminales con soporte ANSI.

## Funciones

### `ui.tema(config)`

Establece el tema global de colores.

**Parámetros:**
- `config`: diccionario con claves opcionales:
  - `bg`: color de fondo (hex, ej: `"#2b2b2b"`)
  - `fg`: color de texto (hex, ej: `"#ffffff"`)
  - `primary`: color primario (hex, ej: `"#3b82f6"`)
  - `secondary`: color secundario (hex, ej: `"#6b7280"`)
  - `accent`: color de acento (hex, ej: `"#10b981"`)
  - `border`: color de bordes (hex, ej: `"#4b5563"`)
  - `text_bg`: fondo de inputs (hex, ej: `"#1f2937"`)
  - `error`: color de error (hex, ej: `"#ef4444"`)

```argo
ui.tema({ bg: "#1a1a2e", fg: "#e0e0e0", primary: "#0f3460" });
```

---

### `ui.columna(id_padre, gap)`

Crea un layout vertical (columna). Retorna el ID del nodo.

**Parámetros:**
- `id_padre`: ID del nodo padre (0 para raíz)
- `gap`: espacio entre elementos

```argo
let root = ui.columna(0, 1);
```

---

### `ui.fila(id_padre, gap)`

Crea un layout horizontal (fila). Retorna el ID del nodo.

```argo
let fila = ui.fila(root, 2);
```

---

### `ui.texto(id_padre, texto, opts?)`

Crea un widget de texto. Retorna el ID del nodo.

**Parámetros:**
- `id_padre`: ID del nodo padre
- `texto`: cadena a mostrar
- `opts` (opcional): diccionario con:
  - `color`: color del texto (hex)
  - `grande`: `true` para texto grande (usa color accent)

```argo
ui.texto(root, "Hola mundo");
ui.texto(root, "Título", { grande: true });
```

---

### `ui.boton(id_padre, texto, id_callback)`

Crea un botón. Retorna el ID del nodo.

**Parámetros:**
- `id_padre`: ID del nodo padre
- `texto`: etiqueta del botón
- `id_callback`: valor retornado al presionar Enter

```argo
let btn = ui.boton(root, " Click me ", 1);
```

---

### `ui.input(id_padre, placeholder)`

Crea un campo de entrada de texto. Retorna el ID del nodo.

```argo
let input = ui.input(root, "Escribe algo...");
```

---

### `ui.separador(id_padre)`

Crea una línea separadora horizontal. Retorna el ID del nodo.

```argo
ui.separador(root);
```

---

### `ui.ejecutar()`

Inicia el loop de eventos de la UI. Retorna el `id_callback` del botón presionado cuando se presiona Enter, o `Nulo` con `q`.

**Controles:**
- `Tab`: cambiar foco al siguiente widget
- `Enter`: activar el widget enfocado (retorna callback ID)
- `q`: salir del loop

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

Destruye todo el árbol de UI y resetea el estado.

```argo
ui.reset();
```

---

## Ejemplo Completo

```argo
ui.tema({ bg: "#2b2b2b", fg: "#ffffff", primary: "#3b82f6", accent: "#10b981" });

let root = ui.columna(0, 1);
ui.texto(root, "Mi Aplicación", { grande: true });
ui.separador(root);
ui.texto(root, "Bienvenido a Argo TUI");
let btn = ui.boton(root, " Iniciar ", 1);
let input = ui.input(root, "Nombre...");

let accion = ui.ejecutar();
if (accion == 1) {
    print("Iniciado!");
}
```
