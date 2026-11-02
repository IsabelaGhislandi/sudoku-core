use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// Duas células de uma unidade com exatamente os mesmos 2 candidatos: esses dígitos
/// ficam com elas, seja qual for a ordem, então saem das outras células da unidade.
pub struct NakedPair;

impl Technique for NakedPair {
    fn level(&self) -> Difficulty {
        Difficulty::Medio
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for unit in units() {
            for (pos, &a) in unit.iter().enumerate() {
                let pair = candidates.get(a);
                if pair.count() != 2 {
                    continue;
                }
                for &b in &unit[pos + 1..] {
                    if candidates.get(b) != pair {
                        continue;
                    }
                    let pairs: Vec<(usize, u8)> = unit
                        .iter()
                        .filter(|&&i| i != a && i != b)
                        .flat_map(|&i| pair.digits().map(move |v| (i, v)))
                        .filter(|&(i, v)| candidates.get(i).contains(v))
                        .collect();
                    // Sem nada a remover, repetir esta dedução travaria o solver.
                    if !pairs.is_empty() {
                        return Some(Deduction {
                            action: Action::Eliminate(pairs),
                            technique: "Naked Pair",
                            level: Difficulty::Medio,
                        });
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::techniques::Action;

    fn without_other_digits(c: &mut CandidateGrid, cell: usize) {
        for v in 3..=9 {
            c.eliminate(cell, v);
        }
    }

    fn pair_in_cells_0_and_1() -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        without_other_digits(&mut c, 0);
        without_other_digits(&mut c, 1);
        c
    }

    #[test]
    fn elimina_o_par_das_outras_celulas_da_unidade() {
        // A linha 0 vem antes do box 0 na ordem das unidades.
        let expected: Vec<(usize, u8)> = (2..9).flat_map(|i| [(i, 1), (i, 2)]).collect();
        let d = NakedPair
            .apply(&Grid::empty(), &pair_in_cells_0_and_1())
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
        assert_eq!(d.technique, "Naked Pair");
        assert_eq!(d.level, Difficulty::Medio);
    }

    #[test]
    fn so_lista_candidatos_presentes() {
        // A célula 2 já perdeu o 1: esse par não pode aparecer na lista.
        let mut c = pair_in_cells_0_and_1();
        c.eliminate(2, 1);
        let d = NakedPair.apply(&Grid::empty(), &c).unwrap();
        let Action::Eliminate(pairs) = d.action else {
            panic!("deveria eliminar");
        };
        assert!(!pairs.contains(&(2, 1)));
        assert!(pairs.contains(&(2, 2)));
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(NakedPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = pair_in_cells_0_and_1();
        for i in [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20] {
            c.eliminate(i, 1);
            c.eliminate(i, 2);
        }
        assert_eq!(NakedPair.apply(&Grid::empty(), &c), None);
    }
}
