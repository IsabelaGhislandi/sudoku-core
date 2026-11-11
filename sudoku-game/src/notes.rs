//! As anotações que a pessoa faz nas células vazias.

/// Anotações de uma célula: os números pequenos de "pode ser". Bit n ligado quer dizer que o
/// dígito n está anotado (n de 1 a 9).
///
/// É a mesma máscara do `CandidateSet` do core, com outro papel: candidatos são deduzidos pelo
/// solver, anotações são o que a pessoa escreveu. Só a `Game` liga e desliga, então a regra
/// "anotação só em célula vazia" não pode ser contornada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Notes(u16);

impl Notes {
    /// `true` se `value` (de 1 a 9) está anotado.
    pub fn contains(self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    /// Os dígitos anotados, em ordem crescente.
    pub fn values(self) -> Vec<u8> {
        (1..=9).filter(|&v| self.contains(v)).collect()
    }

    /// `true` sem nenhuma anotação, o caso de toda célula com número.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Liga o dígito se estava desligado, e desliga se estava ligado. Quem chama garante que
    /// `value` está em 1..=9.
    pub(crate) fn toggle(&mut self, value: u8) {
        self.0 ^= 1 << value;
    }

    /// Desliga o dígito, estivesse ligado ou não. Quem chama garante que `value` está em 1..=9.
    pub(crate) fn remove(&mut self, value: u8) {
        self.0 &= !(1 << value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anotacoes_comecam_vazias() {
        let notes = Notes::default();
        assert!(notes.is_empty());
        assert_eq!(notes.values(), Vec::<u8>::new());
    }

    #[test]
    fn toggle_liga_e_desliga_um_digito() {
        let mut notes = Notes::default();
        notes.toggle(7);
        notes.toggle(3);
        assert!(notes.contains(3) && notes.contains(7));
        assert_eq!(notes.values(), vec![3, 7]);
        notes.toggle(3);
        assert_eq!(notes.values(), vec![7]);
        assert!(!notes.is_empty());
    }
}
