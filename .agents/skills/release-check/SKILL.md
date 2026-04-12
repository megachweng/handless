---
name: release-check
description: Use before cutting a release to verify build, version, and smoke test.
---

# Release Check

Pre-release verification for Handless. Run all checks, report pass/fail per item, and block release on any failure.

## Steps

### 1. Version Sync

All three files must contain the **same** version string:

- `src-tauri/tauri.conf.json` → `"version": "X.Y.Z"` (primary source — release workflow reads this)
- `src-tauri/Cargo.toml` → `version = "X.Y.Z"`
- `package.json` → `"version": "X.Y.Z"`

Read all three files and compare. Any mismatch is a **Fail**.

### 2. CHANGELOG.md

- `CHANGELOG.md` must have a `## [X.Y.Z] - YYYY-MM-DD` section matching the version from step 1.
- The `## [Unreleased]` section should be empty (all items moved to the new version section).
- The version section must have at least one entry under Added, Changed, or Fixed.

### 3. Translation Consistency

Run:

```bash
bun run check:translations
```

Non-zero exit code is a **Fail**. Report any missing or extra keys.

### 4. TypeScript Lint & Type Check

Run in parallel:

```bash
bun run lint
npx tsc --noEmit
```

Non-zero exit code on either is a **Fail**.

### 5. Formatting

Run in parallel:

```bash
bun run format:check
cd src-tauri && cargo fmt --check
```

Non-zero exit code on either is a **Fail**.

### 6. Rust Clippy

Run:

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

Non-zero exit code is a **Fail**. Warnings promoted to errors.

### 7. Cargo.lock Sync

Run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml --locked
```

Fails if `Cargo.lock` is out of date with `Cargo.toml`.

### 8. Bindings Up-to-Date

Specta auto-generates `src/bindings.ts` during `cargo check`. After the Rust check in step 7, verify:

```bash
git diff --name-only src/bindings.ts
```

If `bindings.ts` has uncommitted changes, bindings are stale — **Fail**.

### 9. Local Release Build Smoke Test

Run:

```bash
bun run tauri build --config src-tauri/tauri.release-check.conf.json
```

This local smoke test must compile the production app and produce the native bundle, but it should not require updater-signing secrets that only exist in CI. It intentionally disables Tauri updater artifacts while keeping the rest of the release build path intact. Non-zero exit code is a **Fail**. This is the slowest step; run it last.

### 10. Clean Working Tree

Run:

```bash
git status --porcelain
```

Any uncommitted changes (other than the release commit itself) is a **Fail**. Everything must be committed before triggering the release workflow.

## Output

Print a summary table:

```
## Release Check: vX.Y.Z

| #  | Check                  | Status |
|----|------------------------|--------|
| 1  | Version sync           | Pass   |
| 2  | CHANGELOG.md           | Pass   |
| 3  | Translations           | Pass   |
| 4  | Lint & type check      | Pass   |
| 5  | Formatting             | Pass   |
| 6  | Clippy                 | Pass   |
| 7  | Cargo.lock sync        | Pass   |
| 8  | Bindings up-to-date    | Pass   |
| 9  | Local release build    | Pass   |
| 10 | Clean working tree     | Pass   |

Result: READY TO RELEASE ✓
```

Any **Fail** must be fixed before release. After all checks pass, the release can be triggered via `gh workflow run release.yml`. CI remains responsible for updater artifact signing and any secrets required for that path.
