use crate::difficulty::Difficulty;
use crate::grid::Grid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduction {
    pub index: usize,
    pub value: u8,
    pub technique: &'static str,
    pub level: Difficulty,
}

/// Uma técnica de resolução (pattern Strategy): olha a grade e devolve, se
/// existir, uma jogada logicamente dedutível.
pub trait Technique {
    fn apply(&self, grid: &Grid) -> Option<Deduction>;
}

mod hidden_single;
mod naked_single;

pub use hidden_single::HiddenSingle;
pub use naked_single::NakedSingle;
