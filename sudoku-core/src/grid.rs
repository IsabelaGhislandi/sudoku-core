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

pub fn row_of(index: usize) -> usize {
    index / 9
}

pub fn col_of(index: usize) -> usize {
    index % 9
}

pub fn box_of(index: usize) -> usize {
    (row_of(index) / 3) * 3 + col_of(index) / 3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict {
    pub a: usize,
    pub b: usize,
}

impl Grid {
    /// True se colocar `value` em `index` não viola linha/coluna/box.
    pub fn can_place(&self, index: usize, value: u8) -> bool {
        let (r, c, b) = (row_of(index), col_of(index), box_of(index));
        for i in 0..81 {
            if i == index {
                continue;
            }
            if let Cell::Filled(v) = self.cells[i] {
                if v == value && (row_of(i) == r || col_of(i) == c || box_of(i) == b) {
                    return false;
                }
            }
        }
        true
    }
}

/// Lista pares de células preenchidas que conflitam (mesmo valor na linha/coluna/box).
pub fn validate(grid: &Grid) -> Vec<Conflict> {
    let mut out = Vec::new();
    for i in 0..81 {
        if let Cell::Filled(vi) = grid.get(i) {
            for j in (i + 1)..81 {
                if let Cell::Filled(vj) = grid.get(j) {
                    if vi == vj
                        && (row_of(i) == row_of(j)
                            || col_of(i) == col_of(j)
                            || box_of(i) == box_of(j))
                    {
                        out.push(Conflict { a: i, b: j });
                    }
                }
            }
        }
    }
    out
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

    #[test]
    fn helpers_de_linha_coluna_box() {
        assert_eq!((row_of(0), col_of(0), box_of(0)), (0, 0, 0));
        assert_eq!((row_of(80), col_of(80), box_of(80)), (8, 8, 8));
        assert_eq!((row_of(40), col_of(40), box_of(40)), (4, 4, 4));
        // index 3 = linha 0, coluna 3 => box 1
        assert_eq!(box_of(3), 1);
    }

    #[test]
    fn can_place_respeita_linha_coluna_box() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5)); // linha 0, coluna 0, box 0
        assert!(!grid.can_place(1, 5)); // mesma linha
        assert!(!grid.can_place(9, 5)); // mesma coluna
        assert!(!grid.can_place(10, 5)); // mesmo box
        assert!(grid.can_place(80, 5)); // longe: ok
        assert!(grid.can_place(1, 6)); // outro valor: ok
    }

    #[test]
    fn validate_detecta_conflito() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(1, Cell::Filled(5)); // mesma linha => conflito
        let conflitos = validate(&grid);
        assert_eq!(conflitos, vec![Conflict { a: 0, b: 1 }]);
    }

    #[test]
    fn validate_sem_conflito_retorna_vazio() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(1, Cell::Filled(6));
        assert!(validate(&grid).is_empty());
    }
}
