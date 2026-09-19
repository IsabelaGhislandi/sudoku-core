use crate::grid::{Cell, Grid};
use crate::techniques::{Deduction, HiddenSingle, NakedSingle, Technique};

pub struct LogicalSolver {
    techniques: Vec<Box<dyn Technique>>,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub grid: Grid,
    pub steps: Vec<Deduction>,
    pub solved: bool,
}

impl LogicalSolver {
    /// Solver com as técnicas disponíveis na fatia "Fácil".
    pub fn new() -> Self {
        LogicalSolver {
            techniques: vec![Box::new(NakedSingle), Box::new(HiddenSingle)],
        }
    }

    /// Primeira jogada logicamente dedutível, ou None se empacar.
    pub fn next_step(&self, grid: &Grid) -> Option<Deduction> {
        for technique in &self.techniques {
            if let Some(d) = technique.apply(grid) {
                return Some(d);
            }
        }
        None
    }

    /// Aplica técnicas repetidamente até resolver ou empacar.
    pub fn solve(&self, grid: &Grid) -> SolveResult {
        let mut work = grid.clone();
        let mut steps = Vec::new();
        loop {
            if work.is_complete() {
                return SolveResult {
                    grid: work,
                    steps,
                    solved: true,
                };
            }
            match self.next_step(&work) {
                Some(d) => {
                    work.set(d.index, Cell::Filled(d.value));
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

impl Default for LogicalSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Próxima jogada logicamente dedutível — reutiliza as técnicas do solver.
/// Alimenta a feature de dica do módulo Game (futuro).
pub fn next_hint(grid: &Grid) -> Option<Deduction> {
    LogicalSolver::new().next_step(grid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn resolve_grade_com_uma_celula_vazia() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert_eq!(result.grid.to_line(), solucao());
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
    fn next_step_devolve_primeira_deducao() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        let step = LogicalSolver::new().next_step(&grid).unwrap();
        assert_eq!(step.index, 40);
    }
}
