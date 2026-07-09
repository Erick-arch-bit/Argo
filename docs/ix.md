# Módulo `ix` — Window Manager

El módulo `ix` implementa un gestor de ventanas flotante para terminales con soporte ANSI.

## Funciones

### `ix.crear_ventana(titulo, ancho, alto, opts?)`

Crea una ventana flotante. Retorna el ID de la ventana.

**Parámetros:**
- `titulo`: título de la ventana
- `ancho`: ancho en caracteres
- `alto`: alto en líneas
- `opts` (opcional): diccionario con:
  - `x`: posición horizontal inicial
  - `y`: posición vertical inicial
  - `callback`: valor retornado al presionar Enter

```argo
let w1 = ix.crear_ventana("Mi Ventana", 30, 10);
let w2 = ix.crear_ventana("Otra", 20, 8, { x: 50, y: 20, callback: 42 });
```

---

### `ix.mover_ventana(id, x, y)`

Mueve una ventana a la posición (x, y).

```argo
ix.mover_ventana(w1, 10, 5);
```

---

### `ix.cambiar_foco(id)`

Cambia el foco a la ventana especificada (la trae al frente).

```argo
ix.cambiar_foco(w2);
```

---

### `ix.cerrar_ventana(id)`

Cierra y elimina una ventana.

```argo
ix.cerrar_ventana(w1);
```

---

### `ix.ejecutar()`

Inicia el loop de eventos del window manager. Retorna el `callback` de la ventana cuando se presiona Enter.

**Controles:**
- `Flechas`: mover la ventana enfocada
- `Tab`: cambiar foco a la siguiente ventana
- `Enter`: activar callback de la ventana enfocada
- `q`: salir del loop

```argo
let accion = ix.ejecutar();
if (accion == 42) {
    print("Ventana 2 activada!");
}
```

---

### `ix.listar_ventanas()`

Retorna un arreglo con los IDs de todas las ventanas abiertas.

```argo
let ventanas = ix.listar_ventanas();
print("Ventanas abiertas:", len(ventanas));
```

---

### `ix.info_ventana(id)`

Retorna un diccionario con la información de la ventana.

**Retorno:**
```argo
{
    titulo: "Mi Ventana",
    x: 10,
    y: 5,
    ancho: 30,
    alto: 10,
    z_index: 0
}
```

```argo
let info = ix.info_ventana(w1);
print(info.titulo, info.x, info.y);
```

---

## Ejemplo Completo

```argo
let w1 = ix.crear_ventana("Terminal", 40, 15, { x: 5, y: 2, callback: 1 });
let w2 = ix.crear_ventana("Editor", 30, 10, { x: 50, y: 5, callback: 2 });
let w3 = ix.crear_ventana("Consola", 25, 8, { x: 20, y: 20, callback: 3 });

let accion = ix.ejecutar();
if (accion == 1) {
    print("Terminal activada");
} else if (accion == 2) {
    print("Editor activado");
} else if (accion == 3) {
    print("Consola activada");
}
```
