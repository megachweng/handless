<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  Chuyển giọng nói thành văn bản miễn phí, mã nguồn mở cho macOS.<br/>
  Nhấn phím tắt, nói, nhận văn bản trong bất kỳ ứng dụng nào. Chạy cục bộ để bảo mật hoặc sử dụng API đám mây.
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
  <b>Tiếng Việt</b> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Tính năng

- **Phiên âm cục bộ** -- tải mô hình trong Cài đặt, chạy hoàn toàn trên thiết bị
- **Cloud STT** qua OpenAI hoặc Soniox
- **Phát hiện hoạt động giọng nói** (chỉ mô hình cục bộ)
- **Hậu xử lý LLM** để chỉnh sửa hoặc định dạng lại bản phiên âm
- **macOS** (Intel & Apple Silicon)
- **17 ngôn ngữ**

## Cài đặt

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

Biên dịch từ mã nguồn: xem [BUILD.md](../BUILD.md).

## CLI

**Điều khiển từ xa** (giao tiếp với phiên bản đang chạy):

```bash
handless --toggle-transcription    # Bật/tắt ghi âm
handless --toggle-post-process     # Bật/tắt ghi âm + hậu xử lý
handless --cancel                  # Hủy thao tác hiện tại
```

**Tùy chọn khởi động:**

```bash
handless --start-hidden            # Không hiện cửa sổ chính
handless --no-tray                 # Không hiện biểu tượng khay
handless --debug                   # Ghi log chi tiết
handless --help                    # Tất cả tùy chọn
```

Kết hợp tùy ý: `handless --start-hidden --no-tray`

> **macOS:** gọi trực tiếp tệp nhị phân: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Khắc phục sự cố

`Cmd+Shift+D` mở bảng gỡ lỗi.

## Đóng góp

Xem [CONTRIBUTING.md](../CONTRIBUTING.md). Về bản dịch: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Giấy phép

[MIT](../LICENSE)

## Lời cảm ơn

Phát triển từ [Handy](https://github.com/cjpais/Handy) v0.7.8.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
