# Build Instructions

This guide covers how to set up the development environment and build Handless from source.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 20.19 or newer
- [pnpm](https://pnpm.io/) 10.33.2
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)
- Windows build tools for Tauri

## Setup Instructions

### 1. Clone the Repository

```bash
git clone git@github.com:elwin/handless.git
cd handless
```

### 2. Install Dependencies

```bash
pnpm install
```

### 3. Start Dev Server

```bash
pnpm tauri dev
```
