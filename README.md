# ClipboardWatcher

ClipboardWatcher is a lightweight desktop application built with Tauri v2, React, and TypeScript that monitors your clipboard history and provides an intuitive interface to manage copied text and images.

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
- **React 18**: Modern UI library with hooks
- **TypeScript**: Type-safe development
- **Vite**: Fast build tool and dev server

### Backend
- **Rust**: High-performance system programming language
- **Tauri v2**: Lightweight desktop application framework
- **SQLite**: Local database for persistent storage
- **rusqlite**: Rust bindings for SQLite

### Platform Support
- macOS (using NSPasteboard API)

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

Detailed documentation is available in the `docs/` directory:
- Rust ownership and error handling
- Tauri event system
- Debugging duplicate events
- And more...

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
