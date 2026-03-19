<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Reconnaissance vocale macOS gratuite et open source.<br/>
  Appuyez sur un raccourci, parlez, obtenez du texte dans n'importe quelle application. Fonctionne localement pour la confidentialité ou via des APIs cloud.
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
  <b>Français</b> ·
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

## Fonctionnalités

- **Transcription locale** -- téléchargez des modèles dans les Paramètres, fonctionne entièrement sur l'appareil
- **STT cloud** via OpenAI ou Soniox
- **Détection d'activité vocale** (modèles locaux uniquement)
- **Post-traitement par LLM** pour nettoyer ou reformater les transcriptions
- **macOS** (Intel et Apple Silicon)
- **17 langues**

## Installation

**[Télécharger pour macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel et Apple Silicon)

Compiler depuis les sources : voir [BUILD.md](../BUILD.md).

## CLI

**Contrôle à distance** (communique avec une instance en cours d'exécution) :

```bash
handless --toggle-transcription    # Basculer l'enregistrement
handless --toggle-post-process     # Basculer l'enregistrement + post-traitement
handless --cancel                  # Annuler l'opération en cours
```

**Options de démarrage :**

```bash
handless --start-hidden            # Sans fenêtre principale
handless --no-tray                 # Sans icône dans la barre des menus
handless --debug                   # Journalisation détaillée
handless --help                    # Toutes les options
```

Combinez librement : `handless --start-hidden --no-tray`

> **macOS :** invoquez le binaire directement : `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Dépannage

`Cmd+Shift+D` ouvre le panneau de débogage.

## Contribuer

Voir [CONTRIBUTING.md](../CONTRIBUTING.md). Pour les traductions : [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Licence

[MIT](../LICENSE)

## Remerciements

Dérivé de [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
