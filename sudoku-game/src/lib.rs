//! sudoku-game — regras de uma partida de Sudoku: jogadas, vidas, dicas, anotações, tempo,
//! salvamento (`SavedGame`, serializável com a feature `serde`) e reinício.
//! Biblioteca pura, como o sudoku-core: sem relógio, sem IO, sem rede, sem UI. A casca (app,
//! navegador, servidor) informa o que acontece e guarda o que precisa ser guardado.

mod config;
mod game;
mod hint;
mod notes;
mod result;
mod save;
mod stats;

pub use config::GameConfig;
pub use game::{
    CellState, Game, HintOutcome, MoveError, PlaceOutcome, PuzzleError, Restart, Status,
};
pub use notes::Notes;
pub use result::{GameResult, Outcome, RESULT_VERSION, ResultError, SavedResult};
pub use save::{RestoreError, SAVE_VERSION, SavedGame};
pub use stats::{LevelStats, level_stats, personal_ranking};

// A casca só precisa depender deste crate: os tipos do core que aparecem na API vêm junto
// (`Puzzle` traz duas `Grid`, e ler uma célula da `Grid` devolve um `Cell`).
pub use sudoku_core::{Cell, Difficulty, Grid, Puzzle};
