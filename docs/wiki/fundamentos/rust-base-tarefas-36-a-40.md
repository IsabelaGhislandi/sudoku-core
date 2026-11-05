# Fundamentos de Rust — consolidado das Tarefas 36 a 40

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 36 a
> 40. As páginas por-task ([36](../36-jogadas-e-result.md) … [40](../40-relogio-injetado-e-saturating.md)) contam a
> história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 31 a 35](rust-base-tarefas-31-a-35.md).

Este bloco constrói a partida: jogadas, anotações, dicas, a API vista de fora e o relógio. O fio que liga as cinco
tasks é **regra de jogo como valor**: o erro de uma jogada é um `Result`, o estado é guardado em tipos que não
aceitam combinações sem sentido, e o tempo e o acaso entram de fora, como dados, para que tudo seja
determinístico e testável.

Sumário:
1. [`Result` para regra de jogo, `panic!` para erro de programação](#1-result-para-regra-de-jogo-panic-para-erro-de-programação)
2. [Enums com dados e o operador `?`](#2-enums-com-dados-e-o-operador-)
3. [`&self`, `&mut self` e o estado que só anda para frente](#3-self-mut-self-e-o-estado-que-só-anda-para-frente)
4. [Newtype e bits: `struct Notes(u16)`](#4-newtype-e-bits-struct-notesu16)
5. [Visibilidade em degraus: `pub`, `pub(crate)` e privado](#5-visibilidade-em-degraus-pub-pubcrate-e-privado)
6. [Aritmética explícita: `wrapping_*`, `saturating_*`, `checked_*`](#6-aritmética-explícita-wrapping_-saturating_-checked_)
7. [Determinismo: acaso e tempo entram de fora](#7-determinismo-acaso-e-tempo-entram-de-fora)
8. [Testes de integração e a API completa](#8-testes-de-integração-e-a-api-completa)
9. [Tabela-resumo dos conceitos](#9-tabela-resumo-dos-conceitos)

---

## 1. `Result` para regra de jogo, `panic!` para erro de programação

Rust separa dois tipos de falha:

- **Falha esperada**, que o chamador deve tratar: devolver `Result<T, E>`. Jogar numa célula ocupada, um índice
  81 vindo da tela, pedir dica sem dicas. Nenhum desses é bug, é a pessoa jogando (página 36).
- **Erro de programação**, que nunca deveria acontecer: `panic!`, `unreachable!`, indexação fora do array.
  `game.cell(81)` entra em pânico, como `Grid::get`, porque ler uma célula inexistente é bug de quem chamou.

Em JavaScript, os dois casos costumam virar `throw`. Em Rust, o tipo da função diz qual é qual: quem vê
`-> Result<PlaceOutcome, MoveError>` sabe que precisa tratar o erro, e o compilador avisa se o resultado for
ignorado (`#[must_use]` vem embutido no `Result`).

`from_puzzle -> Result<Game, PuzzleError>` aplica a mesma ideia na **porta de entrada**: um `Puzzle` montado à
mão é conferido uma vez, e daí em diante a `Game` pode confiar nele.

## 2. Enums com dados e o operador `?`

Um enum do Rust é uma **união com etiqueta**: cada variante pode carregar dados diferentes.

```rust
pub enum PlaceOutcome {
    Correct,
    Wrong { index: usize, value: u8 },
}
```

A resposta de `place` tem duas camadas: `Err(MoveError)` (a jogada nem foi aceita, nada mudou) e
`Ok(PlaceOutcome)` (foi aceita, certa ou errada). Errar é uma jogada válida que custa vida, não um erro da API.

O operador `?` encurta a propagação: `self.check_empty_cell(index, value)?;` devolve o `Err` na hora se houver,
ou segue com o valor do `Ok`. É como um `if (err) return err;` que o compilador escreve. Os helpers de validação
(`check_running`, `check_empty_cell`) existem para a **ordem das recusas** ficar num lugar só.

## 3. `&self`, `&mut self` e o estado que só anda para frente

- `fn cell(&self, ...)` só lê: pode ser chamada por quantos quiserem ao mesmo tempo.
- `fn place(&mut self, ...)` muda a partida: exige acesso exclusivo, e o borrow checker garante que ninguém mais
  está lendo durante a mudança.

Junto com os campos privados, isso transforma as regras em garantias: `lives_left` só diminui porque o único
código que mexe nele é o de `place`, e não existe método que o aumente. "Uma vida perdida não volta" não depende
de disciplina, depende de não existir o caminho (página 36).

## 4. Newtype e bits: `struct Notes(u16)`

Um **newtype** é uma struct de um campo só que dá um nome e regras próprias a um tipo comum (página 37). Por
dentro, `Notes` é um `u16` em que o bit n diz se o dígito n está anotado. Por fora, quem usa vê `contains`,
`values` e `is_empty`, e não consegue confundir anotações com candidatos (`CandidateSet`), apesar de os dois
serem a mesma máscara.

Operações de bit usadas:

| Expressão | O que faz |
|---|---|
| `1 << v` | máscara só com o bit `v` |
| `mask & (1 << v) != 0` | o bit `v` está ligado? |
| `mask ^= 1 << v` | inverte o bit `v` (XOR): liga se estava desligado, desliga se estava ligado |

O newtype não custa nada em tempo de execução: depois de compilado, é só um `u16`.

## 5. Visibilidade em degraus: `pub`, `pub(crate)` e privado

| Nível | Quem enxerga | Exemplo |
|---|---|---|
| privado (padrão) | o próprio módulo | campos da `Game` |
| `pub(crate)` | o crate inteiro, não quem usa o crate | `Notes::toggle`, `hint::pick` |
| `pub` + exportado no `lib.rs` | todo mundo | `Game`, `Notes::contains` |

`Notes::toggle` é `pub(crate)` porque a regra "anotação só em célula vazia" mora na `Game` (página 37). Se a tela
pudesse chamar `toggle` direto, a regra seria contornável.

## 6. Aritmética explícita: `wrapping_*`, `saturating_*`, `checked_*`

O Rust não deixa overflow passar em silêncio: em `debug`, `u64::MAX + 1` entra em pânico. Quando o overflow é
esperado ou precisa de outra regra, o método diz qual:

- `wrapping_add` / `wrapping_mul`: dá a volta, como no C. É o comportamento certo para hashes e RNG, como o
  SplitMix64 da dica (página 38) e a seed seguinte (`seed.wrapping_add(1)`) do `Game::new`.
- `saturating_sub`: para no limite. É o certo para tempo, que não pode ficar negativo (página 40).
- `checked_sub`: devolve `Option`, para quem precisa saber que estourou.

A escolha documenta a intenção: quem lê `wrapping_mul` sabe que o estouro é proposital.

## 7. Determinismo: acaso e tempo entram de fora

O `sudoku-game` não sorteia nada sozinho e não lê relógio:

- **Acaso**: a dica sorteia a célula com `splitmix64(seed ^ dicas_usadas)` (página 38). Mesma partida, mesmas
  dicas, no celular e no servidor. Um **golden test** (`dica_da_seed_42_fica_travada`) trava o resultado, como os
  da seed 42 do gerador.
- **Tempo**: a casca passa `now` em milissegundos (página 40). O núcleo só soma, e os testes simulam minutos em
  zero milissegundos.

Os dois são a mesma técnica: tirar a dependência do mundo de dentro da função e recebê-la como parâmetro. É o
que permite ao servidor do PvP refazer uma partida e conferir o que o celular diz.

## 8. Testes de integração e a API completa

Cada arquivo em `tests/` é um crate separado que só vê o que é `pub` (página 39). O teste de integração do
`sudoku-game` foi escrito **sem** `use sudoku_core`, porque o app vai depender só do Game, e isso revelou que
`Cell` e `Grid` precisavam ser re-exportados: **todo tipo que aparece numa assinatura pública precisa estar
acessível para quem usa a API**. Testes inline nunca pegariam isso, porque de dentro do crate tudo é visível.

Na mesma task, o `cargo deny` acusou a dependência por caminho sem versão como curinga, e a opção
`allow-wildcard-paths` liberou só esse caso. Lição: suposição sobre ferramenta só cai rodando a ferramenta.

## 9. Tabela-resumo dos conceitos

| Conceito | Onde apareceu | Em uma frase |
|---|---|---|
| `Result` vs. `panic!` | 36 | Falha esperada é valor de retorno; bug é pânico. |
| Enum com dados | 36 | Cada variante carrega os seus campos, como `Wrong { index, value }`. |
| Operador `?` | 36 | Devolve o `Err` na hora, ou segue com o valor do `Ok`. |
| `&mut self` | 36 | Mudança exige acesso exclusivo, garantido pelo compilador. |
| Newtype | 37 | Struct de um campo que dá nome e regras a um tipo comum, sem custo. |
| XOR em bits | 37 | `^=` inverte um bit: liga e desliga com a mesma operação. |
| `pub(crate)` | 37, 38 | Visível no crate, invisível para quem usa o crate. |
| `wrapping_*` | 38 | Overflow proposital, para hashes e RNG. |
| Golden test | 38 | Trava uma saída determinística para mudanças virarem decisão consciente. |
| Testes em `tests/` | 39 | Crate separado que só vê a API pública. |
| Re-export completo | 39 | Todo tipo de uma assinatura pública precisa estar acessível. |
| Relógio injetado | 40 | O tempo entra como parâmetro; o núcleo nunca lê relógio. |
| `Option<u64>` como estado | 40 | O instante só existe quando o relógio está rodando. |
| `saturating_sub` | 40 | Subtração que para em zero, para grandezas que não ficam negativas. |
| Assinatura que muda | 40 | O compilador lista cada chamada a corrigir. |
