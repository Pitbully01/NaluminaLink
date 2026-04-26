# naluminaLink

A modular Linux audio router built on PipeWire, with a GUI-first direction inspired by tools like Wave Link and Banana.

## Status

The project is currently in an early implementation phase. Right now it includes:

- a desktop test UI for browsing PipeWire nodes and testing mixer-style layout ideas
- per-channel dual-mix sends (independent monitor and stream levels) in the desktop UI
- workspace controls for node filtering and adjustable channel visibility
- scene preset buttons (Balanced / Monitor Focus / Stream Boost) for fast mix shaping
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

For runtime diagnostics:

```bash
RUST_LOG=nalumina_link=debug cargo run
```

## Internationalization (i18n)

- All user-facing strings are resolved by translation keys, not hardcoded literals in main application flow.
- Language files are located in `lang/` (`en.json`, `de.json`).
- Placeholders use the `{{name}}` format and are replaced at runtime.
- Language is detected from `NALUMINALINK_LANG` first, then `LANG`, with fallback to English.
- Technical error sources in UI status messages are also localized via i18n keys.

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
- `src/features/mod.rs` - feature module wiring
- `src/features/node_discovery/mod.rs` - node discovery feature exports
- `src/features/node_discovery/domain.rs` - node discovery domain model (`NodeEntry`)
- `src/features/node_discovery/service.rs` - PipeWire node collection and CLI node rendering
- `src/features/ui/mod.rs` - desktop UI bootstrap and `NaluminaApp` root state
- `src/features/ui/components/mod.rs` - shared UI helper components (section headers and progress bars)
- `src/features/ui/state/mod.rs` - UI state module wiring and exports
- `src/features/ui/state/channel_state.rs` - channel state store, mix data types, and UI defaults
- `src/features/ui/state/status_keys.rs` - typed i18n key enum for UI status messages
- `src/features/ui/state/status_state.rs` - centralized UI status message state and transitions
- `src/features/ui/render/mod.rs` - render module wiring
- `src/features/ui/render/layout.rs` - panel layout and main composition
- `src/features/ui/render/channel_strip.rs` - channel strip state and controls
- `src/features/ui/render/mix.rs` - monitor/stream mix calculations
- `src/features/ui/refresh/mod.rs` - refresh module wiring
- `src/features/ui/refresh/worker.rs` - node refresh worker start logic
- `src/features/ui/refresh/polling.rs` - non-blocking refresh polling and status handling
- `src/features/ui/refresh/result.rs` - typed refresh result/error objects (including error source metadata)
- `src/features/ui/refresh/defaults.rs` - node default channel state synchronization
- `src/features/ui/theme.rs` - UI theme setup
- `src/shared/mod.rs` - shared module wiring
- `src/shared/i18n/mod.rs` - shared i18n exports
- `src/shared/i18n/loader.rs` - language loading and placeholder interpolation
- `lang/en.json` - English translation strings
- `lang/de.json` - German translation strings
- `context.md` - live project context and engineering constraints
- `.gitignore` - Rust/editor ignore defaults

## Notes

- Linux-only project scope.
- Current GUI is intentionally a test surface, not final product UX.
- Realtime constraints have priority over convenience abstractions.
