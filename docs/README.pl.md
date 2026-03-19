<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Darmowa, otwartoźródłowa zamiana mowy na tekst dla macOS.<br/>
  Naciśnij skrót, mów, otrzymaj tekst w dowolnej aplikacji. Działaj lokalnie dla prywatności lub korzystaj z API chmurowych.
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
  <a href="README.uk.md">Українська</a> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <b>Polski</b> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Funkcje

- **Lokalna transkrypcja** -- pobierz modele w Ustawieniach, działa w pełni na urządzeniu
- **Chmurowe STT** przez OpenAI lub Soniox
- **Detekcja aktywności głosowej** (tylko modele lokalne)
- **Przetwarzanie końcowe LLM** do czyszczenia lub przeformatowania transkrypcji
- **macOS** (Intel & Apple Silicon)
- **17 języków**

## Instalacja

**[Pobierz na macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel i Apple Silicon)

Kompilacja ze źródeł: zobacz [BUILD.md](../BUILD.md).

## CLI

**Zdalne sterowanie** (komunikuje się z działającą instancją):

```bash
handless --toggle-transcription    # Przełącz nagrywanie
handless --toggle-post-process     # Przełącz nagrywanie + przetwarzanie końcowe
handless --cancel                  # Anuluj bieżącą operację
```

**Flagi uruchomieniowe:**

```bash
handless --start-hidden            # Bez okna głównego
handless --no-tray                 # Bez ikony w zasobniku
handless --debug                   # Szczegółowe logowanie
handless --help                    # Wszystkie flagi
```

Łącz dowolnie: `handless --start-hidden --no-tray`

> **macOS:** wywołaj plik binarny bezpośrednio: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Rozwiązywanie problemów

`Cmd+Shift+D` otwiera panel debugowania.

## Współtworzenie

Zobacz [CONTRIBUTING.md](../CONTRIBUTING.md). Dla tłumaczeń: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Licencja

[MIT](../LICENSE)

## Podziękowania

Rozwidlenie z [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
