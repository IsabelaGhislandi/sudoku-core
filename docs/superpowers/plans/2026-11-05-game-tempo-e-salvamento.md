# Game — Fatia 2: Tempo e Salvamento Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dar à partida do `sudoku-game` um relógio (pausar, retomar, tempo jogado), uma "foto"
salvável e restaurável com validação, serialização opcional via `serde` e o atalho de reiniciar.

**Architecture:** O relógio são dois campos na `Game` (tempo acumulado e o instante em que voltou a
rodar), alimentados por instantes em milissegundos que a casca informa. A foto é um tipo próprio,
`SavedGame`, com campos simples (texto e números): o `sudoku-core` não ganha `serde`, e a JSON fica
legível e segura para JavaScript. `Game::restore` confere a foto inteira antes de virar partida. O
`serde` só entra com a feature `serde`, que só acrescenta `derive`s ao `SavedGame`.

**Tech Stack:** Rust edition 2024, workspace existente. Novas: `serde` (opcional, feature `serde`)
e `serde_json` (só `dev-dependencies`), ambas pela skill `adding-dependencies`.

**Spec:** [`docs/superpowers/specs/2026-11-04-game-design.md`](../specs/2026-11-04-game-design.md), seção 11
(e seções 2, 5 e 7).

## Global Constraints

- Rust edition 2024; MSRV `rust-version = "1.97"` (não subir sem motivo registrado).
- `sudoku-game` é puro: sem relógio, arquivo, rede ou aleatoriedade do sistema. Instantes entram
  como `now: u64`, em milissegundos de um relógio monotônico da casca.
- `sudoku-core` não muda: nem API, nem saída, nem `Cargo.toml`. Os golden tests da seed 42 passam
  sem edição.
- Sem a feature `serde`, o `sudoku-game` continua dependendo só do `sudoku-core`.
- Dependência nova só depois do resumo de aprovação da skill `adding-dependencies` e do "ok" da
  pessoa; `serde` com `default-features = false`; `cargo deny check` passando.
- Foto **versão 1**. Restaurada, a partida vem sempre pausada.
- Nomes no código em inglês; comentários, `///` e mensagens de `assert!` em português; nomes de
  funções `#[test]` em português.
- Testes unitários inline; testes da API pública em `sudoku-game/tests/`.
- Commits em inglês, Conventional Commits: um `feat` com o código e um `docs` com a wiki da task.
- Antes de cada commit: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace --all-features` e `cargo test --workspace` limpos.

## Decisões do plano (revisões da spec, com motivo)

- **`status` não vai na foto: é derivado do tabuleiro e das vidas** (sem célula vazia → `Won`;
  zero vidas → `Lost`; senão `Playing`). A spec pede que um status incoerente seja recusado.
  Derivá-lo elimina essa classe inteira de foto incoerente e deixa uma única checagem:
  zero vidas **e** tabuleiro completo é impossível (`InconsistentStatus`).
- **A seed vai na foto como texto decimal.** JSON lida por JavaScript perde precisão acima de
  2^53, e a seed é um `u64` qualquer. `elapsed_ms` fica número (2^53 ms são 285 mil anos).
- **O relógio para no fim porque `place` e `hint` recebem `now`.** É a forma mais direta de a
  vitória ou derrota congelar o tempo sem depender da casca (spec 11.5 autoriza mudar a
  assinatura). `toggle_note` não recebe, porque nunca termina a partida.
- **Partida nova começa pausada.** "Começar" é o primeiro `resume(now)`. É o mesmo estado de uma
  partida restaurada, e a casca trata os dois casos igual.

## Review Focus

1. **Relógio que volta** (`now` menor que o instante do último `resume`, por exemplo depois de
   reiniciar o aparelho): o tempo não pode dar underflow nem pânico; conta zero naquele trecho
   (`saturating_sub`). Teste na Task 1.
2. **Salvar com a partida rodando** inclui o trecho em andamento até `now` e **não pausa** a
   partida original. Teste na Task 2.
3. **Foto com tamanhos errados** (tabuleiro com 80 caracteres, 80 anotações, índice de dica
   ≥ 81, dígito de anotação 0 ou repetido): erro, nunca pânico nem índice fora do array. Teste na
   Task 3.
4. **Dica depois de restaurar** cai na mesma célula que cairia na partida original. Teste na
   Task 2.
5. **Seed maior que 2^53** sobrevive à ida e volta em JSON. Teste na Task 4.

---

### Task 1 (wiki 40): Relógio da partida

**Files:**
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/tests/api.rs`
- Create: `docs/wiki/40-relogio-injetado-e-saturating.md`,
  `docs/wiki/fundamentos/rust-base-tarefas-36-a-40.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Produces:
  - Campos novos na `Game`: `elapsed_ms: u64` (acumulado até a última pausa) e
    `running_since: Option<u64>` (`Some(t)` = rodando desde `t`).
  - `pub fn resume(&mut self, now: u64)`, `pub fn pause(&mut self, now: u64)`: sem efeito se já
    estiver no estado pedido ou se a partida terminou.
  - `pub fn is_paused(&self) -> bool`: `status == Playing && running_since.is_none()`.
  - `pub fn elapsed_ms(&self, now: u64) -> u64`: acumulado mais `now.saturating_sub(t)` se rodando.
  - `MoveError::Paused` (variante nova, a última do enum).
  - Assinaturas mudadas: `place(&mut self, index: usize, value: u8, now: u64)`,
    `hint(&mut self, now: u64)`. `toggle_note` igual.
- Ordem das recusas em `place`/`toggle_note`: `GameOver` → `Paused` → `InvalidIndex` →
  `InvalidValue` → `CellNotEmpty`. Em `hint`: `GameOver` → `Paused` → `NoHintsLeft`.
- Ao virar `Won` ou `Lost`, a `Game` soma o trecho até `now` em `elapsed_ms` e zera
  `running_since`. Depois do fim, `elapsed_ms(qualquer now)` devolve sempre o mesmo valor.

- [ ] **Step 1: Ajustar os testes existentes.** Todos os testes da fatia 1 (inline e `tests/api.rs`)
  passam a chamar `game.resume(0)` logo depois de criar a partida e `now` nas chamadas de
  `place`/`hint` (use `0` quando o tempo não importa). Crie nos testes inline um helper
  `fn started(difficulty: Difficulty, seed: u64) -> Game` que cria com `GameConfig::default()` e
  já chama `resume(0)`.

- [ ] **Step 2: Testes novos que falham**

```rust
#[test]
fn partida_nova_comeca_pausada_e_recusa_acoes() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    let i = first_empty(&game);
    assert!(game.is_paused());
    assert_eq!(game.place(i, 1, 0), Err(MoveError::Paused));
    assert_eq!(game.toggle_note(i, 1), Err(MoveError::Paused));
    assert_eq!(game.hint(0), Err(MoveError::Paused));
    assert_eq!((game.lives_left(), game.hints_left()), (3, 2));
}

#[test]
fn tempo_jogado_nao_conta_as_pausas() {
    let mut game = Game::new(Difficulty::Facil, 42, GameConfig::default());
    game.resume(1_000);
    assert_eq!(game.elapsed_ms(4_000), 3_000);
    game.pause(4_000);
    assert_eq!(game.elapsed_ms(60_000), 3_000);
    game.resume(60_000);
    assert_eq!(game.elapsed_ms(61_500), 4_500);
}

#[test]
fn pausar_ou_retomar_duas_vezes_nao_muda_nada() {
    // resume(1_000), resume(5_000) → elapsed_ms(6_000) == 5_000 (o segundo resume é ignorado);
    // pause(6_000), pause(9_000) → elapsed_ms(20_000) == 5_000
}

#[test]
fn relogio_que_volta_nao_quebra_o_tempo() {
    // resume(10_000); elapsed_ms(4_000) == 0; pause(4_000); elapsed_ms(4_000) == 0
}

#[test]
fn vitoria_para_o_relogio() {
    // started(Facil, 42); preenche todas as vazias com now = 7_000 na última;
    // elapsed_ms(7_000) == 7_000 e elapsed_ms(99_000) == 7_000; is_paused() == false (terminou)
}

#[test]
fn derrota_para_o_relogio() {
    // started(Facil, 42); três erros, o último com now = 2_500 → elapsed_ms(50_000) == 2_500
}

#[test]
fn dica_que_vence_para_o_relogio() {
    // partida de from_puzzle com uma só célula vazia (monte a partir de generate(Facil, 42)),
    // resume(0), hint(800) → Won e elapsed_ms(10_000) == 800
}
```

- [ ] **Step 3: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila (`resume`,
  `elapsed_ms`, `Paused` e as assinaturas novas não existem).

- [ ] **Step 4: Implementar** em `game.rs` conforme o bloco Interfaces. A transição para o fim fica
  num lugar só (por exemplo, `fn end(&mut self, status: Status, now: u64)`), usado por `place` e
  pelo `finish_if_complete`, que passa a receber `now`.

- [ ] **Step 5: Rodar e ver passar.** `cargo test --workspace`; fmt e clippy limpos.

- [ ] **Step 6: Commit**

```bash
git add sudoku-game
git commit -m "feat: add pause, resume and elapsed time to Game"
```

- [ ] **Step 7: Wiki 40, consolidado 36–40 e commit.** Página 40: relógio injetado (por que o
  núcleo não lê `Instant::now()`, e como isso permite testar tempo sem esperar), `Option<u64>`
  como estado "rodando desde", `saturating_sub` vs. `-`, e mudar uma assinatura pública com o
  compilador apontando cada chamada. Consolidado `rust-base-tarefas-36-a-40.md` no formato dos
  anteriores (intensivo por conceito, desde a base), cobrindo as páginas 36 a 40. Indexe os dois
  no `docs/wiki/README.md`.

```bash
git add docs/wiki
git commit -m "docs: add wiki page 40 and fundamentals for tasks 36-40"
```

---

### Task 2 (wiki 41): Foto da partida, salvar e restaurar

**Files:**
- Create: `sudoku-game/src/save.rs`
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/src/lib.rs`
- Create: `docs/wiki/41-foto-da-partida-e-dto.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `elapsed_ms(now)`, `is_paused()` (Task 1).
- Produces:
  - `pub const SAVE_VERSION: u32 = 1;`
  - Em `save.rs`:

```rust
/// Foto de uma partida, em campos simples para virar texto legível.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedGame {
    pub version: u32,
    pub difficulty: String,   // "Facil" | "Medio" | "Dificil" | "MuitoDificil"
    pub seed: String,         // u64 em decimal
    pub givens: String,       // Grid::to_line
    pub solution: String,     // Grid::to_line
    pub board: String,        // Grid::to_line do tabuleiro atual (dados + certos + dicas)
    pub hinted: Vec<u8>,      // índices das células reveladas por dica, em ordem crescente
    pub notes: Vec<String>,   // 81 entradas; "" ou os dígitos anotados em ordem, ex. "37"
    pub lives: u8,            // vidas da configuração
    pub hints: u8,            // dicas da configuração
    pub lives_left: u8,
    pub hints_left: u8,
    pub mistakes: u32,
    pub hints_used: u8,
    pub elapsed_ms: u64,
}
```

  - `pub enum RestoreError { UnknownVersion(u32), UnknownDifficulty, InvalidSeed, InvalidGrid, Puzzle(PuzzleError), BoardMismatch, InvalidHints, InvalidNotes, InvalidLives, InconsistentStatus }`
    com `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`. Nesta task, `restore` produz só
    `UnknownVersion`, `UnknownDifficulty`, `InvalidSeed`, `InvalidGrid` e `Puzzle(_)`; a Task 3
    produz o resto.
  - `Game` guarda a configuração: campo `config: GameConfig`.
  - `pub fn save(&self, now: u64) -> SavedGame` (não muda a partida).
  - `pub fn restore(saved: &SavedGame) -> Result<Game, RestoreError>` (sempre pausada).
- Montagem na restauração: `Puzzle` a partir de difficulty, seed, givens e solution, conferido
  por `from_puzzle` (erro vira `RestoreError::Puzzle`); células `Given` onde `givens` tem número,
  `Hinted` nos índices de `hinted`, `Placed` nos outros números do `board`; `status` derivado
  (ver "Decisões do plano"); `running_since = None`.
- Re-export em `lib.rs`: `SavedGame`, `RestoreError`, `SAVE_VERSION`.

- [ ] **Step 1: Testes que falham** (inline em `save.rs` ou `game.rs`)

```rust
#[test]
fn restaurar_devolve_a_mesma_partida_pausada() {
    let mut game = started(Difficulty::Medio, 7);
    // uma jogada certa, uma errada, uma anotação numa vazia, uma dica, now crescente até 9_000
    let saved = game.save(9_000);
    let restored = Game::restore(&saved).unwrap();
    assert!(restored.is_paused());
    assert_eq!(restored.elapsed_ms(123_456), game.elapsed_ms(9_000));
    for i in 0..81 {
        assert_eq!(restored.cell(i), game.cell(i));
        assert_eq!(restored.notes(i), game.notes(i));
    }
    assert_eq!(
        (restored.lives_left(), restored.hints_left(), restored.mistakes(), restored.hints_used(), restored.status()),
        (game.lives_left(), game.hints_left(), game.mistakes(), game.hints_used(), game.status()),
    );
    assert_eq!(restored.puzzle(), game.puzzle());
}

#[test]
fn salvar_rodando_inclui_o_trecho_atual_e_nao_pausa() {
    // started(Facil, 42) (rodando desde 0); save(5_000).elapsed_ms == 5_000;
    // game.is_paused() == false depois do save
}

#[test]
fn dica_depois_de_restaurar_cai_na_mesma_celula() {
    // started(Dificil, 3); hint(0); clone = Game::restore(&game.save(0)); clone.resume(0);
    // game.hint(0) == clone.hint(0)
}

#[test]
fn partida_terminada_restaura_terminada() {
    // vitória e derrota: restore(&save(t)).status() igual, is_paused() == false, elapsed igual
}

#[test]
fn foto_guarda_seed_e_nivel_em_texto() {
    // Game::new(MuitoDificil, u64::MAX, default): save(0).seed == puzzle().seed.to_string()
    // e difficulty == "MuitoDificil"; version == SAVE_VERSION
}

#[test]
fn versao_nivel_ou_seed_desconhecidos_sao_recusados() {
    // version 2 → UnknownVersion(2); difficulty "Impossivel" → UnknownDifficulty;
    // seed "abc" e "-1" → InvalidSeed; givens com 80 caracteres → InvalidGrid
}
```

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila.

- [ ] **Step 3: Implementar** `save.rs` (tipo, versão, erro, conversões de nível) e `save`/`restore`
  em `game.rs` (que enxerga os campos privados). A conversão nível↔texto é um `match` exaustivo
  nos dois sentidos, para que um nível novo no core quebre a compilação aqui.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace`; fmt e clippy limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: add SavedGame snapshot with save and restore"
```

- [ ] **Step 6: Wiki 41 e commit.** Assunto: o padrão DTO (um tipo só para transporte, separado do
  tipo com regras), por que o `Game` não vira público campo a campo, `to_string`/`parse` e
  `Result` em conversões, e o limite de 2^53 do JSON em JavaScript.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on game snapshots and DTOs"
```

---

### Task 3 (wiki 42): Restauração recusa foto incoerente

**Files:**
- Modify: `sudoku-game/src/save.rs` e/ou `sudoku-game/src/game.rs`
- Create: `docs/wiki/42-validar-entrada-de-fora.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `SavedGame`, `RestoreError`, `Game::restore` (Task 2).
- Produces: `restore` passa a produzir todos os `RestoreError`. Ordem das checagens (a primeira que
  falhar é a devolvida): versão → nível → seed → `givens`/`solution`/`board` legíveis
  (`InvalidGrid`) → puzzle (`Puzzle(_)`) → `board` (`BoardMismatch`) → `hinted` (`InvalidHints`)
  → `notes` (`InvalidNotes`) → vidas (`InvalidLives`) → dicas (`InvalidHints`) → status
  (`InconsistentStatus`).
- Regras:
  - `BoardMismatch`: onde `givens` tem número, `board` tem o mesmo; nos outros números do
    `board`, o valor é o da solução.
  - `InvalidHints`: índice ≥ 81, repetido ou fora de ordem, apontando para célula vazia ou dada no
    `board`; `hinted.len() != hints_used`; `hints_left + hints_used != hints`.
  - `InvalidNotes`: `notes.len() != 81`; caractere fora de `1..=9`; dígito repetido ou fora de
    ordem; anotação numa célula com número no `board`.
  - `InvalidLives`: `lives == 0`; `mistakes > lives`; `lives_left != lives - mistakes`.
  - `InconsistentStatus`: `lives_left == 0` e `board` completo.

- [ ] **Step 1: Testes que falham.** Um helper `fn valid_save() -> SavedGame` (partida Médio seed 7
  com jogada certa, erro, anotação e dica), e um teste por regra que adultera um campo e confere o
  erro exato:

```rust
#[test]
fn numero_errado_travado_eh_recusado() {
    let mut saved = valid_save();
    // troca, no board, o dígito de uma célula Placed por outro diferente da solução
    assert_eq!(Game::restore(&saved), Err(RestoreError::BoardMismatch));
}

#[test]
fn vida_recuperada_eh_recusada() {
    let mut saved = valid_save();
    saved.lives_left += 1;
    assert_eq!(Game::restore(&saved), Err(RestoreError::InvalidLives));
}
```

  Os outros, no mesmo formato: `numero_dado_trocado_eh_recusado` (BoardMismatch),
  `dica_fora_do_tabuleiro_eh_recusada` (índice 81 → InvalidHints, sem pânico),
  `dica_em_celula_vazia_eh_recusada`, `dicas_que_nao_somam_sao_recusadas`,
  `anotacoes_com_tamanho_errado_sao_recusadas` (80 entradas), `anotacao_com_digito_zero_eh_recusada`,
  `anotacao_repetida_eh_recusada` ("33"), `anotacao_em_celula_preenchida_eh_recusada`,
  `configuracao_sem_vidas_eh_recusada`, `derrota_com_tabuleiro_completo_eh_recusada`,
  `puzzle_invalido_na_foto_eh_recusado` (solução com conflito → `Puzzle(InvalidSolution)`).
  E um teste de que `valid_save()` restaura sem erro.

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game restore`: as adulterações passam
  sem erro ou dão pânico.

- [ ] **Step 3: Implementar** as checagens na ordem do bloco Interfaces. Toda leitura de índice
  vindo da foto passa pela checagem de faixa antes de indexar um array.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace`; fmt e clippy limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: reject inconsistent snapshots on restore"
```

- [ ] **Step 6: Wiki 42 e commit.** Assunto: "parse, don't validate" (a `Game` só existe se a foto
  passou), por que dado de fora nunca indexa array sem checagem, erros como enum com variante que
  embrulha outro erro (`Puzzle(PuzzleError)`), e testes de adulteração campo a campo.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on validating untrusted input"
```

---

### Task 4 (wiki 43): Feature `serde` e golden test do formato

**Files:**
- Modify: `sudoku-game/Cargo.toml`, `Cargo.lock`, `sudoku-game/src/save.rs`,
  `.github/workflows/ci.yml`, `CLAUDE.md` (seção Comandos), `CONTRIBUTING.md`, `README.md`
  (Development), `deny.toml` (comentário de licenças, se a lista de crates mudar)
- Create: `sudoku-game/tests/save_json.rs`
- Create: `docs/wiki/43-features-opcionais-e-serde.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `SavedGame` (Task 2), `Game::save`/`restore`.
- Produces: feature `serde = ["dep:serde"]` no `sudoku-game`; com ela, `SavedGame` tem
  `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`. Nada mais muda.

- [ ] **Step 1: Verificar e pedir aprovação.** Siga a skill `adding-dependencies` para `serde`
  (`[dependencies]`, `optional = true`, `default-features = false`, features `derive` e `alloc`
  — confirme no docs.rs quais são necessárias para `String`/`Vec`) e para `serde_json`
  (`[dev-dependencies]`). Mostre os dois resumos de aprovação e **pare até a pessoa aprovar**.
  Atenção: a skill ainda cita `sudoku-core/Cargo.toml` e `sudoku-core/deny.toml`; aqui os caminhos
  são `sudoku-game/Cargo.toml` e `deny.toml` na raiz.

- [ ] **Step 2: Adicionar** com `cargo add -p sudoku-game serde@<versão> --optional --no-default-features --features ...`
  e `cargo add -p sudoku-game --dev serde_json@<versão>`. Declare `[features] serde = ["dep:serde"]`.
  Revise o diff do `Cargo.lock` e rode `cargo deny check`.

- [ ] **Step 3: Testes que falham** em `sudoku-game/tests/save_json.rs`, com
  `#![cfg(feature = "serde")]` no topo:

```rust
#[test]
fn foto_vai_e_volta_em_json() {
    // partida Medio seed 7 com jogada, erro, anotação e dica; json = serde_json::to_string(&save)
    // Game::restore(&serde_json::from_str(&json).unwrap()) responde igual à original
}

#[test]
fn seed_acima_de_2_53_sobrevive_ao_json() {
    // Game::new(Facil, u64::MAX - 1, default); a JSON contém a seed entre aspas,
    // e a seed restaurada == puzzle().seed
}

#[test]
fn formato_da_foto_versao_1_nao_muda() {
    // Game::new(Facil, 42, default); resume(0); place da solução no 1º vazio (now 1_000);
    // toggle_note(2º vazio, 5); hint(2_000); save(3_000);
    // assert_eq!(serde_json::to_string_pretty(&saved).unwrap(), GOLDEN);
}
```

  `GOLDEN` é uma `const &str` com a JSON completa. Gere rodando o teste uma vez, confira à mão que
  os campos batem com a partida (versão 1, `"Facil"`, seed `"42"`, a dica em `hinted`), e cole.
  Um comentário acima explica que mudar este texto quebra partidas salvas: ou é bug, ou é versão
  nova com leitura da antiga.

- [ ] **Step 4: Rodar e ver falhar.** `cargo test -p sudoku-game --features serde --test save_json`:
  não compila (`SavedGame` sem `Serialize`). Confira também que `cargo test -p sudoku-game --test save_json`
  (sem a feature) compila e roda zero testes.

- [ ] **Step 5: Implementar** o `cfg_attr` no `SavedGame`.

- [ ] **Step 6: CI e comandos.** No `ci.yml`, clippy e testes com `--all-features` (mantenha também
  um `cargo test --workspace --locked` sem features, para provar que o crate compila sem `serde`).
  No `CLAUDE.md`, `CONTRIBUTING.md` e `README.md`, acrescente `--all-features` aos comandos de
  clippy e teste, com uma linha dizendo por quê.

- [ ] **Step 7: Rodar e ver passar.** `cargo test --workspace`, `cargo test --workspace --all-features`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo deny check`,
  `cargo fmt --all --check`.

- [ ] **Step 8: Commit** (corpo com o motivo das dependências, como pede a skill)

```bash
git add sudoku-game Cargo.lock .github CLAUDE.md CONTRIBUTING.md README.md deny.toml
git commit -m "feat: add optional serde feature for SavedGame"
```

- [ ] **Step 9: Wiki 43 e commit.** Assunto: features do Cargo (`optional`, `dep:`, `cfg_attr`),
  proc-macros e o que roda na compilação, `dev-dependencies`, `#![cfg(...)]` num arquivo de teste
  e golden test de formato.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on cargo features and serde"
```

---

### Task 5 (wiki 44): Reiniciar e fechamento da fatia

**Files:**
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/src/lib.rs`, `sudoku-game/tests/api.rs`
- Modify: `docs/architecture.md` (linha do módulo 2 e seção do Game), spec (status: fatia 2
  implementada), este plano (checkboxes)
- Create: `docs/wiki/44-reiniciar-e-non-exhaustive.md`; Modify: `docs/wiki/README.md`

**Interfaces:**
- Consumes: `config` guardada na `Game` (Task 2), `Game::new`.
- Produces:

```rust
/// O que reiniciar devolve. `#[non_exhaustive]` porque a fatia 3 acrescenta o resultado do
/// abandono, sem quebrar quem só lê `game`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Restart {
    pub game: Game,
}
```

  - `pub fn restart(&self, seed: u64) -> Restart`: `Game::new(self.puzzle.difficulty, seed, self.config)`.
    Vale em qualquer estado. A partida nova começa pausada.
  - Re-export de `Restart` em `lib.rs`.

- [ ] **Step 1: Testes que falham**

```rust
#[test]
fn reiniciar_comeca_outro_puzzle_do_mesmo_nivel_e_configuracao() {
    let config = GameConfig { lives: NonZeroU8::new(5).unwrap(), hints: 0 };
    let game = Game::new(Difficulty::Dificil, 1, config);
    let new_game = game.restart(2).game;
    assert_eq!(new_game.puzzle().difficulty, Difficulty::Dificil);
    assert_eq!((new_game.lives_left(), new_game.hints_left()), (5, 0));
    assert_ne!(new_game.puzzle().givens, game.puzzle().givens);
    assert!(new_game.is_paused());
    assert_eq!(new_game.elapsed_ms(10_000), 0);
}

#[test]
fn reiniciar_vale_depois_do_fim() { /* partida Lost → restart(9).game.status() == Playing */ }
```

  E em `tests/api.rs`, só com o que é público: `partida_sai_e_volta_sem_perder_nada` (joga,
  `save`, `restore`, `resume`, termina a partida restaurada até a vitória e confere o tempo total
  somando os dois trechos) e `reiniciar_pela_api_publica`.

- [ ] **Step 2: Rodar e ver falhar.** `cargo test -p sudoku-game`: não compila.

- [ ] **Step 3: Implementar** `Restart` e `restart`.

- [ ] **Step 4: Rodar e ver passar.** `cargo test --workspace` e com `--all-features`; fmt, clippy
  e `cargo deny check` limpos.

- [ ] **Step 5: Commit**

```bash
git add sudoku-game
git commit -m "feat: add restart with a new seed at the same level"
```

- [ ] **Step 6: Docs, wiki 44 e commit.** `architecture.md`: módulo 2 "fatia 2 concluída", tempo e
  salvamento descritos. Spec: status da fatia 2 implementada. Plano: checkboxes marcadas. Wiki 44:
  `#[non_exhaustive]` e evolução de API sem quebrar quem chama, e `&self` que devolve um valor
  novo em vez de mutar.

```bash
git add docs
git commit -m "docs: document Game slice 2 and add wiki page 44"
```
