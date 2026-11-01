# 18 — Naked Pair e a iteração por unidades

> Referente à Task 3 do plano da fatia Médio. Commit: `feat: add Naked Pair technique`.

**A técnica:** se duas células da mesma unidade (linha, coluna ou box) só podem ser `{1, 2}`, então o 1 e o
2 ficam com elas, mesmo sem saber em que ordem. Logo, nenhuma outra célula da unidade pode ter esses
dígitos. É a primeira técnica do projeto que **não coloca valor**, só remove candidatos.

## `units()`: dois `from_fn` aninhados

```rust
pub(crate) fn units() -> [[usize; 9]; 27] {
    std::array::from_fn(|u| {
        let mut cells = (0..81).filter(|&i| match u {
            0..=8 => row_of(i) == u,
            9..=17 => col_of(i) == u - 9,
            _ => box_of(i) == u - 18,
        });
        std::array::from_fn(|_| cells.next().expect("toda unidade tem 9 células"))
    })
}
```

- **O tipo de retorno comanda tudo.** O `from_fn` externo cria 27 itens e o interno, 9, ambos inferidos
  de `[[usize; 9]; 27]`.
- **Padrão de faixa no `match`:** `0..=8` cobre as linhas e `_` cobre o resto, as boxes. O `match` precisa
  ser exaustivo, e `_` resolve isso.
- **Por que `.next()` e não `.collect()`?** `collect` não produz arrays de tamanho fixo, pois o iterador
  poderia ter outro número de itens. Daí puxar um item por vez.
- **`.expect`** em código de produção é aceitável aqui: se a unidade não tiver 9 células, é **bug** no
  próprio código, e não uma situação que quem chama poderia tratar.

## `pub(crate)`: finalmente em uso

Sem `pub(crate)`, `units` só seria visível em `techniques` e nos módulos filhos dele, que já é o suficiente
para hoje. Por que usar então? Porque ele **documenta a intenção**: é uma ferramenta interna do crate e
nunca fará parte da API. E o `lib.rs` não a re-exporta.

## A cadeia que monta os pares

```rust
let pares: Vec<(usize, u8)> = unit
    .iter()
    .filter(|&&i| i != a && i != b)
    .flat_map(|&i| par.values().into_iter().map(move |v| (i, v)))
    .filter(|&(i, v)| candidates.get(i).contains(v))
    .collect();
```

Acompanhe o tipo a cada elo:

| Elo | Item resultante |
|---|---|
| `.iter()` | `&usize` |
| `.filter(\|&&i\| ...)` | `&usize`: o `filter` recebe `&&usize`, daí os dois `&` no padrão |
| `.flat_map(...)` | `(usize, u8)`: cada célula vira vários pares, em um único iterador plano |
| `.filter(\|&(i, v)\| ...)` | `(usize, u8)` |

`flat_map` é o mesmo `Array.prototype.flatMap` do JS, mas preguiçoso. O **`move`** é necessário porque a
closure interna (`|v| (i, v)`) sobrevive à externa, que termina a cada célula. Sem `move`, ela guardaria
uma referência a `i`, uma variável que já teria acabado. Com `move`, ela leva uma cópia.

## `let ... else`

```rust
let Action::Eliminate(pares) = d.action else {
    panic!("deveria eliminar");
};
```

Usado nos testes, ele desembrulha a variante ou sai do bloco. O ramo `else` **precisa** divergir (com
`panic!`, `return` etc.), e é assim que o compilador garante que `pares` existe no restante da função. É
como um `if let` invertido, sem o aninhamento.

## A invariante de progresso

```rust
if !pares.is_empty() {
    return Some(Deduction { /* ... */ });
}
```

O par continua existindo depois de aplicado. Sem esse teste, a técnica encontraria o mesmo par para sempre,
e o solver, que agora pode dar um passo sem colocar nenhum valor, entraria em **loop infinito**. Por isso a
regra: uma técnica só responde se muda algo. O teste `retorna_none_quando_nao_ha_o_que_eliminar` protege essa
regra.

## Comparando conjuntos com `!=`

```rust
if candidates.get(b) != par { continue; }
```

`CandidateSet` deriva `PartialEq`, então a comparação olha o bitmask inteiro de uma só vez: dois `u16` iguais
são dois conjuntos iguais.

## Por que isso importa

A técnica entrou na lista do solver sem mudar nenhuma linha de `solve`, o que confirma o pattern Strategy. E,
graças ao `up_to`, os puzzles Fáceis de referência continuam idênticos.
