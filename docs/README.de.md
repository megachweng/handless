<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Kostenlose, quelloffene Sprache-zu-Text-Anwendung für macOS.<br/>
  Tastenkombination drücken, sprechen, Text in jeder App erhalten. Lokal ausführen für Datenschutz oder Cloud-APIs nutzen.
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
  <b>Deutsch</b> ·
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

## Funktionen

- **Lokale Transkription** -- Modelle in den Einstellungen herunterladen, läuft vollständig auf dem Gerät
- **Cloud-Spracherkennung** über OpenAI oder Soniox
- **Sprachaktivitätserkennung** (nur bei lokalen Modellen)
- **LLM-Nachbearbeitung** zum Bereinigen oder Umformatieren von Transkriptionen
- **macOS** (Intel & Apple Silicon)
- **17 Sprachen**

## Installation

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

Aus dem Quellcode erstellen: siehe [BUILD.md](../BUILD.md).

## Befehlszeile

**Fernsteuerung** (kommuniziert mit einer laufenden Instanz):

```bash
handless --toggle-transcription    # Aufnahme umschalten
handless --toggle-post-process     # Aufnahme + Nachbearbeitung umschalten
handless --cancel                  # Aktuellen Vorgang abbrechen
```

**Startoptionen:**

```bash
handless --start-hidden            # Kein Hauptfenster
handless --no-tray                 # Kein Tray-Symbol
handless --debug                   # Ausführliche Protokollierung
handless --help                    # Alle Optionen
```

Frei kombinierbar: `handless --start-hidden --no-tray`

> **macOS:** Die Binärdatei direkt aufrufen: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Fehlerbehebung

`Cmd+Shift+D` öffnet das Debug-Panel.

## Mitwirken

Siehe [CONTRIBUTING.md](../CONTRIBUTING.md). Für Übersetzungen: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Lizenz

[MIT](../LICENSE)

## Danksagungen

Abgezweigt von [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
