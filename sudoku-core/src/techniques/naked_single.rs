use super::{Action, Deduction, Technique};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};

pub struct NakedSingle;

impl Technique for NakedSingle {
    fn level(&self) -> Difficulty {
        Difficulty::Facil
    }

    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for index in 0..81 {
            if let Cell::Empty = grid.get(index) {
                let cands = candidates.get(index);
                if cands.count() == 1 {
                    let value = cands.values()[0];
                    return Some(Deduction {
                        action: Action::Place { index, value },
                        technique: "Naked Single",
                        level: Difficulty::Facil,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::CandidateGrid;
    use crate::grid::Grid;
    use crate::techniques::Action;

    #[test]
    fn deduz_celula_com_unico_candidato() {
        // Linha 0: índices 1..8 = 1,2,3,4,5,6,7,8 => índice 0 só pode ser 9.
        let mut line = String::from(".12345678");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();

        let candidates = CandidateGrid::from_grid(&grid);
        let d = NakedSingle.apply(&grid, &candidates).unwrap();
        assert_eq!(d.action, Action::Place { index: 0, value: 9 });
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        let candidates = CandidateGrid::from_grid(&grid);
        assert_eq!(NakedSingle.apply(&grid, &candidates), None);
    }
}
