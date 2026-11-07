# Refactor — A foto mora no module de salvamento Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Juntar escrita, leitura e conferência da foto (`SavedGame`) num module filho da partida,
`game/save.rs`, e tirar os nomes de nível do module da foto, sem mudar comportamento nem interface.

**Architecture:** Primeiro um module `pub(crate)` `names.rs` recebe `difficulty_name` e
`parse_difficulty`, usados pela foto e pelo resultado. Depois `save.rs` vira
`game/save.rs`, filho de `game`, e leva junto o bloco `impl Game { save, restore }`, o
`parse_notes` e os testes da foto. Como filho, ele enxerga os campos privados da `Game` sem que
nenhum mude de visibilidade.

**Tech Stack:** Rust edition 2024, workspace existente. Nenhuma dependência nova.

**Spec:** [`docs/superpowers/specs/2026-11-07-save-module-design.md`](../specs/2026-11-07-save-module-design.md)

## Global Constraints

- Refactor puro: **nenhuma asserção de teste muda**, e a contagem de testes do workspace é a
  mesma antes e depois, com e sem `--all-features`.
- A interface pública do `sudoku-game` não muda: mesmos itens no `lib.rs`, mesmas assinaturas.
  `sudoku-game/tests/*.rs` passam **sem edição**.
- Foto **versão 1** intacta: `SAVE_VERSION = 1`, e o golden test de `tests/save_json.rs` passa sem
  edição.
- `sudoku-core` não muda. Nenhuma dependência nova.
- Nenhum campo da `Game` muda de visibilidade.
- Nomes no código em inglês; comentários e `///` em português, explicando o porquê.
- Commits em inglês, Conventional Commits: um `refactor` com o código e um `docs` com a wiki.
- Antes de cada commit: `cargo fmt --all`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace` e `cargo test --workspace --all-features` limpos, `cargo build --workspace` sem warnings.

## Como um refactor segue o formato de TDD

Um refactor não tem vermelho de asserção próprio: os testes já existem e passam. No lugar do
ciclo vermelho → verde, cada task usa a prova da spec (seção 4): contagem de testes igual antes
e depois, e um teste movido que ainda pega erro quando o código é quebrado de propósito.

## Review Focus

1. **Um teste de foto esquecido na `game.rs`, ou duplicado nos dois lugares:** a contagem de
   testes denuncia (Task 50, Step 5); os 24 testes que se mudam estão listados por nome.
2. **Um campo ou método da `Game` aberto além do necessário para o filho enxergar:** nada de
   `pub(crate)` em campo; o filho já vê o que é privado (Task 50, Step 3).
3. **`SavedGame`, `RestoreError` ou `SAVE_VERSION` sumindo da API pública** quando o module
   vira privado dentro de `game`: `tests/api.rs` e `tests/save_json.rs` compilam sem edição
   (Task 50, Step 5).
4. **O vazamento `result → save` voltando como `result → game::save`:** depois da Task 49,
   `result.rs` não importa nada de `save` (Task 49, Step 4).
5. **A documentação apontando para o caminho velho:** `architecture.md` 4.1 cita `names.rs`
   (Task 49) e `game/save.rs`, e o `CONTEXT.md` também cita `game/save.rs` (Task 50, Step 8).

---

### Task 49: Nomes salvos num module próprio

**Files:**
- Create: `sudoku-game/src/names.rs`
- Modify: `sudoku-game/src/lib.rs` (`mod names;`), `sudoku-game/src/save.rs` (sai
  `difficulty_name`, `parse_difficulty` e o teste deles), `sudoku-game/src/result.rs` e
  `sudoku-game/src/game.rs` (imports)
- Test: `sudoku-game/src/names.rs` (o teste `nome_do_nivel_vai_e_volta`, movido)
- Docs: `docs/wiki/49-modules-de-dentro-do-crate.md`, `docs/wiki/README.md`,
  `docs/architecture.md` (seção 4.1)

**Interfaces:**
- Consumes: nada de task anterior.
- Produces: `crate::names::difficulty_name(Difficulty) -> &'static str` e
  `crate::names::parse_difficulty(&str) -> Option<Difficulty>`, ambos `pub(crate)`, com o mesmo
  corpo de hoje. A Task 50 importa deles.

- [x] **Step 1: Linha de base**

Run: `cargo test --workspace 2>&1 | grep "test result"` e
`cargo test --workspace --all-features 2>&1 | grep "test result"`
Expected: todas `ok`. Anotar as contagens aqui, logo abaixo deste passo, no próprio plano: são a
referência da Task 49 e da Task 50, mesmo que a sessão caia no meio.

Linha de base (2026-11-07), uma contagem por `test result`, na ordem do Cargo: sem features,
`123 · 4 · 0 (3 ignorados) · 83 · 6 · 0 · 0 · 0 · 0`; com `--all-features`,
`123 · 4 · 0 (3 ignorados) · 83 · 6 · 1 · 3 · 0 · 0`.

- [x] **Step 2: Mover os nomes**

`names.rs` recebe as duas funções, com o `///` atual, e o module doc diz por que existe: o nome
do nível é o mesmo na foto e no resultado salvo, e não pertence a nenhum dos dois. O teste
`nome_do_nivel_vai_e_volta` vai junto, sem mudança. `lib.rs` declara `mod names;` (privado).
`save.rs`, `result.rs` e `game.rs` importam de `crate::names`.

- [x] **Step 3: Provar que o teste movido pega erro**

Trocar `"Medio"` por `"Medio2"` em `parse_difficulty`.
Run: `cargo test -p sudoku-game nome_do_nivel`
Expected: FAIL em `assert_eq!(parse_difficulty(difficulty_name(d)), Some(d))` para `Medio`.
Desfazer e rodar de novo: PASS.

- [x] **Step 4: Refactor e verificação**

Run: `grep -n "save" sudoku-game/src/result.rs` → Expected: nenhum `use crate::save`.
Run: as duas linhas do Step 1 → Expected: mesmas contagens.
Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --all`,
`cargo build --workspace` → limpos.

- [x] **Step 5: Commit do código**

```bash
git add sudoku-game/src docs/superpowers/plans/2026-11-07-save-module.md
git commit -m "refactor: move saved level names into their own module"
```

- [ ] **Step 6: Wiki da task e commit `docs`**

Página `49-modules-de-dentro-do-crate.md`, cabeçalho
`# 49 — Nomes salvos: um module de dentro do crate, `pub(crate)` e dependência entre irmãos`
e ``> Referente à Task 49 do plano do refactor do salvamento. Commit: `refactor: move saved level names into their own module`.``
Seções: `mod` privado declarado no `lib.rs` e o caminho `crate::names`; `pub(crate)` (comparar
com um arquivo TS que exporta, mas o pacote não reexporta no `index.ts`); por que um module
irmão não deveria importar de outro só por conveniência (o vazamento `result → save`).
Indexar no `docs/wiki/README.md`. Em `docs/architecture.md` 4.1, a árvore ganha
`names.rs  difficulty_name e parse_difficulty: o nome do nível na foto e no resultado salvo`.

```bash
git add docs
git commit -m "docs: add wiki page 49 on crate-private modules"
```

---

### Task 50: A foto vira module filho da partida

**Files:**
- Move: `sudoku-game/src/save.rs` → `sudoku-game/src/game/save.rs` (`git mv`)
- Modify: `sudoku-game/src/game.rs` (declara `mod save;` e reexporta; saem `save`, `restore`,
  `parse_notes`, os testes da foto e os helpers `played_game` e `valid_save`),
  `sudoku-game/src/lib.rs` (sai `mod save;`, os itens da foto entram no `pub use game::{...}`)
- Test: `sudoku-game/src/game/save.rs` (`mod tests` com os 24 testes movidos)
- Docs: `docs/wiki/50-module-filho-e-privacidade.md`,
  `docs/wiki/fundamentos/rust-base-tarefas-46-a-50.md`, `docs/wiki/README.md`, `CONTEXT.md`,
  `docs/architecture.md` (seção 4.1), spec do refactor (status)

**Interfaces:**
- Consumes: `crate::names::{difficulty_name, parse_difficulty}` (Task 49).
- Produces: nenhuma interface nova. A API pública fica igual: `Game::save(&self, now: u64) -> SavedGame`
  e `Game::restore(saved: &SavedGame) -> Result<Game, RestoreError>`, agora definidos em
  `game/save.rs`; `sudoku_game::{SavedGame, RestoreError, SAVE_VERSION}` continuam exportados.

- [ ] **Step 1: Linha de base**

As duas linhas do Step 1 da Task 49 → Expected: as mesmas contagens de lá.

- [ ] **Step 2: Mover o arquivo e religar os modules**

`git mv sudoku-game/src/save.rs sudoku-game/src/game/save.rs`. Em `game.rs`: `mod save;` e
`pub use save::{RestoreError, SAVE_VERSION, SavedGame};`. O module fica privado, e só os três
itens saem. Em `lib.rs`: sai `mod save;`, e os três entram no `pub use game::{...}`. Em
`game/save.rs`: `use super::{CellState, Game, PuzzleError};`, mais o que o código movido precisar.
Run: `cargo build --workspace` → compila.

- [ ] **Step 3: Mover o código da foto**

Para `game/save.rs`, sem mudar uma linha do corpo: o bloco `impl Game { pub fn save(&self, now: u64) -> SavedGame; pub fn restore(saved: &SavedGame) -> Result<Game, RestoreError> }`
com os `///` atuais, e a função livre `parse_notes`. Na `game.rs` não fica nada da foto, e o
comentário do `struct Game` sobre `config` continua valendo. Nenhum campo nem método da `Game`
ganha visibilidade: `from_puzzle` já é `pub`, e `solution_value` é privado em `game`, visível para o
filho. O doc do module `save.rs` ganha uma frase sobre por que ele é filho de `game`.

- [ ] **Step 4: Mover os testes da foto**

Para o `mod tests` de `game/save.rs` vão os helpers `played_game` e `valid_save` e estes 24
testes, sem mudança de corpo:
`restaurar_devolve_a_mesma_partida_pausada`, `salvar_rodando_inclui_o_trecho_atual_e_nao_pausa`,
`dica_depois_de_restaurar_cai_na_mesma_celula`, `partida_terminada_restaura_terminada`,
`foto_guarda_seed_e_nivel_em_texto`, `foto_guarda_a_configuracao`,
`versao_nivel_ou_seed_desconhecidos_sao_recusados`, `foto_valida_restaura_sem_erro`,
`numero_errado_travado_eh_recusado`, `numero_dado_trocado_ou_apagado_eh_recusado`,
`dica_fora_do_tabuleiro_eh_recusada`, `dica_em_celula_vazia_ou_dada_eh_recusada`,
`dica_repetida_eh_recusada`, `dicas_que_nao_somam_sao_recusadas`,
`anotacoes_com_tamanho_errado_sao_recusadas`, `anotacao_com_digito_invalido_eh_recusada`,
`anotacao_repetida_ou_fora_de_ordem_eh_recusada`, `anotacao_em_celula_preenchida_eh_recusada`,
`vida_recuperada_eh_recusada`, `erros_alem_das_vidas_sao_recusados`,
`configuracao_sem_vidas_eh_recusada`, `derrota_com_tabuleiro_completo_eh_recusada`,
`puzzle_invalido_na_foto_eh_recusado`, `tempo_enorme_na_foto_nao_estoura`.

Os helpers que os dois blocos de testes usam (`started`, `easy_game`, `solution_value`,
`first_empty` e qualquer outro que o compilador pedir) ficam no `mod tests` da `game.rs` como
`pub(super)`, e o de `save.rs` os importa com `use crate::game::tests::{...}`. Helper que só a
foto usa vai junto com ela.

- [ ] **Step 5: Provar que nada sumiu e que os testes movidos pegam erro**

Run: as duas linhas do Step 1 → Expected: **as mesmas contagens**, inclusive a linha de
`tests/save_json.rs` com `--all-features`.
Run: `grep -c "#\[test\]" sudoku-game/src/game.rs sudoku-game/src/game/save.rs` → Expected:
`game.rs` 44 e `save.rs` 24 (hoje `game.rs` tem 68).
Quebrar de propósito: em `restore`, trocar `i <= p` por `i < p` na conferência das dicas.
Run: `cargo test -p sudoku-game dica_repetida` → Expected: FAIL em `dica_repetida_eh_recusada`
(o `restore` passa a aceitar a dica repetida). Desfazer → PASS.

- [ ] **Step 6: Refactor e verificação**

`use` sobrando na `game.rs` (`NonZeroU8`, `Grid`...) sai. `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`cargo fmt --all`, `cargo build --workspace` → limpos. `git diff --stat master -- sudoku-game/tests`
→ vazio.

- [ ] **Step 7: Commit do código**

```bash
git add sudoku-game/src docs/superpowers/plans/2026-11-07-save-module.md
git commit -m "refactor: move the saved game codec into a child module of game"
```

- [ ] **Step 8: Wiki da task, consolidado e commit `docs`**

Página `50-module-filho-e-privacidade.md`, cabeçalho
`# 50 — A foto como module filho: privacidade por árvore, `impl` em outro arquivo e `git mv``
e ``> Referente à Task 50 do plano do refactor do salvamento. Commit: `refactor: move the saved game codec into a child module of game`.``
Seções: privado em Rust é "privado para a árvore", e o filho vê os campos do pai (comparar com
`#private` em classe JS, que nem um módulo vizinho enxerga); um tipo com blocos `impl` em vários
arquivos (em TS, a classe fica num arquivo só); `game.rs` + pasta `game/`, sem `mod.rs` (edition
2018+); `pub use` de um module privado para manter a API; `pub(super)` nos helpers de teste.

Consolidado `docs/wiki/fundamentos/rust-base-tarefas-46-a-50.md`, no formato do
`rust-base-tarefas-41-a-45.md`: por conceito, das tasks 46 a 50, com link para o bloco anterior.

`CONTEXT.md`: o `SavedGame` aponta para `sudoku-game/src/game/save.rs`. `docs/architecture.md`
4.1: `save.rs` sai da raiz da árvore e aparece como `game/save.rs` (SavedGame, RestoreError,
SAVE_VERSION, `Game::save` e `Game::restore`); a linha da `game.rs` deixa de citar `save` e
`restore`. Spec do refactor: status "implementada". Indexar as duas páginas no `docs/wiki/README.md`.

```bash
git add docs CONTEXT.md
git commit -m "docs: add wiki page 50 and consolidate tasks 46 to 50"
```
