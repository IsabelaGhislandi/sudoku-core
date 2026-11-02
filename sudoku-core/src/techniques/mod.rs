use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Grid, box_of, col_of, row_of};

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

/// As 27 unidades do tabuleiro: linhas, depois colunas, depois boxes, com as células
/// em ordem crescente. A ordem é fixa porque o determinismo do gerador depende dela.
pub(crate) fn units() -> [[usize; 9]; 27] {
    std::array::from_fn(|u| {
        let mut cells = (0..81).filter(|&i| match u {
            0..=8 => row_of(i) == u,
            9..=17 => col_of(i) == u - 9,
            _ => box_of(i) == u - 18,
        });
        std::array::from_fn(|_| cells.next().expect("toda unidade tem 9 células"))
    })
}

mod box_line_reduction;
mod hidden_pair;
mod hidden_single;
mod hidden_triple;
mod naked_pair;
mod naked_single;
mod naked_triple;
mod pointing_pair;

pub use box_line_reduction::BoxLineReduction;
pub use hidden_pair::HiddenPair;
pub use hidden_single::HiddenSingle;
pub use hidden_triple::HiddenTriple;
pub use naked_pair::NakedPair;
pub use naked_single::NakedSingle;
pub use naked_triple::NakedTriple;
pub use pointing_pair::PointingPair;
