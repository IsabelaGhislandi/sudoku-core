use super::{Deduction, Technique};
use crate::candidates::candidates_for;
use crate::difficulty::Difficulty;
use crate::grid::{box_of, col_of, row_of, Cell, Grid};

pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn apply(&self, grid: &Grid) -> Option<Deduction> {
        for value in 1..=9u8 {
            for unit in 0..9 {
                if let Some(index) = sole_spot(grid, value, |i| row_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, value, |i| col_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, value, |i| box_of(i) == unit) {
                    return Some(deduction(index, value));
                }
            }
        }
        None
    }
}

fn deduction(index: usize, value: u8) -> Deduction {
    Deduction {
        index,
        value,
        technique: "Hidden Single",
        level: Difficulty::Facil,
    }
}

/// Retorna a única célula vazia da unidade que pode receber `value`, se existir
/// exatamente uma.
fn sole_spot(grid: &Grid, value: u8, in_unit: impl Fn(usize) -> bool) -> Option<usize> {
    let mut found: Option<usize> = None;
    for i in 0..81 {
        if !in_unit(i) {
            continue;
        }
        if let Cell::Empty = grid.get(i) {
            if candidates_for(grid, i).contains(value) {
                if found.is_some() {
                    return None; // mais de um lugar possível
                }
                found = Some(i);
            }
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::candidates_for;
    use crate::grid::Grid;

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

        let d = HiddenSingle.apply(&grid).unwrap();
        assert_eq!(d.index, 0);
        assert_eq!(d.value, 5);
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        assert_eq!(HiddenSingle.apply(&grid), None);
    }
}
