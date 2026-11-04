# Engine — Fatia de Nível Muito Difícil Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer o `sudoku-core` gerar puzzles Muito Difíceis (resolvíveis só por lógica, com no máximo 22 números), sem alterar nenhum puzzle Fácil, Médio ou Difícil.

**Architecture:** Duas técnicas novas de eliminação, Swordfish e XY-Wing, com nível `MuitoDificil`, entram **no fim** da lista do solver. O gerador ganha um ramo próprio para o Muito Difícil, que cava como os outros e, quando o puzzle não atende o aceite, **refina** (devolve K números da solução e recava) em vez de recomeçar. Um teto de trabalho determinístico limita a espera. `GenError::Unsupported` sai da API.

**Tech Stack:** Rust edition 2024, crate `sudoku-core`, `rand` 0.10 e `rand_chacha` 0.10. Nenhuma dependência nova.

**Spec:** `docs/superpowers/specs/2026-11-03-engine-muito-dificil-design.md`

## Global Constraints

- `generate` para Fácil, Médio e Difícil não muda para nenhuma seed; os golden tests `puzzle_da_seed_42_e_reproduzivel`, `puzzle_medio_da_seed_42_e_reproduzivel` e `puzzle_dificil_da_seed_42_e_reproduzivel` não podem ser editados.
- `MAX_ATTEMPTS_MEDIUM` continua 50 e `MAX_ATTEMPTS_HARD` continua 250.
- Mesma seed + mesmo nível → mesmo puzzle. Sem `HashMap`, relógio ou IO na biblioteca.
- `sudoku-core` continua puro; dependências continuam só `rand` e `rand_chacha`.
- Ordem das técnicas: Naked Single, Hidden Single, Naked Pair, Hidden Pair, Pointing Pair, Box/Line Reduction, Naked Triple, Hidden Triple, X-Wing, Swordfish, XY-Wing.
- Varredura determinística: dígito no laço de fora, linhas antes de colunas (como o `XWing`); no XY-Wing, pivô, primeira asa e segunda asa por índice crescente.
- `Eliminate` nunca vem vazia, só lista pares que removem um candidato presente, em ordem crescente de célula.
- Muito Difícil: no máximo **22** números dados, resolvido por completo pelo solver lógico, exigindo ao menos uma técnica de nível `MuitoDificil`.
- Meta de tempo: pior caso de até **1 s por chamada** em 200 seeds consecutivas, build de release; no máximo **10 de 200** seeds com `AttemptsExhausted`. Não atingiu? Pare e traga os números.
- Nomes no código em inglês; testes com nomes em português; comentários em português; commits em inglês (Conventional Commits).
- Antes de cada commit de código: `cargo fmt --manifest-path sudoku-core/Cargo.toml` e `cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings` limpo.
- Branch nova `feat/engine-muito-dificil`, a partir de `master` atualizado (com a spec já mergeada) ou de `docs/spec-engine-muito-dificil`. Por task, um commit de código e um `docs` com a página da wiki, indexada em `docs/wiki/README.md`.

## Review Focus

- **Vazamento para os níveis antigos:** Fácil, Médio e Difícil de 50 seeds cada saem idênticos aos de antes da fatia. Comparação na Task 3, com a referência gravada na Task 1.
- **XY-Wing eliminando das próprias asas:** com as asas se enxergando, elas também "enxergam as duas asas", mas não podem perder o z. Pares exatos em `asas_que_se_enxergam_nao_perdem_o_digito`, Task 2.
- **Linha com o dígito numa posição só no Swordfish:** é um Hidden Single, não pode entrar como linha do padrão. Teste `retorna_none_com_linha_de_uma_posicao`, Task 1.
- **Asa igual ao pivô** ({x, y} e {x, y}): é um Naked Pair, não XY-Wing. Teste `retorna_none_quando_a_asa_repete_o_pivo`, Task 2.
- **Números devolvidos no refinamento vindos de outra solução:** todo número dado do Muito Difícil bate com `Puzzle.solution`. Teste `givens_do_muito_dificil_batem_com_a_solucao`, Task 3.

---

### Task 1: Swordfish

**Files:**
- Create: `sudoku-core/src/techniques/swordfish.rs`, `docs/wiki/30-swordfish.md`, `docs/wiki/fundamentos/rust-base-tarefas-26-a-30.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: `Technique`, `Action`, `Deduction`, `CandidateGrid::{from_grid, get, eliminate}`.
- Produces: `pub struct Swordfish`, `technique: "Swordfish"`, `level: Difficulty::MuitoDificil`. Exportado em `techniques` e `lib.rs`; adicionado ao fim de `all_techniques()`.

- [x] **Step 1: Criar a branch `feat/engine-muito-dificil`**

- [x] **Step 2: Gravar a referência dos níveis antigos, antes de mudar qualquer código**

Teste temporário `sudoku-core/tests/referencia.rs`, **sem commit**, `#[ignore]`, que imprime `givens.to_line()` de `generate(nível, s)` para Fácil, Médio e Difícil, `s in 0..50`, e `Err` quando houver.

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test referencia -- --ignored --nocapture > .superpowers/referencia.txt`
Expected: 150 linhas. Comparado na Task 3.

- [x] **Step 3: Escrever os testes que falham**

Montagem: `CandidateGrid::from_grid(&Grid::empty())`, eliminando o 5 das linhas pedidas fora das colunas pedidas (helper `five_in_rows_at(&[(row, &[cols])])`, no molde do `five_in_rows` do X-Wing). Esperado nos casos por linha: o 5 sai das colunas 1, 4 e 7 nas linhas 1, 2, 3, 5, 6 e 7, ou seja `[10, 13, 16, 19, 22, 25, 28, 31, 34, 46, 49, 52, 55, 58, 61, 64, 67, 70]`.

```rust
#[test] fn elimina_o_digito_das_colunas_nas_outras_linhas() {
    // linhas 0, 4 e 8 com o 5 só nas colunas {1, 4, 7}
    // Eliminate(esperado); technique "Swordfish"; level MuitoDificil
}
#[test] fn aceita_linha_com_duas_posicoes() { /* linha 8 só em {1, 7}; mesmo esperado */ }
#[test] fn funciona_trocando_linha_por_coluna() {
    // colunas 0, 4 e 8 com o 5 só nas linhas {1, 4, 7}
    // esperado: [10, 11, 12, 14, 15, 16, 37, 38, 39, 41, 42, 43, 64, 65, 66, 68, 69, 70]
}
#[test] fn retorna_none_com_quatro_colunas() { /* linhas 0, 4, 8 em {1, 4}, {4, 7}, {1, 2} */ }
#[test] fn retorna_none_com_linha_de_uma_posicao() { /* linhas 0, 4 em {1, 4, 7}, linha 8 só em {4} */ }
#[test] fn retorna_none_sem_padrao() { /* grade vazia */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* caso principal, com o esperado já eliminado */ }
```

- [x] **Step 4: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml swordfish`
Expected: erro de compilação, `Swordfish` não existe.

- [x] **Step 5: Implementar `Swordfish`**

Mesmo esqueleto do `XWing::apply`: para `value in 1..=9`, por linha e depois por coluna, monta as 9 máscaras de posições; considera só linhas com 2 ou 3 posições; para `a < b < c`, se a união das três máscaras tem exatamente 3 bits, elimina `value` dessas posições nas outras 6 linhas; primeira lista não vazia (ordenada por célula) vira a dedução. Se compartilhar a montagem das máscaras com o `x_wing.rs`, os testes do X-Wing têm de continuar passando sem mudança.

- [x] **Step 6: Rodar e ver passar; rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo PASS, incluindo os três golden tests.

- [x] **Step 7: Commit do código**

```bash
git add sudoku-core/src
git commit -m "feat: add Swordfish technique"
```

- [x] **Step 8: Wiki 30 e consolidado das tasks 26 a 30**

`docs/wiki/30-swordfish.md`: conceitos de Rust da task (generalizar o X-Wing, máscaras `u16` com `|` e `count_ones`, laços triplos com `a < b < c`, helper compartilhado entre módulos). `docs/wiki/fundamentos/rust-base-tarefas-26-a-30.md`: consolidado das páginas 26 a 30, no formato dos anteriores. Indexar os dois em `docs/wiki/README.md`.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on Swordfish and consolidated doc (tasks 26-30)"
```

---

### Task 2: XY-Wing, solver e rating

**Files:**
- Create: `sudoku-core/src/techniques/xy_wing.rs`, `docs/wiki/31-xy-wing.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`, `sudoku-core/src/rating.rs`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: os mesmos da Task 1, mais `row_of`, `col_of`, `box_of`, `CandidateSet::{count, values, contains}`.
- Produces: `pub struct XYWing`, `technique: "XY-Wing"`, `level: Difficulty::MuitoDificil`. Exportado em `techniques` e `lib.rs`; adicionado ao fim de `all_techniques()`, depois do `Swordfish`.

- [x] **Step 1: Escrever os testes da técnica, que falham**

Montagem: grade vazia; cada célula listada fica só com os candidatos indicados (eliminar os outros 7 ou 8 dígitos).

```rust
#[test] fn elimina_z_das_celulas_que_enxergam_as_duas_asas() {
    // pivô 0 = {1,2}; asa 4 = {1,3} (linha do pivô); asa 19 = {2,3} (box do pivô)
    // esperado: Eliminate([(1,3),(2,3),(21,3),(22,3),(23,3)]); "XY-Wing"; MuitoDificil
}
#[test] fn asas_que_se_enxergam_nao_perdem_o_digito() {
    // pivô 0 = {1,2}; asa 1 = {1,3}; asa 9 = {2,3}
    // esperado: [(2,3),(10,3),(11,3),(18,3),(19,3),(20,3)]  (sem 1 nem 9)
}
#[test] fn retorna_none_quando_as_asas_nao_compartilham_z() { /* pivô 0 = {1,2}; 4 = {1,3}; 19 = {2,4} */ }
#[test] fn retorna_none_quando_a_asa_repete_o_pivo() { /* pivô 0 = {1,2}; 4 = {1,2}; 19 = {1,2} */ }
#[test] fn retorna_none_sem_padrao() { /* grade vazia */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* caso principal com o 3 já fora de 1, 2, 21, 22, 23 */ }
```

- [x] **Step 2: Escrever os testes de solver e rating, que falham**

Em `solver.rs`:

```rust
#[test] fn so_as_tecnicas_de_muito_dificil_resolvem_o_puzzle() {
    // Exige XY-Wing (achado no experimento da spec).
    let grid = Grid::from_line(".82....6.7..4...1.....9.75....1.8..6...235...4...6.....9.3..2.........3.15...9.4.").unwrap();
    assert!(!LogicalSolver::up_to(Difficulty::Dificil).solve(&grid).solved);
    let result = LogicalSolver::new().solve(&grid);
    assert!(result.solved);
    assert!(validate(&result.grid).is_empty());
    assert!(result.steps.iter().any(|s| s.technique == "XY-Wing"));
}
#[test] fn swordfish_entra_na_resolucao() {
    // Exige Swordfish e XY-Wing.
    // "46..1.....5.9.64.3..3..8...2..5.9.....1....9.......34...........8.4..925.398....."
    // mesmas asserções, com technique == "Swordfish"
}
#[test] fn up_to_dificil_nao_usa_tecnica_de_muito_dificil() {
    // nos dois puzzles acima, nenhum passo de up_to(Dificil) tem level MuitoDificil
}
```

Em `rating.rs`:

```rust
#[test] fn puzzle_que_exige_xy_wing_eh_muito_dificil() { /* puzzle do XY-Wing → Some(MuitoDificil) */ }
#[test] fn puzzle_que_so_sai_no_chute_continua_muito_dificil() {
    // Solução única, mas nem o solver completo resolve (puzzle de Arto Inkala).
    let grid = Grid::from_line("8..........36......7..9.2...5...7.......457.....1...3...1....68..85...1..9....4..").unwrap();
    assert!(!LogicalSolver::new().solve(&grid).solved);
    assert_eq!(rate(&grid), Some(Difficulty::MuitoDificil));
}
```

- [x] **Step 3: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml xy_wing`
Expected: erro de compilação, `XYWing` não existe.

- [x] **Step 4: Implementar `XYWing`**

"Enxerga" = célula diferente na mesma linha, coluna ou box. Para cada pivô bivalor {x, y} (x < y) por índice: para cada asa 1 bivalor que o pivô enxerga, contendo x e não y (z = o outro dígito); para cada asa 2 bivalor, diferente da asa 1, que o pivô enxerga e que é exatamente {y, z}: elimina z das células, fora as duas asas, que enxergam as duas asas e têm z. Primeira lista não vazia vira a dedução. Pré-filtrar as células bivalores numa lista fixa (`[usize; 81]` + tamanho, ou `Vec` montado uma vez por chamada) para não varrer 81³ combinações.

- [x] **Step 5: Atualizar o doc comment de `rate`**

A nota "hoje só existem técnicas até Difícil" sai. Fica: `MuitoDificil` também cobre puzzles de solução única que nem as técnicas de Muito Difícil resolvem ("pelo menos Muito Difícil").

- [x] **Step 6: Rodar e ver passar; rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo PASS, incluindo os três golden tests.

- [x] **Step 7: Commit do código**

```bash
git add sudoku-core/src
git commit -m "feat: add XY-Wing technique"
```

- [x] **Step 8: Wiki 31**

`docs/wiki/31-xy-wing.md`: conceitos de Rust da task (função `sees` com `||`, iteradores aninhados com `filter`, pré-filtrar para cortar combinações, `let ... else`/`if let` para extrair o z). Indexar.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on XY-Wing"
```

---

### Task 3: Gerador de Muito Difícil

**Files:**
- Create: `docs/wiki/32-gerador-muito-dificil.md`
- Modify: `sudoku-core/src/generator.rs`, `sudoku-core/tests/api.rs`, `docs/wiki/12-gerador-rng-com-seed-e-traits-em-escopo.md`, `docs/wiki/fundamentos/rust-base-tarefas-11-a-15.md`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: `LogicalSolver::up_to`, `full_solution`, `dig`, as técnicas das Tasks 1 e 2.
- Produces:
  - `pub enum GenError { AttemptsExhausted(Difficulty) }` (sem `Unsupported`); `match` de `generate` cobre os quatro níveis, sem braço `_`.
  - `const MAX_GIVENS_VERY_HARD: usize = 22;`
  - `const REFILL_VERY_HARD: usize = 6;` (K), `const MAX_REFINEMENTS_VERY_HARD: usize = 80;`, `const MAX_SOLUTIONS_VERY_HARD: usize = 20;` (provisórios, fixados na Task 4)
  - `fn generate_very_hard(seed: u64, rng: &mut ChaCha12Rng) -> Result<Puzzle, GenError>`
  - `fn redig(puzzle: &mut Grid, solver: &LogicalSolver, rng: &mut ChaCha12Rng)`
  - `fn refill(puzzle: &Grid, solution: &Grid, count: usize, rng: &mut ChaCha12Rng) -> Grid`
  - `fn accepts_very_hard(givens: &Grid, solver: &LogicalSolver) -> bool`

- [x] **Step 1: Escrever os testes que falham**

Em `generator.rs`. As seeds abaixo são provisórias: rode `generate` em release para `0..20` e use, nos testes, as que terminam em menos de 200 ms, porque o CI roda em debug. Cada teste chama `generate` uma vez só.

```rust
#[test] fn gera_puzzle_muito_dificil_valido() {
    let p = generate(Difficulty::MuitoDificil, SEED).unwrap();
    assert_eq!(p.difficulty, Difficulty::MuitoDificil);
    assert_eq!(count_solutions(&p.givens, 2), 1);
    assert_eq!(rate(&p.givens), Some(Difficulty::MuitoDificil));
    let result = LogicalSolver::new().solve(&p.givens);
    assert!(result.solved);
    assert_eq!(result.grid, p.solution);
    let givens = (0..81).filter(|&i| p.givens.get(i) != Cell::Empty).count();
    assert!(givens <= 22, "{givens} números");
}
#[test] fn givens_do_muito_dificil_batem_com_a_solucao() {
    // todo i com givens.get(i) == Filled(v) tem solution.get(i) == Filled(v)
}
#[test] fn mesma_seed_gera_mesmo_puzzle_muito_dificil() { /* duas chamadas, assert_eq! */ }
#[test] fn seeds_extremas_geram_muito_dificil_valido() {
    // 0 e u64::MAX: Ok com rate == MuitoDificil, ou AttemptsExhausted(MuitoDificil) só se a
    // medição mostrar que é o caso daquela seed (anote no comentário)
}
#[test] fn eliminacoes_do_muito_dificil_nunca_removem_a_resposta() {
    // no molde de eliminacoes_nunca_removem_a_resposta, para uma seed de Muito Difícil
}
```

Remover `niveis_nao_suportados_ainda_retornam_erro`. Em `tests/api.rs`, `fluxo_completo_muito_dificil` no molde de `fluxo_completo_dificil`, com a asserção de no máximo 22 números.

- [x] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml muito_dificil`
Expected: FAIL com `Unsupported(MuitoDificil)`.

- [x] **Step 3: Implementar o gerador**

O ramo `Difficulty::MuitoDificil` chama `generate_very_hard`, que segue o pseudocódigo da seção 8 da spec. Decisões fixadas aqui:
- `accepts_very_hard`: no máximo `MAX_GIVENS_VERY_HARD` números, `solver.solve` resolve, e algum passo tem `level == MuitoDificil`. Uma chamada a `solve` só.
- `refill`: embaralha as células vazias com o `rng` e devolve a solução nas `count` primeiras (sem repetição).
- `redig`: embaralha as células preenchidas com o `rng` e tenta tirar cada uma, mantendo-a vazia só se `solver.solve` ainda resolve. Pode reaproveitar o laço de `dig` recebendo a lista de células, desde que `dig` consuma o RNG exatamente como hoje.
- O refinamento troca o puzzle atual pelo candidato quando ele tem no máximo tantos números quanto o atual.

Remover `GenError::Unsupported` e o braço `_`.

- [x] **Step 4: Rodar e ver passar; rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo PASS, incluindo os três golden tests.

- [x] **Step 5: Comparar com a referência da Task 1**

Rodar o teste temporário `referencia` de novo e comparar com `.superpowers/referencia.txt`.
Expected: as 150 linhas idênticas (`diff` sem saída).

- [x] **Step 6: Commit do código**

```bash
git add sudoku-core/src sudoku-core/tests/api.rs
git commit -m "feat: generate Very Hard puzzles by refining near misses"
```

- [x] **Step 7: Wiki 32 e notas sobre `Unsupported`**

`docs/wiki/32-gerador-muito-dificil.md`: conceitos de Rust da task (remover uma variante e deixar o `match` exaustivo trabalhar a favor, `clone` de `Grid` para tentar um candidato, busca local com estado mutável, RNG compartilhado e determinismo). Na wiki 12 e no consolidado 11–15, acrescentar uma nota curta de que `Unsupported` saiu na fatia Muito Difícil, sem reescrever o texto original. Indexar.

```bash
git add docs/wiki
git commit -m "docs: add wiki page on Very Hard generation"
```

---

### Task 4: Desempenho, constantes finais e documentação

**Files:**
- Create: `docs/wiki/33-desempenho-do-muito-dificil.md`
- Modify: `sudoku-core/src/generator.rs`, `sudoku-core/src/techniques/*.rs` (se a otimização pedir), `sudoku-core/tests/desempenho.rs`, `docs/architecture.md`, `docs/superpowers/specs/2026-06-14-sudoku-engine-design.md`, `docs/superpowers/specs/2026-11-03-engine-muito-dificil-design.md`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: tudo da Task 3.
- Produces: valores finais de `REFILL_VERY_HARD`, `MAX_REFINEMENTS_VERY_HARD`, `MAX_SOLUTIONS_VERY_HARD`; golden test do Muito Difícil.

- [x] **Step 1: Escrever o teste de desempenho**

Em `tests/desempenho.rs`, `muito_dificil_cabe_na_meta_de_tempo`, `#[ignore]`, no molde de `dificil_cabe_na_meta_de_tempo`: 200 seeds, imprime mediana, pior e esgotadas; afirma `worst <= 1 s` e `exhausted <= 10`.

- [x] **Step 2: Medir a linha de base**

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test desempenho muito_dificil -- --ignored --nocapture`
Anote mediana, pior e esgotadas. Grave também a referência do Muito Difícil (as 50 primeiras seeds) acrescentando o nível ao teste temporário `referencia`.

- [x] **Step 3: Otimizar sem mudar saída**

Medir antes de mexer. Caminhos prováveis: tirar alocações do caminho quente de Swordfish e XY-Wing; fazer o `solve` usado por `dig`/`redig` parar assim que empaca, sem montar a lista de passos que ninguém lê. Depois de cada mudança: suíte inteira e `referencia` dos **quatro** níveis idêntica.

- [x] **Step 4: Ajustar as constantes por medição**

Testar algumas combinações de `REFILL_VERY_HARD` e `MAX_REFINEMENTS_VERY_HARD` em torno de (6, 80) e escolher a de menor p90; depois, o maior `MAX_SOLUTIONS_VERY_HARD` que mantém o pior caso em até 1 s. Aqui a saída do Muito Difícil muda de propósito, porque o golden test ainda não existe.

- [x] **Step 5: Ponto de parada**

Run: o comando do Step 2.
Expected: PASS (pior ≤ 1 s, esgotadas ≤ 10 de 200). **Se falhar, pare** e traga mediana, p90, pior e esgotadas para a pessoa responsável. Não relaxe a meta, o critério nem o limite de 22 números.

- [x] **Step 6: Golden test e seed esgotada**

```rust
#[test] fn puzzle_muito_dificil_da_seed_42_e_reproduzivel() {
    // Golden test do Muito Difícil. Também trava as três constantes e a ordem das técnicas.
    // assert_eq!(generate(MuitoDificil, 42).unwrap().givens.to_line(), "<saída medida>");
}
#[test] fn muito_dificil_desiste_depois_do_teto_de_trabalho() {
    // Só se uma das 200 seeds medidas esgotou: essa seed devolve
    // Err(AttemptsExhausted(MuitoDificil)). Se nenhuma esgotou, este teste não entra.
}
```

Se a seed 42 for lenta em debug, ainda assim é ela: é a seed do contrato.

- [x] **Step 7: Rodar a suíte inteira, fmt e clippy**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml` e `cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings`
Expected: tudo PASS, sem warnings. Apagar o teste temporário `referencia.rs`.

- [x] **Step 8: Commit do código**

```bash
git add sudoku-core
git commit -m "perf: speed up Very Hard generation and fix its constants"
```

- [x] **Step 9: Documentação**

- `docs/architecture.md`: status do módulo (engine completo), tabela de níveis (Muito Difícil sempre resolvível por lógica, no máximo 22 números), técnicas, API (`generate` aceita os quatro níveis, `GenError` só com `AttemptsExhausted`, `Swordfish` e `XYWing` expostos), fluxo de geração com refinamento e as constantes finais, números medidos na seção 10. Na seção 10, tirar "a próxima fatia adiciona as técnicas de Muito Difícil" e registrar a ideia de encher Fácil e Médio.
- `Status` da spec original do engine: os quatro níveis concluídos. `Status` da spec do Muito Difícil: implementado, com os números finais medidos e o que mudou em relação ao experimento.
- `docs/wiki/33-desempenho-do-muito-dificil.md`: conceitos de Rust da task (medir antes de otimizar, build de release, alocação no caminho quente, constantes como contrato). Indexar.

```bash
git add docs
git commit -m "docs: update architecture and add wiki page on Very Hard performance"
```
