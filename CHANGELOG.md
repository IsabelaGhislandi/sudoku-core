# Changelog

All notable changes to this project are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project
uses [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Generator output is part of the contract.** The same seed and difficulty must give the same
puzzle across versions. Any change that alters generated puzzles is listed under a
**Puzzle output** heading, together with the levels it affects.

## [Unreleased]

The first release (`0.1.0`) will be tagged together with the first browser version of the game.

### Added

- **Browser frontend** (`web/`, Svelte 5 + TypeScript + Vite): home, level choice, game and history
  screens. Notes, hints, pause (the board hides while paused), restart with confirmation, a saved
  game with Continuar, and a history of finished, lost (Derrota) and abandoned (Largada) games, all
  stored in `localStorage`.
- **Destacar** toggle on the game screen (on by default): tapping a filled cell highlights every
  cell with the same number.
- **`sudoku-wasm` crate**: the WebAssembly bridge (`wasm-bindgen`) that exposes a `sudoku-game` match
  to JavaScript as `WebGame`, with a JSON view of the board, the existing `SavedGame` and
  `SavedResult` JSON formats, and `historyEntries` to check and sort the stored history.
- CI job `web`: builds the WASM package, then type-checks, tests and builds the web app.
- **`sudoku-game` crate** with the rules of a match, on top of `sudoku-core`: `Game::new` and
  `Game::from_puzzle` (which validates the puzzle and returns a `PuzzleError` otherwise), `place` (a correct number locks; a wrong one costs a life and leaves the
  board), `toggle_note` (per-cell pencil notes), `hint` (reveals a cell from the solution), win and
  loss. `GameConfig` defaults to 3 lives and 2 hints. Refused actions return a `MoveError` and
  change nothing.
- **Game results** in `sudoku-game`: `Game::result` for a finished game, `Game::abandon`, and
  `restart` now returns the abandonment result of an unfinished game (it also takes `now` and the
  wall-clock date). `level_stats` and `personal_ranking` compute per-level statistics and the
  personal ranking. `SavedResult` (format v1) stores a result, serializable with the `serde`
  feature.
- The repository is now a Cargo workspace (`sudoku-core`, `sudoku-game`), with a shared
  `Cargo.lock` and `deny.toml` at the root.
- `Grid` and `Cell`, with parsing and serialization to an 81-character line
  (`Grid::from_line`, `Grid::to_line`).
- `validate`, which reports conflicting cells in rows, columns and boxes.
- `CandidateSet`, `CandidateGrid` and `candidates_for` for pencil marks.
- `count_solutions`, a backtracking solver that branches on the most constrained cell. It is
  used to check that a puzzle has a unique solution.
- `LogicalSolver` with `solve`, `next_step` and `up_to(level)`. Each `Deduction` records its
  `Action` (place a value or eliminate candidates), the technique used and its level.
- Solving techniques behind the `Technique` trait: Naked Single, Hidden Single, Naked Pair,
  Hidden Pair, Pointing Pair, Box/Line Reduction, Naked Triple, Hidden Triple, X-Wing,
  Swordfish and XY-Wing.
- `rate`, which classifies a grid by the hardest technique needed to solve it.
- `generate(difficulty, seed)` for all four levels: `Facil`, `Medio`, `Dificil` and
  `MuitoDificil`. Very Hard puzzles have at most 22 clues and can be solved by logic alone.
- `GenError::AttemptsExhausted` for the rare seeds that use up a level's work budget.
- Golden tests that lock the generator output for seed 42.

### Puzzle output

- `Game::new` retries with the next seed when `generate` returns `AttemptsExhausted` (seed 15 on
  Very Hard becomes 16).
- The hinted cell is deterministic: `splitmix64(seed ^ hint_number) % empty_cells`. A golden test
  locks the first hint for seed 42 on Easy (cell 40, value 9).
- The generator uses `ChaCha12Rng`, so seeds stay reproducible across `rand` versions.
- Fixed work budgets are part of the determinism contract: `MAX_ATTEMPTS_MEDIUM` = 50,
  `MAX_ATTEMPTS_HARD` = 250, and for Very Hard `REFILL_VERY_HARD`,
  `MAX_REFINEMENTS_VERY_HARD` and `MAX_SOLUTIONS_VERY_HARD`.

### Requirements

- Minimum supported Rust version (MSRV): 1.97.

[Unreleased]: https://github.com/IsabelaGhislandi/sudoku-core/commits/master
