# 46 — Abandonar: `self` que consome, `bool::then` e uma assinatura que cresce

> Referente à Task 2 do plano da fatia 3 do Game. Commit: `feat: return the abandonment result from abandon and restart`.

Pela spec, reiniciar ou largar uma partida no meio **conta como derrota**. Esta task dá as duas
portas para isso: `abandon`, que larga a partida, e `restart`, que agora devolve o resultado do
abandono junto com a partida nova.

## Três jeitos de receber a partida

```rust
pub fn result(&self, finished_at_ms: u64) -> Option<GameResult>       // lê
pub fn place(&mut self, index: usize, value: u8, now: u64) -> ...    // altera
pub fn abandon(self, now: u64, finished_at_ms: u64) -> Option<GameResult> // consome
```

- `&self` **empresta para ler**: a partida continua igual e utilizável.
- `&mut self` **empresta para alterar**: a partida muda, mas continua de quem chamou.
- `self` **recebe a posse**: a partida passa a ser do método, que a descarta no fim.

Depois de `game.abandon(...)`, a variável `game` não existe mais para quem chamou:

```rust
let r = game.abandon(700, today);
game.place(0, 5, 800); // erro de compilação: borrow of moved value: `game`
```

É uma regra de negócio ("uma partida abandonada não aceita jogadas") garantida pelo **compilador**,
sem um status `Abandoned` e sem `if` em cada ação. Em JavaScript, o objeto continuaria vivo, e só
uma checagem em tempo de execução impediria o uso.

Por que `restart` continua com `&self`? Porque "jogar de novo" também vale numa partida terminada,
e a casca pode querer mostrar a tela de fim enquanto a nova carrega. A posse fica com quem chama, e
é ele quem troca uma partida pela outra.

## `bool::then`: um `Option` a partir de uma condição

```rust
fn abandoned_result(&self, now: u64, finished_at_ms: u64) -> Option<GameResult> {
    (self.status == Status::Playing).then(|| {
        self.result_with(Outcome::Abandoned, self.elapsed_ms(now), finished_at_ms)
    })
}
```

`condição.then(|| valor)` devolve `Some(valor)` se a condição for verdadeira e `None` se não. É o
`cond ? valor : undefined` do JavaScript, mas o resultado é um `Option`, que quem chama precisa
tratar. A closure (`|| ...`) só roda se a condição for verdadeira, então o resultado nem é montado
quando a partida já terminou. Existe também `then_some(valor)`, que recebe o valor pronto e por
isso o calcula sempre.

## Um helper, duas portas

`abandon` e `restart` chamam o mesmo `abandoned_result`. A regra "abandono só de partida não
terminada" mora num lugar só, e as duas portas não têm como divergir. A partida pausada ou nunca
começada também conta: o tempo vem de `elapsed_ms(now)`, que para uma partida pausada é o
acumulado (zero, se nunca começou).

## A assinatura que cresceu

```rust
// fatia 2
pub fn restart(&self, seed: u64) -> Restart
// fatia 3
pub fn restart(&self, seed: u64, now: u64, finished_at_ms: u64) -> Restart
```

O `#[non_exhaustive]` da página 44 protegeu o **valor devolvido**: o campo novo `abandoned` não
quebrou quem desestrutura com `..`. Já os **parâmetros** novos quebram quem chama, e isso não tem
atributo que evite. Foi aceitável porque o crate não é publicado e só os testes o chamam; num crate
público, seria versão nova (semver *major*), ou um método novo ao lado do antigo.
