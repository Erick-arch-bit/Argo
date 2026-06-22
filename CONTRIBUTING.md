# Contribuir a Argo

Gracias por tu interés en contribuir a Argo. Este documento describe el flujo de trabajo y los lineamientos para contribuir.

## Flujo de trabajo

### Para contribuidores externos

1. **Haz un fork** del repositorio y crea tu rama desde `contribuciones`.
2. **Desarrolla en tu rama** siguiendo los estándares del código.
3. **Asegúrate de que compile** sin errores ni advertencias:
   ```bash
   cargo build
   cargo clippy
   ```
4. **Prueba tus cambios** ejecutando el intérprete con los scripts de ejemplo.
5. **Abre un Pull Request** contra la rama `contribuciones`.

### Flujo interno de aprobación

1. Los PRs en `contribuciones` y `seguridad` se revisan y se fusionan a `revision`.
2. En `revision` se hacen las pruebas de integración finales.
3. Una vez aprobado, se fusiona de `revision` a `dev`.
4. Los tags `v*` disparan automáticamente el workflow de release, compilando binarios para las 4 plataformas soportadas.

## Estándares de código

- El código debe compilar sin warnings con `cargo clippy`.
- Los identificadores y comentarios van en **español** (coherencia con el código existente).
- Las funciones deben ser relativamente cortas y con una sola responsabilidad.
- No agregues dependencias externas sin discusión previa.
- Los mensajes de error deben ser descriptivos y en español.

## Nuevas características

### Módulos de la biblioteca estándar

Cada módulo de la stdlib debe:
1. Crear un archivo `src/stdlib/<nombre>.rs` con una función `pub fn crear_modulo() -> Objeto`.
2. Registrar el módulo en `src/stdlib/mod.rs` (`pub mod <nombre>;` + inyección en `inyectar_stdlib`).
3. Seguir el patrón de `HashMap<LlaveHash, Objeto>` con funciones nativas.

### Serialización bytecode

Al añadir nuevas variantes a `Expression` o `Statement`:
1. Agregar el OpCode correspondiente en `src/evaluator/bytecode.rs`.
2. Implementar la serialización en `ast_a_bytes` o `sentencia_a_bytes`.
3. Implementar la deserialización en `bytes_a_ast` o `bytes_a_sentencia`.
4. Verificar el formato con el flujo de caché en `src/evaluator/modulo.rs`.

## Ramas

- `dev` — Rama principal de desarrollo. **Protegida**: solo el mantenedor puede hacer push directo; el resto requiere PR aprobado.
- `contribuciones` — Rama de integración para características del lenguaje aportadas por la comunidad. Los PRs de nuevas funcionalidades deben dirigirse aquí.
- `seguridad` — Rama para parches y mejoras de seguridad.
- `revision` — Rama interna de aprobación. Los cambios aprobados en `contribuciones` y `seguridad` se fusionan aquí para pruebas finales antes de pasar a `dev`.

## Release

El mantenedor ejecuta el proceso de release:
1. Actualizar la versión en `Cargo.toml`.
2. Confirmar todos los cambios, crear el tag `vX.Y.Z` y empujarlo.
3. El workflow de CI/CD (`release.yml`) compila binarios para Linux, macOS (ARM + Intel) y Windows, genera checksums SHA256 y publica todo en GitHub Releases.

## Reportar issues

Usa el rastreador de issues del repositorio para reportar errores o solicitar características. Incluye:
- Una descripción clara del problema o sugerencia.
- Código de ejemplo que reproduzca el error (si aplica).
- El resultado esperado y el resultado obtenido.

## Licencia

Al contribuir, aceptas que tus contribuciones se licencien bajo los términos de la [MIT License](LICENSE).
