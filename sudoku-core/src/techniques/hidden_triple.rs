use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// Três dígitos que, numa unidade, só cabem dentro das mesmas 3 células: essas células
/// ficam com eles, então os outros candidatos delas saem.
pub struct HiddenTriple;

impl Technique for HiddenTriple {
    fn level(&self) -> Difficulty {
        Difficulty::Dificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for (u, unit) in units().iter().enumerate() {
            // posicoes[d]: máscara das posições da unidade onde o dígito d cabe, já mantida
            // pelo CandidateGrid. Comparar máscaras evita montar e comparar listas de células.
            let positions: [u16; 10] = std::array::from_fn(|d| candidates.positions(u, d as u8));
            // Só dígitos com 1 a 3 posições podem formar um triple. Com mais de 3, a
            // união nunca teria 3 bits; pular antes poupa combinações. Com 0, o dígito
            // já foi colocado na unidade: contá-lo faria um par parecer triple e
            // removeria candidatos verdadeiros.
            let fits = |d: u8| (1..=3).contains(&positions[d as usize].count_ones());
            for a in (1..=9u8).filter(|&d| fits(d)) {
                for b in ((a + 1)..=9u8).filter(|&d| fits(d)) {
                    for c in ((b + 1)..=9u8).filter(|&d| fits(d)) {
                        let trio = [a, b, c];
                        let spots = trio.iter().fold(0, |m, &d| m | positions[d as usize]);
                        if spots.count_ones() != 3 {
                            continue;
                        }
                        let pairs: Vec<(usize, u8)> = unit
                            .iter()
                            .enumerate()
                            .filter(|&(pos, _)| spots & (1 << pos) != 0)
                            .flat_map(|(_, &i)| candidates.get(i).digits().map(move |v| (i, v)))
                            .filter(|(_, v)| !trio.contains(v))
                            .collect();
                        // Sem nada a remover, repetir esta dedução travaria o solver.
                        if !pairs.is_empty() {
                            return Some(Deduction {
                                action: Action::Eliminate(pairs),
                                technique: "Hidden Triple",
                                level: Difficulty::Dificil,
                            });
                        }
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
    use crate::grid::Cell;

    /// Na linha 0, os dígitos pedidos só cabem nas células pedidas.
    fn only_in(c: &mut CandidateGrid, digits: &[u8], cells: &[usize]) {
        for i in (0..9).filter(|i| !cells.contains(i)) {
            for &v in digits {
                c.eliminate(i, v);
            }
        }
    }

    fn triple_in_0_1_2() -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        only_in(&mut c, &[1, 2, 3], &[0, 1, 2]);
        c
    }

    fn expected() -> Vec<(usize, u8)> {
        [0, 1, 2]
            .into_iter()
            .flat_map(|i| (4..=9).map(move |v| (i, v)))
            .collect()
    }

    #[test]
    fn remove_os_outros_candidatos_das_tres_celulas() {
        let d = HiddenTriple
            .apply(&Grid::empty(), &triple_in_0_1_2())
            .unwrap();
        assert_eq!(d.action, Action::Eliminate(expected()));
        assert_eq!(d.technique, "Hidden Triple");
        assert_eq!(d.level, Difficulty::Dificil);
    }

    #[test]
    fn aceita_digito_em_so_duas_das_tres_celulas() {
        let mut c = triple_in_0_1_2();
        c.eliminate(2, 3);
        let d = HiddenTriple.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(expected()));
    }

    #[test]
    fn ignora_digito_ja_colocado() {
        // Com o 9 colocado, a máscara dele na linha 0 é vazia. Se ela contasse,
        // {1, 2, 9} pareceria um triple em 0, 1 e 2 e removeria candidatos
        // verdadeiros da célula 2.
        let mut grid = Grid::empty();
        grid.set(8, Cell::Filled(9));
        let mut c = CandidateGrid::from_grid(&grid);
        only_in(&mut c, &[1], &[0, 1, 8]);
        only_in(&mut c, &[2], &[0, 1, 2, 8]);
        assert_eq!(HiddenTriple.apply(&grid, &c), None);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(HiddenTriple.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = triple_in_0_1_2();
        for i in 0..3 {
            for v in 4..=9 {
                c.eliminate(i, v);
            }
        }
        assert_eq!(HiddenTriple.apply(&Grid::empty(), &c), None);
    }
}
