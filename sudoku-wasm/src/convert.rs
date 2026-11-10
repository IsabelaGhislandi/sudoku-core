//! Conversões entre o que o JavaScript manda e os tipos do `sudoku-game`.

use sudoku_game::Difficulty;

/// Converte um instante do JS (`performance.now()` ou `Date.now()`, um `number`) para os
/// milissegundos inteiros que o `sudoku-game` usa.
pub fn to_ms(now: f64) -> u64 {
    // O `as` de float para inteiro satura: negativo vira 0, NaN vira 0, e o que passa do
    // `u64::MAX` vira o máximo. O `performance.now()` recomeça do zero a cada recarga da página,
    // e isso não pode virar underflow no relógio da partida.
    now as u64
}

/// Lê o nível pelo nome que o Rust usa na foto e no resultado (`"Facil"`, `"MuitoDificil"`...).
///
/// Os nomes repetem os do `sudoku-game` (que os guarda num module privado): são parte do formato
/// da foto, então não mudam, e um nome errado aqui é pego pelo teste de ida e volta da foto.
pub fn parse_difficulty(name: &str) -> Result<Difficulty, String> {
    match name {
        "Facil" => Ok(Difficulty::Facil),
        "Medio" => Ok(Difficulty::Medio),
        "Dificil" => Ok(Difficulty::Dificil),
        "MuitoDificil" => Ok(Difficulty::MuitoDificil),
        _ => Err("UnknownDifficulty".to_string()),
    }
}

/// O nome do nível no formato da foto e do resultado: o inverso de `parse_difficulty`.
pub fn difficulty_name(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Facil => "Facil",
        Difficulty::Medio => "Medio",
        Difficulty::Dificil => "Dificil",
        Difficulty::MuitoDificil => "MuitoDificil",
    }
}

/// Lê uma seed em decimal. Ela vem como texto porque um `number` do JS só guarda inteiros
/// exatos até 2^53, e a seed é um `u64` qualquer.
pub fn parse_seed(text: &str) -> Result<u64, String> {
    text.parse().map_err(|_| "InvalidSeed".to_string())
}

/// O nome da variante de um erro do `sudoku-game` (`"CellNotEmpty"`, `"Paused"`...), que o JS
/// recebe como texto da exceção. O `Debug` derivado das variantes sem dados é exatamente o nome,
/// e os nomes já fazem parte da API pública do `sudoku-game`.
pub fn error_name(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_negativo_ou_nan_vira_zero() {
        assert_eq!(to_ms(-5.0), 0);
        assert_eq!(to_ms(f64::NAN), 0);
        assert_eq!(to_ms(1234.9), 1234);
    }

    #[test]
    fn seed_maxima_atravessa_a_ponte() {
        assert_eq!(parse_seed("18446744073709551615"), Ok(u64::MAX));
        assert_eq!(parse_seed("abc"), Err("InvalidSeed".to_string()));
        assert!(parse_seed("-1").is_err());
    }

    #[test]
    fn nivel_pelo_nome() {
        assert_eq!(parse_difficulty("Facil"), Ok(Difficulty::Facil));
        assert_eq!(parse_difficulty("Medio"), Ok(Difficulty::Medio));
        assert_eq!(parse_difficulty("Dificil"), Ok(Difficulty::Dificil));
        assert_eq!(
            parse_difficulty("MuitoDificil"),
            Ok(Difficulty::MuitoDificil)
        );
        assert_eq!(
            parse_difficulty("facil"),
            Err("UnknownDifficulty".to_string())
        );
    }
}
