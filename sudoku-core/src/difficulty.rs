//! Os níveis de dificuldade e a ordem entre eles.

/// O nível de um puzzle, definido pela técnica mais avançada que a resolução lógica
/// exige. A ordem das variantes é a da escala (`Facil < Medio < ...`): o solver usa
/// `max` para achar o nível de um puzzle e `<=` para se limitar a um nível.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    /// Resolve só com Naked Single e Hidden Single.
    Facil,
    /// Exige Naked Pair, Hidden Pair ou Pointing Pair.
    Medio,
    /// Exige Box/Line Reduction, Naked Triple, Hidden Triple ou X-Wing.
    Dificil,
    /// Exige Swordfish ou XY-Wing. Em `rate`, cobre também os puzzles que só saem por
    /// tentativa e erro.
    MuitoDificil,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordena_do_facil_ao_muito_dificil() {
        assert!(Difficulty::Facil < Difficulty::Medio);
        assert!(Difficulty::Medio < Difficulty::Dificil);
        assert!(Difficulty::Dificil < Difficulty::MuitoDificil);
    }

    #[test]
    fn max_pega_o_nivel_mais_alto() {
        let levels = [Difficulty::Facil, Difficulty::Dificil, Difficulty::Medio];
        assert_eq!(levels.iter().max().copied(), Some(Difficulty::Dificil));
    }
}
