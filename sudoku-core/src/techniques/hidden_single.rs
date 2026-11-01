use super::{Action, Deduction, Technique};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid, box_of, col_of, row_of};

pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn level(&self) -> Difficulty {
        Difficulty::Facil
    }

    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for value in 1..=9u8 {
            for unit in 0..9 {
                if let Some(index) = sole_spot(grid, candidates, value, |i| row_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, candidates, value, |i| col_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, candidates, value, |i| box_of(i) == unit) {
                    return Some(deduction(index, value));
                }
            }
        }
        None
    }
}

fn deduction(index: usize, value: u8) -> Deduction {
    Deduction {
        action: Action::Place { index, value },
        technique: "Hidden Single",
        level: Difficulty::Facil,
    }
}

/// Retorna a única célula vazia da unidade que pode receber `value`, se existir
/// exatamente uma.
fn sole_spot(
    grid: &Grid,
    candidates: &CandidateGrid,
    value: u8,
    in_unit: impl Fn(usize) -> bool,
) -> Option<usize> {
    let mut found: Option<usize> = None;
    for i in 0..81 {
        if !in_unit(i) {
            continue;
        }
        if let Cell::Empty = grid.get(i)
            && candidates.get(i).contains(value)
        {
            if found.is_some() {
                return None; // mais de um lugar possível
            }
            found = Some(i);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::{CandidateGrid, candidates_for};
    use crate::grid::Grid;
    use crate::techniques::Action;

    #[test]
    fn deduz_unico_lugar_do_digito_na_linha() {
        // Coloca 5 em cada coluna 1..8 (linhas/boxes distintos), de modo que,
        // na linha 0, o dígito 5 só caiba no índice 0 — mas o índice 0 ainda
        // tem vários candidatos (logo NÃO é naked single).
        let mut grid = Grid::empty();
        for index in [12, 24, 28, 40, 52, 56, 68, 80] {
            grid.set(index, Cell::Filled(5));
        }
        // confirma que é "hidden" e não "naked": índice 0 tem >1 candidato
        assert!(candidates_for(&grid, 0).count() > 1);

        let candidatos = CandidateGrid::from_grid(&grid);
        let d = HiddenSingle.apply(&grid, &candidatos).unwrap();
        assert_eq!(d.action, Action::Place { index: 0, value: 5 });
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        let candidatos = CandidateGrid::from_grid(&grid);
        assert_eq!(HiddenSingle.apply(&grid, &candidatos), None);
    }
}
