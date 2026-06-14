# 01 — Scaffold, crate e editions

> Referente à **Task 1** do plano da fatia Fácil. Commit: `chore: scaffold sudoku-core crate`.

## O que é um "crate"

Um **crate** é a unidade de compilação do Rust — o equivalente a um "pacote/projeto".
`cargo new --lib sudoku-core` cria um crate **biblioteca** (sem `main`, feito para ser
usado por outros), com:

- `Cargo.toml` — manifesto: nome, versão, **edition** e dependências (análogo ao `package.json`).
- `src/lib.rs` — a raiz da biblioteca (o "ponto de entrada" dos módulos).
- `Cargo.lock` — versões exatas resolvidas (análogo ao `package-lock.json`).

## `cargo` ≈ `npm`

| Tarefa | Web (npm) | Rust (cargo) |
|--------|-----------|--------------|
| Criar projeto | `npm init` | `cargo new` |
| Instalar deps | `npm install` | resolvido no `cargo build`/`test` a partir do `Cargo.toml` |
| Rodar testes | `npm test` | `cargo test` |
| Manifesto | `package.json` | `Cargo.toml` |
| Lockfile | `package-lock.json` | `Cargo.lock` |

Rodamos sempre da raiz do repo com `--manifest-path sudoku-core/Cargo.toml`, então o
diretório atual não importa.

## Editions

A **edition** é uma "versão da linguagem" (2015, 2018, 2021, 2024). Ela muda regras de
sintaxe/idioma, **não** a versão do compilador. Crates de editions diferentes convivem no
mesmo build. Decidimos sempre usar a **mais recente** (`2024`).

## Convenção: biblioteca não versiona `Cargo.lock`

Para **bibliotecas**, o convencional é **não** commitar o `Cargo.lock` (quem consome é que
fixa as versões); para **binários**, sim. Quando o app Tauri entrar, o lock dele é que vale.
Por isso o `Cargo.lock` ficou fora do git, e `target/` (artefatos de build) está no `.gitignore`.
