# Frontend — O jogo no navegador Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Jogar uma partida inteira de Sudoku no navegador do PC (escolher nível, jogar, sair e
continuar, ver o histórico), com todas as regras vindas do `sudoku-game` compilado para WebAssembly.

**Architecture:** Um crate novo e fino, `sudoku-wasm`, embrulha a `Game` num tipo `WebGame`
exportado com `wasm-bindgen`. Tudo que cruza a ponte é texto: seeds em decimal, a foto e os
resultados nos formatos JSON que já existem (`SavedGame`, `SavedResult`) e um "retrato" da partida
(`GameView`) para a tela. O app em `web/` (Svelte 5 + TypeScript + Vite) desenha as telas, fornece o
relógio e o seed e guarda tudo no `localStorage`. A lógica do TypeScript que não depende do WASM
(navegação, armazenamento, formatação) fica em módulos puros testados com Vitest.

**Tech Stack:** Rust edition 2024, `wasm-bindgen`, `serde`/`serde_json`, `wasm-pack`; Node 24,
Svelte 5, TypeScript, Vite, Vitest, `svelte-check`.

**Spec:** [`docs/superpowers/specs/2026-11-09-web-design.md`](../specs/2026-11-09-web-design.md)

## Global Constraints

- `sudoku-core` e `sudoku-game` **não mudam**. Se a ponte precisar de algo que a API pública não
  dá, pare e discuta antes (spec, seção 8).
- Nenhuma regra do jogo no TypeScript: certo/errado, vidas, dicas e fim de partida vêm do WASM
  (spec, seção 5).
- Dependências novas passam pela skill `adding-dependencies` **antes** do `cargo add`, com
  aprovação explícita. Pacotes npm também são listados com o motivo e aprovados antes do
  `npm install`. `cargo deny check` continua passando.
- `sudoku-wasm` compila e testa no target nativo (`cargo test --workspace`) **e** em
  `wasm32-unknown-unknown`.
- Nomes no código em inglês (Rust e TypeScript); comentários e `///` em português, explicando o
  porquê. Nomes de teste em português, só a função de teste (Rust) ou a string do `it` (Vitest).
- Commits em inglês, Conventional Commits: por task, um `feat` com o código e o plano marcado, e um
  `docs` com a página da wiki. Branch: `feat/web-frontend`, a partir do `master`.
- Antes de cada commit: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace`, `cargo test --workspace --all-features`; e, a partir da Task 52, em
  `web/`: `npm run check`, `npm test`, `npm run build`, todos limpos.
- Disco `C:` com ~18 GB livres: confira `df -h /c` antes de instalar o target `wasm32`, o
  `wasm-pack` e o `node_modules`. Abaixo de 5 GB, pare e avise.
- Textos da tela em português, como no spec: `Jogar`, `Continuar`, `Histórico`, `Início`,
  `Anotar`, `Dica`, `Recomeçar`, `Fácil`, `Médio`, `Difícil`, `Muito Difícil`, `Derrota`,
  `Largada`, `Gerando...`.
- A cada 5 tasks, um consolidado em `docs/wiki/fundamentos/` (o próximo é o das tasks 51 a 55, na
  Task 55).

## Review Focus

1. **`localStorage` indisponível ou cheio** (aba anônima, cota estourada, `setItem` lançando
   exceção): o jogo continua jogável e nada trava; só não salva. Teste em `storage.test.ts`
   com um `Storage` falso que lança (Task 55, Step 4).
2. **Histórico com um registro corrompido no meio de registros bons**, ou o valor inteiro não
   sendo um array: os bons aparecem, o ruim some. Testes em `sudoku-wasm` para `history_entries`
   (Task 56, Steps 2 e 3) e em `storage.test.ts` para `appendResult` sobre lixo (Task 55, Step 3).
3. **Seed acima de 2^53 atravessando a ponte:** a seed `18446744073709551615` precisa voltar
   idêntica na foto. Teste em `sudoku-wasm` (Task 51, Step 3) e o `randomSeed` do TS devolvendo
   decimal dentro do `u64` (Task 52, Step 2).
4. **Fechar a aba logo depois de uma jogada:** a foto salva depois da jogada traz a jogada.
   Teste de ida e volta em `sudoku-wasm` (Task 55, Step 2); o app salva depois de **cada** ação,
   sem depender do `beforeunload` (Task 55, Step 6).
5. **`now` estranho vindo do JS** (o `performance.now()` recomeça do zero ao recarregar a página;
   `NaN`, negativo): a conversão satura em 0 e a partida restaurada volta pausada. Teste de
   `to_ms` em `sudoku-wasm` (Task 51, Step 2).

## Estrutura de arquivos

```
sudoku-wasm/
  Cargo.toml          cdylib + rlib; sudoku-game com feature serde, wasm-bindgen, serde, serde_json
  src/lib.rs          WebGame (#[wasm_bindgen]) e history_entries; doc do crate
  src/view.rs         GameView/CellView: o retrato serializável da partida
  src/convert.rs      to_ms, parse_difficulty, parse_seed, nomes de erro
  src/history.rs      history_entries: filtra, restaura e ordena o histórico
web/
  package.json        scripts build:wasm, dev, build, check, test
  vite.config.ts, tsconfig.json, svelte.config.js, index.html
  src/main.ts         carrega o WASM e monta o App
  src/App.svelte      troca de telas e histórico do navegador
  src/lib/types.ts    GameView, CellView, HistoryEntry, Level (espelho do JSON da ponte)
  src/lib/levels.ts   LEVELS: id do Rust → rótulo em português
  src/lib/clock.ts    formatClock, randomSeed, monotonicNow, wallClockNow
  src/lib/navigation.ts  Screen, NavEvent, nextScreen
  src/lib/storage.ts  partida guardada e histórico no localStorage
  src/lib/history.ts  formatEntry: a linha do histórico
  src/lib/*.test.ts   Vitest, ao lado de cada módulo
  src/screens/Home.svelte, Levels.svelte, GameScreen.svelte, History.svelte
  src/components/Board.svelte, NumberPad.svelte
```

`web/src/lib/wasm-pkg/` é gerado pelo `wasm-pack` e entra no `.gitignore`, junto com
`web/node_modules/` e `web/dist/`.

## A ponte (vale para todas as tasks)

Nomes em Rust com `snake_case`; o nome no JS vem do `js_name` indicado. Erros voltam como
`Err(String)`, que o JS recebe como exceção com o texto. O texto é o nome da variante
(`format!("{e:?}")` de `MoveError`/`RestoreError`) ou `"UnknownDifficulty"`/`"InvalidSeed"`.

```rust
#[wasm_bindgen] pub struct WebGame { game: Game }

#[wasm_bindgen] impl WebGame {
    #[wasm_bindgen(constructor)]
    pub fn new(difficulty: &str, seed: &str) -> Result<WebGame, String>;          // Task 51
    pub fn view(&self, now: f64) -> String;                                        // Task 51
    pub fn place(&mut self, index: usize, value: u8, now: f64) -> Result<String, String>; // Task 51: "correct" | "wrong"
    #[wasm_bindgen(js_name = toggleNote)]
    pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), String>;  // Task 53
    pub fn hint(&mut self, now: f64) -> Result<(), String>;                        // Task 53
    pub fn pause(&mut self, now: f64);                                             // Task 53
    pub fn resume(&mut self, now: f64);                                            // Task 51
    pub fn restart(&mut self, seed: &str, now: f64, finished_at_ms: f64) -> Result<Option<String>, String>; // Task 54
    pub fn save(&self, now: f64) -> String;                                        // Task 55
    pub fn restore(json: &str) -> Result<WebGame, String>;                         // Task 55, estático
    pub fn result(&self, finished_at_ms: f64) -> Option<String>;                   // Task 55
    pub fn abandon(self, now: f64, finished_at_ms: f64) -> Option<String>;         // Task 55
}
#[wasm_bindgen(js_name = historyEntries)]
pub fn history_entries(json: &str) -> String;                                      // Task 56
```

`view` devolve o `GameView` em JSON, com `#[serde(rename_all = "camelCase")]`:

```json
{ "difficulty": "Medio", "status": "playing", "paused": false, "elapsedMs": 222000,
  "livesLeft": 3, "hintsLeft": 2,
  "cells": [ { "kind": "given", "value": 5, "notes": [] },
             { "kind": "empty", "value": null, "notes": [3, 7] } ] }
```

`status` ∈ `playing | won | lost`; `kind` ∈ `empty | given | placed | hinted`; `cells` tem 81
entradas. `result`, `abandon` e `restart` devolvem um `SavedResult` em JSON; `save`/`restore` usam
o `SavedGame` em JSON. `history_entries` recebe o array do histórico e devolve
`[{ "difficulty", "outcome", "elapsedMs", "finishedAtMs" }]`, do mais recente para o mais antigo.

Testar `#[wasm_bindgen]` no nativo: os métodos acima não criam `JsValue` no corpo, então
`cargo test` chama `WebGame` direto. Se o macro impedir isso no nativo, mova o corpo para funções
Rust comuns em `src/session.rs` e deixe o `#[wasm_bindgen]` só delegando; a API JS não muda.

---

### Task 51: A ponte mínima: começar, ver e jogar

**Files:**
- Create: `sudoku-wasm/Cargo.toml`, `sudoku-wasm/src/lib.rs`, `sudoku-wasm/src/view.rs`,
  `sudoku-wasm/src/convert.rs`
- Modify: `Cargo.toml` (membro `sudoku-wasm`), `Cargo.lock`, `CLAUDE.md` (dependências do
  `sudoku-wasm` na seção Código), `CONTEXT.md` (`WebGame`, `GameView`)
- Wiki: `docs/wiki/51-ponte-wasm-bindgen.md`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: `Game::new`, `Game::cell`, `Game::notes`, `Game::lives_left`, `Game::hints_left`,
  `Game::status`, `Game::is_paused`, `Game::elapsed_ms`, `Game::place`, `Game::resume`.
- Produces: `WebGame::{new, view, place, resume}`; `convert::{to_ms(f64) -> u64,
  parse_difficulty(&str) -> Result<Difficulty, String>, parse_seed(&str) -> Result<u64, String>}`;
  `view::GameView::from_game(&Game, now: u64) -> GameView`.

- [x] **Step 0: Dependências.** Siga a skill `adding-dependencies` para `wasm-bindgen` (última
      versão), `serde` (mesma do `sudoku-game`, `derive`) e `serde_json`, no `sudoku-wasm`.
      Peça aprovação, depois `cargo add`. `rustup target add wasm32-unknown-unknown`.
      `cargo deny check` → Expected: ok.
- [x] **Step 1: Esqueleto que compila.** `crate-type = ["cdylib", "rlib"]`, `publish = false`,
      campos herdados do workspace, `#![warn(missing_docs)]`. `sudoku-game = { path, features =
      ["serde"] }`. Assinaturas da Task 51 com corpo `todo!()`; `to_ms` devolvendo `1`.
      `cargo build -p sudoku-wasm` e `cargo build -p sudoku-wasm --target wasm32-unknown-unknown`
      → Expected: compilam.
- [x] **Step 2: Ciclo — `now` estranho satura em zero**
      `fn now_negativo_ou_nan_vira_zero()`: `to_ms(-5.0) == 0`, `to_ms(f64::NAN) == 0`,
      `to_ms(1234.9) == 1234`. → `cargo test -p sudoku-wasm convert` → Expected: FAIL em
      `to_ms(-5.0) == 0` (left 1). → `now as u64` (o `as` de float satura e leva NaN a 0). → PASS.
- [x] **Step 3: Ciclo — seed acima de 2^53 atravessa sem perder precisão**
      `fn seed_maxima_atravessa_a_ponte()`: `parse_seed("18446744073709551615") == Ok(u64::MAX)`,
      `parse_seed("abc") == Err("InvalidSeed")`, `parse_seed("-1")` é `Err`. → FAIL na primeira
      asserção (`todo!` vira panic: troque o `todo!` por `Err(String::new())` antes de rodar, para
      o vermelho ser de asserção). → `str::parse::<u64>`. → PASS.
- [x] **Step 4: Ciclo — nível pelo nome do Rust**
      `fn nivel_pelo_nome()`: os quatro nomes `"Facil"`, `"Medio"`, `"Dificil"`, `"MuitoDificil"`
      dão as quatro variantes; `"facil"` dá `Err("UnknownDifficulty")`. Mesmo vermelho de asserção
      (corpo errado primeiro). → `match` exaustivo. → PASS.
- [x] **Step 5: Ciclo — retrato de uma partida nova**
      `fn retrato_de_partida_nova()`: `WebGame::new("Facil", "42")`, `view(0.0)` parseado com
      `serde_json::Value`: `status == "playing"`, `paused == true`, `livesLeft == 3`,
      `hintsLeft == 2`, `elapsedMs == 0`, `cells.len() == 81`, e as células `given` batem com
      `Game::new(Facil, 42, default).cell(i)`. → FAIL (comece `from_game` com `cells` vazio).
      → `GameView`/`CellView` com `Serialize`. → PASS.
- [x] **Step 6: Ciclo — jogada certa e errada pelo nome**
      `fn jogada_certa_e_errada()`: na partida da seed 42, depois de `resume(0.0)`, achar uma célula
      vazia `i` e o valor certo pela solução (`Game::new(...).puzzle().solution`):
      `place(i, certo, 10.0) == Ok("correct")`; outro índice vazio com valor errado dá
      `Ok("wrong")` e `livesLeft == 2` no `view`; `place(i, certo, 20.0)` de novo dá
      `Err("CellNotEmpty")`. → FAIL. → mínimo. → PASS.
- [x] **Step 7: Ciclo — pausada recusa jogada**
      `fn pausada_recusa_jogada()`: sem `resume`, `place(...) == Err("Paused")`. Se passar de
      primeira (o `MoveError` já cobre), prove: troque o mapeamento de erro por `"x"`, veja falhar,
      desfaça.
- [x] **Step 8: Refactor** com tudo verde. `cargo test --workspace`, `--all-features`, `clippy`,
      `fmt`, `cargo build -p sudoku-wasm --target wasm32-unknown-unknown`, `cargo deny check`.
      Atualize `CLAUDE.md` (dependências do `sudoku-wasm`: só `sudoku-game`, `wasm-bindgen`,
      `serde`, `serde_json`) e `CONTEXT.md`.
- [x] **Step 9: Commit do código**
      `git commit -m "feat: add the sudoku-wasm bridge to start and play a game"`
- [x] **Step 10: Wiki 51** (`# 51 — A ponte para o navegador: wasm-bindgen, cdylib e texto como
      contrato`): `cdylib` + `rlib`, o que o `#[wasm_bindgen]` gera, `Result<T, String>` virando
      exceção no JS, por que seeds viajam como texto (2^53, comparar com `Number.MAX_SAFE_INTEGER`
      e `BigInt`), o `as` que satura. Indexe no README.
      `git commit -m "docs: add wiki page 51 on the wasm-bindgen bridge"`

### Task 52: O tabuleiro no navegador

**Files:**
- Create: `web/` (scaffold do Vite `svelte-ts`), `web/src/lib/{types,levels,clock}.ts`,
  `web/src/lib/clock.test.ts`, `web/src/components/{Board,NumberPad}.svelte`,
  `web/src/screens/GameScreen.svelte`
- Modify: `.gitignore`, `README.md` (como rodar o app), `CLAUDE.md` (seção Comandos: `npm`)
- Wiki: `docs/wiki/52-wasm-no-navegador-com-vite-e-svelte.md`

**Interfaces:**
- Consumes: `WebGame::{new, view, place, resume}` (Task 51).
- Produces: `types.ts` com `GameView`, `CellView`, `Level = 'Facil' | 'Medio' | 'Dificil' |
  'MuitoDificil'`; `levels.ts` com `LEVELS: { id: Level; label: string }[]` na ordem do spec;
  `clock.ts` com `formatClock(ms: number): string`, `randomSeed(): string`,
  `monotonicNow(): number` (`performance.now()`), `wallClockNow(): number` (`Date.now()`);
  `GameScreen.svelte` com props `game: WebGame`, `onExit: () => void`, `onFinished: () => void`.

- [x] **Step 0: Ferramentas e pacotes.** `df -h /c`. Liste e peça aprovação de: `wasm-pack`
      (`cargo install wasm-pack`), o scaffold `npm create vite@latest web -- --template svelte-ts`
      e `vitest` como devDependency. Scripts no `package.json`: `build:wasm` =
      `wasm-pack build ../sudoku-wasm --target web --out-dir ../web/src/lib/wasm-pkg`,
      `dev` = `npm run build:wasm && vite`, `build` = `npm run build:wasm && vite build`,
      `check` = `svelte-check`, `test` = `vitest run`. `.gitignore` com `web/node_modules/`,
      `web/dist/`, `web/src/lib/wasm-pkg/`.
- [x] **Step 1: Esqueleto que compila.** `types.ts`, `levels.ts`, `clock.ts` com `formatClock`
      devolvendo `''` e `randomSeed` devolvendo `'0'`. `main.ts` faz `await init()` do pacote antes
      de montar o `App`. `npm run check` → Expected: 0 erros.
- [x] **Step 2: Ciclo — seed aleatória cabe num u64 em decimal**
      `it('seed aleatória é um u64 em decimal')`: 50 chamadas; cada uma casa `/^\d+$/` e
      `BigInt(s) <= 18446744073709551615n`; nem todas iguais. → `npm test -- clock` → FAIL em
      "nem todas iguais". → `crypto.getRandomValues(new BigUint64Array(1))[0].toString()`. → PASS.
- [x] **Step 3: Ciclo — relógio da barra**
      `it('formata o relógio da barra')`: `formatClock(0) == '00:00'`, `formatClock(222_000) ==
      '03:42'`, `formatClock(59_999) == '00:59'`, `formatClock(3_723_000) == '1:02:03'`.
      → FAIL em `'00:00'`. → mínimo. → PASS.
- [x] **Step 4: Tela do jogo jogável** (sem teste automático: componente visual, spec seção 8).
      `App.svelte` provisório: cria `new WebGame('Facil', randomSeed())`, chama
      `resume(monotonicNow())` e mostra `GameScreen`. `GameScreen` guarda `view` num `$state`,
      reparseado do `game.view(now)` depois de cada ação e a cada 1 s (relógio). Barra: rótulo do
      nível, `formatClock(view.elapsedMs)`, vidas. `Board`: 9×9, linhas grossas entre os boxes,
      célula selecionada destacada, `given` em outro peso. `NumberPad`: 1–9; com uma célula vazia
      selecionada chama `game.place`. `"wrong"` pisca a célula em vermelho por ~600 ms. Exceções
      da ponte (ex.: `CellNotEmpty`) são ignoradas na tela. Botões com pelo menos 44 px. Antes de
      gerar o puzzle, mostre `Gerando...` e ceda um quadro (`await tick()` +
      `requestAnimationFrame`), para a tela não congelar sem retorno.
- [x] **Step 5: Medir a geração no WASM.** `npm run build` + `npx vite preview`; no console, cronometre
      `new WebGame('MuitoDificil', seed)` em 20 seeds e anote mediana e
      pior caso na wiki 52. Se a mediana passar de 1 s, **pare** e traga os números para decidir o
      Web Worker (spec, seção 7) antes de seguir.
- [x] **Step 6: Verificação manual.** `npm run dev`; jogue até acertar e errar um número; perca as 3
      vidas numa partida e veja que o tabuleiro para de aceitar jogadas.
- [x] **Step 7: Refactor** com tudo verde. Comandos Rust + `npm run check`, `npm test`,
      `npm run build`. `CLAUDE.md` (Comandos) e `README.md` ganham `npm run dev` e `npm test` em
      `web/`.
- [x] **Step 8: Commit do código** `git commit -m "feat: play a game on the board in the browser"`
- [x] **Step 9: Wiki 52** (`# 52 — WASM no navegador: wasm-pack, init assíncrono e runes do
      Svelte`): o que o `wasm-pack --target web` gera, por que o `init()` é `await`, a memória do
      WASM (um `WebGame` é um ponteiro; `free()`), `$state` comparado a `useState`, os números da
      medição. `git commit -m "docs: add wiki page 52 on running wasm in the browser"`

### Task 53: Anotar, dica e pausa

**Files:**
- Modify: `sudoku-wasm/src/lib.rs`, `web/src/screens/GameScreen.svelte`,
  `web/src/components/{Board,NumberPad}.svelte`
- Wiki: `docs/wiki/53-anotacoes-dica-e-pausa.md`

**Interfaces:**
- Consumes: `Game::{toggle_note, hint, pause}`.
- Produces: `WebGame::{toggle_note (js toggleNote), hint, pause}`.

- [x] **Step 1: Esqueleto que compila.** As três assinaturas com corpo errado (`Ok(())` sem
      chamar a `Game`; `pause` vazio).
- [x] **Step 2: Ciclo — anotação aparece no retrato**
      `fn anotacao_aparece_no_retrato()`: célula vazia `i`, `toggle_note(i, 3)` e `(i, 7)`:
      `cells[i].notes == [3, 7]`; de novo `(i, 3)`: `[7]`. → FAIL em `[3, 7]` (veio `[]`). → PASS.
- [x] **Step 3: Ciclo — dica revela uma célula**
      `fn dica_revela_uma_celula()`: depois de `resume`, `hint(5.0)`: exatamente uma célula a mais
      com `kind == "hinted"` e `hintsLeft == 1`; com as dicas esgotadas, `Err("NoHintsLeft")`.
      → FAIL. → PASS.
- [x] **Step 4: Ciclo — pausa congela o tempo**
      `fn pausa_congela_o_tempo()`: `resume(0.0)`, `pause(1000.0)`: `view(5000.0)` tem
      `paused == true` e `elapsedMs == 1000`. → FAIL (`paused` false). → PASS.
- [x] **Step 5: Tela.** Botão `Anotar` alterna o modo (estado só da tela); no modo, o `NumberPad`
      chama `toggleNote`. Célula vazia mostra as notas numa grade 3×3 pequena. `Dica (n)` chama
      `hint`, desabilitado com `hintsLeft == 0`. Botão de pausa na barra alterna `pause`/`resume`;
      pausado, o tabuleiro é trocado por um aviso "Pausado" e as ferramentas ficam desabilitadas.
      `visibilitychange` para `hidden` chama `pause` (spec, seção 7).
- [x] **Step 6: Verificação manual.** Anote, peça as 2 dicas, pause, troque de aba e volte: o tempo
      não andou.
- [x] **Step 7: Refactor** com tudo verde (comandos Rust e `web/`).
- [x] **Step 8: Commit do código** `git commit -m "feat: add notes, hints and pause to the board"`
- [x] **Step 9: Wiki 53** (`# 53 — Anotações, dica e pausa: um bitset visto do JS, eventos da
      página e estado só da tela`): `Notes` virando `number[]` na ponte, o que é estado da partida
      (Rust) e o que é estado da tela (modo anotar), `visibilitychange` comparado ao
      `beforeunload`. `git commit -m "docs: add wiki page 53 on notes, hints and pause"`

### Task 54: Telas e navegação

**Files:**
- Create: `web/src/lib/navigation.ts`, `web/src/lib/navigation.test.ts`,
  `web/src/screens/{Home,Levels,History}.svelte`
- Modify: `sudoku-wasm/src/lib.rs` (`restart`), `web/src/App.svelte`,
  `web/src/screens/GameScreen.svelte`
- Wiki: `docs/wiki/54-telas-e-navegacao.md`

**Interfaces:**
- Consumes: `Game::restart`, `GameResult::save`, `LEVELS` (Task 52).
- Produces: `WebGame::restart`; `navigation.ts` com
  `type Screen = 'home' | 'levels' | 'game' | 'history'`,
  `type NavEvent = { type: 'play' } | { type: 'continue' } | { type: 'openHistory' } |
  { type: 'levelChosen' } | { type: 'back' } | { type: 'gameOver' } | { type: 'home' }`,
  `nextScreen(screen: Screen, event: NavEvent): Screen`.

- [x] **Step 1: Esqueleto que compila.** `nextScreen` devolvendo `screen`; `restart` devolvendo
      `Ok(None)` sem trocar a partida.
- [x] **Step 2: Ciclo — recomeçar troca a partida e devolve o abandono**
      `fn recomecar_devolve_o_abandono()`: partida da seed 42, `resume(0.0)`, `restart("7", 3000.0,
      1_700_000_000_000.0)`: o JSON devolvido tem `"outcome": "Abandoned"`, `"seed": "42"`,
      `"elapsed_ms": 3000`; o `view` depois mostra outro tabuleiro e `paused == true`. Partida
      terminada (perca as 3 vidas antes) devolve `Ok(None)`. → FAIL. → PASS.
- [x] **Step 3: Ciclo — caminho de ida**
      `it('vai do início ao jogo pelo nível')`: `home` + `play` → `levels`; `levels` +
      `levelChosen` → `game`; `home` + `continue` → `game`; `home` + `openHistory` → `history`.
      → `npm test -- navigation` → FAIL em `levels`. → PASS.
- [x] **Step 4: Ciclo — caminho de volta e fim**
      `it('volta e termina no histórico')`: `back` de `levels`, `game` e `history` → `home`;
      `game` + `gameOver` → `history`; `history` + `home` → `home`. Evento sem sentido na tela
      (`levels` + `gameOver`) mantém a tela. → FAIL. → PASS.
- [x] **Step 5: Telas.** `Home`: título, `Jogar`, `Histórico` (o `Continuar` entra na Task 55).
      `Levels`: os quatro de `LEVELS` e `← voltar`. `App.svelte` guarda `screen` num `$state`,
      aplica `nextScreen` e faz `history.pushState({ screen })` a cada troca; `popstate` aplica
      `back`. `GameScreen` ganha `← <nível>` na barra (chama `onExit`) e `Recomeçar`: pede
      `confirm('Largar esta partida e começar outra?')` e chama `restart(randomSeed(), ...)` (o
      abandono devolvido é guardado na Task 55; até lá, descartado). `History.svelte` provisório,
      só com o botão `Início`.
- [x] **Step 6: Verificação manual.** Navegue por todas as telas pelos botões e pelo voltar do
      navegador; o voltar nunca sai do site a partir do jogo.
- [x] **Step 7: Refactor** com tudo verde.
- [x] **Step 8: Commit do código** `git commit -m "feat: add home, level and game screens with navigation"`
- [x] **Step 9: Wiki 54** (`# 54 — Telas como máquina de estados: união discriminada no TS, enum no
      Rust e o histórico do navegador`): `NavEvent` comparado a um `enum` com dados do Rust,
      exaustividade (`never` no TS × `match`), `pushState`/`popstate`.
      `git commit -m "docs: add wiki page 54 on screens and navigation"`

### Task 55: Guardar e continuar

**Files:**
- Create: `web/src/lib/storage.ts`, `web/src/lib/storage.test.ts`
- Modify: `sudoku-wasm/src/lib.rs` (`save`, `restore`, `result`, `abandon`), `web/src/App.svelte`,
  `web/src/screens/{Home,GameScreen}.svelte`, `CONTEXT.md` se um tipo novo surgir
- Wiki: `docs/wiki/55-guardar-e-continuar.md`, `docs/wiki/fundamentos/rust-base-tarefas-51-a-55.md`

**Interfaces:**
- Consumes: `Game::{save, restore, result, abandon}`, `SavedGame`, `SavedResult` (serde).
- Produces: `WebGame::{save, restore, result, abandon}`; `storage.ts` com
  `SAVED_GAME_KEY = 'sudoku.savedGame'`, `HISTORY_KEY = 'sudoku.history'`,
  `loadSavedGame(s: Storage): string | null`, `storeSavedGame(s: Storage, json: string): void`,
  `clearSavedGame(s: Storage): void`, `loadHistory(s: Storage): string` (texto de um array JSON),
  `appendResult(s: Storage, resultJson: string): void`. `Storage` é a interface do DOM
  (`getItem`/`setItem`/`removeItem`); nenhuma função lança.

- [x] **Step 1: Esqueleto que compila.** Os quatro métodos com corpo errado (`save` devolve
      `"{}"`, `restore` devolve `Err`, `result`/`abandon` devolvem `None`); `storage.ts` com
      funções vazias e `loadHistory` devolvendo `''`.
- [x] **Step 2: Ciclo — foto depois da jogada traz a jogada**
      `fn foto_depois_da_jogada_traz_a_jogada()`: seed 42, `resume`, uma jogada certa e uma anotação,
      `save(50.0)`; `WebGame::restore(&json)`: o `view` tem a jogada, a anotação, as vidas e
      `paused == true`. Foto com `"version": 99` dá `Err("UnknownVersion(99)")`; `"lixo"` dá `Err`.
      → FAIL. → PASS.
- [x] **Step 3: Ciclo — histórico só cresce, mesmo sobre lixo**
      `it('acrescenta resultados ao histórico')`: com um `Storage` em memória, dois
      `appendResult` dão um array de 2 em `loadHistory`; com `HISTORY_KEY = 'não é json'` ou
      `'{"a":1}'`, `appendResult` dá um array só com o novo; vazio, `loadHistory == '[]'`.
      → FAIL. → PASS.
- [x] **Step 4: Ciclo — armazenamento quebrado não derruba o jogo**
      `it('ignora Storage que lança')`: um `Storage` cujos três métodos lançam; nenhuma função de
      `storage.ts` lança, `loadSavedGame` dá `null`, `loadHistory` dá `'[]'`. → FAIL. → `try/catch`.
      → PASS.
- [x] **Step 5: Ciclo — resultado só no fim, abandono só no meio**
      `fn resultado_e_abandono()`: partida em andamento: `result(..)` é `None`, `abandon(..)` traz
      `"Abandoned"`; partida perdida (3 erros): `result(..)` traz `"OutOfLives"` e `abandon(..)`
      é `None`. → FAIL. → PASS.
- [x] **Step 6: Ligar no app.**
      - `GameScreen` chama `storeSavedGame(localStorage, game.save(now))` depois de **cada**
        `place`, `toggleNote`, `hint`, `pause`/`resume` e `restart`, e ao sair pela seta.
      - Fim de partida (`status` vira `won`/`lost`): `appendResult(localStorage, game.result(wallClockNow()))`,
        `clearSavedGame`, e `onFinished` (que dispara `gameOver`), nessa ordem.
      - `Recomeçar`: o abandono devolvido pelo `restart` vai para o `appendResult`.
      - `Home` mostra `Continuar` quando `loadSavedGame` não é `null`; `continue` faz
        `WebGame.restore`; se lançar, `clearSavedGame` e o botão some, sem entrar no histórico.
      - `Jogar` → nível com partida guardada: `confirm('Largar a partida guardada?')`; sim: restaura,
        `abandon` → `appendResult` → `clearSavedGame`, e começa a nova; não: fica na tela de nível.
      - Partida nova ou continuada abre pausada; o primeiro toque no tabuleiro ou no botão de pausa
        chama `resume`.
- [x] **Step 7: Verificação manual.** Jogue, feche a aba logo depois de uma jogada, abra de novo:
      `Continuar` traz a jogada e o tempo de antes. Em `localStorage.setItem('sudoku.savedGame',
      'x')`, recarregue: o `Continuar` some e nada quebra.
- [x] **Step 8: Refactor** com tudo verde.
- [x] **Step 9: Commit do código** `git commit -m "feat: save the game in progress and continue it later"`
- [x] **Step 10: Wiki 55 e consolidado.** `# 55 — Guardar e continuar: serde_json na ponte, `self`
      que consome visto do JS e try/catch no armazenamento` (o `abandon(self)` liberando o objeto no
      JS, `Option<String>` virando `string | undefined`, `localStorage` síncrono). E o consolidado
      `fundamentos/rust-base-tarefas-51-a-55.md`, no formato dos anteriores.
      `git commit -m "docs: add wiki page 55 and consolidate tasks 51 to 55"`

### Task 56: Histórico

**Files:**
- Create: `sudoku-wasm/src/history.rs`, `web/src/lib/history.ts`, `web/src/lib/history.test.ts`
- Modify: `sudoku-wasm/src/lib.rs`, `web/src/screens/{History,Home}.svelte`, `web/src/App.svelte`
- Wiki: `docs/wiki/56-historico.md`

**Interfaces:**
- Consumes: `GameResult::restore`, `SavedResult` (serde), `loadHistory` (Task 55), `LEVELS`.
- Produces: `history_entries(json: &str) -> String` (js `historyEntries`); `types.ts` ganha
  `HistoryEntry = { difficulty: Level; outcome: 'Won' | 'OutOfLives' | 'Abandoned'; elapsedMs:
  number; finishedAtMs: number }`; `history.ts` com
  `formatEntry(entry: HistoryEntry, timeZone?: string): { text: string; label: string | null }`.

- [x] **Step 1: Esqueleto que compila.** `history_entries` devolvendo `"[]"`; `formatEntry`
      devolvendo `{ text: '', label: null }`.
- [x] **Step 2: Ciclo — histórico ordenado, mais recente primeiro**
      `fn historico_mais_recente_primeiro()`: array com dois `SavedResult` válidos (datas `1000` e
      `2000`): saída com 2 entradas, a de `finishedAtMs == 2000` primeiro, `difficulty`, `outcome`
      e `elapsedMs` copiados. → FAIL (veio `[]`). → cada elemento parseado como `serde_json::Value`
      e depois `SavedResult`, `GameResult::restore`, ordenação estável decrescente. → PASS.
- [x] **Step 3: Ciclo — registro ruim some, os bons ficam**
      `fn registro_ruim_some()`: o array do Step 2 com `{"version": 99, ...}`, `"lixo"` e `42` no
      meio: continuam as 2 entradas boas. Entrada `"não é json"` ou `"{}"` inteira dá `[]`.
      → FAIL ou passa de primeira; se passar, prove quebrando o filtro. → PASS.
- [x] **Step 4: Ciclo — linha do histórico**
      `it('formata a linha do histórico')`, com `timeZone = 'America/Sao_Paulo'`:
      `{ Medio, Won, 600_000, Date.UTC(2026, 5, 13, 15) }` → `text == 'Médio — 10 min — 13/06/2026'`,
      `label == null`; `OutOfLives` → `label == 'Derrota'`; `Abandoned` → `label == 'Largada'`;
      `elapsedMs = 59_000` → `'< 1 min'`; `90_000` → `'2 min'` (arredonda). → `npm test -- history`
      → FAIL. → `Math.round`, `Intl.DateTimeFormat('pt-BR', { timeZone })`. → PASS.
- [x] **Step 5: Tela.** `History.svelte` lê `historyEntries(loadHistory(localStorage))`, mostra uma
      linha por entrada com o rótulo ao lado quando houver, "Nenhuma partida ainda." quando vazio,
      e o botão `Início` no fim. Chega pelo `Histórico` da `Home` e pelo fim da partida.
- [x] **Step 6: Verificação manual.** Termine uma partida ganhando (dicas ajudam) e outra perdendo,
      recomece uma terceira: três linhas, com `Derrota` e `Largada` nas certas.
- [x] **Step 7: Refactor** com tudo verde.
- [x] **Step 8: Commit do código** `git commit -m "feat: show the game history"`
- [x] **Step 9: Wiki 56** (`# 56 — Histórico: filtrar o que vem de fora, `filter_map` e
      `Intl` no lugar de formatar à mão`). `git commit -m "docs: add wiki page 56 on the game history"`

### Task 57: CI do frontend e documentação da fase

**Files:**
- Modify: `.github/workflows/ci.yml` (job `web`), `docs/architecture.md` (seções 2, 4, 8, 9 e 11:
  WASM como ponte, Tauri como empacotamento, módulo 3 concluído, artefatos de processo),
  `docs/wiki/15-ci-cd-com-github-actions.md` só se precisar de nota, `CHANGELOG.md`
- Wiki: `docs/wiki/57-ci-do-frontend.md`

**Interfaces:**
- Consumes: scripts `build:wasm`, `check`, `test`, `build` (Task 52).
- Produces: job `web` no CI.

- [ ] **Step 1: Job `web`** em `ubuntu-latest`: `actions/checkout@v7`, `dtolnay/rust-toolchain@stable`
      com `targets: wasm32-unknown-unknown`, `Swatinem/rust-cache@v2`,
      `taiki-e/install-action@v2` com `tool: wasm-pack`, `actions/setup-node` com `node-version: 24`
      e cache npm de `web/package-lock.json`; em `web/`: `npm ci`, `npm run build:wasm`,
      `npm run check`, `npm test`, `npm run build`. Comentários em português, como o resto do
      arquivo.
- [ ] **Step 2: Prova do vermelho.** Numa branch local, quebre de propósito um teste do Vitest e
      um erro de tipo; rode os mesmos comandos do job localmente e veja falhar; desfaça. (O CI de
      verdade roda no PR.)
- [ ] **Step 3: Documentação da fase.** `architecture.md` e `CHANGELOG.md` atualizados; spec com
      status "implementada".
- [ ] **Step 4: Refactor e verificação final** com tudo verde: comandos Rust, `cargo deny check`,
      comandos de `web/`, `cargo build -p sudoku-wasm --target wasm32-unknown-unknown`.
- [ ] **Step 5: Commit** `git commit -m "chore: build and test the web frontend in CI"`
      (e `docs: record the browser frontend in the architecture`, se separar ficar mais claro).
- [ ] **Step 6: Wiki 57** (`# 57 — CI do frontend: um job para o WASM, npm ci e o lockfile como
      contrato`). `git commit -m "docs: add wiki page 57 on the frontend CI"`
