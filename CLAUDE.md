# Sudoku — convenções do projeto

Jogo de Sudoku em Rust + Svelte (APK via Tauri). Visão de design: [`docs/architecture.md`](docs/architecture.md).
Desenvolvimento **spec-driven + modular + TDD**: cada módulo segue `spec → plano → implementação`.

## Commits

**Mensagens sempre em inglês**, no padrão **Conventional Commits** — mesmo que specs, planos e
wiki sejam escritos em português. O histórico do git segue a convenção da indústria e fica
pronto para portfólio e colaboração.

```
type: short summary in the imperative

Optional body explaining why, wrapped at ~72 columns.
```

Tipos em uso: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`.

- O assunto descreve **o que o commit faz**, não o que você fez ("add rate", não "added rate").
- Um commit por unidade lógica. O padrão do projeto é um commit `feat`/`fix` com o código e um
  commit `docs` com a página da wiki correspondente.
- As mensagens de commit de exemplo dentro dos planos (`docs/superpowers/plans/`) também
  seguem esta regra, mesmo com o texto ao redor em português. Se escrever um plano novo,
  já escreva os `git commit -m` dele em inglês.

## Testes

TDD de verdade: escreva o teste, **veja-o falhar**, implemente, veja passar.

- Testes unitários ficam **inline** em cada módulo, num bloco `#[cfg(test)] mod tests`.
- Testes de integração da API pública ficam em `sudoku-core/tests/` (só enxergam o que é `pub`).
- Nomes de teste em português, descrevendo o comportamento:
  `fn puzzle_resolvivel_por_singles_eh_facil()`.
- Nada de afirmar que algo passa sem ter rodado o comando e visto a saída.

## Comandos

Rodam da raiz do repositório, com `--manifest-path`, para funcionarem de qualquer diretório:

```bash
cargo test  --manifest-path sudoku-core/Cargo.toml          # tudo
cargo test  --manifest-path sudoku-core/Cargo.toml rating   # um módulo
cargo test  --manifest-path sudoku-core/Cargo.toml --test api
cargo build --manifest-path sudoku-core/Cargo.toml          # deve ficar sem warnings
cargo clippy --manifest-path sudoku-core/Cargo.toml
```

## Código

- Rust **edition 2024**. Preferir sempre as versões mais novas de edition e dependências.
- `sudoku-core` é uma **biblioteca pura**: sem UI, sem IO, sem Tauri, sem relógio. Timer,
  persistência e histórico pertencem ao módulo Game (futuro).
- Dependências mínimas: hoje só `rand` e `rand_chacha`. Cada nova dependência precisa de motivo.
- Comentários e doc comments (`///`) em **português**, explicando o *porquê*.
- Determinismo do gerador é **contrato**: mesma seed + mesmo nível → mesmo puzzle, inclusive
  entre versões da lib. Por isso o RNG é o `ChaCha12Rng`, e um golden test trava a saída da
  seed 42. Mudou a saída? Ou é bug, ou é decisão consciente que vai no mesmo commit.

## Documentação

- **Uma página de wiki por task**, em `docs/wiki/`, em português, focada nos conceitos de Rust
  que apareceram naquela task (o projeto também é um estudo de Rust, vindo de background web).
  Indexe a página nova em `docs/wiki/README.md`.
- A cada 5 tasks, um documento consolidado em `docs/wiki/fundamentos/`.
- `docs/architecture.md` é a visão de design; mantenha-o em dia quando uma decisão mudar.
