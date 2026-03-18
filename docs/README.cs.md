<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Bezplatný, open-source převod řeči na text pro macOS.<br/>
  Stiskněte zkratku, mluvte, získejte text v jakékoli aplikaci. Spouštějte lokálně pro soukromí nebo použijte cloudová API.
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
  <a href="README.pl.md">Polski</a> ·
  <b>Čeština</b>
</p>

## Funkce

- **Lokální transkripce** -- stáhněte modely v Nastavení, běží plně na zařízení
- **Cloudové STT** přes OpenAI nebo Soniox
- **Detekce hlasové aktivity** (pouze lokální modely)
- **LLM post-processing** pro úpravu nebo přeformátování transkripcí
- **macOS** (Intel & Apple Silicon)
- **17 jazyků**

## Instalace

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

Sestavení ze zdrojového kódu: viz [BUILD.md](../BUILD.md).

## CLI

**Vzdálené ovládání** (komunikuje s běžící instancí):

```bash
handless --toggle-transcription    # Přepnout nahrávání
handless --toggle-post-process     # Přepnout nahrávání + post-processing
handless --cancel                  # Zrušit aktuální operaci
```

**Příznaky spuštění:**

```bash
handless --start-hidden            # Bez hlavního okna
handless --no-tray                 # Bez ikony v liště
handless --debug                   # Podrobné logování
handless --help                    # Všechny příznaky
```

Libovolně kombinujte: `handless --start-hidden --no-tray`

> **macOS:** spusťte binární soubor přímo: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Řešení problémů

`Cmd+Shift+D` otevře panel ladění.

## Přispívání

Viz [CONTRIBUTING.md](../CONTRIBUTING.md). Pro překlady: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Licence

[MIT](../LICENSE)

## Poděkování

Odštěpeno z [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
