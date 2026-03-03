# midi-editor

**A simple, cross-platform, open-source MIDI editor built with Rust.**

---

## Highlights

- [x] **Cross-platform**: Windows / macOS / Linux
- [x] **Simple by design**: minimal UI, fast workflow
- [x] **Open-source**: contributions welcome
- [x] **Standard MIDI File (SMF)** support (import/export)

> Status: **WIP / Experimental** — breaking changes may happen until `v1.0`.

---

## Features

- Piano roll editing (notes, velocity, length)
- Event list (precise edits, inspection)
- Quantize / humanize (basic utilities) *(planned / partial)*
- Tempo & time signature support *(planned / partial)*
- Import / export `.mid` (SMF)

---

## Getting Started

### Install (from source)

#### 1) Prerequisites

- Rust (stable): https://rustup.rs

#### 2) Build & Run

```bash
git clone https://github.com/catfoot-dev/midi-editor.git
cd midi-editor

# Debug
cargo run

# Release
cargo run --release
```

#### 3) Build a binary

```bash
cargo build --release
```

---

## Usage

- Open: File → Open… (or drag & drop if supported)
<!-- - Edit:
    - piano roll: click/drag notes
    - event list: edit exact time/velocity
- Export: File → Export… to .mid -->

---

## License
This project is licensed under the MIT License.
See [LICENSE](./LICENSE) for details.
