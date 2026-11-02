# 19 — Hidden Pair e máscaras de posição

> Referente à Task 4 do plano da fatia Médio. Commit: `feat: add Hidden Pair technique`.

**A técnica:** se, numa unidade, os dígitos 1 e 2 só cabem nas células A e B, então A e B ficam com o 1 e o
2, mesmo que tenham outros candidatos. Esses outros candidatos saem.

## Naked e hidden: duas visões do mesmo fato

| | Naked Pair | Hidden Pair |
|---|---|---|
| Olha para | os **dígitos de cada célula** | as **células de cada dígito** |
| Padrão | 2 células com os mesmos 2 dígitos | 2 dígitos nas mesmas 2 células |
| Remove | esses dígitos das **outras** células | os **outros** dígitos dessas células |

## Máscara de posições

```rust
let mut posicoes = [0u16; 10];
for (pos, &i) in unit.iter().enumerate() {
    for v in candidates.get(i).values() {
        posicoes[v as usize] |= 1 << pos;
    }
}
```

É a mesma ideia do `CandidateSet`, agora invertida: o bit `pos` ligado significa "o dígito cabe nessa posição
da unidade". Assim, "dois dígitos nas mesmas células" vira uma única comparação de inteiros:

```rust
if posicoes[b as usize] != onde { continue; }
```

O array tem 10 posições para que o índice seja o próprio dígito, deixando o índice 0 sem uso. Gastar uma
posição a mais em troca dessa clareza é um bom negócio.

## `as usize`: conversão explícita

Rust **nunca** converte tipos numéricos sozinho, e índice de array precisa ser `usize`. Por isso aparece
`v as usize`, com `v: u8`. Vindo de JS, onde tudo é `number`, parece burocracia, mas isso evita um tipo de bug
real: `as` de um tipo maior para um menor **corta bits em silêncio**. Aqui é seguro, porque de `u8` para
`usize` nunca há perda.

## `enumerate` e padrões na closure

```rust
.enumerate()
.filter(|&(pos, _)| onde & (1 << pos) != 0)
.flat_map(|(_, &i)| /* ... */)
```

`enumerate` transforma cada item em `(posição, item)`, como o `index` em `arr.map((item, index) => ...)`. Os
padrões variam com o que cada closure recebe:

- `filter` recebe uma **referência** ao item, então o padrão é `&(pos, _)`.
- `flat_map` recebe o item em si, cujo segundo campo é `&usize`, então o padrão é `(_, &i)`.

`_` ignora o campo.

## Por que isso importa

Os testes incluem um caso que uma implementação apressada erraria: `nao_confunde_digitos_em_celulas_diferentes`.
Dois dígitos, cada um em duas células, mas não nas **mesmas** duas, não formam um par. Com máscaras, esse erro
fica difícil de cometer.
