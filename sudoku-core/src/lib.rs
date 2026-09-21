//! sudoku-core — geração, resolução, validação e classificação de Sudoku 9×9.
//! Biblioteca pura: sem UI, sem IO, sem Tauri.

mod backtracking;
mod candidates;
mod difficulty;
mod generator;
mod grid;
mod rating;
mod solver;
mod techniques;

pub use backtracking::count_solutions;
pub use candidates::{CandidateSet, candidates_for};
pub use difficulty::Difficulty;
pub use generator::{GenError, Puzzle, generate};
pub use grid::{Cell, Conflict, Grid, ParseError, validate};
pub use rating::rate;
pub use solver::{LogicalSolver, SolveResult, next_hint};
pub use techniques::{Deduction, HiddenSingle, NakedSingle, Technique};
