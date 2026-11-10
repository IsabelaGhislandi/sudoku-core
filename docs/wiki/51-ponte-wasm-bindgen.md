# 51 — A ponte para o navegador: `wasm-bindgen`, `cdylib` e texto como contrato

> Referente à Task 51 do plano do frontend. Commit: `feat: add the sudoku-wasm bridge to start and play a game`.

Até aqui, o jogo só existia nos testes. Para ele rodar no navegador, o Rust precisa virar
WebAssembly (WASM), e o JavaScript precisa conseguir chamá-lo. Esta task cria o crate
`sudoku-wasm`, a ponte entre os dois. Ele é fino de propósito: nenhuma regra de jogo mora ali.
Começar, jogar, errar e perder vida continuam sendo decisões da `Game`, no `sudoku-game`.

```
sudoku-core  ──►  sudoku-game  ──►  sudoku-wasm  ──►  web/ (Svelte, próxima task)
```

## Um crate, dois produtos: `cdylib` e `rlib`

```toml
# sudoku-wasm/Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]
```

Por padrão, uma biblioteca Rust compila para um `rlib`: um formato que só outro código Rust
consegue usar. O navegador não lê `rlib`. Ele precisa de um módulo WASM com funções exportadas, e
é isso que o `cdylib` ("C dynamic library") produz quando o alvo é `wasm32-unknown-unknown`: um
`.wasm` com uma interface plana, sem os metadados do Rust.

Por que os dois? O `cdylib` é o que vai para o navegador. O `rlib` é o que deixa o `cargo test`
rodar os testes no PC, no target nativo, como nos outros crates. Sem ele, cada teste precisaria de
um navegador ou de um runtime de WASM.

Em termos de JavaScript: é como um pacote que publica tanto um bundle para o browser quanto um
módulo para o Node. O código é um só; o que muda é a forma de entrega.

## O que o `#[wasm_bindgen]` faz

O WASM, sozinho, só sabe trocar números (inteiros e floats) entre o módulo e quem o chama. Não
existe string, objeto ou exceção na fronteira. O `wasm-bindgen` gera a cola que falta, dos dois
lados:

```rust
#[wasm_bindgen]
pub struct WebGame {
    game: Game,
}

#[wasm_bindgen]
impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(difficulty: &str, seed: &str) -> Result<WebGame, String> { ... }

    pub fn view(&self, now: f64) -> String { ... }
}
```

- **No Rust:** funções `extern "C"` que recebem ponteiros e tamanhos, e convertem para `&str`, `u8`,
  `WebGame`...
- **No JavaScript:** uma classe `WebGame` de verdade, com `new WebGame("Facil", "42")` e
  `game.view(performance.now())`. A string do JS é copiada para a memória do WASM na ida, e a
  `String` do Rust é copiada de volta na volta.

O `WebGame` que o JS segura não contém a partida. Ele é um **ponteiro** para uma `Game` que vive na
memória do WASM. Por isso a classe gerada tem um método `free()`: o coletor de lixo do JS não sabe
que precisa liberar aquela memória do lado do Rust. (O `wasm-bindgen` também registra um
`FinalizationRegistry` para liberar sozinho, mas sem garantia de quando.)

## `Result<T, String>` vira exceção

```rust
pub fn place(&mut self, index: usize, value: u8, now: f64) -> Result<String, String> {
    let outcome = self
        .game
        .place(index, value, convert::to_ms(now))
        .map_err(convert::error_name)?;
    ...
}
```

O JavaScript não tem `Result`. O `wasm-bindgen` traduz `Ok(v)` para o retorno normal e `Err(e)`
para um `throw e`. No JS, fica assim:

```ts
try {
  game.place(10, 7, performance.now()); // "correct" ou "wrong"
} catch (e) {
  // e === "CellNotEmpty", "Paused", "GameOver"...
}
```

O erro viaja como texto, com o nome da variante do `MoveError`. O `error_name` usa o `Debug`
derivado, que para variantes sem dados é exatamente o nome:

```rust
pub fn error_name(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}
```

Uma jogada recusada não é uma falha inesperada: a tela pode simplesmente ignorá-la. Por isso ela
continua sendo `Result` no Rust, e não `panic`. Um `panic` no WASM vira um erro "unreachable" no
JS, sem mensagem útil, e deixa a instância num estado em que não dá mais para confiar.

## Texto como contrato: o limite de 2^53

O seed entra como `&str`, e não como `u64`:

```rust
/// Lê uma seed em decimal. Ela vem como texto porque um `number` do JS só guarda inteiros
/// exatos até 2^53, e a seed é um `u64` qualquer.
pub fn parse_seed(text: &str) -> Result<u64, String> {
    text.parse().map_err(|_| "InvalidSeed".to_string())
}
```

O `number` do JavaScript é um float de 64 bits. Ele só representa inteiros exatos até
`Number.MAX_SAFE_INTEGER` (2^53 − 1). Acima disso, os inteiros "pulam": `2 ** 53 + 1 === 2 ** 53`
dá `true`. Um seed `u64` pode ir até 2^64 − 1. Se ele cruzasse a ponte como `number`, dois seeds
diferentes poderiam virar o mesmo puzzle.

Existiam duas saídas:

- **`BigInt`:** o `wasm-bindgen` converte `u64` para `BigInt` sozinho. Funciona, mas espalha
  `BigInt` pelo app (`42n`), e o `BigInt` não passa pelo `JSON.stringify`.
- **Texto em decimal:** o mesmo formato que a foto (`SavedGame.seed`) e o resultado
  (`SavedResult.seed`) já usam. Foi a escolha. O teste trava o caso extremo:

```rust
#[test]
fn seed_maxima_atravessa_a_ponte() {
    assert_eq!(parse_seed("18446744073709551615"), Ok(u64::MAX));
    ...
}
```

O retrato da partida segue a mesma ideia: em vez de dezenas de getters cruzando a ponte, um
`GameView` serializado com `serde_json` numa chamada só. No JS, é um `JSON.parse(game.view(now))`.
O `#[serde(rename_all = "camelCase")]` faz os nomes saírem no padrão do JavaScript (`livesLeft`,
`elapsedMs`) sem mudar os nomes em Rust (`lives_left`, `elapsed_ms`).

## O `as` de float para inteiro satura

O tempo vem do `performance.now()`, que é um `number` (`f64`). O `sudoku-game` trabalha com `u64`.
A conversão é uma linha:

```rust
pub fn to_ms(now: f64) -> u64 {
    now as u64
}
```

Desde o Rust 1.45, o `as` de float para inteiro **satura**: um negativo vira `0`, um valor acima do
máximo vira `u64::MAX`, e `NaN` vira `0`. Antes disso, era comportamento indefinido. O teste deixa
explícito o que a ponte promete:

```rust
assert_eq!(to_ms(-5.0), 0);
assert_eq!(to_ms(f64::NAN), 0);
assert_eq!(to_ms(1234.9), 1234); // trunca, não arredonda
```

No JS, o equivalente mais próximo seria `Math.max(0, Math.trunc(x)) || 0`, escrito à mão. Isso
importa porque o `performance.now()` recomeça do zero a cada recarga da página: o `now` pode
"voltar" em relação ao que a partida já viu. O `sudoku-game` já trata o relógio voltando com
`saturating_sub` (wiki 40), e a ponte não pode introduzir um underflow antes de chegar lá.

## Testar `#[wasm_bindgen]` sem navegador

Os testes do `sudoku-wasm` rodam com o `cargo test` comum, no PC:

```rust
#[test]
fn jogada_certa_e_errada() {
    let mut game = WebGame::new("Facil", "42").expect("nível e seed válidos");
    game.resume(0.0);
    ...
    assert_eq!(game.place(first, right, 10.0), Ok("correct".to_string()));
}
```

Isso funciona porque nenhum método cria um `JsValue` no corpo. Fora do alvo WASM, o
`#[wasm_bindgen]` deixa as funções como Rust normal, e só a cola de exportação fica de fora. Se
algum método precisasse de um `JsValue` (um objeto JS de verdade), o teste nativo quebraria em
tempo de execução, e ele precisaria rodar num runtime de WASM. Manter a ponte em texto e números
simples também mantém os testes baratos.
