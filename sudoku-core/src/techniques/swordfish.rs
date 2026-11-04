use super::{Action, Deduction, Technique, line_masks, outside_lines};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// O X-Wing com 3 linhas: em três linhas, um dígito só cabe em 2 ou 3 posições de cada,
/// e todas caem nas mesmas 3 colunas. Ele ocupa uma célula por linha, nessas colunas,
/// então sai dessas colunas nas outras linhas. O mesmo vale trocando linha por coluna.
pub struct Swordfish;

impl Technique for Swordfish {
    fn level(&self) -> Difficulty {
        Difficulty::MuitoDificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        let by_row: fn(usize, usize) -> usize = |line, pos| line * 9 + pos;
        let by_column = |column: usize, pos: usize| pos * 9 + column;
        for value in 1..=9u8 {
            for (cell, first_unit) in [(by_row, 0), (by_column, 9)] {
                if let Some(pairs) = search(candidates, value, first_unit, cell) {
                    return Some(Deduction {
                        action: Action::Eliminate(pairs),
                        technique: "Swordfish",
                        level: Difficulty::MuitoDificil,
                    });
                }
            }
        }
        None
    }
}

/// Procura o Swordfish de `value`, com `cell(linha, posição)` definindo o que é uma
/// "linha". Devolve os pares em ordem de célula, ou None se não houver padrão com algo
/// a remover.
fn search(
    candidates: &CandidateGrid,
    value: u8,
    first_unit: usize,
    cell: fn(usize, usize) -> usize,
) -> Option<Vec<(usize, u8)>> {
    let masks = line_masks(candidates, value, first_unit);
    // Uma posição só é Hidden Single; mais de 3 não cabe em 3 colunas.
    let fits = |m: u16| (2..=3).contains(&m.count_ones());
    for a in (0..9).filter(|&a| fits(masks[a])) {
        for b in ((a + 1)..9).filter(|&b| fits(masks[b])) {
            if (masks[a] | masks[b]).count_ones() > 3 {
                continue;
            }
            for c in ((b + 1)..9).filter(|&c| fits(masks[c])) {
                let columns = masks[a] | masks[b] | masks[c];
                if columns.count_ones() != 3 {
                    continue;
                }
                let pairs = outside_lines(&masks, &[a, b, c], columns, value, cell);
                // Sem nada a remover, repetir esta dedução travaria o solver.
                if !pairs.is_empty() {
                    return Some(pairs);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Em cada linha pedida, o 5 só fica nas colunas indicadas para ela.
    fn five_in_rows_at(rows: &[(usize, &[usize])]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for &(r, columns) in rows {
            for col in (0..9).filter(|col| !columns.contains(col)) {
                c.eliminate(r * 9 + col, 5);
            }
        }
        c
    }

    fn fives(cells: &[usize]) -> Vec<(usize, u8)> {
        cells.iter().map(|&i| (i, 5)).collect()
    }

    /// O 5 sai das colunas 1, 4 e 7 nas linhas 1, 2, 3, 5, 6 e 7.
    fn expected_rows_0_4_8() -> Vec<(usize, u8)> {
        fives(&[
            10, 13, 16, 19, 22, 25, 28, 31, 34, 46, 49, 52, 55, 58, 61, 64, 67, 70,
        ])
    }

    const COLUMNS_1_4_7: &[usize] = &[1, 4, 7];

    #[test]
    fn elimina_o_digito_das_colunas_nas_outras_linhas() {
        let c = five_in_rows_at(&[(0, COLUMNS_1_4_7), (4, COLUMNS_1_4_7), (8, COLUMNS_1_4_7)]);
        let d = Swordfish.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected_rows_0_4_8()));
        assert_eq!(d.technique, "Swordfish");
        assert_eq!(d.level, Difficulty::MuitoDificil);
    }

    #[test]
    fn aceita_linha_com_duas_posicoes() {
        let c = five_in_rows_at(&[(0, COLUMNS_1_4_7), (4, COLUMNS_1_4_7), (8, &[1, 7])]);
        let d = Swordfish.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected_rows_0_4_8()));
    }

    #[test]
    fn funciona_trocando_linha_por_coluna() {
        // Nas colunas 0, 4 e 8, o 5 só fica nas linhas 1, 4 e 7.
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        for col in [0, 4, 8] {
            for r in (0..9).filter(|r| ![1, 4, 7].contains(r)) {
                c.eliminate(r * 9 + col, 5);
            }
        }
        let expected = fives(&[
            10, 11, 12, 14, 15, 16, 37, 38, 39, 41, 42, 43, 64, 65, 66, 68, 69, 70,
        ]);
        let d = Swordfish.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected));
    }

    #[test]
    fn retorna_none_com_quatro_colunas() {
        let c = five_in_rows_at(&[(0, &[1, 4]), (4, &[4, 7]), (8, &[1, 2])]);
        assert_eq!(Swordfish.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_com_linha_de_uma_posicao() {
        // Com o 5 numa posição só, a linha 8 é um Hidden Single, não parte do padrão.
        let c = five_in_rows_at(&[(0, COLUMNS_1_4_7), (4, COLUMNS_1_4_7), (8, &[4])]);
        assert_eq!(Swordfish.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(Swordfish.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = five_in_rows_at(&[(0, COLUMNS_1_4_7), (4, COLUMNS_1_4_7), (8, COLUMNS_1_4_7)]);
        for (i, v) in expected_rows_0_4_8() {
            c.eliminate(i, v);
        }
        assert_eq!(Swordfish.apply(&Grid::empty(), &c), None);
    }
}
