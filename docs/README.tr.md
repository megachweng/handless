<p align="center">
  <img src="../src-tauri/icons/logo.png" alt="Handless logo" width="128" height="128" />
</p>

<h1 align="center">Handless</h1>

<p align="center">
  macOS için ücretsiz, açık kaynaklı konuşmadan metne dönüştürücü.<br/>
  Bir kısayola basın, konuşun, herhangi bir uygulamada metin elde edin. Gizlilik için yerel olarak çalıştırın veya bulut API'lerini kullanın.
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
  <b>Türkçe</b> ·
  <a href="README.uk.md">Українська</a> ·
  <a href="README.vi.md">Tiếng Việt</a> ·
  <a href="README.pl.md">Polski</a> ·
  <a href="README.cs.md">Čeština</a>
</p>

## Özellikler

- **Yerel transkripsiyon** -- Ayarlar'dan modelleri indirin, tamamen cihaz üzerinde çalışır
- **Bulut STT** OpenAI veya Soniox üzerinden
- **Ses Etkinlik Algılama** (yalnızca yerel modeller)
- **LLM ile son işleme** transkripsiyonları temizlemek veya yeniden biçimlendirmek için
- **macOS** (Intel ve Apple Silicon)
- **17 dil**

## Kurulum

**macOS (Homebrew):**

```sh
brew tap ElwinLiu/tap
brew install --cask handless
```

Kaynaktan derlemek için: [BUILD.md](../BUILD.md) dosyasına bakın.

## CLI

**Uzaktan kontrol** (çalışan bir örnekle iletişim kurar):

```bash
handless --toggle-transcription    # Kaydı aç/kapat
handless --toggle-post-process     # Kayıt + son işlemeyi aç/kapat
handless --cancel                  # Mevcut işlemi iptal et
```

**Başlatma seçenekleri:**

```bash
handless --start-hidden            # Ana pencere yok
handless --no-tray                 # Tepsi simgesi yok
handless --debug                   # Ayrıntılı günlük kaydı
handless --help                    # Tüm seçenekler
```

Serbestçe birleştirebilirsiniz: `handless --start-hidden --no-tray`

> **macOS:** ikili dosyayı doğrudan çalıştırın: `/Applications/Handless.app/Contents/MacOS/Handless --toggle-transcription`

## Sorun Giderme

`Cmd+Shift+D` hata ayıklama panelini açar.

## Katkı

[CONTRIBUTING.md](../CONTRIBUTING.md) dosyasına bakın. Çeviriler için: [CONTRIBUTING_TRANSLATIONS.md](../CONTRIBUTING_TRANSLATIONS.md).

## Lisans

[MIT](../LICENSE)

## Teşekkürler

[Handy](https://github.com/cjpais/Handy) v0.7.8'den çatallanmış bir projedir.

[Whisper](https://github.com/openai/whisper) | [whisper.cpp](https://github.com/ggerganov/whisper.cpp) | [NeMo Parakeet](https://github.com/NVIDIA/NeMo) | [Moonshine](https://github.com/usefulsensors/moonshine) | [SenseVoice](https://github.com/FunAudioLLM/SenseVoice) | [Silero VAD](https://github.com/snakers4/silero-vad) | [Tauri](https://tauri.app) | [Handy](https://github.com/cjpais/Handy)
