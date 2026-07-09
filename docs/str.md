# Módulo `str` — Cadenas de Texto

El módulo `str` proporciona funciones para manipular cadenas de texto.

## Funciones

### `str.longitud(cadena)`

Retorna la longitud en bytes de la cadena.

```argo
print(str.longitud("Hola"));  // 4
```

---

### `str.mayusculas(cadena)`

Retorna la cadena en mayúsculas.

```argo
print(str.mayusculas("hola"));  // "HOLA"
```

---

### `str.minusculas(cadena)`

Retorna la cadena en minúsculas.

```argo
print(str.minusculas("HOLA"));  // "hola"
```

---

### `str.recortar(cadena)`

Elimina espacios en blanco al inicio y final.

```argo
print(str.recortar("  hola  "));  // "hola"
```

---

### `str.dividir(cadena, separador)`

Divide la cadena por el separador y retorna un arreglo.

```argo
let partes = str.dividir("a,b,c", ",");
print(partes);  // ["a", "b", "c"]
```

---

### `str.contiene(cadena, subcadena)`

Retorna `true` si la cadena contiene la subcadena.

```argo
print(str.contiene("Hola mundo", "mundo"));  // true
```

---

### `str.subcadena(cadena, inicio, fin?)`

Extrae una subcadena desde `inicio` hasta `fin` (o el final).

```argo
print(str.subcadena("Hola", 1, 3));  // "ol"
print(str.subcadena("Hola", 2));     // "la"
```

---

### `str.reemplazar(cadena, objetivo, reemplazo)`

Reemplaza todas las ocurrencias de `objetivo` con `reemplazo`.

```argo
print(str.reemplazar("hola mundo", "mundo", "Argo"));  // "hola Argo"
```

---

### `str.empieza_con(cadena, prefijo)`

Retorna `true` si la cadena empieza con el prefijo.

```argo
print(str.empieza_con("Hola", "Ho"));  // true
```

---

### `str.termina_con(cadena, sufijo)`

Retorna `true` si la cadena termina con el sufijo.

```argo
print(str.termina_con("Hola", "la"));  // true
```

---

### `str.indice_de(cadena, subcadena, inicio?)`

Retorna el índice de la primera ocurrencia, o `-1` si no existe.

```argo
print(str.indice_de("Hola mundo", "mundo"));  // 5
print(str.indice_de("Hola mundo", "x"));     // -1
```

---

### `str.invertir(cadena)`

Invierte la cadena carácter por carácter.

```argo
print(str.invertir("Hola"));  // "aloH"
```

---

### `str.repetir(cadena, veces)`

Repite la cadena `veces` veces.

```argo
print(str.repetir("Ha", 3));  // "HaHaHa"
```

---

### `str.a_arreglo(cadena)`

Divide la cadena en un arreglo de caracteres individuales.

```argo
let chars = str.a_arreglo("ABC");
print(chars);  // ["A", "B", "C"]
```

---

### `str.codigo_en(cadena, indice)`

Retorna el punto de código Unicode en la posición `indice`.

```argo
print(str.codigo_en("A", 0));  // 65
```

---

### `str.de_codigo(codigo)`

Convierte un punto de código Unicode a una cadena de un carácter.

```argo
print(str.de_codigo(65));  // "A"
```
