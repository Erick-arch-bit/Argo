# Módulo `json` — JSON

El módulo `json` permite parsear y serializar datos JSON.

## Funciones

### `json.parsear(cadena)`

Parsea una cadena JSON y retorna un objeto Argo. Si la cadena contiene cabeceras HTTP, las ignora automáticamente.

```argo
let datos = json.parsear('{"nombre": "Argo", "version": 1}');
print(datos.nombre);  // "Argo"
print(datos.version);  // 1
```

---

### `json.stringificar(objeto)`

Serializa un objeto Argo a cadena JSON.

```argo
let datos = { nombre: "Argo", activo: true };
let json_str = json.stringificar(datos);
print(json_str);  // {"nombre":"Argo","activo":true}
```

---

## Notas

- La profundidad máxima de anidamiento es 128 niveles.
- Los tipos no soportados se serializan como `"__tipo_no_soportado__"`.
