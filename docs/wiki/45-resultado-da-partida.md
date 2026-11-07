# 45 — Resultado da partida: enum de desfecho, `Option` para "só quando terminou" e dois relógios

> Referente à Task 1 do plano da fatia 3 do Game. Commit: `feat: add game result for finished games`.

A fatia 3 começa pelo registro que vai para o histórico: o `GameResult`. Ele nasce quando a partida
termina (esta task) ou quando é abandonada (task seguinte).

## Um enum para o desfecho

```rust
pub enum Outcome {
    Won,
    OutOfLives,
    Abandoned,
}
```

Daria para usar um `bool` (`won: bool`), mas a spec quer que o histórico diferencie "perdeu as
vidas" de "abandonou". Com dois `bool` (`won`, `abandoned`), existiria a combinação sem sentido
`won: true, abandoned: true`. O enum tem exatamente os três casos possíveis, nenhum a mais. É a
mesma ideia do `CellState` da fatia 1: **o tipo não aceita estados impossíveis**.

Em TypeScript, seria uma união de strings (`'won' | 'outOfLives' | 'abandoned'`). A diferença é que
o `match` do Rust obriga a tratar todos os casos: se um quarto desfecho aparecer, todo `match` sem
ele deixa de compilar.

## `Option` para "só existe quando terminou"

```rust
pub fn result(&self, finished_at_ms: u64) -> Option<GameResult>
```

Uma partida em andamento não tem resultado. Em vez de devolver um resultado "vazio" ou entrar em
pânico, o método devolve `None`. Quem chama é obrigado a olhar:

```rust
if let Some(r) = game.result(today) {
    history.push(r);
}
```

O `match` dentro do método converte um enum no outro e sai cedo com `return None`:

```rust
let outcome = match self.status {
    Status::Playing => return None,
    Status::Won => Outcome::Won,
    Status::Lost => Outcome::OutOfLives,
};
```

`return` dentro de um braço de `match` é uma expressão do tipo `!` ("nunca volta"), que combina com
qualquer tipo. Por isso os três braços "têm o mesmo tipo" e o código compila.

## Repetível de propósito

`result` recebe `&self`, não muda nada e dá o mesmo valor toda vez. A casca registra o resultado
no instante em que a partida termina. Se o app fechar e reabrir, restaurar a partida terminada e
pedir de novo dá o mesmo registro, e a casca sabe que já o guardou. Um método que "entregasse uma
vez só" precisaria de um estado a mais (`delivered: bool`), que também teria de ir para a foto.

## `#[non_exhaustive]` numa struct de leitura

```rust
#[non_exhaustive]
pub struct GameResult { pub difficulty: Difficulty, /* ... */ }
```

Na página 44, o atributo servia para o `Restart` **crescer** sem quebrar ninguém. Aqui ele tem um
segundo efeito: de fora do crate, ninguém **monta** um `GameResult` com `GameResult { ... }`. A
casca só lê os campos de um resultado que veio da `Game` (ou, na Task 4, de um registro salvo).
Dentro do crate, os testes montam à mão sem problema: a restrição vale só para quem está fora.

## Dois relógios

A fatia 2 trouxe o `now`, de um relógio **monotônico**: só anda para frente, serve para medir
duração e não tem relação com o calendário. O resultado precisa de uma **data** ("13/06/2026"), e
essa vem de outro relógio, o **de parede**: milissegundos desde 1/1/1970 em UTC, o mesmo de
`Date.now()` em JavaScript.

| | `now` | `finished_at_ms` |
|---|---|---|
| Relógio | monotônico | de parede (UTC) |
| Serve para | medir tempo jogado | dizer quando foi |
| Pode pular? | não | sim (ajuste de hora) |

Os dois são `u64`, e o compilador não distingue um do outro. Por isso os nomes carregam a
diferença (`now` × `finished_at_ms`), e o tempo do resultado vem só do `now`.
