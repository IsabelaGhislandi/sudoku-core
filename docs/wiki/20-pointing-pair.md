# 20 — Pointing Pair e funções como parâmetro

> Referente à Task 5 do plano da fatia Médio. Commit: `feat: add Pointing Pair technique`.

**A técnica:** se, dentro de um box, o 5 só cabe em células de uma mesma linha, então o 5 desse box está
nessa linha. Logo, ele não pode estar no resto da linha, fora do box. O mesmo vale para colunas.

O nome fala em par, mas a técnica também vale para 3 células alinhadas, por isso o teste
`aceita_tres_celulas_alinhadas`. Com uma célula só, já seria um Hidden Single.

## Função como parâmetro: `fn` vs. `impl Fn`

```rust
fn fora_do_box(/* ... */, linha_de: fn(usize) -> usize) -> Option<Vec<(usize, u8)>>

fora_do_box(candidates, value, b, &celulas, row_of)
```

Linha e coluna usam a mesma lógica, mudando só "como saber a qual linha a célula pertence". Por isso a
função recebe `row_of` ou `col_of` como parâmetro. No `sole_spot`, usamos `impl Fn`. Qual a diferença?

| | `fn(usize) -> usize` | `impl Fn(usize) -> usize` |
|---|---|---|
| Aceita | função comum, ou closure que **não captura** nada | qualquer closure |
| Representação | um ponteiro de função | monomorfizada, uma cópia por tipo |
| Aqui | basta, `row_of` e `col_of` não capturam nada | seria necessária se capturasse algo |

O `sole_spot` precisa de `impl Fn` porque suas closures capturam `unit`.

## `or_else`: tente isto, senão aquilo

```rust
let pares = fora_do_box(candidates, value, b, &celulas, row_of)
    .or_else(|| fora_do_box(candidates, value, b, &celulas, col_of));
```

`or_else` só chama a closure se o primeiro resultado for `None`. É como o `??` do JavaScript, em que o lado
direito também só é avaliado quando preciso. Com `.or(valor)`, os dois lados seriam calculados sempre.

## `then_some`: de `bool` para `Option`

```rust
(!pares.is_empty()).then_some(pares)
```

Devolve `Some(pares)` se a condição for verdadeira e `None` se não. É um atalho para
`if cond { Some(x) } else { None }`. O valor é calculado **antes** da chamada. Para adiar um cálculo caro, existe
`.then(|| ...)`.

## Faixa como conjunto: `(2..=3).contains(...)`

```rust
if !(2..=3).contains(&celulas.len()) { continue; }
```

Uma faixa é um valor comum em Rust, com métodos próprios. `contains` recebe uma **referência**, daí o `&`.

## Por que isso importa

Com as três técnicas prontas, o solver já consegue resolver puzzles de Médio. Falta o gerador produzi-los, na Task
21. E `solve` não mudou nenhuma linha desde que a primeira delas entrou.
