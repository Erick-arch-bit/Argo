# Política de Seguridad de Argo

## Reportar una vulnerabilidad

Si descubres una vulnerabilidad de seguridad en Argo, **no** abras un issue público. En su lugar, sigue estos pasos:

1. **Contacta al mantenedor** enviando un mensaje privado o un correo electrónico.
2. **Describe el problema** con el mayor detalle posible: pasos para reproducir, impacto potencial y una sugerencia de solución si la tienes.
3. **Espera una respuesta** antes de divulgar públicamente. Trabajaremos para confirmar y parchar la vulnerabilidad en un plazo razonable.
4. **Una vez publicado el parche**, se te reconocerá como responsable del reporte (si así lo deseas).

## Cadena de suministro (Supply Chain)

### Integridad de los binarios publicados

Cada release de Argo publica binarios precompilados para Linux, macOS y Windows, acompañados de su checksum **SHA256**. Antes de instalar un binario, verifica su integridad:

```bash
# Descargar el binario y su checksum
curl -sSLO https://github.com/Erick-arch-bit/Argo-Lang/releases/download/v1.5.1/argo-linux-amd64
curl -sSLO https://github.com/Erick-arch-bit/Argo-Lang/releases/download/v1.5.1/argo-linux-amd64.sha256

# Verificar (Linux)
sha256sum -c argo-linux-amd64.sha256

# Verificar (macOS)
shasum -a 256 -c argo-linux-amd64.sha256
```

### Pipeline de CI/CD

El workflow de release (`.github/workflows/release.yml`) sigue el principio de **mínimo privilegio**:
- Permiso únicamente `contents: write` para crear la release y subir assets
- Todas las acciones de GitHub están ancladas a un **SHA específico** (sin etiquetas mutables)
- Los binarios se compilan de forma aislada por plataforma con caché segmentada
- La publicación se realiza mediante `gh` CLI oficial, sin acciones de terceros no auditadas

### Dependencias

Argo tiene **cero dependencias externas** en tiempo de ejecución. Todo el lenguaje, incluyendo la biblioteca estándar, está implementado exclusivamente en Rust con la biblioteca estándar (`std`). Esto elimina por completo el riesgo de ataques a la cadena de suministro a través de paquetes de terceros.

## Caché de módulos remotos

Las importaciones vía URL (`import "https://..."`) almacenan el código fuente y su AST compilado en `~/.argo/cache/`:
- Los archivos `.argbc` contienen AST serializado, no código ejecutable arbitrario
- La caché es local al usuario que ejecuta Argo
- Solo se accede a las URLs especificadas explícitamente en el código fuente

## Ramas de seguridad

Los parches de seguridad se gestionan en la rama `seguridad`. Una vez probados y aprobados, se fusionan a `dev` y se publican en una nueva versión.

## Prácticas recomendadas

- No ejecutes scripts Argo de fuentes no confiables.
- Verifica los checksums SHA256 de los binarios descargados.
- Mantén tu instalación actualizada con la última versión.
- Si encuentras algún comportamiento inesperado sospechoso, repórtalo siguiendo el proceso anterior.
