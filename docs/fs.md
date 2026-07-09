# Módulo `fs` — Sistema de Archivos

El módulo `fs` permite leer y escribir archivos. Todas las rutas están sandbooxed (sin `..`, sin `~`, sin rutas absolutas).

## Funciones

### `fs.leer(ruta)`

Lee el contenido de un archivo y lo retorna como cadena.

```argo
let contenido = fs.leer("datos.txt");
print(contenido);
```

---

### `fs.escribir(ruta, contenido)`

Escribe `contenido` en el archivo `ruta`. Retorna `true` si fue exitoso.

```argo
fs.escribir("salida.txt", "Hola mundo");
```

---

### `fs.existe(ruta)`

Retorna `true` si la ruta existe.

```argo
print(fs.existe("datos.txt"));  // true o false
```

---

### `fs.es_directorio(ruta)`

Retorna `true` si la ruta es un directorio.

```argo
print(fs.es_directorio("src"));  // true
```

---

### `fs.es_archivo(ruta)`

Retorna `true` si la ruta es un archivo.

```argo
print(fs.es_archivo("README.md"));  // true
```

---

### `fs.eliminar(ruta)`

Elimina el archivo en `ruta`. Retorna `true` si fue exitoso.

```argo
fs.eliminar("temp.txt");
```

---

### `fs.crear_directorio(ruta)`

Crea un directorio (y sus padres). Retorna `true` si fue exitoso.

```argo
fs.crear_directorio("src/components");
```

---

### `fs.listar(ruta)`

Lista los archivos del directorio como un arreglo de cadenas.

```argo
let archivos = fs.listar(".");
print(archivos);
```
