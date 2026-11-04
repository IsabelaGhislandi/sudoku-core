# 38 — Dica determinística: SplitMix64, `wrapping_*` e golden test

> Referente à Task 5 do plano da fatia 1 do Game. Commit: `feat: add deterministic hints`.

Uma dica revela o número certo de uma célula vazia sorteada pelo jogo. São 2 por partida. A spec
pede que o sorteio seja **determinístico**: a mesma partida, com as mesmas ações, recebe as mesmas
dicas nas mesmas células. Assim o servidor do PvP pode refazer a partida e conferir o que o celular
diz, e um bug de dica pode ser reproduzido só com a seed.

## A regra

```rust
pub(crate) fn pick(seed: u64, hint_number: u8, empties: usize) -> usize {
    (splitmix64(seed ^ u64::from(hint_number)) % empties as u64) as usize
}
```

Para a dica número `hint_number` (0 na primeira), o jogo mistura a seed do puzzle com esse número,
tira o resto pela quantidade de células vazias e pega a célula nessa posição, contando as vazias
em ordem de índice. Não há estado de RNG guardado na partida: tudo sai da seed e de
`hints_used`, que a partida já tem. Isso vai facilitar o salvamento na fatia 2, porque não há nada
extra para salvar.

`u64::from(hint_number)` converte o `u8` para `u64` sem perda. O Rust não converte números
sozinho, e o `From` deixa claro que a conversão nunca falha. Já `as` serve para conversões que podem
cortar bits, como `usize` → `u64`, e por isso pede mais atenção de quem lê.

## Por que não o `rand`?

O core usa `rand` + `rand_chacha` para gerar puzzles. Para uma dica basta uma função de mistura,
e o **SplitMix64** (Steele, Lea e Flood, 2014) tem cinco linhas e é público:

```rust
pub(crate) fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}
```

Escrever cinco linhas evita uma dependência, e com ela todo o processo da skill
`adding-dependencies` (verificar o crate, licença, `cargo deny`). Também protege o contrato: as
dicas não podem mudar porque uma biblioteca de terceiros trocou um algoritmo numa versão nova.
Foi o mesmo motivo que levou o core a usar `ChaCha12Rng` em vez de `StdRng` (wiki 14).

O `_` dentro das constantes (`0x9E37_79B9_...`) é só um separador visual, como o `1_000_000` de
outras linguagens. O compilador ignora.

## Overflow: `wrapping_*`

Em Rust, `a + b` que passa do limite do tipo **entra em pânico em modo debug** (e dá a volta em
release, sem aviso). O SplitMix64 depende justamente de dar a volta: ele multiplica números de 64
bits e quer só os 64 bits de baixo. `wrapping_add` e `wrapping_mul` dizem isso de forma explícita.
O resultado é o mesmo em debug e em release, e quem lê sabe que a volta é proposital.

| Método | Ao passar do limite |
|---|---|
| `+`, `*` | pânico em debug, volta em release |
| `wrapping_add` | sempre volta (módulo 2⁶⁴) |
| `checked_add` | devolve `None` |
| `saturating_add` | para no máximo |

`Game::new` (wiki 35) usa o mesmo `wrapping_add` para passar da seed `u64::MAX` para 0.

## Testes que travam o algoritmo

Dois testes impedem que a regra mude sem ninguém perceber:

- **Vetor conhecido:** `splitmix64(0) == 0xE220A8397B1DCDAF`, o primeiro valor da referência
  pública do algoritmo. Se uma constante ou um deslocamento sair errado, o teste falha, mesmo que as
  dicas "pareçam aleatórias".
- **Golden da partida:** a primeira dica de `Game::new(Facil, 42, default)` é a célula 40, com
  valor 9. Esse valor não foi calculado à mão. O teste foi escrito com um valor provisório
  (`index: 0, value: 0`), falhou mostrando o valor real, e o valor real foi gravado. É a mesma
  técnica dos golden tests da seed 42 do gerador (wiki 14): o teste não diz que 40 é "certo", diz
  que **não pode mudar**. Se mudar, ou é bug ou é decisão consciente, registrada no mesmo commit.

## Uma `Vec` pequena

```rust
let empties: Vec<usize> = (0..81).filter(|&i| self.cells[i] == CellState::Empty).collect();
let index = empties[hint::pick(self.puzzle.seed, self.hints_used, empties.len())];
```

A wiki 26 recomendava evitar `Vec` no caminho quente do gerador. Aqui não é caminho quente: uma
dica acontece no máximo duas vezes por partida, por um toque da pessoa. Uma `Vec` de até 81
índices deixa o código direto. A divisão por `empties.len()` nunca é por zero: uma partida em
andamento sempre tem célula vazia. Ela começa com pelo menos uma (`from_puzzle` recusa enunciado
sem célula vazia, wiki 36), e a última célula preenchida termina a partida em vitória.
