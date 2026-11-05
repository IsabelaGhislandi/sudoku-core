# 43 — Features do Cargo, `serde` opcional e golden test de formato

> Referente à Task 4 do plano da fatia 2 do Game. Commit: `feat: add optional serde feature for SavedGame`.

A foto da partida (wiki 41) precisava virar texto para o app gravar no disco. A escolha, feita em
conversa, foi o **`serde`**, a biblioteca padrão do Rust para serialização, mas **opcional**: o
`sudoku-game` continua sem dependência externa para quem só quer as regras.

## O que o `serde` faz, e por que não deixa o jogo lento

O `serde` separa duas coisas: **o que** se serializa (as traits `Serialize` e `Deserialize`) e **em
qual formato** (`serde_json`, TOML, binário...). O `sudoku-game` só diz "esta struct é
serializável", e a casca escolhe o formato.

```rust
#[derive(Serialize, Deserialize)]
pub struct SavedGame { /* ... */ }
```

O `derive` é uma **proc-macro**: um programa Rust que roda **durante a compilação**, lê a struct e
gera o código de serialização, campo a campo. Em tempo de execução não há reflexão nem inspeção de
tipos, como acontece no `JSON.stringify` do JavaScript. O código final é parecido com o que alguém
escreveria à mão. O custo está na compilação (o `syn` precisa entender o código Rust), não no jogo.

Isso também explica por que a skill `adding-dependencies` olha `build.rs` e proc-macros: os dois
**executam código na máquina de quem compila**.

## Features do Cargo

Uma *feature* é um interruptor de compilação. No `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0.229", default-features = false, features = ["derive", "alloc"], optional = true }

[features]
serde = ["dep:serde"]
```

- `optional = true`: o `serde` só é baixado e compilado se alguém ligar a feature.
- `serde = ["dep:serde"]`: a feature `serde` liga a dependência `serde`. O prefixo `dep:` diz que se
  trata da dependência, não de outra feature de mesmo nome.
- `default-features = false` + `["derive", "alloc"]`: só o necessário. O `alloc` cobre `String` e
  `Vec`; o `std` completo não faz falta.

Quem usa liga a feature na própria dependência. O app Tauri vai escrever
`sudoku-game = { path = "...", features = ["serde"] }`.

## `cfg_attr`: um atributo que só existe com a feature

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SavedGame { /* ... */ }
```

`cfg_attr(condição, atributo)` aplica o atributo só se a condição valer. Sem a feature, a linha
some antes da compilação, e o crate nem menciona o `serde`. É parecido com o `#ifdef` do C, mas
restrito a condições que o Cargo conhece.

O mesmo vale para um arquivo de teste inteiro: `tests/save_json.rs` começa com
`#![cfg(feature = "serde")]` (com `!`, o atributo vale para o arquivo todo). Sem a feature, ele
compila vazio e roda zero testes.

## `dev-dependencies`

O `serde_json` está em `[dev-dependencies]`: entra só em testes, exemplos e benchmarks. Quem
depende do `sudoku-game` nunca o recebe, e o crate continua sem escolher formato.

## Testar as duas configurações

Uma feature dobra o número de jeitos de compilar o crate. Por isso o CI agora roda:

```bash
cargo test   --workspace                  # sem a feature: o crate compila sem o serde?
cargo test   --workspace --all-features   # com a feature: o JSON funciona?
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

`--all-features` liga todas as features de todos os crates do workspace.

## Golden test de formato

O teste `formato_da_foto_versao_1_nao_muda` trava o JSON exato de uma partida fixa (Fácil, seed 42,
uma jogada, uma anotação e uma dica). Ele faz duas coisas:

1. **Escrever:** a foto de hoje vira exatamente aquele texto.
2. **Ler:** aquele texto, que é o que um app antigo deixou no disco, ainda abre e restaura.

Se alguém renomear um campo do `SavedGame`, o JSON muda e partidas salvas param de abrir. O teste
quebra e obriga a decisão: ou é bug, ou é uma versão 2 do formato, com leitura da versão 1. É a
mesma ideia do golden test da seed 42 do gerador (wiki 14).

## Uma licença nova: `Unicode-3.0`

O `serde_derive` usa o `syn`, que usa o `unicode-ident` para saber quais caracteres valem num nome
Rust. O `unicode-ident` traz tabelas de dados do Unicode, sob a licença `Unicode-3.0`, e o `cargo
deny` recusou, porque o `deny.toml` só permitia MIT e Apache-2.0. É uma licença permissiva, aprovada
pela OSI, e entrou na lista depois de aprovação explícita, com o motivo num comentário. É o
`deny.toml` funcionando como deve: licença nova só entra por decisão consciente.
