//! Classificação de puzzles vindos de fora pelo nível que exigem.

use crate::backtracking::count_solutions;
use crate::difficulty::Difficulty;
use crate::grid::{Grid, validate};
use crate::solver::LogicalSolver;

/// Classifica um puzzle pela técnica mais avançada que sua resolução lógica exige.
/// Retorna None se a grade não tiver solução única.
///
/// MuitoDificil fica no topo da escala e quer dizer "pelo menos Muito Difícil": cobre os
/// puzzles que exigem Swordfish ou XY-Wing e também os de solução única que nem essas
/// técnicas resolvem, que só saem por tentativa e erro. O gerador nunca produz estes
/// últimos; eles só chegam por puzzles vindos de fora.
pub fn rate(grid: &Grid) -> Option<Difficulty> {
    // Provar por busca que uma grade quase vazia com conflito não tem solução pode
    // demorar demais; o conflito já responde.
    if !validate(grid).is_empty() || count_solutions(grid, 2) != 1 {
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

    fn solution() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn puzzle_resolvivel_por_singles_eh_facil() {
        let mut grid = Grid::from_line(solution()).unwrap();
        grid.set(40, Cell::Empty);
        assert_eq!(rate(&grid), Some(Difficulty::Facil));
    }

    #[test]
    fn puzzle_que_exige_par_eh_medio() {
        // Saída de generate(Medio, 42): os singles empacam e só técnicas de Médio resolvem.
        let grid = Grid::from_line(
            "94.3....12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.5...68",
        )
        .unwrap();
        assert_eq!(rate(&grid), Some(Difficulty::Medio));
    }

    #[test]
    fn puzzle_que_exige_tecnica_de_dificil_eh_dificil() {
        // Saída de generate(Dificil, 42): as técnicas de Médio empacam.
        let grid = Grid::from_line(
            ".2..574...79..6...1.............1...5...6.87.4...3.1.5.459...6.9.......26.....7..",
        )
        .unwrap();
        assert_eq!(rate(&grid), Some(Difficulty::Dificil));
    }

    #[test]
    fn puzzle_que_exige_xy_wing_eh_muito_dificil() {
        let grid = Grid::from_line(
            ".82....6.7..4...1.....9.75....1.8..6...235...4...6.....9.3..2.........3.15...9.4.",
        )
        .unwrap();
        assert_eq!(rate(&grid), Some(Difficulty::MuitoDificil));
    }

    #[test]
    fn puzzle_que_so_sai_no_chute_continua_muito_dificil() {
        // Solução única, mas nem o solver completo resolve (puzzle de Arto Inkala).
        let grid = Grid::from_line(
            "8..........36......7..9.2...5...7.......457.....1...3...1....68..85...1..9....4..",
        )
        .unwrap();
        assert!(!LogicalSolver::new().solve(&grid).solved);
        assert_eq!(rate(&grid), Some(Difficulty::MuitoDificil));
    }

    #[test]
    fn grade_sem_solucao_unica_retorna_none() {
        assert_eq!(rate(&Grid::empty()), None);
    }
}
