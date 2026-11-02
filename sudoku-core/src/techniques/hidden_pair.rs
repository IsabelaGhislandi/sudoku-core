use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// Dois dígitos que, numa unidade, só cabem nas mesmas 2 células: essas células
/// ficam com eles, então os outros candidatos delas saem.
pub struct HiddenPair;

impl Technique for HiddenPair {
    fn level(&self) -> Difficulty {
        Difficulty::Medio
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for unit in units() {
            // posicoes[d]: máscara das posições da unidade onde o dígito d cabe.
            // Comparar máscaras evita montar e comparar listas de células.
            let mut posicoes = [0u16; 10];
            for (pos, &i) in unit.iter().enumerate() {
                for v in candidates.get(i).values() {
                    posicoes[v as usize] |= 1 << pos;
                }
            }
            for a in 1..=9u8 {
                let onde = posicoes[a as usize];
                if onde.count_ones() != 2 {
                    continue;
                }
                for b in (a + 1)..=9u8 {
                    if posicoes[b as usize] != onde {
                        continue;
                    }
                    let pares: Vec<(usize, u8)> = unit
                        .iter()
                        .enumerate()
                        .filter(|&(pos, _)| onde & (1 << pos) != 0)
                        .flat_map(|(_, &i)| {
                            candidates.get(i).values().into_iter().map(move |v| (i, v))
                        })
                        .filter(|&(_, v)| v != a && v != b)
                        .collect();
                    // Sem nada a remover, repetir esta dedução travaria o solver.
                    if !pares.is_empty() {
                        return Some(Deduction {
                            action: Action::Eliminate(pares),
                            technique: "Hidden Pair",
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

    /// O 1 e o 2 só cabem nas células 0 e 1 da linha 0.
    fn fixture() -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in 2..9 {
            c.eliminate(i, 1);
            c.eliminate(i, 2);
        }
        c
    }

    #[test]
    fn elimina_os_outros_candidatos_das_duas_celulas() {
        let esperado: Vec<(usize, u8)> = [0, 1]
            .into_iter()
            .flat_map(|i| (3..=9).map(move |v| (i, v)))
            .collect();
        let d = HiddenPair.apply(&Grid::empty(), &fixture()).unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
        assert_eq!(d.technique, "Hidden Pair");
        assert_eq!(d.level, Difficulty::Medio);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(HiddenPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_as_celulas_ja_so_tem_o_par() {
        let mut c = fixture();
        for celula in [0, 1] {
            for v in 3..=9 {
                c.eliminate(celula, v);
            }
        }
        assert_eq!(HiddenPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn nao_confunde_digitos_em_celulas_diferentes() {
        // Na linha 0, o 1 só cabe em {0, 1} e o 2 só cabe em {0, 2}.
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in 2..9 {
            c.eliminate(i, 1);
        }
        for i in [1, 3, 4, 5, 6, 7, 8] {
            c.eliminate(i, 2);
        }
        assert_eq!(HiddenPair.apply(&Grid::empty(), &c), None);
    }
}
