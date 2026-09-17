use sudoku_core::{
    candidates_for, count_solutions, generate, next_hint, rate, validate, Cell, Difficulty,
};

#[test]
fn fluxo_completo_facil() {
    let puzzle = generate(Difficulty::Facil, 123).unwrap();

    // enunciado é único e Fácil
    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Facil));
    assert!(validate(&puzzle.givens).is_empty());

    // a dica aponta uma jogada que bate com a solução
    let hint = next_hint(&puzzle.givens).expect("deve haver uma dica");
    assert_eq!(puzzle.solution.get(hint.index), Cell::Filled(hint.value));

    // candidates_for é acessível pela API pública
    let alguma_vazia = (0..81).find(|&i| puzzle.givens.get(i) == Cell::Empty).unwrap();
    assert!(candidates_for(&puzzle.givens, alguma_vazia).count() >= 1);
}
