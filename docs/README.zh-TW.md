<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  免費、開源的 macOS 語音轉文字工具。<br/>
  按下快捷鍵，說話，即可在任意應用程式中取得文字。支援本地執行以保護隱私，也可使用雲端 API。
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
  <b>繁體中文</b> ·
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
  <a href="README.cs.md">Čeština</a>
</p>

## 功能特色

- **本地轉錄** -- 在設定中下載模型，完全在裝置上執行
- **雲端語音轉文字**，支援 OpenAI 或 Soniox
- **語音活動偵測**（僅限本地模型）
- **LLM 後處理**，用於清理或重新格式化轉錄內容
- **macOS**（Intel 與 Apple Silicon）
- **17 種語言**

## 安裝

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

從原始碼建置：請參閱 [BUILD.md](../BUILD.md)。

## 命令列

**遠端控制**（與正在執行的實例通訊）：

```bash
handless --toggle-transcription    # 切換錄音
handless --toggle-post-process     # 切換錄音 + 後處理
handless --cancel                  # 取消目前操作
```

**啟動參數：**

```bash
handless --start-hidden            # 不顯示主視窗
handless --no-tray                 # 不顯示系統匣圖示
handless --debug                   # 詳細日誌輸出
handless --help                    # 所有參數
```

可自由組合：`handless --start-hidden --no-tray`

> **macOS：** 直接呼叫二進位檔案：`/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## 疑難排解

`Cmd+Shift+D` 開啟除錯面板。

## 參與貢獻

請參閱 [CONTRIBUTING.md](../CONTRIBUTING.md)。翻譯相關：[CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md)。

## 授權條款

[MIT](../LICENSE)

## 致謝

基於 [Handy](https://github.com/cjpais/Handy) v0.7.8 分叉開發。

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
