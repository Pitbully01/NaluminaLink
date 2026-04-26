# Project: PipeWire Audio Router (Rust)

## Project Goal

This project builds a **modular audio router for Linux (primarily Arch Linux)** on top of PipeWire.

Primary focus areas:

- dynamic audio routing
- virtual audio channels
- plugin processing (DSP chains)
- GUI-driven control, with CLI support for diagnostics and automation

This is **not** intended to be a classic mixer. Conceptually, it is closer to:

> a Linux implementation inspired by Elgato Wave Link / Banana-style workflows

---

## Planned Core Features

- [ ] Create/remove virtual audio channels
- [ ] Assign application/device sources to channels
- [ ] Connect channels to each other and to outputs
- [ ] DSP chains per channel
- [ ] Persistent routing configuration
- [ ] GUI for control and monitoring
- [ ] CLI for debug and automation
- [x] Early desktop test UI
- [x] Dual-mix UI baseline (monitor/stream sends)
- [x] Modular feature/shared folder structure

---

## Explicit Non-Goals

- Terminal-only final product
- DAW functionality (recording, timeline editing)
- Full replacement for professional audio workstations
- Cross-platform-first scope (Linux only)

Notes:

- A GUI is the target; the current desktop UI is an early test surface.
- The long-term interaction model should feel similar to Wave Link / Banana routing UX.

---

## Architecture Overview

The system is organized into three major layers:

### 1. Audio Layer (PipeWire Integration)

Responsible for:

- connecting to PipeWire
- creating/managing nodes (streams, filters)
- port handling and linking

Tech:

- Rust `pipewire` crate
- PipeWire filter/node APIs

---

### 2. DSP / Plugin Layer

Responsible for:

- per-channel audio processing
- plugin hosting

Planned support:

- primary: LV2 (Linux-native)
- optional later: VST3

Key constraints:

- realtime-safe processing is mandatory
- no heap allocation in the audio thread
- plugins run in or near realtime-sensitive paths

---

### 3. Routing Engine (Core Logic)

Responsible for:

- channel lifecycle and state
- graph/routing relationships between nodes
- plugin chain assignment
- system state updates and propagation

This is the **central project logic**.

---

## Channel Concept

Each channel is modeled as:

```
Input → DSP Chain → Output
```

Conceptual Rust model:

```rust
struct Channel {
    id: String,
    input_node: NodeId,
    output_node: NodeId,
    plugins: Vec<PluginInstance>,
}
```

---

## DSP Chain Concept

- processing order matters
- each plugin receives and transforms audio buffers
- processing can be sample-based or block-based

Example:

```
Input → Gain → EQ → Reverb → Output
```

---

## UI/CLI Status

The system is not CLI-only.

Current status:

- `cargo run` starts the desktop test UI
- `cargo run -- list-nodes` lists PipeWire nodes in CLI
- `cargo run -- doctor` prints a basic health/status message
- UI includes dual-mix sends, node filtering, visible-channel limits, and scene presets
- refresh flow uses background worker + polling with typed result/error handling
- CLI remains a debug and automation companion

---

## Current Code Layout

Current source layout follows feature-oriented modules with shared infrastructure:

- `src/features/node_discovery/` for node discovery domain/service
- `src/features/ui/` for UI composition, rendering, state, and refresh flows
- `src/shared/i18n/` for language loading and translation access

---

## Concurrency & Realtime Constraints

Critical rules:

- Audio thread must:
    - never block
    - avoid mutex locking
    - avoid heap allocations

- State/control communication should use:
    - lock-free queues or message passing
    - immutable snapshots or double-buffered state transitions

---

## State Management

Two distinct states:

1. **Control State (GUI/CLI/Main Thread)**
2. **Audio State (Realtime Thread)**

Synchronization approach:

- event queues
- immutable snapshots

---

## Persistence

Planned:

- JSON or TOML configuration
- restore routing/session state on startup

Example:

```json
{
    "channels": [
        {
            "id": "music",
            "plugins": ["eq", "reverb"]
        }
    ]
}
```

---

## Development Strategy

### Phase 1

- connect to PipeWire
- enumerate nodes

Status: implemented as the first working baseline.

### Phase 2

- simple pass-through filter node

Status: not implemented yet.

Current practical step: expand from UI control baseline to actual routing/filter graph execution in PipeWire.

### Phase 3

- gain/volume control

### Phase 4

- multiple channels

### Phase 5

- inter-channel routing

### Phase 6

- LV2 plugin system

### Phase 7

- optional VST3 support

---

## Key Design Decisions

- Rust for safety and concurrency correctness
- PipeWire as the single audio backend dependency
- user-facing text managed through language files (i18n), not hardcoded strings
- GUI-centered product direction, with CLI for diagnostics/automation
- prefer modularity and clarity in early phases
- realtime safety is non-negotiable

### i18n Model (Current)

- language files in `lang/` (`en.json`, `de.json`)
- UI/CLI text is resolved via keys
- dynamic values use placeholders (`{{name}}`)
- language detection via `NALUMINALINK_LANG`, fallback via `LANG`

---

## Things To Avoid

- quick hacks in audio-thread code paths
- global mutable state without strict ownership boundaries
- blocking IO in audio path
- tight coupling between routing and DSP concerns

---

## Optional Future Scope

- routing presets/scenes
- network audio support (e.g. RTP)
- web control surface (control only, no rendering)
- plugin sandboxing (separate process)
- full channel-based GUI for routing, monitoring, and DSP management

---

## Notes For Coding Agents

When working on this project:

- prefer simple and traceable solutions
- follow realtime constraints strictly
- change architecture only with explicit rationale
- avoid unnecessary abstraction
- keep code debuggable (logging outside realtime audio thread)

If uncertain:

-> choose the smallest viable implementation over overengineering

---

## Current Focus

> A minimal working baseline with PipeWire node discovery, desktop test UI, i18n-backed user text, and foundations for a full routing GUI
