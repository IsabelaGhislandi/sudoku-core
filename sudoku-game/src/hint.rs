//! Sorteio da célula de uma dica. É determinístico e faz parte do contrato, como a seed do
//! gerador: a mesma partida pede as mesmas dicas nas mesmas células, no celular e no servidor.

/// SplitMix64: mistura um `u64` num valor de aparência aleatória. É o algoritmo público de
/// Steele, Lea e Flood (2014), escrito aqui porque são cinco linhas e o `sudoku-game` não
/// precisa de uma dependência de RNG para isso. Mudar estas constantes muda as dicas.
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Posição, entre as `empties` células vazias (em ordem crescente de índice), da célula que a
/// dica número `hint_number` (0 na primeira) revela: `splitmix64(seed ^ hint_number) % empties`.
/// `empties` precisa ser maior que zero.
pub(crate) fn pick(seed: u64, hint_number: u8, empties: usize) -> usize {
    (splitmix64(seed ^ u64::from(hint_number)) % empties as u64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix64_bate_com_o_vetor_conhecido() {
        assert_eq!(splitmix64(0), 0xE220A8397B1DCDAF);
    }

    #[test]
    fn pick_fica_dentro_do_intervalo() {
        for hint_number in 0..10 {
            assert!(pick(42, hint_number, 57) < 57);
        }
        assert_eq!(pick(42, 0, 1), 0);
    }
}
