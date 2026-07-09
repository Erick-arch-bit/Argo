# Módulo `gpu` — Framebuffer por Software

El módulo `gpu` implementa un GPU por software con framebuffer RGBA. Los framebuffers son diccionarios: `{ buffer: Buffer, ancho: int, alto: int }`.

## Funciones

### `gpu.crear_buffer(ancho, alto)`

Crea un framebuffer de `ancho` x `alto` píxeles lleno de ceros. Máximo 256MB.

```argo
let fb = gpu.crear_buffer(800, 600);
```

---

### `gpu.pixel(fb, x, y, r, g, b)`

Establece un píxel en las coordenadas (x, y) con color RGB (0-255).

```argo
gpu.pixel(fb, 100, 100, 255, 0, 0);  // Rojo
```

---

### `gpu.linea(fb, x1, y1, x2, y2, r, g, b)`

Dibuja una línea desde (x1, y1) hasta (x2, y2) usando el algoritmo de Bresenham.

```argo
gpu.linea(fb, 0, 0, 100, 100, 0, 255, 0);  // Línea verde
```

---

### `gpu.rect(fb, x, y, ancho, alto, r, g, b)`

Dibuja un rectángulo relleno.

```argo
gpu.rect(fb, 50, 50, 200, 100, 0, 0, 255);  // Rectángulo azul
```

---

### `gpu.circulo(fb, cx, cy, radio, r, g, b)`

Dibuja un círculo relleno centrado en (cx, cy).

```argo
gpu.circulo(fb, 400, 300, 50, 255, 255, 0);  // Círculo amarillo
```

---

### `gpu.guardar(fb, ruta)`

Guarda el framebuffer como imagen PPM.

```argo
gpu.guardar(fb, "imagen.ppm");
```

---

### `gpu.mostrar(fb)`

Abre el framebuffer con el visor de imágenes del sistema.

```argo
gpu.mostrar(fb);
```

---

### `gpu.limpiar(fb, r, g, b)`

Llena todo el framebuffer con un color sólido.

```argo
gpu.limpiar(fb, 0, 0, 0);  // Negro
```
