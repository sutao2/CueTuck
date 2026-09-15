<p align="center">
  <img src="../../desktop/src/assets/app-icon.png" width="88" alt="CueTuck">
</p>

# CueTuck · 唤词

**Your prompts, a shortcut away.**

[简体中文](../../README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · **Español** · [Français](README.fr.md) · [Deutsch](README.de.md)

Tu lanzador de prompts para escritorio. Abre CueTuck con un atajo global, busca una plantilla, completa sus variables y copia el resultado a tu herramienta de IA. La biblioteca local, las imágenes de referencia y la gestión de Skills mantienen tus flujos de trabajo a mano.

[Sitio web](https://prompt.likh.cn/) · [Descargar versión preliminar](https://github.com/sutao2/CueTuck/releases) · [Documentación](../../docs/INDEX.md) · [Informar de un problema](https://github.com/sutao2/CueTuck/issues)

![Lanzador independiente de CueTuck: búsqueda de prompts y acciones rápidas](../../docs/assets/readme/launcher-search.png)

## Sigue trabajando con un solo atajo

Pulsa **Control + Space** de forma predeterminada (configurable en Ajustes), escribe una palabra clave y selecciona con **↑ / ↓**. **Enter** copia los prompts sin variables; si las tienen, complétalas, revisa la vista previa y copia el resultado. También puedes crear un prompt, optimizar el texto con tu propio modelo de IA o buscar en la comunidad.

## Funciones

| Función | Qué permite |
|---|---|
| Lanzador independiente | Abrir con un atajo global configurable, buscar, completar variables y copiar. Crear prompts, mejorarlos con IA o buscar explícitamente en el catálogo. |
| Catálogo de prompts | Explorar por categoría, modelo o palabra clave; consultar imágenes, fuentes y autores; descargar o guardar favoritos. |
| Biblioteca local | Clasificar, buscar, marcar favoritos y editar texto y referencias. Alternar entre tarjetas y lista. |
| Traducción e IA | Alternar entre chino, original e inglés en el catálogo. Usar tu propio modelo para traducción y mejora locales, eligiéndolo de una lista obtenida del proveedor. |
| Gestión de Skills | Explorar fuentes públicas, descubrir Skills locales, gestionar agente y ámbito, instalar, restaurar copias y buscar actualizaciones manualmente. |
| Integración MCP | Permitir que agentes compatibles busquen, lean y completen prompts locales. Las herramientas del catálogo público son opcionales. |

## De la organización al uso

### Plantillas guardadas localmente

La aplicación de escritorio utiliza SQLite. Las plantillas admiten `{{variables}}` y referencias. Inicia sesión cuando necesites favoritos del catálogo, publicación o sincronización de tu biblioteca.

![Biblioteca local con prompts de ejemplo](../../docs/assets/readme/library.png)

### Completa, revisa y copia

Adapta una plantilla a otro objetivo, público o material. Revisa el resultado antes de copiarlo, conservando la plantilla para el siguiente uso.

![Vista previa de un prompt con variables completadas](../../docs/assets/readme/variables.png)

### Siempre a un atajo de distancia

El lanzador tiene su propia ventana y navegación por teclado. Completa las variables y pulsa Enter para copiar. También puedes convertir una idea nueva en un prompt o solicitar una mejora con IA.

![Variables en el lanzador independiente](../../docs/assets/readme/launcher.png)

> Las capturas muestran el código actual en la vista previa del navegador. El catálogo contiene material público; la biblioteca y el lanzador usan ejemplos. SQLite nativo, los atajos del sistema y la instalación de archivos requieren la aplicación de escritorio. Los paquetes publicados pueden ir por detrás del código actual.

## Instalación y actualizaciones

Descarga el archivo correspondiente en [GitHub Releases](https://github.com/sutao2/CueTuck/releases).

| Plataforma | Paquete | Compatibilidad |
|---|---|---|
| macOS · Apple Silicon | `.dmg` | arm64; verificado en un Mac real |
| Windows | `.exe` | x64; instalación, inicio y desinstalación comprobados en CI |
| Linux | — | Todavía sin verificar |

En **Ajustes → Actualizaciones** puedes comprobar versiones, ver el progreso de descarga y confirmar la instalación. Es una versión preliminar. Los paquetes de macOS tienen firma ad hoc, sin notarización de Apple; consulta las [notas de instalación](../../deploy/README.md).

## Primeros pasos

1. Crea un prompt local o descarga una plantilla del catálogo.
2. Elige **Usar**, completa las variables y copia el resultado a tu herramienta de IA.
3. Configura el atajo, el idioma y la apariencia en Ajustes.
4. En **IA y modelos**, introduce la URL del servicio y tu clave API; obtén la lista y selecciona un modelo.
5. Para sincronizar, guardar favoritos del catálogo o publicar, inicia sesión y establece un apodo público.

La configuración local de IA no implica inferencia sin conexión. Las claves se guardan en el almacén de credenciales del sistema; la traducción y la mejora envían el texto correspondiente al proveedor elegido. Revisa el texto y los adjuntos públicos antes de publicar.

## Skills y MCP

Gestiona carpetas de Skills para Codex, Claude Code, Cursor, Pi y OpenCode, en ámbitos globales o de proyecto. La instalación y sustitución incluyen confirmación y copias de seguridad. Instalar un Skill no instala automáticamente sus dependencias MCP ni ejecuta sus scripts. Consulta la [especificación de Skills](../../docs/specs/skills/spec.md).

MCP es un **servicio stdio en Rust** que se compila por separado; no necesita Node.js ni uv para ejecutarse. Por defecto solo expone herramientas locales de lectura. Genera la configuración en **Ajustes → Red y proxy → Integración MCP para agentes** y sigue la [guía MCP](../../docs/how-to/mcp-clients.md).

## Desarrollo

Necesitas Node.js 22+. Para la aplicación nativa también hacen falta Rust y las dependencias de compilación de Tauri de tu plataforma.

```bash
git clone https://github.com/sutao2/CueTuck.git
cd CueTuck/desktop
npm ci
npm test
npm run dev
```

Esto inicia la vista previa del navegador. Detén ese servidor antes de ejecutar `npm run tauri dev` para abrir la aplicación nativa.

[Desarrollo local](../../docs/how-to/local-dev.md) · [Despliegue y distribución](../../deploy/README.md) · [Contribuir](../../CONTRIBUTING.md) · [Criterios de prueba](../../docs/reference/test-gates.md)

La documentación técnica está principalmente en chino. Al informar de un fallo, incluye sistema, versión, pasos de reproducción y capturas sin datos personales. No incluyas claves ni tokens de sesión.
