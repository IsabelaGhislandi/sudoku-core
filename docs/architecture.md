# Arquitetura — Sudoku

> Visão de **design** do projeto. Para os aprendizados de Rust durante a implementação,
> veja a [wiki](wiki/README.md). Para o detalhe completo de cada decisão, veja a
> [spec do Core/Engine](superpowers/specs/2026-06-14-sudoku-engine-design.md) e o
> [plano da fatia Fácil](superpowers/plans/2026-06-14-engine-facil-vertical-slice.md).

## 1. Visão geral

Jogo de Sudoku para celular (APK Android via **Tauri**), com backend em **Rust** e
frontend em **Svelte**. Desenvolvimento **modular** e **spec-driven**: cada módulo segue
o ciclo `spec → plano → implementação` com **TDD**.

## 2. Decomposição em módulos

| # | Módulo | Responsabilidade | Status |
|---|--------|------------------|--------|
| 1 | **Core/Engine** (`sudoku-core`) | Gerar, resolver, validar e classificar tabuleiros. Lógica pura. | **em implementação** |
| 2 | Game/Sessão | Estado de uma partida: jogadas, anotações, erros, **timer**, dicas, **histórico de partidas**. | futuro |
| 3 | Frontend/UI | Tabuleiro, input, seleção de dificuldade, telas (Svelte). | futuro |
| 4 | Tauri/Empacotamento | Ponte Rust↔Svelte e build do APK. | futuro |
| 5 | Online | Partida por código com outro player. | futuro distante |

A fronteira é dura: **`sudoku-core` é uma biblioteca pura** — sem UI, sem IO, sem Tauri.
Isso a torna testável isoladamente com `cargo test` e reaproveitável por todos os outros módulos.

## 3. Decisões de design (Core/Engine)

| Decisão | Escolha | Motivo |
|---------|---------|--------|
| Tamanho da grade | 9×9 clássico | Padrão do Sudoku |
| Solução | Única garantida | Sudoku "de verdade" tem uma resposta só |
| Geração | Determinística por seed | Testes reprodutíveis + futuro "puzzle por código" |
| Critério de dificuldade | Técnicas de resolução (solver "humano") | Mais justo que contar pistas |
| Muito Difícil | Pode exigir backtracking | Quando a lógica pura não basta |
| Organização | Crate único `sudoku-core` | Simples, testável, fácil de evoluir (YAGNI sobre workspace) |
| Dependências | Apenas `rand` | Mínimo necessário; nada de Tauri/IO/UI |

**Padrão espinha-dorsal:** *Strategy* via trait `Technique` organiza as técnicas de
resolução; `enum` + `match` modelam os dados; `Result`/`Option` modelam falha e ausência.

## 4. Arquitetura interna do `sudoku-core`

Submódulos isolados, cada um com propósito único e testável de forma independente:

```
sudoku-core/
├── Cargo.toml          edition 2024, dep: rand 0.10
└── src/
    ├── lib.rs          declara os módulos e re-exporta a API pública
    ├── difficulty.rs   enum Difficulty (Facil < Medio < Dificil < MuitoDificil)
    ├── grid.rs         Grid, Cell; parsing/serialização; can_place; validate; helpers de unidade
    ├── candidates.rs   CandidateSet (bitmask u16) + candidates_for
    ├── techniques/
    │   ├── mod.rs          trait Technique + struct Deduction
    │   ├── naked_single.rs
    │   └── hidden_single.rs
    ├── solver.rs       LogicalSolver (solve + next_step) e next_hint
    ├── backtracking.rs count_solutions (unicidade por força bruta)
    ├── rating.rs       rate (classifica pela técnica mais avançada exigida)
    └── generator.rs    generate, Puzzle, GenError (grade completa → cava células)
```

Responsabilidades:

- **`grid`** — tipos base (`Grid`, `Cell = Empty | Filled(1..=9)`), parsing/serialização
  (string de 81 chars, `.` = vazio), `can_place`, detecção de conflitos linha/coluna/box.
- **`candidates`** — candidatos de uma célula como bitmask `u16` (bit *n* = dígito *n* possível).
- **`techniques`** — cada técnica implementa o trait `Technique` e devolve uma `Deduction`
  (célula resolvida + nome da técnica + nível). Adicionar técnica nova **não toca no solver**.
- **`solver`** — `LogicalSolver` aplica as técnicas em ordem e produz a **lista de passos**
  (serve tanto para o rating quanto para a dica). `next_hint` reusa a primeira dedução.
- **`backtracking`** — `count_solutions` resolve por força bruta e conta soluções até um
  limite (limite 2 ⇒ testa unicidade).
- **`rating`** — roda o solver lógico e mapeia a técnica mais avançada usada → `Difficulty`;
  cai para `MuitoDificil` quando só o backtracking resolve.
- **`generator`** — gera grade completa válida (backtracking randomizado **com seed**) e
  remove células mantendo solução única e solubilidade lógica no nível alvo.

## 5. Mapeamento dificuldade → técnica

Cada nível **inclui** as técnicas dos anteriores. O nível de um tabuleiro é a **técnica
mais avançada necessária** para resolvê-lo.

| Nível | Técnica mais avançada necessária |
|-------|----------------------------------|
| **Fácil** | Naked Single, Hidden Single |
| **Médio** | + Naked/Hidden Pairs, Pointing Pairs |
| **Difícil** | + Naked/Hidden Triples, Box-Line Reduction, X-Wing |
| **Muito Difícil** | + Swordfish, XY-Wing — e/ou backtracking quando a lógica não basta |

> A fatia em implementação cobre **apenas o nível Fácil** (Naked/Hidden Single). Puzzles
> que exigem técnicas mais avançadas aparecem temporariamente como `MuitoDificil` até as
> fatias seguintes adicionarem as técnicas intermediárias.

## 6. API pública

```rust
pub enum Difficulty { Facil, Medio, Dificil, MuitoDificil }

pub struct Puzzle { givens: Grid, solution: Grid, difficulty: Difficulty, seed: u64 }

fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>;
fn rate(grid: &Grid) -> Option<Difficulty>;
fn validate(grid: &Grid) -> Vec<Conflict>;
fn next_hint(grid: &Grid) -> Option<Deduction>;
fn count_solutions(grid: &Grid, limit: usize) -> usize;
fn candidates_for(grid: &Grid, index: usize) -> CandidateSet;
```

Tipos expostos: `Grid`, `Cell`, `Difficulty`, `Puzzle`, `Deduction`, `Conflict`,
`CandidateSet`, `ParseError`, `GenError`, `SolveResult`, e as técnicas
`NakedSingle`/`HiddenSingle` via trait `Technique`.

## 7. Fluxo de geração

```
seed → RNG → grade resolvida completa
           → remover células (mantendo unicidade via backtracking)
           → rate (classificar nível)
           → bate o nível alvo? sim → Puzzle | não → tenta outro padrão / GenError
```

Determinismo é contrato: **mesma seed + mesmo nível → puzzle idêntico, sempre.**

## 8. Stack e convenções

- **Rust** edition **2024**, crate único `sudoku-core`. Dependência única: **`rand` 0.10** (RNG com seed).
- Testes unitários **inline** em cada módulo (`#[cfg(test)] mod tests`); teste de
  integração da API pública em `sudoku-core/tests/`.
- Comandos rodam da raiz do repo via `--manifest-path sudoku-core/Cargo.toml`.
- Commits em **inglês**, padrão Conventional Commits.

## 9. Processo e skills

Metodologia **spec-driven + modular + TDD**. Skills que apoiam o fluxo (em `.agents/skills/`,
fixadas em `skills-lock.json`):

| Skill | Papel |
|-------|-------|
| `agile-roadmap` | Sequencia as fatias/fases em um roadmap. |
| `agile-tdd` | Conduz o ciclo Red-Green-Refactor de cada unidade. |
| `improve-codebase-architecture` | Acha oportunidades de aprofundar módulos (testabilidade). |

Artefatos de processo do Engine:
- **Spec:** `docs/superpowers/specs/2026-06-14-sudoku-engine-design.md`
- **Plano (fatia Fácil):** `docs/superpowers/plans/2026-06-14-engine-facil-vertical-slice.md`

## 10. Evolução futura

- Fatias seguintes adicionam técnicas Médio → Difícil → Muito Difícil (rating fica mais preciso).
- `Deduction` evolui para também representar **eliminação de candidatos** (não só colocar valor).
- Módulo Game consumirá `validate` e `next_hint`.
- **Módulo Game — timer + histórico de partidas:** cronometrar quanto o usuário leva para
  resolver e manter um histórico de partidas concluídas, ex.:
  - `Jogo Médio — 10 min — 13/06/2026`
  - `Jogo Muito Difícil — 17 min — 13/06/2026`

  O engine permanece **puro** (sem relógio nem IO): tempo, data e persistência são
  responsabilidade do módulo Game, que já recebe a `Difficulty` e o `seed` do `Puzzle`.
  Entra na **spec do módulo Game** quando ela for escrita.
- Seed pública habilita "compartilhar puzzle por código" (base do futuro multiplayer).
- Promoção a workspace multi-crate só se houver necessidade real.
