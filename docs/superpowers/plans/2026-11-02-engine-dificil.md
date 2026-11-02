# Engine — Fatia de Nível Difícil Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fazer o `sudoku-core` gerar e classificar puzzles de nível Difícil, sem alterar nenhum puzzle Fácil ou Médio.

**Architecture:** Quatro técnicas novas de eliminação, todas com nível `Dificil`, entram **no fim** da lista do solver. Fácil e Médio continuam cavando com `up_to(nível)`, e `rate` só chega às técnicas novas quando as antigas empacam, então nenhuma saída antiga muda. O gerador ganha o nível Difícil com o mesmo laço de tentativas do Médio.

**Tech Stack:** Rust edition 2024, crate `sudoku-core`, `rand` 0.10 e `rand_chacha` 0.10. Nenhuma dependência nova.

**Spec:** `docs/superpowers/specs/2026-11-02-engine-dificil-design.md`

## Global Constraints

- `generate(Facil, seed)` e `generate(Medio, seed)` não mudam para nenhuma seed; `puzzle_da_seed_42_e_reproduzivel` e `puzzle_medio_da_seed_42_e_reproduzivel` não podem ser editados.
- `MAX_TENTATIVAS_MEDIO` continua 50, e o Médio consome o RNG exatamente como hoje.
- Mesma seed + mesmo nível → mesmo puzzle. Sem `HashMap`, relógio ou IO na biblioteca.
- `sudoku-core` continua puro; dependências continuam só `rand` e `rand_chacha`.
- Ordem das técnicas: Naked Single, Hidden Single, Naked Pair, Hidden Pair, Pointing Pair, Box/Line Reduction, Naked Triple, Hidden Triple, X-Wing.
- Varredura em ordem crescente de índice (unidades na ordem de `units()`, células, dígitos), linha antes de coluna.
- `Eliminate` nunca vem vazia, e só lista pares que removem um candidato presente. Os pares vêm em ordem crescente de célula e, dentro da célula, de dígito.
- Meta de tempo: até 1 s no pior caso, em 200 seeds consecutivas, no build de release. Não atingiu? Pare e traga os números.
- Testes com nomes em português e TDD de verdade; comentários em português; commits em inglês (Conventional Commits).
- Antes de cada commit de código: `cargo fmt`, e `cargo clippy --manifest-path sudoku-core/Cargo.toml --all-targets -- -D warnings` limpo.
- Branch nova `feat/engine-dificil`, a partir de `master` atualizado. Por task, um commit de código e um `docs` com a página da wiki, indexada em `docs/wiki/README.md`.

## Review Focus

- **Vazamento para o Médio:** puzzles Médios de várias seeds nunca usam técnica de Difícil e continuam com `rate == Medio`. Teste na Task 5.
- **Par disfarçado de triple:** duas células com {1,2} e uma terceira cheia não formam Naked Triple; isso é um Naked Pair. Teste na Task 2.
- **Dígito já colocado na unidade:** Hidden Triple não pode contar um dígito sem posições (já colocado) como parte de um triple, ou removeria candidatos verdadeiros. Teste na Task 3.
- **X-Wing com um dígito em 3 colunas numa das linhas:** não é X-Wing, devolve `None`. Teste na Task 4.
- **Seeds nos extremos** (`0` e `u64::MAX`) em Difícil: devolvem um puzzle válido, sem panic. Teste na Task 5.

---

### Task 1: Box/Line Reduction

**Files:**
- Create: `sudoku-core/src/techniques/box_line_reduction.rs`, `docs/wiki/22-box-line-reduction.md`
- Modify: `sudoku-core/src/techniques/mod.rs`, `sudoku-core/src/solver.rs`, `sudoku-core/src/lib.rs`, `docs/wiki/README.md`

**Interfaces:**
- Consumes: `Technique`, `Action`, `Deduction`, `units()`, `CandidateGrid::{get, eliminate}`, `box_of/row_of/col_of`.
- Produces: `pub struct BoxLineReduction`, `technique: "Box/Line Reduction"`, `level: Difficulty::Dificil`. Exportado em `techniques` e `lib.rs`; adicionado ao fim de `all_techniques()`.

- [ ] **Step 1: Criar a branch `feat/engine-dificil` a partir de `master` atualizado**

- [ ] **Step 2: Gravar a referência de Fácil e Médio, antes de mudar qualquer código**

Teste temporário `sudoku-core/tests/referencia.rs`, **sem commit**, `#[ignore]`, que imprime `givens.to_line()` de `generate(Facil, s)` e `generate(Medio, s)` para `s in 0..50`.

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test referencia -- --ignored --nocapture > .superpowers/referencia.txt`
Expected: 100 linhas de 81 caracteres. Comparado nas Tasks 4 e 5.

- [ ] **Step 3: Escrever os testes que falham**

Montagem: `CandidateGrid::from_grid(&Grid::empty())`, eliminando o 5 da linha 0 fora das células pedidas.

```rust
#[test] fn elimina_o_digito_do_resto_do_box() {
    // 5 só em [0, 1] na linha 0 → sai de 9, 10, 11, 18, 19, 20
    // esperado: [(9,5),(10,5),(11,5),(18,5),(19,5),(20,5)]; technique "Box/Line Reduction"; level Dificil
}
#[test] fn funciona_em_coluna() { /* 5 só em [0, 9] na coluna 0 → sai de 1, 2, 10, 11, 19, 20 */ }
#[test] fn retorna_none_sem_alinhamento_no_box() { /* 5 só em [0, 3] na linha 0 */ }
#[test] fn retorna_none_com_uma_celula_so() { /* 5 só em [0] na linha 0 */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* [0, 1] e o 5 já fora do resto do box 0 */ }
```

Atenção: na montagem de `elimina_o_digito_do_resto_do_box`, o box 0 inteiro ainda tem o 5 nas linhas 1 e 2, então a técnica antiga **Pointing Pair** não se aplica ao box 0; o teste chama `BoxLineReduction.apply` direto.

- [ ] **Step 4: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml box_line`
Expected: erro de compilação, `BoxLineReduction` não existe.

- [ ] **Step 5: Implementar `BoxLineReduction`**

Para `value in 1..=9`, para cada uma das 18 primeiras unidades de `units()` (linhas, depois colunas): células da unidade com `value` candidato; só segue com 2 ou 3; se todas estão no mesmo box, elimina `value` das células desse box fora da unidade que ainda o têm. Primeira lista não vazia vira a dedução.

Adicionar ao fim de `all_techniques()`, exportar em `techniques/mod.rs` e `lib.rs`.

- [ ] **Step 6: Rodar e ver passar, mais a suíte inteira**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: tudo verde, incluindo os dois golden tests.

- [ ] **Step 7: Commits**

```bash
git commit -m "feat: add Box/Line Reduction technique"
git commit -m "docs: add wiki page on Box/Line Reduction"
```

---

### Task 2: Naked Triple

**Files:**
- Create: `sudoku-core/src/techniques/naked_triple.rs`, `docs/wiki/23-naked-triple-e-combinacoes.md`
- Modify: `sudoku-core/src/candidates.rs`, `techniques/mod.rs`, `solver.rs`, `lib.rs`, `docs/wiki/README.md`

**Interfaces:**
- Produces: `CandidateSet::union(self, other: CandidateSet) -> CandidateSet`; `pub struct NakedTriple`, `technique: "Naked Triple"`, `level: Dificil`, no fim de `all_techniques()`.

- [ ] **Step 1: Testes que falham**

```rust
// candidates.rs
#[test] fn uniao_junta_os_candidatos() { /* {1,2} ∪ {2,3} == {1,2,3} */ }

// naked_triple.rs — linha 0, células 0, 1, 2 com {1,2}, {2,3}, {1,3}
#[test] fn elimina_os_tres_digitos_das_outras_celulas() {
    // esperado: (3..9).flat_map(|i| [(i,1),(i,2),(i,3)]); technique "Naked Triple"; level Dificil
}
#[test] fn aceita_celula_com_os_tres_candidatos() { /* {1,2,3}, {1,2}, {2,3} → mesmo resultado */ }
#[test] fn uniao_de_dois_digitos_nao_eh_triple() { /* {1,2}, {1,2}, {1,2,3,...} sem outras: devolve None */ }
#[test] fn retorna_none_sem_padrao() { /* grade vazia */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* 1, 2, 3 já fora do resto da linha 0 e do box 0 */ }
```

`uniao_de_dois_digitos_nao_eh_triple`: células 0 e 1 com {1,2}, célula 2 com tudo. Nenhuma combinação de 3 células com 2 ou 3 candidatos existe, então `None`. O Naked Pair cobre esse caso.

- [ ] **Step 2: Rodar e ver falhar** — `cargo test --manifest-path sudoku-core/Cargo.toml naked_triple` → erro de compilação.

- [ ] **Step 3: Implementar**

Para cada unidade: células com 2 ou 3 candidatos, na ordem da unidade; para cada combinação `a < b < c` delas, se a união tem exatamente 3 dígitos, elimina esses dígitos das outras células da unidade que os têm. Primeira lista não vazia vira a dedução. Sem alocar combinações: três laços aninhados sobre os índices.

- [ ] **Step 4: Suíte inteira verde** — `cargo test --manifest-path sudoku-core/Cargo.toml`.

- [ ] **Step 5: Commits**

```bash
git commit -m "feat: add Naked Triple technique"
git commit -m "docs: add wiki page on Naked Triple"
```

---

### Task 3: Hidden Triple

**Files:**
- Create: `sudoku-core/src/techniques/hidden_triple.rs`, `docs/wiki/24-hidden-triple.md`
- Modify: `techniques/mod.rs`, `solver.rs`, `lib.rs`, `docs/wiki/README.md`

**Interfaces:**
- Produces: `pub struct HiddenTriple`, `technique: "Hidden Triple"`, `level: Dificil`, no fim de `all_techniques()`.

- [ ] **Step 1: Testes que falham**

Montagem: na linha 0, os dígitos 1, 2 e 3 eliminados das células 3..9, ficando só em 0, 1 e 2 (que mantêm todos os candidatos).

```rust
#[test] fn remove_os_outros_candidatos_das_tres_celulas() {
    // esperado: [0,1,2].flat_map(|i| (4..=9).map(|v| (i,v))); technique "Hidden Triple"; level Dificil
}
#[test] fn aceita_digito_em_so_duas_das_tres_celulas() { /* além disso, 3 sai da célula 2 → mesmo resultado */ }
#[test] fn ignora_digito_ja_colocado() {
    // linha 0: 9 colocado na célula 8 (Grid e CandidateGrid::place); 1 só em 0 e 1; 2 só em 0, 1 e 2.
    // Sem a regra da máscara não vazia, {1, 2, 9} pareceria um triple em 0, 1, 2 e removeria
    // candidatos verdadeiros da célula 2. Esperado: None.
}
#[test] fn retorna_none_sem_padrao() { /* grade vazia */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* células 0, 1, 2 só com {1,2,3} */ }
```

- [ ] **Step 2: Rodar e ver falhar** — `cargo test --manifest-path sudoku-core/Cargo.toml hidden_triple` → erro de compilação.

- [ ] **Step 3: Implementar**

Máscaras de posição por unidade, como no `HiddenPair`. Para `a < b < c` em `1..=9`, cada um com máscara não vazia, se a união das três máscaras tem exatamente 3 bits, remove das 3 células os candidatos diferentes de `a`, `b`, `c`. Primeira lista não vazia vira a dedução.

- [ ] **Step 4: Suíte inteira verde.**

- [ ] **Step 5: Commits**

```bash
git commit -m "feat: add Hidden Triple technique"
git commit -m "docs: add wiki page on Hidden Triple"
```

---

### Task 4: X-Wing

**Files:**
- Create: `sudoku-core/src/techniques/x_wing.rs`, `docs/wiki/25-x-wing.md`, `docs/wiki/fundamentos/rust-base-tarefas-21-a-25.md`
- Modify: `techniques/mod.rs`, `solver.rs`, `lib.rs`, `docs/wiki/README.md`

**Interfaces:**
- Produces: `pub struct XWing`, `technique: "X-Wing"`, `level: Dificil`, no fim de `all_techniques()`.

- [ ] **Step 1: Testes que falham**

Montagem: o 5 eliminado das linhas 0 e 4, exceto nas colunas 1 e 7.

```rust
#[test] fn elimina_o_digito_das_colunas_nas_outras_linhas() {
    // esperado: colunas 1 e 7, linhas 1,2,3,5,6,7,8, em ordem crescente de célula:
    // [(10,5),(16,5),(19,5),(25,5),(28,5),(34,5),(46,5),(52,5),(55,5),(61,5),(64,5),(70,5),(73,5),(79,5)]
    // technique "X-Wing"; level Dificil
}
#[test] fn funciona_trocando_linha_por_coluna() { /* 5 só nas linhas 1 e 7 das colunas 0 e 4 → sai das linhas 1 e 7, demais colunas */ }
#[test] fn retorna_none_com_tres_colunas_numa_linha() { /* linha 4 com o 5 também na coluna 3 */ }
#[test] fn retorna_none_sem_padrao() { /* grade vazia */ }
#[test] fn retorna_none_quando_nao_ha_o_que_eliminar() { /* o 5 já fora das colunas 1 e 7 nas outras linhas */ }
```

Em `retorna_none_com_tres_colunas_numa_linha`, conferir que nenhum X-Wing em coluna aparece por acaso na montagem; se aparecer, ajuste a montagem, não a técnica.

- [ ] **Step 2: Rodar e ver falhar** — `cargo test --manifest-path sudoku-core/Cargo.toml x_wing` → erro de compilação.

- [ ] **Step 3: Implementar**

Para `value in 1..=9`: primeiro por linhas, depois por colunas. Para cada linha, a máscara das colunas onde `value` é candidato; para `r1 < r2` com máscaras iguais e de exatamente 2 bits, elimina `value` dessas 2 colunas nas outras linhas. A versão por colunas é a mesma trocando os papéis. Os pares são ordenados por célula antes de devolver.

- [ ] **Step 4: Suíte inteira verde e referência intacta**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`, depois a referência do Step 2 da Task 1 para um arquivo novo e `diff` com `.superpowers/referencia.txt`.
Expected: tudo verde; `diff` vazio.

- [ ] **Step 5: Commits**

```bash
git commit -m "feat: add X-Wing technique"
git commit -m "docs: add wiki page on X-Wing and consolidated doc (tasks 21-25)"
```

---

### Task 5: Gerador de Difícil

**Files:**
- Modify: `sudoku-core/src/generator.rs`, `rating.rs`, `solver.rs`, `tests/api.rs`, `tests/desempenho.rs`, `docs/architecture.md`, `docs/wiki/README.md`
- Create: `docs/wiki/26-gerador-dificil.md`

**Interfaces:**
- Consumes: `LogicalSolver::up_to(Difficulty::Dificil)`, `rate`, `dig`, `full_solution`.
- Produces: `generate(Difficulty::Dificil, seed)`; `const MAX_TENTATIVAS_DIFICIL: usize`.

- [ ] **Step 1: Testes que falham, em `generator.rs`**

```rust
#[test] fn gera_puzzle_dificil_valido_e_unico() { /* seed 42: difficulty, seed, count_solutions == 1, rate == Some(Dificil) */ }
#[test] fn mesma_seed_gera_mesmo_puzzle_dificil() { /* seed 7 */ }
#[test] fn seeds_extremas_geram_dificil_valido() { /* 0 e u64::MAX */ }
#[test] fn eliminacoes_do_dificil_nunca_removem_a_resposta() { /* seeds 1..=3, como o teste do Médio, exigindo ao menos um passo de nível Dificil */ }
#[test] fn medio_nunca_usa_tecnica_de_dificil() { /* seeds 0..5: todos os passos com level <= Medio; rate == Some(Medio) */ }
```

`niveis_nao_suportados_ainda_retornam_erro` passa a conferir só `MuitoDificil`.

- [ ] **Step 2: Rodar e ver falhar** — `cargo test --manifest-path sudoku-core/Cargo.toml generator` → os testes de Difícil falham com `Unsupported(Dificil)`.

- [ ] **Step 3: Implementar**

Ramo `Difficulty::Dificil` em `generate`, com o mesmo laço do Médio e `MAX_TENTATIVAS_DIFICIL`, valor provisório 200. O laço pode virar uma função comum `fn tentar(level, max, rng)`; o Médio continua com 50 tentativas e o mesmo consumo de RNG. Doc comment de `generate` e nota de `rate` atualizados.

- [ ] **Step 4: Medir antes de fixar o limite**

`dificil_cabe_na_meta_de_tempo` em `tests/desempenho.rs`, igual ao do Médio, imprimindo também o máximo de tentativas usado (pode ser medido com um teste temporário que replica o laço, sem commit).

Run: `cargo test --release --manifest-path sudoku-core/Cargo.toml --test desempenho -- --ignored --nocapture`
Expected: pior caso ≤ 1 s nas 200 seeds, nenhuma `AttemptsExhausted`. **Se falhar, parar e trazer mediana, pior caso, seeds acima de 1 s e tentativas.**

Fixar `MAX_TENTATIVAS_DIFICIL` em cerca de 4 vezes o máximo observado, arredondado, desde que o pior caso continue na meta.

- [ ] **Step 5: Golden test e fixtures**

- `puzzle_dificil_da_seed_42_e_reproduzivel`, com a saída real de `generate(Dificil, 42)`.
- `rating.rs`: `puzzle_que_exige_tecnica_de_dificil_eh_dificil`, com essa grade.
- `solver.rs`: `so_as_tecnicas_de_dificil_resolvem_o_puzzle`, com a mesma grade: `up_to(Medio)` empaca, `new()` resolve e chega à solução.
- `tests/api.rs`: `fluxo_completo_dificil`, seed 123, como o do Médio, exigindo um passo de nível `Dificil`.

- [ ] **Step 6: Verificação completa**

Run: fmt, clippy `-D warnings`, `cargo build --locked` e `cargo test --locked`, mais o `diff` da referência.
Expected: tudo verde, `diff` vazio. Apagar `tests/referencia.rs` e `.superpowers/referencia.txt`.

- [ ] **Step 7: Commits**

```bash
git commit -m "feat: generate Hard puzzles with bounded attempts"
git commit -m "docs: update architecture and add wiki page on Hard generation"
```
