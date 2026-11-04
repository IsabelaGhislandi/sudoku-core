use sudoku_core::{Cell, Difficulty, GenError, Puzzle, generate};

use crate::config::GameConfig;

/// O que a tela mostra numa célula. Não existe estado "errado": um número errado nunca fica
/// no tabuleiro, só custa uma vida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    Empty,
    /// Número do enunciado.
    Given(u8),
    /// Número certo colocado pela pessoa. Fica travado.
    Placed(u8),
    /// Número certo revelado por uma dica. Fica travado.
    Hinted(u8),
}

impl CellState {
    /// O número da célula, se houver.
    pub fn value(self) -> Option<u8> {
        match self {
            CellState::Empty => None,
            CellState::Given(v) | CellState::Placed(v) | CellState::Hinted(v) => Some(v),
        }
    }
}

/// Em que pé está a partida. `Won` e `Lost` são finais: depois deles, nenhuma ação é aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Lost,
}

/// Uma partida em andamento: o puzzle e tudo o que a pessoa fez nele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    puzzle: Puzzle,
    cells: [CellState; 81],
    lives_left: u8,
    hints_left: u8,
    mistakes: u32,
    hints_used: u8,
    status: Status,
}

impl Game {
    /// Começa uma partida nova do nível pedido.
    ///
    /// Se a seed esgotar o teto de trabalho do gerador (`AttemptsExhausted`), tenta a seed
    /// seguinte (`seed + 1`, com volta no `u64::MAX`), até conseguir. Essa regra faz parte do
    /// determinismo: mesma seed e mesmo nível dão sempre a mesma partida, e `puzzle().seed`
    /// diz qual seed foi usada de fato.
    pub fn new(difficulty: Difficulty, seed: u64, config: GameConfig) -> Game {
        let mut seed = seed;
        loop {
            match generate(difficulty, seed) {
                Ok(puzzle) => return Game::from_puzzle(puzzle, config),
                Err(GenError::AttemptsExhausted(_)) => seed = seed.wrapping_add(1),
            }
        }
    }

    /// Começa uma partida a partir de um puzzle já gerado (por exemplo, recebido do servidor).
    pub fn from_puzzle(puzzle: Puzzle, config: GameConfig) -> Game {
        let cells = std::array::from_fn(|i| match puzzle.givens.get(i) {
            Cell::Filled(v) => CellState::Given(v),
            Cell::Empty => CellState::Empty,
        });
        Game {
            puzzle,
            cells,
            lives_left: config.lives.get(),
            hints_left: config.hints,
            mistakes: 0,
            hints_used: 0,
            status: Status::Playing,
        }
    }

    pub fn puzzle(&self) -> &Puzzle {
        &self.puzzle
    }

    /// Estado da célula `index` (0..81, linha a linha). Entra em pânico fora disso, como
    /// `Grid::get`: ler uma célula inexistente é erro de programação, não jogada.
    pub fn cell(&self, index: usize) -> CellState {
        self.cells[index]
    }

    pub fn lives_left(&self) -> u8 {
        self.lives_left
    }

    pub fn hints_left(&self) -> u8 {
        self.hints_left
    }

    /// Números errados colocados até agora.
    pub fn mistakes(&self) -> u32 {
        self.mistakes
    }

    pub fn hints_used(&self) -> u8 {
        self.hints_used
    }

    pub fn status(&self) -> Status {
        self.status
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partida_nova_comeca_jogando_com_vidas_e_dicas_cheias() {
        let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        assert_eq!(game.status(), Status::Playing);
        assert_eq!((game.lives_left(), game.hints_left()), (3, 2));
        assert_eq!((game.mistakes(), game.hints_used()), (0, 0));
        assert_eq!(game.puzzle(), &generate(Difficulty::Facil, 42).unwrap());
    }

    #[test]
    fn celulas_dadas_e_vazias_espelham_o_enunciado() {
        let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        for i in 0..81 {
            match game.puzzle().givens.get(i) {
                Cell::Filled(v) => assert_eq!(game.cell(i), CellState::Given(v)),
                Cell::Empty => assert_eq!(game.cell(i), CellState::Empty),
            }
        }
        let empties = (0..81)
            .filter(|&i| game.cell(i) == CellState::Empty)
            .count();
        assert_eq!(empties, 57, "a seed 42 do Fácil tem 57 células vazias");
    }

    #[test]
    fn seed_esgotada_passa_para_a_seguinte() {
        // A seed 15 esgota o teto do Muito Difícil; a 16 não.
        let game = Game::new(Difficulty::MuitoDificil, 15, GameConfig::default());
        assert_eq!(game.puzzle().seed, 16);
    }
}
