# 13 — API pública: visibilidade, `pub use` e testes de integração

> Referente à **Task 13** do plano. Commit: `feat: expose public API with next_hint and integration test`.

A última task da fatia Fácil não adiciona algoritmo novo, só uma função de uma linha
(`next_hint`). O trabalho aqui é **decidir o que o mundo lá fora enxerga** do crate e provar,
com um teste escrito "do lado de fora", que essa superfície basta para o fluxo completo.

## Dois níveis de visibilidade: módulo e item

```rust
mod generator;                                   // módulo PRIVADO
pub use generator::{generate, GenError, Puzzle}; // itens re-exportados
```

Em Rust, **tudo é privado por padrão**, e a privacidade vale em dois lugares:

| Declaração | Significa |
|---|---|
| `mod x;` | o módulo `x` existe, mas só este crate navega por `crate::x::...` |
| `pub mod x;` | quem usa o crate pode escrever `sudoku_core::x::...` |
| `pub fn f` dentro de `x` | `f` é pública **em relação a `x`**, mas só alcançável se o caminho até ela for público |
| `pub use x::f;` | publica `f` **na raiz**: `sudoku_core::f` |

O ponto que confunde vindo de JS: `pub fn generate` dentro de um `mod generator` privado
**não** vaza para fora. Ser `pub` num módulo privado é como um `export` num arquivo que ninguém
importa. É o `pub use` em `lib.rs` que abre a porta.

### O efeito colateral: warnings de `dead_code` somem

Antes desta task, `cargo build` avisava coisas como:

```
warning: function `generate` is never used
warning: struct `LogicalSolver` is never constructed
```

Para o compilador, uma função `pub` num módulo privado que ninguém do crate chama **é código
morto**: não há caminho de fora até ela. Testes em `#[cfg(test)]` não contam, porque só existem
no build de teste. Assim que `lib.rs` re-exporta esses itens, eles viram API e os warnings
desaparecem. O warning estava certo: até agora, essas funções não serviam para nada fora dos
testes.

## `lib.rs` como fachada

```rust
pub use backtracking::count_solutions;
pub use candidates::{candidates_for, CandidateSet};
pub use difficulty::Difficulty;
pub use generator::{generate, GenError, Puzzle};
pub use grid::{validate, Cell, Conflict, Grid, ParseError};
pub use rating::rate;
pub use solver::{next_hint, LogicalSolver, SolveResult};
pub use techniques::{Deduction, HiddenSingle, NakedSingle, Technique};
```

Quem usa o crate vê uma lista **plana**: `sudoku_core::generate`, `sudoku_core::Grid`. Nada de
`sudoku_core::generator::generate`. A organização interna em arquivos vira detalhe de
implementação:

- Dá para mover `rate` para outro arquivo sem quebrar ninguém, desde que o `pub use` continue.
- Helpers como `row_of`, `col_of`, `box_of`, `fill` e `dig_facil` continuam **internos**. O
  módulo Game não precisa deles, e não os expor mantém liberdade para mudá-los.

É o mesmo papel de um `index.ts` com `export { x } from './x'`, só que em Rust o compilador
**impõe** o limite: não existe importar um arquivo "por dentro" contornando a fachada.

## `//!`: doc da própria crate

```rust
//! sudoku-core — geração, resolução, validação e classificação de Sudoku 9×9.
//! Biblioteca pura: sem UI, sem IO, sem Tauri.
```

- `///` documenta **o item logo abaixo** (função, struct...).
- `//!` documenta **o item que contém o comentário**, aqui o crate inteiro.

É o texto que aparece na página inicial do `cargo doc --open`.

## Testes de integração: a pasta `tests/`

```
sudoku-core/
├── src/        testes unitários inline (#[cfg(test)] mod tests)
└── tests/
    └── api.rs  teste de integração
```

Cada arquivo em `tests/` é compilado como um **crate separado** que depende de `sudoku_core`,
exatamente como o futuro módulo Game vai depender. Consequências:

| | Teste unitário (`src/`) | Teste de integração (`tests/`) |
|---|---|---|
| Acessa itens privados | sim (`use super::*`) | **não**, só a API pública |
| Importa como | `crate::grid::Grid` | `sudoku_core::Grid` |
| Testa | uma peça isolada | o fluxo, pela porta da frente |
| Rodar só ele | `cargo test nome_do_modulo` | `cargo test --test api` |

Por isso o teste falhou do jeito certo antes da implementação:

```
error[E0432]: unresolved imports `sudoku_core::candidates_for`, `sudoku_core::count_solutions`, ...
```

As funções já existiam, mas não estavam **visíveis** de fora. Esse é o valor do teste de
integração: ele pega exatamente o erro "esqueci de exportar", que nenhum teste unitário pega.

### Hífen vira underscore

O pacote se chama `sudoku-core` (`Cargo.toml`), mas no código é `sudoku_core`. Identificadores
Rust não aceitam `-`, então o Cargo converte automaticamente.

## `next_hint`: reuso em vez de lógica nova

```rust
pub fn next_hint(grid: &Grid) -> Option<Deduction> {
    LogicalSolver::new().next_step(grid)
}
```

A dica é **a mesma pergunta** que o solver faz a cada passo: "qual a próxima jogada dedutível?".
Então não há algoritmo novo, só um nome voltado para o domínio do jogo. Quando as próximas
fatias adicionarem técnicas ao `LogicalSolver`, as dicas ficam mais espertas automaticamente.

O teste valida a propriedade que importa para o jogador: **a dica nunca mente**.

```rust
let hint = next_hint(&puzzle.givens).expect("deve haver uma dica");
assert_eq!(puzzle.solution.get(hint.index), Cell::Filled(hint.value));
```

`.expect("msg")` é um `.unwrap()` com mensagem: se for `None`, o panic mostra o texto. Em teste
isso deixa a falha autoexplicativa. Em código de produção, prefira tratar o `None`.

## Faxina: import usado só em teste

O build mostrou `warning: unused import: validate` em `generator.rs`. O `validate` só aparece
nos testes do gerador, então no build normal ele sobra. A correção foi mover o import para
**dentro** do `mod tests`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::validate;
```

Tudo dentro de `#[cfg(test)]` só existe no build de teste, e isso vale também para os `use`.

## Sugestões do clippy (opcionais)

`cargo clippy` hoje sugere `collapsible_if` em 4 lugares, como:

```rust
if let Cell::Filled(v) = grid.get(i) {
    if row_of(i) == r || col_of(i) == c || box_of(i) == b { ... }
}
```

Na edition 2024 isso pode virar um único `if` com **let chains**:

```rust
if let Cell::Filled(v) = grid.get(i)
    && (row_of(i) == r || col_of(i) == c || box_of(i) == b)
{ ... }
```

Fica como exercício. O plano deixa as sugestões do clippy como oportunidade de aprendizado
opcional nesta fatia.

## Fim da fatia Fácil

`sudoku-core` agora cumpre o objetivo do plano de ponta a ponta:

```rust
let puzzle = generate(Difficulty::Facil, 123)?;  // gera
rate(&puzzle.givens);                             // classifica
validate(&grid);                                  // aponta conflitos
next_hint(&grid);                                 // dá dica
```

**Próximas fatias:** técnicas de nível Médio (Naked/Hidden Pairs, Pointing Pairs), com
`dig` e `rate` parametrizados por nível. Depois vêm Difícil e Muito Difícil, e o `Deduction`
passa a representar também eliminação de candidatos.
