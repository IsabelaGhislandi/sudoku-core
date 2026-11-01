# Engine — Fatia de Nível Médio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer o `sudoku-core` gerar e classificar puzzles de nível Médio, sem alterar nenhum puzzle Fácil.

**Architecture:** O solver passa a manter um `CandidateGrid` durante a resolução, e cada `Deduction` carrega uma `Action` (`Place` ou `Eliminate`). Três técnicas novas entram na lista do solver, que aceita um nível máximo; o Fácil continua usando só singles. O gerador de Médio repete tentativas com um único RNG até `rate` devolver `Medio`, com limite fixo de tentativas.

**Tech Stack:** Rust edition 2024, crate `sudoku-core`, `rand` 0.10 e `rand_chacha` 0.10. Nenhuma dependência nova.

**Spec:** `docs/superpowers/specs/2026-11-01-engine-medio-design.md`

## Global Constraints

- `generate(Facil, seed)` não muda para nenhuma seed; `puzzle_da_seed_42_e_reproduzivel` não pode ser editado.
- Mesma seed + mesmo nível → mesmo puzzle, inclusive entre versões. Sem `HashMap`, relógio ou IO na biblioteca.
- `sudoku-core` continua puro; dependências continuam só `rand` e `rand_chacha`.
- A ordem de varredura de `NakedSingle` e `HiddenSingle` não muda.
- Ordem das técnicas: Naked Single, Hidden Single, Naked Pair, Hidden Pair, Pointing Pair.
- Uma técnica só devolve `Eliminate` com pares que removem um candidato presente, e nunca com lista vazia.
- Meta de tempo: até 1 s no pior caso, em 200 seeds consecutivas, no build de release. Não atingiu? Pare e traga os números.
- Testes com nomes em português e TDD de verdade; comentários em português; commits em inglês (Conventional Commits).
- Antes de cada commit de código: `cargo fmt`, e `cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings` limpo.
- A implementação roda em uma branch nova `feat/engine-medio`, criada a partir de `master`, com um commit de código e um `docs` com a página da wiki por task, indexada em `docs/wiki/README.md`.

## Review Focus

- **Grade com conflito** (dois 5 na mesma linha): `solve` termina sem panic e `rate` devolve `None`. Teste na Task 2.
- **Célula sem nenhum candidato** (grade contraditória): o solver empaca, em vez de dar panic ou entrar em loop. Teste na Task 2.
- **Grade já completa:** `solve` devolve `solved: true` sem passos e `rate` devolve `Facil`. Teste na Task 2.
- **Seeds nos extremos** (`0` e `u64::MAX`) em Médio: devolvem um puzzle válido, sem panic. Teste na Task 6.
- **Vazamento de técnicas para o Fácil:** puzzles Fáceis de várias seeds nunca têm passo `Eliminate` e continuam com `rate == Facil`. Teste na Task 6.

---

### Task 1: Candidatos guardados (`CandidateGrid`)

**Files:**
- Modify: `sudoku-core/src/candidates.rs`, `sudoku-core/src/lib.rs`
- Create: `docs/wiki/16-candidategrid-e-estado-mutavel.md`

**Interfaces:**
- Produces: `CandidateSet::remove(&mut self, value: u8)`; `pub struct CandidateGrid` (`Debug, Clone, PartialEq, Eq`) com `from_grid(&Grid) -> Self`, `get(&self, index: usize) -> CandidateSet`, `place(&mut self, index: usize, value: u8)`, `eliminate(&mut self, index: usize, value: u8)`. Exportado em `lib.rs`.

- [ ] **Step 1: Criar a branch `feat/engine-medio` a partir de `master` atualizado**

- [ ] **Step 2: Gravar a referência do Fácil, antes de mudar qualquer código**

Crie o arquivo temporário `sudoku-core/tests/referencia_facil.rs`, **sem commit**, com um teste `#[ignore]` que imprime `generate(Difficulty::Facil, seed).unwrap().givens.to_line()` para as seeds `0..50`.

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test referencia_facil -- --ignored --nocapture > "$TEMP/referencia-facil.txt"`
Expected: 50 linhas de 81 caracteres no arquivo, que fica **fora do repositório**. Ele é comparado nas Tasks 2 e 6.

- [ ] **Step 3: Escrever os testes que falham, em `candidates.rs`**

```rust
#[test]
fn construcao_bate_com_candidates_for() { /* para toda célula de uma grade com pistas: get(i) == candidates_for(&grid, i) */ }

#[test]
fn colocar_limpa_a_celula_e_os_vizinhos() {
    let mut c = CandidateGrid::from_grid(&Grid::empty());
    c.place(0, 5);
    assert_eq!(c.get(0).count(), 0);
    assert!(!c.get(8).contains(5));   // linha
    assert!(!c.get(72).contains(5));  // coluna
    assert!(!c.get(20).contains(5));  // box
    assert!(c.get(40).contains(5));   // sem relação com a célula 0
}

#[test]
fn colocar_equivale_a_reconstruir_da_grade() { /* após place, igual a from_grid da grade com o valor colocado */ }

#[test]
fn eliminar_remove_so_o_par_pedido() {
    let mut c = CandidateGrid::from_grid(&Grid::empty());
    c.eliminate(0, 5);
    assert!(!c.get(0).contains(5));
    assert!(c.get(0).contains(6));
    assert!(c.get(1).contains(5));
}
```

- [ ] **Step 4: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml candidates`
Expected: erro de compilação, `CandidateGrid` não existe.

- [ ] **Step 5: Implementar `CandidateSet::remove` e `CandidateGrid`, com as assinaturas de Interfaces**

- [ ] **Step 6: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml candidates`
Expected: todos os testes do módulo passam.

- [ ] **Step 7: Commit do código**

```bash
git add sudoku-core/src/candidates.rs sudoku-core/src/lib.rs
git commit -m "feat: add CandidateGrid to keep candidates during a solve"
```

- [ ] **Step 8: Escrever a página 16 da wiki e fazer o commit**

Temas: array de tipo `Copy`, `&mut self` vs. consumir, por que os candidatos deixaram de ser só calculados.

```bash
git add docs/wiki/16-candidategrid-e-estado-mutavel.md docs/wiki/README.md
git commit -m "docs: add wiki page on CandidateGrid and mutable state"
```

---

### Task 2: `Action`, nova trait `Technique` e solver por nível

**Files:**
- Modify: `sudoku-core/src/techniques/mod.rs`, `techniques/naked_single.rs`, `techniques/hidden_single.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/generator.rs`, `sudoku-core/src/lib.rs`, `sudoku-core/tests/api.rs`
- Create: `docs/wiki/17-enums-com-dados-e-api-por-nivel.md`

**Interfaces:**
- Consumes: `CandidateGrid` (Task 1).
- Produces:
  - `pub enum Action { Place { index: usize, value: u8 }, Eliminate(Vec<(usize, u8)>) }` (`Debug, Clone, PartialEq, Eq`), exportado em `lib.rs`.
  - `pub struct Deduction { pub action: Action, pub technique: &'static str, pub level: Difficulty }`.
  - `trait Technique { fn level(&self) -> Difficulty; fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction>; }`.
  - `LogicalSolver::new()` (todas as técnicas), `LogicalSolver::up_to(level: Difficulty)` (filtra por `level() <= level`), `next_step(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction>`.

- [ ] **Step 1: Escrever os testes novos e adaptar os existentes em `solver.rs`, `naked_single.rs`, `hidden_single.rs`**

Os testes atuais passam a usar `CandidateGrid::from_grid(&grid)` e `Action::Place { .. }`. Novos, em `solver.rs`:

```rust
#[test]
fn solve_termina_em_grade_com_conflito() {
    let mut grid = Grid::empty();
    grid.set(0, Cell::Filled(5));
    grid.set(1, Cell::Filled(5));
    let _ = LogicalSolver::new().solve(&grid); // não pode dar panic nem travar
    assert_eq!(crate::rating::rate(&grid), None);
}

#[test]
fn empaca_em_celula_sem_candidatos() {
    // Linha 0 com 1 a 8 e o 9 na coluna 0, mais abaixo: a célula 0 fica sem candidatos.
    let mut grid = Grid::from_line(&format!(".12345678{}", ".".repeat(72))).unwrap();
    grid.set(27, Cell::Filled(9));
    assert!(!LogicalSolver::new().solve(&grid).solved);
}

#[test]
fn grade_completa_nao_tem_passos() {
    let grid = Grid::from_line(solucao()).unwrap();
    let result = LogicalSolver::new().solve(&grid);
    assert!(result.solved);
    assert!(result.steps.is_empty());
    assert_eq!(crate::rating::rate(&grid), Some(Difficulty::Facil));
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml solver`
Expected: erro de compilação em `Action` e na nova assinatura.

- [ ] **Step 3: Implementar a mudança**

- `NakedSingle` e `HiddenSingle` leem de `candidates`, sem mudar a ordem de varredura.
- `solve` cria o `CandidateGrid` e aplica a ação: `Place` faz `set` na grade e `place` nos candidatos; `Eliminate` chama `eliminate` para cada par.
- `dig_facil` usa `LogicalSolver::up_to(Difficulty::Facil)`.
- Remova `next_hint` de `solver.rs` e `lib.rs`, e o trecho correspondente de `tests/api.rs`, inclusive o import.
- Em `rate`, devolva `None` logo no início se `validate(grid)` não estiver vazio. Sem essa guarda, `count_solutions` tentaria provar por busca que uma grade quase vazia com conflito não tem solução, o que pode demorar demais.

- [ ] **Step 4: Rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo passa, com `puzzle_da_seed_42_e_reproduzivel` **sem edição**.

- [ ] **Step 5: Comparar com a referência do Fácil**

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test referencia_facil -- --ignored --nocapture > "$TEMP/referencia-facil-2.txt"` e compare os dois arquivos (`git diff --no-index`), só nas linhas de 81 caracteres.
Expected: as 50 linhas são idênticas. Se houver diferença, é bug: conserte antes do commit.

- [ ] **Step 6: Commit do código, sem o arquivo temporário**

```bash
git add sudoku-core/src sudoku-core/tests/api.rs
git commit -m "feat: represent eliminations in Deduction and cap the solver by level"
```

- [ ] **Step 7: Escrever a página 17 da wiki e fazer o commit**

Temas: enum com dados e `match`, `Vec` em variante (o tipo deixa de poder ser `Copy`), mudar uma trait pública, por que `next_hint` saiu.

```bash
git add docs/wiki/17-enums-com-dados-e-api-por-nivel.md docs/wiki/README.md
git commit -m "docs: add wiki page on data-carrying enums and the level-capped API"
```

---

### Task 3: Naked Pair

**Files:**
- Create: `sudoku-core/src/techniques/naked_pair.rs`, `docs/wiki/18-naked-pair-e-iteracao-por-unidades.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`

**Interfaces:**
- Consumes: `Technique`, `Action`, `CandidateGrid`.
- Produces: `pub struct NakedPair`, exportado em `lib.rs`; `pub(crate) fn units() -> [[usize; 9]; 27]` em `techniques/mod.rs`, com linhas, depois colunas, depois boxes, e células em ordem crescente. Usado também pelas Tasks 4 e 5.

**Regra de ordenação:** a técnica devolve a primeira unidade, na ordem de `units()`, que tenha algo a eliminar. Os pares vêm ordenados por célula e depois por dígito.

- [ ] **Step 1: Escrever os testes que falham**

Fixture: a partir de `CandidateGrid::from_grid(&Grid::empty())`, elimine `3..=9` das células 0 e 1, que ficam com `{1, 2}`.

```rust
#[test]
fn elimina_o_par_das_outras_celulas_da_unidade() {
    // A linha 0 vem antes do box 0 em units().
    let esperado: Vec<(usize, u8)> = (2..9).flat_map(|i| [(i, 1), (i, 2)]).collect();
    let d = NakedPair.apply(&Grid::empty(), &par_nas_celulas_0_e_1()).unwrap();
    assert_eq!(d.action, Action::Eliminate(esperado));
    assert_eq!(d.technique, "Naked Pair");
    assert_eq!(d.level, Difficulty::Medio);
}

#[test]
fn retorna_none_sem_padrao() { /* CandidateGrid de Grid::empty() => None */ }

#[test]
fn retorna_none_quando_nao_ha_o_que_eliminar() {
    // Remove 1 e 2 de todas as outras células da linha 0 e do box 0.
    // Em nenhuma unidade o par elimina algo.
}
```

Em `solver.rs`:

```rust
#[test]
fn up_to_facil_nao_usa_tecnica_de_medio() {
    // Mesma fixture: sem singles, só o Naked Pair se aplica.
    assert_eq!(LogicalSolver::up_to(Difficulty::Facil).next_step(&grid, &candidatos), None);
    let d = LogicalSolver::new().next_step(&grid, &candidatos).unwrap();
    assert_eq!(d.technique, "Naked Pair");
}
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml naked_pair`
Expected: erro de compilação, `NakedPair` não existe.

- [ ] **Step 3: Implementar `NakedPair` e `units()` e registrar a técnica depois de `HiddenSingle`**

- [ ] **Step 4: Rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo passa, com o golden da seed 42 intacto.

- [ ] **Step 5: Commit do código**

```bash
git add sudoku-core/src
git commit -m "feat: add Naked Pair technique"
```

- [ ] **Step 6: Escrever a página 18 da wiki e fazer o commit**

Temas: `flat_map`, arrays aninhados, `pub(crate)`, comparar por valor com `PartialEq`.

```bash
git add docs/wiki/18-naked-pair-e-iteracao-por-unidades.md docs/wiki/README.md
git commit -m "docs: add wiki page on Naked Pair and iterating over units"
```

---

### Task 4: Hidden Pair

**Files:**
- Create: `sudoku-core/src/techniques/hidden_pair.rs`, `docs/wiki/19-hidden-pair-e-mascaras.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`

**Interfaces:**
- Consumes: `units()`, `Technique`, `Action`, `CandidateGrid`.
- Produces: `pub struct HiddenPair`, exportado em `lib.rs`.

Varredura: unidades na ordem de `units()`, pares de dígitos `a < b` em ordem crescente. A técnica devolve o primeiro par com algo a eliminar, ordenado por célula e dígito.

- [ ] **Step 1: Escrever os testes que falham**

Fixture: a partir do `CandidateGrid` de `Grid::empty()`, elimine 1 e 2 das células 2 a 8, de modo que eles só caibam nas células 0 e 1 da linha 0.

```rust
#[test]
fn elimina_os_outros_candidatos_das_duas_celulas() {
    let esperado: Vec<(usize, u8)> = [0, 1].into_iter().flat_map(|i| (3..=9).map(move |v| (i, v))).collect();
    let d = HiddenPair.apply(&Grid::empty(), &fixture()).unwrap();
    assert_eq!(d.action, Action::Eliminate(esperado));
    assert_eq!(d.technique, "Hidden Pair");
    assert_eq!(d.level, Difficulty::Medio);
}

#[test]
fn retorna_none_sem_padrao() { /* CandidateGrid de Grid::empty() => None */ }

#[test]
fn retorna_none_quando_as_celulas_ja_so_tem_o_par() { /* também elimina 3..=9 das células 0 e 1 => None */ }

#[test]
fn nao_confunde_digitos_em_celulas_diferentes() { /* 1 só em {0, 1} e 2 só em {0, 2} na linha => None */ }
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml hidden_pair`
Expected: erro de compilação, `HiddenPair` não existe.

- [ ] **Step 3: Implementar `HiddenPair` e registrar a técnica depois de `NakedPair`**

- [ ] **Step 4: Rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo passa.

- [ ] **Step 5: Commit do código**

```bash
git add sudoku-core/src
git commit -m "feat: add Hidden Pair technique"
```

- [ ] **Step 6: Escrever a página 19 da wiki e fazer o commit**

Temas: `move` em closures aninhadas, conjuntos de células como máscara, naked e hidden como duas visões do mesmo fato.

```bash
git add docs/wiki/19-hidden-pair-e-mascaras.md docs/wiki/README.md
git commit -m "docs: add wiki page on Hidden Pair and cell masks"
```

---

### Task 5: Pointing Pair e consolidado 16–20

**Files:**
- Create: `sudoku-core/src/techniques/pointing_pair.rs`, `docs/wiki/20-pointing-pair.md`, `docs/wiki/fundamentos/rust-base-tarefas-16-a-20.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`

**Interfaces:**
- Consumes: `Technique`, `Action`, `CandidateGrid`.
- Produces: `pub struct PointingPair`, exportado em `lib.rs`.

Varredura: dígitos de 1 a 9, boxes de 0 a 8, testando linha antes de coluna. Só se aplica com 2 ou 3 células candidatas no box. Pares ordenados por célula.

- [ ] **Step 1: Escrever os testes que falham**

Fixture: a partir do `CandidateGrid` de `Grid::empty()`, elimine 5 das células 2, 9, 10, 11, 18, 19 e 20, de modo que no box 0 o 5 só caiba nas células 0 e 1.

```rust
#[test]
fn elimina_o_digito_do_resto_da_linha() {
    let esperado: Vec<(usize, u8)> = (3..9).map(|i| (i, 5)).collect();
    let d = PointingPair.apply(&Grid::empty(), &fixture()).unwrap();
    assert_eq!(d.action, Action::Eliminate(esperado));
    assert_eq!(d.technique, "Pointing Pair");
    assert_eq!(d.level, Difficulty::Medio);
}

#[test]
fn elimina_o_digito_do_resto_da_coluna() { /* 5 só nas células 0 e 9 do box 0 => elimina das células 27, 36, 45, 54, 63, 72 */ }

#[test]
fn aceita_tres_celulas_alinhadas() { /* 5 nas células 0, 1 e 2 do box 0 => elimina de 3..9 */ }

#[test]
fn retorna_none_sem_alinhamento() { /* 5 nas células 0 e 10 do box 0, e em todas as dos demais boxes => None */ }

#[test]
fn retorna_none_quando_nao_ha_o_que_eliminar() { /* fixture + elimina 5 de 3..9 => None */ }
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml pointing_pair`
Expected: erro de compilação, `PointingPair` não existe.

- [ ] **Step 3: Implementar `PointingPair` e registrar a técnica depois de `HiddenPair`**

- [ ] **Step 4: Rodar a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo passa.

- [ ] **Step 5: Commit do código**

```bash
git add sudoku-core/src
git commit -m "feat: add Pointing Pair technique"
```

- [ ] **Step 6: Escrever a página 20 e o consolidado 16–20, indexar os dois e fazer o commit**

O consolidado segue o formato de `fundamentos/rust-base-tarefas-11-a-15.md`, organizado por conceito.

```bash
git add docs/wiki/20-pointing-pair.md docs/wiki/fundamentos/rust-base-tarefas-16-a-20.md docs/wiki/README.md
git commit -m "docs: add wiki page on Pointing Pair and consolidated doc (tasks 16-20)"
```

---

### Task 6: Gerador de Médio, medição e fechamento

**Files:**
- Modify: `sudoku-core/src/generator.rs`, `sudoku-core/src/rating.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/tests/api.rs`, `docs/architecture.md`
- Create: `sudoku-core/tests/desempenho.rs`, `docs/wiki/21-gerador-medio-e-tentativas.md`

**Interfaces:**
- Consumes: `LogicalSolver::up_to`, as três técnicas, `rate`.
- Produces: `GenError::AttemptsExhausted(Difficulty)`; `fn dig(solution: &Grid, solver: &LogicalSolver, rng: &mut ChaCha12Rng) -> Grid`, no lugar de `dig_facil`; `const MAX_TENTATIVAS_MEDIO: usize`, começando em `50`.

- [ ] **Step 1: Escrever os testes que falham, em `generator.rs`**

```rust
#[test]
fn gera_puzzle_medio_valido_e_unico() {
    let p = generate(Difficulty::Medio, 42).unwrap();
    assert_eq!(p.difficulty, Difficulty::Medio);
    assert_eq!(count_solutions(&p.givens, 2), 1);
    assert_eq!(crate::rating::rate(&p.givens), Some(Difficulty::Medio));
}

#[test]
fn mesma_seed_gera_mesmo_puzzle_medio() { /* duas chamadas com seed 7 => iguais */ }

#[test]
fn seeds_extremas_geram_medio_valido() { /* seeds 0 e u64::MAX: rate == Some(Medio) */ }

#[test]
fn eliminacoes_nunca_removem_a_resposta() {
    // Seeds 1, 2 e 3: em cada passo de solve(&givens), Eliminate nunca remove o dígito da
    // solution naquela célula e Place sempre coloca o valor da solution. Ao fim, grid == solution,
    // e há ao menos um passo Eliminate.
}

#[test]
fn facil_nunca_usa_tecnica_de_medio() {
    // Seeds 0..5: nenhum passo de solve(&givens) é Eliminate e rate == Some(Facil).
}
```

Em `niveis_nao_suportados_ainda_retornam_erro`, troque `Medio` por `Dificil`.

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml generator`
Expected: os testes de Médio falham com `Unsupported(Medio)`.

- [ ] **Step 3: Generalizar `dig` e implementar o laço do Médio, seguindo a seção 9 da spec**

O fluxo do Fácil continua sem laço e consome o RNG como hoje. `Dificil` e `MuitoDificil` continuam `Unsupported`.

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml generator`
Expected: todos passam.

- [ ] **Step 5: Medir, em `tests/desempenho.rs`**

Crie um teste `#[ignore]`, `medio_cabe_na_meta_de_tempo`, que cronometra `generate(Medio, seed)` com `std::time::Instant` para as seeds `0..200`. Ele imprime o pior tempo e a mediana, e falha se alguma seed devolver erro ou passar de 1 s.

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test desempenho -- --ignored --nocapture`
Expected: passa. **Se falhar, pare e traga os números.** Se passar, ajuste `MAX_TENTATIVAS_MEDIO` de acordo com o maior número de tentativas observado, deixando folga, e registre a medição na wiki.

- [ ] **Step 6: Travar o Médio com testes de valor conhecido**

Com a saída de `generate(Medio, 42)` em mãos, depois de fixar `MAX_TENTATIVAS_MEDIO`, escreva:

- em `generator.rs`, `puzzle_medio_da_seed_42_e_reproduzivel`, um golden test no formato do Fácil;
- em `rating.rs`, `puzzle_que_exige_par_eh_medio`, usando essa grade: `rate == Some(Medio)`;
- em `solver.rs`, `so_as_tecnicas_de_medio_resolvem_o_puzzle`: `up_to(Facil)` empaca, `new()` resolve e chega à `solution`;
- em `tests/api.rs`, `fluxo_completo_medio`: gera, confere unicidade e `rate`, e usa `Action` pela API pública.

- [ ] **Step 7: Rodar tudo como o CI, e comparar o Fácil de novo**

```bash
cargo fmt --manifest-path sudoku-core/Cargo.toml --check
cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings
cargo build --manifest-path sudoku-core/Cargo.toml --locked
cargo test --manifest-path sudoku-core/Cargo.toml --locked
```

Expected: tudo limpo. Repita a comparação da Task 2, Step 5: as 50 linhas continuam idênticas. Depois apague `sudoku-core/tests/referencia_facil.rs` e os dois arquivos de referência. Confira que `Cargo.lock` não mudou.

- [ ] **Step 8: Commit do código**

Atualize antes o doc comment de `generate` e a nota em `rate`, que dizem que só existe Fácil.

```bash
git add sudoku-core/src sudoku-core/tests/api.rs sudoku-core/tests/desempenho.rs
git commit -m "feat: generate Medium puzzles with bounded attempts"
```

- [ ] **Step 9: Atualizar a documentação e fazer o commit**

Em `docs/architecture.md`, atualize a tabela de módulos, a árvore de arquivos, a API pública (sem `next_hint`), a seção 5 e a evolução futura, registrando que a dica pertence ao Game. Página 21 da wiki, com os temas: laço com RNG compartilhado, por que o limite é por tentativas, medição em release e o número medido.

```bash
git add docs/architecture.md docs/wiki/21-gerador-medio-e-tentativas.md docs/wiki/README.md
git commit -m "docs: update architecture and add wiki page on Medium generation"
```
