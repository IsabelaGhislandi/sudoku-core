# Arquitetura — Sudoku

> Visão de **design** do projeto. Para os aprendizados de Rust durante a implementação,
> veja a [wiki](wiki/README.md). Para o detalhe completo de cada decisão, veja a
> [spec do Core/Engine](superpowers/specs/2026-06-14-sudoku-engine-design.md), a
> specs das fatias [Médio](superpowers/specs/2026-11-01-engine-medio-design.md) e
> [Difícil](superpowers/specs/2026-11-02-engine-dificil-design.md), e os planos das fatias
> [Fácil](superpowers/plans/2026-06-14-engine-facil-vertical-slice.md),
> [Médio](superpowers/plans/2026-11-01-engine-medio.md) e
> [Difícil](superpowers/plans/2026-11-02-engine-dificil.md).

## 1. Visão geral

Jogo de Sudoku para celular (APK Android via **Tauri**), com backend em **Rust** e
frontend em **Svelte**. Desenvolvimento **modular** e **spec-driven**: cada módulo segue
o ciclo `spec → plano → implementação` com **TDD**.

## 2. Decomposição em módulos

| # | Módulo | Responsabilidade | Status |
|---|--------|------------------|--------|
| 1 | **Core/Engine** (`sudoku-core`) | Gerar, resolver, validar e classificar tabuleiros. Lógica pura. | **fatias Fácil, Médio e Difícil concluídas** |
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
| Dependências | `rand` (sem features padrão) + `rand_chacha` | Mínimo necessário; nada de Tauri/IO/UI nem aleatoriedade do SO |

**Padrão espinha-dorsal:** *Strategy* via trait `Technique` organiza as técnicas de
resolução; `enum` + `match` modelam os dados; `Result`/`Option` modelam falha e ausência.

## 4. Arquitetura interna do `sudoku-core`

Submódulos isolados, cada um com propósito único e testável de forma independente:

```
sudoku-core/
├── Cargo.toml          edition 2024, publish = false, deps: rand 0.10 (sem default), rand_chacha 0.10
├── deny.toml           regras do cargo-deny (RustSec, licenças, origem)
└── src/
    ├── lib.rs          declara os módulos e re-exporta a API pública
    ├── difficulty.rs   enum Difficulty (Facil < Medio < Dificil < MuitoDificil)
    ├── grid.rs         Grid, Cell; parsing/serialização; can_place; validate; helpers de unidade
    ├── candidates.rs   CandidateSet (bitmask u16), candidates_for, CandidateGrid
    ├── techniques/
    │   ├── mod.rs          trait Technique, Deduction, Action, units() (tabela fixa)
    │   ├── naked_single.rs     Fácil
    │   ├── hidden_single.rs    Fácil
    │   ├── naked_pair.rs       Médio
    │   ├── hidden_pair.rs      Médio
    │   ├── pointing_pair.rs    Médio
    │   ├── box_line_reduction.rs  Difícil
    │   ├── naked_triple.rs     Difícil
    │   ├── hidden_triple.rs    Difícil
    │   └── x_wing.rs           Difícil
    ├── solver.rs       LogicalSolver (new, up_to, solve, next_step)
    ├── backtracking.rs count_solutions (unicidade por força bruta)
    ├── rating.rs       rate (classifica pela técnica mais avançada exigida)
    └── generator.rs    generate, Puzzle, GenError (grade completa → cava células)
```

Testes de integração em `sudoku-core/tests/`: `api.rs` (fluxos Fácil, Médio e Difícil pela API pública) e
`desempenho.rs` (medição dos geradores de Médio e Difícil, `#[ignore]`, só em release).

Responsabilidades:

- **`grid`** — tipos base (`Grid`, `Cell = Empty | Filled(1..=9)`), parsing/serialização
  (string de 81 chars, `.` = vazio), `can_place`, detecção de conflitos linha/coluna/box.
- **`candidates`** — candidatos de uma célula como bitmask `u16` (bit *n* = dígito *n* possível).
  `CandidateGrid` guarda os candidatos de todas as células durante uma resolução, porque as
  eliminações das técnicas de Médio e Difícil não aparecem na grade.
- **`techniques`** — cada técnica implementa o trait `Technique` e devolve uma `Deduction`
  (uma `Action`, `Place` ou `Eliminate`, mais o nome da técnica e o nível). Uma técnica só
  devolve `Eliminate` se remover ao menos um candidato presente, para o solver não entrar em
  loop. Adicionar técnica nova **não toca em `solve`**.
- **`solver`** — `LogicalSolver` aplica as técnicas em ordem, da mais simples para a mais
  difícil, e produz a **lista de passos** usada pelo rating. `up_to(nível)` limita as técnicas;
  cada gerador usa `up_to(nível)`, e as técnicas novas entram no **fim** da lista: é isso que
  mantém inalterados os puzzles dos níveis já existentes.
- **`backtracking`** — `count_solutions` resolve por força bruta (ramificando na célula com
  menos candidatos — heurística MRV) e conta soluções até um
  limite (limite 2 ⇒ testa unicidade).
- **`rating`** — roda o solver lógico e mapeia a técnica mais avançada usada → `Difficulty`;
  cai para `MuitoDificil` quando só o backtracking resolve.
- **`generator`** — gera grade completa válida (backtracking randomizado **com seed**) e
  remove células enquanto o solver do nível alvo ainda resolve, o que também garante solução
  única. No Médio e no Difícil, repete com o mesmo RNG até o puzzle exigir o nível pedido, até
  um número fixo de tentativas (`MAX_ATTEMPTS_MEDIUM` = 50, `MAX_ATTEMPTS_HARD` = 250,
  parte do contrato de determinismo). O aceite equivale a `rate`, sem repetir a busca de
  unicidade.

## 5. Mapeamento dificuldade → técnica

Cada nível **inclui** as técnicas dos anteriores. O nível de um tabuleiro é a **técnica
mais avançada necessária** para resolvê-lo.

| Nível | Técnica mais avançada necessária |
|-------|----------------------------------|
| **Fácil** | Naked Single, Hidden Single |
| **Médio** | + Naked/Hidden Pairs, Pointing Pairs |
| **Difícil** | + Naked/Hidden Triples, Box-Line Reduction, X-Wing |
| **Muito Difícil** | + Swordfish, XY-Wing — e/ou backtracking quando a lógica não basta |

> As fatias concluídas cobrem **Fácil, Médio e Difícil**. O Pointing Pair também vale para 3
> células alinhadas no box. A Box/Line Reduction fica no Difícil, e não no Médio, porque no
> Médio mudaria todos os puzzles Médios já gerados. Puzzles que exigem técnicas além do Difícil
> aparecem como `MuitoDificil`, até a fatia Muito Difícil.

## 6. API pública

```rust
pub enum Difficulty { Facil, Medio, Dificil, MuitoDificil }

pub struct Puzzle { givens: Grid, solution: Grid, difficulty: Difficulty, seed: u64 }

fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>;
fn rate(grid: &Grid) -> Option<Difficulty>;
fn validate(grid: &Grid) -> Vec<Conflict>;
fn count_solutions(grid: &Grid, limit: usize) -> usize;
fn candidates_for(grid: &Grid, index: usize) -> CandidateSet;

pub enum Action { Place { index: usize, value: u8 }, Eliminate(Vec<(usize, u8)>) }
pub struct Deduction { action: Action, technique: &'static str, level: Difficulty }
pub enum GenError { Unsupported(Difficulty), AttemptsExhausted(Difficulty) }
```

`generate` aceita Fácil, Médio e Difícil; Muito Difícil devolve `GenError::Unsupported`.

Tipos expostos: `Grid`, `Cell`, `Difficulty`, `Puzzle`, `Deduction`, `Action`, `Conflict`,
`CandidateSet`, `CandidateGrid`, `ParseError`, `GenError`, `LogicalSolver`, `SolveResult`, e as
técnicas `NakedSingle`, `HiddenSingle`, `NakedPair`, `HiddenPair`, `PointingPair`,
`BoxLineReduction`, `NakedTriple`, `HiddenTriple` e `XWing` via trait `Technique`.

**A dica do jogo não fica no core.** Ela sorteia uma célula vazia e copia o valor de
`Puzzle.solution`, então pertence ao módulo Game. O antigo `next_hint` foi removido.

## 7. Fluxo de geração

```
seed → RNG → grade resolvida completa
           → remover células enquanto o solver do nível resolve (o que garante unicidade)
           → classificar o nível (mesmo resultado de rate)
           → bate o nível alvo? sim → Puzzle | não → nova tentativa, até o limite → GenError
```

Determinismo é contrato: **mesma seed + mesmo nível → puzzle idêntico, sempre** — inclusive
entre versões da lib, porque o RNG é o `ChaCha12Rng` (algoritmo fixo) e não o `StdRng`, cujo
algoritmo o `rand` pode trocar. Um *golden test* por nível trava a saída da seed 42 para
flagrar qualquer mudança acidental.

## 8. Stack e convenções

- **Rust** edition **2024**, crate único `sudoku-core`. Dependências: **`rand` 0.10**, sem as
  features padrão (o core não usa StdRng nem a aleatoriedade do SO), e **`rand_chacha` 0.10** (`ChaCha12Rng`, de algoritmo fixo, para o determinismo da seed).
- Testes unitários **inline** em cada módulo (`#[cfg(test)] mod tests`); teste de
  integração da API pública em `sudoku-core/tests/`.
- Comandos rodam da raiz do repo via `--manifest-path sudoku-core/Cargo.toml`.
- Commits em **inglês**, padrão Conventional Commits.
- Código formatado com `rustfmt` (configuração padrão); finais de linha LF via `.gitattributes`.
- **CI** (GitHub Actions, `.github/workflows/ci.yml`): fmt, clippy, build e testes em
  Ubuntu/Windows/macOS a cada PR, com `-D warnings` e `--locked`. **CD** fica para o módulo
  Tauri: build do APK a cada tag `v*`, anexado a uma GitHub Release.
  Spec: `docs/superpowers/specs/2026-09-19-ci-design.md`.
- **Dependências** (`.github/workflows/dependencias.yml`): `cargo deny check` com as regras de
  `sudoku-core/deny.toml` a cada PR e toda segunda-feira. Ao adicionar uma dependência, o agente segue a
  skill `.claude/skills/adding-dependencies` (verificar o crate e pedir aprovação antes do `cargo add`).

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
- **Spec (fatia Médio):** `docs/superpowers/specs/2026-11-01-engine-medio-design.md`
- **Plano (fatia Médio):** `docs/superpowers/plans/2026-11-01-engine-medio.md`
- **Spec (fatia Difícil):** `docs/superpowers/specs/2026-11-02-engine-dificil-design.md`
- **Plano (fatia Difícil):** `docs/superpowers/plans/2026-11-02-engine-dificil.md`

## 10. Evolução futura

- A próxima fatia adiciona as técnicas de Muito Difícil (rating fica mais preciso).
- Módulo Game trata `AttemptsExhausted` do Difícil (cerca de 1 seed em 5.000) sorteando outra seed.
- Módulo Game consumirá `validate` e implementará a dica, sorteando uma célula vazia e usando
  `Puzzle.solution`. Como o valor vem da solução, jogadas erradas não afetam a dica.
- A meta de tempo (até 5 s no celular, estimada como 1 s neste PC) foi medida só neste PC, em
  200 seeds: pior caso de 44 ms no Médio; no Difícil, cerca de 1 s com o PC livre, limitado pelas
  250 tentativas. Confirmar em um aparelho real quando o módulo Tauri existir.
- **Módulo Game — timer + histórico de partidas:** cronometrar quanto o usuário leva para
  resolver e manter um histórico de partidas concluídas, ex.:
  - `Jogo Médio — 10 min — 13/06/2026`
  - `Jogo Muito Difícil — 17 min — 13/06/2026`

  O engine permanece **puro** (sem relógio nem IO): tempo, data e persistência são
  responsabilidade do módulo Game, que já recebe a `Difficulty` e o `seed` do `Puzzle`.
  Entra na **spec do módulo Game** quando ela for escrita.
- Seed pública habilita "compartilhar puzzle por código" (base do futuro multiplayer).
- Promoção a workspace multi-crate só se houver necessidade real.
