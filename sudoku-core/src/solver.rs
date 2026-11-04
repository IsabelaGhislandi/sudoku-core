use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};
use crate::techniques::{
    Action, BoxLineReduction, Deduction, HiddenPair, HiddenSingle, HiddenTriple, NakedPair,
    NakedSingle, NakedTriple, PointingPair, Swordfish, Technique, XWing, XYWing, units,
};

pub struct LogicalSolver {
    techniques: Vec<Box<dyn Technique>>,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub grid: Grid,
    pub steps: Vec<Deduction>,
    pub solved: bool,
}

/// Todas as técnicas, da mais simples para a mais difícil. O solver sempre tenta
/// do início da lista, então só recorre a uma técnica difícil quando as simples
/// empacam. É isso que faz o nível do puzzle refletir o que ele realmente exige.
fn all_techniques() -> Vec<Box<dyn Technique>> {
    vec![
        // Os dois singles abrem a lista em todo nível: `hardest_step` depende disso.
        Box::new(NakedSingle),
        Box::new(HiddenSingle),
        Box::new(NakedPair),
        Box::new(HiddenPair),
        Box::new(PointingPair),
        Box::new(BoxLineReduction),
        Box::new(NakedTriple),
        Box::new(HiddenTriple),
        Box::new(XWing),
        Box::new(Swordfish),
        Box::new(XYWing),
    ]
}

impl LogicalSolver {
    /// Solver com todas as técnicas implementadas.
    pub fn new() -> Self {
        LogicalSolver {
            techniques: all_techniques(),
        }
    }

    /// Solver só com as técnicas até `level`. O gerador de Fácil depende disso: se
    /// técnicas de Médio entrassem nele, todos os puzzles Fáceis mudariam.
    pub fn up_to(level: Difficulty) -> Self {
        LogicalSolver {
            techniques: all_techniques()
                .into_iter()
                .filter(|t| t.level() <= level)
                .collect(),
        }
    }

    /// Primeira dedução lógica disponível, ou None se empacar.
    pub fn next_step(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for technique in &self.techniques {
            if let Some(d) = technique.apply(grid, candidates) {
                return Some(d);
            }
        }
        None
    }

    /// Aplica técnicas repetidamente até resolver ou empacar.
    pub fn solve(&self, grid: &Grid) -> SolveResult {
        let mut work = grid.clone();
        let mut candidates = CandidateGrid::from_grid(&work);
        let mut steps = Vec::new();
        loop {
            if work.is_complete() {
                return SolveResult {
                    grid: work,
                    steps,
                    solved: true,
                };
            }
            match self.next_step(&work, &candidates) {
                Some(d) => {
                    match &d.action {
                        Action::Place { index, value } => {
                            work.set(*index, Cell::Filled(*value));
                            candidates.place(*index, *value);
                        }
                        Action::Eliminate(pairs) => {
                            for &(index, value) in pairs {
                                candidates.eliminate(index, value);
                            }
                        }
                    }
                    steps.push(d);
                }
                None => {
                    return SolveResult {
                        grid: work,
                        steps,
                        solved: false,
                    };
                }
            }
        }
    }
}

impl LogicalSolver {
    /// O nível da técnica mais difícil que `solve` usaria, ou `None` se ele empacaria. É o
    /// que o gerador pergunta milhares de vezes por puzzle, então responde sem montar a
    /// lista de passos e colocando os singles em lote.
    ///
    /// Dá sempre a mesma resposta que `solve`, desde que a grade seja parte de uma solução
    /// válida (é o caso de tudo que o gerador cava). Nessa condição, um single disponível
    /// continua disponível até ser colocado, então qualquer ordem de colocar os singles
    /// chega à mesma grade. E `solve` só passa da terceira técnica quando os dois singles
    /// empacam: as técnicas mais difíceis veem aqui exatamente as grades que veriam lá.
    pub(crate) fn hardest_step(&self, grid: &Grid) -> Option<Difficulty> {
        let mut work = grid.clone();
        let mut candidates = CandidateGrid::from_grid(&work);
        let mut hardest = Difficulty::Facil;
        loop {
            place_singles(&mut work, &mut candidates);
            if work.is_complete() {
                return Some(hardest);
            }
            // As duas primeiras técnicas são os singles, que acabaram de empacar.
            let deduction = self.techniques[2..]
                .iter()
                .find_map(|t| t.apply(&work, &candidates))?;
            hardest = hardest.max(deduction.level);
            match deduction.action {
                Action::Place { index, value } => {
                    work.set(index, Cell::Filled(value));
                    candidates.place(index, value);
                }
                Action::Eliminate(pairs) => {
                    for (index, value) in pairs {
                        candidates.eliminate(index, value);
                    }
                }
            }
        }
    }
}

/// Coloca Naked Singles e Hidden Singles até não sobrar nenhum, em qualquer ordem. Só serve
/// para grades que são parte de uma solução válida (veja `hardest_step`).
fn place_singles(work: &mut Grid, candidates: &mut CandidateGrid) {
    let place = |work: &mut Grid, candidates: &mut CandidateGrid, i: usize, bits: u16| {
        let value = bits.trailing_zeros() as u8;
        work.set(i, Cell::Filled(value));
        candidates.place(i, value);
    };
    loop {
        let mut progress = false;
        for i in 0..81 {
            let bits = candidates.get(i).bits();
            if bits.count_ones() == 1 {
                place(work, candidates, i, bits);
                progress = true;
            }
        }
        for unit in units() {
            // Dígitos que cabem em exatamente uma célula da unidade: `once` junta os que
            // apareceram ao menos uma vez, `twice` os que apareceram de novo.
            let (mut once, mut twice) = (0u16, 0u16);
            for &i in unit {
                let bits = candidates.get(i).bits();
                twice |= once & bits;
                once |= bits;
            }
            let mut hidden = once & !twice;
            while hidden != 0 {
                let digit = hidden.isolate_lowest_one(); // só o bit mais baixo
                hidden &= hidden - 1;
                // A célula pode ter perdido o dígito para outro single desta unidade.
                if let Some(&i) = unit
                    .iter()
                    .find(|&&i| candidates.get(i).bits() & digit != 0)
                {
                    place(work, candidates, i, digit);
                    progress = true;
                }
            }
        }
        if !progress {
            return;
        }
    }
}

impl Default for LogicalSolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn solution() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn resolve_grade_com_uma_celula_vazia() {
        let mut grid = Grid::from_line(solution()).unwrap();
        grid.set(40, Cell::Empty);
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert_eq!(result.grid.to_line(), solution());
        assert_eq!(result.steps.len(), 1);
    }

    #[test]
    fn empaca_quando_nao_ha_tecnica_aplicavel() {
        let grid = Grid::empty();
        let result = LogicalSolver::new().solve(&grid);
        assert!(!result.solved);
        assert!(result.steps.is_empty());
    }

    #[test]
    fn solve_termina_em_grade_com_conflito() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(1, Cell::Filled(5));
        let _ = LogicalSolver::new().solve(&grid); // não pode dar panic nem travar
        assert_eq!(crate::rating::rate(&grid), None);
    }

    #[test]
    fn empaca_em_celula_sem_candidatos() {
        // Linha 0 com 1 a 8 e um 9 mais abaixo na coluna 0: a célula 0 fica sem candidatos.
        let mut grid = Grid::from_line(&format!(".12345678{}", ".".repeat(72))).unwrap();
        grid.set(27, Cell::Filled(9));
        assert!(!LogicalSolver::new().solve(&grid).solved);
    }

    #[test]
    fn grade_completa_nao_tem_passos() {
        let grid = Grid::from_line(solution()).unwrap();
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert!(result.steps.is_empty());
        assert_eq!(crate::rating::rate(&grid), Some(Difficulty::Facil));
    }

    #[test]
    fn up_to_facil_resolve_puzzle_de_singles() {
        let mut grid = Grid::from_line(solution()).unwrap();
        grid.set(40, Cell::Empty);
        assert!(LogicalSolver::up_to(Difficulty::Facil).solve(&grid).solved);
    }

    #[test]
    fn up_to_facil_nao_usa_tecnica_de_medio() {
        // Sem singles disponíveis: só o Naked Pair das células 0 e 1 se aplica.
        let grid = Grid::empty();
        let mut candidates = CandidateGrid::from_grid(&grid);
        for cell in [0, 1] {
            for v in 3..=9 {
                candidates.eliminate(cell, v);
            }
        }
        assert_eq!(
            LogicalSolver::up_to(Difficulty::Facil).next_step(&grid, &candidates),
            None
        );
        let d = LogicalSolver::new().next_step(&grid, &candidates).unwrap();
        assert_eq!(d.technique, "Naked Pair");
    }

    #[test]
    fn so_as_tecnicas_de_medio_resolvem_o_puzzle() {
        // Saída de generate(Medio, 42).
        let grid = Grid::from_line(
            "94.3....12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.5...68",
        )
        .unwrap();
        assert!(!LogicalSolver::up_to(Difficulty::Facil).solve(&grid).solved);
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert_eq!(
            result.grid.to_line(),
            "948352671251967483763481259127836594834795126695214837586179342372648915419523768"
        );
    }

    #[test]
    fn so_as_tecnicas_de_dificil_resolvem_o_puzzle() {
        // Saída de generate(Dificil, 42).
        let grid = Grid::from_line(
            ".2..574...79..6...1.............1...5...6.87.4...3.1.5.459...6.9.......26.....7..",
        )
        .unwrap();
        assert!(!LogicalSolver::up_to(Difficulty::Medio).solve(&grid).solved);
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert_eq!(
            result.grid.to_line(),
            "826357419379416258154892637732581946591264873468739125245978361917643582683125794"
        );
    }

    /// Exige XY-Wing: achado no experimento da spec do Muito Difícil.
    const NEEDS_XY_WING: &str =
        ".82....6.7..4...1.....9.75....1.8..6...235...4...6.....9.3..2.........3.15...9.4.";
    /// Exige Swordfish e XY-Wing: também do experimento.
    const NEEDS_SWORDFISH: &str =
        "46..1.....5.9.64.3..3..8...2..5.9.....1....9.......34...........8.4..925.398.....";

    /// Confere que só o solver completo resolve o puzzle, e que usa a técnica pedida.
    fn assert_needs_very_hard(line: &str, technique: &str) {
        let grid = Grid::from_line(line).unwrap();
        assert!(
            !LogicalSolver::up_to(Difficulty::Dificil)
                .solve(&grid)
                .solved
        );
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert!(crate::grid::validate(&result.grid).is_empty());
        assert!(
            result.steps.iter().any(|s| s.technique == technique),
            "deveria usar {technique}"
        );
    }

    #[test]
    fn so_as_tecnicas_de_muito_dificil_resolvem_o_puzzle() {
        assert_needs_very_hard(NEEDS_XY_WING, "XY-Wing");
    }

    #[test]
    fn swordfish_entra_na_resolucao() {
        assert_needs_very_hard(NEEDS_SWORDFISH, "Swordfish");
    }

    #[test]
    fn up_to_dificil_nao_usa_tecnica_de_muito_dificil() {
        for line in [NEEDS_XY_WING, NEEDS_SWORDFISH] {
            let grid = Grid::from_line(line).unwrap();
            let result = LogicalSolver::up_to(Difficulty::Dificil).solve(&grid);
            assert!(
                result
                    .steps
                    .iter()
                    .all(|s| s.level != Difficulty::MuitoDificil)
            );
        }
    }

    /// O que `hardest_step` precisa devolver, calculado pelo `solve` completo.
    fn expected_hardest(solver: &LogicalSolver, grid: &Grid) -> Option<Difficulty> {
        let result = solver.solve(grid);
        result.solved.then(|| {
            result
                .steps
                .iter()
                .map(|s| s.level)
                .max()
                .unwrap_or(Difficulty::Facil)
        })
    }

    #[test]
    fn hardest_step_concorda_com_solve() {
        // Os dois puzzles de Muito Difícil e, de cada um, as grades com um número a menos:
        // umas ainda resolvem, outras empacam, em todos os níveis de solver.
        let mut grids = Vec::new();
        for line in [NEEDS_XY_WING, NEEDS_SWORDFISH] {
            let grid = Grid::from_line(line).unwrap();
            for i in (0..81).filter(|&i| grid.get(i) != Cell::Empty) {
                let mut fewer = grid.clone();
                fewer.set(i, Cell::Empty);
                grids.push(fewer);
            }
            grids.push(grid);
        }
        let mut solved = 0;
        for level in [
            Difficulty::Facil,
            Difficulty::Medio,
            Difficulty::Dificil,
            Difficulty::MuitoDificil,
        ] {
            let solver = LogicalSolver::up_to(level);
            for grid in &grids {
                let expected = expected_hardest(&solver, grid);
                solved += usize::from(expected.is_some());
                assert_eq!(
                    solver.hardest_step(grid),
                    expected,
                    "{level:?} {}",
                    grid.to_line()
                );
            }
        }
        assert!(solved > 0, "nenhuma grade resolvida: o teste não diz nada");
    }

    #[test]
    fn hardest_step_de_grade_completa_eh_facil() {
        let grid = Grid::from_line(solution()).unwrap();
        assert_eq!(
            LogicalSolver::new().hardest_step(&grid),
            Some(Difficulty::Facil)
        );
    }

    #[test]
    fn next_step_devolve_primeira_deducao() {
        let mut grid = Grid::from_line(solution()).unwrap();
        grid.set(40, Cell::Empty);
        let candidates = CandidateGrid::from_grid(&grid);
        let step = LogicalSolver::new().next_step(&grid, &candidates).unwrap();
        assert_eq!(
            step.action,
            Action::Place {
                index: 40,
                value: 5
            }
        );
    }
}
