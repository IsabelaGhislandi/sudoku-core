//! Candidatos: os dígitos que ainda cabem em cada célula vazia.

use crate::grid::{Cell, Grid, box_of, col_of, row_of};
use crate::techniques::units;

/// Bitmask dos dígitos candidatos 1..=9 (bit n ligado => dígito n é candidato).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateSet(u16);

impl CandidateSet {
    /// `true` se `value` é candidato. `value` vai de 1 a 9; acima de 15 dá panic em
    /// debug (shift maior que o `u16`).
    pub fn contains(&self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    /// Os dígitos candidatos em ordem crescente. Aloca um `Vec`, então fica para quem
    /// está fora do caminho quente; as técnicas usam `digits`.
    pub fn values(&self) -> Vec<u8> {
        self.digits().collect()
    }

    /// Os mesmos dígitos de `values`, sem alocar um `Vec`. As técnicas usam esta forma
    /// porque o gerador as chama milhares de vezes por puzzle.
    pub(crate) fn digits(self) -> impl Iterator<Item = u8> {
        (1..=9).filter(move |&v| self.contains(v))
    }

    /// A máscara crua: bit n ligado quer dizer dígito n candidato. Para técnicas que
    /// combinam conjuntos com operações de bit no caminho quente.
    pub(crate) fn bits(self) -> u16 {
        self.0
    }

    /// Quantos dígitos são candidatos. Com 1, a célula é um Naked Single.
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    /// Tira `value` do conjunto. Tirar um dígito que não está lá não muda nada.
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
///
/// Guarda também, para cada unidade e dígito, a máscara das posições da unidade onde o
/// dígito ainda cabe. As técnicas de unidade (Hidden Pair, Pointing Pair, X-Wing...)
/// precisam dela a cada chamada, e montá-la de novo era a maior parte do tempo delas.
/// Ela é sempre consequência de `cells`: só muda junto, em `place` e `eliminate`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateGrid {
    cells: [CandidateSet; 81],
    /// `positions[u][d]`: bit `p` ligado quer dizer que `units()[u][p]` tem `d` candidato.
    positions: [[u16; 10]; 27],
}

/// As três unidades de cada célula, com a posição da célula dentro de cada uma, na mesma
/// numeração de `units()`: linha `r`, coluna `9 + c`, box `18 + b`.
static SLOTS: [[(usize, usize); 3]; 81] = build_slots();

const fn build_slots() -> [[(usize, usize); 3]; 81] {
    let mut slots = [[(0, 0); 3]; 81];
    let mut i = 0;
    while i < 81 {
        let (r, c) = (i / 9, i % 9);
        slots[i] = [
            (r, c),
            (9 + c, r),
            (18 + r / 3 * 3 + c / 3, r % 3 * 3 + c % 3),
        ];
        i += 1;
    }
    slots
}

impl CandidateGrid {
    /// Dá o mesmo resultado que `candidates_for` em cada célula, mas lê a grade uma vez
    /// só: o gerador monta candidatos a cada solve, milhares de vezes por puzzle.
    pub fn from_grid(grid: &Grid) -> Self {
        let cells = Self::cells_from_grid(grid);
        let mut positions = [[0u16; 10]; 27];
        for (i, set) in cells.iter().enumerate() {
            let mut bits = set.0;
            while bits != 0 {
                let value = bits.trailing_zeros() as usize;
                bits &= bits - 1; // desliga o bit mais baixo
                for (u, pos) in SLOTS[i] {
                    positions[u][value] |= 1 << pos;
                }
            }
        }
        CandidateGrid { cells, positions }
    }

    fn cells_from_grid(grid: &Grid) -> [CandidateSet; 81] {
        // usados[u]: dígitos já presentes na unidade u (linhas, colunas, boxes).
        let mut used = [0u16; 27];
        for i in 0..81 {
            if let Cell::Filled(v) = grid.get(i) {
                for u in [row_of(i), 9 + col_of(i), 18 + box_of(i)] {
                    used[u] |= 1 << v;
                }
            }
        }
        std::array::from_fn(|i| match grid.get(i) {
            Cell::Filled(_) => CandidateSet(0),
            Cell::Empty => {
                let seen = used[row_of(i)] | used[9 + col_of(i)] | used[18 + box_of(i)];
                CandidateSet(ALL_DIGITS & !seen)
            }
        })
    }

    /// Os candidatos da célula `index`, vazio se ela estiver preenchida.
    pub fn get(&self, index: usize) -> CandidateSet {
        self.cells[index]
    }

    /// Máscara das posições da unidade `unit` (numeração de `units()`) onde `value` ainda
    /// é candidato.
    pub(crate) fn positions(&self, unit: usize, value: u8) -> u16 {
        self.positions[unit][value as usize]
    }

    /// Registra um valor colocado: a célula fica sem candidatos, e o dígito sai
    /// da linha, da coluna e do box dela.
    pub fn place(&mut self, index: usize, value: u8) {
        let all_units = units();
        for (u, _) in SLOTS[index] {
            for &i in &all_units[u] {
                self.eliminate(i, value);
            }
        }
        let mut rest = self.cells[index].0;
        while rest != 0 {
            let digit = rest.trailing_zeros() as u8;
            rest &= rest - 1;
            self.eliminate(index, digit);
        }
    }

    /// Remove o candidato, mantendo a tabela de posições em dia. Remover um candidato que
    /// já não está lá não muda nada.
    pub fn eliminate(&mut self, index: usize, value: u8) {
        if self.cells[index].contains(value) {
            self.cells[index].remove(value);
            for (u, pos) in SLOTS[index] {
                self.positions[u][value as usize] &= !(1 << pos);
            }
        }
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

    /// Confere a tabela de posições contra as células, unidade por unidade.
    fn assert_positions_match(c: &CandidateGrid) {
        for (u, unit) in units().iter().enumerate() {
            for value in 1..=9u8 {
                let expected = unit
                    .iter()
                    .enumerate()
                    .filter(|&(_, &i)| c.get(i).contains(value))
                    .fold(0u16, |m, (pos, _)| m | (1 << pos));
                assert_eq!(
                    c.positions(u, value),
                    expected,
                    "unidade {u}, dígito {value}"
                );
            }
        }
    }

    #[test]
    fn posicoes_acompanham_construcao_colocacao_e_eliminacao() {
        let mut grid = grid_with_givens();
        let mut c = CandidateGrid::from_grid(&grid);
        assert_positions_match(&c);
        grid.set(27, Cell::Filled(7));
        c.place(27, 7);
        assert_positions_match(&c);
        c.eliminate(30, 4);
        c.eliminate(30, 4); // eliminar de novo não pode desfazer nada
        c.eliminate(80, 1);
        assert_positions_match(&c);
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
