# Regras de anotação Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Duas regras novas para as anotações (o "lápis"):

1. **Limpeza automática.** Quando um número certo entra numa célula (jogada certa ou dica), a
   anotação desse número some de todas as células da mesma linha, coluna e caixa.
2. **Anotação em conflito é recusada.** Anotar um número que já está na linha, na coluna ou na
   caixa da célula não entra, e os números que causam o conflito piscam no tabuleiro. Apagar uma
   anotação que já existe continua sempre permitido.

**Architecture:** As duas regras são regras do jogo, então moram na `Game` do `sudoku-game` (o
PvP vai precisar delas no servidor). A regra 1 muda o `lock`; a regra 2 ganha uma consulta
pública `Game::conflicts(index, value)` e uma recusa nova, `MoveError::NoteConflict`. O
`sudoku-wasm` expõe a consulta, e a tela só pisca as células que o WASM devolve.

**Tech Stack:** Rust edition 2024, `wasm-bindgen`, Svelte 5, TypeScript.

**Origem:** pedido em conversa (2026-11-11). Decisões: anotação em conflito é recusada (não entra
com aviso); a dica também limpa as anotações dos vizinhos, porque também é um número certo. Isso
resolve o "limpeza automática de anotações" que estava em aberto na `architecture.md`, seção 10.

## Global Constraints

- Nenhuma dependência nova.
- O `sudoku-core` não muda. Os helpers de linha, coluna e caixa dele são privados; o
  `sudoku-game` tem o próprio, pequeno.
- Nenhuma regra no TypeScript: quem diz se há conflito, e onde, é o WASM.
- Fotos antigas com anotações em conflito continuam sendo restauradas (só a ação nova recusa).
- Wiki: o `master` vai até a página 58; as 59 e 60 estão na branch `feat/jungle-scene`. As
  páginas deste plano são a **61** e a **62**.

---

### Task 61: Número certo limpa as anotações dos vizinhos

**Files:**
- Modify: `sudoku-game/src/game.rs`, `docs/architecture.md` (seção 10), `CONTEXT.md` se o
  glossário falar de anotação
- Wiki: `docs/wiki/61-limpar-anotacoes-dos-vizinhos.md`

**Interfaces:**
- Produces: nenhuma API nova. `place` (certo) e `hint` passam a apagar a anotação `value` das
  células que "enxergam" `index` (mesma linha, coluna ou caixa).

- [x] **Step 1: Esqueleto que compila.** `fn sees(a: usize, b: usize) -> bool` privado em
      `game.rs`, devolvendo `false`.
- [x] **Step 2: Ciclo — jogada certa limpa a anotação nos vizinhos**
      Renomeia e reescreve `numero_certo_limpa_so_as_anotacoes_da_propria_celula` para
      `fn numero_certo_limpa_a_anotacao_dos_vizinhos()`: anota o valor certo `v` de uma célula
      vazia `a` em `a`, num vizinho vazio de linha, num de coluna e num de caixa, e numa célula
      vazia longe (que não enxerga `a`) onde `v` também não conflita; coloca `v` em `a`.
      → `cargo test -p sudoku-game anotac` → Expected: FAIL na asserção de que o vizinho de linha
      ficou sem `v`. → `lock` percorre `0..81` e desliga `value` onde `sees(index, i)`; `sees`
      compara linha, coluna e caixa. → Expected: PASS.
- [x] **Step 3: Ciclo — outros números anotados nos vizinhos ficam**
      `fn limpeza_so_apaga_o_numero_colocado()`: vizinho com `v` e outro número `u`; depois da
      jogada certa, o vizinho fica só com `u`. → passa de primeira; prova: troque o desligar do
      bit por `Notes::default()`, veja FAIL, desfaça. → PASS.
- [x] **Step 4: Ciclo — dica também limpa os vizinhos**
      `fn dica_limpa_a_anotacao_dos_vizinhos()`: seed 42 do Fácil, a dica é `index 40, value 9`
      (golden). Anota 9 num vizinho vazio de 40 antes da dica; depois, o vizinho não tem 9.
      → passa de primeira (a dica usa `lock`); prova: tire a limpeza do `lock` e mova para o
      `place`, veja FAIL, desfaça. → PASS.
- [x] **Step 5: Refactor** com tudo verde. Atualize os doc comments de `lock`, `place`, `hint` e
      do `CellState`/`Notes` que falem "só da própria célula". `cargo test --workspace`,
      `clippy`, `fmt`.
- [x] **Step 6: Commit do código**
      `git commit -m "feat: clear a placed number from its neighbours' notes"`
- [x] **Step 7: Wiki 61** (`# 61 — Limpar anotações dos vizinhos: empréstimo de campos
      disjuntos, iterar índices e uma regra que muda de ideia`), índice no `README`, seção 10 da
      `architecture.md`. `git commit -m "docs: add wiki page 61 on clearing neighbour notes"`

### Task 62: Anotação em conflito pisca e não entra

**Files:**
- Modify: `sudoku-game/src/game.rs`, `sudoku-game/tests/api.rs` (se precisar), `sudoku-wasm/src/lib.rs`,
  `web/src/screens/GameScreen.svelte`, `web/src/components/Board.svelte`, `CONTEXT.md`
- Wiki: `docs/wiki/62-anotacao-em-conflito.md`

**Interfaces:**
- Produces: `MoveError::NoteConflict`; `Game::conflicts(&self, index: usize, value: u8) ->
  Vec<usize>` (células com número `value` que enxergam `index`, em ordem crescente; pânico fora
  de 0..81, como `cell`); no WASM, `conflicts(index, value) -> String` com o array JSON
  (tudo que cruza a ponte é texto); a tela ganha `conflicting: number[]` no `Board`.

- [x] **Step 1: Esqueleto que compila.** `NoteConflict` na `MoveError`; `conflicts` devolvendo
      `Vec::new()`; `WebGame::conflicts` devolvendo `"[]"`.
- [x] **Step 2: Ciclo — conflicts acha os números da linha, coluna e caixa**
      `fn conflicts_acha_o_numero_nos_vizinhos()`: numa célula vazia `i`, para um número `v`
      que aparece nos vizinhos, o resultado é exatamente o conjunto das células com `v` que
      enxergam `i` (conferido varrendo o tabuleiro no teste). Para o valor certo de `i`, vazio.
      → FAIL (veio vazio). → filtra `0..81` com `sees` e `cell(j).value() == Some(value)`. → PASS.
- [x] **Step 3: Ciclo — anotar número em conflito é recusado**
      `fn anotacao_em_conflito_eh_recusada()`: `toggle_note(i, v)` com `v` em conflito →
      `Err(MoveError::NoteConflict)` e `notes(i)` vazio. → FAIL (veio `Ok`). → no `toggle_note`,
      se o bit está desligado e `conflicts` não é vazio, recusa. → PASS. Ajuste os testes antigos
      que anotavam números fixos (3, 7, 1...) para usar números sem conflito.
- [x] **Step 4: Ciclo — apagar anotação nunca é recusado**
      `fn apagar_anotacao_em_conflito_eh_permitido()`: foto com a anotação `v` (em conflito) na
      célula `i`, restaurada; `toggle_note(i, v)` → `Ok`, anotação apagada. → passa de primeira;
      prova: tire a condição "bit desligado", veja FAIL, desfaça. → PASS.
- [x] **Step 5: Ciclo — a ponte**
      `fn conflitos_atravessam_a_ponte()` no `sudoku-wasm`: `toggle_note` em conflito volta
      `Err("NoteConflict")` e `conflicts(i, v)` volta o JSON com as células. → FAIL (`"[]"`).
      → `serde_json::to_string(&self.game.conflicts(..))`. → PASS.
- [x] **Step 6: Tela.** No modo anotar, `NoteConflict` → `conflicting = JSON.parse(game.conflicts(i, v))`
      por 900 ms; o `Board` pisca essas células (animação CSS, `prefers-reduced-motion` só muda o
      fundo). Verificação manual no `npm run dev`: anotar um número da linha pisca e não entra;
      apagar continua funcionando; colocar um número certo some com a anotação dele na linha.
- [x] **Step 7: Refactor** com tudo verde. `cargo test --workspace --all-features`, `clippy`,
      `fmt`, `npm run check`, `npm test`, `npm run build`.
- [x] **Step 8: Commit do código**
      `git commit -m "feat: refuse a note that clashes with its row, column or box"`
- [x] **Step 9: Wiki 62** (`# 62 — Anotação em conflito: variante nova num enum público, consulta
      separada da ação e animação que respeita o sistema`), índice no `README`.
      `git commit -m "docs: add wiki page 62 on conflicting notes"`
