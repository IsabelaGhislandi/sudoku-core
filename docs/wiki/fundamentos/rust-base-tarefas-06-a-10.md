# Fundamentos de Rust — consolidado das Tarefas 6 a 10

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo),
> a partir de tudo que implementamos da Task 6 à Task 10. Geramos um destes **a cada 5 tarefas**.
> As páginas por-task ([06](../06-candidatos-bitmask-e-operacoes-de-bit.md) … [10](../10-recursao-backtracking-e-mut-refs.md))
> contam a história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 1 a 5](rust-base-tarefas-01-a-05.md).

Se as tarefas 1–5 foram sobre **modelar dados** (struct, enum, `Option`/`Result`, ownership),
as tarefas 6–10 são sobre **modelar comportamento**: traits, closures, dispatch, e os dois
jeitos de resolver um Sudoku.

Sumário:
1. [Newtype: dar significado a um primitivo](#1-newtype-dar-significado-a-um-primitivo)
2. [Bits: um conjunto dentro de um inteiro](#2-bits-um-conjunto-dentro-de-um-inteiro)
3. [Traits: o polimorfismo do Rust](#3-traits-o-polimorfismo-do-rust)
4. [Os dois dispatches: `impl Trait` vs `dyn Trait`](#4-os-dois-dispatches-impl-trait-vs-dyn-trait)
5. [`Box`, o heap e tipos sem tamanho conhecido](#5-box-o-heap-e-tipos-sem-tamanho-conhecido)
6. [Closures a fundo: captura e a família `Fn`](#6-closures-a-fundo-captura-e-a-família-fn)
7. [Lifetimes na prática: `&'static str` vs `String`](#7-lifetimes-na-prática-static-str-vs-string)
8. [`Option` como ferramenta de fluxo](#8-option-como-ferramenta-de-fluxo)
9. [`&mut`, deref e mutação contida](#9-mut-deref-e-mutação-contida)
10. [Recursão e backtracking](#10-recursão-e-backtracking)
11. [Organizando módulos: pasta, `mod.rs` e `pub use`](#11-organizando-módulos-pasta-modrs-e-pub-use)
12. [Padrões que apareceram](#12-padrões-que-apareceram)
13. [Tabela-resumo dos conceitos](#13-tabela-resumo-dos-conceitos)

---

## 1. Newtype: dar significado a um primitivo

```rust
pub struct CandidateSet(u16);
```

Uma **tuple struct** é uma struct cujos campos não têm nome — acessados por posição (`.0`,
`.1`). Com **um** campo só, ela vira o padrão **newtype**: embrulhar um tipo primitivo para
dar a ele um significado próprio.

Três ganhos concretos:

| Ganho | Sem newtype (`u16` cru) | Com `CandidateSet` |
|---|---|---|
| **Significado** | "é um número" | "é um conjunto de candidatos" |
| **Segurança de tipo** | dá pra somar com qualquer `u16` | o compilador barra usos errados |
| **Encapsulamento** | os bits ficam expostos | campo privado; só a API do tipo mexe |

O campo `.0` é **privado** por não ter `pub`: de fora do módulo ninguém liga um bit na mão. É a
mesma ideia do `cells` privado dentro de `Grid` (Task 3) — o tipo garante suas próprias regras.

Custo em runtime: **zero**. O compilador remove o embrulho; `CandidateSet` ocupa exatamente os
2 bytes do `u16`.

## 2. Bits: um conjunto dentro de um inteiro

Um conjunto de até 9 elementos cabe num `u16`, um bit por dígito: bit ligado = está no
conjunto.

```
dígito:   9 8 7 6 5 4 3 2 1 0
bit:      1 1 1 1 1 1 1 1 1 0   = 0b11_1111_1110
```

| Operador | Nome | Como conjunto |
|----------|------|---------------|
| `1 << n` | shift left | cria a máscara só com o elemento `n` |
| `a \| b` | OR | **união** |
| `a & b` | AND | **interseção** |
| `!a` | NOT | **complemento** |
| `a & !b` | AND NOT | **diferença** (`a` menos `b`) |

```rust
self.0 & (1 << value) != 0     // pertence?
used |= 1 << v;                // adiciona (|= é "OR e atribui")
CandidateSet(all & !used)      // todos MENOS os usados = diferença
self.0.count_ones()            // tamanho do conjunto
```

- **`count_ones()`** é método nativo dos inteiros; muitas CPUs fazem em uma instrução
  (`popcount`). Contar elementos sai de graça.
- **`0b11_1111_1110`** — literal **binário** (`0b`); os `_` são só legibilidade, como vírgula
  de milhar. Existem também `0x` (hex) e `0o` (octal).
- Operações de conjunto viram uma instrução de CPU, **sem alocação** — bem diferente de um
  `HashSet<u8>`, que aloca no heap e faz hashing.

## 3. Traits: o polimorfismo do Rust

Uma **trait** é um contrato de comportamento: o parente mais próximo da `interface`.

```rust
pub trait Technique {
    fn apply(&self, grid: &Grid) -> Option<Deduction>;   // assinatura, sem corpo
}

impl Technique for NakedSingle {                          // implementação fica FORA do tipo
    fn apply(&self, grid: &Grid) -> Option<Deduction> { ... }
}
```

Duas diferenças importantes vindo do mundo web:

1. **A implementação mora num bloco separado**, `impl Trait for Tipo`. O tipo não "declara"
   as traits que cumpre — dá até para implementar uma trait sua para um tipo alheio. Em
   TypeScript/Java, a classe precisa dizer `implements X` na declaração.
2. **`&self` é explícito.** O método diz, na assinatura, se lê (`&self`), muta (`&mut self`)
   ou consome (`self`) a instância.

### Unit struct: um tipo sem dados

```rust
pub struct NakedSingle;      // sem campos, sem chaves
```

Existe só para ter um tipo onde pendurar o `impl`. O nome é ao mesmo tempo o **valor** — daí
`NakedSingle.apply(&grid)`, sem `::new()` nem `{}`. Quando a estratégia não tem estado, esse é
o custo mínimo: **zero bytes**.

### Trait vs. derive

Já conhecíamos traits pelo `#[derive(...)]` (Tasks 1–5). É a mesma coisa por outro caminho:
`derive` pede ao compilador para **gerar** o `impl` de traits da std; aqui escrevemos o `impl`
**à mão** para uma trait nossa.

## 4. Os dois dispatches: `impl Trait` vs `dyn Trait`

Rust tem **duas** formas de polimorfismo, e usamos as duas neste bloco:

```rust
fn sole_spot(..., in_unit: impl Fn(usize) -> bool)   // estático  (Task 8)
techniques: Vec<Box<dyn Technique>>                  // dinâmico  (Task 9)
```

| | `impl Trait` / genéricos | `dyn Trait` |
|---|---|---|
| Tipo decidido em | **compilação** | **runtime** |
| Mecanismo | **monomorfização**: uma cópia do código por tipo | **vtable**: ponteiro para o código |
| Custo por chamada | zero; pode inlinear | um salto indireto |
| Tamanho do binário | cresce | uma versão só |
| Lista heterogênea? | **não** | **sim** |

**Como escolher:** se os tipos são conhecidos em cada ponto de chamada, use genérico/`impl Trait`
(mais rápido). Se você precisa **guardar tipos diferentes juntos** ou decidir em runtime,
use `dyn`.

Foi exatamente essa a divisão: `sole_spot` recebe uma closure por chamada (estático), enquanto
o solver guarda `NakedSingle` **e** `HiddenSingle` na mesma `Vec` (dinâmico) — e a lista vai
crescer nas fatias Médio/Difícil sem o solver mudar uma linha.

## 5. `Box`, o heap e tipos sem tamanho conhecido

Rust precisa saber o **tamanho** de tudo que vai na stack. Tipos assim são `Sized`; a maioria
é. Mas `dyn Technique` significa "algum tipo que implementa a trait" — pode ser uma struct
vazia ou uma de 200 bytes. É **unsized**, e não cabe direto numa variável.

**`Box<T>`** é um ponteiro **dono** de um valor no heap:

- o `Box` em si tem tamanho conhecido → pode ir na stack, dentro de uma `Vec`, etc.;
- quando o `Box` sai de escopo, o valor no heap é liberado junto — RAII, sem GC;
- `Box<dyn Trait>` é um **fat pointer**: duas palavras, uma para o dado e outra para a
  **vtable** (a tabela com os endereços dos métodos daquele tipo concreto).

```
Box<dyn Technique>  ─┬─→  o dado (a struct, no heap)
                     └─→  a vtable (onde está o apply desse tipo)
```

`Box::new(NakedSingle)` produz `Box<NakedSingle>`, que vira `Box<dyn Technique>`
automaticamente (*unsized coercion*) porque o tipo do destino já está declarado.

**Stack vs heap, resumido:** stack é rápida e de tamanho fixo, liberada ao sair do escopo;
heap é flexível e um pouco mais cara. `[Cell; 81]` mora na stack; `Vec`, `String` e `Box`
guardam seus dados no heap.

## 6. Closures a fundo: captura e a família `Fn`

```rust
|i| row_of(i) == unit
```

Uma **closure** é uma função anônima que **captura** variáveis do escopo onde foi escrita —
aqui, `unit`, que veio do loop externo. É desse "fechar em volta do ambiente" que vem o nome.
Cada volta do loop cria uma closure carregando o `unit` daquela volta.

Toda closure tem um tipo **anônimo e único** — não existe nome para escrever na assinatura. O
que se declara é a **trait** que ela satisfaz:

| Trait | A closure… | Chamável |
|-------|-----------|----------|
| `Fn` | só **lê** o que capturou | várias vezes |
| `FnMut` | **modifica** o que capturou | várias vezes |
| `FnOnce` | **consome** o que capturou | uma vez só |

O compilador escolhe sozinho a mais permissiva que o corpo permitir. Quem **recebe** a closure
é que decide o que exigir — peça a menos restritiva que resolve (`FnOnce` < `FnMut` < `Fn` em
exigência sobre a closure).

Como `unit` é um `usize` (tipo `Copy`), a captura é uma cópia barata e nenhum empréstimo fica
pendurado. Se a closure capturasse algo não-`Copy`, entrariam em cena `move` e lifetimes.

**Por que closures importam aqui:** a mesma `sole_spot` serve para linha, coluna e box porque
"como decidir se um índice pertence à unidade" virou um **parâmetro**. Sem isso, seriam três
funções quase idênticas.

## 7. Lifetimes na prática: `&'static str` vs `String`

```rust
pub technique: &'static str,       // "Naked Single"
```

Toda referência em Rust tem um **tempo de vida** (lifetime): por quanto tempo o dado apontado
é válido. Quase sempre o compilador infere e você não escreve nada. `'static` é o caso
extremo: "vive enquanto o programa viver".

Literais de texto (`"Naked Single"`) estão **embutidos no binário**, logo já são
`&'static str`. Guardar a referência não aloca nada e nunca "expira".

| | `&str` | `String` |
|---|---|---|
| O que é | **empréstimo** de um texto | texto **dono**, no heap |
| Aloca? | não | sim |
| Pode crescer? | não | sim (`push_str`) |
| Use quando | só lê; nome fixo conhecido em compilação | monta/modifica em runtime |

Regra prática: **parâmetro** de função → `&str` (aceita os dois); **campo** que guarda nome
fixo → `&'static str`; texto construído em runtime → `String`.

## 8. `Option` como ferramenta de fluxo

Nas tasks 1–5, `Option` era "pode não ter valor". Aqui ele virou **ferramenta de controle de
fluxo**, em três papéis:

**(a) "não se aplica" ≠ erro.**

```rust
fn apply(&self, grid: &Grid) -> Option<Deduction>
```

Uma técnica não achar jogada não é falha — por isso `Option`, e não `Result`. `Result` é para
o que deu **errado**; `Option` para o que legitimamente **pode não existir**.

**(b) acumulador de três estados**, sem flag booleana auxiliar:

```rust
let mut found: Option<usize> = None;
...
if found.is_some() { return None; }   // achou um segundo => não é único
found = Some(i);
```

`None` no fim = nenhum lugar; `Some(i)` no fim = exatamente um; segundo achado = curto-circuito.

**(c) encadeamento de tentativas:**

```rust
for technique in &self.techniques {
    if let Some(d) = technique.apply(grid) { return Some(d); }
}
None
```

Métodos úteis: `.is_some()` / `.is_none()` (perguntam sem desembrulhar), `.unwrap()` (assume,
dá panic — só em teste), `if let Some(x)` (extrai o dado), `match` (trata os dois casos de
forma exaustiva).

## 9. `&mut`, deref e mutação contida

```rust
fn solve_recursive(grid: &mut Grid, limit: usize, count: &mut usize)
```

**`&T`** empresta para ler; **`&mut T`** empresta **exclusivamente** para modificar. A regra do
borrow checker: **N referências `&`** OU **uma única `&mut`**, nunca as duas ao mesmo tempo.

Na recursão isso funciona porque o `&mut` é repassado **um nível por vez**: enquanto a chamada
filha o tem, a mãe está parada esperando. Um único tabuleiro atravessa toda a árvore de busca
— nada de clonar 81 células por nó.

- **`*count += 1`** — o `*` é o **deref**: `count` é um ponteiro para o número; `*count` é o
  número. Ler também exige (`if *count >= limit`).
- **Por que o contador é `&mut` e não retorno?** Porque todos os ramos da recursão precisam
  **ver o mesmo** contador para abortar assim que o limite é atingido. Um valor somado a
  posteriori não permitiria a poda.

### O padrão "clonar na entrada"

```rust
pub fn count_solutions(grid: &Grid, limit: usize) -> usize {
    let mut work = grid.clone();       // cópia própria
    ...
}
```

A API pública recebe `&Grid` (só-leitura) e clona internamente; a mutação fica **contida**
dentro da função. Quem chama nunca tem seu tabuleiro alterado por baixo. O `solve` do
`LogicalSolver` faz igual. É a forma idiomática de oferecer uma API imutável por fora e
mutável por dentro.

Três usos da palavra `mut`, que confundem no começo:

| Forma | Onde | Significa |
|---|---|---|
| `let mut x` | variável local | posso reatribuir/mutar **esta variável** |
| `&mut T` | parâmetro/empréstimo | empresto **exclusivamente** para mutar |
| `&mut self` | método | o método muta a instância |

## 10. Recursão e backtracking

**Backtracking** = tentar uma opção, explorar o que vem depois, e **desfazer** se não der.

```rust
grid.set(index, Cell::Filled(value));   // 1. tenta
solve_recursive(grid, limit, count);    // 2. explora o resto
grid.set(index, Cell::Empty);           // 3. DESFAZ
```

Sem o passo 3 o tabuleiro ficaria sujo com tentativas fracassadas e os ramos seguintes
partiriam de um estado errado.

Toda recursão precisa de **caso base**. Aqui é "não há mais célula vazia":

```rust
let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
match next {
    None => *count += 1,          // caso base: solução completa
    Some(index) => { /* passo recursivo */ }
}
```

- **`.find()`** devolve `Option` com o primeiro elemento que satisfaz o predicado; é *lazy*,
  para na primeira célula vazia.
- **Poda (`limit`)** é o que torna a função utilizável: uma grade vazia tem ~6,67 × 10²¹
  soluções. Com `limit = 2`, a pergunta vira decidível e roda em milissegundos.

| `count_solutions(g, 2)` | Significa |
|---|---|
| `0` | impossível (contradição) |
| `1` | **solução única** — puzzle válido |
| `2` | ambíguo (≥2 soluções) |

### Dois solvers, duas perguntas

| | `LogicalSolver` (Task 9) | `count_solutions` (Task 10) |
|---|---|---|
| Como | aplica técnicas dedutivas | tenta 1–9 e volta atrás |
| Responde | "dá para resolver **pensando**?" | "**quantas** soluções existem?" |
| Serve para | dicas, passo-a-passo, dificuldade | garantir unicidade |
| Pode empacar | sim (`solved: false`) | não |

Nenhuma técnica lógica prova unicidade; só a busca exaustiva prova. Por isso os dois convivem
— e o gerador (Task 12) usa ambos.

## 11. Organizando módulos: pasta, `mod.rs` e `pub use`

Um módulo com **filhos** deixa de ser um arquivo e vira uma pasta com `mod.rs`:

```
src/
  lib.rs                 → mod techniques;
  techniques/
    mod.rs               → mod naked_single;  +  pub use naked_single::NakedSingle;
    naked_single.rs
    hidden_single.rs
```

- `mod x;` **declara** o módulo. Sem essa linha o arquivo **não é compilado** — Rust não tem
  "auto-import por pasta" como o `index.js`/barrel do JS.
- **`pub use ...`** é um **re-export**: de fora escreve-se `techniques::NakedSingle` em vez de
  `techniques::naked_single::NakedSingle`. A estrutura de arquivos para de vazar na API.
- Itens **sem `pub`** são privados ao módulo. `sole_spot` e `deduction` são helpers internos:
  existem para o código ficar legível, sem entrar na API.
- Enquanto ninguém **consome** um `pub use`, o compilador avisa `unused import` — foi o que
  aconteceu entre as tasks 7 e 9, até o solver usar as técnicas.

**Visibilidade, resumida:** privado é o padrão; `pub` abre para quem enxerga o módulo. (Há
também `pub(crate)`, "público só dentro do crate" — vamos usar quando a API pública for
desenhada na Task 13.)

## 12. Padrões que apareceram

- **Strategy** — a trait `Technique` com implementações intercambiáveis, guardadas numa lista.
  Adicionar uma técnica nova não altera o solver; é o *open/closed principle* na prática.
- **Newtype** — `CandidateSet(u16)`: significado e encapsulamento com custo zero.
- **Construtor por convenção** — `new()` não é palavra-chave, é convenção; se não tem
  argumentos, o idiomático (e o Clippy, via lint `new_without_default`) pede também um
  `impl Default`.
- **Clonar na entrada** — API imutável por fora, mutação contida por dentro.
- **Guard clause** — `if !in_unit(i) { continue; }` descarta cedo em vez de aninhar mais um
  nível de `if`.

## 13. Tabela-resumo dos conceitos

| Conceito | Essência | Exemplo nosso |
|----------|----------|---------------|
| Newtype / tuple struct | embrulhar primitivo para dar significado | `CandidateSet(u16)` |
| Operações de bit | conjunto dentro de um inteiro, sem alocar | `all & !used`, `count_ones()` |
| Trait (própria) | contrato de comportamento, `impl X for Y` | `Technique::apply` |
| Unit struct | tipo sem dados, só para pendurar `impl` | `NakedSingle`, `HiddenSingle` |
| Dispatch estático | genérico monomorfizado, custo zero | `impl Fn(usize) -> bool` |
| Dispatch dinâmico | vtable em runtime, lista heterogênea | `Vec<Box<dyn Technique>>` |
| `Box<T>` | ponteiro dono no heap; dá tamanho ao unsized | `Box::new(NakedSingle)` |
| Closure + família `Fn` | função anônima que captura o ambiente | `\|i\| row_of(i) == unit` |
| `&'static str` | referência a texto que vive o programa todo | `technique: "Naked Single"` |
| `Option` como fluxo | ausência legítima, acumulador, curto-circuito | `sole_spot`, `next_step` |
| `&mut` + deref | empréstimo exclusivo para mutar; `*x` lê/escreve | `solve_recursive` |
| Recursão + backtracking | tentar, explorar, desfazer; caso base; poda | `count_solutions` |
| Módulo em pasta + `pub use` | `mod.rs` organiza; re-export esconde arquivos | `techniques/` |
