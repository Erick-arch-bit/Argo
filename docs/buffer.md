# Módulo `buffer` — Buffers Binarios

El módulo `buffer` permite trabajar con datos binarios de bajo nivel.

## Funciones

### `buffer.alloc(tamano)`

Crea un buffer de `tamano` bytes lleno de ceros. Máximo 1GB.

```argo
let buf = buffer.alloc(1024);
print(buffer.longitud(buf));  // 1024
```

---

### `buffer.write(buffer, indice, valor)`

Retorna un nuevo buffer con el byte en `indice` establecido a `valor` (0-255).

```argo
let buf = buffer.alloc(8);
let buf2 = buffer.write(buf, 0, 65);
print(buffer.read(buf2, 0));  // 65
```

---

### `buffer.read(buffer, indice)`

Retorna el valor del byte en `indice`.

```argo
let buf = buffer.alloc(8);
let buf2 = buffer.write(buf, 0, 42);
print(buffer.read(buf2, 0));  // 42
```

---

### `buffer.longitud(buffer)`

Retorna la longitud del buffer en bytes.

```argo
let buf = buffer.alloc(256);
print(buffer.longitud(buf));  // 256
```

---

### `buffer.a_cadena(buffer)`

Convierte los bytes del buffer a una cadena UTF-8 (con pérdida).

```argo
let buf = buffer.de_cadena("Hola");
print(buffer.a_cadena(buf));  // "Hola"
```

---

### `buffer.de_cadena(cadena)`

Convierte una cadena a su representación en bytes.

```argo
let buf = buffer.de_cadena("Hello");
print(buffer.longitud(buf));  // 5
```

---

### `buffer.copiar(origen, destino, posicion)`

Copia bytes de `origen` a `destino` empezando en `posicion`.

```argo
let src = buffer.de_cadena("AB");
let dst = buffer.alloc(4);
let dst2 = buffer.copiar(src, dst, 1);
print(buffer.a_cadena(dst2));  // "\0AB\0"
```
