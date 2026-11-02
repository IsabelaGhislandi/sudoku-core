use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::{Grid, box_of, col_of, row_of};

/// Dentro de um box, as células candidatas a um dígito (2 ou 3) estão todas na mesma
/// linha ou coluna: o dígito do box fica nessa linha ou coluna, então sai do resto dela.
pub struct PointingPair;

impl Technique for PointingPair {
    fn level(&self) -> Difficulty {
        Difficulty::Medio
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for value in 1..=9u8 {
            for b in 0..9 {
                // Até 3 células num array na pilha, em vez de um Vec: esta técnica roda
                // a cada passo do solver, e alocar ali pesava no gerador.
                let mut buffer = [0usize; 3];
                let mut quantas = 0;
                for &i in &units()[18 + b] {
                    if candidates.get(i).contains(value) {
                        if quantas < 3 {
                            buffer[quantas] = i;
                        }
                        quantas += 1;
                    }
                }
                if !(2..=3).contains(&quantas) {
                    continue;
                }
                let celulas = &buffer[..quantas];
                let pares = fora_do_box(candidates, value, b, celulas, row_of)
                    .or_else(|| fora_do_box(candidates, value, b, celulas, col_of));
                if let Some(pares) = pares {
                    return Some(Deduction {
                        action: Action::Eliminate(pares),
                        technique: "Pointing Pair",
                        level: Difficulty::Medio,
                    });
                }
            }
        }
        None
    }
}

/// Se as células estiverem todas na mesma linha (ou coluna, conforme `linha_de`),
/// devolve os pares a remover fora do box. None se não estiverem alinhadas ou se não
/// houver nada a remover: repetir a dedução travaria o solver.
fn fora_do_box(
    candidates: &CandidateGrid,
    value: u8,
    b: usize,
    celulas: &[usize],
    linha_de: fn(usize) -> usize,
) -> Option<Vec<(usize, u8)>> {
    let linha = linha_de(celulas[0]);
    if !celulas.iter().all(|&i| linha_de(i) == linha) {
        return None;
    }
    let pares: Vec<(usize, u8)> = (0..81)
        .filter(|&i| linha_de(i) == linha && box_of(i) != b && candidates.get(i).contains(value))
        .map(|i| (i, value))
        .collect();
    (!pares.is_empty()).then_some(pares)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX_0: [usize; 9] = [0, 1, 2, 9, 10, 11, 18, 19, 20];

    /// No box 0, o 5 só cabe nas células pedidas.
    fn cinco_so_em(celulas: &[usize]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in BOX_0 {
            if !celulas.contains(&i) {
                c.eliminate(i, 5);
            }
        }
        c
    }

    #[test]
    fn elimina_o_digito_do_resto_da_linha() {
        let esperado: Vec<(usize, u8)> = (3..9).map(|i| (i, 5)).collect();
        let d = PointingPair
            .apply(&Grid::empty(), &cinco_so_em(&[0, 1]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
        assert_eq!(d.technique, "Pointing Pair");
        assert_eq!(d.level, Difficulty::Medio);
    }

    #[test]
    fn elimina_o_digito_do_resto_da_coluna() {
        let esperado = vec![(27, 5), (36, 5), (45, 5), (54, 5), (63, 5), (72, 5)];
        let d = PointingPair
            .apply(&Grid::empty(), &cinco_so_em(&[0, 9]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
    }

    #[test]
    fn aceita_tres_celulas_alinhadas() {
        let esperado: Vec<(usize, u8)> = (3..9).map(|i| (i, 5)).collect();
        let d = PointingPair
            .apply(&Grid::empty(), &cinco_so_em(&[0, 1, 2]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado));
    }

    #[test]
    fn retorna_none_sem_alinhamento() {
        let c = cinco_so_em(&[0, 10]);
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_com_uma_celula_so() {
        // Uma célula só é um Hidden Single, não um Pointing Pair.
        let c = cinco_so_em(&[0]);
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = cinco_so_em(&[0, 1]);
        for i in 3..9 {
            c.eliminate(i, 5);
        }
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }
}
