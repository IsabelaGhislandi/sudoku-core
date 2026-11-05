use std::num::NonZeroU8;

use sudoku_core::{Cell, Difficulty, GenError, Grid, Puzzle, generate, validate};

use crate::config::GameConfig;
use crate::hint;
use crate::notes::Notes;
use crate::save::{RestoreError, SAVE_VERSION, SavedGame, difficulty_name, parse_difficulty};

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

/// Por que um `Puzzle` foi recusado por `Game::from_puzzle`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleError {
    /// A solução tem célula vazia.
    IncompleteSolution,
    /// A solução repete um número numa linha, coluna ou box.
    InvalidSolution,
    /// Um número do enunciado é diferente do da solução.
    GivenMismatch,
    /// O enunciado não tem célula vazia: não haveria o que jogar.
    NoEmptyCell,
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
    /// A partida está pausada (ou ainda não começou): o relógio não está rodando.
    Paused,
}

/// O que `Game::restart` devolve. `#[non_exhaustive]` porque a fatia 3 acrescenta o resultado
/// do abandono (reiniciar uma partida em andamento conta como derrota), sem quebrar quem já lê
/// `game`. Fora do crate, só dá para ler os campos, nunca montar o valor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Restart {
    /// A partida nova, pausada: "começar" é o primeiro `resume`.
    pub game: Game,
}

/// Uma partida em andamento: o puzzle e tudo o que a pessoa fez nele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    puzzle: Puzzle,
    /// Guardada para a foto e para reiniciar com as mesmas regras.
    config: GameConfig,
    cells: [CellState; 81],
    notes: [Notes; 81],
    lives_left: u8,
    hints_left: u8,
    mistakes: u32,
    hints_used: u8,
    status: Status,
    /// Tempo jogado até a última pausa (ou até o fim), em milissegundos.
    elapsed_ms: u64,
    /// `Some(t)`: o relógio está rodando desde o instante `t` da casca. `None`: pausada, ainda
    /// não começada ou terminada.
    running_since: Option<u64>,
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
                // O gerador sempre entrega puzzle válido, com solução única e células vazias.
                Ok(puzzle) => return Game::start(puzzle, config),
                Err(GenError::AttemptsExhausted(_)) => seed = seed.wrapping_add(1),
            }
        }
    }

    /// Começa uma partida a partir de um puzzle já gerado.
    ///
    /// Os campos de `Puzzle` são públicos, então ele pode chegar montado à mão ou de fora (de um
    /// arquivo, da rede). Por isso o puzzle é conferido antes de virar partida, nesta ordem:
    /// solução completa, solução sem conflito, números dados iguais à solução e pelo menos uma
    /// célula vazia. Sem isso, uma entrada ruim poderia derrubar a partida (dica dividindo por
    /// zero) ou deixá-la sem fim.
    pub fn from_puzzle(puzzle: Puzzle, config: GameConfig) -> Result<Game, PuzzleError> {
        if !puzzle.solution.is_complete() {
            return Err(PuzzleError::IncompleteSolution);
        }
        if !validate(&puzzle.solution).is_empty() {
            return Err(PuzzleError::InvalidSolution);
        }
        let mut has_empty = false;
        for i in 0..81 {
            match puzzle.givens.get(i) {
                Cell::Empty => has_empty = true,
                given if given != puzzle.solution.get(i) => {
                    return Err(PuzzleError::GivenMismatch);
                }
                _ => {}
            }
        }
        if !has_empty {
            return Err(PuzzleError::NoEmptyCell);
        }
        Ok(Game::start(puzzle, config))
    }

    /// Monta a partida de um puzzle já conferido.
    fn start(puzzle: Puzzle, config: GameConfig) -> Game {
        let cells = std::array::from_fn(|i| match puzzle.givens.get(i) {
            Cell::Filled(v) => CellState::Given(v),
            Cell::Empty => CellState::Empty,
        });
        Game {
            puzzle,
            config,
            cells,
            notes: [Notes::default(); 81],
            lives_left: config.lives.get(),
            hints_left: config.hints,
            mistakes: 0,
            hints_used: 0,
            status: Status::Playing,
            elapsed_ms: 0,
            running_since: None,
        }
    }

    /// Começa outro puzzle do mesmo nível e com a mesma configuração, a partir de uma seed nova
    /// que vem da casca (o Game não sorteia seeds). Vale em qualquer estado: em andamento,
    /// pausada ou terminada ("jogar de novo").
    pub fn restart(&self, seed: u64) -> Restart {
        Restart {
            game: Game::new(self.puzzle.difficulty, seed, self.config),
        }
    }

    /// Tira a foto da partida no instante `now`. Se o relógio estiver rodando, o trecho até
    /// `now` entra no tempo salvo, mas a partida em si não muda nem pausa.
    pub fn save(&self, now: u64) -> SavedGame {
        // Mesmo formato de `Grid::to_line`: '.' para vazia, o dígito para as outras.
        let board = self
            .cells
            .iter()
            .map(|c| c.value().map_or('.', |v| (b'0' + v) as char))
            .collect();
        SavedGame {
            version: SAVE_VERSION,
            difficulty: difficulty_name(self.puzzle.difficulty).to_string(),
            seed: self.puzzle.seed.to_string(),
            givens: self.puzzle.givens.to_line(),
            solution: self.puzzle.solution.to_line(),
            board,
            hinted: (0..81u8)
                .filter(|&i| matches!(self.cells[usize::from(i)], CellState::Hinted(_)))
                .collect(),
            notes: self
                .notes
                .iter()
                .map(|n| n.values().iter().map(|&v| (b'0' + v) as char).collect())
                .collect(),
            lives: self.config.lives.get(),
            hints: self.config.hints,
            lives_left: self.lives_left,
            hints_left: self.hints_left,
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            elapsed_ms: self.elapsed_ms(now),
        }
    }

    /// Restaura uma partida a partir da foto. A partida volta sempre pausada (ou terminada, se
    /// a foto era de uma partida terminada); quem decide continuar é a pessoa.
    ///
    /// A foto vem de fora e pode estar corrompida ou adulterada, então é conferida antes de
    /// virar partida. O status não vem da foto: é derivado do tabuleiro e das vidas.
    pub fn restore(saved: &SavedGame) -> Result<Game, RestoreError> {
        if saved.version != SAVE_VERSION {
            return Err(RestoreError::UnknownVersion(saved.version));
        }
        let difficulty = parse_difficulty(&saved.difficulty)?;
        let seed: u64 = saved.seed.parse().map_err(|_| RestoreError::InvalidSeed)?;
        let grid = |line: &str| Grid::from_line(line).map_err(|_| RestoreError::InvalidGrid);
        let (givens, solution, board) = (
            grid(&saved.givens)?,
            grid(&saved.solution)?,
            grid(&saved.board)?,
        );
        let lives = NonZeroU8::new(saved.lives).ok_or(RestoreError::InvalidLives)?;
        let config = GameConfig {
            lives,
            hints: saved.hints,
        };
        let puzzle = Puzzle {
            givens,
            solution,
            difficulty,
            seed,
        };
        let mut game = Game::from_puzzle(puzzle, config).map_err(RestoreError::Puzzle)?;

        // Tabuleiro: os dados no lugar, e todo outro número igual ao da solução.
        for i in 0..81 {
            match (game.cells[i], board.get(i)) {
                (CellState::Given(g), Cell::Filled(v)) if g == v => {}
                (CellState::Empty, Cell::Empty) => {}
                (CellState::Empty, Cell::Filled(v)) if v == game.solution_value(i) => {
                    game.cells[i] = CellState::Placed(v);
                }
                _ => return Err(RestoreError::BoardMismatch),
            }
        }
        // Dicas: índices em ordem estritamente crescente (sem repetição), cada um numa célula
        // preenchida por jogada. A faixa é conferida antes de indexar o array.
        let mut previous = None;
        for &i in &saved.hinted {
            let i = usize::from(i);
            if i >= 81 || previous.is_some_and(|p| i <= p) {
                return Err(RestoreError::InvalidHints);
            }
            let CellState::Placed(v) = game.cells[i] else {
                return Err(RestoreError::InvalidHints);
            };
            game.cells[i] = CellState::Hinted(v);
            previous = Some(i);
        }
        game.notes = parse_notes(&saved.notes, &game.cells)?;

        // Cada erro custa uma vida, e o `&&` impede a subtração de dar underflow.
        let lives_ok = saved.lives_left <= saved.lives
            && saved.mistakes == u32::from(saved.lives - saved.lives_left);
        if !lives_ok {
            return Err(RestoreError::InvalidLives);
        }
        if saved.hinted.len() != usize::from(saved.hints_used)
            || u16::from(saved.hints_left) + u16::from(saved.hints_used) != u16::from(saved.hints)
        {
            return Err(RestoreError::InvalidHints);
        }
        let complete = !game.cells.contains(&CellState::Empty);
        if complete && saved.lives_left == 0 {
            return Err(RestoreError::InconsistentStatus);
        }

        game.lives_left = saved.lives_left;
        game.hints_left = saved.hints_left;
        game.mistakes = saved.mistakes;
        game.hints_used = saved.hints_used;
        game.elapsed_ms = saved.elapsed_ms;
        game.status = if complete {
            Status::Won
        } else if game.lives_left == 0 {
            Status::Lost
        } else {
            Status::Playing
        };
        Ok(game)
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

    /// A partida está em andamento com o relógio parado. Uma partida nova começa assim:
    /// "começar" é o primeiro `resume`. Uma partida terminada não está pausada.
    pub fn is_paused(&self) -> bool {
        self.status == Status::Playing && self.running_since.is_none()
    }

    /// Liga o relógio no instante `now` (milissegundos de um relógio monotônico da casca). Sem
    /// efeito se já estiver rodando ou se a partida terminou, para a casca poder chamar sem
    /// conferir antes.
    pub fn resume(&mut self, now: u64) {
        if self.is_paused() {
            self.running_since = Some(now);
        }
    }

    /// Para o relógio no instante `now`, guardando o trecho jogado. Sem efeito se já estiver
    /// parado.
    pub fn pause(&mut self, now: u64) {
        self.elapsed_ms = self.elapsed_ms(now);
        self.running_since = None;
    }

    /// Tempo jogado até `now`, sem contar as pausas. Se o relógio da casca voltar (`now` antes
    /// do último `resume`), aquele trecho conta zero em vez de dar underflow. A soma também
    /// satura: o tempo acumulado pode vir de uma foto adulterada, e somar não pode estourar.
    pub fn elapsed_ms(&self, now: u64) -> u64 {
        match self.running_since {
            Some(since) => self.elapsed_ms.saturating_add(now.saturating_sub(since)),
            None => self.elapsed_ms,
        }
    }

    /// Coloca `value` na célula `index`, conferindo na hora com a solução. `now` é o instante
    /// da jogada: se ela terminar a partida, o relógio para ali.
    ///
    /// Certo: o número trava, e a partida termina em vitória se era a última célula vazia.
    /// Errado: perde uma vida, a célula continua vazia, e a partida termina em derrota se as
    /// vidas acabaram. As recusas são conferidas nesta ordem: partida terminada, pausada,
    /// índice, número e célula ocupada.
    pub fn place(&mut self, index: usize, value: u8, now: u64) -> Result<PlaceOutcome, MoveError> {
        self.check_empty_cell(index, value)?;
        if value == self.solution_value(index) {
            self.lock(index, CellState::Placed(value));
            self.finish_if_complete(now);
            Ok(PlaceOutcome::Correct)
        } else {
            self.mistakes += 1;
            self.lives_left -= 1;
            if self.lives_left == 0 {
                self.end(Status::Lost, now);
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
    /// quantas dicas já foram usadas. Recusa com `GameOver` depois do fim, com `Paused` com o
    /// relógio parado e com `NoHintsLeft` quando as dicas acabaram, nessa ordem. Revelar a
    /// última célula vazia vence a partida, e o relógio para em `now`.
    pub fn hint(&mut self, now: u64) -> Result<HintOutcome, MoveError> {
        self.check_running()?;
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
        self.finish_if_complete(now);
        Ok(HintOutcome { index, value })
    }

    /// Termina a partida e congela o relógio em `now`. É o único lugar onde a partida acaba,
    /// para que o tempo total não dependa de a casca lembrar de pausar depois do fim.
    fn end(&mut self, status: Status, now: u64) {
        self.pause(now);
        self.status = status;
    }

    /// As recusas comuns a toda ação: partida terminada, depois partida pausada.
    fn check_running(&self) -> Result<(), MoveError> {
        if self.status != Status::Playing {
            return Err(MoveError::GameOver);
        }
        if self.running_since.is_none() {
            return Err(MoveError::Paused);
        }
        Ok(())
    }

    /// Trava um número certo na célula e apaga as anotações dela (só dela).
    fn lock(&mut self, index: usize, state: CellState) {
        self.cells[index] = state;
        self.notes[index] = Notes::default();
    }

    /// As recusas comuns a toda ação numa célula vazia, na ordem documentada em `place`.
    fn check_empty_cell(&self, index: usize, value: u8) -> Result<(), MoveError> {
        self.check_running()?;
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

    fn finish_if_complete(&mut self, now: u64) {
        if !self.cells.contains(&CellState::Empty) {
            self.end(Status::Won, now);
        }
    }
}

/// Lê as anotações da foto: exatamente 81 entradas, cada uma com dígitos de 1 a 9 em ordem
/// estritamente crescente (o formato que `save` escreve), e nenhuma em célula com número.
fn parse_notes(entries: &[String], cells: &[CellState; 81]) -> Result<[Notes; 81], RestoreError> {
    if entries.len() != 81 {
        return Err(RestoreError::InvalidNotes);
    }
    let mut notes = [Notes::default(); 81];
    for (i, digits) in entries.iter().enumerate() {
        if !digits.is_empty() && cells[i] != CellState::Empty {
            return Err(RestoreError::InvalidNotes);
        }
        let mut previous = 0;
        for ch in digits.chars() {
            let value = match ch.to_digit(10) {
                Some(d @ 1..=9) if d > previous => d,
                _ => return Err(RestoreError::InvalidNotes),
            };
            notes[i].toggle(value as u8);
            previous = value;
        }
    }
    Ok(notes)
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

    /// Partida já começada: relógio rodando desde 0.
    fn started(difficulty: Difficulty, seed: u64) -> Game {
        let mut game = Game::new(difficulty, seed, GameConfig::default());
        game.resume(0);
        game
    }

    fn easy_game() -> Game {
        started(Difficulty::Facil, 42)
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
            game.place(i, w, 0).unwrap();
        }
    }

    #[test]
    fn numero_certo_trava_na_celula() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v, 0), Ok(PlaceOutcome::Correct));
        assert_eq!(game.cell(i), CellState::Placed(v));
        assert_eq!(game.place(i, v, 0), Err(MoveError::CellNotEmpty));
        assert_eq!(game.lives_left(), 3);
    }

    #[test]
    fn numero_errado_tira_vida_e_sai_do_tabuleiro() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let w = wrong_value(&game, i);
        assert_eq!(
            game.place(i, w, 0),
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
        assert_eq!(game.place(i, v, 0), Err(MoveError::GameOver));
        assert_eq!(game.cell(i), CellState::Empty);
    }

    #[test]
    fn uma_vida_perde_no_primeiro_erro() {
        let config = GameConfig {
            lives: NonZeroU8::MIN,
            hints: 2,
        };
        let mut game = Game::new(Difficulty::Facil, 42, config);
        game.resume(0);
        make_mistakes(&mut game, 1);
        assert_eq!(game.status(), Status::Lost);
    }

    #[test]
    fn preencher_todas_as_celulas_vence() {
        let mut game = easy_game();
        for i in 0..81 {
            if game.cell(i) == CellState::Empty {
                let v = solution_value(&game, i);
                assert_eq!(game.place(i, v, 0), Ok(PlaceOutcome::Correct));
            }
        }
        assert_eq!(game.status(), Status::Won);
        assert_eq!(game.mistakes(), 0);
        let g = first_given(&game);
        assert_eq!(game.place(g, 1, 0), Err(MoveError::GameOver));
    }

    #[test]
    fn jogada_em_celula_dada_nao_conta_erro() {
        let mut game = easy_game();
        let g = first_given(&game);
        for v in 1..=9 {
            assert_eq!(game.place(g, v, 0), Err(MoveError::CellNotEmpty));
        }
        assert_eq!((game.lives_left(), game.mistakes()), (3, 0));
    }

    #[test]
    fn indice_ou_valor_invalido_nao_conta_erro() {
        let mut game = easy_game();
        let i = first_empty(&game);
        assert_eq!(game.place(81, 1, 0), Err(MoveError::InvalidIndex));
        assert_eq!(game.place(i, 0, 0), Err(MoveError::InvalidValue));
        assert_eq!(game.place(i, 10, 0), Err(MoveError::InvalidValue));
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
        game.place(a, v, 0).unwrap();
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
        game.place(i, w, 0).unwrap();
        assert_eq!(game.notes(i).values(), vec![2, 4]);
    }

    #[test]
    fn anotacao_so_em_celula_vazia() {
        let mut game = easy_game();
        let g = first_given(&game);
        assert_eq!(game.toggle_note(g, 1), Err(MoveError::CellNotEmpty));
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        game.place(i, v, 0).unwrap();
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
        let h = game.hint(0).unwrap();
        assert_eq!(game.cell(h.index), CellState::Hinted(h.value));
        assert_eq!(h.value, solution_value(&game, h.index));
        assert_eq!((game.hints_left(), game.hints_used()), (1, 1));
        assert_eq!((game.lives_left(), game.mistakes()), (3, 0));
        assert_eq!(
            game.place(h.index, h.value, 0),
            Err(MoveError::CellNotEmpty)
        );
    }

    #[test]
    fn dica_eh_deterministica() {
        let (mut a, mut b) = (easy_game(), easy_game());
        let first = (a.hint(0).unwrap(), a.hint(0).unwrap());
        let second = (b.hint(0).unwrap(), b.hint(0).unwrap());
        assert_eq!(first, second);
        assert_ne!(first.0.index, first.1.index);
    }

    #[test]
    fn dica_da_seed_42_fica_travada() {
        // Golden: mudou? É bug ou decisão consciente, registrada no mesmo commit.
        let mut game = easy_game();
        assert_eq!(
            game.hint(0),
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
        let h = game.hint(0).unwrap();
        assert!(game.notes(h.index).is_empty());
        let other = first_empty(&game);
        assert_eq!(game.notes(other).values(), vec![1]);
    }

    #[test]
    fn sem_dicas_restantes_recusa() {
        let mut game = easy_game();
        game.hint(0).unwrap();
        game.hint(0).unwrap();
        assert_eq!(game.hint(0), Err(MoveError::NoHintsLeft));
        assert_eq!(game.hints_used(), 2);

        let config = GameConfig {
            lives: NonZeroU8::new(3).unwrap(),
            hints: 0,
        };
        let mut no_hints = Game::new(Difficulty::Facil, 42, config);
        no_hints.resume(0);
        assert_eq!(no_hints.hint(0), Err(MoveError::NoHintsLeft));
    }

    #[test]
    fn dica_na_ultima_celula_vence() {
        let mut game = easy_game();
        let open = empties(&game);
        for &i in &open[1..] {
            let v = solution_value(&game, i);
            game.place(i, v, 0).unwrap();
        }
        let h = game.hint(0).unwrap();
        assert_eq!(h.index, open[0]);
        assert_eq!(game.status(), Status::Won);
    }

    fn easy_puzzle() -> Puzzle {
        generate(Difficulty::Facil, 42).unwrap()
    }

    #[test]
    fn puzzle_gerado_eh_aceito() {
        let game = Game::from_puzzle(easy_puzzle(), GameConfig::default()).unwrap();
        assert_eq!(
            game,
            Game::new(Difficulty::Facil, 42, GameConfig::default())
        );
    }

    #[test]
    fn puzzle_com_solucao_incompleta_eh_recusado() {
        let mut puzzle = easy_puzzle();
        puzzle.solution.set(0, Cell::Empty);
        assert_eq!(
            Game::from_puzzle(puzzle, GameConfig::default()),
            Err(PuzzleError::IncompleteSolution)
        );
    }

    #[test]
    fn puzzle_com_solucao_que_quebra_as_regras_eh_recusado() {
        let mut puzzle = easy_puzzle();
        let i = (0..81)
            .find(|&i| puzzle.givens.get(i) == Cell::Empty)
            .unwrap();
        // Duas células da mesma linha com o mesmo número.
        let neighbor = if i % 9 == 0 { i + 1 } else { i - 1 };
        let copied = puzzle.solution.get(neighbor);
        puzzle.solution.set(i, copied);
        assert_eq!(
            Game::from_puzzle(puzzle, GameConfig::default()),
            Err(PuzzleError::InvalidSolution)
        );
    }

    #[test]
    fn puzzle_com_numero_dado_diferente_da_solucao_eh_recusado() {
        let mut puzzle = easy_puzzle();
        let g = (0..81)
            .find(|&i| puzzle.givens.get(i) != Cell::Empty)
            .unwrap();
        let Cell::Filled(v) = puzzle.givens.get(g) else {
            unreachable!()
        };
        puzzle
            .givens
            .set(g, Cell::Filled(if v == 9 { 1 } else { v + 1 }));
        assert_eq!(
            Game::from_puzzle(puzzle, GameConfig::default()),
            Err(PuzzleError::GivenMismatch)
        );
    }

    #[test]
    fn puzzle_sem_celula_vazia_eh_recusado() {
        // Sem a recusa, a partida nunca terminaria e a dica dividiria por zero.
        let mut puzzle = easy_puzzle();
        puzzle.givens = puzzle.solution.clone();
        assert_eq!(
            Game::from_puzzle(puzzle, GameConfig::default()),
            Err(PuzzleError::NoEmptyCell)
        );
    }

    #[test]
    fn dica_depois_do_fim_eh_recusada() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        assert_eq!(game.hint(0), Err(MoveError::GameOver));
        assert_eq!(game.hints_left(), 2);
    }

    #[test]
    fn partida_nova_comeca_pausada_e_recusa_acoes() {
        let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        let i = first_empty(&game);
        assert!(game.is_paused());
        assert_eq!(game.place(i, 1, 0), Err(MoveError::Paused));
        assert_eq!(game.toggle_note(i, 1), Err(MoveError::Paused));
        assert_eq!(game.hint(0), Err(MoveError::Paused));
        assert_eq!((game.lives_left(), game.hints_left()), (3, 2));
        assert!(game.notes(i).is_empty());
    }

    #[test]
    fn partida_pausada_no_meio_recusa_acoes() {
        let mut game = easy_game();
        game.pause(500);
        assert!(game.is_paused());
        let i = first_empty(&game);
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v, 600), Err(MoveError::Paused));
        assert_eq!(game.cell(i), CellState::Empty);
    }

    #[test]
    fn tempo_jogado_nao_conta_as_pausas() {
        let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        game.resume(1_000);
        assert_eq!(game.elapsed_ms(4_000), 3_000);
        game.pause(4_000);
        assert_eq!(game.elapsed_ms(60_000), 3_000);
        game.resume(60_000);
        assert_eq!(game.elapsed_ms(61_500), 4_500);
    }

    #[test]
    fn pausar_ou_retomar_duas_vezes_nao_muda_nada() {
        let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        game.resume(1_000);
        game.resume(5_000);
        assert_eq!(game.elapsed_ms(6_000), 5_000);
        game.pause(6_000);
        game.pause(9_000);
        assert_eq!(game.elapsed_ms(20_000), 5_000);
    }

    #[test]
    fn relogio_que_volta_nao_quebra_o_tempo() {
        let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        game.resume(10_000);
        assert_eq!(game.elapsed_ms(4_000), 0);
        game.pause(4_000);
        assert_eq!(game.elapsed_ms(4_000), 0);
    }

    #[test]
    fn vitoria_para_o_relogio() {
        let mut game = easy_game();
        for i in empties(&game) {
            let v = solution_value(&game, i);
            game.place(i, v, 7_000).unwrap();
        }
        assert_eq!(game.status(), Status::Won);
        assert_eq!(game.elapsed_ms(7_000), 7_000);
        assert_eq!(game.elapsed_ms(99_000), 7_000);
        assert!(!game.is_paused(), "partida terminada não está pausada");
        game.resume(100_000);
        assert_eq!(game.elapsed_ms(200_000), 7_000);
    }

    #[test]
    fn derrota_para_o_relogio() {
        let mut game = easy_game();
        let i = first_empty(&game);
        for now in [1_000, 2_000, 2_500] {
            let w = wrong_value(&game, i);
            game.place(i, w, now).unwrap();
        }
        assert_eq!(game.status(), Status::Lost);
        assert_eq!(game.elapsed_ms(50_000), 2_500);
    }

    #[test]
    fn dica_que_vence_para_o_relogio() {
        let mut puzzle = easy_puzzle();
        puzzle.givens = puzzle.solution.clone();
        puzzle.givens.set(0, Cell::Empty);
        let mut game = Game::from_puzzle(puzzle, GameConfig::default()).unwrap();
        game.resume(0);
        game.hint(800).unwrap();
        assert_eq!(game.status(), Status::Won);
        assert_eq!(game.elapsed_ms(10_000), 800);
    }

    /// Partida Médio seed 7 com uma jogada certa, um erro, uma anotação e uma dica, até 9_000 ms.
    fn played_game() -> Game {
        let mut game = started(Difficulty::Medio, 7);
        let open = empties(&game);
        let v = solution_value(&game, open[0]);
        game.place(open[0], v, 1_000).unwrap();
        let w = wrong_value(&game, open[1]);
        game.place(open[1], w, 2_000).unwrap();
        game.toggle_note(open[2], 4).unwrap();
        game.toggle_note(open[2], 8).unwrap();
        game.hint(3_000).unwrap();
        game
    }

    fn assert_same_game(a: &Game, b: &Game, now_a: u64, now_b: u64) {
        assert_eq!(a.puzzle(), b.puzzle());
        for i in 0..81 {
            assert_eq!(a.cell(i), b.cell(i), "célula {i}");
            assert_eq!(a.notes(i), b.notes(i), "anotações da célula {i}");
        }
        assert_eq!(
            (a.lives_left(), a.hints_left(), a.mistakes(), a.hints_used()),
            (b.lives_left(), b.hints_left(), b.mistakes(), b.hints_used()),
        );
        assert_eq!(a.status(), b.status());
        assert_eq!(a.elapsed_ms(now_a), b.elapsed_ms(now_b));
    }

    #[test]
    fn restaurar_devolve_a_mesma_partida_pausada() {
        let game = played_game();
        let restored = Game::restore(&game.save(9_000)).unwrap();
        assert!(restored.is_paused());
        assert_same_game(&restored, &game, 123_456, 9_000);
    }

    #[test]
    fn salvar_rodando_inclui_o_trecho_atual_e_nao_pausa() {
        let game = easy_game();
        assert_eq!(game.save(5_000).elapsed_ms, 5_000);
        assert!(!game.is_paused());
        assert_eq!(game.elapsed_ms(6_000), 6_000);
    }

    #[test]
    fn dica_depois_de_restaurar_cai_na_mesma_celula() {
        let mut game = started(Difficulty::Dificil, 3);
        game.hint(0).unwrap();
        let mut copy = Game::restore(&game.save(0)).unwrap();
        copy.resume(0);
        assert_eq!(game.hint(0), copy.hint(0));
    }

    #[test]
    fn partida_terminada_restaura_terminada() {
        let mut won = easy_game();
        for i in empties(&won) {
            let v = solution_value(&won, i);
            won.place(i, v, 4_000).unwrap();
        }
        let mut lost = easy_game();
        make_mistakes(&mut lost, 3);
        for game in [won, lost] {
            let restored = Game::restore(&game.save(50_000)).unwrap();
            assert!(!restored.is_paused());
            assert_same_game(&restored, &game, 99_000, 50_000);
        }
    }

    #[test]
    fn foto_guarda_seed_e_nivel_em_texto() {
        let game = Game::new(Difficulty::MuitoDificil, u64::MAX, GameConfig::default());
        let saved = game.save(0);
        assert_eq!(saved.version, SAVE_VERSION);
        assert_eq!(saved.difficulty, "MuitoDificil");
        assert_eq!(saved.seed, game.puzzle().seed.to_string());
    }

    #[test]
    fn foto_guarda_a_configuracao() {
        let config = GameConfig {
            lives: NonZeroU8::new(5).unwrap(),
            hints: 1,
        };
        let game = Game::new(Difficulty::Facil, 42, config);
        let restored = Game::restore(&game.save(0)).unwrap();
        assert_eq!(restored.config, config);
    }

    #[test]
    fn versao_nivel_ou_seed_desconhecidos_sao_recusados() {
        let good = played_game().save(0);
        let mut saved = good.clone();
        saved.version = 2;
        assert_eq!(Game::restore(&saved), Err(RestoreError::UnknownVersion(2)));
        let mut saved = good.clone();
        saved.difficulty = "Impossivel".to_string();
        assert_eq!(Game::restore(&saved), Err(RestoreError::UnknownDifficulty));
        for seed in ["abc", "-1", ""] {
            let mut saved = good.clone();
            saved.seed = seed.to_string();
            assert_eq!(Game::restore(&saved), Err(RestoreError::InvalidSeed));
        }
        let mut saved = good.clone();
        saved.givens.pop();
        assert_eq!(Game::restore(&saved), Err(RestoreError::InvalidGrid));
    }

    fn valid_save() -> SavedGame {
        played_game().save(9_000)
    }

    /// Primeira célula da foto com o estado pedido, olhando a partida restaurada.
    fn find_cell(saved: &SavedGame, pred: impl Fn(CellState) -> bool) -> usize {
        let game = Game::restore(saved).unwrap();
        (0..81)
            .find(|&i| pred(game.cell(i)))
            .expect("há célula assim")
    }

    fn set_char(line: &mut String, index: usize, ch: char) {
        line.replace_range(index..index + 1, &ch.to_string());
    }

    /// Um dígito diferente de `ch`.
    fn other_digit(ch: char) -> char {
        if ch == '9' {
            '1'
        } else {
            (ch as u8 + 1) as char
        }
    }

    fn assert_rejected(saved: &SavedGame, error: RestoreError) {
        assert_eq!(Game::restore(saved), Err(error));
    }

    #[test]
    fn foto_valida_restaura_sem_erro() {
        assert!(Game::restore(&valid_save()).is_ok());
    }

    #[test]
    fn numero_errado_travado_eh_recusado() {
        let mut saved = valid_save();
        let i = find_cell(&saved, |c| matches!(c, CellState::Placed(_)));
        let wrong = other_digit(saved.board.as_bytes()[i] as char);
        set_char(&mut saved.board, i, wrong);
        assert_rejected(&saved, RestoreError::BoardMismatch);
    }

    #[test]
    fn numero_dado_trocado_ou_apagado_eh_recusado() {
        let good = valid_save();
        let g = find_cell(&good, |c| matches!(c, CellState::Given(_)));
        let mut saved = good.clone();
        let wrong = other_digit(saved.board.as_bytes()[g] as char);
        set_char(&mut saved.board, g, wrong);
        assert_rejected(&saved, RestoreError::BoardMismatch);
        let mut saved = good.clone();
        set_char(&mut saved.board, g, '.');
        assert_rejected(&saved, RestoreError::BoardMismatch);
    }

    #[test]
    fn dica_fora_do_tabuleiro_eh_recusada() {
        let mut saved = valid_save();
        saved.hinted = vec![81];
        assert_rejected(&saved, RestoreError::InvalidHints);
        saved.hinted = vec![255];
        assert_rejected(&saved, RestoreError::InvalidHints);
    }

    #[test]
    fn dica_em_celula_vazia_ou_dada_eh_recusada() {
        let good = valid_save();
        let e = find_cell(&good, |c| c == CellState::Empty);
        let g = find_cell(&good, |c| matches!(c, CellState::Given(_)));
        for i in [e, g] {
            let mut saved = good.clone();
            saved.hinted = vec![i as u8];
            assert_rejected(&saved, RestoreError::InvalidHints);
        }
    }

    #[test]
    fn dica_repetida_eh_recusada() {
        let mut saved = valid_save();
        let h = saved.hinted[0];
        saved.hinted = vec![h, h];
        saved.hints_used = 2;
        saved.hints_left = 0;
        assert_rejected(&saved, RestoreError::InvalidHints);
    }

    #[test]
    fn dicas_que_nao_somam_sao_recusadas() {
        let good = valid_save();
        let mut saved = good.clone();
        saved.hints_left += 1;
        assert_rejected(&saved, RestoreError::InvalidHints);
        let mut saved = good.clone();
        saved.hints_used += 1;
        saved.hints_left -= 1;
        assert_rejected(&saved, RestoreError::InvalidHints);
    }

    #[test]
    fn anotacoes_com_tamanho_errado_sao_recusadas() {
        let mut saved = valid_save();
        saved.notes.pop();
        assert_rejected(&saved, RestoreError::InvalidNotes);
        saved.notes.push(String::new());
        saved.notes.push(String::new());
        assert_rejected(&saved, RestoreError::InvalidNotes);
    }

    #[test]
    fn anotacao_com_digito_invalido_eh_recusada() {
        let good = valid_save();
        let e = find_cell(&good, |c| c == CellState::Empty);
        for bad in ["0", "a", "1 2"] {
            let mut saved = good.clone();
            saved.notes[e] = bad.to_string();
            assert_rejected(&saved, RestoreError::InvalidNotes);
        }
    }

    #[test]
    fn anotacao_repetida_ou_fora_de_ordem_eh_recusada() {
        let good = valid_save();
        let e = find_cell(&good, |c| c == CellState::Empty);
        for bad in ["33", "73"] {
            let mut saved = good.clone();
            saved.notes[e] = bad.to_string();
            assert_rejected(&saved, RestoreError::InvalidNotes);
        }
    }

    #[test]
    fn anotacao_em_celula_preenchida_eh_recusada() {
        let mut saved = valid_save();
        let g = find_cell(&saved, |c| matches!(c, CellState::Given(_)));
        saved.notes[g] = "5".to_string();
        assert_rejected(&saved, RestoreError::InvalidNotes);
    }

    #[test]
    fn vida_recuperada_eh_recusada() {
        let mut saved = valid_save();
        saved.lives_left += 1;
        assert_rejected(&saved, RestoreError::InvalidLives);
    }

    #[test]
    fn erros_alem_das_vidas_sao_recusados() {
        let mut saved = valid_save();
        saved.mistakes = 4;
        saved.lives_left = 0;
        assert_rejected(&saved, RestoreError::InvalidLives);
    }

    #[test]
    fn configuracao_sem_vidas_eh_recusada() {
        let mut saved = valid_save();
        saved.lives = 0;
        assert_rejected(&saved, RestoreError::InvalidLives);
    }

    #[test]
    fn derrota_com_tabuleiro_completo_eh_recusada() {
        let mut saved = valid_save();
        saved.board = saved.solution.clone();
        saved.hinted.clear();
        saved.hints_used = 0;
        saved.hints_left = saved.hints;
        saved.notes = vec![String::new(); 81];
        saved.mistakes = u32::from(saved.lives);
        saved.lives_left = 0;
        assert_rejected(&saved, RestoreError::InconsistentStatus);
    }

    #[test]
    fn puzzle_invalido_na_foto_eh_recusado() {
        let mut saved = valid_save();
        let first = saved.solution.as_bytes()[0] as char;
        set_char(&mut saved.solution, 1, first);
        assert_rejected(&saved, RestoreError::Puzzle(PuzzleError::InvalidSolution));
    }

    #[test]
    fn reiniciar_comeca_outro_puzzle_do_mesmo_nivel_e_configuracao() {
        let config = GameConfig {
            lives: NonZeroU8::new(5).unwrap(),
            hints: 0,
        };
        let mut game = Game::new(Difficulty::Dificil, 1, config);
        game.resume(0);
        let new_game = game.restart(2).game;
        assert_eq!(new_game.puzzle().difficulty, Difficulty::Dificil);
        assert_eq!((new_game.lives_left(), new_game.hints_left()), (5, 0));
        assert_eq!(
            new_game.puzzle(),
            Game::new(Difficulty::Dificil, 2, config).puzzle()
        );
        assert_ne!(new_game.puzzle().givens, game.puzzle().givens);
        assert!(new_game.is_paused());
        assert_eq!(new_game.elapsed_ms(10_000), 0);
    }

    #[test]
    fn reiniciar_vale_depois_do_fim() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        assert_eq!(game.status(), Status::Lost);
        let new_game = game.restart(9).game;
        assert_eq!(new_game.status(), Status::Playing);
        assert_eq!(new_game.lives_left(), 3);
    }

    #[test]
    fn tempo_enorme_na_foto_nao_estoura() {
        let mut saved = valid_save();
        saved.elapsed_ms = u64::MAX;
        let mut game = Game::restore(&saved).unwrap();
        game.resume(0);
        assert_eq!(game.elapsed_ms(1_000), u64::MAX);
        game.pause(2_000);
        assert_eq!(game.save(3_000).elapsed_ms, u64::MAX);
    }
}
