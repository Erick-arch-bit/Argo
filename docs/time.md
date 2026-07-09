# Módulo `time` — Tiempo

El módulo `time` proporciona acceso al reloj del sistema y funciones de espera.

## Funciones

### `time.ahora()`

Retorna el tiempo actual en milisegundos desde la época UNIX.

```argo
let t = time.ahora();
print("Tiempo:", t);
```

---

### `time.dormir(milisegundos)`

Duerme el hilo actual durante los milisegundos especificados. Retorna `Nulo`.

```argo
print("Iniciando...");
time.dormir(1000);
print("1 segundo después");
```

---

### `time.segundos()`

Retorna el tiempo actual en segundos desde la época UNIX.

```argo
let secs = time.segundos();
print("Segundos:", secs);
```

---

### `time.micros()`

Retorna el tiempo actual en microsegundos desde la época UNIX.

```argo
let micros = time.micros();
print("Microsegundos:", micros);
```
