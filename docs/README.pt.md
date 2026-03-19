<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Conversão de voz em texto para macOS, gratuita e de código aberto.<br/>
  Pressione um atalho, fale e obtenha texto em qualquer aplicativo. Execute localmente para privacidade ou use APIs na nuvem.
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
  <b>Português</b> ·
  <a href="README.ru.md">Русский</a> ·
  <a href="README.ar.md">العربية</a> ·
  <a href="README.it.md">Italiano</a> ·
  <a href="README.tr.md">Türkçe</a> ·
  <a href="README.uk.md">Українська</a> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Funcionalidades

- **Transcrição local** -- baixe modelos nas Configurações, executa totalmente no dispositivo
- **STT na nuvem** via OpenAI ou Soniox
- **Detecção de atividade de voz** (apenas modelos locais)
- **Pós-processamento com LLM** para limpar ou reformatar transcrições
- **macOS** (Intel & Apple Silicon)
- **17 idiomas**

## Instalação

**[Baixar para macOS](https://github.com/ElwinLiu/handless/releases/latest)** (Intel e Apple Silicon)

Compilar a partir do código-fonte: consulte [BUILD.md](../BUILD.md).

## Linha de comando

**Controle remoto** (comunica-se com uma instância em execução):

```bash
handless --toggle-transcription    # Alternar gravação
handless --toggle-post-process     # Alternar gravação + pós-processamento
handless --cancel                  # Cancelar operação atual
```

**Opções de inicialização:**

```bash
handless --start-hidden            # Sem janela principal
handless --no-tray                 # Sem ícone na bandeja
handless --debug                   # Registro detalhado
handless --help                    # Todas as opções
```

Combine livremente: `handless --start-hidden --no-tray`

> **macOS:** invoque o binário diretamente: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Solução de problemas

`Cmd+Shift+D` abre o painel de depuração.

## Contribuindo

Consulte [CONTRIBUTING.md](../CONTRIBUTING.md). Para traduções: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Licença

[MIT](../LICENSE)

## Agradecimentos

Derivado de [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
