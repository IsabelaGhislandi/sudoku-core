//! Testes de integração: só enxergam o que o `sudoku-game` exporta. Nada de `use sudoku_core`,
//! para provar que a casca consegue jogar dependendo só deste crate.

use sudoku_game::{Cell, CellState, Difficulty, Game, GameConfig, MoveError, PlaceOutcome, Status};

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
    let first = game.hint().unwrap();
    let second = game.hint().unwrap();
    assert_eq!(game.cell(first.index), CellState::Hinted(first.value));
    assert_eq!(game.cell(second.index), CellState::Hinted(second.value));

    for i in empties(&game) {
        let v = solution_value(&game, i);
        assert_eq!(game.place(i, v), Ok(PlaceOutcome::Correct));
    }
    assert_eq!(game.status(), Status::Won);
    assert_eq!((game.mistakes(), game.hints_used()), (0, 2));
}

#[test]
fn partida_completa_ate_a_derrota() {
    let mut game = Game::new(Difficulty::Dificil, 7, GameConfig::default());
    let i = empties(&game)[0];
    game.toggle_note(i, 4).unwrap();
    let right = solution_value(&game, i);
    let wrong = if right == 9 { 1 } else { right + 1 };
    for lives in [2, 1, 0] {
        assert_eq!(
            game.place(i, wrong),
            Ok(PlaceOutcome::Wrong {
                index: i,
                value: wrong
            })
        );
        assert_eq!(game.lives_left(), lives);
    }
    assert_eq!(game.status(), Status::Lost);
    assert_eq!(game.notes(i).values(), vec![4]);
    assert_eq!(game.place(i, right), Err(MoveError::GameOver));
    assert_eq!(game.hint(), Err(MoveError::GameOver));
    assert_eq!(game.toggle_note(i, 5), Err(MoveError::GameOver));
}
