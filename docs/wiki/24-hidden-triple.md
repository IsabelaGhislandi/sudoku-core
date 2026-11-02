# 24 — Hidden Triple: `fold`, `any` e o teste que precisava falhar

> Referente à Task 3 do plano da fatia Difícil. Commit: `feat: add Hidden Triple technique`.

Três dígitos que, numa unidade, só cabem dentro das **mesmas 3 células**. Essas células ficam com os três
dígitos, em alguma ordem, então todos os **outros** candidatos delas saem. É o espelho do Naked Triple: lá, olhamos
os candidatos de cada célula; aqui, as posições de cada dígito.

## Máscaras de posição, de novo

Como no Hidden Pair (página 19), cada dígito ganha um `u16` com um bit por posição da unidade. A união das
posições de três dígitos é um OU das três máscaras:

```rust
let onde = trio.iter().fold(0, |m, &d| m | posicoes[d as usize]);
if onde.count_ones() != 3 { continue; }
```

- **`fold`** percorre o iterador carregando um acumulador: começa em `0` e, a cada dígito, faz `m | máscara`. É
  o `reduce` do JavaScript com valor inicial: `trio.reduce((m, d) => m | posicoes[d], 0)`.
- **`count_ones()`** conta os bits ligados. Exatamente 3 bits significa que os três dígitos estão confinados em 3
  células.

## O dígito que não está mais lá

```rust
if trio.iter().any(|&d| posicoes[d as usize] == 0) { continue; }
```

Um dígito já colocado na unidade não tem posição nenhuma: a máscara dele é `0`. Sem essa linha, ele "some" do
OU e um **par** pareceria um triple:

```
linha 0:  9 colocado na célula 8
          1 só nas células 0 e 1
          2 só nas células 0, 1 e 2
          {1, 2, 9} → posições {0, 1, 2}: 3 bits!
```

A técnica concluiria que a célula 2 só pode ter 1, 2 ou 9, e removeria dela candidatos verdadeiros. Um puzzle
gerado com esse bug poderia ter a "solução" errada.

**`any`** devolve `true` se o teste vale para pelo menos um elemento, e para de olhar no primeiro que vale. É o
`some` do JavaScript.

## Ver o teste falhar de propósito

O teste `ignora_digito_ja_colocado` foi escrito antes da implementação, como todos. Mas um erro de compilação só
prova que a técnica não existia, não que o teste pega **esse** bug. Por isso a implementação foi escrita primeiro
**sem** a linha do `any`, e o teste falhou:

```
techniques::hidden_triple::tests::ignora_digito_ja_colocado --- FAILED
```

Só depois a regra entrou e o teste passou. Assim fica provado que o teste distingue o código certo do errado, e
não passaria por acaso.
