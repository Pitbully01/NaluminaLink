# Projekt: PipeWire Audio Router (Rust)

## 🧠 Ziel des Projekts

Dieses Projekt implementiert einen **modularen Audio-Router für Linux (primär Arch Linux)** auf Basis von PipeWire.

Der Fokus liegt auf:

- dynamischem Audio-Routing
- virtuellen Audio-Channels
- Plugin-Processing (DSP Chains)
- CLI-basierter Steuerung (kein GUI)

Das Projekt ist **kein klassischer Mixer**, sondern eher:

> ein linux port von elgato wave link 3

---

## 🎯 Kernfeatures (geplant)

- [ ] Virtuelle Audio-Channels erstellen/löschen
- [ ] Audioquellen (Apps, Devices) Channels zuweisen
- [ ] Channels miteinander verbinden
- [ ] DSP-Ketten pro Channel (Plugins)
- [ ] Persistente Routing-Konfiguration
- [ ] CLI Interface für Steuerung

---

## ❌ Nicht-Ziele (bewusst ausgeschlossen)

- Kein GUI (auch langfristig nicht geplant)
- Keine DAW-Funktionalität (kein Recording, kein Timeline Editing)
- Kein vollständiger Ersatz für professionelle Audio-Workstations
- Kein Fokus auf Cross-Platform (Linux-only)

---

## 🧱 Architektur-Übersicht

Das System ist in drei Hauptschichten unterteilt:

### 1. Audio Layer (PipeWire Integration)

Verantwortlich für:

- Verbindung zu PipeWire
- Erstellen von Nodes (Streams / Filter)
- Port-Handling und Linking

Technologien:

- Rust `pipewire` crate
- PipeWire Filter Nodes (`pw_filter`)

---

### 2. DSP / Plugin Layer

Verantwortlich für:

- Audioverarbeitung innerhalb eines Channels
- Plugin-Hosting

Geplante Unterstützung:

- Primär: LV2 (Linux-native Plugins)
- Optional später: VST3 über VST3 SDK

Wichtige Einschränkungen:

- Realtime-safe Verarbeitung erforderlich
- Keine dynamische Speicherallokation im Audio-Thread
- Plugins laufen im Audio-Thread

---

### 3. Routing Engine (Core Logic)

Verantwortlich für:

- Verwaltung von Channels
- Verbindungen zwischen Nodes
- Plugin-Ketten
- Zustand des Systems

Dies ist die **zentrale Logik des Projekts**.

---

## 🔁 Konzept: Channel

Ein Channel ist die zentrale Einheit:

```
Input → DSP Chain → Output
```

Rust-Struktur (konzeptionell):

```rust
struct Channel {
    id: String,
    input_node: NodeId,
    output_node: NodeId,
    plugins: Vec<PluginInstance>,
}
```

---

## 🔌 Konzept: DSP Chain

- Reihenfolge ist entscheidend
- Jeder Plugin-Prozess erhält Audio-Buffer
- Verarbeitung erfolgt sample-weise oder blockweise

Beispiel:

```
Input → Gain → EQ → Reverb → Output
```

---

## ⚙️ CLI Interface

Das System wird ausschließlich über CLI gesteuert.

Beispielbefehle:

```bash
mixer add-channel music
mixer add-plugin music reverb
mixer connect firefox music
mixer connect music output
mixer remove-channel music
```

---

## 🧵 Concurrency & Realtime Constraints

WICHTIG:

- Audio-Thread darf:
    - ❌ nicht blockieren
    - ❌ keine Mutex Locks verwenden
    - ❌ keine Heap-Allokationen durchführen

- Kommunikation erfolgt über:
    - Lock-free Queues
    - Message Passing
    - Double-buffered State

---

## 🧠 State Management

Zwei getrennte Zustände:

1. **Control State (CLI / Main Thread)**
2. **Audio State (Realtime Thread)**

Synchronisation über:

- Event Queue
- Immutable Snapshots

---

## 📦 Persistenz

Geplant:

- Speicherung als JSON oder TOML
- Wiederherstellung beim Start

Beispiel:

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

## 🧪 Entwicklungsstrategie

### Phase 1

- Verbindung zu PipeWire herstellen
- Nodes auflisten

### Phase 2

- Einfacher Pass-through Filter Node

### Phase 3

- Gain (Volume Control)

### Phase 4

- Mehrere Channels

### Phase 5

- Routing zwischen Channels

### Phase 6

- Plugin-System (LV2)

### Phase 7

- Optional: VST3 Support

---

## ⚠️ Wichtige Designentscheidungen

- Rust wird für Sicherheit und Concurrency verwendet
- PipeWire ist die einzige Audio-Backend-Abhängigkeit
- CLI-first Design (kein GUI-Overhead)
- Modularität ist wichtiger als Performance in frühen Phasen
- Realtime-Sicherheit hat oberste Priorität

---

## 🚫 Dinge, die vermieden werden sollen

- „quick hacks“ im Audio-Thread
- globale mutable States
- blocking IO im Audiopfad
- enge Kopplung zwischen Routing und DSP

---

## 🔮 Zukunft (optional)

- Preset-System für Routing
- Netzwerk-Audio (z. B. RTP)
- Web-Interface (nur Control, nicht Rendering)
- Plugin Sandbox (separater Prozess)

---

## 🧭 Für KI / Coding Agents

Beim Arbeiten an diesem Projekt:

- Bevorzuge **einfache, nachvollziehbare Lösungen**
- Halte dich strikt an Realtime-Constraints
- Verändere Architektur nur mit Begründung
- Vermeide unnötige Abstraktion
- Schreibe Code, der debugbar ist (Logging außerhalb Audio-Thread)

Wenn unsicher:
→ lieber minimal implementieren als overengineeren

---

## 📌 Aktueller Fokus

> Minimal funktionierender Audio-Pass-Through über PipeWire Filter Node
