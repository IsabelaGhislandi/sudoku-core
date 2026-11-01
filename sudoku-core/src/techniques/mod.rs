use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// O que uma dedução muda: coloca um valor ou remove candidatos.
///
/// As técnicas de Médio não colocam valores, então um par (célula, valor) não
/// basta para representar todas as deduções.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Place {
        index: usize,
        value: u8,
    },
    /// Pares (célula, dígito) a remover. Nunca vem vazia, e todo par remove um
    /// candidato presente: sem isso, o solver repetiria a dedução para sempre.
    Eliminate(Vec<(usize, u8)>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduction {
    pub action: Action,
    pub technique: &'static str,
    pub level: Difficulty,
}

/// Uma técnica de resolução (pattern Strategy): olha a grade e os candidatos e
/// devolve, se existir, uma dedução lógica.
pub trait Technique {
    /// Nível da técnica, usado para montar um solver limitado por dificuldade.
    fn level(&self) -> Difficulty;
    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction>;
}

mod hidden_single;
mod naked_single;

pub use hidden_single::HiddenSingle;
pub use naked_single::NakedSingle;
