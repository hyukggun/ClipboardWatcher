# ClipboardWatcher

> A lightweight macOS desktop application for managing clipboard history

[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/hyukggun/ClipboardWatcher)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-v2-orange.svg)](https://tauri.app/)
[![React](https://img.shields.io/badge/React-19-61dafb.svg)](https://react.dev/)

ClipboardWatcher is a modern desktop application built with Tauri v2, React 19, and TypeScript that automatically monitors your clipboard and provides an intuitive interface to manage copied text and images with fuzzy search capabilities.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/hyukggun/ClipboardWatcher.git
cd ClipboardWatcher

# Install dependencies
yarn install

# Run in development mode
yarn tauri dev
```

## Features

- **Automatic Clipboard Monitoring**: Continuously tracks clipboard changes in the background
- **Text and Image Support**: Captures both text entries and images (PNG format)
- **Persistent Storage**: Saves clipboard history to a local SQLite database
- **Category Filtering**: Filter entries by type (All, Text, Images)
- **Fuzzy Search**: Quickly find text entries using FZF (Fuzzy Finder) algorithm
- **System Tray Integration**: Access the app from the system tray with a single click
- **Quick Actions**:
  - Paste copied items back to clipboard
  - Delete individual entries
  - Clear entire history
- **Auto-hide**: Window automatically hides when it loses focus

## Tech Stack

### Frontend
- **React 19**: Latest React with improved hooks and performance
- **TypeScript 5.6**: Type-safe development with latest features
- **Vite 6**: Ultra-fast build tool and dev server with HMR

### Backend
- **Rust**: High-performance system programming language
- **Tauri v2**: Lightweight desktop application framework
- **SQLite**: Local database for persistent storage
- **rusqlite**: Rust bindings for SQLite

### Platform Support
- **macOS**: Full support using NSPasteboard API
- **Windows/Linux**: Coming soon

### Key Dependencies
- **rusqlite 0.32**: SQLite database with bundled support
- **objc2-app-kit 0.3**: Native macOS clipboard integration
- **chrono 0.4**: Timestamp management
- **base64 0.21**: Image encoding for storage

## Version

Current version: **0.1.0**

## Prerequisites

- **Node.js**: v18 or higher
- **Yarn**: Package manager
- **Rust**: Latest stable version
- **Xcode Command Line Tools** (macOS)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/hyukggun/ClipboardWatcher.git
cd ClipboardWatcher
```

2. Install dependencies:
```bash
yarn install
```

3. Install Rust dependencies:
```bash
cd src-tauri
cargo build
cd ..
```

## Development

### Start Development Mode

Run both the Rust backend and frontend dev server:

```bash
yarn tauri dev
```

This command:
- Starts the Vite dev server on port 1420
- Enables HMR (Hot Module Replacement) on port 1421
- Launches the Tauri application window

### Frontend-Only Development

Run the Vite dev server without Tauri:

```bash
yarn dev
```

Note: Tauri commands will not work in this mode.

### Type Checking

Check TypeScript and types:

```bash
yarn check
```

Watch mode for continuous type checking:

```bash
yarn check:watch
```

### Rust Backend Commands

```bash
cd src-tauri

# Build the Rust backend
cargo build

# Run tests
cargo test

# Check code without building
cargo check

# Clean build artifacts
cargo clean
```

## Build for Production

Build the application for distribution:

```bash
yarn tauri build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`.

## Project Structure

```
ClipboardWatcher/
├── src/                      # React frontend
│   ├── components/           # React components
│   │   ├── ClipboardCard.tsx
│   │   └── Sidebar.tsx
│   ├── App.tsx               # Main application component
│   ├── main.tsx              # Application entry point
│   ├── types.ts              # TypeScript type definitions
│   └── index.css             # Global styles
├── src-tauri/                # Rust backend
│   ├── src/
│   │   ├── lib.rs            # Main application logic
│   │   ├── main.rs           # Entry point
│   │   ├── base.rs           # Clipboard platform API
│   │   ├── db.rs             # Database operations
│   │   ├── model.rs          # Data models
│   │   └── fzf.rs            # Fuzzy finder implementation
│   ├── icons/                # Application icons
│   ├── Cargo.toml            # Rust dependencies
│   └── tauri.conf.json       # Tauri configuration
├── docs/                     # Documentation
└── CLAUDE.md                 # Development guide for Claude Code

```

## Architecture

### Frontend (React)
- **Component-based UI**: Modular React components with TypeScript
- **State Management**: React hooks (useState, useEffect, useMemo)
- **Event Handling**: Listens to Tauri events for real-time updates
- **Vite Dev Server**: Runs on port 1420 with HMR on port 1421

### Backend (Tauri + Rust)
- **Main Logic**: `src-tauri/src/lib.rs` contains core application logic
- **Tauri Commands**: Rust functions annotated with `#[tauri::command]`
- **Clipboard Polling**: Background thread monitors clipboard changes every 1 second
- **Event Emission**: Emits events to frontend when clipboard changes or entries are deleted
- **Database**: SQLite database for persistent clipboard history

### Communication Flow

```
User copies text/image
        ↓
macOS NSPasteboard updates
        ↓
Rust polling thread detects change
        ↓
Save to SQLite database
        ↓
Emit "clipboard-changed" event
        ↓
React frontend receives event
        ↓
Update UI with new entry
```

## Adding New Tauri Commands

1. Define a command in `src-tauri/src/lib.rs`:

```rust
#[tauri::command]
fn my_custom_command(param: String) -> Result<String, String> {
    // Your logic here
    Ok(format!("Processed: {}", param))
}
```

2. Register the command in the builder:

```rust
.invoke_handler(tauri::generate_handler![
    load_clipboard_events_at_startup,
    delete_clipboard_entry,
    hide_window,
    my_custom_command  // Add your command here
])
```

3. Call from the frontend:

```typescript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke<string>('my_custom_command', {
  param: 'Hello'
});
```

## Configuration

### Tauri Configuration
Edit `src-tauri/tauri.conf.json` to customize:
- Window size and behavior
- Bundle settings
- Application identifier
- Dev/build commands

### Database Location
The SQLite database is stored in the app's data directory:
- macOS: `~/Library/Application Support/com.clipboardwatcher.app/clipboard_history.db`

## Troubleshooting

### Clipboard not detecting changes
- Ensure the app has necessary permissions on macOS
- Check the terminal for `[POLLING]` debug logs

### Duplicate entries appearing
- Verify React Strict Mode is disabled in `src/main.tsx`
- Check browser console for `[EVENT]` logs

### Build fails
- Update Rust: `rustup update`
- Clean and rebuild: `cargo clean && cargo build`

For more detailed troubleshooting guides, see the `docs/` directory.

## Documentation

Comprehensive documentation is available in the `docs/` directory:

### Technical Guides
- **01-rust-ownership-optional-handling.md**: Rust ownership patterns and Option handling
- **02-rust-references-vs-cpp.md**: Rust references compared to C++ pointers
- **03-rust-compile-time-safety.md**: Compile-time safety features in Rust
- **04-thread-spawn-error-hints.md**: Thread spawning and error handling

### Implementation Guides
- **05-tauri-app-deployment.md**: Application deployment strategies
- **06-hide-window-command-usage.md**: Window management implementation
- **07-clipboard-delete-event-handling.md**: Delete event implementation and updates
- **08-rust-result-type-in-event-payload.md**: Handling Result types in events

### Troubleshooting
- **09-duplicate-clipboard-event-analysis.md**: Analysis of duplicate event issues
- **10-duplicate-event-debugging-guide.md**: Step-by-step debugging guide
- **11-duplicate-card-rendering-fix.md**: Fix for duplicate card rendering with React

## Recent Improvements

### v0.1.0 (Latest)
- ✅ **Fixed**: Duplicate clipboard entry rendering issue
- ✅ **Fixed**: React Strict Mode causing double event listeners
- ✅ **Improved**: Event deduplication with ID checking
- ✅ **Improved**: React key optimization using unique identifiers
- ✅ **Added**: Comprehensive debug logging for troubleshooting
- ✅ **Added**: Delete event emission from backend
- ✅ **Migrated**: Frontend from Svelte to React 19
- ✅ **Implemented**: FZF (Fuzzy Finder) algorithm for search

## Known Issues

- Image paste functionality not yet implemented (copy back to clipboard works for text only)
- Database file location is hardcoded to app data directory
- No settings UI for customizing poll interval or history size
- Search only works for text items (images excluded)

## Roadmap

### v0.2.0 (Planned)
- [ ] Image paste support
- [ ] Settings panel for configuration
- [ ] Keyboard shortcuts (global hotkeys)
- [ ] Dark/Light theme toggle
- [ ] Export/Import clipboard history

### v0.3.0 (Future)
- [ ] Windows and Linux support
- [ ] Cloud sync capabilities
- [ ] Advanced filtering and tagging
- [ ] Rich text format support
- [ ] Multi-language support

## Performance

- **Memory Usage**: ~50-70MB at runtime
- **Database Size**: ~1KB per text entry, varies for images
- **Polling Interval**: 1 second (configurable in code)
- **Startup Time**: < 2 seconds on Apple Silicon Macs

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[MIT License](LICENSE)

## Acknowledgments

Built with:
- [Tauri](https://tauri.app/)
- [React](https://react.dev/)
- [Vite](https://vitejs.dev/)
- [Rust](https://www.rust-lang.org/)
