use sudoku_core::{Cell, Difficulty, GenError, Puzzle, generate};

use crate::config::GameConfig;
use crate::hint;
use crate::notes::Notes;

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

/// O que aconteceu com uma jogada aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceOutcome {
    /// O número estava certo e travou na célula.
    Correct,
    /// O número estava errado: custou uma vida e não ficou no tabuleiro. Volta a célula e o
    /// número tentados, para a tela mostrar o vermelho no lugar certo.
    Wrong { index: usize, value: u8 },
}

/// A célula que uma dica revelou e o número certo dela.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintOutcome {
    pub index: usize,
    pub value: u8,
}

/// Por que uma ação foi recusada. Uma ação recusada não muda nada na partida: não custa vida
/// nem conta como erro.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    /// A partida já terminou (vitória ou derrota).
    GameOver,
    /// Índice fora de 0..81.
    InvalidIndex,
    /// Número fora de 1..=9.
    InvalidValue,
    /// A célula já tem número (do enunciado, de jogada certa ou de dica).
    CellNotEmpty,
    /// Não sobrou dica.
    NoHintsLeft,
}

/// Uma partida em andamento: o puzzle e tudo o que a pessoa fez nele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    puzzle: Puzzle,
    cells: [CellState; 81],
    notes: [Notes; 81],
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
            notes: [Notes::default(); 81],
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

    /// Anotações da célula `index`. Uma célula com número nunca tem anotações. Entra em pânico
    /// fora de 0..81, como `cell`.
    pub fn notes(&self, index: usize) -> Notes {
        self.notes[index]
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

    /// Coloca `value` na célula `index`, conferindo na hora com a solução.
    ///
    /// Certo: o número trava, e a partida termina em vitória se era a última célula vazia.
    /// Errado: perde uma vida, a célula continua vazia, e a partida termina em derrota se as
    /// vidas acabaram. As recusas são conferidas nesta ordem: partida terminada, índice,
    /// número e célula ocupada.
    pub fn place(&mut self, index: usize, value: u8) -> Result<PlaceOutcome, MoveError> {
        self.check_empty_cell(index, value)?;
        if value == self.solution_value(index) {
            self.lock(index, CellState::Placed(value));
            self.finish_if_complete();
            Ok(PlaceOutcome::Correct)
        } else {
            self.mistakes += 1;
            self.lives_left -= 1;
            if self.lives_left == 0 {
                self.status = Status::Lost;
            }
            Ok(PlaceOutcome::Wrong { index, value })
        }
    }

    /// Liga ou desliga a anotação `value` na célula `index`. Só vale em célula vazia, e nada
    /// muda nas outras células. Recusa as mesmas coisas que `place`, na mesma ordem.
    pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), MoveError> {
        self.check_empty_cell(index, value)?;
        self.notes[index].toggle(value);
        Ok(())
    }

    /// Revela o número certo de uma célula vazia, sorteada pelo jogo, e o trava como `Hinted`.
    ///
    /// O sorteio é determinístico (ver `hint::pick`): depende só da seed do puzzle e de
    /// quantas dicas já foram usadas. Recusa com `GameOver` depois do fim e com `NoHintsLeft`
    /// quando as dicas acabaram, nessa ordem. Revelar a última célula vazia vence a partida.
    pub fn hint(&mut self) -> Result<HintOutcome, MoveError> {
        if self.status != Status::Playing {
            return Err(MoveError::GameOver);
        }
        if self.hints_left == 0 {
            return Err(MoveError::NoHintsLeft);
        }
        let empties: Vec<usize> = (0..81)
            .filter(|&i| self.cells[i] == CellState::Empty)
            .collect();
        let index = empties[hint::pick(self.puzzle.seed, self.hints_used, empties.len())];
        let value = self.solution_value(index);
        self.lock(index, CellState::Hinted(value));
        self.hints_left -= 1;
        self.hints_used += 1;
        self.finish_if_complete();
        Ok(HintOutcome { index, value })
    }

    /// Trava um número certo na célula e apaga as anotações dela (só dela).
    fn lock(&mut self, index: usize, state: CellState) {
        self.cells[index] = state;
        self.notes[index] = Notes::default();
    }

    /// As recusas comuns a toda ação numa célula vazia, na ordem documentada em `place`.
    fn check_empty_cell(&self, index: usize, value: u8) -> Result<(), MoveError> {
        if self.status != Status::Playing {
            return Err(MoveError::GameOver);
        }
        if index >= 81 {
            return Err(MoveError::InvalidIndex);
        }
        if !(1..=9).contains(&value) {
            return Err(MoveError::InvalidValue);
        }
        if self.cells[index] != CellState::Empty {
            return Err(MoveError::CellNotEmpty);
        }
        Ok(())
    }

    fn solution_value(&self, index: usize) -> u8 {
        match self.puzzle.solution.get(index) {
            Cell::Filled(v) => v,
            Cell::Empty => unreachable!("a solução de um Puzzle é completa"),
        }
    }

    fn finish_if_complete(&mut self) {
        if !self.cells.contains(&CellState::Empty) {
            self.status = Status::Won;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU8;

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

    fn easy_game() -> Game {
        Game::new(Difficulty::Facil, 42, GameConfig::default())
    }

    fn solution_value(game: &Game, index: usize) -> u8 {
        match game.puzzle().solution.get(index) {
            Cell::Filled(v) => v,
            Cell::Empty => panic!("a solução não tem células vazias"),
        }
    }

    fn wrong_value(game: &Game, index: usize) -> u8 {
        let right = solution_value(game, index);
        if right == 9 { 1 } else { right + 1 }
    }

    fn first_empty(game: &Game) -> usize {
        (0..81)
            .find(|&i| game.cell(i) == CellState::Empty)
            .expect("há célula vazia")
    }

    fn first_given(game: &Game) -> usize {
        (0..81)
            .find(|&i| matches!(game.cell(i), CellState::Given(_)))
            .expect("há célula dada")
    }

    fn make_mistakes(game: &mut Game, count: usize) {
        let i = first_empty(game);
        for _ in 0..count {
            let w = wrong_value(game, i);
            game.place(i, w).unwrap();
        }
    }

    #[test]
    fn numero_certo_trava_na_celula() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v), Ok(PlaceOutcome::Correct));
        assert_eq!(game.cell(i), CellState::Placed(v));
        assert_eq!(game.place(i, v), Err(MoveError::CellNotEmpty));
        assert_eq!(game.lives_left(), 3);
    }

    #[test]
    fn numero_errado_tira_vida_e_sai_do_tabuleiro() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let w = wrong_value(&game, i);
        assert_eq!(
            game.place(i, w),
            Ok(PlaceOutcome::Wrong { index: i, value: w })
        );
        assert_eq!(game.cell(i), CellState::Empty);
        assert_eq!((game.lives_left(), game.mistakes()), (2, 1));
        assert_eq!(game.status(), Status::Playing);
    }

    #[test]
    fn tres_erros_terminam_a_partida_em_derrota() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        assert_eq!(game.status(), Status::Lost);
        assert_eq!((game.lives_left(), game.mistakes()), (0, 3));
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v), Err(MoveError::GameOver));
        assert_eq!(game.cell(i), CellState::Empty);
    }

    #[test]
    fn uma_vida_perde_no_primeiro_erro() {
        let config = GameConfig {
            lives: NonZeroU8::MIN,
            hints: 2,
        };
        let mut game = Game::new(Difficulty::Facil, 42, config);
        make_mistakes(&mut game, 1);
        assert_eq!(game.status(), Status::Lost);
    }

    #[test]
    fn preencher_todas_as_celulas_vence() {
        let mut game = easy_game();
        for i in 0..81 {
            if game.cell(i) == CellState::Empty {
                let v = solution_value(&game, i);
                assert_eq!(game.place(i, v), Ok(PlaceOutcome::Correct));
            }
        }
        assert_eq!(game.status(), Status::Won);
        assert_eq!(game.mistakes(), 0);
        let g = first_given(&game);
        assert_eq!(game.place(g, 1), Err(MoveError::GameOver));
    }

    #[test]
    fn jogada_em_celula_dada_nao_conta_erro() {
        let mut game = easy_game();
        let g = first_given(&game);
        for v in 1..=9 {
            assert_eq!(game.place(g, v), Err(MoveError::CellNotEmpty));
        }
        assert_eq!((game.lives_left(), game.mistakes()), (3, 0));
    }

    #[test]
    fn indice_ou_valor_invalido_nao_conta_erro() {
        let mut game = easy_game();
        let i = first_empty(&game);
        assert_eq!(game.place(81, 1), Err(MoveError::InvalidIndex));
        assert_eq!(game.place(i, 0), Err(MoveError::InvalidValue));
        assert_eq!(game.place(i, 10), Err(MoveError::InvalidValue));
        assert_eq!((game.lives_left(), game.mistakes()), (3, 0));
        assert_eq!(game.cell(i), CellState::Empty);
    }

    fn empties(game: &Game) -> Vec<usize> {
        (0..81)
            .filter(|&i| game.cell(i) == CellState::Empty)
            .collect()
    }

    #[test]
    fn anotacao_liga_e_desliga() {
        let mut game = easy_game();
        let i = first_empty(&game);
        assert_eq!(game.toggle_note(i, 3), Ok(()));
        assert_eq!(game.toggle_note(i, 7), Ok(()));
        assert_eq!(game.notes(i).values(), vec![3, 7]);
        assert_eq!(game.toggle_note(i, 3), Ok(()));
        assert_eq!(game.notes(i).values(), vec![7]);
    }

    #[test]
    fn numero_certo_limpa_so_as_anotacoes_da_propria_celula() {
        let mut game = easy_game();
        let (a, b) = (empties(&game)[0], empties(&game)[1]);
        game.toggle_note(a, 5).unwrap();
        game.toggle_note(b, 5).unwrap();
        let v = solution_value(&game, a);
        game.place(a, v).unwrap();
        assert!(game.notes(a).is_empty());
        assert_eq!(game.notes(b).values(), vec![5]);
    }

    #[test]
    fn numero_errado_mantem_as_anotacoes() {
        let mut game = easy_game();
        let i = first_empty(&game);
        game.toggle_note(i, 2).unwrap();
        game.toggle_note(i, 4).unwrap();
        let w = wrong_value(&game, i);
        game.place(i, w).unwrap();
        assert_eq!(game.notes(i).values(), vec![2, 4]);
    }

    #[test]
    fn anotacao_so_em_celula_vazia() {
        let mut game = easy_game();
        let g = first_given(&game);
        assert_eq!(game.toggle_note(g, 1), Err(MoveError::CellNotEmpty));
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        game.place(i, v).unwrap();
        assert_eq!(game.toggle_note(i, 1), Err(MoveError::CellNotEmpty));
        assert!(game.notes(i).is_empty());
    }

    #[test]
    fn anotacao_invalida_ou_depois_do_fim_eh_recusada() {
        let mut game = easy_game();
        let i = first_empty(&game);
        assert_eq!(game.toggle_note(81, 1), Err(MoveError::InvalidIndex));
        assert_eq!(game.toggle_note(i, 0), Err(MoveError::InvalidValue));
        assert_eq!(game.toggle_note(i, 10), Err(MoveError::InvalidValue));
        make_mistakes(&mut game, 3);
        assert_eq!(game.toggle_note(i, 1), Err(MoveError::GameOver));
        assert!(game.notes(i).is_empty());
    }

    #[test]
    fn dica_revela_o_valor_da_solucao_e_trava() {
        let mut game = easy_game();
        let h = game.hint().unwrap();
        assert_eq!(game.cell(h.index), CellState::Hinted(h.value));
        assert_eq!(h.value, solution_value(&game, h.index));
        assert_eq!((game.hints_left(), game.hints_used()), (1, 1));
        assert_eq!((game.lives_left(), game.mistakes()), (3, 0));
        assert_eq!(game.place(h.index, h.value), Err(MoveError::CellNotEmpty));
    }

    #[test]
    fn dica_eh_deterministica() {
        let (mut a, mut b) = (easy_game(), easy_game());
        let first = (a.hint().unwrap(), a.hint().unwrap());
        let second = (b.hint().unwrap(), b.hint().unwrap());
        assert_eq!(first, second);
        assert_ne!(first.0.index, first.1.index);
    }

    #[test]
    fn dica_da_seed_42_fica_travada() {
        // Golden: mudou? É bug ou decisão consciente, registrada no mesmo commit.
        let mut game = easy_game();
        assert_eq!(
            game.hint(),
            Ok(HintOutcome {
                index: 40,
                value: 9
            })
        );
    }

    #[test]
    fn dica_limpa_as_anotacoes_da_celula() {
        let mut game = easy_game();
        for i in empties(&game) {
            game.toggle_note(i, 1).unwrap();
        }
        let h = game.hint().unwrap();
        assert!(game.notes(h.index).is_empty());
        let other = first_empty(&game);
        assert_eq!(game.notes(other).values(), vec![1]);
    }

    #[test]
    fn sem_dicas_restantes_recusa() {
        let mut game = easy_game();
        game.hint().unwrap();
        game.hint().unwrap();
        assert_eq!(game.hint(), Err(MoveError::NoHintsLeft));
        assert_eq!(game.hints_used(), 2);

        let config = GameConfig {
            lives: NonZeroU8::new(3).unwrap(),
            hints: 0,
        };
        let mut no_hints = Game::new(Difficulty::Facil, 42, config);
        assert_eq!(no_hints.hint(), Err(MoveError::NoHintsLeft));
    }

    #[test]
    fn dica_na_ultima_celula_vence() {
        let mut game = easy_game();
        let open = empties(&game);
        for &i in &open[1..] {
            let v = solution_value(&game, i);
            game.place(i, v).unwrap();
        }
        let h = game.hint().unwrap();
        assert_eq!(h.index, open[0]);
        assert_eq!(game.status(), Status::Won);
    }

    #[test]
    fn dica_depois_do_fim_eh_recusada() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        assert_eq!(game.hint(), Err(MoveError::GameOver));
        assert_eq!(game.hints_left(), 2);
    }
}
