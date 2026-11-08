//! Foto de uma partida: o que a casca guarda no disco para restaurar depois.
//!
//! É um tipo só de transporte (DTO), separado da `Game`, com campos simples (texto e números).
//! Assim os tipos do `sudoku-core` não precisam saber serializar, a forma salva fica legível, e a
//! `Game` só nasce de uma foto depois de `Game::restore` conferir tudo.
//!
//! Este module mora dentro de `game` (é filho dele) porque escrever e ler a foto exige mexer nos
//! campos privados da `Game`. Em Rust, um item privado é visível no module onde nasce e em todos
//! os descendentes: como filho, a foto enxerga esses campos sem que nenhum precise virar
//! `pub(crate)`. Todo o formato (versão, escrita, leitura e conferência) fica neste arquivo.

use std::num::NonZeroU8;

use sudoku_core::{Cell, Grid, Puzzle};

use super::{CellState, Game, PuzzleError, Status};
use crate::config::GameConfig;
use crate::names::{difficulty_name, parse_difficulty};
use crate::notes::Notes;

/// Versão do formato da foto. Uma partida salva por uma versão antiga do app precisa continuar
/// abrindo: mudar a forma da foto é versão nova, com leitura da antiga.
pub const SAVE_VERSION: u32 = 1;

/// Foto de uma partida, em campos simples para virar texto legível. Os campos são públicos
/// porque a foto vem de fora (disco, rede) e não carrega garantia nenhuma: quem garante é
/// `Game::restore`.
///
/// Com a feature `serde`, a foto ganha `Serialize` e `Deserialize`, e a casca escolhe o formato
/// (o app usa JSON). Sem a feature, o crate não depende do `serde`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SavedGame {
    /// Sempre `SAVE_VERSION` numa foto nova.
    pub version: u32,
    /// `"Facil"`, `"Medio"`, `"Dificil"` ou `"MuitoDificil"`.
    pub difficulty: String,
    /// A seed em decimal. Texto, e não número, porque JSON lida por JavaScript perde precisão
    /// acima de 2^53, e a seed é um `u64` qualquer.
    pub seed: String,
    /// Enunciado, no formato de `Grid::to_line`.
    pub givens: String,
    /// Solução, no formato de `Grid::to_line`.
    pub solution: String,
    /// Tabuleiro atual (dados, certos e dicas), no formato de `Grid::to_line`.
    pub board: String,
    /// Índices das células reveladas por dica, em ordem crescente.
    pub hinted: Vec<u8>,
    /// 81 entradas, uma por célula: `""` ou os dígitos anotados em ordem crescente, como `"37"`.
    pub notes: Vec<String>,
    /// Vidas da configuração da partida.
    pub lives: u8,
    /// Dicas da configuração da partida.
    pub hints: u8,
    /// Vidas que restavam, como em `Game::lives_left`.
    pub lives_left: u8,
    /// Dicas que restavam, como em `Game::hints_left`.
    pub hints_left: u8,
    /// Números errados até o salvamento, como em `Game::mistakes`.
    pub mistakes: u32,
    /// Dicas pedidas até o salvamento, como em `Game::hints_used`.
    pub hints_used: u8,
    /// Tempo jogado até o instante do salvamento, em milissegundos.
    pub elapsed_ms: u64,
}

/// Por que uma foto foi recusada por `Game::restore`. Uma foto recusada nunca vira partida.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreError {
    /// Versão de formato que esta versão do jogo não sabe ler.
    UnknownVersion(u32),
    /// Nível com nome desconhecido.
    UnknownDifficulty,
    /// Seed que não é um `u64` em decimal.
    InvalidSeed,
    /// Enunciado, solução ou tabuleiro que não é uma linha de 81 células.
    InvalidGrid,
    /// O puzzle da foto seria recusado por `Game::from_puzzle`.
    Puzzle(PuzzleError),
    /// O tabuleiro não bate com o enunciado ou com a solução.
    BoardMismatch,
    /// As dicas não batem: índice inválido, célula errada ou contagem que não fecha.
    InvalidHints,
    /// As anotações não batem: tamanho, dígito inválido ou anotação em célula com número.
    InvalidNotes,
    /// As vidas não batem com a configuração e os erros.
    InvalidLives,
    /// Derrota com o tabuleiro completo: estado impossível.
    InconsistentStatus,
}

/// Escrever e ler a foto. Os métodos são da `Game`, mas moram aqui, junto do formato: mudar a foto
/// mexe só neste arquivo.
impl Game {
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
        let difficulty =
            parse_difficulty(&saved.difficulty).ok_or(RestoreError::UnknownDifficulty)?;
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
    use crate::game::tests::{
        easy_game, empties, make_mistakes, solution_value, started, wrong_value,
    };
    use sudoku_core::Difficulty;

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
