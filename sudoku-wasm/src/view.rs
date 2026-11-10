//! O retrato da partida que a tela desenha.

use crate::convert::difficulty_name;
use serde::Serialize;
use sudoku_game::{CellState, Game, Status};

/// Tudo o que a tela precisa para desenhar a partida num instante.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    /// O nível, pelo nome do Rust (`"Medio"`).
    pub difficulty: &'static str,
    /// `"playing"`, `"won"` ou `"lost"`.
    pub status: &'static str,
    /// Se o relógio está parado.
    pub paused: bool,
    /// Tempo jogado, sem as pausas.
    pub elapsed_ms: u64,
    /// Vidas que restam.
    pub lives_left: u8,
    /// Dicas que restam.
    pub hints_left: u8,
    /// As 81 células, linha por linha.
    pub cells: Vec<CellView>,
}

/// Uma célula do retrato.
#[derive(Debug, Serialize)]
pub struct CellView {
    /// `"empty"`, `"given"`, `"placed"` ou `"hinted"`.
    pub kind: &'static str,
    /// O número, se houver.
    pub value: Option<u8>,
    /// As anotações, em ordem crescente.
    pub notes: Vec<u8>,
}

impl CellView {
    fn from_game(game: &Game, index: usize) -> CellView {
        let state = game.cell(index);
        CellView {
            kind: match state {
                CellState::Empty => "empty",
                CellState::Given(_) => "given",
                CellState::Placed(_) => "placed",
                CellState::Hinted(_) => "hinted",
            },
            value: state.value(),
            notes: game.notes(index).values(),
        }
    }
}

impl GameView {
    /// Tira o retrato da partida no instante `now`.
    pub fn from_game(game: &Game, now: u64) -> GameView {
        GameView {
            difficulty: difficulty_name(game.puzzle().difficulty),
            status: match game.status() {
                Status::Playing => "playing",
                Status::Won => "won",
                Status::Lost => "lost",
            },
            paused: game.is_paused(),
            elapsed_ms: game.elapsed_ms(now),
            lives_left: game.lives_left(),
            hints_left: game.hints_left(),
            cells: (0..81).map(|i| CellView::from_game(game, i)).collect(),
        }
    }
}
