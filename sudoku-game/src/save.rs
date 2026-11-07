//! Foto de uma partida: o que a casca guarda no disco para restaurar depois.
//!
//! É um tipo só de transporte (DTO), separado da `Game`, com campos simples (texto e números).
//! Assim os tipos do `sudoku-core` não precisam saber serializar, a forma salva fica legível, e a
//! `Game` só nasce de uma foto depois de `Game::restore` conferir tudo.

use crate::game::PuzzleError;

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
    pub lives_left: u8,
    pub hints_left: u8,
    pub mistakes: u32,
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
