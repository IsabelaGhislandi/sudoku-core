//! Nomes salvos: o texto que representa um nível na foto (`SavedGame`) e no resultado salvo
//! (`SavedResult`).
//!
//! Mora num module próprio porque não pertence a nenhum dos dois formatos: os dois escrevem o
//! nível do mesmo jeito, e o resultado não deveria depender do module da foto só para isso.

use sudoku_core::Difficulty;

/// Nome do nível nos formatos salvos. `match` exaustivo: um nível novo no core quebra a
/// compilação aqui.
pub(crate) fn difficulty_name(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Facil => "Facil",
        Difficulty::Medio => "Medio",
        Difficulty::Dificil => "Dificil",
        Difficulty::MuitoDificil => "MuitoDificil",
    }
}

/// O contrário de `difficulty_name`. Devolve `Option` porque a foto e o resultado salvo usam o
/// mesmo nome, e cada um transforma o `None` no seu próprio erro.
pub(crate) fn parse_difficulty(name: &str) -> Option<Difficulty> {
    match name {
        "Facil" => Some(Difficulty::Facil),
        "Medio" => Some(Difficulty::Medio),
        "Dificil" => Some(Difficulty::Dificil),
        "MuitoDificil" => Some(Difficulty::MuitoDificil),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nome_do_nivel_vai_e_volta() {
        for d in [
            Difficulty::Facil,
            Difficulty::Medio,
            Difficulty::Dificil,
            Difficulty::MuitoDificil,
        ] {
            assert_eq!(parse_difficulty(difficulty_name(d)), Some(d));
        }
        assert_eq!(parse_difficulty("facil"), None);
    }
}
