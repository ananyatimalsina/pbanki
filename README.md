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

Place your Anki collection at `./pbanki/collection/`

### PocketBook

```bash
make          # debug build
make release  # release build
```

Requires: `cargo-zigbuild` and `zig`

## Installation

1. Copy binary to device: `/mnt/ext1/applications/pbanki.app`
2. Copy your Anki collection to: `/mnt/ext1/applications/pbanki/collection/`
3. (Optional) Create configuration file at: `/mnt/ext1/applications/pbanki/config.toml`
4. Launch from Applications menu

## Configuration

pbAnki uses a TOML configuration file. On first launch, a default config will be created at `/mnt/ext1/applications/pbanki/config.toml` (or `./pbanki/config.toml` for desktop builds).

### Configuration Options

```toml
[general]
# Language code for Anki i18n (e.g., "en", "de", "fr", "ja", "es", "pt", "ru", "zh", "ko")
language = "en"

# Collection path (relative to app directory or absolute)
collection_path = "/mnt/ext1/applications/pbanki/collection"

[ankiweb]
# AnkiWeb synchronization settings
# Leave empty to disable sync

# AnkiWeb username (email)
username = ""

# Password (plain text - will be encrypted in future versions)
# WARNING: Do not share this file if password is filled
password = ""

# Session token (populated after successful login)
# This avoids storing password long-term
token = ""

# Sync automatically on app start
auto_sync = false

# Sync automatically after session ends
sync_on_exit = false
```

### Supported Languages

pbAnki uses Anki's built-in translations, supporting 70+ languages including:
- English (`en`, `en-GB`)
- European: `es`, `fr`, `de`, `it`, `pt-BR`, `pt-PT`, `nl`, `sv-SE`, `da`, `fi`, `cs`, `hu`, `el`, `pl`, `tr`, `ru`, `uk`
- Asian: `ja`, `ko`, `zh-CN`, `zh-TW`, `th`, `vi`, `id`, `hi-IN`
- Middle Eastern: `ar`, `fa`, `he`
- And many more...

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
- Study cards with scheduling
- Answer cards with Again/Hard/Good/Easy ratings
- Display interval durations on rating buttons
- Real-time deck count updates after answering
- Support for type-in cards (`[[type:Back]]`)
- Multi-language support via Anki's i18n system (70+ languages)
- Configurable collection path and language settings

## TODOs

- [ ] **AnkiWeb Sync** - Implement synchronization with AnkiWeb
- [ ] **MathJax Rendering** - Add support for LaTeX/MathJax formulas in cards
- [ ] **Image Support** - Display images embedded in cards from media folder
- [ ] **UI Polishing** - Improve e-ink optimized interface and user experience

## License

See [LICENSE](LICENSE)
