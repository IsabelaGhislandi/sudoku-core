//! Testes de integração: só enxergam o que o `sudoku-game` exporta. Nada de `use sudoku_core`,
//! para provar que a casca consegue jogar dependendo só deste crate.

use sudoku_game::{
    Cell, CellState, Difficulty, Game, GameConfig, MoveError, PlaceOutcome, Restart, SavedGame,
    Status,
};

fn solution_value(game: &Game, index: usize) -> u8 {
    // `Puzzle`, `Grid` e `Cell` vêm pelo re-export.
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

#[test]
fn partida_completa_ate_a_vitoria() {
    let mut game = Game::new(Difficulty::Medio, 7, GameConfig::default());
    game.resume(0);
    let first = game.hint(0).unwrap();
    let second = game.hint(0).unwrap();
    assert_eq!(game.cell(first.index), CellState::Hinted(first.value));
    assert_eq!(game.cell(second.index), CellState::Hinted(second.value));

    for i in empties(&game) {
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v, 0), Ok(PlaceOutcome::Correct));
    }
    assert_eq!(game.status(), Status::Won);
    assert_eq!((game.mistakes(), game.hints_used()), (0, 2));
}

#[test]
fn partida_completa_ate_a_derrota() {
    let mut game = Game::new(Difficulty::Dificil, 7, GameConfig::default());
    game.resume(0);
    let i = empties(&game)[0];
    game.toggle_note(i, 4).unwrap();
    let right = solution_value(&game, i);
    let wrong = if right == 9 { 1 } else { right + 1 };
    for lives in [2, 1, 0] {
        assert_eq!(
            game.place(i, wrong, 0),
            Ok(PlaceOutcome::Wrong {
                index: i,
                value: wrong
            })
        );
        assert_eq!(game.lives_left(), lives);
    }
    assert_eq!(game.status(), Status::Lost);
    assert_eq!(game.notes(i).values(), vec![4]);
    assert_eq!(game.place(i, right, 0), Err(MoveError::GameOver));
    assert_eq!(game.hint(0), Err(MoveError::GameOver));
    assert_eq!(game.toggle_note(i, 5), Err(MoveError::GameOver));
}

#[test]
fn partida_sai_e_volta_sem_perder_nada() {
    let mut game = Game::new(Difficulty::Facil, 11, GameConfig::default());
    game.resume(1_000);
    let open = empties(&game);
    let half = open.len() / 2;
    for &i in &open[..half] {
        let v = solution_value(&game, i);
        game.place(i, v, 5_000).unwrap();
    }
    game.toggle_note(open[half], 3).unwrap();
    // O app vai para o fundo: pausa e grava a foto.
    game.pause(6_000);
    let saved: SavedGame = game.save(6_000);

    // Dias depois, outro processo: o relógio monotônico recomeçou do zero.
    let mut restored = Game::restore(&saved).unwrap();
    assert!(restored.is_paused());
    assert_eq!(restored.notes(open[half]).values(), vec![3]);
    restored.resume(100);
    for &i in &open[half..] {
        let v = solution_value(&restored, i);
        restored.place(i, v, 2_100).unwrap();
    }
    assert_eq!(restored.status(), Status::Won);
    // 5 s antes de sair (1_000 → 6_000) + 2 s depois de voltar (100 → 2_100).
    assert_eq!(restored.elapsed_ms(50_000), 7_000);
}

#[test]
fn reiniciar_pela_api_publica() {
    let game = Game::new(Difficulty::Medio, 3, GameConfig::default());
    let Restart { game: new_game, .. } = game.restart(4);
    assert_eq!(new_game.puzzle().difficulty, Difficulty::Medio);
    assert!(new_game.is_paused());
}
