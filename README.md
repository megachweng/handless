# Handless

Windows desktop speech-to-text app built with Tauri 2, Rust, React, and TypeScript.

## Development

```bash
pnpm install
pnpm run tauri dev
pnpm run tauri build
pnpm run lint
pnpm run test:playwright
pnpm run check:translations
```

## Features

- **Local transcription** -- download models in Settings, runs fully on-device
- **Cloud STT** via OpenAI or Soniox
- **Voice Activity Detection** (local models only)
- **LLM post-processing** to clean up or reformat transcriptions
- **Windows**
- **English and Chinese UI**

## CLI

```bash
handless --toggle-transcription    # Toggle recording
handless --toggle-post-process     # Toggle recording + post-processing
handless --cancel                  # Cancel current operation
handless --start-hidden            # No main window
handless --no-tray                 # No tray icon
handless --debug                   # Verbose logging
handless --help                    # All flags
```

Combine freely: `handless --start-hidden --no-tray`

## Troubleshooting

`Ctrl+Shift+D` opens the debug panel.

## License

[MIT](LICENSE)
