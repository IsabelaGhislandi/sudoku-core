#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Facil,
    Medio,
    Dificil,
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
        let niveis = [Difficulty::Facil, Difficulty::Dificil, Difficulty::Medio];
        assert_eq!(niveis.iter().max().copied(), Some(Difficulty::Dificil));
    }
}
