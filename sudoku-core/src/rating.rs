use crate::backtracking::count_solutions;
use crate::difficulty::Difficulty;
use crate::grid::Grid;
use crate::solver::LogicalSolver;

/// Classifica um puzzle pela técnica mais avançada que sua resolução lógica exige.
/// Cai para MuitoDificil quando só o backtracking resolve.
/// Retorna None se a grade não tiver solução única.
///
/// Nota: nesta fatia só existem técnicas de nível Fácil, então puzzles que
/// exigem técnicas intermediárias aparecem como MuitoDificil. Planos futuros
/// (Médio/Difícil) refinam essa classificação ao adicionar mais técnicas.
pub fn rate(grid: &Grid) -> Option<Difficulty> {
    if count_solutions(grid, 2) != 1 {
        return None;
    }
    let result = LogicalSolver::new().solve(grid);
    if result.solved {
        let level = result
            .steps
            .iter()
            .map(|s| s.level)
            .max()
            .unwrap_or(Difficulty::Facil);
        Some(level)
    } else {
        Some(Difficulty::MuitoDificil)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Cell, Grid};

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn puzzle_resolvivel_por_singles_eh_facil() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        assert_eq!(rate(&grid), Some(Difficulty::Facil));
    }

    #[test]
    fn grade_sem_solucao_unica_retorna_none() {
        assert_eq!(rate(&Grid::empty()), None);
    }
}
