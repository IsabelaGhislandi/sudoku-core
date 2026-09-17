use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;

use crate::backtracking::count_solutions;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};
use crate::solver::LogicalSolver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub givens: Grid,
    pub solution: Grid,
    pub difficulty: Difficulty,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenError {
    Unsupported(Difficulty),
}

/// Gera um puzzle determinístico no nível pedido. Nesta fatia, só Fácil.
///
/// Contrato: mesma seed + mesmo nível => sempre o mesmo puzzle, inclusive entre
/// versões da lib. Por isso o RNG é o `ChaCha12Rng` (algoritmo fixo) e não o
/// `StdRng`, cujo algoritmo o `rand` se reserva o direito de trocar. A seed é
/// pública e vai virar código de puzzle compartilhável.
pub fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError> {
    if difficulty != Difficulty::Facil {
        return Err(GenError::Unsupported(difficulty));
    }
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let solution = full_solution(&mut rng);
    let givens = dig_facil(&solution, &mut rng);
    Ok(Puzzle {
        givens,
        solution,
        difficulty: Difficulty::Facil,
        seed,
    })
}

/// Constrói uma grade completa e válida via backtracking randomizado.
fn full_solution(rng: &mut ChaCha12Rng) -> Grid {
    let mut grid = Grid::empty();
    fill(&mut grid, rng);
    grid
}

fn fill(grid: &mut Grid, rng: &mut ChaCha12Rng) -> bool {
    let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
    match next {
        None => true,
        Some(index) => {
            let mut values: Vec<u8> = (1..=9).collect();
            values.shuffle(rng);
            for value in values {
                if grid.can_place(index, value) {
                    grid.set(index, Cell::Filled(value));
                    if fill(grid, rng) {
                        return true;
                    }
                    grid.set(index, Cell::Empty);
                }
            }
            false
        }
    }
}

/// Remove células enquanto (a) a solução continua única e (b) o solver lógico
/// "Fácil" ainda resolve por completo — o que mantém a dificuldade em Fácil.
fn dig_facil(solution: &Grid, rng: &mut ChaCha12Rng) -> Grid {
    let mut puzzle = solution.clone();
    let mut order: Vec<usize> = (0..81).collect();
    order.shuffle(rng);
    let solver = LogicalSolver::new();
    for index in order {
        let removed = puzzle.get(index);
        puzzle.set(index, Cell::Empty);
        let unique = count_solutions(&puzzle, 2) == 1;
        let logical = solver.solve(&puzzle).solved;
        if !(unique && logical) {
            puzzle.set(index, removed); // restaura
        }
    }
    puzzle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::validate;

    #[test]
    fn gera_puzzle_facil_valido_e_unico() {
        let puzzle = generate(Difficulty::Facil, 42).unwrap();
        assert_eq!(puzzle.difficulty, Difficulty::Facil);
        assert_eq!(puzzle.seed, 42);
        // solução completa e consistente
        assert!(puzzle.solution.is_complete());
        assert!(validate(&puzzle.solution).is_empty());
        // o enunciado tem solução única e é classificado como Fácil
        assert_eq!(count_solutions(&puzzle.givens, 2), 1);
        assert_eq!(crate::rating::rate(&puzzle.givens), Some(Difficulty::Facil));
        // o enunciado removeu pelo menos algumas células
        let vazias = (0..81).filter(|&i| puzzle.givens.get(i) == Cell::Empty).count();
        assert!(vazias > 0);
    }

    #[test]
    fn puzzle_da_seed_42_e_reproduzivel() {
        // Golden test: trava a saída da seed 42 para que trocar de RNG (ou de versão
        // da lib de RNG) não passe despercebido. A seed é contrato público — o plano
        // prevê compartilhar puzzle por código.
        let puzzle = generate(Difficulty::Facil, 42).unwrap();
        assert_eq!(
            puzzle.givens.to_line(),
            "94.35...12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.....68"
        );
    }

    #[test]
    fn mesma_seed_gera_mesmo_puzzle() {
        let a = generate(Difficulty::Facil, 7).unwrap();
        let b = generate(Difficulty::Facil, 7).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn seeds_diferentes_geram_puzzles_diferentes() {
        let a = generate(Difficulty::Facil, 1).unwrap();
        let b = generate(Difficulty::Facil, 2).unwrap();
        assert_ne!(a.solution, b.solution);
    }

    #[test]
    fn niveis_nao_suportados_ainda_retornam_erro() {
        assert_eq!(
            generate(Difficulty::Medio, 1),
            Err(GenError::Unsupported(Difficulty::Medio))
        );
    }
}
