use std::num::NonZeroU8;

/// Regras ajustáveis de uma partida. O solo usa o padrão; no PvP, quem cria a sala escolhe o
/// limite de erros (ver `architecture.md`, seção 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameConfig {
    /// Vidas no início. `NonZeroU8` porque uma partida sem vidas já nasceria perdida: o tipo
    /// impede esse estado, em vez de uma checagem em tempo de execução.
    pub lives: NonZeroU8,
    /// Dicas disponíveis. Zero é válido: partida sem dicas.
    pub hints: u8,
}

impl Default for GameConfig {
    /// 3 vidas e 2 dicas, as regras do modo solo.
    fn default() -> Self {
        GameConfig {
            lives: NonZeroU8::new(3).expect("3 não é zero"),
            hints: 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_padrao_tem_tres_vidas_e_duas_dicas() {
        let config = GameConfig::default();
        assert_eq!(config.lives.get(), 3);
        assert_eq!(config.hints, 2);
    }
}
