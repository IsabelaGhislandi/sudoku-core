# 40 — Relógio injetado: `Option<u64>` como estado, `saturating_sub` e uma assinatura que muda

> Referente à Task 1 do plano da fatia 2 do Game. Commit: `feat: add pause, resume and elapsed time to Game`.

A partida ganhou um relógio: pausar, retomar e saber quanto tempo foi jogado. O detalhe é que o
`sudoku-game` **continua sem ler relógio nenhum**.

## Por que o núcleo não chama `Instant::now()`

No navegador, `std::time::Instant` nem existe do mesmo jeito. No servidor do PvP, o tempo que vale
é o do servidor, não o do celular. E num teste, ler o relógio de verdade obriga a **esperar** e
deixa o resultado variar com a carga da máquina.

Por isso a casca informa o instante, um `u64` de milissegundos de um relógio monotônico (que só
anda para frente, ao contrário do relógio de parede), e o núcleo só faz contas:

```rust
game.resume(1_000);
assert_eq!(game.elapsed_ms(4_000), 3_000);
game.pause(4_000);
assert_eq!(game.elapsed_ms(60_000), 3_000); // 56 s de pausa não contam
```

O teste "esperou" um minuto em zero milissegundos. Isso se chama **injeção de dependência** na
forma mais simples: em vez de o código buscar o tempo, ele recebe o tempo. No mundo web, é parecido
com passar `Date.now()` como argumento em vez de chamá-lo dentro da função, para poder testá-la.

## `Option<u64>`: "rodando desde quando?"

O estado do relógio são dois campos:

```rust
elapsed_ms: u64,             // jogado até a última pausa
running_since: Option<u64>,  // Some(t): rodando desde t; None: parado
```

Um `bool` `paused` mais um `u64` `started_at` também funcionariam, mas permitiriam um estado sem
sentido: `paused = false` com `started_at` lixo. Com `Option`, **o instante só existe quando o
relógio está rodando**. É a mesma ideia do `NonZeroU8` (wiki 35): escolher um tipo em que o estado
inválido nem pode ser escrito.

`elapsed_ms` vira um `match`:

```rust
match self.running_since {
    Some(since) => self.elapsed_ms + now.saturating_sub(since),
    None => self.elapsed_ms,
}
```

## `saturating_sub`: quando o relógio volta

Se a casca mandar um `now` menor que o do `resume` (um bug na casca, um aparelho reiniciado),
`now - since` dá **underflow**: em `debug` o programa entra em pânico, em `release` o número dá a
volta e vira bilhões de milissegundos. Nenhum dos dois é aceitável num jogo.

A família de operações aritméticas do Rust deixa a escolha explícita:

| Método | `3u64.xxx_sub(5)` | Quando usar |
|---|---|---|
| `-` | pânico (debug) / dá a volta (release) | quando o underflow é impossível |
| `wrapping_sub` | `u64::MAX - 1` | hashes e RNG (wiki 38) |
| `saturating_sub` | `0` | grandezas que não podem ser negativas, como tempo |
| `checked_sub` | `None` | quando quem chama precisa saber que falhou |

Aqui o trecho conta zero, e o teste `relogio_que_volta_nao_quebra_o_tempo` trava isso.

## Uma partida nova começa pausada

"Começar" é o primeiro `resume`. Assim, uma partida nova e uma restaurada do disco (que a próxima
task vai trazer) ficam no mesmo estado, e a casca trata as duas do mesmo jeito. Com o relógio
parado, `place`, `toggle_note` e `hint` devolvem `MoveError::Paused`, e a checagem entrou num helper
único, `check_running`, usado pelos três.

`pause` e `resume` não devolvem erro quando não há o que fazer. A casca pode chamar `pause` em todo
evento de "app foi para o fundo" sem conferir o estado antes.

## O relógio para sozinho no fim

Se a vitória dependesse de a casca lembrar de chamar `pause`, um esquecimento faria o tempo total
crescer para sempre. Por isso `place` e `hint` passaram a receber `now`, e o fim da partida mora num
lugar só:

```rust
fn end(&mut self, status: Status, now: u64) {
    self.pause(now);
    self.status = status;
}
```

## Mudar uma assinatura pública com o compilador de guia

`place(index, value)` virou `place(index, value, now)`. Em JavaScript, uma chamada antiga com dois
argumentos continuaria rodando, com `now` `undefined`. Em Rust, a primeira compilação listou **cada
chamada** que precisava mudar (69 erros), e só depois de corrigir todas o código voltou a compilar.
Mudar uma API dá medo em linguagens dinâmicas. Em Rust, o compilador faz a busca.
