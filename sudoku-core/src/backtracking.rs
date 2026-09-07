use crate::grid::{Cell, Grid};

/// Conta soluções até o limite `limit`. Use limit=2 para testar unicidade (== 1).
pub fn count_solutions(grid: &Grid, limit: usize) -> usize {
    let mut work = grid.clone();
    let mut count = 0;
    solve_recursive(&mut work, limit, &mut count);
    count
}

fn solve_recursive(grid: &mut Grid, limit: usize, count: &mut usize) {
    if *count >= limit {
        return;
    }
    let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
    match next {
        None => {
            *count += 1; // grade completa encontrada
        }
        Some(index) => {
            for value in 1..=9u8 {
                if grid.can_place(index, value) {
                    grid.set(index, Cell::Filled(value));
                    solve_recursive(grid, limit, count);
                    grid.set(index, Cell::Empty);
                    if *count >= limit {
                        return;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn grade_resolvida_tem_uma_solucao() {
        let grid = Grid::from_line(solucao()).unwrap();
        assert_eq!(count_solutions(&grid, 2), 1);
    }

    #[test]
    fn uma_celula_vazia_continua_unica() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        assert_eq!(count_solutions(&grid, 2), 1);
    }

    #[test]
    fn grade_vazia_tem_muitas_solucoes() {
        // limite 2: para assim que encontra a segunda
        assert_eq!(count_solutions(&Grid::empty(), 2), 2);
    }
}
