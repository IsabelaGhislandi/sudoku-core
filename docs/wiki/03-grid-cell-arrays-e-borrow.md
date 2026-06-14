# 03 — Grid, Cell: arrays fixos e borrow checker

> Referente à **Task 3** do plano. Commit: `feat: add Grid and Cell with empty/get/set/is_complete`.

```rust
pub enum Cell { Empty, Filled(u8) }
pub struct Grid { cells: [Cell; 81] }
```

## Array de tamanho fixo `[Cell; 81]`

`[Cell; 81]` é um **array de tamanho fixo**, alocado na *stack* — diferente de `Vec<Cell>`,
que é dinâmico e vive na *heap*. O tamanho **faz parte do tipo**: `[Cell; 81]` e `[Cell; 80]`
são tipos diferentes, e isso é checado em compilação.

Por que aqui: o Sudoku é sempre 9×9 = 81 células. Tamanho fixo é mais rápido (sem alocação
dinâmica) e tornaria impossível, por construção, um tabuleiro com o número errado de células.

Indexamos linearmente por `0..81` em vez de `[linha][coluna]`. A conversão índice ↔
(linha, coluna, box) vem na Task 5.

## Variante com dado: `Filled(u8)`

`Cell` é `Empty | Filled(u8)`. A variante `Filled` **carrega um valor** (o dígito 1..=9).
É como modelar "vazio OU preenchido com n" — sem precisar de `null` nem de um sentinela
mágico tipo `0 = vazio`.

## `matches!` — padrão como booleano

```rust
self.cells.iter().all(|c| matches!(c, Cell::Filled(_)))
```

`matches!(valor, padrão)` devolve `true`/`false` se o valor casa com o padrão. O `_` dentro
de `Filled(_)` significa "qualquer dígito, não me importa qual". É a forma curta de um
`match` que retornaria `true`/`false`.

## Borrow checker: `&self` vs `&mut self`

| Método | Recebe | Significado |
|--------|--------|-------------|
| `get` | `&self` | empréstimo **só-leitura** |
| `set` | `&mut self` | empréstimo **mutável** (exclusivo) |
| `empty` | nada | **construtor** (associated function, sem `self`) |

A regra do *borrow checker*: a qualquer momento você pode ter **vários leitores `&`** OU
**um único escritor `&mut`**, nunca os dois ao mesmo tempo. É isso que dá segurança de
memória sem garbage collector. Consequência prática: como `set` precisa de `&mut self`, a
variável tem de ser `let mut grid` — senão o compilador recusa.

> `empty()` não recebe `self`: é uma *associated function* (tipo um método estático),
> chamada como `Grid::empty()`. Construtores idiomáticos costumam se chamar `new`/`empty`.

## Iteradores zero-cost

`.iter().all(...)` percorre as células de forma *lazy*. A ideia é a mesma de `.every()` no
JS, mas em Rust os iteradores são **zero-cost**: compilam para algo tão rápido quanto um
laço escrito à mão.
