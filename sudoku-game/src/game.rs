//! A partida: o estado de um jogo em andamento e as ações que a pessoa faz nele.

use sudoku_core::{Cell, Difficulty, GenError, Puzzle, generate, validate};

use crate::config::GameConfig;
use crate::hint;
use crate::notes::Notes;
use crate::result::{GameResult, Outcome};

// A foto é filha da partida: ela lê e monta os campos privados da `Game` sem que nenhum deles
// precise abrir para o resto do crate. Só os três itens da foto saem daqui, para o `lib.rs`.
mod save;
pub use save::{RestoreError, SAVE_VERSION, SavedGame};

/// O que a tela mostra numa célula. Não existe estado "errado": um número errado nunca fica
/// no tabuleiro, só custa uma vida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellState {
    /// Sem número: a pessoa ainda pode jogar ou anotar aqui.
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
    /// Em andamento, pausada ou não (ver `is_paused`).
    Playing,
    /// Todas as células preenchidas.
    Won,
    /// As vidas acabaram. Abandonar não leva aqui: o abandono só existe no `GameResult`.
    Lost,
}

/// O que aconteceu com uma jogada aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaceOutcome {
    /// O número estava certo e travou na célula.
    Correct,
    /// O número estava errado: custou uma vida e não ficou no tabuleiro. Volta a célula e o
    /// número tentados, para a tela mostrar o vermelho no lugar certo.
    Wrong {
        /// A célula tentada, que continua vazia.
        index: usize,
        /// O número errado que a pessoa tentou.
        value: u8,
    },
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
    /// A célula revelada, que agora está como `CellState::Hinted`.
    pub index: usize,
    /// O número da solução nessa célula.
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
    /// O número já está na linha, na coluna ou na caixa da célula, então não pode ser anotado
    /// ali. `Game::conflicts` diz em quais células ele está.
    NoteConflict,
}

/// O que `Game::restart` devolve. `#[non_exhaustive]` para poder ganhar campos sem quebrar quem
/// já lê os atuais. Fora do crate, só dá para ler os campos, nunca montar o valor.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Restart {
    /// A partida nova, pausada: "começar" é o primeiro `resume`.
    pub game: Game,
    /// O resultado do abandono, se a partida antiga não tinha terminado: reiniciar no meio conta
    /// como derrota. `None` se ela já tinha terminado, porque aí ela já tem o seu resultado.
    pub abandoned: Option<GameResult>,
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
    ///
    /// Se a partida não tinha terminado, ela conta como abandonada, e o `Restart` traz o
    /// resultado do abandono, com o tempo jogado até `now` e a data `finished_at_ms` (relógio de
    /// parede, ver `GameResult`).
    pub fn restart(&self, seed: u64, now: u64, finished_at_ms: u64) -> Restart {
        Restart {
            game: Game::new(self.puzzle.difficulty, seed, self.config),
            abandoned: self.abandoned_result(now, finished_at_ms),
        }
    }

    /// Larga a partida sem começar outra (por exemplo, voltar ao menu e escolher outro nível).
    ///
    /// Se ela não tinha terminado, conta como derrota por abandono, mesmo pausada ou nunca
    /// começada, e o resultado vem com o tempo jogado até `now`. Se já tinha terminado, devolve
    /// `None`: ela já tem o seu resultado (`result`). Recebe `self`, e não `&self`, para que o
    /// compilador recuse qualquer jogada numa partida abandonada.
    pub fn abandon(self, now: u64, finished_at_ms: u64) -> Option<GameResult> {
        self.abandoned_result(now, finished_at_ms)
    }

    /// O resultado de abandonar agora, ou `None` se a partida já terminou.
    fn abandoned_result(&self, now: u64, finished_at_ms: u64) -> Option<GameResult> {
        (self.status == Status::Playing)
            .then(|| self.result_with(Outcome::Abandoned, self.elapsed_ms(now), finished_at_ms))
    }

    /// O puzzle desta partida, com enunciado, solução, nível e seed.
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

    /// Vidas que ainda restam. Cada número errado custa uma; em zero, a partida está perdida.
    pub fn lives_left(&self) -> u8 {
        self.lives_left
    }

    /// Dicas que ainda podem ser pedidas.
    pub fn hints_left(&self) -> u8 {
        self.hints_left
    }

    /// Números errados colocados até agora.
    pub fn mistakes(&self) -> u32 {
        self.mistakes
    }

    /// Dicas pedidas até agora. Vai para o `GameResult` quando a partida termina.
    pub fn hints_used(&self) -> u8 {
        self.hints_used
    }

    /// Em que pé está a partida.
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
    /// Certo: o número trava, a anotação dele some da linha, da coluna e da caixa (ver `lock`), e
    /// a partida termina em vitória se era a última célula vazia. Errado: perde uma vida, a célula continua vazia, e a partida termina em derrota se as
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
    /// muda nas outras células. Recusa as mesmas coisas que `place`, na mesma ordem, e depois
    /// recusa com `NoteConflict` **ligar** um número que já está na linha, na coluna ou na caixa.
    /// Desligar é sempre aceito: uma foto antiga pode trazer anotação em conflito, e a pessoa
    /// precisa conseguir apagá-la.
    pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), MoveError> {
        self.check_empty_cell(index, value)?;
        if !self.notes[index].contains(value) && !self.conflicts(index, value).is_empty() {
            return Err(MoveError::NoteConflict);
        }
        self.notes[index].toggle(value);
        Ok(())
    }

    /// As células com o número `value` que enxergam `index` (mesma linha, coluna ou caixa), em
    /// ordem crescente. Vazio quando `value` pode ser anotado ali. É o que a tela pisca quando
    /// `toggle_note` recusa com `NoteConflict`.
    ///
    /// Entra em pânico fora de 0..81, como `cell`. Sem isso, o 81 seria lido como a coluna 0 da
    /// linha 9 e devolveria células de verdade, uma resposta errada em silêncio.
    pub fn conflicts(&self, index: usize, value: u8) -> Vec<usize> {
        assert!(index < 81, "célula {index} fora do tabuleiro");
        (0..81)
            .filter(|&i| sees(index, i) && self.cells[i].value() == Some(value))
            .collect()
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

    /// O resultado de uma partida **terminada** (vitória ou derrota por vidas), ou `None` se ela
    /// ainda está em andamento. `finished_at_ms` é a data do registro, pelo relógio de parede da
    /// casca (ver `GameResult`).
    ///
    /// É repetível: pedir de novo dá o mesmo resultado (só a data muda, se mudar o argumento), e
    /// pedir não muda a partida. A casca registra o resultado quando `place` ou `hint` terminam a
    /// partida, antes de salvar a foto; por isso não precisa registrar de novo ao restaurar uma
    /// partida terminada. O tempo é o congelado no fim, e não cresce depois.
    pub fn result(&self, finished_at_ms: u64) -> Option<GameResult> {
        let outcome = match self.status {
            Status::Playing => return None,
            Status::Won => Outcome::Won,
            Status::Lost => Outcome::OutOfLives,
        };
        Some(self.result_with(outcome, self.elapsed_ms, finished_at_ms))
    }

    /// Monta o resultado com o desfecho e o tempo dados; o resto vem da partida.
    pub(crate) fn result_with(
        &self,
        outcome: Outcome,
        elapsed_ms: u64,
        finished_at_ms: u64,
    ) -> GameResult {
        GameResult {
            difficulty: self.puzzle.difficulty,
            seed: self.puzzle.seed,
            outcome,
            elapsed_ms,
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            config: self.config,
            finished_at_ms,
        }
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

    /// Trava um número certo na célula, apaga todas as anotações dela e apaga a anotação desse
    /// número nas células que a enxergam. Um número certo nunca mais cabe na linha, na coluna nem
    /// na caixa dele, então a anotação ali já estava errada; apagar poupa a pessoa de fazer isso à
    /// mão. Vale para jogada certa e para dica, que também é número certo.
    fn lock(&mut self, index: usize, state: CellState) {
        self.cells[index] = state;
        self.notes[index] = Notes::default();
        if let Some(value) = state.value() {
            for i in (0..81).filter(|&i| sees(index, i)) {
                self.notes[i].remove(value);
            }
        }
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

/// `true` se as células `a` e `b` são diferentes e dividem linha, coluna ou caixa.
fn sees(a: usize, b: usize) -> bool {
    let (row, col) = (|i: usize| i / 9, |i: usize| i % 9);
    let block = |i: usize| (row(i) / 3, col(i) / 3);
    a != b && (row(a) == row(b) || col(a) == col(b) || block(a) == block(b))
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
    pub(super) fn started(difficulty: Difficulty, seed: u64) -> Game {
        let mut game = Game::new(difficulty, seed, GameConfig::default());
        game.resume(0);
        game
    }

    pub(super) fn easy_game() -> Game {
        started(Difficulty::Facil, 42)
    }

    pub(super) fn solution_value(game: &Game, index: usize) -> u8 {
        match game.puzzle().solution.get(index) {
            Cell::Filled(v) => v,
            Cell::Empty => panic!("a solução não tem células vazias"),
        }
    }

    pub(super) fn wrong_value(game: &Game, index: usize) -> u8 {
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

    pub(super) fn make_mistakes(game: &mut Game, count: usize) {
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

    pub(super) fn empties(game: &Game) -> Vec<usize> {
        (0..81)
            .filter(|&i| game.cell(i) == CellState::Empty)
            .collect()
    }

    /// Os dois menores números que dá para anotar na célula `index`, sem conflito.
    pub(super) fn two_notable(game: &Game, index: usize) -> (u8, u8) {
        let mut free = (1..=9).filter(|&v| can_note(game, index, v));
        let first = free.next().expect("há número livre para anotar");
        let second = free.next().expect("há dois números livres para anotar");
        (first, second)
    }

    #[test]
    fn anotacao_liga_e_desliga() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let (a, b) = two_notable(&game, i);
        assert_eq!(game.toggle_note(i, a), Ok(()));
        assert_eq!(game.toggle_note(i, b), Ok(()));
        assert_eq!(game.notes(i).values(), vec![a, b]);
        assert_eq!(game.toggle_note(i, a), Ok(()));
        assert_eq!(game.notes(i).values(), vec![b]);
    }

    /// Linha, coluna e caixa calculadas aqui, à parte do `sees` que está sendo testado.
    fn same_house(a: usize, b: usize) -> bool {
        let (row, col) = (|i: usize| i / 9, |i: usize| i % 9);
        let block = |i: usize| (row(i) / 3, col(i) / 3);
        a != b && (row(a) == row(b) || col(a) == col(b) || block(a) == block(b))
    }

    /// `true` se nenhuma célula com número que enxerga `index` tem `value`: dá para anotar.
    fn can_note(game: &Game, index: usize, value: u8) -> bool {
        (0..81).all(|j| !same_house(index, j) || game.cell(j).value() != Some(value))
    }

    /// Uma célula vazia `a`, o valor certo `v` dela e células vazias onde `v` pode ser anotado:
    /// uma na linha, uma na coluna, uma só na caixa e uma longe, que não enxerga `a`.
    struct Spread {
        a: usize,
        v: u8,
        row: usize,
        col: usize,
        block: usize,
        far: usize,
    }

    fn spread(game: &Game) -> Spread {
        let empty = empties(game);
        empty
            .iter()
            .find_map(|&a| {
                let v = solution_value(game, a);
                let pick = |keep: &dyn Fn(usize) -> bool| {
                    empty
                        .iter()
                        .copied()
                        .find(|&i| i != a && keep(i) && can_note(game, i, v))
                };
                Some(Spread {
                    a,
                    v,
                    row: pick(&|i| i / 9 == a / 9)?,
                    col: pick(&|i| i % 9 == a % 9)?,
                    block: pick(&|i| same_house(a, i) && i / 9 != a / 9 && i % 9 != a % 9)?,
                    far: pick(&|i| !same_house(a, i))?,
                })
            })
            .expect("a seed 42 do Fácil tem uma célula com vizinhos vazios em toda direção")
    }

    #[test]
    fn numero_certo_limpa_a_anotacao_dos_vizinhos() {
        let mut game = easy_game();
        let s = spread(&game);
        for i in [s.a, s.row, s.col, s.block, s.far] {
            game.toggle_note(i, s.v).unwrap();
        }
        game.place(s.a, s.v, 0).unwrap();
        assert!(game.notes(s.a).is_empty());
        assert!(!game.notes(s.row).contains(s.v), "vizinho de linha");
        assert!(!game.notes(s.col).contains(s.v), "vizinho de coluna");
        assert!(!game.notes(s.block).contains(s.v), "vizinho de caixa");
        assert_eq!(
            game.notes(s.far).values(),
            vec![s.v],
            "longe fica como estava"
        );
    }

    #[test]
    fn limpeza_so_apaga_o_numero_colocado() {
        let mut game = easy_game();
        let s = spread(&game);
        let other = (1..=9)
            .find(|&u| u != s.v && can_note(&game, s.row, u))
            .expect("o vizinho de linha aceita outro número");
        game.toggle_note(s.row, s.v).unwrap();
        game.toggle_note(s.row, other).unwrap();
        game.place(s.a, s.v, 0).unwrap();
        assert_eq!(game.notes(s.row).values(), vec![other]);
    }

    #[test]
    fn conflicts_acha_o_numero_nos_vizinhos() {
        let game = easy_game();
        let i = first_empty(&game);
        let clashing = (1..=9)
            .find(|&v| !can_note(&game, i, v))
            .expect("a primeira vazia tem algum número nos vizinhos");
        let expected: Vec<usize> = (0..81)
            .filter(|&j| same_house(i, j) && game.cell(j).value() == Some(clashing))
            .collect();
        assert!(!expected.is_empty());
        assert_eq!(game.conflicts(i, clashing), expected);
        assert_eq!(
            game.conflicts(i, solution_value(&game, i)),
            Vec::<usize>::new()
        );
    }

    #[test]
    #[should_panic(expected = "fora do tabuleiro")]
    fn conflicts_fora_do_tabuleiro_entra_em_panico() {
        easy_game().conflicts(81, 1);
    }

    #[test]
    fn anotacao_em_conflito_eh_recusada() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let clashing = (1..=9)
            .find(|&v| !can_note(&game, i, v))
            .expect("a primeira vazia tem algum número nos vizinhos");
        assert_eq!(game.toggle_note(i, clashing), Err(MoveError::NoteConflict));
        assert!(game.notes(i).is_empty());
    }

    #[test]
    fn apagar_anotacao_em_conflito_eh_permitido() {
        // Uma foto de antes desta regra pode trazer anotação em conflito: ela abre, e dá para
        // apagar a anotação.
        let game = easy_game();
        let i = first_empty(&game);
        let clashing = (1..=9)
            .find(|&v| !can_note(&game, i, v))
            .expect("a primeira vazia tem algum número nos vizinhos");
        let mut saved = game.save(0);
        saved.notes[i] = clashing.to_string();
        let mut old = Game::restore(&saved).expect("foto antiga continua abrindo");
        old.resume(0);
        assert_eq!(old.notes(i).values(), vec![clashing]);
        assert_eq!(old.toggle_note(i, clashing), Ok(()));
        assert!(old.notes(i).is_empty());
    }

    #[test]
    fn dica_limpa_a_anotacao_dos_vizinhos() {
        // A primeira dica da seed 42 do Fácil revela o 9 na célula 40 (golden logo abaixo).
        let mut game = easy_game();
        let neighbour = empties(&game)
            .into_iter()
            .find(|&i| same_house(40, i) && can_note(&game, i, 9))
            .expect("a célula 40 tem um vizinho vazio que aceita o 9");
        game.toggle_note(neighbour, 9).unwrap();
        assert_eq!(game.hint(0).map(|h| (h.index, h.value)), Ok((40, 9)));
        assert!(!game.notes(neighbour).contains(9));
    }

    #[test]
    fn numero_errado_mantem_as_anotacoes() {
        let mut game = easy_game();
        let i = first_empty(&game);
        let (a, b) = two_notable(&game, i);
        game.toggle_note(i, a).unwrap();
        game.toggle_note(i, b).unwrap();
        let w = wrong_value(&game, i);
        game.place(i, w, 0).unwrap();
        assert_eq!(game.notes(i).values(), vec![a, b]);
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
        // A primeira dica da seed 42 do Fácil revela a célula 40.
        let mut game = easy_game();
        let (a, b) = two_notable(&game, 40);
        game.toggle_note(40, a).unwrap();
        game.toggle_note(40, b).unwrap();
        let far = empties(&game)
            .into_iter()
            .find(|&i| i != 40 && !same_house(40, i))
            .expect("há célula vazia longe da 40");
        let (c, _) = two_notable(&game, far);
        game.toggle_note(far, c).unwrap();
        assert_eq!(game.hint(0).map(|h| h.index), Ok(40));
        assert!(game.notes(40).is_empty());
        assert_eq!(game.notes(far).values(), vec![c]);
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
    #[test]
    fn reiniciar_comeca_outro_puzzle_do_mesmo_nivel_e_configuracao() {
        let config = GameConfig {
            lives: NonZeroU8::new(5).unwrap(),
            hints: 0,
        };
        let mut game = Game::new(Difficulty::Dificil, 1, config);
        game.resume(0);
        let new_game = game.restart(2, 0, DATE).game;
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
        let restart = game.restart(9, 0, DATE);
        assert_eq!(
            restart.abandoned, None,
            "a partida terminada já tem o seu resultado"
        );
        let new_game = restart.game;
        assert_eq!(new_game.status(), Status::Playing);
        assert_eq!(new_game.lives_left(), 3);
    }

    #[test]
    fn abandonar_no_meio_vira_derrota_por_abandono() {
        let mut game = easy_game();
        make_mistakes(&mut game, 1);
        let r = game.abandon(2_500, DATE).unwrap();
        assert_eq!(r.outcome, Outcome::Abandoned);
        assert_eq!((r.elapsed_ms, r.mistakes), (2_500, 1));
    }

    #[test]
    fn abandonar_partida_que_nunca_comecou_conta() {
        let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
        let r = game.abandon(9_000, DATE).unwrap();
        assert_eq!((r.outcome, r.elapsed_ms), (Outcome::Abandoned, 0));
    }

    #[test]
    fn abandonar_partida_terminada_nao_da_outro_resultado() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        assert_eq!(game.abandon(0, DATE), None);
    }

    #[test]
    fn reiniciar_no_meio_devolve_o_abandono() {
        let mut game = easy_game();
        game.pause(1_200);
        let restart = game.restart(2, 5_000, DATE);
        let r = restart.abandoned.unwrap();
        assert_eq!((r.outcome, r.elapsed_ms), (Outcome::Abandoned, 1_200));
        assert_eq!(restart.game.puzzle().seed, 2);
    }

    /// Data de relógio de parede (13/06/2026 12:00 UTC), em milissegundos desde a época Unix.
    const DATE: u64 = 1_781_352_000_000;

    #[test]
    fn partida_em_andamento_nao_tem_resultado() {
        let game = easy_game();
        assert_eq!(game.result(DATE), None);
    }

    #[test]
    fn derrota_por_vidas_vira_resultado() {
        let mut game = easy_game();
        make_mistakes(&mut game, 3);
        let r = game.result(DATE).unwrap();
        assert_eq!(r.outcome, Outcome::OutOfLives);
        assert_eq!(
            (r.difficulty, r.seed),
            (Difficulty::Facil, game.puzzle().seed)
        );
        assert_eq!((r.mistakes, r.hints_used), (3, 0));
        assert_eq!(r.config, GameConfig::default());
        assert_eq!(r.finished_at_ms, DATE);
    }

    #[test]
    fn vitoria_vira_resultado_com_o_tempo_congelado() {
        let mut game = easy_game();
        game.hint(0).unwrap();
        while let Some(i) = (0..81).find(|&i| game.cell(i) == CellState::Empty) {
            let v = solution_value(&game, i);
            game.place(i, v, 4_000).unwrap();
        }
        let r = game.result(DATE).unwrap();
        assert_eq!(r.outcome, Outcome::Won);
        assert_eq!((r.elapsed_ms, r.hints_used), (4_000, 1));
        // Pedir de novo, com outra data, só muda a data: nada na partida mudou.
        assert_eq!(game.result(DATE + 1).unwrap().elapsed_ms, 4_000);
        assert_eq!(game.result(DATE), Some(r));
    }
}
