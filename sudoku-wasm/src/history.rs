//! O histórico que a tela lista: os resultados guardados pela casca, conferidos e ordenados.

use crate::convert::difficulty_name;
use serde::Serialize;
use serde_json::Value;
use std::cmp::Reverse;
use sudoku_game::{GameResult, Outcome, SavedResult};

/// Uma linha do histórico, como a tela recebe.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryEntry {
    difficulty: &'static str,
    outcome: &'static str,
    elapsed_ms: u64,
    finished_at_ms: u64,
}

/// Lê o histórico guardado (um array JSON de `SavedResult`) e devolve as entradas que a tela
/// mostra, em JSON, da mais recente para a mais antiga.
///
/// O histórico vem de fora e pode ter registros corrompidos, de uma versão desconhecida ou
/// editados à mão. Cada registro é conferido sozinho, pelo `GameResult::restore`: um registro ruim
/// some, e os bons continuam aparecendo. Um valor que nem é um array vira um histórico vazio.
pub fn history_entries(json: &str) -> String {
    let records: Vec<Value> = serde_json::from_str(json).unwrap_or_default();
    let mut entries: Vec<HistoryEntry> = records
        .into_iter()
        .filter_map(|record| serde_json::from_value::<SavedResult>(record).ok())
        .filter_map(|saved| GameResult::restore(&saved).ok())
        .map(|result| HistoryEntry {
            difficulty: difficulty_name(result.difficulty),
            outcome: outcome_name(result.outcome),
            elapsed_ms: result.elapsed_ms,
            finished_at_ms: result.finished_at_ms,
        })
        .collect();
    // Estável: dois resultados com a mesma data ficam na ordem em que foram gravados.
    entries.sort_by_key(|entry| Reverse(entry.finished_at_ms));
    serde_json::to_string(&entries).expect("as entradas só têm campos simples")
}

/// O nome do desfecho no formato do `SavedResult`.
fn outcome_name(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Won => "Won",
        Outcome::OutOfLives => "OutOfLives",
        Outcome::Abandoned => "Abandoned",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    /// Um `SavedResult` válido em JSON, no formato da versão 1.
    fn saved(difficulty: &str, outcome: &str, elapsed_ms: u64, finished_at: &str) -> Value {
        json!({
            "version": 1,
            "difficulty": difficulty,
            "seed": "42",
            "outcome": outcome,
            "elapsed_ms": elapsed_ms,
            "mistakes": 0,
            "hints_used": 0,
            "lives": 3,
            "hints": 2,
            "finished_at": finished_at,
        })
    }

    fn entries(input: &Value) -> Value {
        serde_json::from_str(&history_entries(&input.to_string())).expect("saída em JSON")
    }

    #[test]
    fn historico_mais_recente_primeiro() {
        let input = json!([
            saved("Facil", "Won", 600_000, "1000"),
            saved("Medio", "OutOfLives", 30_000, "2000"),
        ]);

        assert_eq!(
            entries(&input),
            json!([
                { "difficulty": "Medio", "outcome": "OutOfLives", "elapsedMs": 30_000, "finishedAtMs": 2000 },
                { "difficulty": "Facil", "outcome": "Won", "elapsedMs": 600_000, "finishedAtMs": 1000 },
            ])
        );
    }

    #[test]
    fn registro_ruim_some() {
        let mut bad_version = saved("Facil", "Won", 1, "1500");
        bad_version["version"] = 99.into();
        let input = json!([
            saved("Facil", "Won", 600_000, "1000"),
            bad_version,
            "lixo",
            42,
            saved("Medio", "OutOfLives", 30_000, "2000"),
        ]);

        let kept = entries(&input);
        assert_eq!(kept.as_array().map(Vec::len), Some(2));
        assert_eq!(kept[0]["finishedAtMs"], 2000);
        assert_eq!(kept[1]["finishedAtMs"], 1000);

        assert_eq!(history_entries("não é json"), "[]");
        assert_eq!(history_entries("{}"), "[]");
    }
}
