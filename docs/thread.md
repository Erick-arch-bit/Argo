# Módulo `thread` — Concurrencia

El módulo `thread` permite ejecutar código en hilos separados del sistema operativo.

## Funciones

### `thread.spawn(codigo_fuente)`

Crea un nuevo hilo del sistema operativo que lexea, parsea y evalúa el código Argo dado en un entorno aislado. Retorna `true` inmediatamente.

```argo
thread.spawn('print("Hola desde otro hilo!")');
print("Esto se imprime inmediatamente");
```

---

## Notas

- Cada hilo tiene su propio entorno de variables.
- Los errores en el hilo se imprimen a stderr.
- No hay soporte para compartir datos entre hilos actualmente (usa canales para comunicación futura).
