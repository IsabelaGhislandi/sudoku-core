use super::{Action, Deduction, Technique, units};
use crate::candidates::CandidateGrid;
use crate::difficulty::Difficulty;
use crate::grid::Grid;

/// Três células de uma unidade cujos candidatos, juntos, são exatamente 3 dígitos:
/// esses dígitos ficam com elas, então saem das outras células da unidade. Cada célula
/// pode ter só 2 dos 3 dígitos.
pub struct NakedTriple;

impl Technique for NakedTriple {
    fn level(&self) -> Difficulty {
        Difficulty::Dificil
    }

    fn apply(&self, _grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction> {
        for unit in units() {
            // Só células com 2 ou 3 candidatos podem fazer parte do triple; células
            // preenchidas têm 0 e ficam de fora.
            let celulas: Vec<usize> = unit
                .iter()
                .copied()
                .filter(|&i| (2..=3).contains(&candidates.get(i).count()))
                .collect();
            // Três laços sobre posições crescentes: cada combinação aparece uma vez,
            // sempre na mesma ordem, sem montar a lista de combinações.
            for x in 0..celulas.len() {
                for y in (x + 1)..celulas.len() {
                    for z in (y + 1)..celulas.len() {
                        let trio = [celulas[x], celulas[y], celulas[z]];
                        let digitos = candidates
                            .get(trio[0])
                            .union(candidates.get(trio[1]))
                            .union(candidates.get(trio[2]));
                        if digitos.count() != 3 {
                            continue;
                        }
                        let pares: Vec<(usize, u8)> = unit
                            .iter()
                            .filter(|i| !trio.contains(i))
                            .flat_map(|&i| digitos.values().into_iter().map(move |v| (i, v)))
                            .filter(|&(i, v)| candidates.get(i).contains(v))
                            .collect();
                        // Sem nada a remover, repetir esta dedução travaria o solver.
                        if !pares.is_empty() {
                            return Some(Deduction {
                                action: Action::Eliminate(pares),
                                technique: "Naked Triple",
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

    /// Deixa a célula só com os dígitos pedidos.
    fn so_com(c: &mut CandidateGrid, celula: usize, digitos: &[u8]) {
        for v in 1..=9 {
            if !digitos.contains(&v) {
                c.eliminate(celula, v);
            }
        }
    }

    fn triple(a: &[u8], b: &[u8], d: &[u8]) -> CandidateGrid {
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        so_com(&mut c, 0, a);
        so_com(&mut c, 1, b);
        so_com(&mut c, 2, d);
        c
    }

    fn esperado_linha_0() -> Vec<(usize, u8)> {
        (3..9).flat_map(|i| [(i, 1), (i, 2), (i, 3)]).collect()
    }

    #[test]
    fn elimina_os_tres_digitos_das_outras_celulas() {
        let c = triple(&[1, 2], &[2, 3], &[1, 3]);
        let d = NakedTriple.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado_linha_0()));
        assert_eq!(d.technique, "Naked Triple");
        assert_eq!(d.level, Difficulty::Dificil);
    }

    #[test]
    fn aceita_celula_com_os_tres_candidatos() {
        let c = triple(&[1, 2, 3], &[1, 2], &[2, 3]);
        let d = NakedTriple.apply(&Grid::empty(), &c).unwrap();
        assert_eq!(d.action, Action::Eliminate(esperado_linha_0()));
    }

    #[test]
    fn uniao_de_dois_digitos_nao_eh_triple() {
        // Duas células com {1,2} e uma cheia: é um Naked Pair, não um triple.
        let mut c = CandidateGrid::from_grid(&Grid::empty());
        so_com(&mut c, 0, &[1, 2]);
        so_com(&mut c, 1, &[1, 2]);
        assert_eq!(NakedTriple.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_sem_padrao() {
        let c = CandidateGrid::from_grid(&Grid::empty());
        assert_eq!(NakedTriple.apply(&Grid::empty(), &c), None);
    }

    #[test]
    fn retorna_none_quando_nao_ha_o_que_eliminar() {
        let mut c = triple(&[1, 2], &[2, 3], &[1, 3]);
        // O triple vale na linha 0 e no box 0: os dígitos já saíram do resto dos dois.
        for i in [3, 4, 5, 6, 7, 8, 9, 10, 11, 18, 19, 20] {
            for v in 1..=3 {
                c.eliminate(i, v);
            }
        }
        assert_eq!(NakedTriple.apply(&Grid::empty(), &c), None);
    }
}
