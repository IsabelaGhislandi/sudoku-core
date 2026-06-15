use crate::grid::{box_of, col_of, row_of, Cell, Grid};

/// Bitmask dos dígitos candidatos 1..=9 (bit n ligado => dígito n é candidato).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateSet(u16);

impl CandidateSet {
    pub fn contains(&self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    pub fn values(&self) -> Vec<u8> {
        (1..=9).filter(|&v| self.contains(v)).collect()
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// Candidatos de uma célula vazia = dígitos não vistos na sua linha, coluna ou box.
/// Retorna conjunto vazio para células já preenchidas.
pub fn candidates_for(grid: &Grid, index: usize) -> CandidateSet {
    if let Cell::Filled(_) = grid.get(index) {
        return CandidateSet(0);
    }
    let (r, c, b) = (row_of(index), col_of(index), box_of(index));
    let mut used: u16 = 0;
    for i in 0..81 {
        if let Cell::Filled(v) = grid.get(i) {
            if row_of(i) == r || col_of(i) == c || box_of(i) == b {
                used |= 1 << v;
            }
        }
    }
    let all: u16 = 0b11_1111_1110; // bits 1..=9
    CandidateSet(all & !used)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn candidatos_de_celula_vazia() {
        let mut grid = Grid::empty();
        grid.set(1, Cell::Filled(1)); // linha 0
        grid.set(2, Cell::Filled(3)); // linha 0 e box 0
        grid.set(9, Cell::Filled(2)); // coluna 0 e box 0
        let cands = candidates_for(&grid, 0);
        assert_eq!(cands.values(), vec![4, 5, 6, 7, 8, 9]);
        assert_eq!(cands.count(), 6);
        assert!(cands.contains(4));
        assert!(!cands.contains(1));
    }

    #[test]
    fn celula_preenchida_nao_tem_candidatos() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        assert_eq!(candidates_for(&grid, 0).count(), 0);
    }
}
