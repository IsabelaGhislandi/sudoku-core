# Wiki — Aprendizados de Rust

Anotações de aprendizado conforme o Sudoku é implementado, vindo de um background web.
Uma página por task da fatia em andamento. Para a **arquitetura de design**, veja
[`../architecture.md`](../architecture.md).

## Fundamentos consolidados

Documentos **intensivos por conceito** (desde a base do Rust), gerados **a cada 5 tarefas**:

- [Fundamentos de Rust — Tarefas 1 a 5](fundamentos/rust-base-tarefas-01-a-05.md)
- [Fundamentos de Rust — Tarefas 6 a 10](fundamentos/rust-base-tarefas-06-a-10.md)

## Índice por task

- [01 — Scaffold, crate e editions](01-scaffold-crate-e-editions.md)
- [02 — Enums e `derive` (e o truque do `Ord`)](02-enums-e-derive.md)
- [03 — Grid, Cell: arrays fixos e borrow checker](03-grid-cell-arrays-e-borrow.md)
- [04 — Parsing: Result, enum de erro e pattern matching](04-parsing-result-e-pattern-matching.md)
- [05 — Regras do Sudoku: índice→coordenada, `if let` e `Vec`](05-regras-do-sudoku-if-let-e-vec.md)
- [06 — Candidatos: bitmask `u16` e operações de bit](06-candidatos-bitmask-e-operacoes-de-bit.md)
- [07 — Traits, o pattern Strategy e módulos em pasta](07-traits-strategy-e-modulos.md)
- [08 — Closures, `impl Fn` e `Option` como acumulador](08-closures-e-impl-trait.md)
- [09 — Trait objects, `Box<dyn Trait>` e dispatch dinâmico](09-trait-objects-e-dispatch-dinamico.md)
- [10 — Recursão, backtracking e referências `&mut`](10-recursao-backtracking-e-mut-refs.md)
- [11 — Rating: cadeias de iteradores, `max` e `Option`](11-rating-iteradores-e-option.md)
- [12 — Gerador: RNG com seed, traits em escopo e MRV](12-gerador-rng-com-seed-e-traits-em-escopo.md)
- [13 — API pública: visibilidade, `pub use` e testes de integração](13-api-publica-visibilidade-e-testes-de-integracao.md)
- [14 — RNG reproduzível entre versões e golden tests](14-rng-reproduzivel-e-golden-tests.md)
- [15 — CI/CD com GitHub Actions](15-ci-cd-com-github-actions.md)

> Cada nova task ganha uma página aqui.
