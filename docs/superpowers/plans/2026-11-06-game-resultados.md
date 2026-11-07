# Game — Fatia 3: Resultados Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer toda partida do `sudoku-game` terminar num resultado (vitória, derrota por vidas
ou abandono), com estatísticas e ranking pessoal por nível e um formato salvável e versionado.

**Architecture:** Um module novo, `result.rs`, define o `GameResult` (tipado) e o `SavedResult`
(DTO de campos simples, no mesmo padrão do `SavedGame`). A `Game` ganha `result` (partida
terminada), `abandon` (consome a partida) e um `restart` que devolve o abandono. Um module
`stats.rs` tem as funções puras `level_stats` e `personal_ranking` sobre `&[GameResult]`. O `serde`
continua opcional e só deriva no `SavedResult`.

**Tech Stack:** Rust edition 2024, workspace existente. Nenhuma dependência nova (`serde` e
`serde_json` já estão no `sudoku-game`).

**Spec:** [`docs/superpowers/specs/2026-11-04-game-design.md`](../specs/2026-11-04-game-design.md), seção 12
(e seções 2, 7 e 11).

## Global Constraints

- Rust edition 2024; MSRV `rust-version = "1.97"`.
- `sudoku-game` é puro: sem relógio, arquivo, rede ou aleatoriedade. `now: u64` é o relógio
  monotônico da casca (ms); `finished_at_ms: u64` é o relógio de parede da casca (ms desde a época
  Unix, UTC). São números diferentes e nunca se misturam.
- `sudoku-core` não muda. A foto `SavedGame` versão 1 não muda: o golden test de
  `tests/save_json.rs` passa sem edição.
- Nenhuma dependência nova. Sem a feature `serde`, o crate depende só do `sudoku-core`.
- `SavedResult` **versão 1** (`RESULT_VERSION = 1`).
- Nomes no código em inglês; comentários, `///` e mensagens de `assert!` em português; nomes de
  funções `#[test]` em português.
- Testes unitários inline; testes da API pública em `sudoku-game/tests/`.
- Commits em inglês, Conventional Commits: um `feat` com o código e um `docs` com a wiki da task.
- Antes de cada commit: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace --all-features` e `cargo test --workspace` limpos.

## Decisões do plano (dentro do que a spec deixa ao builder)

- **`GameResult` com campos `pub` e `#[non_exhaustive]`.** A casca lê os campos, mas só obtém um
  resultado da `Game` ou de `GameResult::restore`. Dentro do crate, os testes montam à mão.
- **O resultado da partida terminada sai de um método** (`Game::result`), e não de `place`/`hint`.
  As assinaturas da fatia 1 e 2 não mudam, e o método é repetível (spec 12.2).
- **A porcentagem de vitórias é `Option<u8>`, arredondada para o inteiro mais próximo** (meio
  para cima): 2 de 3 → 67, 1 de 3 → 33. `None` sem partidas. Nada de ponto flutuante.
- **`parse_difficulty` passa a devolver `Option<Difficulty>`**, e cada DTO mapeia para o seu
  erro. É `pub(crate)`, então ninguém de fora sente.
- **Os refactors A e D da revisão de arquitetura não entram neste plano.** O resultado mora num
  module próprio e quase não toca a `game.rs`; os refactors podem vir depois, em PR à parte.

## Review Focus

1. **Abandonar uma partida pausada que nunca começou** (`Game::new` e direto para `abandon`):
   resultado `Abandoned` com tempo 0, não `None`. Teste na Task 2.
2. **Reiniciar uma partida terminada** ("jogar de novo") não pode devolver resultado: a casca já
   registrou o da vitória/derrota. Teste na Task 2.
3. **Pedir o resultado de uma partida terminada tempos depois** (`now` sem relação): o tempo do
   resultado é o congelado no fim, não cresce. `result` nem recebe `now`. Teste na Task 1.
4. **Nível sem partidas** na lista, ou lista vazia: `played == 0`, `best_ms == None`,
   `win_percentage() == None`, sem divisão por zero. Teste na Task 3.
5. **Registro salvo com campos estranhos** (seed `"-1"`, data `"abc"`, desfecho `"won"` com
   minúscula, `lives: 0`, versão 2): erro, nunca pânico nem resultado. Teste na Task 4.

---

### Task 1: O resultado de uma partida terminada

**Files:**
- Create: `sudoku-game/src/result.rs`
- Modify: `sudoku-game/src/lib.rs` (declarar o module, re-exportar `GameResult` e `Outcome`)
- Modify: `sudoku-game/src/game.rs` (método `result`)
- Create: `docs/wiki/45-resultado-da-partida.md`, `docs/wiki/fundamentos/rust-base-tarefas-41-a-45.md`
- Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces:
  ```rust
  // result.rs
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum Outcome { Won, OutOfLives, Abandoned }

  #[derive(Debug, Clone, PartialEq, Eq)]
  #[non_exhaustive]
  pub struct GameResult {
      pub difficulty: Difficulty,
      pub seed: u64,
      pub outcome: Outcome,
      pub elapsed_ms: u64,
      pub mistakes: u32,
      pub hints_used: u8,
      pub config: GameConfig,
      pub finished_at_ms: u64,
  }

  // game.rs
  impl Game {
      pub fn result(&self, finished_at_ms: u64) -> Option<GameResult>;
      /// Monta o resultado com o desfecho e o tempo dados. Usado por `result` e pela Task 2.
      pub(crate) fn result_with(&self, outcome: Outcome, elapsed_ms: u64, finished_at_ms: u64) -> GameResult;
  }
  ```

- [ ] **Step 1: Escrever os testes que falham** (em `game.rs`, `mod tests`, usando os helpers
  `easy_game`, `make_mistakes`, `first_empty`, `solution_value` que já existem)

```rust
const DATE: u64 = 1_781_352_000_000; // 13/06/2026 12:00 UTC

#[test]
fn partida_em_andamento_nao_tem_resultado() {
    let game = easy_game();
    assert_eq!(game.result(DATE), None);
}

#[test]
fn derrota_por_vidas_vira_resultado() {
    let mut game = easy_game();
    make_mistakes(&mut game, 3); // `place` com now = 0
    let r = game.result(DATE).unwrap();
    assert_eq!(r.outcome, Outcome::OutOfLives);
    assert_eq!((r.difficulty, r.seed), (Difficulty::Facil, game.puzzle().seed));
    assert_eq!((r.mistakes, r.hints_used), (3, 0));
    assert_eq!(r.config, GameConfig::default());
    assert_eq!(r.finished_at_ms, DATE);
}

#[test]
fn vitoria_vira_resultado_com_o_tempo_congelado() {
    let mut game = easy_game(); // rodando desde 0
    game.hint(0).unwrap();
    while let Some(i) = (0..81).find(|&i| game.cell(i) == CellState::Empty) {
        let v = solution_value(&game, i);
        game.place(i, v, 4_000).unwrap();
    }
    let r = game.result(DATE).unwrap();
    assert_eq!(r.outcome, Outcome::Won);
    assert_eq!((r.elapsed_ms, r.hints_used), (4_000, 1));
    // Pedir de novo, com outra data, só muda a data: nada na partida mudou.
    assert_eq!(game.result(DATE + 1).unwrap().elapsed_ms, 4_000);
    assert_eq!(game.result(DATE), Some(r));
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p sudoku-game resultado`
Expected: FAIL na compilação (`Outcome`/`result` não existem).

- [ ] **Step 3: Implementar `Outcome`, `GameResult`, `Game::result` e `Game::result_with`**

`result` devolve `Some` só para `Status::Won` → `Outcome::Won` e `Status::Lost` →
`Outcome::OutOfLives`, com `elapsed_ms` igual ao campo congelado (a partida terminada não tem
relógio rodando). Doc comments explicam que `finished_at_ms` é relógio de parede e que o método é
repetível (spec 12.2). `lib.rs`: `mod result;` e `pub use result::{GameResult, Outcome};`.

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test -p sudoku-game`
Expected: PASS.

- [ ] **Step 5: Commit do código**

```bash
git add sudoku-game/src
git commit -m "feat: add game result for finished games"
```

- [ ] **Step 6: Wiki da task e consolidado 41 a 45**

Página 45: enum de desfecho, `Option` para "só existe quando terminou", `#[non_exhaustive]` numa
struct (só leitura de fora) versus no `Restart`, e por que dois relógios (`now` × `finished_at_ms`).
Consolidado `rust-base-tarefas-41-a-45.md` no estilo dos anteriores. Indexar os dois no
`docs/wiki/README.md`.

```bash
git add docs/wiki
git commit -m "docs: add wiki page 45 and consolidate tasks 41 to 45"
```

---

### Task 2: Abandonar e reiniciar devolvem o abandono

**Files:**
- Modify: `sudoku-game/src/game.rs` (`abandon`, `restart`, struct `Restart`, testes existentes
  de `restart` nas linhas ~1284-1312)
- Modify: `sudoku-game/tests/api.rs` (`reiniciar_pela_api_publica` e um teste novo)
- Create: `docs/wiki/46-abandonar-e-self.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `Game::result_with`, `Outcome::Abandoned` (Task 1).
- Produces:
  ```rust
  impl Game {
      pub fn abandon(self, now: u64, finished_at_ms: u64) -> Option<GameResult>;
      pub fn restart(&self, seed: u64, now: u64, finished_at_ms: u64) -> Restart;
  }
  #[non_exhaustive]
  pub struct Restart {
      pub game: Game,
      /// O resultado do abandono, se a partida antiga não tinha terminado.
      pub abandoned: Option<GameResult>,
  }
  ```

- [ ] **Step 1: Escrever os testes que falham** (em `game.rs`)

```rust
#[test]
fn abandonar_no_meio_vira_derrota_por_abandono() {
    let mut game = easy_game(); // rodando desde 0
    make_mistakes(&mut game, 1);
    let r = game.abandon(2_500, DATE).unwrap();
    assert_eq!(r.outcome, Outcome::Abandoned);
    assert_eq!((r.elapsed_ms, r.mistakes), (2_500, 1));
}

#[test]
fn abandonar_partida_que_nunca_comecou_conta() {
    let game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    let r = game.abandon(9_000, DATE).unwrap();
    assert_eq!((r.outcome, r.elapsed_ms), (Outcome::Abandoned, 0));
}

#[test]
fn abandonar_partida_terminada_nao_da_outro_resultado() {
    let mut game = easy_game();
    make_mistakes(&mut game, 3);
    assert_eq!(game.abandon(0, DATE), None);
}

#[test]
fn reiniciar_no_meio_devolve_o_abandono() {
    let mut game = easy_game();
    game.pause(1_200);
    let restart = game.restart(2, 5_000, DATE);
    let r = restart.abandoned.unwrap();
    assert_eq!((r.outcome, r.elapsed_ms), (Outcome::Abandoned, 1_200));
    assert_eq!(restart.game.puzzle().seed, 2);
}
```

Atualizar os dois testes de `restart` existentes para `restart(seed, 0, DATE)`. Em
`reiniciar_vale_depois_do_fim`, acrescentar `assert_eq!(restart.abandoned, None)`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p sudoku-game abandon`
Expected: FAIL na compilação (`abandon` não existe, `restart` com 1 argumento).

- [ ] **Step 3: Implementar `abandon` e o novo `restart`**

Um helper privado `abandoned_result(&self, now, finished_at_ms) -> Option<GameResult>`: `None` se
`status != Playing`, senão `result_with(Outcome::Abandoned, self.elapsed_ms(now), finished_at_ms)`.
`abandon` e `restart` o chamam. Doc de `abandon` explica que consumir `self` impede jogar depois de
abandonar, e que "uma partida terminada já tem o seu resultado" (spec 12.2).

- [ ] **Step 4: Teste de integração** (em `tests/api.rs`; ajustar `reiniciar_pela_api_publica`
  para `game.restart(4, 0, 0)`)

```rust
#[test]
fn abandonar_pela_api_publica() {
    let mut game = Game::new(Difficulty::Medio, 3, GameConfig::default());
    game.resume(0);
    let Restart { abandoned, .. } = game.restart(4, 700, 1);
    assert_eq!(abandoned.map(|r| (r.outcome, r.elapsed_ms)), Some((Outcome::Abandoned, 700)));
    assert_eq!(game.abandon(700, 1).map(|r| r.outcome), Some(Outcome::Abandoned));
}
```

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test --workspace --all-features`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add sudoku-game
git commit -m "feat: return the abandonment result from abandon and restart"
```

- [ ] **Step 7: Wiki 46** — `self` versus `&self` (posse, mover a partida para dentro do método
  e o compilador recusar o uso depois), e mudar a assinatura de `restart` sem quebrar o
  `#[non_exhaustive]` de quem lê. Indexar.

```bash
git add docs/wiki
git commit -m "docs: add wiki page 46 on abandoning and taking self"
```

---

### Task 3: Estatísticas e ranking pessoal

**Files:**
- Create: `sudoku-game/src/stats.rs`
- Modify: `sudoku-game/src/lib.rs` (`mod stats;` e re-exports)
- Modify: `sudoku-game/tests/api.rs` (teste de fluxo)
- Create: `docs/wiki/47-estatisticas-e-iteradores.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `GameResult`, `Outcome` (Task 1).
- Produces:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  pub struct LevelStats { pub played: u32, pub won: u32, pub best_ms: Option<u64> }
  impl LevelStats { pub fn win_percentage(&self) -> Option<u8>; }

  pub fn level_stats(results: &[GameResult], difficulty: Difficulty) -> LevelStats;
  pub fn personal_ranking(results: &[GameResult], difficulty: Difficulty) -> Vec<&GameResult>;
  ```

- [ ] **Step 1: Escrever os testes que falham** (em `stats.rs`; um helper
  `fn result(difficulty, outcome, elapsed_ms, finished_at_ms) -> GameResult` monta à mão, com seed
  0, sem erros, sem dicas e config padrão; o `mod tests` faz `use Difficulty::*; use Outcome::*;`)

```rust
#[test]
fn estatisticas_contam_so_o_nivel_pedido() {
    let results = [
        result(Medio, Won, 9_000, 1),
        result(Medio, OutOfLives, 3_000, 2),
        result(Medio, Abandoned, 1_000, 3),
        result(Facil, Won, 500, 4),
    ];
    let s = level_stats(&results, Medio);
    assert_eq!((s.played, s.won, s.best_ms), (3, 1, Some(9_000)));
    assert_eq!(s.win_percentage(), Some(33));
}

#[test]
fn porcentagem_arredonda_para_o_mais_proximo() {
    let s = LevelStats { played: 3, won: 2, best_ms: Some(1) };
    assert_eq!(s.win_percentage(), Some(67));
}

#[test]
fn nivel_sem_partidas_nao_tem_porcentagem_nem_melhor_tempo() {
    let s = level_stats(&[], Dificil);
    assert_eq!(s, LevelStats::default());
    assert_eq!(s.win_percentage(), None);
}

#[test]
fn ranking_so_tem_vitorias_do_menor_tempo_para_o_maior() {
    let results = [
        result(Medio, Won, 9_000, 1),
        result(Medio, OutOfLives, 100, 2),
        result(Medio, Won, 4_000, 3),
        result(Facil, Won, 50, 4),
    ];
    let times: Vec<u64> = personal_ranking(&results, Medio).iter().map(|r| r.elapsed_ms).collect();
    assert_eq!(times, vec![4_000, 9_000]);
}

#[test]
fn empate_no_tempo_vai_para_a_mais_antiga() {
    let results = [result(Medio, Won, 4_000, 20), result(Medio, Won, 4_000, 10)];
    let dates: Vec<u64> = personal_ranking(&results, Medio).iter().map(|r| r.finished_at_ms).collect();
    assert_eq!(dates, vec![10, 20]);
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p sudoku-game stats`
Expected: FAIL na compilação.

- [ ] **Step 3: Implementar `stats.rs`**

`win_percentage`: `None` se `played == 0`, senão `((won * 100 + played / 2) / played) as u8`, com
a conta em `u64` para não estourar. `personal_ranking`: filtra nível e `Outcome::Won`, ordena com
`sort_by_key(|r| (r.elapsed_ms, r.finished_at_ms))`, que é estável (empate total mantém a ordem da
lista, spec 12.3). `lib.rs`: `pub use stats::{LevelStats, level_stats, personal_ranking};`.

- [ ] **Step 4: Teste de integração do fluxo** (em `tests/api.rs`)

`historico_de_tres_partidas`: uma vitória (Fácil, seed 7: `resume(0)`, colocar todas as vazias
com `now = 60_000`), uma derrota (Fácil, seed 8, 3 erros) e um abandono (Fácil, seed 9,
`resume(0)` e `restart(10, 500, 3).abandoned`). Junta os três resultados num `Vec` e confere
`level_stats(&results, Difficulty::Facil)` = `played 3, won 1, best_ms Some(60_000)`,
`win_percentage() == Some(33)`, e `personal_ranking(...).len() == 1`.

- [ ] **Step 5: Rodar e ver passar**

Run: `cargo test --workspace --all-features`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add sudoku-game
git commit -m "feat: add per-level statistics and personal ranking"
```

- [ ] **Step 7: Wiki 47** — iteradores (`filter`, `min`, `count`), `Option` no lugar de dividir por
  zero, `sort_by_key` estável com tupla como chave, e `Vec<&T>` com o tempo de vida preso à fatia
  de entrada. Indexar.

```bash
git add docs/wiki
git commit -m "docs: add wiki page 47 on statistics and iterators"
```

---

### Task 4: Resultado salvável, versionado e travado por golden test

**Files:**
- Modify: `sudoku-game/src/result.rs` (`SavedResult`, `ResultError`, `RESULT_VERSION`, `save`, `restore`)
- Modify: `sudoku-game/src/save.rs` (`parse_difficulty` → `Option<Difficulty>`) e o uso em `game.rs`
- Modify: `sudoku-game/src/lib.rs` (re-exports)
- Create: `sudoku-game/tests/result_json.rs`
- Modify: `docs/architecture.md` (seção 4.1: árvore de arquivos e "Regras da fatia 3"; status na tabela da seção 2),
  `docs/superpowers/specs/2026-11-04-game-design.md` (linha de status: fatia 3 implementada),
  `CHANGELOG.md` (`### Added`), `README.md` se ele lista a API do `sudoku-game`
- Create: `docs/wiki/48-resultado-salvavel.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `GameResult`, `Outcome` (Task 1); `difficulty_name`, `parse_difficulty` (save.rs).
- Produces:
  ```rust
  pub const RESULT_VERSION: u32 = 1;

  #[derive(Debug, Clone, PartialEq, Eq)]
  #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
  pub struct SavedResult {
      pub version: u32,
      pub difficulty: String,   // "Facil" | "Medio" | "Dificil" | "MuitoDificil"
      pub seed: String,         // u64 em decimal
      pub outcome: String,      // "Won" | "OutOfLives" | "Abandoned"
      pub elapsed_ms: u64,
      pub mistakes: u32,
      pub hints_used: u8,
      pub lives: u8,
      pub hints: u8,
      pub finished_at: String,  // ms desde a época Unix, em decimal
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub enum ResultError { UnknownVersion(u32), UnknownDifficulty, InvalidSeed, UnknownOutcome, InvalidDate, InvalidLives }

  impl GameResult {
      pub fn save(&self) -> SavedResult;
      pub fn restore(saved: &SavedResult) -> Result<GameResult, ResultError>;
  }
  ```

- [ ] **Step 1: Escrever os testes que falham** (em `result.rs`, `mod tests`; um helper
  `valid_saved()` devolve `GameResult::save` de um resultado montado à mão)

```rust
#[test]
fn resultado_vai_e_volta() {
    let r = sample(); // Medio, seed u64::MAX, Abandoned, 1_234 ms, 2 erros, 1 dica, config padrão, DATE
    assert_eq!(GameResult::restore(&r.save()), Ok(r));
}

#[test]
fn registro_estranho_eh_recusado() {
    let cases: [(fn(&mut SavedResult), ResultError); 6] = [
        (|s| s.version = 2, ResultError::UnknownVersion(2)),
        (|s| s.difficulty = "medio".into(), ResultError::UnknownDifficulty),
        (|s| s.seed = "-1".into(), ResultError::InvalidSeed),
        (|s| s.outcome = "won".into(), ResultError::UnknownOutcome),
        (|s| s.finished_at = "abc".into(), ResultError::InvalidDate),
        (|s| s.lives = 0, ResultError::InvalidLives),
    ];
    for (break_it, expected) in cases {
        let mut saved = valid_saved();
        break_it(&mut saved);
        assert_eq!(GameResult::restore(&saved), Err(expected));
    }
}
```

E em `tests/result_json.rs` (`#![cfg(feature = "serde")]`, como `save_json.rs`), o golden test:

```rust
#[test]
fn formato_do_resultado_v1_nao_muda() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    game.resume(0);
    let i = (0..81).find(|&i| game.cell(i) == CellState::Empty).unwrap();
    let wrong = if solution_value(&game, i) == 9 { 1 } else { solution_value(&game, i) + 1 };
    for now in [1_000, 2_000, 3_000] {
        game.place(i, wrong, now).unwrap();
    }
    let saved = game.result(1_781_352_000_000).unwrap().save();
    let expected = r#"{
  "version": 1,
  "difficulty": "Facil",
  "seed": "42",
  "outcome": "OutOfLives",
  "elapsed_ms": 3000,
  "mistakes": 3,
  "hints_used": 0,
  "lives": 3,
  "hints": 2,
  "finished_at": "1781352000000"
}"#;
    assert_eq!(serde_json::to_string_pretty(&saved).unwrap(), expected);
    let back: SavedResult = serde_json::from_str(expected).unwrap();
    assert_eq!(GameResult::restore(&back).unwrap(), game.result(1_781_352_000_000).unwrap());
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test -p sudoku-game --all-features resultado`
Expected: FAIL na compilação (`SavedResult` não existe).

- [ ] **Step 3: Implementar**

`parse_difficulty(name: &str) -> Option<Difficulty>`; `Game::restore` passa a fazer
`.ok_or(RestoreError::UnknownDifficulty)?` (o teste `nome_do_nivel_vai_e_volta` muda para
`Some`/`None`). Os nomes do desfecho seguem o mesmo par `outcome_name`/`parse_outcome` com `match`
exaustivo. `restore` confere a versão primeiro, depois os campos na ordem da struct; não confere
coerência entre eles (spec 12.4). `lib.rs`: `pub use result::{GameResult, Outcome, RESULT_VERSION, ResultError, SavedResult};`.

- [ ] **Step 4: Rodar e ver passar, nas duas configurações**

Run: `cargo test --workspace --all-features` e `cargo test --workspace`
Expected: PASS nos dois; `save_json.rs` sem edição.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: make game results savable with a versioned format"
```

- [ ] **Step 6: Documentação da fatia**

`architecture.md` (seção 2: Game "fatias 1, 2 e 3 concluídas"; seção 4.1: `result.rs` e
`stats.rs` na árvore e um parágrafo "Regras da fatia 3"), status da spec, `CHANGELOG.md`, e a
wiki 48 (um segundo DTO no mesmo padrão, versão por registro e por que o golden test vale mais
num histórico de meses). Indexar.

```bash
git add docs CHANGELOG.md README.md
git commit -m "docs: document Game slice 3 and add wiki page 48"
```
