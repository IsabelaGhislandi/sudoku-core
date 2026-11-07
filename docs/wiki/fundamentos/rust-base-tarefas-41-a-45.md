# Fundamentos de Rust — consolidado das Tarefas 41 a 45

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 41 a
> 45. As páginas por-task ([41](../41-foto-da-partida-e-dto.md) … [45](../45-resultado-da-partida.md)) contam a
> história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 36 a 40](rust-base-tarefas-36-a-40.md).

Este bloco faz a partida **sair do processo e voltar**: a foto que vai para o disco, a conferência do que volta, a
serialização opcional, o reinício e o primeiro registro do histórico. O fio que liga as cinco tasks é a
**fronteira com o mundo de fora**: o que entra é desconfiado até provar o contrário, o que sai tem forma estável, e
os tipos públicos são desenhados para crescer sem quebrar quem os usa.

Sumário:
1. [DTO: um tipo para transportar, outro para jogar](#1-dto-um-tipo-para-transportar-outro-para-jogar)
2. ["Parse, don't validate"](#2-parse-dont-validate)
3. [Converter texto com `Result` e `?`](#3-converter-texto-com-result-e-)
4. [Features do Cargo e `cfg_attr`](#4-features-do-cargo-e-cfg_attr)
5. [Golden test de formato](#5-golden-test-de-formato)
6. [`#[non_exhaustive]`: crescer sem quebrar](#6-non_exhaustive-crescer-sem-quebrar)
7. [`Option` e enums para estados que não existem sempre](#7-option-e-enums-para-estados-que-não-existem-sempre)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. DTO: um tipo para transportar, outro para jogar

A `Game` guarda o estado em tipos ricos (`[CellState; 81]`, `NonZeroU8`, `Option<u64>`), que não aceitam
combinações sem sentido. O disco e a rede só entendem texto e números. Em vez de ensinar a `Game` a se serializar,
o projeto tem um **DTO** (*data transfer object*), o `SavedGame`, com campos simples e públicos (página 41).

- O DTO **não carrega garantia nenhuma**: pode ter vindo editado à mão.
- A `Game` só nasce de um DTO por `Game::restore`, que confere tudo.
- Os tipos do `sudoku-core` não precisam saber serializar.

Em TypeScript, é a diferença entre o JSON que chega de uma API (`unknown`) e o objeto do domínio, que só existe
depois de passar por um validador como o `zod`.

## 2. "Parse, don't validate"

Validar é olhar o dado e dizer "está ok", mas continuar com o mesmo tipo frouxo. **Parsear** é transformar o dado
frouxo num tipo que **só pode** estar certo (página 42):

```rust
let lives = NonZeroU8::new(saved.lives).ok_or(RestoreError::InvalidLives)?;
```

Depois dessa linha, `lives` é um `NonZeroU8`, e nenhum código adiante precisa lembrar de checar o zero. Outras
ferramentas do mesmo bloco:

- **Índice de fora nunca indexa sem conferir:** `if i >= 81 { return Err(...) }` antes de `cells[i]`.
- **`let ... else`** exige um formato e sai cedo se não for: `let CellState::Placed(v) = cells[i] else { return Err(...) };`.
- **`&&` evita underflow:** `a <= b && x == b - a`: a subtração só roda se a primeira parte for verdadeira.
- **Erro que embrulha erro:** `RestoreError::Puzzle(PuzzleError)` reaproveita as regras de `from_puzzle`.

## 3. Converter texto com `Result` e `?`

`str::parse::<u64>()` devolve `Result<u64, ParseIntError>`. O `map_err` troca o erro da biblioteca pelo erro do
domínio, e o `?` devolve cedo:

```rust
let seed: u64 = saved.seed.parse().map_err(|_| RestoreError::InvalidSeed)?;
```

A seed vai como **texto** porque o JSON lido por JavaScript perde precisão acima de 2^53, e a seed é um `u64`
qualquer. Tempos em milissegundos ficam número: 2^53 ms são 285 mil anos.

## 4. Features do Cargo e `cfg_attr`

Uma **feature** liga código e dependências opcionais na compilação (página 43). O `sudoku-game` tem a feature
`serde`, desligada por padrão:

```toml
[dependencies]
serde = { version = "1", default-features = false, features = ["derive", "alloc"], optional = true }

[features]
serde = ["dep:serde"]
```

E o atributo só existe com ela ligada:

```rust
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
```

Consequência: o projeto precisa testar **as duas configurações** (`cargo test` e `cargo test --all-features`), e o
teste de JSON só compila com a feature (`#![cfg(feature = "serde")]`).

## 5. Golden test de formato

O golden test da seed 42 trava a saída do gerador. O da foto trava o **JSON**: uma partida fixa, salva, tem de dar
exatamente o texto esperado. Mudou a forma? Ou é bug, ou é versão nova do formato, com leitura da antiga. É o que
garante que uma partida salva por uma versão antiga do app continue abrindo depois de atualizar.

## 6. `#[non_exhaustive]`: crescer sem quebrar

Acrescentar um campo a uma struct pública quebra quem a monta (`Restart { game }`) ou desestrutura sem `..`. Com
`#[non_exhaustive]`, quem está **fora do crate** só pode ler campos ou desestruturar com `..` (página 44):

```rust
let Restart { game: new_game, .. } = game.restart(4);
```

O mesmo atributo tem dois usos neste bloco:

- no `Restart`, para a fatia 3 acrescentar o resultado do abandono sem quebrar ninguém;
- no `GameResult` (página 45), para que a casca **leia** o registro mas não consiga **fabricá-lo**: ele só vem da
  `Game` ou de um registro salvo.

Em enums, ele obriga quem faz `match` de fora a ter um braço `_`.

## 7. `Option` e enums para estados que não existem sempre

Dois padrões fecham o bloco (página 45):

- **`Option` para "só existe às vezes":** `Game::result` devolve `None` com a partida em andamento, em vez de um
  resultado "vazio". `return None` dentro de um braço de `match` tem o tipo `!` ("nunca volta"), que combina com
  qualquer tipo.
- **Enum no lugar de vários `bool`:** `Outcome { Won, OutOfLives, Abandoned }` tem exatamente os três casos; dois
  `bool` teriam uma quarta combinação impossível.

E um cuidado que o compilador não pega: **dois `u64` com sentidos diferentes**. O `now` (relógio monotônico, mede
duração) e o `finished_at_ms` (relógio de parede, data em UTC) são o mesmo tipo, então os nomes carregam a
diferença.

## 8. Tabela-resumo dos conceitos

| Conceito | Onde aparece | Equivalente aproximado em JS/TS |
|---|---|---|
| DTO separado do domínio | `SavedGame` × `Game` | JSON `unknown` × objeto validado |
| Parse, don't validate | `NonZeroU8::new(..).ok_or(..)?` | `zod.parse` devolvendo o tipo certo |
| `map_err` + `?` | `seed.parse().map_err(..)?` | `try/catch` que relança outro erro |
| `let ... else` | exigir `CellState::Placed` | `if (!(x instanceof Y)) return` |
| Feature do Cargo | `serde` opcional | dependência opcional + build flag |
| `cfg_attr` | `derive` só com a feature | código atrás de `process.env` no build |
| Golden test de formato | `save_json.rs` | snapshot test do Jest |
| `#[non_exhaustive]` | `Restart`, `GameResult` | sem equivalente direto |
| `Option` | `Game::result` | `T \| undefined`, mas obrigatório tratar |
| Enum de desfecho | `Outcome` | união de strings literais |
