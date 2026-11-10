//! sudoku-wasm — a ponte entre o `sudoku-game` e o JavaScript do navegador.
//!
//! Casca fina: nenhuma regra de jogo mora aqui. Tudo que cruza a ponte é texto ou número
//! simples: seeds em decimal (o `number` do JS perde precisão acima de 2^53), a foto e os
//! resultados nos formatos JSON do `sudoku-game`, e o retrato da partida (`GameView`) em JSON.
//! Erros voltam como `Err(String)`, que o JS recebe como exceção com o nome do erro.

// Todo item público precisa de doc: com o `-D warnings` do CI, faltar uma vira erro.
#![warn(missing_docs)]

mod convert;
mod view;

use sudoku_game::{Game, GameConfig, PlaceOutcome};
use view::GameView;
use wasm_bindgen::prelude::*;

/// Uma partida, vista do JavaScript.
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
}

#[wasm_bindgen]
impl WebGame {
    /// Começa uma partida nova do nível `difficulty` (nome do Rust, como `"Facil"`) com a seed
    /// `seed` em decimal. A partida nasce pausada.
    #[wasm_bindgen(constructor)]
    pub fn new(difficulty: &str, seed: &str) -> Result<WebGame, String> {
        let difficulty = convert::parse_difficulty(difficulty)?;
        let seed = convert::parse_seed(seed)?;
        Ok(WebGame {
            game: Game::new(difficulty, seed, GameConfig::default()),
        })
    }

    /// O retrato da partida no instante `now`, em JSON (ver `GameView`).
    pub fn view(&self, now: f64) -> String {
        let view = GameView::from_game(&self.game, convert::to_ms(now));
        serde_json::to_string(&view).expect("o retrato só tem campos serializáveis")
    }

    /// Coloca `value` na célula `index`. Devolve `"correct"` ou `"wrong"`; uma jogada recusada
    /// volta como erro com o nome do `MoveError`.
    pub fn place(&mut self, index: usize, value: u8, now: f64) -> Result<String, String> {
        let outcome = self
            .game
            .place(index, value, convert::to_ms(now))
            .map_err(convert::error_name)?;
        Ok(match outcome {
            PlaceOutcome::Correct => "correct",
            PlaceOutcome::Wrong { .. } => "wrong",
        }
        .to_string())
    }

    /// Liga o relógio no instante `now`.
    pub fn resume(&mut self, now: f64) {
        self.game.resume(convert::to_ms(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use sudoku_game::{Cell, CellState, Difficulty, GameConfig};

    /// A partida de referência da seed 42 no Fácil: a mesma que `WebGame::new("Facil", "42")`.
    fn reference() -> Game {
        Game::new(Difficulty::Facil, 42, GameConfig::default())
    }

    /// As células vazias da partida de referência e o número certo de cada uma.
    fn empty_cells_with_answers() -> Vec<(usize, u8)> {
        let game = reference();
        (0..81)
            .filter(|&i| game.cell(i) == CellState::Empty)
            .map(|i| match game.puzzle().solution.get(i) {
                Cell::Filled(v) => (i, v),
                Cell::Empty => panic!("a solução é completa"),
            })
            .collect()
    }

    fn view_json(game: &WebGame, now: f64) -> Value {
        serde_json::from_str(&game.view(now)).expect("o retrato é JSON válido")
    }

    #[test]
    fn retrato_de_partida_nova() {
        let game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        let view = view_json(&game, 0.0);
        let reference = reference();

        assert_eq!(view["difficulty"], "Facil");
        assert_eq!(view["status"], "playing");
        assert_eq!(view["paused"], true);
        assert_eq!(view["livesLeft"], 3);
        assert_eq!(view["hintsLeft"], 2);
        assert_eq!(view["elapsedMs"], 0);
        let cells = view["cells"].as_array().expect("cells é um array");
        assert_eq!(cells.len(), 81);
        for (i, cell) in cells.iter().enumerate() {
            match reference.cell(i) {
                CellState::Given(v) => {
                    assert_eq!(cell["kind"], "given", "célula {i}");
                    assert_eq!(cell["value"], v, "célula {i}");
                }
                _ => {
                    assert_eq!(cell["kind"], "empty", "célula {i}");
                    assert_eq!(cell["value"], Value::Null, "célula {i}");
                }
            }
        }
    }

    #[test]
    fn jogada_certa_e_errada() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        let empties = empty_cells_with_answers();
        let (first, right) = empties[0];
        let (second, answer) = empties[1];
        let wrong = answer % 9 + 1;

        assert_eq!(game.place(first, right, 10.0), Ok("correct".to_string()));
        assert_eq!(game.place(second, wrong, 15.0), Ok("wrong".to_string()));
        assert_eq!(view_json(&game, 15.0)["livesLeft"], 2);
        assert_eq!(
            game.place(first, right, 20.0),
            Err("CellNotEmpty".to_string())
        );
    }

    #[test]
    fn pausada_recusa_jogada() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        let (index, value) = empty_cells_with_answers()[0];

        assert_eq!(game.place(index, value, 10.0), Err("Paused".to_string()));
    }
}
