# Game — Fatia 1: Workspace e Partida Básica Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transformar o repositório num Cargo workspace e criar o crate `sudoku-game`, com uma
partida jogável: colocar números, vidas, dicas, anotações, vitória e derrota.

**Architecture:** *Functional core, imperative shell*. O `sudoku-game` é puro (sem relógio, IO ou
rede) e depende do `sudoku-core` pelo caminho. Uma `Game` guarda o `Puzzle` e o estado das 81
células, e cada ação (`place`, `toggle_note`, `hint`) devolve um resultado ou um `MoveError`, sem
nunca entrar em pânico por entrada do jogador.

**Tech Stack:** Rust edition 2024, Cargo workspace (resolver 3), `sudoku-core`. Nenhuma
dependência externa nova.

**Spec:** [`docs/superpowers/specs/2026-11-04-game-design.md`](../specs/2026-11-04-game-design.md)

## Global Constraints

- Rust edition 2024; MSRV `rust-version = "1.97"`.
- `sudoku-game` é puro: sem relógio, arquivo, rede, UI ou aleatoriedade do sistema.
- Nenhuma dependência externa nova nesta fatia. A dica usa um `splitmix64` escrito no próprio crate.
- A API pública e a saída do `sudoku-core` não mudam: os golden tests da seed 42 passam sem edição.
- Padrão: **3 vidas** e **2 dicas** por partida.
- Nomes no código em inglês; comentários, `///` e mensagens de `assert!` em português; nomes de
  funções `#[test]` em português.
- Testes unitários inline (`#[cfg(test)] mod tests`); testes da API pública em `sudoku-game/tests/`.
- Commits em inglês, Conventional Commits: um `feat`/`chore` com o código e um `docs` com a página
  de wiki da task.
- `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings` e
  `cargo test --workspace` limpos antes de cada commit.

## Review Focus

1. **Índice ≥ 81 ou valor fora de 1..=9** em `place`/`toggle_note`: devolve erro, sem perder vida
   nem entrar em pânico. Teste na Task 3 e na Task 4.
2. **Jogar numa célula que já tem número** (dada, certa ou de dica): `CellNotEmpty`, sem perder
   vida e sem contar erro. Teste na Task 3.
3. **Qualquer ação depois do fim** (`Won` ou `Lost`), incluindo dica e anotação: `GameOver`, e o
   estado não muda. Teste nas Tasks 3, 4 e 5.
4. **Dica que preenche a última célula vazia** termina a partida como vitória. Teste na Task 5.
5. **Configurações no limite:** `hints = 0` recusa a primeira dica; `lives = 1` perde no primeiro
   erro. Teste nas Tasks 3 e 5.

---

### Task 1: Workspace na raiz do repositório

Só a reorganização. Nenhum código Rust muda.

**Files:**
- Create: `Cargo.toml` (raiz), `.gitignore` (raiz)
- Move: `sudoku-core/Cargo.lock` → `Cargo.lock`; `sudoku-core/deny.toml` → `deny.toml` (com `git mv`)
- Delete: `sudoku-core/.gitignore`
- Modify: `sudoku-core/Cargo.toml`, `.github/workflows/ci.yml`, `.github/workflows/dependencias.yml`,
  `.github/dependabot.yml`, `CLAUDE.md` (seção Comandos e a linha do `cargo deny`), `README.md`
  (Development), `CONTRIBUTING.md` (comandos e MSRV), `docs/architecture.md` (seções 4 e 8),
  comentário de cabeçalho do `deny.toml`
- Create: `docs/wiki/34-cargo-workspace.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces: workspace com `members = ["sudoku-core"]`; `[workspace.package]` com `edition`,
  `rust-version`, `license` e `repository`, herdados pelos crates com `<campo>.workspace = true`.

- [ ] **Step 1: Criar a branch.** Use `feat/game-partida`, que já existe com a spec commitada.

- [ ] **Step 2: Gravar a referência.** Rode `cargo test --manifest-path sudoku-core/Cargo.toml --locked`
  e anote os totais (hoje: 123 unitários, 4 de API, 3 ignorados de desempenho).

- [ ] **Step 3: Criar o `Cargo.toml` da raiz**

```toml
[workspace]
resolver = "3"
members = ["sudoku-core"]

[workspace.package]
edition = "2024"
rust-version = "1.97"
license = "MIT"
repository = "https://github.com/IsabelaGhislandi/sudoku-core"
```

Mova para cá o comentário sobre o MSRV que hoje está em `sudoku-core/Cargo.toml`. No crate,
troque esses quatro campos por `edition.workspace = true` e equivalentes. `description`,
`readme`, `keywords`, `categories` e `publish = false` continuam no crate.

- [ ] **Step 4: Mover o lock e o deny, e trocar o `.gitignore`.** `git mv` dos dois arquivos para a
  raiz. A raiz ganha um `.gitignore` com `/target`. Apague `sudoku-core/.gitignore` e a pasta
  `sudoku-core/target`.

- [ ] **Step 5: Atualizar os comandos.** Em todo lugar listado em Files, os comandos passam a ser
  estes. O Cargo acha o workspace a partir de qualquer subpasta, então `--manifest-path` deixa de
  ser necessário:

```bash
cargo test   --workspace
cargo test   -p sudoku-core rating
cargo test   -p sudoku-core --test api
cargo build  --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt    --all
cargo deny   check
```

No CI: `cargo fmt --all --check`; clippy, build e testes com `--workspace --locked`; o
`rust-cache` perde o `workspaces: sudoku-core` (o padrão já é a raiz); a cobertura vira
`cargo llvm-cov --workspace --locked --summary-only`; o `dependencias.yml` roda `cargo deny check`.
No Dependabot, o `directory` do cargo vira `/`.

- [ ] **Step 6: Verificar.**
  - `cargo test --workspace --locked`: os mesmos totais do Step 2, golden tests incluídos.
  - `cargo clippy --workspace --all-targets --locked -- -D warnings` e `cargo fmt --all --check`: limpos.
  - `cargo deny check`: `advisories ok, bans ok, licenses ok, sources ok`.
  - `git status`: nenhum arquivo de `target/`; o `Cargo.lock` aparece como renomeado, sem diff de conteúdo.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "chore: turn the repository into a Cargo workspace"
```

- [ ] **Step 8: Wiki 34 e commit.** Página `34-cargo-workspace.md`: workspace, membros, um
  `Cargo.lock` e um `target/` compartilhados, `[workspace.package]` e herança de campos,
  `-p`/`--workspace`, resolver 3, e por que a necessidade real apareceu agora. Indexe no README da wiki.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on Cargo workspaces"
```

---

### Task 2: Crate `sudoku-game` e o começo da partida

**Files:**
- Create: `sudoku-game/Cargo.toml`, `sudoku-game/src/lib.rs`, `sudoku-game/src/config.rs`,
  `sudoku-game/src/game.rs`
- Modify: `Cargo.toml` (raiz: `members = ["sudoku-core", "sudoku-game"]`)
- Create: `docs/wiki/35-crate-sudoku-game-e-nonzero.md`,
  `docs/wiki/fundamentos/rust-base-tarefas-31-a-35.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `sudoku_core::{generate, Puzzle, Difficulty, GenError, Grid, Cell}`.
- Produces:
  - `pub struct GameConfig { pub lives: NonZeroU8, pub hints: u8 }`, com `Default` = 3 e 2.
  - `pub enum CellState { Empty, Given(u8), Placed(u8), Hinted(u8) }`, com
    `pub fn value(self) -> Option<u8>`.
  - `pub enum Status { Playing, Won, Lost }`.
  - `pub struct Game` com `pub fn new(difficulty: Difficulty, seed: u64, config: GameConfig) -> Game`,
    `pub fn from_puzzle(puzzle: Puzzle, config: GameConfig) -> Game`, `pub fn puzzle(&self) -> &Puzzle`,
    `pub fn cell(&self, index: usize) -> CellState`, `pub fn lives_left(&self) -> u8`,
    `pub fn hints_left(&self) -> u8`, `pub fn mistakes(&self) -> u32`,
    `pub fn hints_used(&self) -> u8`, `pub fn status(&self) -> Status`.
  - `lib.rs` re-exporta esses tipos e também `sudoku_core::{Difficulty, Puzzle}`, para a casca
    não precisar depender dos dois crates.

`Cargo.toml` do crate: `name = "sudoku-game"`, `version = "0.1.0"`, campos herdados do workspace,
`publish = false`, `description`, e `sudoku-core = { path = "../sudoku-core" }` como única
dependência. O `deny.toml` aceita dependência por caminho sem mudança.

- [ ] **Step 1: Testes que falham** (inline em `game.rs` e `config.rs`)

```rust
#[test]
fn config_padrao_tem_tres_vidas_e_duas_dicas() {
    let config = GameConfig::default();
    assert_eq!(config.lives.get(), 3);
    assert_eq!(config.hints, 2);
}

#[test]
fn partida_nova_comeca_jogando_com_vidas_e_dicas_cheias() {
    let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    assert_eq!(game.status(), Status::Playing);
    assert_eq!((game.lives_left(), game.hints_left()), (3, 2));
    assert_eq!((game.mistakes(), game.hints_used()), (0, 0));
    assert_eq!(game.puzzle(), &generate(Difficulty::Facil, 42).unwrap());
}

#[test]
fn celulas_dadas_e_vazias_espelham_o_enunciado() {
    let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    for i in 0..81 {
        match game.puzzle().givens.get(i) {
            Cell::Filled(v) => assert_eq!(game.cell(i), CellState::Given(v)),
            Cell::Empty => assert_eq!(game.cell(i), CellState::Empty),
        }
    }
    let empties = (0..81).filter(|&i| game.cell(i) == CellState::Empty).count();
    assert_eq!(empties, 57, "a seed 42 do Fácil tem 57 células vazias");
}

#[test]
fn seed_esgotada_passa_para_a_seguinte() {
    // A seed 15 esgota o teto do Muito Difícil; a 16 não.
    let game = Game::new(Difficulty::MuitoDificil, 15, GameConfig::default());
    assert_eq!(game.puzzle().seed, 16);
}
```

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: falha de compilação (tipos não existem).

- [ ] **Step 3: Implementar.** `Game::new` chama `generate(difficulty, seed)` e, em
  `AttemptsExhausted`, tenta `seed.wrapping_add(1)`, até conseguir. Documente essa regra no `///`
  de `new`, porque ela faz parte do determinismo: mesma seed e mesmo nível dão sempre a mesma
  partida. `from_puzzle` monta as células a partir de `puzzle.givens`. O estado guarda
  `[CellState; 81]`, vidas e dicas restantes, erros, dicas usadas e `Status`.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace`: tudo verde; clippy e fmt limpos.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock sudoku-game
git commit -m "feat: add sudoku-game crate with game setup"
```

- [ ] **Step 6: Wiki 35, consolidado 31 a 35 e commit.** A página 35 cobre: crate de biblioteca
  dependendo de outro por `path`, re-export (`pub use`) entre crates, `NonZeroU8` (tornar estados
  inválidos irrepresentáveis) e `Default`. O consolidado `rust-base-tarefas-31-a-35.md` segue o
  formato dos anteriores. Indexe os dois.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on the sudoku-game crate and consolidated doc (tasks 31-35)"
```

---

### Task 3: Colocar número, vidas, vitória e derrota

**Files:**
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/src/lib.rs`
- Create: `docs/wiki/36-jogadas-e-result.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces:
  - `pub enum PlaceOutcome { Correct, Wrong { index: usize, value: u8 } }`.
  - `pub enum MoveError { GameOver, InvalidIndex, InvalidValue, CellNotEmpty, NoHintsLeft }`
    (`NoHintsLeft` é usado na Task 5).
  - `pub fn place(&mut self, index: usize, value: u8) -> Result<PlaceOutcome, MoveError>`.

Ordem das verificações em `place`: `GameOver` → `InvalidIndex` → `InvalidValue` → `CellNotEmpty`.
Um erro não muda nada no estado. Jogada certa: a célula vira `Placed(v)`, e se não sobrar nenhuma
`Empty`, o status vira `Won`. Jogada errada: a célula **continua `Empty`**, `mistakes` sobe 1,
`lives_left` desce 1, e com zero vidas o status vira `Lost`.

Nos testes, use um helper `fn solution_value(game: &Game, index: usize) -> u8` e um
`fn wrong_value(game: &Game, index: usize) -> u8` (qualquer dígito ≠ solução), e um
`fn first_empty(game: &Game) -> usize`.

- [ ] **Step 1: Testes que falham**

```rust
#[test]
fn numero_certo_trava_na_celula() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    let i = first_empty(&game);
    let v = solution_value(&game, i);
    assert_eq!(game.place(i, v), Ok(PlaceOutcome::Correct));
    assert_eq!(game.cell(i), CellState::Placed(v));
    assert_eq!(game.place(i, v), Err(MoveError::CellNotEmpty));
    assert_eq!(game.lives_left(), 3);
}

#[test]
fn numero_errado_tira_vida_e_sai_do_tabuleiro() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    let i = first_empty(&game);
    let w = wrong_value(&game, i);
    assert_eq!(game.place(i, w), Ok(PlaceOutcome::Wrong { index: i, value: w }));
    assert_eq!(game.cell(i), CellState::Empty);
    assert_eq!((game.lives_left(), game.mistakes()), (2, 1));
    assert_eq!(game.status(), Status::Playing);
}

#[test]
fn tres_erros_terminam_a_partida_em_derrota() { /* 3 jogadas erradas → Lost; a 4ª → Err(GameOver) */ }

#[test]
fn uma_vida_perde_no_primeiro_erro() { /* GameConfig { lives: NonZeroU8::MIN, hints: 2 } → Lost após 1 erro */ }

#[test]
fn preencher_todas_as_celulas_vence() { /* coloca o valor da solução em toda Empty → Won; nova jogada → GameOver */ }

#[test]
fn jogada_em_celula_dada_nao_conta_erro() { /* célula Given, qualquer valor → CellNotEmpty, vidas e mistakes iguais */ }

#[test]
fn indice_ou_valor_invalido_nao_conta_erro() {
    // place(81, 1) → InvalidIndex; place(i, 0) e place(i, 10) → InvalidValue; vidas e mistakes iguais
}
```

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila (`place` não existe).

- [ ] **Step 3: Implementar `place`** em `game.rs` e re-exportar `PlaceOutcome` e `MoveError` em `lib.rs`.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace`; clippy e fmt limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: add number placement with lives, win and loss"
```

- [ ] **Step 6: Wiki 36 e commit.** Assunto: `Result` como valor de retorno de uma ação do jogo
  (erro de regra não é pânico), enums com dados (`Wrong { index, value }`), `&mut self` e a ordem
  das validações.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on game moves and Result"
```

---

### Task 4: Anotações

**Files:**
- Create: `sudoku-game/src/notes.rs`
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/src/lib.rs`
- Create: `docs/wiki/37-anotacoes-e-bitmask.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces:
  - `pub struct Notes(u16)` (`Copy`, `Default` vazio), com `pub fn contains(self, value: u8) -> bool`,
    `pub fn values(self) -> Vec<u8>`, `pub fn is_empty(self) -> bool` e
    `pub(crate) fn toggle(&mut self, value: u8)`.
  - Em `Game`: `pub fn notes(&self, index: usize) -> Notes` e
    `pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), MoveError>`.

Mesma ordem de verificações de `place`. `toggle_note` só vale em célula `Empty`. Uma jogada
**certa** limpa as anotações da própria célula; uma jogada **errada** não mexe nelas. As outras
células nunca mudam.

- [ ] **Step 1: Testes que falham**

```rust
#[test]
fn anotacao_liga_e_desliga() { /* toggle 3 e 7 → values() == [3, 7]; toggle 3 de novo → [7] */ }

#[test]
fn numero_certo_limpa_so_as_anotacoes_da_propria_celula() {
    // anotações 5 em duas células vazias a e b; place certo em a → notes(a) vazio, notes(b) == [5]
}

#[test]
fn numero_errado_mantem_as_anotacoes() { /* anotações [2, 4] em i; place errado em i → notes(i) == [2, 4] */ }

#[test]
fn anotacao_so_em_celula_vazia() { /* Given ou Placed → Err(CellNotEmpty) */ }

#[test]
fn anotacao_invalida_ou_depois_do_fim_eh_recusada() {
    // toggle_note(81, 1) → InvalidIndex; (i, 0) → InvalidValue; partida Lost → GameOver
}
```

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila.

- [ ] **Step 3: Implementar** `Notes` em `notes.rs` (testes unitários dela inline), guardar
  `[Notes; 81]` em `Game` e ajustar `place`.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace`; clippy e fmt limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: add per-cell pencil notes"
```

- [ ] **Step 6: Wiki 37 e commit.** Assunto: newtype sobre `u16`, `Copy`/`Default` derivados,
  visibilidade `pub(crate)` para manter a regra (só a `Game` liga e desliga), e a comparação com o
  `CandidateSet` do core: mesma máscara, papel diferente.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on pencil notes"
```

---

### Task 5: Dicas

**Files:**
- Create: `sudoku-game/src/hint.rs`
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/src/lib.rs`
- Create: `docs/wiki/38-dica-deterministica.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces:
  - `pub struct HintOutcome { pub index: usize, pub value: u8 }`.
  - `pub fn hint(&mut self) -> Result<HintOutcome, MoveError>` em `Game`.
  - Em `hint.rs`: `pub(crate) fn splitmix64(x: u64) -> u64` e
    `pub(crate) fn pick(seed: u64, hint_number: u8, empties: usize) -> usize`.

Regra do sorteio, **parte do contrato de determinismo** (documente no `///`):
`pick = splitmix64(seed ^ (hint_number as u64)) % empties`. `hint_number` é `hints_used` antes da
dica (0 na primeira) e `seed` é `puzzle.seed`. O resultado é a posição entre as células `Empty`,
em ordem crescente de índice. A célula vira `Hinted(v)` com o valor da solução, as anotações dela
somem, `hints_left` desce 1, `hints_used` sobe 1, e se não sobrar `Empty`, o status vira `Won`.
Ordem das verificações: `GameOver` → `NoHintsLeft`.

`splitmix64` é o padrão, com `wrapping_*`: primeiro soma `0x9E3779B97F4A7C15` a `x`, depois
mistura com `z = (z ^ (z >> 30)) * 0xBF58476D1CE4E5B9`, `z = (z ^ (z >> 27)) * 0x94D049BB133111EB`
e devolve `z ^ (z >> 31)`. Um teste de vetor conhecido trava a implementação:
`splitmix64(0) == 0xE220A8397B1DCDAF`.

- [ ] **Step 1: Testes que falham**

```rust
#[test]
fn splitmix64_bate_com_o_vetor_conhecido() {
    assert_eq!(splitmix64(0), 0xE220A8397B1DCDAF);
}

#[test]
fn dica_revela_o_valor_da_solucao_e_trava() {
    // h = hint(); cell(h.index) == Hinted(h.value); h.value é o da solução; hints (1, 1); vidas 3
}

#[test]
fn dica_eh_deterministica() {
    // duas partidas Facil/42 → mesma sequência de duas dicas
}

#[test]
fn dica_da_seed_42_fica_travada() {
    // Golden: a primeira dica de Game::new(Facil, 42, default) é HintOutcome { index: N, value: V }.
    // Rode uma vez, grave N e V aqui. Mudou? É bug ou decisão consciente no mesmo commit.
}

#[test]
fn dica_limpa_as_anotacoes_da_celula() { /* anota nas células vazias, dica → notes(h.index) vazio */ }

#[test]
fn sem_dicas_restantes_recusa() {
    // duas dicas → terceira Err(NoHintsLeft); e GameConfig { hints: 0, .. } recusa a primeira
}

#[test]
fn dica_na_ultima_celula_vence() { /* preenche todas menos uma com place certo; hint() → Won */ }

#[test]
fn dica_depois_do_fim_eh_recusada() { /* partida Lost → Err(GameOver), hints_left igual */ }
```

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila.

- [ ] **Step 3: Implementar** `hint.rs` e `Game::hint`.

- [ ] **Step 4: Rodar e ver passar.** Grave o golden e rode `cargo test --workspace`; clippy e fmt limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: add deterministic hints"
```

- [ ] **Step 6: Wiki 38 e commit.** Assunto: por que não usar `rand` aqui (zero dependência para
  cinco linhas), aritmética `wrapping_*` e overflow em Rust, vetor conhecido como teste, e o golden
  da dica como extensão do contrato de determinismo.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on deterministic hints"
```

---

### Task 6: API pública, testes de integração e documentação do módulo

**Files:**
- Create: `sudoku-game/tests/api.rs`
- Modify: `docs/architecture.md` (tabela de módulos: Game "em andamento: fatia 1 concluída";
  seção nova "Arquitetura do `sudoku-game`" com a árvore do crate e as camadas da spec; seção 9 com
  a spec e o plano), `docs/superpowers/specs/2026-11-04-game-design.md` (Status), `README.md`
  (Roadmap: Game "in progress"), `CHANGELOG.md` (`Added`: crate `sudoku-game`)
- Create: `docs/wiki/39-testes-de-integracao-entre-crates.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: só o que `sudoku_game` exporta (`use sudoku_game::...`), sem `use sudoku_core`.

- [ ] **Step 1: Testes de integração**

```rust
#[test]
fn partida_completa_ate_a_vitoria() {
    // Game::new(Medio, 7, default); duas dicas; place certo no resto → Won, mistakes 0, hints_used 2
}

#[test]
fn partida_completa_ate_a_derrota() {
    // Game::new(Dificil, 7, default); três place errados → Lost; place/hint/toggle_note → GameOver
}
```

O valor da solução vem de `game.puzzle().solution`, pelo re-export.

- [ ] **Step 2: Rodar.** `cargo test -p sudoku-game --test api`: deve passar de primeira, porque
  a API já existe. Se não compilar, falta algum re-export em `lib.rs`: corrija ali.

- [ ] **Step 3: Verificação final do workspace.** `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked` (golden tests do core intactos) e `cargo deny check`.

- [ ] **Step 4: Commit dos testes**

```bash
git add sudoku-game/tests
git commit -m "test: add sudoku-game public API integration tests"
```

- [ ] **Step 5: Documentação, wiki 39 e commit.** Atualize os arquivos listados. A wiki 39 cobre
  testes de integração por crate, `-p`/`--test`, e por que a casca só precisa de `sudoku_game`.

```bash
git add docs README.md CHANGELOG.md
git commit -m "docs: document sudoku-game slice 1 and add wiki page 39"
```
