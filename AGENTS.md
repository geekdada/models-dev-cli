# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A terminal UI (TUI) application for browsing AI model data from the [models.dev](https://models.dev) API. Built in Rust using ratatui for the terminal interface and nucleo-matcher for fuzzy search.

## Commands

- `cargo build` — build the project
- `cargo run` — run the TUI (fetches live data from https://models.dev/api.json on startup)
- `cargo clippy` — lint
- `cargo fmt --check` — check formatting
- `cargo test` — run tests (none currently exist)

## Architecture

The app follows a straightforward Elm-like architecture: fetch data, then loop on render -> poll events -> update state.

- **`src/main.rs`** — Entry point. Fetches API data via blocking HTTP, initializes ratatui terminal, runs the event loop.
- **`src/data.rs`** — API types and data fetching. `ApiData` is `HashMap<String, Provider>` where each `Provider` contains a `HashMap<String, Model>`. Uses `reqwest::blocking` to fetch from `https://models.dev/api.json`.
- **`src/app.rs`** — Application state (`App`) and input handling. Two-level navigation: `View::Level1` (all providers/models) and `View::Level2` (models within a single provider). Fuzzy filtering uses nucleo-matcher against multiple fields (name, id, family, combined provider+model name).
- **`src/ui.rs`** — Rendering. Split layout: left panel has search input + filterable list, right panel shows detail view for the selected item. Scrollbar on detail pane when content overflows.

### Key design notes

- Fuzzy search at Level1 matches both providers and models simultaneously; providers appear first in results, then models sorted by score.
- The `ListItem` enum distinguishes `Provider` vs `Model` entries in the unified list, carrying enough context (provider_id, model_id) for detail rendering.
- Each navigation level maintains its own `Input` state so search text is preserved when navigating back.
