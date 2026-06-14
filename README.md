<p align="center">
  <img src="https://img.shields.io/badge/Rust-000000?style=flat-square&logo=rust&logoColor=white" alt="Rust" />
  <img src="https://img.shields.io/badge/TypeScript-3178C6?style=flat-square&logo=typescript&logoColor=white" alt="TypeScript" />
  <img src="https://img.shields.io/badge/Python-3776AB?style=flat-square&logo=python&logoColor=white" alt="Python" />
  <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="MIT License" />
</p>

<h1 align="center">Sailfish Studio</h1>

<p align="center">
  <strong>A next-generation visual programming IDE, reimagined for speed and extensibility.</strong>
</p>

<p align="center">
  Scratch-like block programming meets professional-grade architecture — powered by Rust, rendered at 60fps, and extensible to the core.
</p>

---

## What is Sailfish Studio?

Sailfish Studio is a visual programming environment inspired by Scratch, but built from the ground up with a modern, high-performance architecture. It allows users to create programs by dragging and connecting visual blocks, while providing a professional development experience with features like real-time collaboration, an extension marketplace, and a custom scripting language (SFL).

Unlike Scratch, Sailfish Studio compiles block programs to efficient bytecode via a Rust-based virtual machine, supports multiplayer editing through CRDT-based conflict resolution, and offers a plugin ecosystem for extending the platform in any direction.

## Architecture

Sailfish Studio is organized as a **6-repo monorepo**:

```
sailfish-studio/
├── sf-core/          # Rust core — VM, parser, renderer, blocks, storage, audio
│   ├── sf-vm/        # Bytecode VM with compiler & runtime
│   ├── sf-parser/    # .sf / .sb3 / .sfl file parsers
│   ├── sf-blocks/    # Block layout, rendering, drag engine
│   ├── sf-renderer/  # WebGL/SVG/batch rendering pipeline
│   ├── sf-storage/   # .sf SQLite project file I/O
│   └── sf-audio/     # Audio synthesis & playback engine
├── sf-editor/        # Next.js + TypeScript editor UI
├── sf-tools/         # Python CLI tools (project validator, asset manager, SFL formatter)
├── docs/             # Complete project documentation
│   ├── requirements/ # Product requirements & specifications
│   ├── design/       # Architecture, UI, security, language design docs
│   ├── development/  # Development guides & standards
│   ├── api/          # API reference
│   ├── operations/   # Deployment & monitoring
│   └── user/         # End-user guides
└── .github/          # CI/CD workflows
```

## Key Features

- **Block-based Programming** — Drag-and-drop visual blocks with 9 categories (Motion, Looks, Sound, Events, Control, Sensing, Operators, Variables, Pen) and 90+ block types
- **High-performance VM** — Rust-compiled bytecode VM running in WebAssembly, with 11 operation modules covering the full Scratch instruction set
- **Multi-format Parsing** — Import Scratch 3.0 (.sb3) projects, native .sf format, and SFL text scripts
- **60fps Canvas Rendering** — HTML5 Canvas with WebGL acceleration, batch rendering, and smooth zoom/pan
- **Undo/Redo** — Snapshot-based history with 200-entry stack for reliable state management
- **Keyboard Shortcuts** — Full keyboard navigation and editing (Ctrl+Z/Y, Delete, Ctrl+A, etc.)
- **Context Menus** — Right-click block operations (duplicate, delete, add comment)
- **Custom SFL Language** — Text-based scripting language with lexical grammar, type system, and block mapping
- **Extension System** — Plugin API with sandbox model, lifecycle management, and marketplace
- **Real-time Collaboration** — CRDT-based conflict resolution, presence awareness, and offline sync
- **Cross-platform** — Tauri desktop app + web browser support

## sf-core (Rust)

The performance-critical core is written in Rust and compiled to WebAssembly for browser execution:

| Crate | Description |
|-------|-------------|
| `sf-vm` | Bytecode compiler & virtual machine runtime with extension support |
| `sf-parser` | Parsers for .sf, .sb3 (Scratch 3.0), and .sfl (Sailfish Language) formats |
| `sf-blocks` | Block data model, layout computation, SVG rendering, and drag physics |
| `sf-renderer` | WebGL, SVG, and batched rendering pipelines |
| `sf-storage` | SQLite-backed .sf project file read/write |
| `sf-audio` | Audio synthesis and playback engine |

### Build sf-core

```bash
cd sf-core
cargo build
cargo test
```

### Compile to WebAssembly

```bash
wasm-pack build sf-vm --target web
```

## sf-editor (TypeScript)

The editor frontend built with Next.js and Zustand:

- **BlockCanvas** — HTML5 Canvas rendering engine with hit testing, zoom/pan, and block drawing
- **Toolbox** — Block palette with drag-to-canvas support
- **Project Store** — Zustand-based state management with snapshot undo/redo
- **VM Bridge** — JSON-RPC 2.0 communication with sf-vm Web Worker
- **Tauri Integration** — Native file dialogs, menus, and system APIs

### Setup sf-editor

```bash
cd sf-editor
npm install
npm test
```

## sf-tools (Python)

CLI utilities for project management:

| Tool | Description |
|------|-------------|
| `project_validator.py` | Validate .sf project structure and integrity |
| `asset_manager.py` | Import, export, and optimize project assets |
| `sfl_formatter.py` | Format and lint SFL source files |

### Setup sf-tools

```bash
cd sf-tools
pip install -e .
```

## Development

### Prerequisites

- Rust 1.75+ (with `wasm-pack`)
- Node.js 18+ (with pnpm)
- Python 3.10+ (with pip)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/DingdingOvO/sailfish-studio.git
cd sailfish-studio

# Build Rust core
cd sf-core && cargo build && cd ..

# Setup editor
cd sf-editor && npm install && npm test && cd ..

# Setup Python tools
cd sf-tools && pip install -e . && cd ..
```

## Documentation

Full documentation is available in the `docs/` directory:

- [Requirements](docs/requirements/README.md) — Product requirements and specifications
- [Architecture Design](docs/design/architecture/) — Process model, IPC protocol, data flow
- [UI Design](docs/design/ui/) — Design principles, color system, components
- [Language Design](docs/design/language/) — SFL grammar, type system, standard library
- [Security Design](docs/design/security/) — Threat model, sandbox, permissions
- [Development Guides](docs/development/) — Setup, coding standards, testing, debugging
- [API Reference](docs/api/) — sf-vm, extension, plugin, IPC, collaboration APIs

## License

This project is licensed under the [MIT License](LICENSE).

---

<p align="center">
  Built with Rust, TypeScript, and Python by the Sailfish Studio team.
</p>
