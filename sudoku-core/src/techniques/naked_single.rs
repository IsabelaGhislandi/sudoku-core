use super::{Deduction, Technique};
use crate::candidates::candidates_for;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};

pub struct NakedSingle;

impl Technique for NakedSingle {
    fn apply(&self, grid: &Grid) -> Option<Deduction> {
        for index in 0..81 {
            if let Cell::Empty = grid.get(index) {
                let cands = candidates_for(grid, index);
                if cands.count() == 1 {
                    let value = cands.values()[0];
                    return Some(Deduction {
                        index,
                        value,
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
    use crate::grid::Grid;

    #[test]
    fn deduz_celula_com_unico_candidato() {
        // Linha 0: índices 1..8 = 1,2,3,4,5,6,7,8 => índice 0 só pode ser 9.
        let mut line = String::from(".12345678");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();

        let d = NakedSingle.apply(&grid).unwrap();
        assert_eq!(d.index, 0);
        assert_eq!(d.value, 9);
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        assert_eq!(NakedSingle.apply(&grid), None);
    }
}
