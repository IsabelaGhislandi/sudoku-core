use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};

pub struct HiddenSingle;

impl Technique for HiddenSingle {
    fn level(&self) -> Difficulty {
        Difficulty::Facil
    }

    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        // posicoes[u][v]: posições da unidade u, entre as células vazias, onde v cabe.
        // Montar tudo numa passada só é bem mais barato do que varrer a unidade de
        // novo para cada dígito, e esta técnica roda a cada passo do solver.
        let all_units = units();
        let mut positions = [[0u16; 10]; 27];
        for (u, unit) in all_units.iter().enumerate() {
            for (pos, &i) in unit.iter().enumerate() {
                if let Cell::Empty = grid.get(i) {
                    // Percorre só os bits ligados, em vez de testar os 9 dígitos.
                    let mut bits = candidates.get(i).bits();
                    while bits != 0 {
                        positions[u][bits.trailing_zeros() as usize] |= 1 << pos;
                        bits &= bits - 1; // desliga o bit mais baixo
                    }
                }
            }
        }
        for value in 1..=9u8 {
            for unit in 0..9 {
                // Linha, coluna e box de mesmo número, nessa ordem.
                for u in [unit, 9 + unit, 18 + unit] {
                    let spots = positions[u][value as usize];
                    if spots.count_ones() == 1 {
                        let index = all_units[u][spots.trailing_zeros() as usize];
                        return Some(deduction(index, value));
                    }
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

        let candidates = CandidateGrid::from_grid(&grid);
        let d = HiddenSingle.apply(&grid, &candidates).unwrap();
        assert_eq!(d.action, Action::Place { index: 0, value: 5 });
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        let candidates = CandidateGrid::from_grid(&grid);
        assert_eq!(HiddenSingle.apply(&grid, &candidates), None);
    }
}
