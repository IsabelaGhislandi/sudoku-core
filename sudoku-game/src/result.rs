//! Resultado de uma partida: o registro que a casca guarda no histórico.
//!
//! Toda partida que a pessoa começa termina num resultado, inclusive a abandonada, para que fugir
//! de uma partida ruim não proteja a estatística (spec do Game, seção 12).

use sudoku_core::Difficulty;

use crate::config::GameConfig;

/// Como a partida terminou. Abandono é derrota, mas fica separado de "vidas esgotadas" para o
/// histórico poder mostrar a diferença.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Won,
    OutOfLives,
    /// Reiniciada ou largada antes de terminar.
    Abandoned,
}

/// O registro de uma partida terminada ou abandonada.
///
/// `#[non_exhaustive]`: de fora do crate, só dá para ler os campos. Um resultado vem da `Game`
/// (`result`, `abandon`, `restart`) ou de um registro salvo, nunca montado à mão pela casca.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GameResult {
    pub difficulty: Difficulty,
    /// A seed de fato usada (`Puzzle.seed`). Permite a um ranking online comparar quem jogou o
    /// mesmo puzzle, e ao servidor refazer a partida.
    pub seed: u64,
    pub outcome: Outcome,
    /// Tempo jogado, sem as pausas, em milissegundos.
    pub elapsed_ms: u64,
    pub mistakes: u32,
    pub hints_used: u8,
    /// Vidas e dicas com que a partida foi jogada: 3 vidas e 1 vida não são a mesma partida.
    pub config: GameConfig,
    /// Quando terminou, pelo **relógio de parede** da casca: milissegundos desde a época Unix,
    /// em UTC. Não é o `now` monotônico das jogadas, que mede duração e não tem calendário.
    pub finished_at_ms: u64,
}
