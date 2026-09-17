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
pub use candidates::{candidates_for, CandidateSet};
pub use difficulty::Difficulty;
pub use generator::{generate, GenError, Puzzle};
pub use grid::{validate, Cell, Conflict, Grid, ParseError};
pub use rating::rate;
pub use solver::{next_hint, LogicalSolver, SolveResult};
pub use techniques::{Deduction, HiddenSingle, NakedSingle, Technique};
