# Wiki — Aprendizados de Rust

Anotações de aprendizado conforme o Sudoku é implementado, vindo de um background web.
Uma página por task da fatia em andamento. Para a **arquitetura de design**, veja
[`../architecture.md`](../architecture.md).

## Fundamentos consolidados

Documentos **intensivos por conceito** (desde a base do Rust), gerados **a cada 5 tarefas**:

- [Fundamentos de Rust — Tarefas 1 a 5](fundamentos/rust-base-tarefas-01-a-05.md)
- [Fundamentos de Rust — Tarefas 6 a 10](fundamentos/rust-base-tarefas-06-a-10.md)
- [Fundamentos de Rust — Tarefas 11 a 15](fundamentos/rust-base-tarefas-11-a-15.md)
- [Fundamentos de Rust — Tarefas 16 a 20](fundamentos/rust-base-tarefas-16-a-20.md)
- [Fundamentos de Rust — Tarefas 21 a 25](fundamentos/rust-base-tarefas-21-a-25.md)
- [Fundamentos de Rust — Tarefas 26 a 30](fundamentos/rust-base-tarefas-26-a-30.md)

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
- [16 — `CandidateGrid`: quando um valor calculado vira estado](16-candidategrid-e-estado-mutavel.md)
- [17 — Enums com dados e o solver limitado por nível](17-enums-com-dados-e-api-por-nivel.md)
- [18 — Naked Pair e a iteração por unidades](18-naked-pair-e-iteracao-por-unidades.md)
- [19 — Hidden Pair e máscaras de posição](19-hidden-pair-e-mascaras.md)
- [20 — Pointing Pair e funções como parâmetro](20-pointing-pair.md)
- [21 — Gerador de Médio: tentativas, medição e uma checagem redundante](21-gerador-medio-e-tentativas.md)
- [22 — Box/Line Reduction: fatias, `copied` e o primeiro nível Difícil](22-box-line-reduction.md)
- [23 — Naked Triple: combinações sem alocar e métodos que consomem `self`](23-naked-triple-e-combinacoes.md)
- [24 — Hidden Triple: `fold`, `any` e o teste que precisava falhar](24-hidden-triple.md)
- [25 — X-Wing: closures que viram `fn` e uma função para duas direções](25-x-wing.md)
- [26 — Gerador de Difícil: medir, achar o gargalo e otimizar sem mudar a saída](26-gerador-dificil.md)
- [27 — Nomes em inglês: uma regra de idioma e um renomeio seguro](27-nomes-em-ingles.md)
- [28 — Segurança de dependências: `cargo-deny`, features e uma skill para a IA](28-seguranca-de-dependencias.md)
- [29 — Licença MIT: o que ela permite e como o Cargo a declara](29-licenca-mit.md)
- [30 — Swordfish: generalizar o X-Wing e extrair um helper comum](30-swordfish.md)
- [31 — XY-Wing: `let ... else`, pré-filtrar e uma função `sees`](31-xy-wing.md)
- [32 — Gerador de Muito Difícil: refinar em vez de recomeçar, e um `match` exaustivo](32-gerador-muito-dificil.md)
- [33 — Desempenho do Muito Difícil: medir, cachear com invariante e constantes como contrato](33-desempenho-do-muito-dificil.md)
- [34 — Cargo workspace: vários crates num repositório só](34-cargo-workspace.md)

> Cada nova task ganha uma página aqui.
