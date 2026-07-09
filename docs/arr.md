# Módulo `arr` — Operaciones con Arreglos

El módulo `arr` proporciona funciones para manipular arreglos (listas) en Argo.

## Funciones

### `arr.len(arreglo)`

Retorna la longitud del arreglo.

```argo
let nums = [1, 2, 3];
print(arr.len(nums));  // 3
```

---

### `arr.push(arreglo, elemento)`

Retorna un nuevo arreglo con el elemento agregado al final.

```argo
let nums = [1, 2];
let nums2 = arr.push(nums, 3);
print(nums2);  // [1, 2, 3]
```

---

### `arr.pop(arreglo)`

Remueve y retorna el último elemento como `{ elemento: <valor>, arreglo: <resto> }`.

```argo
let nums = [1, 2, 3];
let resultado = arr.pop(nums);
print(resultado.elemento);  // 3
print(resultado.arreglo);   // [1, 2]
```

---

### `arr.contiene(arreglo, elemento)`

Retorna `true` si el arreglo contiene un elemento igual.

```argo
let nums = [1, 2, 3];
print(arr.contiene(nums, 2));  // true
print(arr.contiene(nums, 5));  // false
```

---

### `arr.invertir(arreglo)`

Retorna un nuevo arreglo con los elementos en orden inverso.

```argo
let nums = [1, 2, 3];
print(arr.invertir(nums));  // [3, 2, 1]
```

---

### `arr.primero(arreglo)`

Retorna el primer elemento, o `Nulo` si está vacío.

```argo
let nums = [10, 20, 30];
print(arr.primero(nums));  // 10
```

---

### `arr.ultimo(arreglo)`

Retorna el último elemento, o `Nulo` si está vacío.

```argo
let nums = [10, 20, 30];
print(arr.ultimo(nums));  // 30
```

---

### `arr.concatenar(arr1, arr2)`

Retorna un nuevo arreglo combinando ambos.

```argo
let a = [1, 2];
let b = [3, 4];
print(arr.concatenar(a, b));  // [1, 2, 3, 4]
```

---

### `arr.vacio(arreglo)`

Retorna `true` si el arreglo está vacío.

```argo
print(arr.vacio([]));      // true
print(arr.vacio([1]));     // false
```

---

### `arr.indice_de(arreglo, elemento)`

Retorna el índice del primer elemento encontrado, o `-1` si no existe.

```argo
let nums = [10, 20, 30];
print(arr.indice_de(nums, 20));  // 1
print(arr.indice_de(nums, 50));  // -1
```

---

### `arr.plano(arreglo)`

Aplana un nivel de anidamiento.

```argo
let nested = [[1, 2], [3, 4]];
print(arr.plano(nested));  // [1, 2, 3, 4]
```

---

### `arr.map(arreglo, funcion)`

Aplica una función a cada elemento y retorna un nuevo arreglo con los resultados.

```argo
let nums = [1, 2, 3];
let dobles = arr.map(nums, fn(x) { x * 2 });
print(dobles);  // [2, 4, 6]
```

---

### `arr.filter(arreglo, funcion)`

Retorna un nuevo arreglo con los elementos donde la función retorna un valor verdadero.

```argo
let nums = [1, 2, 3, 4, 5];
let pares = arr.filter(nums, fn(x) { x % 2 == 0 });
print(pares);  // [2, 4]
```

---

### `arr.reduce(arreglo, funcion, inicial?)`

Reduce el arreglo a un solo valor usando una función acumuladora.

```argo
let nums = [1, 2, 3, 4];
let suma = arr.reduce(nums, fn(acc, x) { acc + x }, 0);
print(suma);  // 10
```

---

### `arr.find(arreglo, funcion)`

Retorna el primer elemento donde la función retorna verdadero, o `Nulo`.

```argo
let nums = [1, 2, 3, 4];
let par = arr.find(nums, fn(x) { x % 2 == 0 });
print(par);  // 2
```

---

### `arr.every(arreglo, funcion)`

Retorna `true` si todos los elementos pasan la prueba.

```argo
let nums = [2, 4, 6];
print(arr.every(nums, fn(x) { x % 2 == 0 }));  // true
```

---

### `arr.some(arreglo, funcion)`

Retorna `true` si al menos un elemento pasa la prueba.

```argo
let nums = [1, 2, 3];
print(arr.some(nums, fn(x) { x % 2 == 0 }));  // true
```
