# naluminaLink

A modular Linux audio router built on PipeWire, with a GUI-first direction inspired by tools like Wave Link and Banana.

## Status

The project is currently in an early implementation phase. Right now it includes:

- a desktop test UI for browsing PipeWire nodes and testing mixer-style layout ideas
- a CLI command for listing discovered nodes
- a simple `doctor` command for basic health output
- language-file based i18n for visible CLI and UI text

## Run

```bash
cargo run
```

Running without arguments opens the desktop UI.
Additional CLI commands are available for diagnostics:

```bash
cargo run -- help
cargo run -- doctor
cargo run -- list-nodes
cargo run -- ui
```

## Development

```bash
cargo check
cargo run -- list-nodes
```

## Internationalization (i18n)

- All user-facing strings are resolved by translation keys, not hardcoded literals in main application flow.
- Language files are located in `lang/` (`en.json`, `de.json`).
- Placeholders use the `{{name}}` format and are replaced at runtime.
- Language is detected from `NALUMINALINK_LANG` first, then `LANG`, with fallback to English.

## Target Product

- create and remove virtual audio channels
- assign application/device sources to channels
- connect channels to output buses
- build DSP chains per channel
- persist and restore routing configuration
- provide a clear desktop GUI for routing, monitoring, and control

## Non-Goals

- DAW functionality (recording, timeline editing)
- full cross-platform support (Linux-focused)
- replacing professional full audio workstations

## Architecture Direction

- PipeWire as the only audio backend
- realtime-safe audio paths (no blocking/heap allocation in the audio thread)
- strict separation between control state and audio state
- modular routing core with future plugin-processing layer

## Project Structure

- `src/main.rs` - app entrypoint, CLI commands, desktop UI, and PipeWire node discovery view
- `src/i18n.rs` - language loading and placeholder interpolation
- `lang/en.json` - English translation strings
- `lang/de.json` - German translation strings
- `context.md` - live project context and engineering constraints
- `.gitignore` - Rust/editor ignore defaults

## Notes

- Linux-only project scope.
- Current GUI is intentionally a test surface, not final product UX.
- Realtime constraints have priority over convenience abstractions.
