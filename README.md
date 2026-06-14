# Argo

Argo es un lenguaje de programación interpretado, minimalista y extensible, diseñado para ser simple por dentro y por fuera. Está escrito en Rust sin dependencias externas.

## Características

- Variables y asignación
- Tipos: enteros, flotantes, booleanos, cadenas, nulo
- Operadores aritméticos (`+`, `-`, `*`, `/`), lógicos (`&&`, `||`, `!`) y relacionales (`==`, `!=`, `<`, `>`, `<=`, `>=`)
- Estructuras de control: `if`, `else`, `while`, `for`
- `break` para salir de bucles
- Funciones con entorno cerrado (closures)
- Listas (arrays) y diccionarios
- Módulo matemático: `sin`, `cos`, `sqrt`, `abs`, `PI`, `E`
- Sistema de comentarios: `//` línea y `/* */` bloque
- REPL interactivo

## Instalación

### Desde el código fuente

```bash
git clone <url-del-repo>
cd argo
cargo build --release
sudo cp target/release/argo /usr/local/bin/
```

### Con el script de instalación

```bash
chmod +x install.sh
./install.sh
```

## Uso

```
argo                  # Inicia el REPL interactivo
argo <archivo.argo>   # Ejecuta un script
argo init             # Crea un nuevo proyecto
argo run              # Ejecuta el proyecto actual
argo repl             # Inicia el REPL explícitamente
```

### Ejemplo

```rust
// ¡Hola Mundo!
print("Hola desde Argo.");

// Variables y matemáticas
PI = 3.1416;
radio = 5;
area = PI * radio * radio;
print("Área:", area);

// Funciones
fn fib(n) {
    if (n <= 1) { return n; }
    return fib(n - 1) + fib(n - 2);
}
print("fib(10):", fib(10));

// Diccionarios y listas
usuario = { "nombre": "Argo", "version": 1.0 };
nums = [1, 2, 3, 4, 5];
```

## Licencia

Argo se distribuye bajo la licencia **MIT**.

Esta es una licencia permisiva que permite usar, copiar, modificar, fusionar, publicar, distribuir, sublicenciar y vender copias del software, siempre que se incluya el aviso de copyright y el descargo de responsabilidad en todas las copias o partes sustanciales del software.

**El software se proporciona "tal cual", sin garantía de ningún tipo.** Ver el archivo [LICENSE](LICENSE) para más detalles.

## Contribuir

Las contribuciones son bienvenidas. Por favor lee [CONTRIBUTING.md](CONTRIBUTING.md) para conocer el flujo de trabajo y los estándares del proyecto.

## Seguridad

Si encuentras una vulnerabilidad de seguridad, por favor lee [SECURITY.md](SECURITY.md) para saber cómo reportarla.
