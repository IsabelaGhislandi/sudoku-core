//! sudoku-game — regras de uma partida de Sudoku: jogadas, vidas, dicas e anotações.
//! Biblioteca pura, como o sudoku-core: sem relógio, sem IO, sem rede, sem UI. A casca (app,
//! navegador, servidor) informa o que acontece e guarda o que precisa ser guardado.

mod config;
mod game;

pub use config::GameConfig;
pub use game::{CellState, Game, MoveError, PlaceOutcome, Status};

// A casca só precisa depender deste crate: os tipos do core que aparecem na API vêm junto.
pub use sudoku_core::{Difficulty, Puzzle};
