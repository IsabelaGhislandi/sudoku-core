use super::{Action, Deduction, Technique};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Grid, box_of, col_of, row_of};

/// Um pivô com exatamente {x, y} enxerga duas asas, uma {x, z} e outra {y, z}. Se o pivô
/// for x, a primeira asa é z; se for y, a segunda é z. Nos dois casos uma das asas é z,
/// então z sai de toda célula que enxerga as duas asas.
pub struct XYWing;

impl Technique for XYWing {
    fn level(&self) -> Difficulty {
        Difficulty::MuitoDificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        // Só células com 2 candidatos participam: filtrar antes evita varrer 81³ trios.
        let bivalue: Vec<usize> = (0..81)
            .filter(|&i| candidates.get(i).count() == 2)
            .collect();
        for &pivot in &bivalue {
            let mut digits = candidates.get(pivot).digits();
            let (Some(x), Some(y)) = (digits.next(), digits.next()) else {
                continue;
            };
            for &wing_x in bivalue.iter().filter(|&&w| sees(pivot, w)) {
                let wing = candidates.get(wing_x);
                if !wing.contains(x) || wing.contains(y) {
                    continue;
                }
                let Some(z) = wing.digits().find(|&v| v != x) else {
                    continue;
                };
                for &wing_y in bivalue.iter().filter(|&&w| w != wing_x && sees(pivot, w)) {
                    let other = candidates.get(wing_y);
                    if !other.contains(y) || !other.contains(z) {
                        continue;
                    }
                    let pairs: Vec<(usize, u8)> = (0..81)
                        .filter(|&i| i != wing_x && i != wing_y)
                        .filter(|&i| sees(i, wing_x) && sees(i, wing_y))
                        .filter(|&i| candidates.get(i).contains(z))
                        .map(|i| (i, z))
                        .collect();
                    // Sem nada a remover, repetir esta dedução travaria o solver.
                    if !pairs.is_empty() {
                        return Some(Deduction {
                            action: Action::Eliminate(pairs),
                            technique: "XY-Wing",
                            level: Difficulty::MuitoDificil,
                        });
                    }
                }
            }
        }
        None
    }
}

/// Duas células diferentes se enxergam quando dividem linha, coluna ou box.
fn sees(a: usize, b: usize) -> bool {
    a != b && (row_of(a) == row_of(b) || col_of(a) == col_of(b) || box_of(a) == box_of(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Grade vazia em que cada célula listada fica só com os candidatos indicados.
    fn with_cells(cells: &[(usize, &[u8])]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for &(i, keep) in cells {
            for v in (1..=9).filter(|v| !keep.contains(v)) {
                c.eliminate(i, v);
            }
        }
        c
    }

    fn threes(cells: &[usize]) -> Vec<(usize, u8)> {
        cells.iter().map(|&i| (i, 3)).collect()
    }

    /// Pivô 0 = {1,2}; asa 4 = {1,3}, na linha do pivô; asa 19 = {2,3}, no box do pivô.
    fn main_case() -> CandidateGrid {
        with_cells(&[(0, &[1, 2]), (4, &[1, 3]), (19, &[2, 3])])
    }

    #[test]
    fn elimina_z_das_celulas_que_enxergam_as_duas_asas() {
        let d = XYWing.apply(&Grid::empty(), &main_case()).unwrap();
        assert_eq!(d.action, Action::Eliminate(threes(&[1, 2, 21, 22, 23])));
        assert_eq!(d.technique, "XY-Wing");
        assert_eq!(d.level, Difficulty::MuitoDificil);
    }

    #[test]
    fn asas_que_se_enxergam_nao_perdem_o_digito() {
        // As asas 1 e 9 estão no box do pivô e se enxergam, mas o 3 delas fica.
        let c = with_cells(&[(0, &[1, 2]), (1, &[1, 3]), (9, &[2, 3])]);
        let d = XYWing.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(
            d.action,
            Action::Eliminate(threes(&[2, 10, 11, 18, 19, 20]))
        );
    }

    #[test]
    fn retorna_none_quando_as_asas_nao_compartilham_z() {
        let c = with_cells(&[(0, &[1, 2]), (4, &[1, 3]), (19, &[2, 4])]);
        assert_eq!(XYWing.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_a_asa_repete_o_pivo() {
        // Três células {1,2}: isso não é XY-Wing.
        let c = with_cells(&[(0, &[1, 2]), (4, &[1, 2]), (19, &[1, 2])]);
        assert_eq!(XYWing.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(XYWing.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = main_case();
        for (i, v) in threes(&[1, 2, 21, 22, 23]) {
            c.eliminate(i, v);
        }
        assert_eq!(XYWing.apply(&Grid::empty(), &c), None);
    }
}
