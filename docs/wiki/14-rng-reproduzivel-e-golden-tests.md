# 14 — RNG reproduzível entre versões e golden tests

> Fora da numeração original do plano: correção de um risco identificado ao escrever a
> [página 12](12-gerador-rng-com-seed-e-traits-em-escopo.md).
> Commit: `fix: use ChaCha12Rng so seeds stay reproducible across versions`.

O gerador já era determinístico **dentro de uma versão**: mesma seed, mesmo puzzle. O que
faltava era garantir isso **ao longo do tempo**, porque a seed vai virar um código que o
usuário compartilha ("jogue o mesmo puzzle que eu").

## O problema: determinismo ≠ reprodutibilidade

São duas promessas diferentes, e é fácil confundir:

| Promessa | Significa | Quem garantia antes |
|---|---|---|
| **Determinismo** | rodar duas vezes agora dá o mesmo resultado | ✅ `StdRng` com seed |
| **Reprodutibilidade** | rodar daqui a um ano, em outra versão da lib, dá o mesmo resultado | ❌ ninguém |

O `StdRng` do `rand` é um **apelido para "o melhor algoritmo do momento"**. A documentação da
lib é explícita: o algoritmo por trás dele pode ser trocado numa versão futura, e por isso ele
não deve ser usado quando a saída precisa ser reproduzível. Hoje ele é o ChaCha12; amanhã pode
ser outro.

Consequência prática: um `cargo update` poderia mudar **todos os puzzles**. Quem tivesse
anotado a seed `42` como "aquele puzzle bom" receberia um tabuleiro diferente. Pior: nada
avisaria — os testes `mesma_seed_gera_mesmo_puzzle` e `seeds_diferentes_geram_puzzles_diferentes`
continuariam passando, porque eles comparam **duas execuções da mesma versão**.

## A correção: um algoritmo nomeado

```toml
[dependencies]
rand = "0.10"
rand_chacha = "0.10.0"
```

```rust
use rand_chacha::ChaCha12Rng;

let mut rng = ChaCha12Rng::seed_from_u64(seed);
```

`ChaCha12Rng` **é** o algoritmo, não um apelido. O crate `rand_chacha` é mantido pelo mesmo
Rand Project e a documentação dele promete o oposto da do `StdRng`: *"These generators are all
deterministic and portable, with testing against reference vectors"*. "Reference vectors" são
pares seed → saída publicados junto do algoritmo ChaCha; a lib testa contra eles, então mudar
a saída seria um bug, não uma decisão de versão.

> Curiosidade: a doc do `StdRng` sugere o crate `chacha20` (do RustCrypto), que é o que o
> `rand` 0.10 usa internamente. Qualquer um dos dois serve; escolhi o `rand_chacha` por ser do
> mesmo projeto do `rand`, o que reduz a chance de descasamento de versão do `rand_core`. A troca foi puramente de tipo — `full_solution`, `fill` e `dig_facil` só mudaram a
anotação do parâmetro `&mut ChaCha12Rng`.

O código continua compilando por causa dos **traits**: `shuffle` (de `SliceRandom`) e
`seed_from_u64` (de `SeedableRng`) funcionam para qualquer tipo que implemente `RngCore` /
`SeedableRng`. O gerador nunca dependeu do `StdRng` em si, só do comportamento que os traits
descrevem — é o mesmo desacoplamento do trait `Technique` (Task 7), agora vindo de fora.

> **Detalhe de versão importante:** `rand` 0.10 e `rand_chacha` 0.10 precisam concordar na
> versão do `rand_core` (0.10 aqui). Se uma usasse `rand_core` 0.9, os traits seriam **tipos
> diferentes** para o compilador, e o `shuffle` não aceitaria o RNG — um erro confuso do tipo
> "the trait bound is not satisfied" mesmo com o trait importado.

Como o `StdRng` do `rand` 0.10 **já era** ChaCha12 por baixo, e o `seed_from_u64` usa a mesma
expansão de seed, a saída não mudou: os puzzles de todas as seeds continuam idênticos aos de
antes. A mudança não altera o presente; ela **protege o futuro**.

## Golden test: travar a saída

Trocar o RNG sem uma rede de proteção só empurraria o problema. Por isso entrou um teste que
fixa a saída caractere a caractere:

```rust
#[test]
fn puzzle_da_seed_42_e_reproduzivel() {
    let puzzle = generate(Difficulty::Facil, 42).unwrap();
    assert_eq!(
        puzzle.givens.to_line(),
        "94.35...12....7....6..8.2......3...4.....5...6......3.5..1.9.....26..9...1.....68"
    );
}
```

Isso é um **golden test** (ou *characterization test*): em vez de descrever uma propriedade,
ele congela um **valor esperado** conhecido. A pergunta que ele responde é "a saída mudou?".

| Tipo de teste | Exemplo aqui | Detecta |
|---|---|---|
| Propriedade | `count_solutions(givens, 2) == 1` | quebra de regra do domínio |
| Golden | a string das 81 casas da seed 42 | **qualquer** mudança de saída, inclusive acidental |

Características de um golden test, que valem para todos eles:

- **O valor vem da implementação.** Não dá para derivá-lo no papel: rodei, vi o que saiu e
  registrei. Ele não prova que a saída está *correta* — quem prova isso são os outros testes
  (única, Fácil, sem conflitos). Ele prova que a saída está **estável**.
- **Falhar não significa bug.** Significa "algo mudou; foi de propósito?". Se a mudança for
  intencional, atualiza-se o valor no mesmo commit, o que deixa a mudança **visível no diff** —
  que é justamente o que faltava antes.
- **Use com moderação.** Golden test demais deixa a suíte frágil: qualquer refactor legítimo
  quebra dezenas deles. Aqui só existe um, sobre a única saída que é **contrato público**.

Repare que os outros três testes do gerador continuam necessários, e nenhum deles é
substituído por este. O golden diz *o que* sai; eles dizem *por que* aquilo é um Sudoku válido.

## Por que isso importa

A seed deixou de ser um detalhe de teste e virou **parte da API**: ela é o futuro "código do
puzzle" para compartilhar com outra pessoa e, mais adiante, a base do multiplayer. Mudanças
assim — transformar um detalhe interno em promessa pública — pedem dois cuidados juntos:
escolher uma implementação que **possa** cumprir a promessa, e um teste que **avise** quando
ela for quebrada.
