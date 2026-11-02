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
        let unidades = units();
        for value in 1..=9u8 {
            // As 18 primeiras unidades são as linhas e depois as colunas.
            for linha in &unidades[..18] {
                // Conta e confere o box sem montar a lista de células: esta técnica
                // roda a cada passo do solver.
                let mut celulas = linha
                    .iter()
                    .copied()
                    .filter(|&i| candidates.get(i).contains(value));
                let Some(primeira) = celulas.next() else {
                    continue;
                };
                let b = box_of(primeira);
                let mut quantas = 1;
                let mut mesmo_box = true;
                for i in celulas {
                    quantas += 1;
                    mesmo_box &= box_of(i) == b;
                }
                if !(2..=3).contains(&quantas) || !mesmo_box {
                    continue;
                }
                let pares: Vec<(usize, u8)> = (0..81)
                    .filter(|&i| {
                        box_of(i) == b && !linha.contains(&i) && candidates.get(i).contains(value)
                    })
                    .map(|i| (i, value))
                    .collect();
                // Sem nada a remover, repetir esta dedução travaria o solver.
                if !pares.is_empty() {
                    return Some(Deduction {
                        action: Action::Eliminate(pares),
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
    fn cinco_so_em(unidade: impl Iterator<Item = usize>, celulas: &[usize]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in unidade {
            if !celulas.contains(&i) {
                c.eliminate(i, 5);
            }
        }
        c
    }

    fn linha_0(celulas: &[usize]) -> CandidateGrid {
        cinco_so_em(0..9, celulas)
    }

    #[test]
    fn elimina_o_digito_do_resto_do_box() {
        let esperado = vec![(9, 5), (10, 5), (11, 5), (18, 5), (19, 5), (20, 5)];
        let d = BoxLineReduction
            .apply(&Grid::empty(), &linha_0(&[0, 1]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
        assert_eq!(d.technique, "Box/Line Reduction");
        assert_eq!(d.level, Difficulty::Dificil);
    }

    #[test]
    fn funciona_em_coluna() {
        let c = cinco_so_em((0..9).map(|r| r * 9), &[0, 9]);
        let esperado = vec![(1, 5), (2, 5), (10, 5), (11, 5), (19, 5), (20, 5)];
        let d = BoxLineReduction.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
    }

    #[test]
    fn retorna_none_sem_alinhamento_no_box() {
        let c = linha_0(&[0, 3]);
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_com_uma_celula_so() {
        // Uma célula só é um Hidden Single, não uma Box/Line Reduction.
        let c = linha_0(&[0]);
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = linha_0(&[0, 1]);
        for i in [9, 10, 11, 18, 19, 20] {
            c.eliminate(i, 5);
        }
        assert_eq!(BoxLineReduction.apply(&Grid::empty(), &c), None);
    }
}
