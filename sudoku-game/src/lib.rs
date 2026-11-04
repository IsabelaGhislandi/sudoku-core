//! sudoku-game — regras de uma partida de Sudoku: jogadas, vidas, dicas e anotações.
//! Biblioteca pura, como o sudoku-core: sem relógio, sem IO, sem rede, sem UI. A casca (app,
//! navegador, servidor) informa o que acontece e guarda o que precisa ser guardado.

mod config;
mod game;
mod hint;
mod notes;

pub use config::GameConfig;
pub use game::{CellState, Game, HintOutcome, MoveError, PlaceOutcome, PuzzleError, Status};
pub use notes::Notes;

// A casca só precisa depender deste crate: os tipos do core que aparecem na API vêm junto
// (`Puzzle` traz duas `Grid`, e ler uma célula da `Grid` devolve um `Cell`).
pub use sudoku_core::{Cell, Difficulty, Grid, Puzzle};
