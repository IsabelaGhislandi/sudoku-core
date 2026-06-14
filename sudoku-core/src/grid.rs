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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    WrongLength(usize),
    InvalidChar(char),
}

impl Grid {
    pub fn from_line(line: &str) -> Result<Grid, ParseError> {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != 81 {
            return Err(ParseError::WrongLength(chars.len()));
        }
        let mut grid = Grid::empty();
        for (i, ch) in chars.into_iter().enumerate() {
            let cell = match ch {
                '.' | '0' => Cell::Empty,
                '1'..='9' => Cell::Filled(ch as u8 - b'0'),
                other => return Err(ParseError::InvalidChar(other)),
            };
            grid.set(i, cell);
        }
        Ok(grid)
    }

    pub fn to_line(&self) -> String {
        self.cells
            .iter()
            .map(|c| match c {
                Cell::Empty => '.',
                Cell::Filled(n) => (b'0' + *n) as char,
            })
            .collect()
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

    #[test]
    fn from_line_aceita_ponto_e_zero_como_vazio() {
        let grid = Grid::from_line(&".".repeat(81)).unwrap();
        assert!((0..81).all(|i| grid.get(i) == Cell::Empty));
    }

    #[test]
    fn from_line_le_digitos() {
        let mut line = String::from("53..7....");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();
        assert_eq!(grid.get(0), Cell::Filled(5));
        assert_eq!(grid.get(1), Cell::Filled(3));
        assert_eq!(grid.get(2), Cell::Empty);
        assert_eq!(grid.get(4), Cell::Filled(7));
    }

    #[test]
    fn from_line_rejeita_tamanho_errado() {
        assert_eq!(Grid::from_line("123"), Err(ParseError::WrongLength(3)));
    }

    #[test]
    fn from_line_rejeita_char_invalido() {
        let line = "x".to_string() + &".".repeat(80);
        assert_eq!(Grid::from_line(&line), Err(ParseError::InvalidChar('x')));
    }

    #[test]
    fn to_line_faz_roundtrip() {
        let mut line = String::from("534678912");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();
        assert_eq!(grid.to_line(), line);
    }
}
