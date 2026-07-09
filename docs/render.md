# Módulo `render` — Motor de Renderizado (Interno)

El módulo `render` es el motor de renderizado interno utilizado por `ui` e `ix`. No está diseñado para uso directo desde código Argo.

## Características

- **Doble buffer**: backbuffer y frontbuffer para diffing
- **Dirty-rect tracking**: solo redibuja celdas modificadas
- **AnsiBuf**: buffer de 4KB en stack para escritura ANSI zero-alloc
- **Batch de colores**: solo escribe fg/bg cuando cambian entre celdas

## Estructuras

### `Celda`

Una celda individual en la pantalla.

```rust
struct Celda {
    bg: (u8, u8, u8),      // Color de fondo RGB
    fg: (u8, u8, u8),      // Color de texto RGB
    caracter: char,          // Carácter a mostrar
}
```

### `RenderEngine`

El motor de renderizado principal.

```rust
struct RenderEngine {
    ancho: usize,
    alto: usize,
    backbuffer: Vec<Celda>,
    frontbuffer: Vec<Celda>,
    dirty_cells: Vec<bool>,
}
```

## Métodos Principales

### `RenderEngine::new(ancho, alto)`

Crea un nuevo motor de renderizado.

### `poner_celda(x, y, caracter, fg, bg)`

Escribe una celda individual.

### `poner_texto(x, y, texto, fg, bg)`

Escribe una cadena de texto.

### `rellenar(x, y, w, h, caracter, fg, bg)`

Rellena un rectángulo con un carácter.

### `poner_borde(x, y, w, h, color)`

Dibuja un borde con esquinas rectas: `┌─┐│└─┘`.

### `poner_borde_redondeado(x, y, w, h, color)`

Dibuja un borde con esquinas redondeadas: `╭─╮│╰─╯`.

### `poner_rectangulo(x, y, w, h, bg)`

Rellena un rectángulo con un color sólido.

### `poner_texto_centrado(x, y, w, texto, fg, bg)`

Centra texto dentro del ancho `w`.

### `flush(stdout)`

Renderiza solo las celdas sucias usando buffer ANSI zero-alloc.

### `flush_full(stdout)`

Fuerza un redibujado completo.

### `limpiar_todo()`

Limpia todo el backbuffer.

### `limpiar_area(x, y, w, h)`

Limpia un área rectangular específica.

## Notas de Rendimiento

- El flush usa `AnsiBuf` de 4KB en stack para evitar allocations en el hot path
- Los enteros se convierten a ASCII sin usar `format!`
- El batch de colores reduce las secuencias ANSI escribiendo solo cuando cambian
- El bitset de dirty cells permite skip O(1) por celda limpia
