//! Resultado de uma partida: o registro que a casca guarda no histórico.
//!
//! Toda partida que a pessoa começa termina num resultado, inclusive a abandonada, para que fugir
//! de uma partida ruim não proteja a estatística (spec do Game, seção 12).

use std::num::NonZeroU8;

use sudoku_core::Difficulty;

use crate::config::GameConfig;
use crate::save::{difficulty_name, parse_difficulty};

/// Versão do formato do resultado salvo. Cada registro leva a sua: um histórico guarda meses de
/// partidas, gravadas por versões diferentes do app, e um formato novo precisa conviver com os
/// antigos na mesma lista.
pub const RESULT_VERSION: u32 = 1;

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

/// Um resultado em campos simples, para a casca guardar no histórico. Mesmo padrão da foto
/// (`SavedGame`): um tipo só de transporte, sem garantia nenhuma, que só vira `GameResult` por
/// `GameResult::restore`. Com a feature `serde`, ganha `Serialize` e `Deserialize`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SavedResult {
    /// Sempre `RESULT_VERSION` num registro novo.
    pub version: u32,
    /// `"Facil"`, `"Medio"`, `"Dificil"` ou `"MuitoDificil"`.
    pub difficulty: String,
    /// A seed em decimal. Texto, e não número, pelo limite de 2^53 do JSON em JavaScript.
    pub seed: String,
    /// `"Won"`, `"OutOfLives"` ou `"Abandoned"`.
    pub outcome: String,
    pub elapsed_ms: u64,
    pub mistakes: u32,
    pub hints_used: u8,
    /// Vidas da configuração da partida.
    pub lives: u8,
    /// Dicas da configuração da partida.
    pub hints: u8,
    /// A data, em milissegundos desde a época Unix (UTC), em decimal. Texto pelo mesmo motivo da
    /// seed.
    pub finished_at: String,
}

/// Por que um registro foi recusado por `GameResult::restore`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultError {
    /// Versão de formato que esta versão do jogo não sabe ler.
    UnknownVersion(u32),
    UnknownDifficulty,
    /// Seed que não é um `u64` em decimal.
    InvalidSeed,
    UnknownOutcome,
    /// Data que não é um `u64` em decimal.
    InvalidDate,
    /// Zero vidas na configuração: nenhuma partida começa assim.
    InvalidLives,
}

impl GameResult {
    /// O registro em campos simples, pronto para a casca guardar.
    pub fn save(&self) -> SavedResult {
        SavedResult {
            version: RESULT_VERSION,
            difficulty: difficulty_name(self.difficulty).to_string(),
            seed: self.seed.to_string(),
            outcome: outcome_name(self.outcome).to_string(),
            elapsed_ms: self.elapsed_ms,
            mistakes: self.mistakes,
            hints_used: self.hints_used,
            lives: self.config.lives.get(),
            hints: self.config.hints,
            finished_at: self.finished_at_ms.to_string(),
        }
    }

    /// Lê um registro salvo. Confere a versão e se cada campo tem a forma certa, mas **não** a
    /// coerência entre eles (erros × vidas, por exemplo): um histórico editado à mão só engana a
    /// própria pessoa, e um ranking online refaria a partida pela seed (spec, seção 12.4).
    pub fn restore(saved: &SavedResult) -> Result<GameResult, ResultError> {
        if saved.version != RESULT_VERSION {
            return Err(ResultError::UnknownVersion(saved.version));
        }
        let difficulty =
            parse_difficulty(&saved.difficulty).ok_or(ResultError::UnknownDifficulty)?;
        let seed = saved.seed.parse().map_err(|_| ResultError::InvalidSeed)?;
        let outcome = parse_outcome(&saved.outcome).ok_or(ResultError::UnknownOutcome)?;
        let lives = NonZeroU8::new(saved.lives).ok_or(ResultError::InvalidLives)?;
        let finished_at_ms = saved
            .finished_at
            .parse()
            .map_err(|_| ResultError::InvalidDate)?;
        Ok(GameResult {
            difficulty,
            seed,
            outcome,
            elapsed_ms: saved.elapsed_ms,
            mistakes: saved.mistakes,
            hints_used: saved.hints_used,
            config: GameConfig {
                lives,
                hints: saved.hints,
            },
            finished_at_ms,
        })
    }
}

/// Nome do desfecho no registro. `match` exaustivo: um desfecho novo quebra a compilação aqui.
fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Won => "Won",
        Outcome::OutOfLives => "OutOfLives",
        Outcome::Abandoned => "Abandoned",
    }
}

/// O contrário de `outcome_name`.
fn parse_outcome(name: &str) -> Option<Outcome> {
    match name {
        "Won" => Some(Outcome::Won),
        "OutOfLives" => Some(Outcome::OutOfLives),
        "Abandoned" => Some(Outcome::Abandoned),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATE: u64 = 1_781_352_000_000;

    fn sample() -> GameResult {
        GameResult {
            difficulty: Difficulty::Medio,
            seed: u64::MAX,
            outcome: Outcome::Abandoned,
            elapsed_ms: 1_234,
            mistakes: 2,
            hints_used: 1,
            config: GameConfig::default(),
            finished_at_ms: DATE,
        }
    }

    fn valid_saved() -> SavedResult {
        sample().save()
    }

    #[test]
    fn resultado_vai_e_volta() {
        let r = sample();
        assert_eq!(GameResult::restore(&r.save()), Ok(r));
    }

    #[test]
    fn todo_desfecho_vai_e_volta() {
        for outcome in [Outcome::Won, Outcome::OutOfLives, Outcome::Abandoned] {
            let r = GameResult {
                outcome,
                ..sample()
            };
            assert_eq!(GameResult::restore(&r.save()), Ok(r));
        }
    }

    #[test]
    fn registro_estranho_eh_recusado() {
        // Cada caso estraga um campo de um registro válido e diz qual erro esperar.
        type Break = fn(&mut SavedResult);
        let cases: [(Break, ResultError); 6] = [
            (|s| s.version = 2, ResultError::UnknownVersion(2)),
            (
                |s| s.difficulty = "medio".into(),
                ResultError::UnknownDifficulty,
            ),
            (|s| s.seed = "-1".into(), ResultError::InvalidSeed),
            (|s| s.outcome = "won".into(), ResultError::UnknownOutcome),
            (|s| s.finished_at = "abc".into(), ResultError::InvalidDate),
            (|s| s.lives = 0, ResultError::InvalidLives),
        ];
        for (break_it, expected) in cases {
            let mut saved = valid_saved();
            break_it(&mut saved);
            assert_eq!(GameResult::restore(&saved), Err(expected));
        }
    }
}
