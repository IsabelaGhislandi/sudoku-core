# Contributing

Thanks for your interest! This is a personal project and a Rust learning log, but issues and
pull requests are welcome. The full conventions, in Portuguese, are in [`CLAUDE.md`](CLAUDE.md).
This page summarizes them.

## Workflow

Development is **spec-driven and test-driven**. Larger changes follow:

1. **Spec**: a design document in `docs/superpowers/specs/`.
2. **Plan**: a step-by-step implementation plan in `docs/superpowers/plans/`.
3. **Implementation** with TDD: write the test, **watch it fail**, implement, watch it pass.

For a small fix, just open an issue or a PR.

## Before opening a PR

Run these from the repository root. CI runs the same checks on Linux, Windows and macOS,
treats warnings as errors and uses `--locked`.

```bash
cargo fmt    --manifest-path sudoku-core/Cargo.toml
cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings
cargo test   --manifest-path sudoku-core/Cargo.toml
cargo deny   --manifest-path sudoku-core/Cargo.toml check
```

The minimum supported Rust version is set in `sudoku-core/Cargo.toml` (`rust-version`).

## Commits

Commit messages are in **English** and follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type: short summary in the imperative

Optional body explaining why, wrapped at ~72 columns.
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`. Write the summary as what the
commit does ("add rate", not "added rate"). Keep one logical change per commit.

## Code conventions

- **`sudoku-core` is a pure library**: no UI, no IO, no clock. Timers, persistence and history
  belong to the future Game module.
- **Identifiers in English.** Comments, doc comments (`///`) and `assert!` messages are in
  Portuguese and explain the *why*. The `Difficulty` variants (`Facil`, `Medio`...) are the one
  exception and stay as they are.
- **Tests**: unit tests go inline in a `#[cfg(test)] mod tests` block. Public API integration
  tests go in `sudoku-core/tests/`. `#[test]` function names are in Portuguese and describe the
  behavior (`fn puzzle_resolvivel_por_singles_eh_facil()`).
- **Determinism is a contract.** The same seed and difficulty must give the same puzzle across
  versions, and golden tests lock the output for seed 42. If your change alters generated
  puzzles, say so in the PR and add it to [`CHANGELOG.md`](CHANGELOG.md) under **Puzzle output**.
- **Dependencies are kept minimal.** Every new or updated crate needs a reason and a check of
  its maintenance, advisories, license and build-time code. See
  [`.claude/skills/adding-dependencies`](.claude/skills/adding-dependencies/SKILL.md).

## Documentation

Each task adds a page to [`docs/wiki/`](docs/wiki), in Portuguese, about the Rust concepts it
involved. Keep [`docs/architecture.md`](docs/architecture.md) up to date when a design decision
changes.
