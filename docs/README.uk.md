<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Безкоштовний застосунок з відкритим вихідним кодом для перетворення мовлення в текст на macOS.<br/>
  Натисніть комбінацію клавіш, промовте текст і отримайте його в будь-якому застосунку. Працюйте локально для конфіденційності або використовуйте хмарні API.
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
  <a href="README.ru.md">Русский</a> ·
  <a href="README.ar.md">العربية</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.tr.md">Türkçe</a> ·
  <b>Українська</b> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Можливості

- **Локальна транскрипція** -- завантажте моделі в налаштуваннях, працює повністю на пристрої
- **Хмарне розпізнавання мовлення** через OpenAI або Soniox
- **Виявлення голосової активності** (лише для локальних моделей)
- **Постобробка за допомогою LLM** для очищення або переформатування транскрипцій
- **macOS** (Intel та Apple Silicon)
- **17 мов**

## Встановлення

**[Завантажити для macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel та Apple Silicon)

Збірка з вихідного коду: див. [BUILD.md](../BUILD.md).

## Командний рядок

**Віддалене керування** (взаємодія з запущеним екземпляром):

```bash
handless --toggle-transcription    # Перемкнути запис
handless --toggle-post-process     # Перемкнути запис + постобробку
handless --cancel                  # Скасувати поточну операцію
```

**Прапорці запуску:**

```bash
handless --start-hidden            # Без головного вікна
handless --no-tray                 # Без значка в треї
handless --debug                   # Детальне логування
handless --help                    # Усі прапорці
```

Комбінуйте вільно: `handless --start-hidden --no-tray`

> **macOS:** викликайте бінарний файл безпосередньо: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Усунення несправностей

`Cmd+Shift+D` відкриває панель налагодження.

## Участь у проєкті

Див. [CONTRIBUTING.md](../CONTRIBUTING.md). Щодо перекладів: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Ліцензія

[MIT](../LICENSE)

## Подяки

Форк [Handy](https://github.com/cjpais/Handy) версії 0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
