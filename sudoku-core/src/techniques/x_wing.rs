use super::{Action, Deduction, Technique, line_masks};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// Em duas linhas, um dígito só cabe nas mesmas 2 colunas: ele ocupa os cantos de um
/// retângulo, uma vez em cada coluna, então sai dessas colunas nas outras linhas. O
/// mesmo vale trocando linha por coluna.
pub struct XWing;

impl Technique for XWing {
    fn level(&self) -> Difficulty {
        Difficulty::Dificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        // Primeiro por linhas, depois por colunas, como as outras técnicas.
        let by_row = |line: usize, pos: usize| line * 9 + pos;
        let by_column = |column: usize, pos: usize| pos * 9 + column;
        for value in 1..=9u8 {
            for cell in [by_row, by_column] {
                if let Some(pairs) = search(candidates, value, cell) {
                    return Some(Deduction {
                        action: Action::Eliminate(pairs),
                        technique: "X-Wing",
                        level: Difficulty::Dificil,
                    });
                }
            }
        }
        None
    }
}

/// Procura o X-Wing de `value` com `celula(linha, posição)` definindo o que é uma
/// "linha" (linha ou coluna do tabuleiro). Devolve os pares em ordem de célula, ou None
/// se não houver padrão com algo a remover.
fn search(
    candidates: &CandidateGrid,
    value: u8,
    cell: fn(usize, usize) -> usize,
) -> Option<Vec<(usize, u8)>> {
    let masks = line_masks(candidates, value, cell);
    for a in 0..9 {
        if masks[a].count_ones() != 2 {
            continue;
        }
        for b in (a + 1)..9 {
            if masks[b] != masks[a] {
                continue;
            }
            let mut pairs: Vec<(usize, u8)> = (0..9)
                .filter(|&l| l != a && l != b)
                .flat_map(|l| (0..9).map(move |p| (l, p)))
                .filter(|&(_, p)| masks[a] & (1 << p) != 0)
                .map(|(l, p)| cell(l, p))
                .filter(|&i| candidates.get(i).contains(value))
                .map(|i| (i, value))
                .collect();
            pairs.sort_unstable();
            // Sem nada a remover, repetir esta dedução travaria o solver.
            if !pairs.is_empty() {
                return Some(pairs);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Nas linhas pedidas, o 5 só fica nas colunas pedidas.
    fn five_in_rows(rows: &[usize], columns: &[usize]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for &r in rows {
            for col in (0..9).filter(|col| !columns.contains(col)) {
                c.eliminate(r * 9 + col, 5);
            }
        }
        c
    }

    fn expected_rows_0_and_4() -> Vec<(usize, u8)> {
        [10, 16, 19, 25, 28, 34, 46, 52, 55, 61, 64, 70, 73, 79]
            .into_iter()
            .map(|i| (i, 5))
            .collect()
    }

    #[test]
    fn elimina_o_digito_das_colunas_nas_outras_linhas() {
        let c = five_in_rows(&[0, 4], &[1, 7]);
        let d = XWing.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected_rows_0_and_4()));
        assert_eq!(d.technique, "X-Wing");
        assert_eq!(d.level, Difficulty::Dificil);
    }

    #[test]
    fn funciona_trocando_linha_por_coluna() {
        // Nas colunas 0 e 4, o 5 só fica nas linhas 1 e 7.
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for col in [0, 4] {
            for r in (0..9).filter(|r| ![1, 7].contains(r)) {
                c.eliminate(r * 9 + col, 5);
            }
        }
        let expected: Vec<(usize, u8)> = [10, 11, 12, 14, 15, 16, 17, 64, 65, 66, 68, 69, 70, 71]
            .into_iter()
            .map(|i| (i, 5))
            .collect();
        let d = XWing.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
    }

    #[test]
    fn retorna_none_com_tres_colunas_numa_linha() {
        // Linha 4 com o 5 também na coluna 3: as duas linhas não batem.
        let mut c = five_in_rows(&[0], &[1, 7]);
        for col in [0, 2, 4, 5, 6, 8] {
            c.eliminate(36 + col, 5);
        }
        assert_eq!(XWing.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(XWing.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = five_in_rows(&[0, 4], &[1, 7]);
        for (i, v) in expected_rows_0_and_4() {
            c.eliminate(i, v);
        }
        assert_eq!(XWing.apply(&Grid::empty(), &c), None);
    }
}
