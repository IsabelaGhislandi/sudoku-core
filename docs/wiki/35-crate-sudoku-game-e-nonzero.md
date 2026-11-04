# 35 — O crate `sudoku-game`: dependência por caminho, re-export e `NonZeroU8`

> Referente à Task 2 do plano da fatia 1 do Game. Commit: `feat: add sudoku-game crate with game setup`.

O workspace da página 34 ganhou o segundo membro, o `sudoku-game`. Ele guarda as regras de uma
partida e, nesta task, só sabe **começar** uma: gerar o puzzle, montar as 81 células e contar
vidas e dicas.

## Um crate que depende de outro pelo caminho

```toml
[dependencies]
sudoku-core = { path = "../sudoku-core" }
```

Uma dependência com `path` não vem do crates.io: o Cargo compila a pasta indicada. Não há versão a
escolher nem download, e uma mudança no `sudoku-core` aparece no `sudoku-game` na próxima
compilação. No código, o hífen do nome do pacote vira sublinhado: `use sudoku_core::generate;`.

Ser uma dependência local não dá acesso a nada extra. O `sudoku-game` só enxerga o que o
`sudoku-core` marca como `pub`, exatamente como os testes de integração em `sudoku-core/tests/`
(wiki 13). Por isso o core não precisou mudar.

## Re-export: `pub use` entre crates

```rust
pub use config::GameConfig;
pub use game::{CellState, Game, Status};
pub use sudoku_core::{Difficulty, Puzzle};
```

As duas primeiras linhas fazem o mesmo que o `lib.rs` do core: os módulos são privados (`mod
game;`), e só os tipos escolhidos saem. A terceira é nova: ela **republica tipos de outro crate**.
`Game::new` recebe um `Difficulty`, e `game.puzzle()` devolve um `&Puzzle`. Sem o re-export, o app
precisaria depender também do `sudoku-core` só para escrever esses nomes. Com ele,
`sudoku_game::Difficulty` e `sudoku_core::Difficulty` são **o mesmo tipo**, e não uma cópia, então
os valores passam de um crate para o outro sem conversão.

## Estados inválidos irrepresentáveis: `NonZeroU8`

```rust
pub struct GameConfig {
    pub lives: NonZeroU8,
    pub hints: u8,
}
```

Uma partida com 0 vidas nasceria perdida. Dava para checar isso em `Game::new` e devolver um erro,
mas aí todo mundo que cria uma partida teria de tratar um erro que não deveria existir. O
`NonZeroU8` é um `u8` que **não pode ser zero**: o único jeito de criar um é por
`NonZeroU8::new(n)`, que devolve `Option<NonZeroU8>` (`None` para zero), ou por constantes como
`NonZeroU8::MIN` (que vale 1). Depois de criado, o zero já foi descartado, e `Game` não precisa
conferir nada.

Esse é o lema *make invalid states unrepresentable* (tornar estados inválidos irrepresentáveis): em
vez de validar em todo lugar, escolha um tipo em que o valor errado nem pode ser escrito. As dicas
ficaram com `u8` comum porque zero dicas é uma regra válida.

De bônus, `Option<NonZeroU8>` ocupa **1 byte**, o mesmo que um `u8`: o compilador usa o zero,
proibido, para representar o `None`. É a mesma otimização que deixa `Option<&T>` do tamanho de um
ponteiro.

## `Default` com documentação

```rust
impl Default for GameConfig {
    /// 3 vidas e 2 dicas, as regras do modo solo.
    fn default() -> Self { ... }
}
```

`#[derive(Default)]` daria `lives` sem valor possível (`NonZeroU8` não tem `Default`) e `hints: 0`.
Implementar à mão deixa os valores do jogo num lugar só, com um `///` que diz de onde eles vêm.
`GameConfig::default()` e `..Default::default()` passam a funcionar como em qualquer tipo da
biblioteca padrão.

## Montar um array com `std::array::from_fn`

```rust
let cells = std::array::from_fn(|i| match puzzle.givens.get(i) {
    Cell::Filled(v) => CellState::Given(v),
    Cell::Empty => CellState::Empty,
});
```

`from_fn` cria um `[T; N]` chamando a closure com cada índice, de 0 a N-1. O tamanho (81) vem do
tipo do campo `cells: [CellState; 81]`, que o compilador infere. É o jeito de montar um array fixo
sem começar com valores falsos e sobrescrever depois.

## Uma regra de determinismo no `///`

`Game::new` tenta `seed`, `seed + 1`, `seed + 2`... até `generate` não esgotar. O teste
`seed_esgotada_passa_para_a_seguinte` trava o caso conhecido: a seed 15 esgota o Muito Difícil, e a
partida sai com a 16. Como a regra muda qual puzzle a pessoa recebe, ela está escrita no doc comment
de `new`, e não só no código. O `wrapping_add` faz a seed `u64::MAX` voltar para 0 em vez de entrar
em pânico por overflow (mais sobre isso na wiki 38).
