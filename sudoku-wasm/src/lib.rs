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

use sudoku_game::{Game, GameConfig, GameResult, PlaceOutcome};
use view::GameView;
use wasm_bindgen::prelude::*;

/// Um resultado no formato do histórico: o `SavedResult` em JSON.
fn save_result(result: &GameResult) -> String {
    serde_json::to_string(&result.save()).expect("o SavedResult só tem campos simples")
}

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

    /// Liga ou desliga a anotação `value` na célula `index`. Recusa como `place`.
    #[wasm_bindgen(js_name = toggleNote)]
    pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), String> {
        self.game
            .toggle_note(index, value)
            .map_err(convert::error_name)
    }

    /// Revela uma célula vazia, sorteada pelo jogo. Recusa com `GameOver`, `Paused` ou
    /// `NoHintsLeft`.
    pub fn hint(&mut self, now: f64) -> Result<(), String> {
        // A célula revelada não precisa voltar: ela aparece no próximo retrato, como `hinted`.
        self.game
            .hint(convert::to_ms(now))
            .map(|_| ())
            .map_err(convert::error_name)
    }

    /// Para o relógio no instante `now`.
    pub fn pause(&mut self, now: f64) {
        self.game.pause(convert::to_ms(now));
    }

    /// Troca esta partida por uma nova do mesmo nível, com a seed `seed`, pausada. Se a antiga
    /// não tinha terminado, devolve o abandono como `SavedResult` em JSON; se já tinha, `None`.
    pub fn restart(
        &mut self,
        seed: &str,
        now: f64,
        finished_at_ms: f64,
    ) -> Result<Option<String>, String> {
        let seed = convert::parse_seed(seed)?;
        let restart = self
            .game
            .restart(seed, convert::to_ms(now), convert::to_ms(finished_at_ms));
        self.game = restart.game;
        Ok(restart.abandoned.map(|result| save_result(&result)))
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

    /// Perde a partida da seed 42: três números errados na primeira célula vazia.
    fn lose(game: &mut WebGame) {
        let (index, answer) = empty_cells_with_answers()[0];
        let wrong = answer % 9 + 1;
        for _ in 0..3 {
            assert_eq!(game.place(index, wrong, 1.0), Ok("wrong".to_string()));
        }
        assert_eq!(view_json(game, 1.0)["status"], "lost");
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

    #[test]
    fn anotacao_aparece_no_retrato() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        let (index, _) = empty_cells_with_answers()[0];

        game.toggle_note(index, 3)
            .expect("célula vazia aceita anotação");
        game.toggle_note(index, 7)
            .expect("célula vazia aceita anotação");
        assert_eq!(
            view_json(&game, 0.0)["cells"][index]["notes"],
            serde_json::json!([3, 7])
        );

        game.toggle_note(index, 3)
            .expect("célula vazia aceita anotação");
        assert_eq!(
            view_json(&game, 0.0)["cells"][index]["notes"],
            serde_json::json!([7])
        );
    }

    #[test]
    fn dica_revela_uma_celula() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        let hinted = |game: &WebGame| {
            view_json(game, 0.0)["cells"]
                .as_array()
                .expect("cells é um array")
                .iter()
                .filter(|c| c["kind"] == "hinted")
                .count()
        };

        game.hint(5.0).expect("ainda há dicas");
        assert_eq!(hinted(&game), 1);
        assert_eq!(view_json(&game, 5.0)["hintsLeft"], 1);

        game.hint(6.0).expect("ainda há uma dica");
        assert_eq!(game.hint(7.0), Err("NoHintsLeft".to_string()));
    }

    #[test]
    fn pausa_congela_o_tempo() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        game.pause(1000.0);

        let view = view_json(&game, 5000.0);
        assert_eq!(view["paused"], true);
        assert_eq!(view["elapsedMs"], 1000);
    }

    #[test]
    fn recomecar_devolve_o_abandono() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        let before = view_json(&game, 0.0)["cells"].clone();

        let abandoned = game
            .restart("7", 3000.0, 1_700_000_000_000.0)
            .expect("seed válida")
            .expect("a partida estava em andamento");
        let abandoned: Value = serde_json::from_str(&abandoned).expect("resultado em JSON");
        assert_eq!(abandoned["outcome"], "Abandoned");
        assert_eq!(abandoned["seed"], "42");
        assert_eq!(abandoned["elapsed_ms"], 3000);

        let after = view_json(&game, 3000.0);
        assert_ne!(after["cells"], before);
        assert_eq!(after["paused"], true);
    }

    #[test]
    fn recomecar_partida_terminada_nao_devolve_abandono() {
        let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
        game.resume(0.0);
        lose(&mut game);

        assert_eq!(game.restart("7", 10.0, 0.0), Ok(None));
        assert_eq!(view_json(&game, 10.0)["status"], "playing");
    }
}
