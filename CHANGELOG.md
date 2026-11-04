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

- The generator uses `ChaCha12Rng`, so seeds stay reproducible across `rand` versions.
- Fixed work budgets are part of the determinism contract: `MAX_ATTEMPTS_MEDIUM` = 50,
  `MAX_ATTEMPTS_HARD` = 250, and for Very Hard `REFILL_VERY_HARD`,
  `MAX_REFINEMENTS_VERY_HARD` and `MAX_SOLUTIONS_VERY_HARD`.

### Requirements

- Minimum supported Rust version (MSRV): 1.97.

[Unreleased]: https://github.com/IsabelaGhislandi/sudoku-core/commits/master
