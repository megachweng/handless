<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Voz a texto en macOS, gratuito y de código abierto.<br/>
  Pulsa un atajo, habla y obtén texto en cualquier aplicación. Ejecútalo localmente para mayor privacidad o usa APIs en la nube.
</p>

<p align="center">
  <a href="https://handless.elwin.cc"><img src="https://img.shields.io/badge/Website-handless.elwin.cc-ef6f2f" alt="Website" /></a>
  <a href="https://github.com/ElwinLiu/handless/actions/workflows/build-test.yml"><img src="https://github.com/ElwinLiu/handless/actions/workflows/build-test.yml/badge.svg" alt="Build" /></a>
  <a href="https://github.com/ElwinLiu/handless/actions/workflows/lint.yml"><img src="https://github.com/ElwinLiu/handless/actions/workflows/lint.yml/badge.svg" alt="Lint" /></a>
  <a href="../LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" /></a>
</p>

<p align="center">
  <a href="../README.md">English</a> ·
  <a href="README.zh.md">简体中文</a> ·
  <a href="README.zh-TW.md">繁體中文</a> ·
  <a href="README.ja.md">日本語</a> ·
  <a href="README.ko.md">한국어</a> ·
  <b>Español</b> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.pt.md">Português</a> ·
  <a href="README.ru.md">Русский</a> ·
  <a href="README.ar.md">العربية</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.tr.md">Türkçe</a> ·
  <a href="README.uk.md">Українська</a> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Características

- **Transcripción local** -- descarga modelos en Ajustes, se ejecuta completamente en el dispositivo
- **STT en la nube** a través de OpenAI o Soniox
- **Detección de actividad de voz** (solo modelos locales)
- **Posprocesamiento con LLM** para limpiar o reformatear las transcripciones
- **macOS** (Intel y Apple Silicon)
- **17 idiomas**

## Instalación

**[Descargar para macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel y Apple Silicon)

Compilar desde el código fuente: consulta [BUILD.md](../BUILD.md).

## CLI

**Control remoto** (se comunica con una instancia en ejecución):

```bash
handless --toggle-transcription    # Alternar grabación
handless --toggle-post-process     # Alternar grabación + posprocesamiento
handless --cancel                  # Cancelar la operación actual
```

**Opciones de inicio:**

```bash
handless --start-hidden            # Sin ventana principal
handless --no-tray                 # Sin icono en la bandeja
handless --debug                   # Registro detallado
handless --help                    # Todas las opciones
```

Combínalas libremente: `handless --start-hidden --no-tray`

> **macOS:** invoca el binario directamente: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Solución de problemas

`Cmd+Shift+D` abre el panel de depuración.

## Contribuir

Consulta [CONTRIBUTING.md](../CONTRIBUTING.md). Para traducciones: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Licencia

[MIT](../LICENSE)

## Agradecimientos

Derivado de [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
