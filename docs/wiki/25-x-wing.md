# 25 — X-Wing: closures que viram `fn` e uma função para duas direções

> Referente à Task 4 do plano da fatia Difícil. Commit: `feat: add X-Wing technique`.

Em duas linhas, um dígito só cabe nas **mesmas 2 colunas**. Cada linha precisa do dígito uma vez, então ele
ocupa dois cantos opostos do retângulo. Qualquer que seja a diagonal, cada uma das 2 colunas já tem o dígito numa
dessas linhas. Logo, ele sai dessas colunas em todas as **outras** linhas.

```
        col 1   col 7
linha 0   5?  ...  5?      ← o 5 da linha 0 só cabe aqui
linha 4   5?  ...  5?      ← o 5 da linha 4 também
          │         │
          └ o 5 sai das colunas 1 e 7 nas linhas 1, 2, 3, 5, 6, 7, 8
```

O mesmo vale trocando linha por coluna.

## Uma função, duas direções

A busca é idêntica nas duas direções; muda só o que é "linha" e o que é "posição". Em vez de duplicar o código,
a direção vira um parâmetro:

```rust
let por_linha  = |linha: usize, pos: usize| linha * 9 + pos;
let por_coluna = |coluna: usize, pos: usize| pos * 9 + coluna;

for celula in [por_linha, por_coluna] {
    if let Some(pares) = procura(candidates, value, celula) { ... }
}

fn procura(candidates: &CandidateGrid, value: u8, celula: fn(usize, usize) -> usize) -> ...
```

**Closure sem captura vira `fn`.** Cada closure em Rust tem um tipo único e anônimo. Duas closures diferentes não
cabem no mesmo array, porque um array exige um tipo só. Mas uma closure que **não captura nada** do ambiente pode
ser convertida no tipo ponteiro de função `fn(usize, usize) -> usize`. Como o parâmetro de `procura` pede esse
tipo, o Rust converte as duas, e o array `[por_linha, por_coluna]` passa a ter um tipo só.

Se a closure capturasse uma variável (por exemplo, `|l, p| l * largura + p`), essa conversão não seria possível.
Aí seria preciso um genérico (`impl Fn`) ou `Box<dyn Fn>`, como nas páginas 08 e 09.

No Pointing Pair (página 20), o parâmetro `linha_de: fn(usize) -> usize` recebia funções nomeadas, como `row_of`.
Aqui a ideia é a mesma, com closures.

## Montando um array com `from_fn` e `fold`

```rust
let mascaras: [u16; 9] = std::array::from_fn(|l| {
    (0..9)
        .filter(|&p| candidates.get(celula(l, p)).contains(value))
        .fold(0, |m, p| m | (1 << p))
});
```

`std::array::from_fn` cria um array de tamanho fixo chamando a closure para cada índice, sem `Vec` e sem `mut`. O
`fold` junta as posições num bitmask, como no Hidden Triple (página 24).

## Ordenar antes de devolver

Na versão por colunas, a ordem natural dos laços (linha do padrão, depois posição) não é a ordem crescente de
célula. As outras técnicas devolvem os pares em ordem de célula, e o determinismo do gerador depende de uma ordem
fixa. Por isso `pares.sort_unstable()` antes de devolver.

**`sort_unstable`** pode trocar de lugar elementos iguais entre si. Aqui não há pares repetidos, então o resultado
é o mesmo de `sort`, e ele é um pouco mais rápido por não precisar de memória extra.
