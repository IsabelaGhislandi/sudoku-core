# sudoku-core

[![CI](https://github.com/IsabelaGhislandi/sudoku-core/actions/workflows/ci.yml/badge.svg)](https://github.com/IsabelaGhislandi/sudoku-core/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust edition 2024](https://img.shields.io/badge/rust-edition%202024-orange.svg)

A pure Rust engine that **generates, solves, validates and rates** 9×9 Sudoku puzzles.

It's the core of a Sudoku game built in Rust + Svelte and shipped as an Android APK through
Tauri. The engine is a standalone library with no UI, no IO and no clock. That keeps it easy to
test and lets the same code run in the app, on a server, and in a future online mode.

## Features

- **Four difficulty levels**, each defined by the hardest technique needed to solve the puzzle:

  | Level (`Difficulty`) | Hardest technique required |
  |---|---|
  | `Facil` (Easy) | Naked Single, Hidden Single |
  | `Medio` (Medium) | + Naked/Hidden Pair, Pointing Pair |
  | `Dificil` (Hard) | + Naked/Hidden Triple, Box/Line Reduction, X-Wing |
  | `MuitoDificil` (Very Hard) | + Swordfish, XY-Wing; at most 22 clues |

- **Deterministic generation.** The same seed and level always give the same puzzle, across
  library versions too. The RNG is `ChaCha12Rng`, and golden tests lock the output. A seed is
  enough to share a puzzle.
- **Every puzzle has exactly one solution**, and every generated puzzle can be solved by logic
  alone, without guessing.
- **Step-by-step logical solver.** Each step records what it did (place a value or eliminate
  candidates), which technique it used, and that technique's level. A game can use this for
  hints and explanations.
- **Rating** of any grid, plus a backtracking solver that counts solutions to check uniqueness.
- **Two dependencies**: `rand` (default features off) and `rand_chacha`.

## Quick start

The crate isn't on crates.io yet. Add it as a git dependency:

```toml
[dependencies]
sudoku-core = { git = "https://github.com/IsabelaGhislandi/sudoku-core" }
```

```rust
use sudoku_core::{Difficulty, LogicalSolver, count_solutions, generate, rate, validate};

fn main() {
    // Same seed + same difficulty -> same puzzle, always.
    let puzzle = generate(Difficulty::Dificil, 42).expect("seed exhausted, try another");

    println!("{}", puzzle.givens.to_line()); // 81 chars, '.' for empty cells
    assert_eq!(count_solutions(&puzzle.givens, 2), 1); // unique solution
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Dificil));
    assert!(validate(&puzzle.givens).is_empty());

    // Step-by-step logical solve, with the technique behind each deduction.
    let result = LogicalSolver::new().solve(&puzzle.givens);
    for step in result.steps.iter().take(5) {
        println!("{:?} via {}", step.action, step.technique);
    }
    assert!(result.solved);
}
```

Output:

```text
.2..574...79..6...1.............1...5...6.87.4...3.1.5.459...6.9.......26.....7..
Place { index: 60, value: 3 } via Naked Single
Place { index: 69, value: 5 } via Naked Single
Place { index: 15, value: 2 } via Naked Single
Place { index: 20, value: 4 } via Hidden Single
Place { index: 19, value: 5 } via Hidden Single
```

## Public API

```rust
pub enum Difficulty { Facil, Medio, Dificil, MuitoDificil }
pub struct Puzzle { pub givens: Grid, pub solution: Grid, pub difficulty: Difficulty, pub seed: u64 }

fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>;
fn rate(grid: &Grid) -> Option<Difficulty>;
fn validate(grid: &Grid) -> Vec<Conflict>;
fn count_solutions(grid: &Grid, limit: usize) -> usize;
fn candidates_for(grid: &Grid, index: usize) -> CandidateSet;

Grid::from_line(&str) -> Result<Grid, ParseError>;
LogicalSolver::new().solve(&grid) -> SolveResult; // { grid, steps: Vec<Deduction>, solved }
```

Each technique is a type that implements the `Technique` trait (`NakedSingle`, `XWing`,
`XYWing`...), so you can also build a solver limited to a given level with
`LogicalSolver::up_to(level)`.

`generate` has a single error, `GenError::AttemptsExhausted`. It means the seed used up the
level's work budget (about 1 seed in 5,000 on Hard, 1 in 200 on Very Hard). The caller should
just try another seed.

## How generation works

```
seed → ChaCha12 RNG → random complete solution (randomized backtracking)
     → remove clues while the level's solver can still solve it (this also keeps the solution unique)
     → accept if the hardest technique used matches the requested level
```

Medium and Hard retry with the same RNG, up to a fixed number of attempts. Very Hard
*refines* near misses instead of restarting: it puts some clues from the solution back and
removes clues again. Measured in release mode on a desktop over 200 seeds, the worst case is
15 ms on Medium and 431 ms on Hard. Very Hard has a median of 119 ms and a worst case of 793 ms.

## Development

The project is **spec-driven and test-driven**. Each module goes from design spec to
implementation plan to TDD (see [`docs/superpowers/`](docs/superpowers)).

```bash
cargo test   --workspace                  # without optional features
cargo test   --workspace --all-features   # with the serde feature of sudoku-game
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt    --all
cargo deny   check
```

CI runs fmt, clippy, build and tests on Linux, Windows and macOS for every PR, with warnings
treated as errors. `cargo-deny` checks advisories, licenses and crate sources on every PR and
again every week.

## Roadmap

| # | Module | Status |
|---|---|---|
| 1 | **Core/Engine** (`sudoku-core`): generate, solve, validate, rate | ✅ done (all four levels) |
| 2 | Game/Session (`sudoku-game`): moves, lives, hints, notes, timer, game history | 🚧 in progress (slice 1 done: basic game) |
| 3 | Frontend (Svelte): board, input, screens | planned |
| 4 | Tauri packaging: Rust ↔ Svelte bridge, Android APK | planned |
| 5 | Online: two-player race via a shared puzzle code | future |

## Documentation

- [`docs/architecture.md`](docs/architecture.md): design overview and decisions
- [`docs/wiki/`](docs/wiki): a learning log, one page per task, on the Rust concepts each task
  introduced. I came to Rust from web development. Written in Portuguese.

Code identifiers are in English. Comments, docs and test names are in Portuguese.

## License

[MIT](LICENSE) © 2026 IsabelaGhislandi
