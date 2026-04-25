# naluminaLink

A modular Linux audio router built on PipeWire, with a GUI-first direction inspired by tools like Wave Link and Banana.

## Status

The project is currently in an early implementation phase. Right now it includes:

- a desktop test UI for browsing PipeWire nodes and testing mixer-style layout ideas
- per-channel dual-mix sends (independent monitor and stream levels) in the desktop UI
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

- `src/main.rs` - app entrypoint and CLI command orchestration
- `src/ui/mod.rs` - desktop UI bootstrap and `NaluminaApp` root state
- `src/ui/components.rs` - shared UI helper components (section headers and progress bars)
- `src/ui/state/mod.rs` - UI state module wiring
- `src/ui/state/channel_state.rs` - centralized channel state store for levels, mute, and sends
- `src/ui/state/status_state.rs` - centralized UI status message state and transitions
- `src/ui/render/mod.rs` - render module wiring
- `src/ui/render/layout.rs` - panel layout and main composition
- `src/ui/render/channel_strip.rs` - channel strip state and controls
- `src/ui/render/mix.rs` - monitor/stream mix calculations
- `src/ui/refresh/mod.rs` - refresh module wiring
- `src/ui/refresh/worker.rs` - node refresh worker start logic
- `src/ui/refresh/polling.rs` - non-blocking refresh polling and status handling
- `src/ui/refresh/defaults.rs` - node default channel state synchronization
- `src/ui/theme.rs` - UI theme setup
- `src/node_discovery.rs` - PipeWire node collection and CLI node rendering
- `src/models.rs` - shared app models and UI constants
- `src/i18n.rs` - language loading and placeholder interpolation
- `lang/en.json` - English translation strings
- `lang/de.json` - German translation strings
- `context.md` - live project context and engineering constraints
- `.gitignore` - Rust/editor ignore defaults

## Notes

- Linux-only project scope.
- Current GUI is intentionally a test surface, not final product UX.
- Realtime constraints have priority over convenience abstractions.
