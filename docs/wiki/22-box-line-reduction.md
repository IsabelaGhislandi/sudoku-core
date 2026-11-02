# 22 — Box/Line Reduction: fatias, `copied` e o primeiro nível Difícil

> Referente à Task 1 do plano da fatia Difícil. Commit: `feat: add Box/Line Reduction technique`.

Numa linha (ou coluna), se todas as células candidatas a um dígito estão no mesmo box, o dígito daquela linha
fica dentro desse box. Logo, ele sai do resto do box. É o caminho inverso do Pointing Pair, que parte do box e
limpa a linha.

```
linha 0:  [5?][5?][ . ] | [ . ][ . ][ . ] | [ . ][ . ][ . ]     o 5 da linha 0 só cabe no box 0
          ───────────────
box 0:    as outras células (linhas 1 e 2) perdem o 5
```

## Por que ela é Difícil, e não Médio

Muitos guias põem esta técnica ao lado do Pointing Pair. Aqui ela é `Difficulty::Dificil` por **contrato**: o
gerador de Médio cava com `LogicalSolver::up_to(Medio)`. Se a técnica tivesse nível `Medio`, ela entraria nesse
solver, mudaria quais células podem ser cavadas e alteraria **todos** os puzzles Médios já gerados.

Ela também entra no **fim** de `all_techniques()`. Como o solver tenta sempre do início, só chega a ela quando
todas as técnicas de Fácil e Médio empacam. Para um puzzle Fácil ou Médio, isso nunca acontece, então `rate` dá o
mesmo resultado de antes.

## Fatias de array: `&unidades[..18]`

```rust
let unidades = units();              // [[usize; 9]; 27]
for linha in &unidades[..18] { ... } // só as linhas e colunas
```

- `units()` devolve um **array** de tamanho fixo. `[..18]` pede uma **fatia** (`&[[usize; 9]]`) com os 18
  primeiros elementos, sem copiar nada: é uma visão emprestada do array.
- `..18` é um `RangeTo`, o mesmo tipo de intervalo usado em `for i in 0..9`, só que sem início.
- Em JavaScript seria `unidades.slice(0, 18)`, mas lá o `slice` **copia**. Em Rust, a fatia só aponta para o
  pedaço original.

## `iter().copied()`

```rust
let celulas: Vec<usize> = linha.iter().copied().filter(|&i| ...).collect();
```

`iter()` em um `[usize; 9]` produz **referências** (`&usize`). `copied()` transforma cada uma no valor (`usize`),
o que é barato para tipos `Copy` como números. Sem ele, o `Vec` seria de `&usize`, e o padrão `|&i|` do `filter`
precisaria de mais um `&`.

## `contains` em array

```rust
!linha.contains(&i)
```

Arrays e fatias têm `contains`, que recebe uma **referência** ao valor procurado. Para 9 elementos, uma busca
linear é mais simples e rápida do que montar um conjunto.

## A invariante de progresso, de novo

Como nas técnicas de Médio, a dedução só sai se a lista de pares não estiver vazia. Um box em que o dígito já
saiu de todas as outras células daria `Eliminate(vec![])`, e o solver repetiria esse passo para sempre. O teste
`retorna_none_quando_nao_ha_o_que_eliminar` trava essa regra.
