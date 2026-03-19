<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  免费、开源的 macOS 语音转文字工具。<br/>
  按下快捷键，说话，即可在任意应用中获取文字。支持本地运行以保护隐私，也可使用云端 API。
</p>

<p align="center">
  <a href="https://handless.elwin.cc"><img src="https://img.shields.io/badge/Website-handless.elwin.cc-ef6f2f" alt="Website" /></a>
  <a href="https://github.com/ElwinLiu/handless/actions/workflows/build-test.yml"><img src="https://github.com/ElwinLiu/handless/actions/workflows/build-test.yml/badge.svg" alt="Build" /></a>
  <a href="https://github.com/ElwinLiu/handless/actions/workflows/lint.yml"><img src="https://github.com/ElwinLiu/handless/actions/workflows/lint.yml/badge.svg" alt="Lint" /></a>
  <a href="../LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT" /></a>
</p>

<p align="center">
  <a href="../README.md">English</a> ·
  <b>简体中文</b> ·
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
  <a href="README.cs.md">Čeština</a>
</p>

## 功能特性

- **本地转录** -- 在设置中下载模型，完全在设备上运行
- **云端语音转文字**，支持 OpenAI 或 Soniox
- **语音活动检测**（仅限本地模型）
- **LLM 后处理**，用于清理或重新格式化转录内容
- **macOS**（Intel 与 Apple Silicon）
- **17 种语言**

## 安装

**[下载 macOS 版本](https://github.com/ElwinLiu/handless/releases/latest)**（Intel 和 Apple Silicon）

从源码构建：请参阅 [BUILD.md](../BUILD.md)。

## 命令行

**远程控制**（与正在运行的实例通信）：

```bash
handless --toggle-transcription    # 切换录音
handless --toggle-post-process     # 切换录音 + 后处理
handless --cancel                  # 取消当前操作
```

**启动参数：**

```bash
handless --start-hidden            # 不显示主窗口
handless --no-tray                 # 不显示托盘图标
handless --debug                   # 详细日志输出
handless --help                    # 所有参数
```

可自由组合：`handless --start-hidden --no-tray`

> **macOS：** 直接调用二进制文件：`/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## 故障排除

`Cmd+Shift+D` 打开调试面板。

## 参与贡献

请参阅 [CONTRIBUTING.md](../CONTRIBUTING.md)。翻译相关：[CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md)。

## 许可证

[MIT](../LICENSE)

## 致谢

基于 [Handy](https://github.com/cjpais/Handy) v0.7.8 分叉开发。

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
