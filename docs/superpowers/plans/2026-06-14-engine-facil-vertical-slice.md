# Engine — Fatia Vertical "Fácil" Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Entregar o crate Rust `sudoku-core` funcional ponta-a-ponta no nível Fácil: gera, resolve (por lógica), valida e classifica tabuleiros 9×9 de Sudoku.

**Architecture:** Biblioteca Rust pura, sem dependência de Tauri/UI/IO. Técnicas de resolução são "estratégias" intercambiáveis via trait `Technique` (pattern Strategy). O solver lógico aplica técnicas em ordem; um solver por backtracking garante unicidade; o gerador cava células mantendo solução única e solubilidade lógica (o que mantém o nível em Fácil).

**Tech Stack:** Rust (edition 2021), crate único `sudoku-core`, dependência única `rand` (RNG com seed). Testes com `cargo test`.

**Spec de referência:** `docs/superpowers/specs/2026-06-14-sudoku-engine-design.md`

**Convenção de comandos:** todos os comandos rodam a partir da raiz do repositório e usam `--manifest-path sudoku-core/Cargo.toml`, então funcionam independente do diretório atual.

**Convenção de testes:** testes unitários ficam inline em cada módulo, num bloco `#[cfg(test)] mod tests`. O teste de ponta-a-ponta da API pública fica em `sudoku-core/tests/`.

---

## Task 1: Scaffold do crate

**Files:**
- Create: `sudoku-core/Cargo.toml`
- Create: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Criar o crate**

Run: `cargo new --lib --vcs none sudoku-core`
Expected: cria `sudoku-core/Cargo.toml` e `sudoku-core/src/lib.rs` (com um teste de exemplo).

- [ ] **Step 2: Definir o Cargo.toml com a dependência `rand`**

Substitua o conteúdo de `sudoku-core/Cargo.toml` por:

```toml
[package]
name = "sudoku-core"
version = "0.1.0"
edition = "2021"

[dependencies]
rand = "0.8"
```

- [ ] **Step 3: Escrever um teste-fumaça em `src/lib.rs`**

Substitua o conteúdo de `sudoku-core/src/lib.rs` por:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        assert_eq!(2 + 2, 4);
    }
}
```

- [ ] **Step 4: Rodar os testes (e baixar o `rand`)**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: compila, baixa `rand`, e `test tests::smoke ... ok`.

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/Cargo.toml sudoku-core/src/lib.rs
git commit -m "chore: scaffold do crate sudoku-core"
```

---

## Task 2: Enum `Difficulty`

**Files:**
- Create: `sudoku-core/src/difficulty.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/difficulty.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Facil,
    Medio,
    Dificil,
    MuitoDificil,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordena_do_facil_ao_muito_dificil() {
        assert!(Difficulty::Facil < Difficulty::Medio);
        assert!(Difficulty::Medio < Difficulty::Dificil);
        assert!(Difficulty::Dificil < Difficulty::MuitoDificil);
    }

    #[test]
    fn max_pega_o_nivel_mais_alto() {
        let niveis = [Difficulty::Facil, Difficulty::Dificil, Difficulty::Medio];
        assert_eq!(niveis.iter().max().copied(), Some(Difficulty::Dificil));
    }
}
```

Adicione no topo de `sudoku-core/src/lib.rs` (substituindo o conteúdo atual):

```rust
mod difficulty;
```

- [ ] **Step 2: Rodar e ver compilar/passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml difficulty`
Expected: PASS (`ordena_do_facil_ao_muito_dificil` e `max_pega_o_nivel_mais_alto`).

> Nota: a derivação `Ord` segue a ordem de declaração das variantes, por isso a ordenação Fácil < ... < MuitoDificil funciona sem código extra.

- [ ] **Step 3: Commit**

```bash
git add sudoku-core/src/difficulty.rs sudoku-core/src/lib.rs
git commit -m "feat: enum Difficulty ordenável"
```

---

## Task 3: `Grid` e `Cell` (base)

**Files:**
- Create: `sudoku-core/src/grid.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/grid.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Filled(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    cells: [Cell; 81],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_comeca_toda_vazia() {
        let grid = Grid::empty();
        assert!((0..81).all(|i| grid.get(i) == Cell::Empty));
        assert!(!grid.is_complete());
    }

    #[test]
    fn set_e_get_de_uma_celula() {
        let mut grid = Grid::empty();
        grid.set(40, Cell::Filled(7));
        assert_eq!(grid.get(40), Cell::Filled(7));
        assert_eq!(grid.get(0), Cell::Empty);
    }

    #[test]
    fn is_complete_quando_todas_preenchidas() {
        let mut grid = Grid::empty();
        for i in 0..81 {
            grid.set(i, Cell::Filled(1));
        }
        assert!(grid.is_complete());
    }
}
```

Adicione a declaração do módulo em `sudoku-core/src/lib.rs`:

```rust
mod grid;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: FAIL — erro de compilação ("no function or associated item named `empty`").

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/grid.rs`, logo abaixo da struct `Grid` (antes do bloco de testes):

```rust
impl Grid {
    pub fn empty() -> Self {
        Grid { cells: [Cell::Empty; 81] }
    }

    pub fn get(&self, index: usize) -> Cell {
        self.cells[index]
    }

    pub fn set(&mut self, index: usize, cell: Cell) {
        self.cells[index] = cell;
    }

    pub fn is_complete(&self) -> bool {
        self.cells.iter().all(|c| matches!(c, Cell::Filled(_)))
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: PASS (3 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/grid.rs sudoku-core/src/lib.rs
git commit -m "feat: Grid e Cell com empty/get/set/is_complete"
```

---

## Task 4: Parsing e serialização do `Grid`

**Files:**
- Modify: `sudoku-core/src/grid.rs`

- [ ] **Step 1: Escrever o teste que falha**

Adicione ao bloco `#[cfg(test)] mod tests` de `sudoku-core/src/grid.rs`:

```rust
    #[test]
    fn from_line_aceita_ponto_e_zero_como_vazio() {
        let grid = Grid::from_line(&".".repeat(81)).unwrap();
        assert!((0..81).all(|i| grid.get(i) == Cell::Empty));
    }

    #[test]
    fn from_line_le_digitos() {
        let mut line = String::from("53..7....");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();
        assert_eq!(grid.get(0), Cell::Filled(5));
        assert_eq!(grid.get(1), Cell::Filled(3));
        assert_eq!(grid.get(2), Cell::Empty);
        assert_eq!(grid.get(4), Cell::Filled(7));
    }

    #[test]
    fn from_line_rejeita_tamanho_errado() {
        assert_eq!(Grid::from_line("123"), Err(ParseError::WrongLength(3)));
    }

    #[test]
    fn from_line_rejeita_char_invalido() {
        let line = "x".to_string() + &".".repeat(80);
        assert_eq!(Grid::from_line(&line), Err(ParseError::InvalidChar('x')));
    }

    #[test]
    fn to_line_faz_roundtrip() {
        let mut line = String::from("534678912");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();
        assert_eq!(grid.to_line(), line);
    }
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: FAIL — "cannot find type `ParseError`" / "no function `from_line`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/grid.rs` (acima do bloco de testes):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    WrongLength(usize),
    InvalidChar(char),
}

impl Grid {
    pub fn from_line(line: &str) -> Result<Grid, ParseError> {
        let chars: Vec<char> = line.chars().collect();
        if chars.len() != 81 {
            return Err(ParseError::WrongLength(chars.len()));
        }
        let mut grid = Grid::empty();
        for (i, ch) in chars.into_iter().enumerate() {
            let cell = match ch {
                '.' | '0' => Cell::Empty,
                '1'..='9' => Cell::Filled(ch as u8 - b'0'),
                other => return Err(ParseError::InvalidChar(other)),
            };
            grid.set(i, cell);
        }
        Ok(grid)
    }

    pub fn to_line(&self) -> String {
        self.cells
            .iter()
            .map(|c| match c {
                Cell::Empty => '.',
                Cell::Filled(n) => (b'0' + *n) as char,
            })
            .collect()
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: PASS (todos os testes de grid).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/grid.rs
git commit -m "feat: parsing/serialização do Grid (from_line/to_line)"
```

---

## Task 5: Helpers de unidade, `can_place` e `validate`

**Files:**
- Modify: `sudoku-core/src/grid.rs`

- [ ] **Step 1: Escrever o teste que falha**

Adicione ao bloco de testes de `sudoku-core/src/grid.rs`:

```rust
    #[test]
    fn helpers_de_linha_coluna_box() {
        assert_eq!((row_of(0), col_of(0), box_of(0)), (0, 0, 0));
        assert_eq!((row_of(80), col_of(80), box_of(80)), (8, 8, 8));
        assert_eq!((row_of(40), col_of(40), box_of(40)), (4, 4, 4));
        // index 3 = linha 0, coluna 3 => box 1
        assert_eq!(box_of(3), 1);
    }

    #[test]
    fn can_place_respeita_linha_coluna_box() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5)); // linha 0, coluna 0, box 0
        assert!(!grid.can_place(1, 5)); // mesma linha
        assert!(!grid.can_place(9, 5)); // mesma coluna
        assert!(!grid.can_place(10, 5)); // mesmo box
        assert!(grid.can_place(80, 5)); // longe: ok
        assert!(grid.can_place(1, 6)); // outro valor: ok
    }

    #[test]
    fn validate_detecta_conflito() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(1, Cell::Filled(5)); // mesma linha => conflito
        let conflitos = validate(&grid);
        assert_eq!(conflitos, vec![Conflict { a: 0, b: 1 }]);
    }

    #[test]
    fn validate_sem_conflito_retorna_vazio() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        grid.set(1, Cell::Filled(6));
        assert!(validate(&grid).is_empty());
    }
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: FAIL — "cannot find function `row_of`" / "cannot find type `Conflict`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/grid.rs` (acima do bloco de testes):

```rust
pub fn row_of(index: usize) -> usize {
    index / 9
}

pub fn col_of(index: usize) -> usize {
    index % 9
}

pub fn box_of(index: usize) -> usize {
    (row_of(index) / 3) * 3 + col_of(index) / 3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Conflict {
    pub a: usize,
    pub b: usize,
}

impl Grid {
    /// True se colocar `value` em `index` não viola linha/coluna/box.
    pub fn can_place(&self, index: usize, value: u8) -> bool {
        let (r, c, b) = (row_of(index), col_of(index), box_of(index));
        for i in 0..81 {
            if i == index {
                continue;
            }
            if let Cell::Filled(v) = self.cells[i] {
                if v == value && (row_of(i) == r || col_of(i) == c || box_of(i) == b) {
                    return false;
                }
            }
        }
        true
    }
}

/// Lista pares de células preenchidas que conflitam (mesmo valor na linha/coluna/box).
pub fn validate(grid: &Grid) -> Vec<Conflict> {
    let mut out = Vec::new();
    for i in 0..81 {
        if let Cell::Filled(vi) = grid.get(i) {
            for j in (i + 1)..81 {
                if let Cell::Filled(vj) = grid.get(j) {
                    if vi == vj
                        && (row_of(i) == row_of(j)
                            || col_of(i) == col_of(j)
                            || box_of(i) == box_of(j))
                    {
                        out.push(Conflict { a: i, b: j });
                    }
                }
            }
        }
    }
    out
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml grid`
Expected: PASS (todos os testes de grid).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/grid.rs
git commit -m "feat: helpers de unidade, can_place e validate"
```

---

## Task 6: Candidatos (`CandidateSet` + `candidates_for`)

**Files:**
- Create: `sudoku-core/src/candidates.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/candidates.rs`:

```rust
use crate::grid::{box_of, col_of, row_of, Cell, Grid};

/// Bitmask dos dígitos candidatos 1..=9 (bit n ligado => dígito n é candidato).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateSet(u16);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn candidatos_de_celula_vazia() {
        let mut grid = Grid::empty();
        grid.set(1, Cell::Filled(1)); // linha 0
        grid.set(2, Cell::Filled(3)); // linha 0 e box 0
        grid.set(9, Cell::Filled(2)); // coluna 0 e box 0
        let cands = candidates_for(&grid, 0);
        assert_eq!(cands.values(), vec![4, 5, 6, 7, 8, 9]);
        assert_eq!(cands.count(), 6);
        assert!(cands.contains(4));
        assert!(!cands.contains(1));
    }

    #[test]
    fn celula_preenchida_nao_tem_candidatos() {
        let mut grid = Grid::empty();
        grid.set(0, Cell::Filled(5));
        assert_eq!(candidates_for(&grid, 0).count(), 0);
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod candidates;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml candidates`
Expected: FAIL — "no method named `values`" / "cannot find function `candidates_for`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/candidates.rs` (entre a struct e o bloco de testes):

```rust
impl CandidateSet {
    pub fn contains(&self, value: u8) -> bool {
        self.0 & (1 << value) != 0
    }

    pub fn values(&self) -> Vec<u8> {
        (1..=9).filter(|&v| self.contains(v)).collect()
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }
}

/// Candidatos de uma célula vazia = dígitos não vistos na sua linha, coluna ou box.
/// Retorna conjunto vazio para células já preenchidas.
pub fn candidates_for(grid: &Grid, index: usize) -> CandidateSet {
    if let Cell::Filled(_) = grid.get(index) {
        return CandidateSet(0);
    }
    let (r, c, b) = (row_of(index), col_of(index), box_of(index));
    let mut used: u16 = 0;
    for i in 0..81 {
        if let Cell::Filled(v) = grid.get(i) {
            if row_of(i) == r || col_of(i) == c || box_of(i) == b {
                used |= 1 << v;
            }
        }
    }
    let all: u16 = 0b11_1111_1110; // bits 1..=9
    CandidateSet(all & !used)
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml candidates`
Expected: PASS (2 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/candidates.rs sudoku-core/src/lib.rs
git commit -m "feat: CandidateSet e candidates_for"
```

---

## Task 7: Trait `Technique`, `Deduction` e `NakedSingle`

**Files:**
- Create: `sudoku-core/src/techniques/mod.rs`
- Create: `sudoku-core/src/techniques/naked_single.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/techniques/mod.rs`:

```rust
use crate::difficulty::Difficulty;
use crate::grid::Grid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduction {
    pub index: usize,
    pub value: u8,
    pub technique: &'static str,
    pub level: Difficulty,
}

/// Uma técnica de resolução (pattern Strategy): olha a grade e devolve, se
/// existir, uma jogada logicamente dedutível.
pub trait Technique {
    fn apply(&self, grid: &Grid) -> Option<Deduction>;
}

mod naked_single;

pub use naked_single::NakedSingle;
```

Crie `sudoku-core/src/techniques/naked_single.rs`:

```rust
use super::{Deduction, Technique};
use crate::candidates::candidates_for;
use crate::difficulty::Difficulty;
use crate::grid::{Cell, Grid};

pub struct NakedSingle;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    #[test]
    fn deduz_celula_com_unico_candidato() {
        // Linha 0: índices 1..8 = 1,2,3,4,5,6,7,8 => índice 0 só pode ser 9.
        let mut line = String::from(".12345678");
        line.push_str(&".".repeat(72));
        let grid = Grid::from_line(&line).unwrap();

        let d = NakedSingle.apply(&grid).unwrap();
        assert_eq!(d.index, 0);
        assert_eq!(d.value, 9);
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        assert_eq!(NakedSingle.apply(&grid), None);
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod techniques;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml naked_single`
Expected: FAIL — "the trait `Technique` is not implemented for `NakedSingle`" / método `apply` ausente.

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/techniques/naked_single.rs` (entre a struct e o bloco de testes):

```rust
impl Technique for NakedSingle {
    fn apply(&self, grid: &Grid) -> Option<Deduction> {
        for index in 0..81 {
            if let Cell::Empty = grid.get(index) {
                let cands = candidates_for(grid, index);
                if cands.count() == 1 {
                    let value = cands.values()[0];
                    return Some(Deduction {
                        index,
                        value,
                        technique: "Naked Single",
                        level: Difficulty::Facil,
                    });
                }
            }
        }
        None
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml naked_single`
Expected: PASS (2 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/techniques/ sudoku-core/src/lib.rs
git commit -m "feat: trait Technique, Deduction e NakedSingle"
```

---

## Task 8: Técnica `HiddenSingle`

**Files:**
- Create: `sudoku-core/src/techniques/hidden_single.rs`
- Modify: `sudoku-core/src/techniques/mod.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/techniques/hidden_single.rs`:

```rust
use super::{Deduction, Technique};
use crate::candidates::candidates_for;
use crate::difficulty::Difficulty;
use crate::grid::{box_of, col_of, row_of, Cell, Grid};

pub struct HiddenSingle;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidates::candidates_for;
    use crate::grid::Grid;

    #[test]
    fn deduz_unico_lugar_do_digito_na_linha() {
        // Coloca 5 em cada coluna 1..8 (linhas/boxes distintos), de modo que,
        // na linha 0, o dígito 5 só caiba no índice 0 — mas o índice 0 ainda
        // tem vários candidatos (logo NÃO é naked single).
        let mut grid = Grid::empty();
        for index in [12, 24, 28, 40, 52, 56, 68, 80] {
            grid.set(index, Cell::Filled(5));
        }
        // confirma que é "hidden" e não "naked": índice 0 tem >1 candidato
        assert!(candidates_for(&grid, 0).count() > 1);

        let d = HiddenSingle.apply(&grid).unwrap();
        assert_eq!(d.index, 0);
        assert_eq!(d.value, 5);
        assert_eq!(d.level, Difficulty::Facil);
    }

    #[test]
    fn retorna_none_quando_nao_aplica() {
        let grid = Grid::empty();
        assert_eq!(HiddenSingle.apply(&grid), None);
    }
}
```

Adicione em `sudoku-core/src/techniques/mod.rs`, junto às outras declarações:

```rust
mod hidden_single;
```

e junto aos `pub use`:

```rust
pub use hidden_single::HiddenSingle;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml hidden_single`
Expected: FAIL — `Technique` não implementado para `HiddenSingle`.

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/techniques/hidden_single.rs` (entre a struct e o bloco de testes):

```rust
impl Technique for HiddenSingle {
    fn apply(&self, grid: &Grid) -> Option<Deduction> {
        for value in 1..=9u8 {
            for unit in 0..9 {
                if let Some(index) = sole_spot(grid, value, |i| row_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, value, |i| col_of(i) == unit) {
                    return Some(deduction(index, value));
                }
                if let Some(index) = sole_spot(grid, value, |i| box_of(i) == unit) {
                    return Some(deduction(index, value));
                }
            }
        }
        None
    }
}

fn deduction(index: usize, value: u8) -> Deduction {
    Deduction {
        index,
        value,
        technique: "Hidden Single",
        level: Difficulty::Facil,
    }
}

/// Retorna a única célula vazia da unidade que pode receber `value`, se existir
/// exatamente uma.
fn sole_spot(grid: &Grid, value: u8, in_unit: impl Fn(usize) -> bool) -> Option<usize> {
    let mut found: Option<usize> = None;
    for i in 0..81 {
        if !in_unit(i) {
            continue;
        }
        if let Cell::Empty = grid.get(i) {
            if candidates_for(grid, i).contains(value) {
                if found.is_some() {
                    return None; // mais de um lugar possível
                }
                found = Some(i);
            }
        }
    }
    found
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml hidden_single`
Expected: PASS (2 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/techniques/
git commit -m "feat: técnica Hidden Single"
```

---

## Task 9: `LogicalSolver` (solve + next_step)

**Files:**
- Create: `sudoku-core/src/solver.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/solver.rs`:

```rust
use crate::grid::{Cell, Grid};
use crate::techniques::{Deduction, HiddenSingle, NakedSingle, Technique};

pub struct LogicalSolver {
    techniques: Vec<Box<dyn Technique>>,
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub grid: Grid,
    pub steps: Vec<Deduction>,
    pub solved: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn resolve_grade_com_uma_celula_vazia() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        let result = LogicalSolver::new().solve(&grid);
        assert!(result.solved);
        assert_eq!(result.grid.to_line(), solucao());
        assert_eq!(result.steps.len(), 1);
    }

    #[test]
    fn empaca_quando_nao_ha_tecnica_aplicavel() {
        let grid = Grid::empty();
        let result = LogicalSolver::new().solve(&grid);
        assert!(!result.solved);
        assert!(result.steps.is_empty());
    }

    #[test]
    fn next_step_devolve_primeira_deducao() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        let step = LogicalSolver::new().next_step(&grid).unwrap();
        assert_eq!(step.index, 40);
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod solver;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml solver`
Expected: FAIL — "no function `new`" / "no method `solve`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/solver.rs` (entre `SolveResult` e o bloco de testes):

```rust
impl LogicalSolver {
    /// Solver com as técnicas disponíveis na fatia "Fácil".
    pub fn new() -> Self {
        LogicalSolver {
            techniques: vec![Box::new(NakedSingle), Box::new(HiddenSingle)],
        }
    }

    /// Primeira jogada logicamente dedutível, ou None se empacar.
    pub fn next_step(&self, grid: &Grid) -> Option<Deduction> {
        for technique in &self.techniques {
            if let Some(d) = technique.apply(grid) {
                return Some(d);
            }
        }
        None
    }

    /// Aplica técnicas repetidamente até resolver ou empacar.
    pub fn solve(&self, grid: &Grid) -> SolveResult {
        let mut work = grid.clone();
        let mut steps = Vec::new();
        loop {
            if work.is_complete() {
                return SolveResult { grid: work, steps, solved: true };
            }
            match self.next_step(&work) {
                Some(d) => {
                    work.set(d.index, Cell::Filled(d.value));
                    steps.push(d);
                }
                None => {
                    return SolveResult { grid: work, steps, solved: false };
                }
            }
        }
    }
}

impl Default for LogicalSolver {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml solver`
Expected: PASS (3 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/solver.rs sudoku-core/src/lib.rs
git commit -m "feat: LogicalSolver com solve e next_step"
```

---

## Task 10: Backtracking (`count_solutions`)

**Files:**
- Create: `sudoku-core/src/backtracking.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/backtracking.rs`:

```rust
use crate::grid::{Cell, Grid};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn grade_resolvida_tem_uma_solucao() {
        let grid = Grid::from_line(solucao()).unwrap();
        assert_eq!(count_solutions(&grid, 2), 1);
    }

    #[test]
    fn uma_celula_vazia_continua_unica() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        assert_eq!(count_solutions(&grid, 2), 1);
    }

    #[test]
    fn grade_vazia_tem_muitas_solucoes() {
        // limite 2: para assim que encontra a segunda
        assert_eq!(count_solutions(&Grid::empty(), 2), 2);
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod backtracking;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml backtracking`
Expected: FAIL — "cannot find function `count_solutions`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/backtracking.rs` (acima do bloco de testes):

```rust
/// Conta soluções até o limite `limit`. Use limit=2 para testar unicidade (== 1).
pub fn count_solutions(grid: &Grid, limit: usize) -> usize {
    let mut work = grid.clone();
    let mut count = 0;
    solve_recursive(&mut work, limit, &mut count);
    count
}

fn solve_recursive(grid: &mut Grid, limit: usize, count: &mut usize) {
    if *count >= limit {
        return;
    }
    let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
    match next {
        None => {
            *count += 1; // grade completa encontrada
        }
        Some(index) => {
            for value in 1..=9u8 {
                if grid.can_place(index, value) {
                    grid.set(index, Cell::Filled(value));
                    solve_recursive(grid, limit, count);
                    grid.set(index, Cell::Empty);
                    if *count >= limit {
                        return;
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml backtracking`
Expected: PASS (3 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/backtracking.rs sudoku-core/src/lib.rs
git commit -m "feat: count_solutions por backtracking (unicidade)"
```

---

## Task 11: `rate` (classificação de dificuldade)

**Files:**
- Create: `sudoku-core/src/rating.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/rating.rs`:

```rust
use crate::backtracking::count_solutions;
use crate::difficulty::Difficulty;
use crate::grid::Grid;
use crate::solver::LogicalSolver;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Cell, Grid};

    fn solucao() -> &'static str {
        "534678912672195348198342567859761423426853791713924856961537284287419635345286179"
    }

    #[test]
    fn puzzle_resolvivel_por_singles_eh_facil() {
        let mut grid = Grid::from_line(solucao()).unwrap();
        grid.set(40, Cell::Empty);
        assert_eq!(rate(&grid), Some(Difficulty::Facil));
    }

    #[test]
    fn grade_sem_solucao_unica_retorna_none() {
        assert_eq!(rate(&Grid::empty()), None);
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod rating;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml rating`
Expected: FAIL — "cannot find function `rate`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/rating.rs` (acima do bloco de testes):

```rust
/// Classifica um puzzle pela técnica mais avançada que sua resolução lógica exige.
/// Cai para MuitoDificil quando só o backtracking resolve.
/// Retorna None se a grade não tiver solução única.
///
/// Nota: nesta fatia só existem técnicas de nível Fácil, então puzzles que
/// exigem técnicas intermediárias aparecem como MuitoDificil. Planos futuros
/// (Médio/Difícil) refinam essa classificação ao adicionar mais técnicas.
pub fn rate(grid: &Grid) -> Option<Difficulty> {
    if count_solutions(grid, 2) != 1 {
        return None;
    }
    let result = LogicalSolver::new().solve(grid);
    if result.solved {
        let level = result
            .steps
            .iter()
            .map(|s| s.level)
            .max()
            .unwrap_or(Difficulty::Facil);
        Some(level)
    } else {
        Some(Difficulty::MuitoDificil)
    }
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml rating`
Expected: PASS (2 testes).

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/rating.rs sudoku-core/src/lib.rs
git commit -m "feat: rate (classificação por técnicas)"
```

---

## Task 12: Gerador (`generate`, `Puzzle`, `GenError`)

**Files:**
- Create: `sudoku-core/src/generator.rs`
- Modify: `sudoku-core/src/lib.rs`

- [ ] **Step 1: Escrever o teste que falha**

Crie `sudoku-core/src/generator.rs`:

```rust
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use crate::backtracking::count_solutions;
use crate::difficulty::Difficulty;
use crate::grid::{validate, Cell, Grid};
use crate::solver::LogicalSolver;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Puzzle {
    pub givens: Grid,
    pub solution: Grid,
    pub difficulty: Difficulty,
    pub seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenError {
    Unsupported(Difficulty),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gera_puzzle_facil_valido_e_unico() {
        let puzzle = generate(Difficulty::Facil, 42).unwrap();
        assert_eq!(puzzle.difficulty, Difficulty::Facil);
        assert_eq!(puzzle.seed, 42);
        // solução completa e consistente
        assert!(puzzle.solution.is_complete());
        assert!(validate(&puzzle.solution).is_empty());
        // o enunciado tem solução única e é classificado como Fácil
        assert_eq!(count_solutions(&puzzle.givens, 2), 1);
        assert_eq!(crate::rating::rate(&puzzle.givens), Some(Difficulty::Facil));
        // o enunciado removeu pelo menos algumas células
        let vazias = (0..81).filter(|&i| puzzle.givens.get(i) == Cell::Empty).count();
        assert!(vazias > 0);
    }

    #[test]
    fn mesma_seed_gera_mesmo_puzzle() {
        let a = generate(Difficulty::Facil, 7).unwrap();
        let b = generate(Difficulty::Facil, 7).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn seeds_diferentes_geram_puzzles_diferentes() {
        let a = generate(Difficulty::Facil, 1).unwrap();
        let b = generate(Difficulty::Facil, 2).unwrap();
        assert_ne!(a.solution, b.solution);
    }

    #[test]
    fn niveis_nao_suportados_ainda_retornam_erro() {
        assert_eq!(
            generate(Difficulty::Medio, 1),
            Err(GenError::Unsupported(Difficulty::Medio))
        );
    }
}
```

Adicione em `sudoku-core/src/lib.rs`:

```rust
mod generator;
```

- [ ] **Step 2: Rodar e ver falhar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml generator`
Expected: FAIL — "cannot find function `generate`".

- [ ] **Step 3: Implementar o mínimo**

Adicione em `sudoku-core/src/generator.rs` (entre `GenError` e o bloco de testes):

```rust
/// Gera um puzzle determinístico no nível pedido. Nesta fatia, só Fácil.
pub fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError> {
    if difficulty != Difficulty::Facil {
        return Err(GenError::Unsupported(difficulty));
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let solution = full_solution(&mut rng);
    let givens = dig_facil(&solution, &mut rng);
    Ok(Puzzle {
        givens,
        solution,
        difficulty: Difficulty::Facil,
        seed,
    })
}

/// Constrói uma grade completa e válida via backtracking randomizado.
fn full_solution(rng: &mut StdRng) -> Grid {
    let mut grid = Grid::empty();
    fill(&mut grid, rng);
    grid
}

fn fill(grid: &mut Grid, rng: &mut StdRng) -> bool {
    let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
    match next {
        None => true,
        Some(index) => {
            let mut values: Vec<u8> = (1..=9).collect();
            values.shuffle(rng);
            for value in values {
                if grid.can_place(index, value) {
                    grid.set(index, Cell::Filled(value));
                    if fill(grid, rng) {
                        return true;
                    }
                    grid.set(index, Cell::Empty);
                }
            }
            false
        }
    }
}

/// Remove células enquanto (a) a solução continua única e (b) o solver lógico
/// "Fácil" ainda resolve por completo — o que mantém a dificuldade em Fácil.
fn dig_facil(solution: &Grid, rng: &mut StdRng) -> Grid {
    let mut puzzle = solution.clone();
    let mut order: Vec<usize> = (0..81).collect();
    order.shuffle(rng);
    let solver = LogicalSolver::new();
    for index in order {
        let removed = puzzle.get(index);
        puzzle.set(index, Cell::Empty);
        let unique = count_solutions(&puzzle, 2) == 1;
        let logical = solver.solve(&puzzle).solved;
        if !(unique && logical) {
            puzzle.set(index, removed); // restaura
        }
    }
    puzzle
}
```

- [ ] **Step 4: Rodar e ver passar**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml generator`
Expected: PASS (4 testes).

> Nota: `rate` é usado no teste via `crate::rating::rate`; o módulo `rating` já está declarado em `lib.rs` desde a Task 11.

- [ ] **Step 5: Commit**

```bash
git add sudoku-core/src/generator.rs sudoku-core/src/lib.rs
git commit -m "feat: gerador determinístico de puzzles Fácil"
```

---

## Task 13: API pública e teste de integração

**Files:**
- Modify: `sudoku-core/src/lib.rs`
- Modify: `sudoku-core/src/solver.rs`
- Create: `sudoku-core/tests/api.rs`

- [ ] **Step 1: Escrever o teste de integração que falha**

Crie `sudoku-core/tests/api.rs` (usa só a API pública do crate):

```rust
use sudoku_core::{
    candidates_for, count_solutions, generate, next_hint, rate, validate, Cell, Difficulty,
};

#[test]
fn fluxo_completo_facil() {
    let puzzle = generate(Difficulty::Facil, 123).unwrap();

    // enunciado é único e Fácil
    assert_eq!(count_solutions(&puzzle.givens, 2), 1);
    assert_eq!(rate(&puzzle.givens), Some(Difficulty::Facil));
    assert!(validate(&puzzle.givens).is_empty());

    // a dica aponta uma jogada que bate com a solução
    let hint = next_hint(&puzzle.givens).expect("deve haver uma dica");
    assert_eq!(puzzle.solution.get(hint.index), Cell::Filled(hint.value));

    // candidates_for é acessível pela API pública
    let alguma_vazia = (0..81).find(|&i| puzzle.givens.get(i) == Cell::Empty).unwrap();
    assert!(candidates_for(&puzzle.givens, alguma_vazia).count() >= 1);
}
```

- [ ] **Step 2: Adicionar `next_hint` ao solver**

Adicione em `sudoku-core/src/solver.rs`, após o `impl Default for LogicalSolver`:

```rust
/// Próxima jogada logicamente dedutível — reutiliza as técnicas do solver.
/// Alimenta a feature de dica do módulo Game (futuro).
pub fn next_hint(grid: &Grid) -> Option<Deduction> {
    LogicalSolver::new().next_step(grid)
}
```

- [ ] **Step 3: Definir a API pública em `lib.rs`**

Substitua o conteúdo de `sudoku-core/src/lib.rs` por (mantém os módulos privados e re-exporta a superfície pública):

```rust
//! sudoku-core — geração, resolução, validação e classificação de Sudoku 9×9.
//! Biblioteca pura: sem UI, sem IO, sem Tauri.

mod backtracking;
mod candidates;
mod difficulty;
mod generator;
mod grid;
mod rating;
mod solver;
mod techniques;

pub use backtracking::count_solutions;
pub use candidates::{candidates_for, CandidateSet};
pub use difficulty::Difficulty;
pub use generator::{generate, GenError, Puzzle};
pub use grid::{validate, Cell, Conflict, Grid, ParseError};
pub use rating::rate;
pub use solver::{next_hint, LogicalSolver, SolveResult};
pub use techniques::{Deduction, HiddenSingle, NakedSingle, Technique};
```

- [ ] **Step 4: Rodar todos os testes do crate**

Run: `cargo test --manifest-path sudoku-core/Cargo.toml`
Expected: PASS — todos os testes unitários + `fluxo_completo_facil` (integração).

- [ ] **Step 5: Checar lints (informativo)**

Run: `cargo clippy --manifest-path sudoku-core/Cargo.toml`
Expected: compila. O clippy pode sugerir melhorias idiomáticas (ex.: trocar `for i in 0..81 { ...grid.get(i)... }` por iteradores) — são oportunidades de aprendizado, opcionais nesta fatia. (Se `clippy` não estiver instalado: `rustup component add clippy`.)

- [ ] **Step 6: Commit**

```bash
git add sudoku-core/src/lib.rs sudoku-core/src/solver.rs sudoku-core/tests/api.rs
git commit -m "feat: superfície pública do crate + next_hint + teste de integração"
```

---

## Resultado final desta fatia

Ao concluir, `sudoku-core` expõe a API: `generate`, `rate`, `validate`, `next_hint`,
`count_solutions`, `candidates_for`, mais os tipos `Grid`, `Cell`, `Difficulty`,
`Puzzle`, `Deduction`, `Conflict`, `CandidateSet`, e as técnicas `NakedSingle`/`HiddenSingle`
(via trait `Technique`).

**Próximos planos (fatias seguintes):**
1. Técnicas de nível Médio (Naked/Hidden Pairs, Pointing Pairs) + `dig`/`rate` parametrizados por nível.
2. Técnicas de nível Difícil (Triples, Box-Line Reduction, X-Wing).
3. Técnicas/“chute” de Muito Difícil (Swordfish, XY-Wing, fallback de backtracking).
4. `Deduction` evolui para suportar eliminações de candidatos (não só colocação de valor).
