# Módulo `net` — Red

El módulo `net` permite realizar peticiones HTTP/HTTPS raw sobre TCP.

## Funciones

### `net.solicitud(metodo, host:puerto, cuerpo?)`

Envía una petición HTTP raw. Retorna la respuesta completa como cadena.

**Parámetros:**
- `metodo`: `"GET"`, `"POST"`, `"PUT"`, `"PATCH"`, o `"DELETE"`
- `host:puerto`: cadena como `"example.com:80"` o `"example.com:443"`
- `cuerpo` (opcional): cadena con el body de la petición (para POST/PUT/PATCH)

```argo
// GET simple
let respuesta = net.solicitud("GET", "httpbin.org:80", "");
print(respuesta);

// POST con body
let body = '{"nombre": "Argo"}';
let respuesta = net.solicitud("POST", "httpbin.org:80", body);
print(respuesta);
```

---

## Notas

- La función retorna la respuesta HTTP completa incluyendo cabeceras.
- Para HTTPS, el servidor debe soportar la conexión directa.
- No hay soporte para cabeceras personalizadas actualmente.
