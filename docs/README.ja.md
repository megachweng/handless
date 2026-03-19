<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  無料・オープンソースのmacOS音声入力アプリ。<br/>
  ショートカットを押して話すだけで、あらゆるアプリにテキストを入力。プライバシーのためにローカルで実行するか、クラウドAPIを使用できます。
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
  <strong>日本語</strong>
</p>

## 機能

- **ローカル文字起こし** -- 設定からモデルをダウンロードし、完全にデバイス上で動作
- **クラウドSTT** -- OpenAIまたはSoniox経由
- **音声アクティビティ検出** (ローカルモデルのみ)
- **LLM後処理** -- 文字起こしの整形や再フォーマット
- **macOS** (Intel & Apple Silicon)
- **17言語対応**

## インストール

**[macOS 版をダウンロード](https://github.com/ElwinLiu/handless/releases/latest)**（Intel & Apple Silicon）

ソースからビルドする場合は[BUILD.md](../BUILD.md)を参照してください。

## CLI

**リモートコントロール** (実行中のインスタンスと通信):

```bash
handless --toggle-transcription    # 録音の切り替え
handless --toggle-post-process     # 録音+後処理の切り替え
handless --cancel                  # 現在の操作をキャンセル
```

**起動フラグ:**

```bash
handless --start-hidden            # メインウィンドウなし
handless --no-tray                 # トレイアイコンなし
handless --debug                   # 詳細ログ
handless --help                    # 全フラグ一覧
```

自由に組み合わせ可能: `handless --start-hidden --no-tray`

> **macOS:** バイナリを直接実行してください: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## トラブルシューティング

`Cmd+Shift+D` でデバッグパネルを開きます。

## コントリビューション

[CONTRIBUTING.md](../CONTRIBUTING.md)を参照してください。翻訳については[CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md)を参照してください。

## ライセンス

[MIT](../LICENSE)

## 謝辞

[Handy](https://github.com/cjpais/Handy) v0.7.8からフォーク。

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
