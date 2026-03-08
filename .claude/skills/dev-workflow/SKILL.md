---
name: dev-workflow
description: Development commands for running, building, testing, linting, and formatting the Handless project. Use when setting up the dev environment, running the app, building for production, or checking code quality.
---

# Development Workflow

## Environment Setup

```bash
bun install                    # Install dependencies
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
```

## Run & Build

```bash
# Development (if cmake error on macOS, prefix with CMAKE_POLICY_VERSION_MINIMUM=3.5)
bun run tauri dev

# Frontend only (Vite dev server)
bun run dev

# Build frontend
bun run build

# Production build
bun run tauri build
```

## Type Checking

```bash
bunx tsc --noEmit               # Type check without emitting
bun run build                   # Build and validate
```

## Code Quality

```bash
bun run lint              # ESLint check
bun run lint:fix          # ESLint auto-fix
bun run format            # Prettier + cargo fmt
bun run format:check      # Check only
bun run format:frontend   # Prettier only
bun run format:backend    # cargo fmt only
```

## Testing

```bash
bun run test:playwright      # E2E tests
bun run test:playwright:ui   # E2E with UI
bun run check:translations   # Validate translation files
```
