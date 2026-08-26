# Repository Guidelines

## Scope Boundary

Work exclusively inside the `rotomdex-web/` directory tree. Treat every other repository path as read-only context. When a task requires broader scope, pause and request explicit approval before continuing.

## Project Structure & Module Organization

`src/main.rs` initializes logging, connects `RotomDexCore` to Ratzilla’s WebGL2 backend, and installs browser input handlers. `index.html` owns page metadata and browser-level styling. `assets/` contains the favicon, font atlas, and atlas symbol manifests. `scripts/` contains repeatable asset-generation tools. `README.md` documents authorship of this client. Treat `dist/` as Trunk-generated output.

Keep browser input, WebGL2 configuration, canvas behavior, and asset logic local and cohesive.

## Build, Test, and Development Commands

Run these commands from the repository root unless stated otherwise:

- `cargo fmt -p rotomdex-web --check` verifies Rust formatting.
- `cargo clippy -p rotomdex-web --target wasm32-unknown-unknown` checks browser-targeted Rust code.
- `cargo test -p rotomdex-web` runs package tests.
- `cargo check -p rotomdex-web --target wasm32-unknown-unknown` performs a quick WASM validation.
- `cargo build -p rotomdex-web --target wasm32-unknown-unknown` performs a complete WASM compile.
- From `rotomdex-web/`, use `trunk serve` for local development and `trunk build --release` for deployment output.
- Use `./scripts/generate-font-atlas.sh` from `rotomdex-web/` after changing `assets/atlas-symbols/*.txt`.

## Coding Style & Naming Conventions

Use Rust 2024 conventions and the workspace’s 120-column rustfmt limit. Use `snake_case` for functions and modules, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Propagate recoverable failures with `Result`. Write comments that capture browser, WASM, or rendering constraints.

## Testing Guidelines

Place unit tests beside implementations in `#[cfg(test)] mod tests`. Name tests after observable behavior, such as `selection_copies_rectangular_region`. For visual changes, verify resize and zoom behavior, rectangular selection and copying, pitch-black backgrounds, glyph fallback, and block-symbol seams.

## Generated Assets

Store each symbol group in a focused UTF-8 file under `assets/atlas-symbols/`. Regenerate the binary atlas through the script. Keep manifest and atlas changes together for developer handoff. Preserve deterministic inputs.

## Agent Working Style

Preserve Git history and leave completed working-tree changes for the developer to commit. Report changed files and verification results during handoff.

Explore several viable approaches before selecting an implementation. Look for capabilities in existing dependencies, browser APIs, build tooling, and data-driven generation. Pursue creative solutions that reduce code, duplication, state, and maintenance cost. Prefer the smallest cohesive design with clear ownership and reproducible behavior. Keep abstractions proportional to current needs and reuse established project mechanisms wherever they fit cleanly.
