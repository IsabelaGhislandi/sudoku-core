# Fundamentos de Rust — consolidado das Tarefas 1 a 5

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo),
> a partir de tudo que implementamos até a Task 5. Geramos um destes **a cada 5 tarefas**.
> As páginas por-task ([01](../01-scaffold-crate-e-editions.md) … [05](../05-regras-do-sudoku-if-let-e-vec.md))
> contam a história na ordem do código; **este** documento ensina os fundamentos por trás.

Sumário:
1. [Projeto: crate, módulos e Cargo](#1-projeto-crate-módulos-e-cargo)
2. [Atributos vs. dependências (e macros)](#2-atributos-vs-dependências-e-macros)
3. [Variáveis, mutabilidade e tipos escalares](#3-variáveis-mutabilidade-e-tipos-escalares)
4. [Ownership e borrowing — o coração do Rust](#4-ownership-e-borrowing--o-coração-do-rust)
5. [Dados: struct, enum (tipo soma) e `impl`](#5-dados-struct-enum-tipo-soma-e-impl)
6. [Traits e `derive`](#6-traits-e-derive)
7. [Sem `null`, sem exceptions: `Option` e `Result`](#7-sem-null-sem-exceptions-option-e-result)
8. [Pattern matching: `match`, `if let`, `matches!`](#8-pattern-matching-match-if-let-matches)
9. [Iteradores e closures](#9-iteradores-e-closures)
10. [Coleções: array fixo vs `Vec` (stack vs heap)](#10-coleções-array-fixo-vs-vec-stack-vs-heap)
11. [Testes embutidos](#11-testes-embutidos)
12. [Tabela-resumo dos conceitos](#12-tabela-resumo-dos-conceitos)

---

## 1. Projeto: crate, módulos e Cargo

- **Crate** = unidade de compilação (o "projeto/pacote"). O nosso é uma **biblioteca**
  (`--lib`), cuja raiz é `src/lib.rs`.
- **`Cargo.toml`** = manifesto (nome, versão, `edition`, dependências). Análogo ao `package.json`.
- **`Cargo.lock`** = versões exatas resolvidas. Para bibliotecas, não versionamos.
- **`cargo`** ≈ `npm`: `cargo new`, `cargo build`, `cargo test`.
- **Módulos** montam a árvore do crate a partir de arquivos:

```rust
// lib.rs
mod difficulty;  // "existe um módulo difficulty, em difficulty.rs"
mod grid;        // privado: visível dentro do crate, fora da API pública
```

- Sem `pub`, o módulo é **privado** ao crate → daí os warnings de `dead_code` enquanto a
  API pública não é re-exportada (Task 13).
- Rust **não exige declarar antes de usar**: a ordem das definições no arquivo é livre.

## 2. Atributos vs. dependências (e macros)

**Atributo** = anotação que instrui o **compilador**. **Não é** dependência.

```rust
#[derive(Debug, Clone)]   // atributo
#[test]                   // atributo
#[cfg(test)]              // atributo (compilação condicional)
```

**Dependência** = biblioteca externa baixada, declarada no `Cargo.toml`:

```toml
[dependencies]
rand = "0.10"
```

| | Atributo | Dependência |
|---|---|---|
| Categoria | metadado/anotação da linguagem | crate externo |
| Onde mora | no código, com `#[...]` | em `Cargo.toml` |
| Precisa baixar? | não | sim |

- **`derive`** é uma **macro** que **gera código** em tempo de compilação. Os derives básicos
  (`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`, `Default`) são da
  **std** → zero dependência.
- ⚠️ Alguns derives vêm de crates (ex.: `#[derive(Serialize)]` exige `serde`). Derivar **não**
  é uma dependência, mas um derive específico **pode** precisar de uma.
- **Macros** ≠ funções: terminam com `!` quando invocadas (`assert_eq!`, `matches!`, `vec!`,
  `println!`). Elas expandem para código antes da compilação de fato.

## 3. Variáveis, mutabilidade e tipos escalares

```rust
let grid = Grid::empty();        // imutável por padrão
let mut grid = Grid::empty();    // mut = pode mudar
```

- **`let` cria variável imutável por padrão.** Para mudar o valor (ou chamar `&mut self`),
  precisa de **`let mut`**. Imutabilidade é o default — segurança primeiro.
- Tipos escalares usados:
  - **`u8`** — inteiro 0–255 sem sinal (os dígitos 1–9 em `Cell::Filled(u8)`).
  - **`usize`** — inteiro de indexação (tamanho da máquina); **sempre** que for índice de array.
  - **`bool`** — `true`/`false`.
  - **`char`** — um escalar Unicode de 4 bytes (não um byte!).
- **Tupla** agrupa valores de tipos quaisquer: `(usize, usize, usize)`. Desestrutura com
  `let (r, c, b) = (...)`.
- **Retorno implícito:** a **última expressão sem `;`** é o valor de retorno.

```rust
pub fn empty() -> Self {
    Grid { cells: [Cell::Empty; 81] }  // sem ; e sem return → é o retorno
}
```

## 4. Ownership e borrowing — o coração do Rust

Cada valor tem **um dono**. Quando você passa o valor, por padrão a **posse se move**
(o original deixa de ser usável). Para emprestar sem mover, usa-se **referências**:

| Forma | Significado | No código |
|------|-------------|-----------|
| `&self` / `&T` | empréstimo **imutável** (só leitura) | `get`, `is_complete`, `validate(grid: &Grid)` |
| `&mut self` / `&mut T` | empréstimo **mutável** (exclusivo) | `set` |
| `self` / `T` | toma **posse** (move/consome) | `chars.into_iter()` |

**Regra do borrow checker** (verificada em compilação): a qualquer instante você tem
**N referências `&`** OU **uma única `&mut`**, nunca as duas. Isso elimina data races e
*use-after-free* sem garbage collector.

- Consequência prática: como `set` é `&mut self`, a variável precisa ser `let mut grid`.
- **`*` (deref):** ao iterar por referência, você recebe `&u8`; `*n` "desreferencia" para
  pegar o `u8` em si (visto em `to_line`: `(b'0' + *n)`).
- **`Copy` muda esse jogo:** tipos pequenos marcados `Copy` (como `Cell`, `Difficulty`,
  `Conflict`) são **copiados** em vez de movidos — usar o valor não o invalida. `Grid` **não**
  é `Copy` (array de 81), então duplicar exige `.clone()` explícito.

## 5. Dados: struct, enum (tipo soma) e `impl`

Rust **não tem classes**. Separa **dados** de **comportamento**:

```rust
pub struct Grid { cells: [Cell; 81] }   // dados agrupados (campo privado = encapsulado)

pub enum Cell {                          // tipo SOMA: é UMA das variantes
    Empty,                               // sem dado
    Filled(u8),                          // variante que CARREGA um dado
}

impl Grid {                              // comportamento (métodos) vai no impl
    pub fn empty() -> Self { ... }       // sem self = associated function (construtor)
    pub fn get(&self, i: usize) -> Cell { ... }   // com &self = método
}
```

- **`struct`** = registro de campos nomeados. Campo **sem `pub`** é privado (encapsulamento):
  `Grid.cells` só é tocado pelos métodos, garantindo sempre 81 células.
- **`enum`** = tipo **soma**: o valor é **exatamente uma** variante; variantes podem **carregar
  dados** (`Filled(u8)`, `WrongLength(usize)`). É o "OU" da modelagem.
- **`impl`** = bloco de implementação. Pode haver **vários `impl` para o mesmo tipo** (temos 3
  para `Grid`); o compilador junta. Função **com `self`** = método (`grid.get(i)`); **sem
  `self`** = associated function (`Grid::empty()`).
- **`Self`** (maiúsculo) = apelido para "o tipo deste `impl`".

## 6. Traits e `derive`

- **Trait** = um contrato/comportamento que tipos podem implementar (≈ "interface").
  `Debug`, `Clone`, `Ord` são traits da std.
- **`#[derive(...)]`** gera a implementação desses traits automaticamente:

| Derive | Habilita | Por quê |
|--------|----------|---------|
| `Debug` | imprimir com `{:?}` | `assert_eq!` precisa imprimir ao falhar |
| `Clone` | cópia explícita `.clone()` | duplicar `Grid` |
| `Copy` | cópia implícita barata | tipos pequenos (`Cell`, `Difficulty`) |
| `PartialEq`/`Eq` | `==` / `!=` | comparar nos testes |
| `PartialOrd`/`Ord` | `<`, `>`, `.max()`, ordenar | comparar `Difficulty` |

- **Truque do `Ord`:** o `Ord` derivado ordena pela **ordem de declaração** das variantes.
  Declarar `Facil, Medio, Dificil, MuitoDificil` já dá `Facil < Medio < ...` de graça.
- **`Eq` vs `PartialEq`:** `Eq` promete igualdade total (todo valor igual a si mesmo);
  `f64` tem `PartialEq` mas não `Eq` (por causa do `NaN`).

## 7. Sem `null`, sem exceptions: `Option` e `Result`

Rust não tem `null` nem `try/catch`. Ausência e falha são **valores tipados**:

```rust
enum Option<T> { Some(T), None }          // ausência
enum Result<T, E> { Ok(T), Err(E) }       // sucesso ou falha
```

- **`Option<T>`** no lugar de `value | null`:

```rust
niveis.iter().max()           // -> Option<&Difficulty>  (pode estar vazio => None)
```

- **`Result<T, E>`** no lugar de `throw`:

```rust
pub fn from_line(line: &str) -> Result<Grid, ParseError> {
    if chars.len() != 81 {
        return Err(ParseError::WrongLength(chars.len()));  // "throw" tipado
    }
    ...
    Ok(grid)                                               // sucesso
}
```

- O compilador **obriga** a tratar os dois casos. Atalhos: `.unwrap()` (assume sucesso, dá
  *panic* se não for — só em teste), `?` (propaga o erro), ou `match` completo.

| Web | Rust |
|-----|------|
| `value \| null` | `Option<T>` (`Some`/`None`) |
| `throw new Error()` | `return Err(...)` |
| `try { } catch (e) { }` | `match r { Ok(v)=>.., Err(e)=>.. }` |

## 8. Pattern matching: `match`, `if let`, `matches!`

Três formas, do mais completo ao mais enxuto:

```rust
// match: exaustivo, é uma EXPRESSÃO (produz valor)
let cell = match ch {
    '.' | '0'  => Cell::Empty,                   // | = "ou"
    '1'..='9'  => Cell::Filled(ch as u8 - b'0'), // ..= = range inclusivo
    other      => return Err(ParseError::InvalidChar(other)), // captura o resto
};

// if let: trata UM caso e extrai o dado
if let Cell::Filled(v) = self.cells[i] {  // só entra se Filled; v = o dígito
    ...
}

// matches!: padrão -> bool
self.cells.iter().all(|c| matches!(c, Cell::Filled(_)))  // _ = qualquer valor
```

- **`match` é exaustivo:** o compilador exige cobrir **todos** os casos (sem "default
  esquecido"). E é **expressão**: o braço escolhido vira o valor.
- **`if let`** = um `match` de um caso só, idiomático para "se for essa variante, pegue o dado".
- **`matches!`** = macro que devolve `true`/`false` se o valor casa com o padrão.
- Conversão char↔dígito: **`b'0'`** é o byte de `'0'` (48); `ch as u8 - b'0'` vira número,
  `b'0' + n` volta a char. **`as`** é *cast* explícito.

## 9. Iteradores e closures

```rust
self.cells.iter().all(|c| matches!(c, Cell::Filled(_)))
self.cells.iter().map(|c| /* char */).collect()
chars.into_iter().enumerate()        // -> (índice, valor)
```

- **`.iter()`** itera por referência (`&T`); **`.into_iter()`** itera **consumindo** (dá posse).
- **Adaptadores** (lazy): `.map()`, `.filter()`, `.enumerate()`. **Consumidores**: `.all()`,
  `.collect()`, `.max()`, `.count()`.
- **Closure** `|x| corpo` = função anônima (≈ arrow function). Captura variáveis do redor.
- **`.collect()`** materializa o iterador numa coleção; o tipo-alvo vem da anotação/retorno
  (`let chars: Vec<char> = ...collect();` ou `-> String`).
- Iteradores em Rust são **zero-cost**: compilam para algo tão rápido quanto um laço manual.

## 10. Coleções: array fixo vs `Vec` (stack vs heap)

```rust
cells: [Cell; 81]      // array FIXO: tamanho no tipo, vive na STACK
Vec<Conflict>          // vetor DINÂMICO: cresce na HEAP
```

| | `[T; N]` | `Vec<T>` |
|---|---|---|
| Tamanho | fixo, parte do tipo | dinâmico |
| Memória | stack | heap |
| Quando | quantidade conhecida e fixa (81 células) | quantidade variável (lista de conflitos) |

- Criar: `[valor; N]` (repete `valor`, exige `Copy`) vs `Vec::new()` + `.push(...)` ou
  a macro `vec![...]`.
- Indexar array fora do limite → **panic** (Rust checa limites: segurança de memória).

## 11. Testes embutidos

```rust
#[cfg(test)]              // só compila em `cargo test`
mod tests {
    use super::*;         // traz o módulo pai pro escopo

    #[test]               // marca um teste
    fn nome_do_teste() {
        assert!(cond);            // panic se false
        assert_eq!(a, b);         // panic se a != b (imprime os dois lados)
    }
}
```

- Testes ficam **inline**, no mesmo arquivo do código (convenção `mod tests`).
- **`#[cfg(test)]`** = compilação condicional → custo zero no binário final; por isso os
  itens usados **só** em teste ainda contam como `dead_code` num `cargo build`.
- Rodar um subconjunto: `cargo test grid` (filtra por nome).

## 12. Tabela-resumo dos conceitos

| Conceito | Essência | Exemplo nosso |
|----------|----------|---------------|
| Ownership & Borrow | 1 dono; `&` lê, `&mut` escreve; nunca os dois | `get`/`set`, `&Grid` |
| Enum soma + `match` | valor é *uma* variante; `match` exaustivo | `Cell`, `Difficulty`, `ParseError` |
| `Option`/`Result` | ausência e falha sem `null`/exceptions | `from_line`, `.max()` |
| Traits & `derive` | contratos gerados de graça | topo de cada tipo |
| Iteradores & closures | pipelines lazy e zero-cost | `is_complete`, `to_line` |
| Array fixo vs `Vec` | stack fixo vs heap dinâmico | `cells` vs `validate` |
| `impl` | dados (struct/enum) separados do comportamento | os 3 `impl Grid` |
| Atributo ≠ dependência | anotação ao compilador vs crate externo | `#[derive]` vs `rand` |
