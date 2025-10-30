# pbAnki

Anki flashcard app for PocketBook e-readers.

## What is this?

Port of Anki to PocketBook devices using:
- Anki's official rslib backend for full collection compatibility
- Slint UI framework with software rendering for e-ink displays
- inkview-rs bindings for PocketBook SDK integration

## Building

### Desktop (for development)

```bash
cd dev
cargo run
```

Place your Anki collection at `./pbanki/collection/collection.anki2`

### PocketBook

```bash
make          # debug build
make release  # release build
```

Requires: `cargo-zigbuild` and `zig`

## Installation

1. Copy binary to device: `/mnt/ext1/applications/pbanki`
2. Copy your Anki collection to: `/mnt/ext1/applications/pbanki/collection/`
3. Launch from Applications menu

## Project Structure

```
src/      # PocketBook build (uses inkview-rs)
dev/      # Desktop build (for development)
ui/       # Slint UI components (shared)
```

## Current Features

- View deck hierarchies
- Collapse/expand decks
- Show card counts (new/learning/due)

## License

See [LICENSE](LICENSE)
