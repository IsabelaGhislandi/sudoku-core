#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Filled(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    cells: [Cell; 81],
}

impl Grid {
    pub fn empty() -> Self {
        Grid { cells: [Cell::Empty; 81] }
    }

    pub fn get(&self, index: usize) -> Cell {
        self.cells[index]
    }

    pub fn set(&mut self, index: usize, cell: Cell) {
        self.cells[index] = cell;
    }

    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|c| matches!(c, Cell::Filled(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_comeca_toda_vazia() {
        let grid = Grid::empty();
        assert!((0..81).all(|i| grid.get(i) == Cell::Empty));
        assert!(!grid.is_complete());
    }

    #[test]
    fn set_e_get_de_uma_celula() {
        let mut grid = Grid::empty();
        grid.set(40, Cell::Filled(7));
        assert_eq!(grid.get(40), Cell::Filled(7));
        assert_eq!(grid.get(0), Cell::Empty);
    }

    #[test]
    fn is_complete_quando_todas_preenchidas() {
        let mut grid = Grid::empty();
        for i in 0..81 {
            grid.set(i, Cell::Filled(1));
        }
        assert!(grid.is_complete());
    }
}
