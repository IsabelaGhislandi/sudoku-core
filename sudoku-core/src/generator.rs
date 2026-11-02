use rand::SeedableRng;
use rand::seq::SliceRandom;
use rand_chacha::ChaCha12Rng;

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
    AttemptsExhausted(Difficulty),
}

/// Limite de tentativas para gerar um Médio. É um número fixo, e não um limite de tempo,
/// porque a mesma seed precisa dar o mesmo resultado em qualquer aparelho. Faz parte do
/// contrato de determinismo: reduzir pode transformar um puzzle em erro; aumentar só
/// transforma erro em puzzle, sem alterar os que já existem.
const MAX_TENTATIVAS_MEDIO: usize = 50;

/// Limite de tentativas para gerar um Difícil, com o mesmo papel no contrato que o do
/// Médio.
const MAX_TENTATIVAS_DIFICIL: usize = 800;

/// Gera um puzzle determinístico no nível pedido. Por enquanto, Fácil, Médio e Difícil.
///
/// Contrato: mesma seed + mesmo nível => sempre o mesmo puzzle, inclusive entre
/// versões da lib. Por isso o RNG é o `ChaCha12Rng` (algoritmo fixo) e não o
/// `StdRng`, cujo algoritmo o `rand` se reserva o direito de trocar. A seed é
/// pública e vai virar código de puzzle compartilhável.
pub fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError> {
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    match difficulty {
        Difficulty::Facil => {
            let solution = full_solution(&mut rng);
            let solver = LogicalSolver::up_to(Difficulty::Facil);
            let givens = dig(&solution, &solver, &mut rng);
            Ok(Puzzle {
                givens,
                solution,
                difficulty,
                seed,
            })
        }
        Difficulty::Medio => tentar(difficulty, MAX_TENTATIVAS_MEDIO, seed, &mut rng),
        Difficulty::Dificil => tentar(difficulty, MAX_TENTATIVAS_DIFICIL, seed, &mut rng),
        _ => Err(GenError::Unsupported(difficulty)),
    }
}

/// Repete tentativas até o puzzle exigir o nível pedido. Cavar com as técnicas de um
/// nível não garante exigi-las: o puzzle pode continuar resolvível com técnicas mais
/// simples. Todas as tentativas compartilham o mesmo RNG, então o resultado continua
/// determinístico.
///
/// O aceite dá o mesmo resultado de `rate(&givens) == Some(difficulty)`, sem a busca
/// de unicidade: `dig` só devolve puzzles que o solver resolve com deduções forçadas,
/// então eles já são únicos. E como as técnicas acima do nível ficam no fim da lista,
/// o solver limitado faz os mesmos passos que o completo faria.
fn tentar(
    difficulty: Difficulty,
    max: usize,
    seed: u64,
    rng: &mut ChaCha12Rng,
) -> Result<Puzzle, GenError> {
    let solver = LogicalSolver::up_to(difficulty);
    for _ in 0..max {
        let solution = full_solution(rng);
        let givens = dig(&solution, &solver, rng);
        let exigido = solver.solve(&givens).steps.iter().map(|s| s.level).max();
        if exigido.unwrap_or(Difficulty::Facil) == difficulty {
            return Ok(Puzzle {
                givens,
                solution,
                difficulty,
                seed,
            });
        }
    }
    Err(GenError::AttemptsExhausted(difficulty))
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

/// Remove células enquanto o solver recebido ainda resolve o puzzle por completo, o que
/// também garante solução única. É o solver que limita o nível do puzzle.
fn dig(solution: &Grid, solver: &LogicalSolver, rng: &mut ChaCha12Rng) -> Grid {
    let mut puzzle = solution.clone();
    let mut order: Vec<usize> = (0..81).collect();
    order.shuffle(rng);
    for index in order {
        let removed = puzzle.get(index);
        puzzle.set(index, Cell::Empty);
        // Não é preciso testar a unicidade aqui: se o solver resolve só com deduções
        // forçadas, a solução já é única. A checagem por busca custava cerca de 80% do
        // tempo de gerar um Médio.
        if !solver.solve(&puzzle).solved {
            puzzle.set(index, removed); // restaura
        }
    }
    puzzle
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backtracking::count_solutions;
    use crate::grid::validate;
    use crate::techniques::Action;

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
        let vazias = (0..81)
            .filter(|&i| puzzle.givens.get(i) == Cell::Empty)
            .count();
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
    fn puzzle_medio_da_seed_42_e_reproduzivel() {
        // Golden test do Médio, no mesmo espírito do Fácil. Também trava
        // MAX_TENTATIVAS_MEDIO e a ordem das técnicas, que fazem parte do contrato.
        let puzzle = generate(Difficulty::Medio, 42).unwrap();
        assert_eq!(
            puzzle.givens.to_line(),
            "94.3....12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.5...68"
        );
    }

    #[test]
    fn puzzle_dificil_da_seed_42_e_reproduzivel() {
        // Golden test do Difícil. Também trava MAX_TENTATIVAS_DIFICIL e a ordem das
        // técnicas, que fazem parte do contrato.
        let puzzle = generate(Difficulty::Dificil, 42).unwrap();
        assert_eq!(
            puzzle.givens.to_line(),
            ".2..574...79..6...1.............1...5...6.87.4...3.1.5.459...6.9.......26.....7.."
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
            generate(Difficulty::MuitoDificil, 1),
            Err(GenError::Unsupported(Difficulty::MuitoDificil))
        );
    }

    #[test]
    fn gera_puzzle_medio_valido_e_unico() {
        let p = generate(Difficulty::Medio, 42).unwrap();
        assert_eq!(p.difficulty, Difficulty::Medio);
        assert_eq!(p.seed, 42);
        assert_eq!(count_solutions(&p.givens, 2), 1);
        assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Medio));
    }

    #[test]
    fn mesma_seed_gera_mesmo_puzzle_medio() {
        let a = generate(Difficulty::Medio, 7).unwrap();
        let b = generate(Difficulty::Medio, 7).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn seeds_extremas_geram_medio_valido() {
        for seed in [0, u64::MAX] {
            let p = generate(Difficulty::Medio, seed).unwrap();
            assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Medio));
        }
    }

    #[test]
    fn eliminacoes_nunca_removem_a_resposta() {
        for seed in 1..=3 {
            let p = generate(Difficulty::Medio, seed).unwrap();
            let result = LogicalSolver::new().solve(&p.givens);
            let mut eliminacoes = 0;
            for step in &result.steps {
                match &step.action {
                    Action::Place { index, value } => {
                        assert_eq!(p.solution.get(*index), Cell::Filled(*value));
                    }
                    Action::Eliminate(pares) => {
                        eliminacoes += 1;
                        for &(index, value) in pares {
                            assert_ne!(p.solution.get(index), Cell::Filled(value));
                        }
                    }
                }
            }
            assert_eq!(result.grid, p.solution);
            assert!(eliminacoes > 0, "seed {seed} deveria exigir eliminação");
        }
    }

    #[test]
    fn gera_puzzle_dificil_valido_e_unico() {
        let p = generate(Difficulty::Dificil, 42).unwrap();
        assert_eq!(p.difficulty, Difficulty::Dificil);
        assert_eq!(p.seed, 42);
        assert_eq!(count_solutions(&p.givens, 2), 1);
        assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Dificil));
    }

    #[test]
    fn mesma_seed_gera_mesmo_puzzle_dificil() {
        let a = generate(Difficulty::Dificil, 7).unwrap();
        let b = generate(Difficulty::Dificil, 7).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn seeds_extremas_geram_dificil_valido() {
        for seed in [0, u64::MAX] {
            let p = generate(Difficulty::Dificil, seed).unwrap();
            assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Dificil));
        }
    }

    #[test]
    fn eliminacoes_do_dificil_nunca_removem_a_resposta() {
        for seed in 1..=3 {
            let p = generate(Difficulty::Dificil, seed).unwrap();
            let result = LogicalSolver::new().solve(&p.givens);
            for step in &result.steps {
                match &step.action {
                    Action::Place { index, value } => {
                        assert_eq!(p.solution.get(*index), Cell::Filled(*value));
                    }
                    Action::Eliminate(pares) => {
                        for &(index, value) in pares {
                            assert_ne!(p.solution.get(index), Cell::Filled(value));
                        }
                    }
                }
            }
            assert_eq!(result.grid, p.solution);
            assert!(
                result.steps.iter().any(|s| s.level == Difficulty::Dificil),
                "seed {seed} deveria exigir técnica de Difícil"
            );
        }
    }

    #[test]
    fn medio_nunca_usa_tecnica_de_dificil() {
        for seed in 0..5 {
            let p = generate(Difficulty::Medio, seed).unwrap();
            let result = LogicalSolver::new().solve(&p.givens);
            assert!(result.steps.iter().all(|s| s.level <= Difficulty::Medio));
            assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Medio));
        }
    }

    #[test]
    fn facil_nunca_usa_tecnica_de_medio() {
        for seed in 0..5 {
            let p = generate(Difficulty::Facil, seed).unwrap();
            let result = LogicalSolver::new().solve(&p.givens);
            assert!(
                result
                    .steps
                    .iter()
                    .all(|s| matches!(s.action, Action::Place { .. }))
            );
            assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Facil));
        }
    }
}
