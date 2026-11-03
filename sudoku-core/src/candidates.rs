use crate::grid::{Cell, Grid, box_of, col_of, row_of};
use crate::techniques::units;

/// Bitmask dos dígitos candidatos 1..=9 (bit n ligado => dígito n é candidato).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateSet(u16);

impl CandidateSet {
    pub fn contains(&self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    pub fn values(&self) -> Vec<u8> {
        self.digits().collect()
    }

    /// Os mesmos dígitos de `values`, sem alocar um `Vec`. As técnicas usam esta forma
    /// porque o gerador as chama milhares de vezes por puzzle.
    pub(crate) fn digits(self) -> impl Iterator<Item = u8> {
        (1..=9).filter(move |&v| self.contains(v))
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn remove(&mut self, value: u8) {
        self.0 &= !(1 << value);
    }

    /// Dígitos que estão em pelo menos um dos dois conjuntos.
    pub(crate) fn union(self, other: CandidateSet) -> CandidateSet {
        CandidateSet(self.0 | other.0)
    }
}

/// Candidatos de todas as células, guardados durante uma resolução.
///
/// `candidates_for` só enxerga o que está na grade. As técnicas de Médio removem
/// candidatos sem colocar valor nenhum, e essa informação não existe na grade: se
/// não for guardada aqui, se perde no passo seguinte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    cells: [CandidateSet; 81],
}

impl CandidateGrid {
    /// Dá o mesmo resultado que `candidates_for` em cada célula, mas lê a grade uma vez
    /// só: o gerador monta candidatos a cada solve, milhares de vezes por puzzle.
    pub fn from_grid(grid: &Grid) -> Self {
        // usados[u]: dígitos já presentes na unidade u (linhas, colunas, boxes).
        let mut used = [0u16; 27];
        for i in 0..81 {
            if let Cell::Filled(v) = grid.get(i) {
                for u in [row_of(i), 9 + col_of(i), 18 + box_of(i)] {
                    used[u] |= 1 << v;
                }
            }
        }
        CandidateGrid {
            cells: std::array::from_fn(|i| match grid.get(i) {
                Cell::Filled(_) => CandidateSet(0),
                Cell::Empty => {
                    let seen = used[row_of(i)] | used[9 + col_of(i)] | used[18 + box_of(i)];
                    CandidateSet(ALL_DIGITS & !seen)
                }
            }),
        }
    }

    pub fn get(&self, index: usize) -> CandidateSet {
        self.cells[index]
    }

    /// Registra um valor colocado: a célula fica sem candidatos, e o dígito sai
    /// da linha, da coluna e do box dela.
    pub fn place(&mut self, index: usize, value: u8) {
        let all_units = units();
        for u in [row_of(index), 9 + col_of(index), 18 + box_of(index)] {
            for &i in &all_units[u] {
                self.cells[i].remove(value);
            }
        }
        self.cells[index] = CandidateSet(0);
    }

    pub fn eliminate(&mut self, index: usize, value: u8) {
        self.cells[index].remove(value);
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
        if let Cell::Filled(v) = grid.get(i)
            && (row_of(i) == r || col_of(i) == c || box_of(i) == b)
        {
            used |= 1 << v;
        }
    }
    CandidateSet(ALL_DIGITS & !used)
}

/// Bits 1..=9 ligados: todos os dígitos.
const ALL_DIGITS: u16 = 0b11_1111_1110;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn uniao_junta_os_candidatos() {
        let a = CandidateSet((1 << 1) | (1 << 2));
        let b = CandidateSet((1 << 2) | (1 << 3));
        assert_eq!(a.union(b).values(), vec![1, 2, 3]);
    }

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

    fn grid_with_givens() -> Grid {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(13, Cell::Filled(3));
        grid.set(40, Cell::Filled(9));
        grid
    }

    #[test]
    fn construcao_bate_com_candidates_for() {
        let grid = grid_with_givens();
        let candidates = CandidateGrid::from_grid(&grid);
        for i in 0..81 {
            assert_eq!(candidates.get(i), candidates_for(&grid, i), "célula {i}");
        }
    }

    #[test]
    fn colocar_limpa_a_celula_e_os_vizinhos() {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        c.place(0, 5);
        assert_eq!(c.get(0).count(), 0);
        assert!(!c.get(8).contains(5)); // linha
        assert!(!c.get(72).contains(5)); // coluna
        assert!(!c.get(20).contains(5)); // box
        assert!(c.get(40).contains(5)); // sem relação com a célula 0
        assert!(c.get(8).contains(6)); // os outros dígitos do vizinho ficam
    }

    #[test]
    fn colocar_equivale_a_reconstruir_da_grade() {
        // É esta equivalência que mantém os puzzles Fáceis idênticos aos de antes.
        let mut grid = grid_with_givens();
        let mut candidates = CandidateGrid::from_grid(&grid);
        grid.set(27, Cell::Filled(7));
        candidates.place(27, 7);
        assert_eq!(candidates, CandidateGrid::from_grid(&grid));
    }

    #[test]
    fn eliminar_remove_so_o_par_pedido() {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        c.eliminate(0, 5);
        assert!(!c.get(0).contains(5));
        assert!(c.get(0).contains(6));
        assert!(c.get(1).contains(5));
    }
}
