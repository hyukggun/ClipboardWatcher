# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tauri v2 desktop application with SvelteKit frontend running in SPA mode. The frontend uses Svelte 5 with TypeScript, and the backend is written in Rust.

## Development Commands

```bash
# Start Tauri development mode (runs both Rust backend and frontend dev server)
yarn tauri dev

# Build the application for distribution
yarn tauri build

# Frontend-only development (runs Vite dev server on port 1420)
yarn dev

# Frontend build only
yarn build

# Type-check Svelte and TypeScript
yarn check

# Type-check in watch mode
yarn check:watch
```

## Rust Backend Commands

```bash
cd src-tauri

# Build Rust backend
cargo build

# Run tests
cargo test

# Check code without building
cargo check
```

## Architecture

### Frontend (SvelteKit in SPA Mode)
- Located in `src/` directory with SvelteKit file-based routing in `src/routes/`
- Uses `@sveltejs/adapter-static` with `fallback: "index.html"` (no SSR support in Tauri)
- Vite dev server runs on port 1420, HMR on port 1421
- Frontend communicates with Rust backend via Tauri's `invoke()` API from `@tauri-apps/api`

### Backend (Tauri + Rust)
- Main application logic in `src-tauri/src/lib.rs`
- Entry point `src-tauri/src/main.rs` delegates to `clipboardwatcher_lib::run()`
- Library name is `clipboardwatcher_lib` (suffixed to avoid Windows naming conflicts)
- Tauri commands are Rust functions annotated with `#[tauri::command]` and registered in `invoke_handler`
- Data serialization handled by `serde` and `serde_json`

### Adding New Tauri Commands
1. Define function in `src-tauri/src/lib.rs` with `#[tauri::command]` attribute
2. Register in builder: `.invoke_handler(tauri::generate_handler![greet, new_command])`
3. Invoke from frontend: `import { invoke } from '@tauri-apps/api/core'`

### Configuration Files
- `src-tauri/tauri.conf.json` - Tauri app configuration (window size, bundle settings, dev/build commands)
- `src-tauri/Cargo.toml` - Rust dependencies and library configuration
- `vite.config.js` - Vite configuration with Tauri-specific settings
- `svelte.config.js` - SvelteKit adapter configuration for SPA mode
- 힌트나 개선 방향을 제시하고, 사용자의 요청이 아니면 직접 전부를 작성해지마.