<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Бесплатное приложение с открытым исходным кодом для преобразования речи в текст на macOS.<br/>
  Нажмите сочетание клавиш, произнесите текст и получите его в любом приложении. Работайте локально для конфиденциальности или используйте облачные API.
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
  <a href="README.es.md">Español</a> ·
  <a href="README.fr.md">Français</a> ·
  <a href="README.de.md">Deutsch</a> ·
  <a href="README.pt.md">Português</a> ·
  <b>Русский</b> ·
  <a href="README.ar.md">العربية</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.tr.md">Türkçe</a> ·
  <a href="README.uk.md">Українська</a> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Возможности

- **Локальная транскрипция** -- загрузите модели в настройках, работает полностью на устройстве
- **Облачное распознавание речи** через OpenAI или Soniox
- **Детекция голосовой активности** (только для локальных моделей)
- **Постобработка с помощью LLM** для очистки или переформатирования транскрипций
- **macOS** (Intel и Apple Silicon)
- **17 языков**

## Установка

**[Скачать для macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel и Apple Silicon)

Сборка из исходного кода: см. [BUILD.md](../BUILD.md).

## Командная строка

**Удалённое управление** (взаимодействие с запущенным экземпляром):

```bash
handless --toggle-transcription    # Переключить запись
handless --toggle-post-process     # Переключить запись + постобработку
handless --cancel                  # Отменить текущую операцию
```

**Флаги запуска:**

```bash
handless --start-hidden            # Без главного окна
handless --no-tray                 # Без значка в трее
handless --debug                   # Подробное логирование
handless --help                    # Все флаги
```

Комбинируйте свободно: `handless --start-hidden --no-tray`

> **macOS:** вызывайте бинарный файл напрямую: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Устранение неполадок

`Cmd+Shift+D` открывает панель отладки.

## Участие в проекте

См. [CONTRIBUTING.md](../CONTRIBUTING.md). По вопросам перевода: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Лицензия

[MIT](../LICENSE)

## Благодарности

Форк [Handy](https://github.com/cjpais/Handy) версии 0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
