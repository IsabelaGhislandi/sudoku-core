//! Estatísticas e ranking pessoal, por nível: funções puras sobre a lista de resultados que a
//! casca guarda e entrega. O Game não mantém essa lista.

use sudoku_core::Difficulty;

use crate::result::{GameResult, Outcome};

/// Os números de um nível. Abandonos contam como partidas jogadas e não vencidas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LevelStats {
    /// Partidas do nível, com qualquer desfecho.
    pub played: u32,
    /// Partidas do nível vencidas.
    pub won: u32,
    /// Menor tempo entre as vitórias, em milissegundos. `None` sem vitória no nível.
    pub best_ms: Option<u64>,
}

impl LevelStats {
    /// Porcentagem de vitórias, arredondada para o inteiro mais próximo (meio para cima):
    /// 2 de 3 dá 67. `None` sem partidas, em vez de 0%, que diria "perdeu todas". 0% e 100% só
    /// aparecem quando são verdade.
    pub fn win_percentage(&self) -> Option<u8> {
        if self.played == 0 {
            return None;
        }
        // Em u64: `won * 100` não estoura nem com u32::MAX partidas.
        let (won, played) = (u64::from(self.won), u64::from(self.played));
        let rounded = ((won * 100 + played / 2) / played) as u8;
        // 0% diria "perdeu todas" e 100% "venceu todas". Com uma vitória em 201, ou uma derrota
        // em 200, o arredondamento chegaria lá; nesses casos fica em 1% ou 99%.
        if won == 0 || won == played {
            Some(rounded)
        } else {
            Some(rounded.clamp(1, 99))
        }
    }
}

/// Estatísticas do nível `difficulty` na lista `results`.
pub fn level_stats(results: &[GameResult], difficulty: Difficulty) -> LevelStats {
    let level = results.iter().filter(|r| r.difficulty == difficulty);
    let wins = level.clone().filter(|r| r.outcome == Outcome::Won);
    LevelStats {
        played: level.count() as u32,
        won: wins.clone().count() as u32,
        best_ms: wins.map(|r| r.elapsed_ms).min(),
    }
}

/// Ranking pessoal do nível: só as vitórias, do menor tempo para o maior. Empate no tempo vai
/// para a mais antiga (`finished_at_ms` menor); empate também na data mantém a ordem da lista,
/// porque `sort_by_key` é estável. A casca escolhe quantas mostra.
pub fn personal_ranking(results: &[GameResult], difficulty: Difficulty) -> Vec<&GameResult> {
    let mut wins: Vec<&GameResult> = results
        .iter()
        .filter(|r| r.difficulty == difficulty && r.outcome == Outcome::Won)
        .collect();
    wins.sort_by_key(|r| (r.elapsed_ms, r.finished_at_ms));
    wins
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;
    use Difficulty::*;
    use Outcome::*;

    fn result(
        difficulty: Difficulty,
        outcome: Outcome,
        elapsed_ms: u64,
        finished_at_ms: u64,
    ) -> GameResult {
        GameResult {
            difficulty,
            seed: 0,
            outcome,
            elapsed_ms,
            mistakes: 0,
            hints_used: 0,
            config: GameConfig::default(),
            finished_at_ms,
        }
    }

    #[test]
    fn estatisticas_contam_so_o_nivel_pedido() {
        let results = [
            result(Medio, Won, 9_000, 1),
            result(Medio, OutOfLives, 3_000, 2),
            result(Medio, Abandoned, 1_000, 3),
            result(Facil, Won, 500, 4),
        ];
        let s = level_stats(&results, Medio);
        assert_eq!((s.played, s.won, s.best_ms), (3, 1, Some(9_000)));
        assert_eq!(s.win_percentage(), Some(33));
    }

    #[test]
    fn porcentagem_arredonda_para_o_mais_proximo() {
        let s = LevelStats {
            played: 3,
            won: 2,
            best_ms: Some(1),
        };
        assert_eq!(s.win_percentage(), Some(67));
    }

    #[test]
    fn porcentagem_so_eh_0_ou_100_quando_eh_verdade() {
        let stats = |played, won| LevelStats {
            played,
            won,
            best_ms: None,
        };
        assert_eq!(stats(200, 199).win_percentage(), Some(99));
        assert_eq!(stats(201, 1).win_percentage(), Some(1));
        assert_eq!(stats(5, 5).win_percentage(), Some(100));
        assert_eq!(stats(5, 0).win_percentage(), Some(0));
    }

    #[test]
    fn nivel_sem_partidas_nao_tem_porcentagem_nem_melhor_tempo() {
        let s = level_stats(&[], Dificil);
        assert_eq!(s, LevelStats::default());
        assert_eq!(s.win_percentage(), None);
    }

    #[test]
    fn ranking_so_tem_vitorias_do_menor_tempo_para_o_maior() {
        let results = [
            result(Medio, Won, 9_000, 1),
            result(Medio, OutOfLives, 100, 2),
            result(Medio, Won, 4_000, 3),
            result(Facil, Won, 50, 4),
        ];
        let times: Vec<u64> = personal_ranking(&results, Medio)
            .iter()
            .map(|r| r.elapsed_ms)
            .collect();
        assert_eq!(times, vec![4_000, 9_000]);
    }

    #[test]
    fn empate_no_tempo_vai_para_a_mais_antiga() {
        let results = [result(Medio, Won, 4_000, 20), result(Medio, Won, 4_000, 10)];
        let dates: Vec<u64> = personal_ranking(&results, Medio)
            .iter()
            .map(|r| r.finished_at_ms)
            .collect();
        assert_eq!(dates, vec![10, 20]);
    }
}
