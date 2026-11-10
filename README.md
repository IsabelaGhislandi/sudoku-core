# sudoku-core

[![CI](https://github.com/IsabelaGhislandi/sudoku-core/actions/workflows/ci.yml/badge.svg)](https://github.com/IsabelaGhislandi/sudoku-core/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Rust edition 2024](https://img.shields.io/badge/rust-edition%202024-orange.svg)

The pure Rust core of a Sudoku game built in Rust + Svelte and shipped as an Android APK through
Tauri. It is a Cargo workspace with two library crates and a WebAssembly bridge:

| Crate | What it does |
|---|---|
| [`sudoku-core`](sudoku-core) | The engine: **generates, solves, validates and rates** 9×9 puzzles |
| [`sudoku-game`](sudoku-game) | The match rules: moves, lives, hints, notes, timer, save/restore and restart |
| [`sudoku-wasm`](sudoku-wasm) | The bridge that exposes a match to JavaScript, compiled to WebAssembly for the [web app](web) |

Both are standalone libraries with no UI, no IO and no clock. The shell (the app, a browser build,
the future PvP server) reads the clock, writes to disk and talks to the network. The core only
takes data and returns data. That keeps it easy to test and lets the same rules run in the app, on
a server, and in a future online mode.

## Engine features (`sudoku-core`)

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

## Game features (`sudoku-game`)

- **A match on top of the engine.** Each number is checked against the solution right away. A
  correct number locks in place. A wrong one costs a life and leaves the board. There is no undo.
  Zero lives is a loss, and a full board is a win.
- **Configurable rules**: 3 lives and 2 hints by default (`GameConfig`). A hint reveals a cell
  picked deterministically from the seed, so the phone and a server always agree.
- **Per-cell pencil notes**, only on empty cells. They are never changed automatically.
- **Injected clock.** The shell passes instants in milliseconds to `resume`, `pause`, `place` and
  `hint`, and the game adds up the time played. A new or restored game starts paused, and winning
  or losing stops the clock.
- **Save and restore.** `save(now)` returns a `SavedGame` snapshot with plain fields (format v1,
  seed as text so JavaScript keeps its precision). `Game::restore` rejects any corrupted or
  tampered snapshot with a `RestoreError` and never panics.
- **Restart** with a new seed at the same level and with the same rules.
- **Results.** Every game ends in a `GameResult`: a win, a loss by lives, or an abandonment
  (restarting or leaving mid-game counts as a loss). Per-level statistics (`level_stats`: games,
  wins, win percentage, best time) and a personal ranking (`personal_ranking`: wins, fastest
  first) are pure functions over the list the shell keeps.
- **No required dependency** besides `sudoku-core`. The optional `serde` feature makes
  `SavedGame` and `SavedResult` serializable, and golden tests lock their JSON forms.

## Quick start: engine

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

## Quick start: a match

```toml
[dependencies]
sudoku-game = { git = "https://github.com/IsabelaGhislandi/sudoku-core", features = ["serde"] }
```

```rust
use sudoku_game::{Difficulty, Game, GameConfig, PlaceOutcome, SavedGame};

fn main() {
    // 3 lives and 2 hints by default. A new game starts paused.
    let mut game = Game::new(Difficulty::Medio, 7, GameConfig::default());
    game.resume(0); // instants are milliseconds from the shell's monotonic clock

    match game.place(2, 5, 1_200) {
        Ok(PlaceOutcome::Correct) => println!("locked in"),
        Ok(PlaceOutcome::Wrong { index, value }) => println!("{value} at {index} costs a life"),
        Err(e) => println!("refused: {e:?}"), // occupied cell, paused, game over...
    }
    let hint = game.hint(2_000).unwrap(); // deterministic: same game, same hint cell
    println!("hint: {} at {}", hint.value, hint.index);

    // The app goes to the background: pause and store the snapshot.
    game.pause(3_000);
    let saved: SavedGame = game.save(3_000); // serde_json::to_string(&saved) with the feature

    // Later: restore (rejects corrupted or tampered snapshots) and keep playing.
    let mut restored = Game::restore(&saved).expect("valid snapshot");
    restored.resume(0);
    println!("{} lives, {} ms played", restored.lives_left(), restored.elapsed_ms(500));
}
```

Output:

```text
5 at 2 costs a life
hint: 1 at 3
2 lives, 3500 ms played
```

`sudoku-game` re-exports the engine types it uses (`Difficulty`, `Puzzle`, `Grid`, `Cell`), so an
app only needs this one dependency.

## Engine public API

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

## Game public API

```rust
pub struct GameConfig { pub lives: NonZeroU8, pub hints: u8 } // Default: 3 lives, 2 hints

Game::new(difficulty, seed, config) -> Game;                   // retries the next seed if one is exhausted
Game::from_puzzle(puzzle, config) -> Result<Game, PuzzleError>;
game.place(index, value, now) -> Result<PlaceOutcome, MoveError>; // Correct | Wrong { index, value }
game.toggle_note(index, value) -> Result<(), MoveError>;
game.hint(now) -> Result<HintOutcome, MoveError>;              // { index, value }
game.resume(now); game.pause(now); game.elapsed_ms(now) -> u64; game.is_paused() -> bool;
game.save(now) -> SavedGame;
Game::restore(&saved) -> Result<Game, RestoreError>;
game.restart(seed, now, finished_at_ms) -> Restart;             // { game, abandoned, .. }
game.result(finished_at_ms) -> Option<GameResult>;             // Some once the game is over
game.abandon(now, finished_at_ms) -> Option<GameResult>;       // consumes the game
result.save() -> SavedResult; GameResult::restore(&saved) -> Result<GameResult, ResultError>;
level_stats(&results, difficulty) -> LevelStats;               // played, won, best_ms, win_percentage()
personal_ranking(&results, difficulty) -> Vec<&GameResult>;
game.cell(index) -> CellState;  // Empty | Given(v) | Placed(v) | Hinted(v)
game.notes(index) -> Notes; game.status() -> Status;           // Playing | Won | Lost
```

A refused action returns a `MoveError` (`GameOver`, `Paused`, `InvalidIndex`, `InvalidValue`,
`CellNotEmpty`, `NoHintsLeft`) and changes nothing: it costs no life and is not counted as a
mistake.

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

### Play in the browser

The web app lives in [`web/`](web) (Svelte 5, TypeScript, Vite). It needs Node 24, the
`wasm32-unknown-unknown` Rust target and [`wasm-pack`](https://github.com/wasm-bindgen/wasm-pack):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
cd web
npm install
npm run dev     # builds the WASM package, then serves the app at http://localhost:5173
npm test        # Vitest, for the TypeScript logic
npm run check   # svelte-check and tsc
```

## Roadmap

| # | Module | Status |
|---|---|---|
| 1 | **Core/Engine** (`sudoku-core`): generate, solve, validate, rate | ✅ done (all four levels) |
| 2 | Game/Session (`sudoku-game`): moves, lives, hints, notes, timer, game history | 🚧 in progress (slices 1–2 done: basic game, time and saving; next: results, history and stats) |
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
