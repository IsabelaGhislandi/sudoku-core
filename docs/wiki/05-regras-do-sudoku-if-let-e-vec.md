# 05 — Regras do Sudoku: índice→coordenada, `if let` e `Vec`

> Referente à **Task 5** do plano. Commit: `feat: add unit helpers, can_place and validate`.

Aqui o `Grid` passa a "entender" as regras: duas células se **veem** quando compartilham
**linha**, **coluna** ou **box** (bloco 3×3).

## Índice linear → coordenada

As 81 células são indexadas de `0..80`. Convertendo:

```rust
fn row_of(i) -> i / 9          // divisão inteira
fn col_of(i) -> i % 9          // resto
fn box_of(i) -> (row/3)*3 + col/3
```

- `row_of(40) = 4`, `col_of(40) = 4`, `box_of(40) = 4` (centro).
- índice `3` → linha 0, coluna 3 → **box 1** (segundo bloco da primeira faixa).

Guardar como array linear `[Cell; 81]` e converter sob demanda é mais simples que uma
matriz `[[Cell;9];9]`, e essas três funções são reusadas por candidatos, técnicas e solver.

## `if let` — desembrulhar uma variante

```rust
if let Cell::Filled(v) = self.cells[i] {
    // só entra aqui se for Filled; `v` já é o dígito
}
```

`if let padrão = valor { ... }` é um `match` enxuto para **um** caso: executa o bloco só
quando o valor casa com o padrão, **extraindo** os dados de dentro (aqui, o dígito `v`).
É o jeito idiomático de tratar "se estiver preenchida, pegue o número".

## `can_place` — a checagem de jogada legal

Varre as outras 80 células; se alguma já tem o **mesmo valor** numa unidade compartilhada
(linha/coluna/box), a jogada é inválida. O `continue` pula a própria célula:

```rust
for i in 0..81 {
    if i == index { continue; }
    if let Cell::Filled(v) = self.cells[i] {
        if v == value && (mesma_linha || mesma_coluna || mesmo_box) {
            return false;
        }
    }
}
true
```

## `validate` e `Vec<Conflict>`

`validate` devolve **todos** os pares conflitantes. O laço interno começa em `j = i + 1`
para contar cada par **uma única vez** (conflito é simétrico: o par `{0,1}` não repete como
`{1,0}`):

```rust
for i in 0..81 {
    for j in (i+1)..81 { /* compara i com j */ }
}
```

`Vec<T>` é a lista dinâmica do Rust (cresce na *heap*), análoga a um array de JS — diferente
do `[Cell; 81]`, cujo tamanho é fixo em compilação. Criamos com `Vec::new()` e adicionamos
com `.push(...)`.

## `struct Conflict { pub a, pub b }`

Uma `struct` de dados simples. Os campos são `pub` para poderem ser lidos de fora do módulo
(e comparados com `==` nos testes, via `derive(PartialEq)`).

## Nota sobre `dead_code`

Os warnings de "nunca usado" diminuem quando o código passa a se referenciar (os helpers se
usam entre si e dentro do módulo). Os de `Grid`/`Cell` só somem de vez quando a API pública
for re-exportada na Task 13.
