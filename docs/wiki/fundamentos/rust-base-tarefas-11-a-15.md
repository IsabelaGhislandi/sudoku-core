# Fundamentos de Rust — consolidado das Tarefas 11 a 15

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo),
> a partir de tudo que implementamos da Task 11 à Task 15. Geramos um destes **a cada 5 tarefas**.
> As páginas por-task ([11](../11-rating-iteradores-e-option.md) … [15](../15-ci-cd-com-github-actions.md))
> contam a história na ordem do código; **este** documento ensina os fundamentos por trás.
> Os blocos anteriores estão em [Tarefas 1 a 5](rust-base-tarefas-01-a-05.md) e
> [Tarefas 6 a 10](rust-base-tarefas-06-a-10.md).

As tarefas 1–5 foram sobre **modelar dados** e as 6–10 sobre **modelar comportamento**. As
tarefas 11–15 são sobre **compor, publicar e proteger**: juntar as peças em algo útil, decidir o
que o mundo enxerga e garantir que as promessas continuem valendo com o tempo.

Sumário:
1. [Iteradores: cadeias preguiçosas](#1-iteradores-cadeias-preguiçosas)
2. [Expressões e saídas antecipadas](#2-expressões-e-saídas-antecipadas)
3. [`Option`, `Result` ou `panic!`](#3-option-result-ou-panic)
4. [O borrow checker olha o uso, não o escopo](#4-o-borrow-checker-olha-o-uso-não-o-escopo)
5. [Traits em escopo: métodos de extensão](#5-traits-em-escopo-métodos-de-extensão)
6. [Determinismo e reprodutibilidade](#6-determinismo-e-reprodutibilidade)
7. [Visibilidade e a fachada do crate](#7-visibilidade-e-a-fachada-do-crate)
8. [Os três tipos de teste do projeto](#8-os-três-tipos-de-teste-do-projeto)
9. [Cargo: versões, lock e `--locked`](#9-cargo-versões-lock-e---locked)
10. [Ferramentas que viram regra: rustfmt, clippy e warnings](#10-ferramentas-que-viram-regra-rustfmt-clippy-e-warnings)
11. [CI: do combinado à verificação](#11-ci-do-combinado-à-verificação)
12. [Medir antes de otimizar](#12-medir-antes-de-otimizar)
13. [Padrões que apareceram](#13-padrões-que-apareceram)
14. [Tabela-resumo dos conceitos](#14-tabela-resumo-dos-conceitos)

---

## 1. Iteradores: cadeias preguiçosas

```rust
let level = result
    .steps                          // Vec<Deduction>
    .iter()                         // Iterator<Item = &Deduction>
    .map(|s| s.level)               // Iterator<Item = Difficulty>
    .max()                          // Option<Difficulty>
    .unwrap_or(Difficulty::Facil);  // Difficulty
```

Um iterador em Rust é uma **receita de cálculo**, não uma coleção. Há dois tipos de elo:

| Tipo de elo | Exemplos | O que faz |
|---|---|---|
| **Adaptador** (preguiçoso) | `map`, `filter` | só monta a receita; não calcula nada |
| **Consumidor** | `max`, `min_by_key`, `find`, `count`, `collect` | puxa os valores e finalmente executa |

Sem consumidor, nada roda, e o compilador avisa (`unused_must_use`). É a grande diferença para
`arr.map(...)` em JS, que percorre tudo na hora e aloca um array novo. Aqui não há `Vec`
intermediário, e uma cadeia costuma compilar para o mesmo código de um `for` escrito à mão.

Os três `iter` da coleção decidem o que cada elemento é:

| Método | Item | O `Vec` depois |
|---|---|---|
| `.iter()` | `&T` (empresta) | continua disponível |
| `.iter_mut()` | `&mut T` (empresta para mutar) | continua disponível |
| `.into_iter()` | `T` (consome) | deixa de existir |

**Dica de leitura:** acompanhe o **tipo** a cada elo, como nos comentários acima. Quase todo erro
de compilação em cadeia longa vira óbvio assim.

### Por que `max` e `min_by_key` devolvem `Option`

Porque o iterador pode estar **vazio**, e aí não existe maior nem menor. Rust não tem
`-Infinity` nem `undefined` para mascarar o caso. E os dois casos vazios do projeto têm
significado de domínio:

```rust
// rate: nenhum passo => grade já completa => Fácil
.max().unwrap_or(Difficulty::Facil)

// count_solutions: nenhuma célula vazia => caso base da recursão
let next = (0..81)
    .filter(|&i| matches!(grid.get(i), Cell::Empty))
    .min_by_key(|&i| candidates_for(grid, i).count());
```

`max` só compila porque `Difficulty: Ord`. O `derive(PartialOrd, Ord)` ordena as variantes pela
**ordem de declaração**, então o nível mais difícil é literalmente o maior.

Em caso de empate, `min_by_key` devolve o **primeiro** mínimo e `max` devolve o **último** máximo.
Ambos são regras documentadas, e é por isso que usar essas funções não quebra o determinismo.

## 2. Expressões e saídas antecipadas

Em Rust, quase tudo é **expressão**: produz um valor. A última expressão de um bloco, **sem `;`**,
é o valor do bloco.

```rust
pub fn rate(grid: &Grid) -> Option<Difficulty> {
    if count_solutions(grid, 2) != 1 {
        return None;                     // saída antecipada: precisa de return
    }
    let result = LogicalSolver::new().solve(grid);
    if result.solved {
        let level = /* ... */;
        Some(level)                      // valor deste ramo
    } else {
        Some(Difficulty::MuitoDificil)   // valor deste ramo
    }                                    // o if/else inteiro é o retorno da função
}
```

Por isso `return` quase só aparece em **guard clauses**. O compilador confere que todos os
caminhos devolvem o tipo certo. Um `;` a mais no fim transforma o valor em `()`, e o erro
`expected Option<Difficulty>, found ()` é o clássico de quem está começando.

**A ordem das checagens é parte da regra.** Unicidade vem antes do solver porque "Muito Difícil"
significa "tem resposta única, mas a lógica não alcança". Sem a guarda, uma grade sem solução
empacaria no solver e viraria um falso `MuitoDificil`.

## 3. `Option`, `Result` ou `panic!`

São três respostas para "e se não der?", e escolher entre elas é decisão de design da API:

| Ferramenta | Significa | Exemplo nosso |
|---|---|---|
| `Option<T>` | a ausência é uma resposta legítima | `rate` → `None`: essa grade não tem dificuldade |
| `Result<T, E>` | falhou, e quem chama merece saber o motivo | `generate` → `Err(GenError::Unsupported(Medio))` |
| `panic!` / `.unwrap()` | bug: nunca deveria acontecer | `.unwrap()` e `.expect("...")` nos testes |

### Mantenha a ausência fora do enum

Em vez de `Option<Difficulty>`, daria para criar `Difficulty::Invalido`. Seria pior: todo `match`
no app (tela de seleção, histórico) teria que tratar um "nível" que nunca devia estar ali. A
ausência fica no tipo de retorno e a validação fica na **borda**.

### Um erro que carrega dados

```rust
pub enum GenError {
    Unsupported(Difficulty),
}
```

Variantes de enum podem carregar dados. O erro diz **qual** nível foi pedido, sem precisar de
string. Quem chama faz `match` e monta a própria mensagem, o que combina com um core sem UI.

### Tirando o valor de dentro

| Método | Se for `None`/`Err` | Quando usar |
|---|---|---|
| `.unwrap()` | panic | testes |
| `.expect("msg")` | panic com a mensagem | testes; falha autoexplicativa |
| `.unwrap_or(v)` | devolve `v` (sempre avaliado) | padrão barato, como uma variante de enum |
| `.unwrap_or_else(\|\| ...)` | chama a closure | padrão caro de calcular |
| `?` | devolve o erro para quem chamou | função que também devolve `Option`/`Result` |

## 4. O borrow checker olha o uso, não o escopo

Dois trechos do gerador e do backtracking parecem violar a regra "ou N `&`, ou uma `&mut`", e
ambos compilam. Entender o porquê resolve boa parte das brigas com o borrow checker.

### `Copy` não deixa empréstimo pendurado

```rust
let removed = puzzle.get(index);    // Cell é Copy: é uma CÓPIA
puzzle.set(index, Cell::Empty);     // &mut puzzle: nada emprestado, tudo certo
// ...
puzzle.set(index, removed);         // restaura
```

Se `get` devolvesse `&Cell`, a referência ainda estaria em uso na última linha e o `set` do meio
seria recusado. O mesmo vale para `.map(|s| s.level)`: sai de um `&Deduction` sem problema porque
`Difficulty` é `Copy`. Com um campo `String`, daria erro de *move out of borrowed content*.

### O empréstimo termina no último uso

```rust
let next = (0..81)
    .filter(|&i| matches!(grid.get(i), Cell::Empty))   // empresta grid para ler
    .min_by_key(|&i| candidates_for(grid, i).count()); // consome o iterador aqui
match next {                                           // next é Option<usize>: não empresta nada
    Some(index) => { grid.set(index, /* ... */); /* ... */ } // &mut grid: permitido
    /* ... */
}
```

As closures emprestam `grid`, mas o `min_by_key` consome o iterador e o que sobra é um `usize`.
Depois disso, ninguém mais usa o empréstimo, então ele já acabou. Isso se chama **NLL**
(*Non-Lexical Lifetimes*): o empréstimo vai até o **último uso**, não até o fim do bloco.

**Regra prática:** se o borrow checker reclamar, procure quem ainda está **segurando** a referência
mais adiante no código. Devolver um valor `Copy` ou um índice, em vez de uma referência, costuma
resolver.

## 5. Traits em escopo: métodos de extensão

```rust
use rand::SeedableRng;       // traz ChaCha12Rng::seed_from_u64
use rand::seq::SliceRandom;  // traz .shuffle()
```

Nenhum dos dois nomes é escrito no código, e mesmo assim sem eles não compila:

```
error[E0599]: no method named `shuffle` found for struct `Vec<u8>` in the current scope
```

Um método de trait só pode ser chamado onde o trait está **importado**. `Vec` não tem `shuffle`;
é o `SliceRandom` que o **acrescenta** a todo slice:

```rust
impl<T> SliceRandom for [T] { /* ... */ }
```

O `shuffle` chega ao `Vec<u8>` porque um `Vec` se comporta como slice nas chamadas de método
(*deref*). É o jeito de Rust fazer **métodos de extensão** sem *monkey-patching*: em JS, mexer no
`Array.prototype` afeta o programa inteiro. Aqui, a extensão só existe onde você pediu, e duas
bibliotecas com um `.shuffle()` não colidem.

### Depender do comportamento, não do tipo

O `shuffle` é **genérico** sobre o gerador (`fn shuffle<R>(&mut self, rng: &mut R)`). Por isso a
troca de `StdRng` para `ChaCha12Rng` só mudou a anotação de tipo no nosso código. É o mesmo
desacoplamento do trait `Technique`, agora vindo de uma biblioteca.

### Mesmo nome, versões diferentes, tipos diferentes

Para o compilador, um trait do `rand_core` 0.9 e um do `rand_core` 0.10 são **traits diferentes**.
Se `rand` e `rand_chacha` discordassem, surgiria um erro confuso (*"trait bound is not satisfied"*),
com tudo aparentemente importado. Quando vir esse erro, confira versões duplicadas com
`cargo tree -d`. No projeto, as duas libs usam `rand_core` 0.10.

## 6. Determinismo e reprodutibilidade

Parecem iguais, mas são duas promessas:

| Promessa | Significa | Garantido por |
|---|---|---|
| **Determinismo** | rodar duas vezes **agora** dá o mesmo resultado | qualquer RNG com seed |
| **Reprodutibilidade** | rodar **daqui a um ano**, com outra versão da lib, dá o mesmo resultado | um algoritmo com nome fixo |

`StdRng` é um apelido para "o melhor algoritmo do momento", e pode mudar. `ChaCha12Rng` **é** o
algoritmo. Como a seed vai virar um código compartilhável de puzzle, só a segunda promessa serve.

Um RNG com seed é uma **função pura disfarçada**: cada número gerado avança o estado interno. Isso
tem duas consequências no código:

```rust
let mut rng = ChaCha12Rng::seed_from_u64(seed);
let solution = full_solution(&mut rng);      // &mut: gerar número é mutação
let givens = dig_facil(&solution, &mut rng); // continua a MESMA sequência
```

1. O RNG circula como **`&mut`**, para existir uma única sequência em toda a geração.
2. A **ordem das chamadas é contrato**: um `shuffle` a mais no meio muda todos os puzzles.

> ⚠️ **Atenção para a fatia Médio.** O contrato não depende só do RNG. O `dig_facil` usa
> `LogicalSolver::new()` para decidir se pode remover cada célula. Se técnicas de Médio entrarem
> nessa lista, o solver resolve mais puzzles, mais células são removidas e **a saída da seed 42
> muda**, além de os puzzles deixarem de ser Fáceis. O golden test existe para flagrar isso.

## 7. Visibilidade e a fachada do crate

Tudo é privado por padrão, e a privacidade vale **no caminho inteiro** até o item:

| Declaração | Significa |
|---|---|
| `mod x;` | só este crate navega por `crate::x::...` |
| `pub mod x;` | quem usa o crate pode escrever `sudoku_core::x::...` |
| `pub fn f` dentro de `x` | pública em relação a `x`; só alcançável se o caminho for público |
| `pub(crate) fn f` | visível no crate inteiro, nunca fora dele |
| `pub use x::f;` | publica `f` na raiz: `sudoku_core::f` |

O ponto que confunde vindo de JS: `pub fn generate` num `mod generator` privado **não vaza**. É
como um `export` num arquivo que ninguém importa. Quem abre a porta é o `pub use`:

```rust
mod generator;                                   // módulo privado
pub use generator::{GenError, Puzzle, generate}; // fachada
```

Quem usa o crate vê uma lista plana (`sudoku_core::generate`), e o `lib.rs` funciona como um
`index.ts` que o compilador **impõe**: não dá para importar um arquivo por dentro. Dá para mover
`rate` de arquivo sem quebrar ninguém, e helpers como `row_of` e `fill` continuam livres para mudar.

> **Correção ao consolidado anterior:** ele previa usar `pub(crate)` na Task 13, mas isso não
> aconteceu. `row_of` é `pub` dentro do módulo privado `grid` e não é re-exportado, o que na prática
> dá o mesmo efeito: visível no crate, invisível fora. `pub(crate)` só deixaria essa intenção explícita.

### Warnings de `dead_code` como sinal

Antes da fachada, o compilador avisava `function generate is never used`. Ele estava certo: uma
função `pub` em módulo privado que só os testes chamam é **código morto**, porque
`#[cfg(test)]` não existe no build normal. Pelo mesmo motivo, um `use` que só os testes usam deve
ficar **dentro** do `mod tests`.

### Detalhes de nome e documentação

- O pacote é `sudoku-core`, mas o código usa `sudoku_core`: identificadores não aceitam `-`, e o
  Cargo converte automaticamente.
- `///` documenta o item **abaixo**; `//!` documenta o item que **contém** o comentário. No topo do
  `lib.rs`, é a página inicial do `cargo doc --open`.

## 8. Os três tipos de teste do projeto

| | Unitário | Integração | Golden |
|---|---|---|---|
| Onde | `src/`, em `#[cfg(test)] mod tests` | `tests/api.rs` | `generator.rs` (`puzzle_da_seed_42_e_reproduzivel`) |
| Enxerga | também itens privados (`use super::*`) | **só** a API pública | a saída de `generate` |
| Pergunta | "esta peça funciona?" | "o fluxo funciona pela porta da frente?" | "a saída mudou?" |
| Pega | erro de lógica | "esqueci de exportar" | mudança acidental de contrato |

Cada arquivo em `tests/` é compilado como um **crate separado**, exatamente como o módulo Game vai
consumir o core. Por isso, antes da fachada, o teste falhou com `unresolved imports`, mesmo com as
funções já implementadas.

### Propriedade vs. golden

Os dois se complementam, e nenhum substitui o outro:

```rust
assert_eq!(count_solutions(&puzzle.givens, 2), 1);    // propriedade: POR QUE é válido
assert_eq!(puzzle.givens.to_line(), "94.35...12....");  // golden (trecho): O QUE sai
```

Um golden test **não prova que a saída está correta**, porque o valor foi copiado da própria
implementação. Ele prova que a saída está **estável**. Falhar significa "algo mudou; foi de
propósito?". Se foi, o novo valor entra no mesmo commit e a mudança aparece no diff.

Use com moderação: muitos goldens quebram a cada refactor legítimo. Aqui há um só, sobre a única
saída que é contrato público.

A propriedade que mais importa para quem joga também virou teste: **a dica nunca mente**.

```rust
let hint = next_hint(&puzzle.givens).expect("deve haver uma dica");
assert_eq!(puzzle.solution.get(hint.index), Cell::Filled(hint.value));
```

## 9. Cargo: versões, lock e `--locked`

| Arquivo | Diz | Paralelo web |
|---|---|---|
| `Cargo.toml` | quais versões **servem** (`rand = "0.10"` aceita qualquer 0.10.x) | `package.json` |
| `Cargo.lock` | qual versão **está em uso** (hoje, `rand` 0.10.1) | `package-lock.json` |

`--locked` faz o comando **falhar** se o lock não bater com o `Cargo.toml`, em vez de atualizá-lo
em silêncio: `cargo build --locked` ≈ `npm ci`.

Aqui o lock também protege o determinismo: o golden test roda no CI com versões travadas, então
um `cargo update` que mude a saída aparece como PR vermelho.

## 10. Ferramentas que viram regra: rustfmt, clippy e warnings

| Ferramenta | Pergunta | No CI |
|---|---|---|
| `cargo fmt` | está no formato oficial? | `cargo fmt --check` só confere |
| `cargo clippy` | tem código suspeito ou não idiomático? | `--all-targets -- -D warnings` inclui os testes |
| `rustc` | compila sem avisos? | `RUSTFLAGS: -D warnings` |

`-D warnings` (*deny*) transforma todo aviso em erro.

### Let chains: uma sugestão do clippy

O lint `collapsible_if` apontava `if`s aninhados, e a edition 2024 permite unificá-los com
**let chains** (aplicado no commit `refactor: collapse nested ifs into let chains`):

```rust
if let Cell::Filled(v) = self.cells[i]
    && v == value
{ /* ... */ }
```

`if let` e condições comuns se misturam com `&&`, e o `v` extraído já vale na condição seguinte.

### Bytes iguais em todo sistema

Dois arquivos pequenos evitam ruído:

- **`.gitattributes`** com `* text=auto eol=lf` fixa LF em todos os sistemas. Sem isso, o runner
  Windows veria bytes diferentes e o `rustfmt --check` poderia discordar entre sistemas.
- **`.git-blame-ignore-revs`** lista o commit que só formatou o código, para o `git blame` não
  atribuir a ele todas as linhas.

## 11. CI: do combinado à verificação

CI transforma o combinado "rode os testes antes" em algo que **falha sozinho**. A hierarquia:

```
workflow (ci.yml)
└── jobs   → cada um numa máquina nova, em paralelo
    └── steps → comandos em sequência; se um falha, os seguintes não rodam
```

Uma **matriz** multiplica um job por sistema, e `fail-fast: false` mostra todos os resultados em
vez de cancelar no primeiro erro.

As três ideias que valem além deste projeto:

1. **O CI só informa; quem bloqueia é a proteção de branch.** No ruleset `protect master`, os checks
   são obrigatórios e a lista de bypass é vazia, então a regra vale até para a dona do repositório.
2. **Um portão só está provado quando fecha.** Verde não prova nada. O PR de teste com um erro só de
   formatação deixou exatamente um job vermelho e bloqueou o merge.
3. **Check obrigatório é uma string fixa.** Renomear um job, ou acrescentar um `paths:` que impeça o
   workflow de rodar, deixa o PR esperando para sempre um status que nunca chega.

CD (o APK a cada tag `v*`) fica para o módulo Tauri, quando existir algo para entregar.

## 12. Medir antes de otimizar

A primeira versão dos testes do gerador levou **~220 s**. A medição mostrou onde o tempo estava:

| Chamada | Pior tempo (debug) |
|---|---|
| `full_solution` | < 1 ms |
| `solver.solve` | ~60 ms |
| `count_solutions` | **33 s** |

A correção foi a heurística **MRV** (*Minimum Remaining Values*): ramificar na célula com **menos
candidatos**. Com 0 candidatos, o ramo morre na hora; com 1, nem há ramificação. A suíte caiu para
**~8 s**, mudando só a escolha da célula e o laço que testa os dígitos dela.

Essa troca só foi segura porque `count_solutions` devolve uma **contagem**: mudar a ordem da busca
não muda a resposta. O caminho aleatório do gerador está em outra função (`fill`).

Um puzzle de 17 pistas, que antes não terminava em 60 s, virou teste (`puzzle_de_17_pistas_eh_unico`)
para proteger contra regressão.

## 13. Padrões que apareceram

- **Composição** — `rate` e `generate` quase não têm lógica própria: juntam `count_solutions` e
  `LogicalSolver`. `next_hint` é uma linha só, com um nome voltado para o domínio.
- **Fachada** — `lib.rs` esconde a organização interna atrás de uma lista plana.
- **Tentar, verificar, desfazer** — o `dig_facil` remove, confere e restaura, como o backtracking,
  mas de forma **gulosa**: nunca reconsidera uma célula.
- **Validação na borda** — `Option` no retorno de `rate` mantém `Difficulty` limpo.
- **Promessa + teste** — transformar um detalhe em contrato público pede uma implementação capaz de
  cumpri-lo (`ChaCha12Rng`) e um teste que avise quando for quebrado (golden).
- **Dependência no comportamento** — o RNG é trocado sem afetar quem só usa os traits.

## 14. Tabela-resumo dos conceitos

| Conceito | Essência | Exemplo nosso |
|----------|----------|---------------|
| Iterador preguiçoso | adaptadores montam, consumidores executam | `.iter().map(...).max()` |
| `Option` de consumidor | iterador vazio não tem máximo | `.max().unwrap_or(Facil)` |
| Expressão vs `;` | a última expressão é o valor do bloco | `if/else` no fim de `rate` |
| Guard clause | sai cedo no caso inválido | `if count_solutions(...) != 1` |
| `Option` / `Result` / `panic!` | ausência, falha ou bug | `rate`, `generate`, `.expect` |
| Enum de erro com dados | o erro carrega o contexto | `GenError::Unsupported(Difficulty)` |
| `Copy` e empréstimos | cópia não deixa referência pendurada | `let removed = puzzle.get(i)` |
| NLL | empréstimo termina no último uso | `min_by_key` seguido de `grid.set` |
| Trait em escopo | método de extensão só onde é importado | `use rand::seq::SliceRandom` |
| Reprodutibilidade | mesma saída entre versões da lib | `ChaCha12Rng` |
| `&mut` no RNG | gerar número avança o estado | `fill(grid, rng)` |
| Visibilidade + `pub use` | módulo privado, fachada pública | `lib.rs` |
| `//!` vs `///` | documenta o que contém vs o que vem abaixo | topo do `lib.rs` |
| Teste de integração | crate separado, só a API pública | `tests/api.rs` |
| Golden test | congela a saída; prova estabilidade | seed 42 |
| `Cargo.lock` + `--locked` | versões exatas; falha se divergir | CI |
| Let chains | `if let` e `&&` no mesmo `if` | `Grid::can_place` |
| MRV | ramificar na célula mais restrita | `count_solutions` |
| CI + ruleset | verificação automática + bloqueio real | `ci.yml`, `protect master` |
