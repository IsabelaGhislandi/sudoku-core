use sudoku_core::{
    Action, Cell, Difficulty, LogicalSolver, candidates_for, count_solutions, generate, rate,
    validate,
};

#[test]
fn fluxo_completo_facil() {
    let puzzle = generate(Difficulty::Facil, 123).unwrap();

    // enunciado é único e Fácil
    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Facil));
    assert!(validate(&puzzle.givens).is_empty());

    // candidates_for é acessível pela API pública
    let some_empty = (0..81)
        .find(|&i| puzzle.givens.get(i) == Cell::Empty)
        .unwrap();
    assert!(candidates_for(&puzzle.givens, some_empty).count() >= 1);
}

#[test]
fn fluxo_completo_medio() {
    let puzzle = generate(Difficulty::Medio, 123).unwrap();

    // enunciado é único, sem conflitos e Médio
    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert!(validate(&puzzle.givens).is_empty());
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Medio));

    // os passos do solver ficam visíveis pela API pública, incluindo eliminações
    let result = LogicalSolver::new().solve(&puzzle.givens);
    assert!(result.solved);
    assert_eq!(result.grid, puzzle.solution);
    assert!(
        result
            .steps
            .iter()
            .any(|s| matches!(s.action, Action::Eliminate(_)))
    );
}

#[test]
fn fluxo_completo_dificil() {
    let puzzle = generate(Difficulty::Dificil, 123).unwrap();

    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert!(validate(&puzzle.givens).is_empty());
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Dificil));

    let result = LogicalSolver::new().solve(&puzzle.givens);
    assert!(result.solved);
    assert_eq!(result.grid, puzzle.solution);
    assert!(result.steps.iter().any(|s| s.level == Difficulty::Dificil));
}

#[test]
fn fluxo_completo_muito_dificil() {
    let puzzle = generate(Difficulty::MuitoDificil, 14).unwrap();

    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert!(validate(&puzzle.givens).is_empty());
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::MuitoDificil));
    let givens = (0..81)
        .filter(|&i| puzzle.givens.get(i) != Cell::Empty)
        .count();
    assert!(givens <= 22, "{givens} números");

    let result = LogicalSolver::new().solve(&puzzle.givens);
    assert!(result.solved);
    assert_eq!(result.grid, puzzle.solution);
    assert!(
        result
            .steps
            .iter()
            .any(|s| s.level == Difficulty::MuitoDificil)
    );
}
