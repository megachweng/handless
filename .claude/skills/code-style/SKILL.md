---
name: code-style
description: Code style conventions for Rust backend and TypeScript/React frontend. Use when writing new code, reviewing changes, or unsure about project conventions.
---

# Code Style

## Rust

- `cargo fmt` + `cargo clippy` before committing
- Explicit error handling (avoid `unwrap` in production)
- `anyhow::Error` with descriptive context messages; `?` operator to propagate
- `Arc<Mutex<T>>` for shared state in managers
- Log with appropriate levels: `debug!`, `info!`, `eprintln!` for errors
- Builder pattern for initialization chains
- Snake_case for functions/variables, PascalCase for types
- Separate logical sections with comment blocks: `/* ─────────── */`
- New Tauri commands go in `commands/`, business logic in `managers/`
- Use specta for type-safe command bindings (auto-generates `bindings.ts`)

## TypeScript/React

- Strict TypeScript, no `any`
- Functional components with TypeScript interfaces
- `React.FC` for explicit component typing
- Zod schemas for type validation and inference
- `useCallback` hooks for stable function references
- Destructure props with defaults: `disabled = false`
- Prefer interface over type aliases for objects
- PascalCase for components, camelCase for variables/functions
- Tailwind CSS for styling, Radix UI for primitives
- Path alias: `@/` -> `./src/`
- State: Zustand stores in `stores/`
- New settings components go in the appropriate `settings/` subdirectory

## Imports

- Group: external libs, internal modules, relative imports
- Use type imports: `import type { Settings }`
- Named imports preferred over default exports

## Error Handling

- Frontend: Try/catch with user feedback, rollback optimistic updates
- Backend: `?` operator with anyhow context messages

## Component Patterns

- Container component pattern for layout
- Composition over inheritance
- Prop drilling minimized with context where appropriate
