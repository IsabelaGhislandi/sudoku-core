//! Foto da partida em JSON, como o app vai gravar. Só compila com a feature `serde`:
//! `cargo test -p sudoku-game --features serde` (ou `--all-features`).
#![cfg(feature = "serde")]

use sudoku_game::{Cell, CellState, Difficulty, Game, GameConfig, SavedGame};

fn solution_value(game: &Game, index: usize) -> u8 {
    match game.puzzle().solution.get(index) {
        Cell::Filled(v) => v,
        Cell::Empty => panic!("a solução não tem células vazias"),
    }
}

fn empties(game: &Game) -> Vec<usize> {
    (0..81)
        .filter(|&i| game.cell(i) == CellState::Empty)
        .collect()
}

fn to_json(saved: &SavedGame) -> String {
    serde_json::to_string_pretty(saved).unwrap()
}

fn from_json(json: &str) -> Game {
    let saved: SavedGame = serde_json::from_str(json).unwrap();
    Game::restore(&saved).unwrap()
}

#[test]
fn foto_vai_e_volta_em_json() {
    let mut game = Game::new(Difficulty::Medio, 7, GameConfig::default());
    game.resume(0);
    let open = empties(&game);
    let v = solution_value(&game, open[0]);
    game.place(open[0], v, 1_000).unwrap();
    let w = if solution_value(&game, open[1]) == 9 {
        1
    } else {
        solution_value(&game, open[1]) + 1
    };
    game.place(open[1], w, 2_000).unwrap();
    game.toggle_note(open[2], solution_value(&game, open[2]))
        .unwrap();
    game.hint(3_000).unwrap();

    let restored = from_json(&to_json(&game.save(4_000)));
    for i in 0..81 {
        assert_eq!(restored.cell(i), game.cell(i));
        assert_eq!(restored.notes(i), game.notes(i));
    }
    assert_eq!(restored.puzzle(), game.puzzle());
    assert_eq!(
        (
            restored.lives_left(),
            restored.hints_left(),
            restored.mistakes()
        ),
        (game.lives_left(), game.hints_left(), game.mistakes()),
    );
    assert_eq!(restored.elapsed_ms(0), 4_000);
}

#[test]
fn seed_acima_de_2_53_sobrevive_ao_json() {
    let game = Game::new(Difficulty::Facil, u64::MAX - 1, GameConfig::default());
    let seed = game.puzzle().seed;
    assert!(seed > 1 << 53, "a seed do teste precisa passar de 2^53");
    let json = to_json(&game.save(0));
    assert!(
        json.contains(&format!("\"seed\": \"{seed}\"")),
        "a seed vai entre aspas: {json}"
    );
    assert_eq!(from_json(&json).puzzle().seed, seed);
}

/// Golden test do formato versão 1. Uma partida salva por uma versão antiga do app precisa
/// continuar abrindo: mudou este texto? Ou é bug, ou é versão nova do formato, com leitura da
/// antiga, no mesmo commit.
#[test]
fn formato_da_foto_versao_1_nao_muda() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    game.resume(0);
    let open = empties(&game);
    let v = solution_value(&game, open[0]);
    game.place(open[0], v, 1_000).unwrap();
    game.toggle_note(open[1], 2).unwrap();
    game.hint(2_000).unwrap();
    let saved = game.save(3_000);
    assert_eq!(serde_json::to_string(&saved).unwrap(), GOLDEN);
    // O texto travado também precisa abrir: é o que um app antigo deixou no disco.
    let reopened: SavedGame = serde_json::from_str(GOLDEN).unwrap();
    assert_eq!(reopened, saved);
    assert!(Game::restore(&reopened).is_ok());
}

/// Fácil seed 42: o 8 certo na célula 2, a anotação 2 na célula 5 e a dica (o 9) na célula 12,
/// salvos aos 3 s. JSON compacto, quebrado aqui por campo só para leitura.
const GOLDEN: &str = concat!(
    r#"{"version":1,"difficulty":"Facil","seed":"42","#,
    r#""givens":"94.35...12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.....68","#,
    r#""solution":"948352671251967483763481259127836594834795126695214837586179342372648915419523768","#,
    r#""board":"94835...12..9.7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.....68","#,
    r#""hinted":[12],"#,
    // 81 anotações: só a célula 5 tem uma.
    r#""notes":["","","","","","2","#,
    r#""","","","","","","","","","","","","","","","","","","","","","","","","","#,
    r#""","","","","","","","","","","","","","","","","","","","","","","","","","#,
    r#""","","","","","","","","","","","","","","","","","","","","","","","",""],"#,
    r#""lives":3,"hints":2,"lives_left":3,"hints_left":1,"mistakes":0,"hints_used":1,"#,
    r#""elapsed_ms":3000}"#,
);
