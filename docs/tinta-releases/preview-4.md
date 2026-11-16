# Tinta Presenta — Preview 4

> **Status:** 2026-11-16 · draft for the Preview 4 release (10 December 2026), version 0.19.0, with
> the 16 November upstream sync · **Revision history:** 2026-11-16 created (changes from Preview 3,
> 12 November, to #36). Updated at each weekly sync until the freeze on 7 December.

Release notes for the GitHub release and the site, in Spanish, Ukrainian and English. Changes since
Preview 3 (v0.15.0, 12 November 2026). Before publishing: build the Windows installers with
`packaging/windows/tinta-build-release.ps1` (in pdfcraft) and run the smoke test on them.

## Español

**Tinta Presenta — vista previa 4.** Versión de prueba: úsela con copias de sus presentaciones.

Novedades:

- **Imágenes SVG:** los iconos y dibujos SVG de PowerPoint se ven y se conservan al guardar, y puede
  insertar archivos .svg.
- **Vídeo y audio:** las opciones de reproducción (volumen, repetir, pantalla completa, inicio
  automático…) se conservan al guardar en .pptx. Un vídeo que esta versión no puede reproducir
  (H.264, HEVC…) se puede convertir a WebM con su propio FFmpeg, sin salir de la presentación.
- **Más fiel a PowerPoint:** el texto de SmartArt y las leyendas de los gráficos se conservan; copiar, duplicar y desagrupar formas dentro
  de grupos funciona como en PowerPoint.
- **Asistentes y automatización:** cada comando indica qué puede hacer, y una sesión puede limitarse
  a ciertos comandos.
- Del proyecto original (DeckCraft): muchas correcciones de gráficos, imágenes y revisión ortográfica.

Limitaciones conocidas: los vídeos H.264 y HEVC no se reproducen dentro de la aplicación (por las
patentes de esos formatos): conviértalos a WebM o ábralos en el reproductor del sistema; la conversión
necesita FFmpeg instalado.

## Українська

**Tinta Presenta — попередня версія 4.** Тестова версія: працюйте з копіями презентацій.

Що нового:

- **Зображення SVG:** SVG-піктограми й малюнки з PowerPoint показуються і зберігаються, а файли .svg
  можна вставляти.
- **Відео й аудіо:** параметри відтворення (гучність, повтор, на весь екран, автозапуск…)
  зберігаються в .pptx. Відео, яке ця версія не відтворює (H.264, HEVC…), можна перетворити на WebM
  власним FFmpeg, не виходячи з презентації.
- **Ближче до PowerPoint:** текст SmartArt і легенди діаграм зберігаються; копіювання, дублювання й розгрупування фігур у
  групах працюють як у PowerPoint.
- **Асистенти й автоматизація:** кожна команда повідомляє, що вона може робити, а сеанс можна
  обмежити певними командами.
- З основного проєкту (DeckCraft): багато виправлень діаграм, зображень і перевірки правопису.

Відомі обмеження: відео H.264 і HEVC не відтворюються в самій програмі (через патенти на ці
формати): перетворіть їх на WebM або відкрийте в системному програвачі; для перетворення потрібен
встановлений FFmpeg.

## English

**Tinta Presenta — Preview 4.** A test version: work on copies of your presentations.

What's new:

- **SVG pictures:** PowerPoint's SVG icons and drawings show and are kept when you save, and you can
  insert .svg files.
- **Video and audio:** playback options (volume, loop, full screen, start automatically…) are kept
  when saving to .pptx. A video this version can't play (H.264, HEVC…) can be converted to WebM with
  your own FFmpeg without leaving the presentation.
- **Closer to PowerPoint:** SmartArt text and chart legends are kept; copying, duplicating and ungrouping shapes inside groups
  works as in PowerPoint.
- **Assistants and automation:** every command says what it may do, and a session can be limited
  to some commands.
- From the original project (DeckCraft): many chart, picture and spelling fixes.

Known limits: H.264 and HEVC videos don't play inside the app (those formats are patented): convert
them to WebM or open them in the system's player; conversion needs FFmpeg installed.
