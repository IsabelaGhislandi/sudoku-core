use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Grid, box_of};

/// Numa linha (ou coluna), as células candidatas a um dígito (2 ou 3) estão todas no
/// mesmo box: o dígito da linha fica nesse box, então sai do resto dele. É o caminho
/// inverso do Pointing Pair.
pub struct BoxLineReduction;

impl Technique for BoxLineReduction {
    fn level(&self) -> Difficulty {
        Difficulty::Dificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        let all_units = units();
        for value in 1..=9u8 {
            // As 18 primeiras unidades são as linhas e depois as colunas.
            for line in &all_units[..18] {
                // Conta e confere o box sem montar a lista de células: esta técnica
                // roda a cada passo do solver.
                let mut cells = line
                    .iter()
                    .copied()
                    .filter(|&i| candidates.get(i).contains(value));
                let Some(first) = cells.next() else {
                    continue;
                };
                let b = box_of(first);
                let mut found = 1;
                let mut same_box = true;
                for i in cells {
                    found += 1;
                    same_box &= box_of(i) == b;
                }
                if !(2..=3).contains(&found) || !same_box {
                    continue;
                }
                let pairs: Vec<(usize, u8)> = (0..81)
                    .filter(|&i| {
                        box_of(i) == b && !line.contains(&i) && candidates.get(i).contains(value)
                    })
                    .map(|i| (i, value))
                    .collect();
                // Sem nada a remover, repetir esta dedução travaria o solver.
                if !pairs.is_empty() {
                    return Some(Deduction {
                        action: Action::Eliminate(pairs),
                        technique: "Box/Line Reduction",
                        level: Difficulty::Dificil,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// O 5 só cabe nas células pedidas da unidade dada.
    fn five_only_in(unit: impl Iterator<Item = usize>, cells: &[usize]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in unit {
            if !cells.contains(&i) {
                c.eliminate(i, 5);
            }
        }
        c
    }

    fn row_0(cells: &[usize]) -> CandidateGrid {
        five_only_in(0..9, cells)
    }

    #[test]
    fn elimina_o_digito_do_resto_do_box() {
        let expected = vec![(9, 5), (10, 5), (11, 5), (18, 5), (19, 5), (20, 5)];
        let d = BoxLineReduction
            .apply(&Grid::empty(), &row_0(&[0, 1]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
        assert_eq!(d.technique, "Box/Line Reduction");
        assert_eq!(d.level, Difficulty::Dificil);
    }

    #[test]
    fn funciona_em_coluna() {
        let c = five_only_in((0..9).map(|r| r * 9), &[0, 9]);
        let expected = vec![(1, 5), (2, 5), (10, 5), (11, 5), (19, 5), (20, 5)];
        let d = BoxLineReduction.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
    }

    #[test]
    fn retorna_none_sem_alinhamento_no_box() {
        let c = row_0(&[0, 3]);
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_com_uma_celula_so() {
        // Uma célula só é um Hidden Single, não uma Box/Line Reduction.
        let c = row_0(&[0]);
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = row_0(&[0, 1]);
        for i in [9, 10, 11, 18, 19, 20] {
            c.eliminate(i, 5);
        }
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }
}
