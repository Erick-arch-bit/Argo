# Módulo `math` — Matemáticas

El módulo `math` proporciona funciones matemáticas estándar. Todas las funciones trigonométricas trabajan en radianes.

## Constantes

| Constante | Valor | Descripción |
|-----------|-------|-------------|
| `math.PI` | 3.141592653589793 | Número π |
| `math.E` | 2.718281828459045 | Número de Euler |

## Funciones Trigonométricas

### `math.sin(x)`

Seno de `x` (radianes).

```argo
print(math.sin(math.PI / 2));  // 1
```

---

### `math.cos(x)`

Coseno de `x` (radianes).

```argo
print(math.cos(0));  // 1
```

---

### `math.tan(x)`

Tangente de `x` (radianes).

```argo
print(math.tan(math.PI / 4));  // ~1
```

## Funciones Básicas

### `math.sqrt(x)`

Raíz cuadrada.

```argo
print(math.sqrt(16));  // 4
```

---

### `math.abs(x)`

Valor absoluto.

```argo
print(math.abs(-5));  // 5
```

---

### `math.pow(base, exp)`

Potencia: `base ^ exp`.

```argo
print(math.pow(2, 10));  // 1024
```

---

### `math.log(x)`

Logaritmo natural.

```argo
print(math.log(math.E));  // 1
```

---

### `math.log10(x)`

Logaritmo base 10.

```argo
print(math.log10(100));  // 2
```

---

### `math.floor(x)`

Redondeo hacia abajo.

```argo
print(math.floor(3.7));  // 3
```

---

### `math.ceil(x)`

Redondeo hacia arriba.

```argo
print(math.ceil(3.2));  // 4
```

---

### `math.round(x)`

Redondeo al entero más cercano.

```argo
print(math.round(3.5));  // 4
```

---

### `math.max(a, b)`

Máximo de dos valores.

```argo
print(math.max(10, 20));  // 20
```

---

### `math.min(a, b)`

Mínimo de dos valores.

```argo
print(math.min(10, 20));  // 10
```

---

### `math.random(min, max)`

Número pseudo-aleatorio entero en el rango `[min, max]`.

```argo
let dado = math.random(1, 6);
print(dado);
```
