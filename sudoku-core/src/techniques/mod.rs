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

/// As 27 unidades do tabuleiro: linhas, depois colunas, depois boxes, com as células
/// em ordem crescente. A ordem é fixa porque o determinismo do gerador depende dela.
///
/// A tabela é calculada uma vez, na compilação: as técnicas a consultam a cada passo do
/// solver, e o gerador de Difícil chama o solver milhares de vezes.
pub(crate) fn units() -> &'static [[usize; 9]; 27] {
    &UNITS
}

static UNITS: [[usize; 9]; 27] = build_units();

/// `const fn` só aceita laços `while`, então a montagem não usa iteradores.
const fn build_units() -> [[usize; 9]; 27] {
    let mut all_units = [[0; 9]; 27];
    let mut k = 0;
    while k < 9 {
        let mut j = 0;
        while j < 9 {
            all_units[k][j] = k * 9 + j; // linha k
            all_units[9 + k][j] = j * 9 + k; // coluna k
            all_units[18 + k][j] = (k / 3 * 3 + j / 3) * 9 + k % 3 * 3 + j % 3; // box k
            j += 1;
        }
        k += 1;
    }
    all_units
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{box_of, col_of, row_of};

    #[test]
    fn unidades_batem_com_linha_coluna_e_box() {
        for (u, unit) in units().iter().enumerate() {
            let expected: Vec<usize> = (0..81)
                .filter(|&i| match u {
                    0..=8 => row_of(i) == u,
                    9..=17 => col_of(i) == u - 9,
                    _ => box_of(i) == u - 18,
                })
                .collect();
            assert_eq!(unit.to_vec(), expected, "unidade {u}");
        }
    }
}

mod box_line_reduction;
mod hidden_pair;
mod hidden_single;
mod hidden_triple;
mod naked_pair;
mod naked_single;
mod naked_triple;
mod pointing_pair;
mod x_wing;

pub use box_line_reduction::BoxLineReduction;
pub use hidden_pair::HiddenPair;
pub use hidden_single::HiddenSingle;
pub use hidden_triple::HiddenTriple;
pub use naked_pair::NakedPair;
pub use naked_single::NakedSingle;
pub use naked_triple::NakedTriple;
pub use pointing_pair::PointingPair;
pub use x_wing::XWing;
