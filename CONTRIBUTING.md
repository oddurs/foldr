# Contributing

Foldr is early in development. Start with the [roadmap](ROADMAP.md) and the
[CLI-first concept](cairn/items/0005-record-cli-first-concept-and-rust-architecture.md).

## Find work

Install [Cairn](https://github.com/oddurs/cairn), then:

```sh
cairn next
cairn show ITEM_ID
cairn prompt ITEM_ID
```

Every work item has context, acceptance criteria, and dependencies. Discuss larger
changes in a GitHub issue or pull request, and keep the repository's Cairn items
updated. Claims coordinate writers sharing the same local item directory; they
do not reserve work across independent clones.

```sh
cairn claim ITEM_ID
cairn note ITEM_ID "What changed or what you learned"
cairn tick ITEM_ID CRITERION_NUMBER
cairn close ITEM_ID --result "What the completed work concluded"
cairn check --strict --render --prompts
```

Do not edit ROADMAP.md directly. Change the items and run `cairn render`. Do not
check off criteria that have not been verified. Follow [AGENTS.md](AGENTS.md) when
using a coding agent.

For automatic backlog merging, run `cairn init --git` once in your clone; the
repository's Git attributes need its local merge driver.

## Build and verify

Use Rust 1.85 or newer. The repository's toolchain file selects stable Rust.

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo build --workspace --locked
cargo test --workspace --locked
cargo run -p foldr-cli -- --help
cargo run -p foldr-cli -- --version
```

CI checks macOS and Linux with both stable Rust and the minimum supported version.
Add integration checks when introducing filesystem behavior. Use temporary test
folders, preserve unrelated metadata, and report unsupported features explicitly.
The core should not depend on terminal rendering or a graphical interface.

## Pull requests

Explain the observable change, reference its Cairn item, and describe the relevant
verification. Preserve the scope of native properties: a change to one directory
does not necessarily affect its existing files or newly created children in the
same way.

Contributions are provided under the project's [MIT license](LICENSE).
