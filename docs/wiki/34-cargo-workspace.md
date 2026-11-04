# 34 — Cargo workspace: vários crates num repositório só

> Referente à Task 1 do plano da fatia 1 do Game. Commit: `chore: turn the repository into a Cargo workspace`.

Até aqui o repositório tinha um crate só, o `sudoku-core`, e todo comando precisava de
`--manifest-path sudoku-core/Cargo.toml`. O módulo Game traz um segundo crate, o `sudoku-game`, e
depois vêm o app Tauri e o servidor do PvP. O `architecture.md` dizia para virar workspace "só com
necessidade real", e ela chegou: vários crates que dependem uns dos outros e precisam andar juntos.

## O que é um workspace

Um **workspace** é um grupo de crates (os **membros**) que o Cargo trata como um projeto só. Vindo
da web, é parecido com um monorepo de `npm`/`pnpm workspaces`: cada pacote tem o seu
`package.json`, mas há um lockfile só na raiz.

```toml
# Cargo.toml da raiz: não tem [package], só [workspace]
[workspace]
resolver = "3"
members = ["sudoku-core"]
```

- **Um `Cargo.lock` só, na raiz.** Todos os membros usam as mesmas versões das dependências. Se o
  `sudoku-game` e o `sudoku-core` usassem `rand`, seria a mesma versão para os dois.
- **Um `target/` só, na raiz.** O `sudoku-core` é compilado uma vez e reaproveitado por quem
  depende dele.
- **Um `deny.toml` só.** O `cargo deny check` olha o grafo de dependências do workspace inteiro.

Foi por isso que o `Cargo.lock` e o `deny.toml` foram movidos com `git mv` para a raiz, e o
`.gitignore` com `/target` também. O `git mv` faz o Git registrar a mudança como **renomeação**, e
o histórico do arquivo continua acessível com `git log --follow Cargo.lock`.

## Campos herdados: `[workspace.package]`

Alguns campos valem para todos os crates. Em vez de repetir em cada `Cargo.toml`, eles ficam no
workspace e cada crate herda:

```toml
# raiz
[workspace.package]
edition = "2024"
rust-version = "1.97"
license = "MIT"
repository = "https://github.com/IsabelaGhislandi/sudoku-core"

# sudoku-core/Cargo.toml
[package]
name = "sudoku-core"
edition.workspace = true
rust-version.workspace = true
```

`edition.workspace = true` é açúcar para `edition = { workspace = true }`: "pegue o valor do
workspace". A herança é **explícita**: um campo só é herdado se o crate pedir. Campos próprios de
cada crate, como `name`, `description` e `keywords`, continuam no crate. Assim, subir o MSRV é uma
mudança num lugar só, e os crates não ficam com valores diferentes sem querer.

## `resolver = "3"`

O **resolver** é o algoritmo que escolhe as versões das dependências. Num crate solto, ele vem da
edition (a 2024 já usa o 3). Num workspace, ele é declarado na raiz, porque vale para todos os
membros. O resolver 3 leva em conta o `rust-version`: entre duas versões compatíveis de um crate,
ele prefere uma que compile com o MSRV do projeto.

## Comandos: `--workspace`, `-p` e `--all`

O Cargo procura o `Cargo.toml` subindo pelas pastas até achar um `[workspace]`. Por isso os
comandos funcionam de **qualquer pasta** do repositório, sem `--manifest-path`:

| Comando | O que faz |
|---|---|
| `cargo test --workspace` | testa todos os membros |
| `cargo test -p sudoku-core rating` | só o crate `sudoku-core` (`-p` = `--package`), filtrando por `rating` |
| `cargo clippy --workspace --all-targets` | lint de todos os membros, incluindo testes |
| `cargo fmt --all` | formata todos os membros (no `fmt`, a opção chama `--all`) |

Sem `--workspace`, um comando rodado na raiz de um workspace **virtual** (sem `[package]` na raiz)
já age sobre todos os membros. O `--workspace` explícito deixa o comando claro e igual em qualquer
pasta.

## O que mudou junto

- **CI:** os comandos ganharam `--workspace`, e o `rust-cache` perdeu o `workspaces: sudoku-core`,
  porque o `target/` agora está na raiz, que é o padrão dele.
- **Dependabot:** o `directory` do cargo virou `/`, onde está o `Cargo.lock`.
- **README, CONTRIBUTING, CLAUDE.md e `architecture.md`:** comandos novos.

A prova de que nada quebrou foi rodar os testes antes e depois: 123 unitários, 4 de API e 3
ignorados nos dois casos, golden tests incluídos, e o `Cargo.lock` renomeado sem nenhuma linha
diferente.
