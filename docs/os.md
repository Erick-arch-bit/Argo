# Módulo `os` — Sistema Operativo

El módulo `os` proporciona acceso al sistema operativo y al entorno de ejecución.

## Funciones

### `os.ejecutar(comando)`

Ejecuta un comando del shell y retorna su salida stdout.

```argo
let salida = os.ejecutar("ls -la");
print(salida);
```

---

### `os.variables(llave)`

Retorna el valor de una variable de entorno, o `Nulo` si no existe.

```argo
let home = os.variables("HOME");
print(home);
```

---

### `os.exit(codigo)`

Termina el proceso con el código de salida especificado.

```argo
os.exit(0);
```

---

### `os.directorio_actual()`

Retorna la ruta del directorio de trabajo actual.

```argo
let cwd = os.directorio_actual();
print(cwd);
```

---

### `os.directorio_temporal()`

Retorna la ruta del directorio temporal del sistema.

```argo
let tmp = os.directorio_temporal();
print(tmp);
```

---

### `os.argumentos()`

Retorna los argumentos de línea de comandos como un arreglo.

```argo
let args = os.argumentos();
print(args);
```

---

### `os.procesadores()`

Retorna el número de cores de CPU disponibles.

```argo
let cores = os.procesadores();
print("CPU cores:", cores);
```
