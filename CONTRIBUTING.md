# Contribuir a Argo

Gracias por tu interés en contribuir a Argo. Este documento describe el flujo de trabajo y los lineamientos para contribuir.

## Flujo de trabajo

1. **Haz un fork** del repositorio y crea tu rama desde `dev`.
2. **Desarrolla en tu rama** siguiendo los estándares del código.
3. **Asegúrate de que compile** sin errores ni advertencias:
   ```bash
   cargo build
   cargo clippy
   ```
4. **Prueba tus cambios** ejecutando el intérprete con los scripts de ejemplo.
5. **Abre un Pull Request** contra la rama `dev`.

## Estándares de código

- El código debe compilar sin warnings con `cargo clippy`.
- Los identificadores y comentarios van en **español** (coherencia con el código existente).
- Las funciones deben ser relativamente cortas y con una sola responsabilidad.
- No agregues dependencias externas sin discusión previa.
- Los mensajes de error deben ser descriptivos y en español.

## Ramas

- `dev` — Rama principal de desarrollo.
- `contribuciones` — Rama de integración para características del lenguaje aportadas por la comunidad. Los PRs de nuevas funcionalidades deben dirigirse aquí.
- `seguridad` — Rama para parches y mejoras de seguridad.

## Reportar issues

Usa el rastreador de issues del repositorio para reportar errores o solicitar características. Incluye:
- Una descripción clara del problema o sugerencia.
- Código de ejemplo que reproduzca el error (si aplica).
- El resultado esperado y el resultado obtenido.

## Licencia

Al contribuir, aceptas que tus contribuciones se licencien bajo los términos de la [MIT License](LICENSE).
