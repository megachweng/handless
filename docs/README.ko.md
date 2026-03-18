<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  무료 오픈소스 macOS 음성 입력 앱.<br/>
  단축키를 누르고 말하면 모든 앱에 텍스트가 입력됩니다. 프라이버시를 위해 로컬에서 실행하거나 클라우드 API를 사용할 수 있습니다.
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
  <strong>한국어</strong> ·
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

## 기능

- **로컬 음성 인식** -- 설정에서 모델을 다운로드하여 완전히 기기에서 실행
- **클라우드 STT** -- OpenAI 또는 Soniox 지원
- **음성 활동 감지** (로컬 모델 전용)
- **LLM 후처리** -- 음성 인식 결과 정리 및 재포맷
- **macOS** (Intel & Apple Silicon)
- **17개 언어 지원**

## 설치

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

소스에서 빌드하려면 [BUILD.md](../BUILD.md)를 참조하세요.

## CLI

**원격 제어** (실행 중인 인스턴스와 통신):

```bash
handless --toggle-transcription    # 녹음 전환
handless --toggle-post-process     # 녹음 + 후처리 전환
handless --cancel                  # 현재 작업 취소
```

**시작 플래그:**

```bash
handless --start-hidden            # 메인 창 없이 시작
handless --no-tray                 # 트레이 아이콘 없음
handless --debug                   # 상세 로그
handless --help                    # 모든 플래그 보기
```

자유롭게 조합 가능: `handless --start-hidden --no-tray`

> **macOS:** 바이너리를 직접 실행하세요: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## 문제 해결

`Cmd+Shift+D`를 누르면 디버그 패널이 열립니다.

## 기여하기

[CONTRIBUTING.md](../CONTRIBUTING.md)를 참조하세요. 번역 관련: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## 라이선스

[MIT](../LICENSE)

## 감사의 말

[Handy](https://github.com/cjpais/Handy) v0.7.8에서 포크되었습니다.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
