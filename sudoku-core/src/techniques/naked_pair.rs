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
                let par = candidates.get(a);
                if par.count() != 2 {
                    continue;
                }
                for &b in &unit[pos + 1..] {
                    if candidates.get(b) != par {
                        continue;
                    }
                    let pares: Vec<(usize, u8)> = unit
                        .iter()
                        .filter(|&&i| i != a && i != b)
                        .flat_map(|&i| par.digits().map(move |v| (i, v)))
                        .filter(|&(i, v)| candidates.get(i).contains(v))
                        .collect();
                    // Sem nada a remover, repetir esta dedução travaria o solver.
                    if !pares.is_empty() {
                        return Some(Deduction {
                            action: Action::Eliminate(pares),
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

    fn sem_outros_digitos(c: &mut CandidateGrid, celula: usize) {
        for v in 3..=9 {
            c.eliminate(celula, v);
        }
    }

    fn par_nas_celulas_0_e_1() -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        sem_outros_digitos(&mut c, 0);
        sem_outros_digitos(&mut c, 1);
        c
    }

    #[test]
    fn elimina_o_par_das_outras_celulas_da_unidade() {
        // A linha 0 vem antes do box 0 na ordem das unidades.
        let esperado: Vec<(usize, u8)> = (2..9).flat_map(|i| [(i, 1), (i, 2)]).collect();
        let d = NakedPair
            .apply(&Grid::empty(), &par_nas_celulas_0_e_1())
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
        assert_eq!(d.technique, "Naked Pair");
        assert_eq!(d.level, Difficulty::Medio);
    }

    #[test]
    fn so_lista_candidatos_presentes() {
        // A célula 2 já perdeu o 1: esse par não pode aparecer na lista.
        let mut c = par_nas_celulas_0_e_1();
        c.eliminate(2, 1);
        let d = NakedPair.apply(&Grid::empty(), &c).unwrap();
        let Action::Eliminate(pares) = d.action else {
            panic!("deveria eliminar");
        };
        assert!(!pares.contains(&(2, 1)));
        assert!(pares.contains(&(2, 2)));
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(NakedPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = par_nas_celulas_0_e_1();
        for i in [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20] {
            c.eliminate(i, 1);
            c.eliminate(i, 2);
        }
        assert_eq!(NakedPair.apply(&Grid::empty(), &c), None);
    }
}
