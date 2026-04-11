# models-dev-cli

A terminal UI for browsing AI model data from [models.dev](https://models.dev).

![Rust](https://img.shields.io/badge/Rust-2021-orange) ![License](https://img.shields.io/badge/license-MIT-blue)

## Features

- Browse AI providers and their models in a two-level hierarchy
- Fuzzy search across providers and models simultaneously
- Detail panel showing capabilities, cost, limits, modalities, and metadata
- Keyboard-driven navigation with scrollable detail view

## Install

```sh
cargo install --path .
```

This installs the `models` binary.

## Usage

```sh
models
```

The app fetches live data from the [models.dev API](https://models.dev/api.json) on startup, then presents an interactive TUI.

### Keybindings

| Key | Action |
|---|---|
| Type | Fuzzy search |
| Up / Down | Navigate list |
| Enter | Drill into provider |
| Esc | Back / Quit |
| PgUp / PgDn | Scroll detail panel |
| q | Quit |

## Development

```sh
cargo build       # build
cargo run         # run
cargo clippy      # lint
cargo fmt --check # check formatting
```

## License

[MIT](LICENSE)
