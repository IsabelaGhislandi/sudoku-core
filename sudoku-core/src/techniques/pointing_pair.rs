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
                let spots = candidates.positions(18 + b, value);
                let found = spots.count_ones() as usize;
                if !(2..=3).contains(&found) {
                    continue;
                }
                // Até 3 células num array na pilha, em vez de um Vec: esta técnica roda
                // a cada passo do solver, e alocar ali pesava no gerador.
                let mut buffer = [0usize; 3];
                let mut rest = spots;
                for slot in buffer.iter_mut().take(found) {
                    *slot = units()[18 + b][rest.trailing_zeros() as usize];
                    rest &= rest - 1; // desliga o bit mais baixo
                }
                let cells = &buffer[..found];
                let pairs = outside_box(candidates, value, b, cells, row_of, 0)
                    .or_else(|| outside_box(candidates, value, b, cells, col_of, 9));
                if let Some(pairs) = pairs {
                    return Some(Deduction {
                        action: Action::Eliminate(pairs),
                        technique: "Pointing Pair",
                        level: Difficulty::Medio,
                    });
                }
            }
        }
        None
    }
}

/// Se as células estiverem todas na mesma linha (ou coluna, conforme `line_of`),
/// devolve os pares a remover fora do box. None se não estiverem alinhadas ou se não
/// houver nada a remover: repetir a dedução travaria o solver. `first_unit` é a unidade
/// da primeira linha (0) ou coluna (9), na numeração de `units()`.
fn outside_box(
    candidates: &CandidateGrid,
    value: u8,
    b: usize,
    cells: &[usize],
    line_of: fn(usize) -> usize,
    first_unit: usize,
) -> Option<Vec<(usize, u8)>> {
    let line = line_of(cells[0]);
    if !cells.iter().all(|&i| line_of(i) == line) {
        return None;
    }
    // As células de uma linha ou coluna em `units()` estão em ordem crescente, então os
    // pares saem em ordem de célula.
    let unit = &units()[first_unit + line];
    let mut pairs = Vec::new();
    let mut spots = candidates.positions(first_unit + line, value);
    while spots != 0 {
        let i = unit[spots.trailing_zeros() as usize];
        spots &= spots - 1;
        if box_of(i) != b {
            pairs.push((i, value));
        }
    }
    (!pairs.is_empty()).then_some(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOX_0: [usize; 9] = [0, 1, 2, 9, 10, 11, 18, 19, 20];

    /// No box 0, o 5 só cabe nas células pedidas.
    fn five_only_in(cells: &[usize]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for i in BOX_0 {
            if !cells.contains(&i) {
                c.eliminate(i, 5);
            }
        }
        c
    }

    #[test]
    fn elimina_o_digito_do_resto_da_linha() {
        let expected: Vec<(usize, u8)> = (3..9).map(|i| (i, 5)).collect();
        let d = PointingPair
            .apply(&Grid::empty(), &five_only_in(&[0, 1]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
        assert_eq!(d.technique, "Pointing Pair");
        assert_eq!(d.level, Difficulty::Medio);
    }

    #[test]
    fn elimina_o_digito_do_resto_da_coluna() {
        let expected = vec![(27, 5), (36, 5), (45, 5), (54, 5), (63, 5), (72, 5)];
        let d = PointingPair
            .apply(&Grid::empty(), &five_only_in(&[0, 9]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
    }

    #[test]
    fn aceita_tres_celulas_alinhadas() {
        let expected: Vec<(usize, u8)> = (3..9).map(|i| (i, 5)).collect();
        let d = PointingPair
            .apply(&Grid::empty(), &five_only_in(&[0, 1, 2]))
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
    }

    #[test]
    fn retorna_none_sem_alinhamento() {
        let c = five_only_in(&[0, 10]);
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_com_uma_celula_so() {
        // Uma célula só é um Hidden Single, não um Pointing Pair.
        let c = five_only_in(&[0]);
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = five_only_in(&[0, 1]);
        for i in 3..9 {
            c.eliminate(i, 5);
        }
        assert_eq!(PointingPair.apply(&Grid::empty(), &c), None);
    }
}
