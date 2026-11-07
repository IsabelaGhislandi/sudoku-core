//! Resultado de partida em JSON, como o app vai gravar no histórico. Só compila com a feature
//! `serde`: `cargo test -p sudoku-game --features serde` (ou `--all-features`).
#![cfg(feature = "serde")]

use sudoku_game::{Cell, CellState, Difficulty, Game, GameConfig, GameResult, SavedResult};

/// 13/06/2026 12:00 UTC, em milissegundos desde a época Unix.
const DATE: u64 = 1_781_352_000_000;

fn solution_value(game: &Game, index: usize) -> u8 {
    match game.puzzle().solution.get(index) {
        Cell::Filled(v) => v,
        Cell::Empty => panic!("a solução não tem células vazias"),
    }
}

#[test]
fn formato_do_resultado_v1_nao_muda() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    game.resume(0);
    let i = (0..81).find(|&i| game.cell(i) == CellState::Empty).unwrap();
    let wrong = if solution_value(&game, i) == 9 {
        1
    } else {
        solution_value(&game, i) + 1
    };
    for now in [1_000, 2_000, 3_000] {
        game.place(i, wrong, now).unwrap();
    }
    let saved = game.result(DATE).unwrap().save();
    let expected = r#"{
  "version": 1,
  "difficulty": "Facil",
  "seed": "42",
  "outcome": "OutOfLives",
  "elapsed_ms": 3000,
  "mistakes": 3,
  "hints_used": 0,
  "lives": 3,
  "hints": 2,
  "finished_at": "1781352000000"
}"#;
    assert_eq!(serde_json::to_string_pretty(&saved).unwrap(), expected);
    let back: SavedResult = serde_json::from_str(expected).unwrap();
    assert_eq!(
        GameResult::restore(&back).unwrap(),
        game.result(DATE).unwrap()
    );
}
