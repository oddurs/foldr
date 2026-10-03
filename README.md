# foldr

A CLI-first folder inspector and configuration tool for macOS and Linux, written
in Rust.

Folders carry more than filenames: permissions, extended attributes, native
flags, inherited access rules, and filesystem-specific policies. Foldr aims to
make those properties discoverable, explain their effects, and provide deliberate
edits and reusable presets.

## Status

Early development. The repository contains a Rust workspace, CLI help and version
entry points, and a dependency-ordered implementation plan. Folder inspection and
editing commands are planned; they are not implemented yet. A GUI or background
service is not required.

## Direction

- Inspect folder properties and explain available capabilities.
- Preserve unusual Unix filenames and binary metadata.
- Edit notes, supported native flags, and basic permissions.
- Preview changes and apply partial presets that preserve unrelated properties.
- Verify writes and provide recovery with conflict-aware undo.

The [roadmap](ROADMAP.md) is generated from [Cairn items](cairn/items).
The [concept and architecture](cairn/items/0005-record-cli-first-concept-and-rust-architecture.md)
describe the intended behavior and release boundaries.

## Development

Install Rust through [rustup](https://rustup.rs/). The minimum supported Rust
version is 1.85; the development toolchain tracks stable. Initial targets are
macOS and Linux, with CI covering both stable Rust and the minimum version.

```sh
cargo build --workspace --locked
cargo run -p foldr-cli -- --help
cargo run -p foldr-cli -- --version
```

Run the local checks:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

## Architecture

- `crates/foldr-core`: folder models, native platform adapters, capability
  descriptions, change planning, presets, and recovery.
- `crates/foldr-cli`: command parsing, terminal output, and structured output.

Keep platform-specific operations in core adapters and presentation in the CLI.
The initial scaffold contains adapter modules; their implementation is scheduled
in the backlog.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Project work is tracked with
[Cairn](https://github.com/oddurs/cairn) as Markdown in this repository.

## License

[MIT](LICENSE).
