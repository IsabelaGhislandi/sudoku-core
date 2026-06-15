# 06 — Candidatos: bitmask `u16` e operações de bit

> Referente à **Task 6** do plano. Commit: `feat: add CandidateSet and candidates_for`.

"Candidatos" de uma célula vazia = os dígitos 1–9 que ainda **cabem** ali (não aparecem na
linha, coluna nem box). Em vez de um `Vec<u8>` ou `HashSet`, guardamos isso como um único
inteiro de 16 bits — um **bitmask**.

## A ideia do bitmask

```rust
pub struct CandidateSet(u16);
```

Cada **bit** do `u16` representa um dígito: bit 1 = dígito 1, bit 2 = dígito 2, … bit 9 =
dígito 9. Ligado (1) = é candidato; desligado (0) = não é. Os bits 0 e 10–15 ficam sem uso.

```
dígito:   9 8 7 6 5 4 3 2 1 0
bit:      1 1 1 1 1 1 1 1 1 0   = 0b11_1111_1110  (todos 1..=9 candidatos)
```

Vantagem: um conjunto de até 9 elementos cabe em **2 bytes**, e operações de conjunto viram
operações de bit (rápidas e sem alocação).

## `struct CandidateSet(u16)` — tuple struct

É uma **tuple struct**: uma struct cujos campos não têm nome, acessados por posição. O único
campo é `.0` (o `u16` interno). Ele é **privado**, então de fora ninguém mexe nos bits direto
— só pelos métodos. É o padrão **newtype**: embrulhar um tipo primitivo para dar a ele um
significado e uma API próprios.

## As operações de bit

| Operador | Nome | Faz |
|----------|------|-----|
| `1 << n` | shift left | cria uma máscara só com o bit `n` ligado |
| `a \| b` | OR | liga os bits presentes em `a` **ou** `b` (união) |
| `a & b` | AND | mantém só os bits em `a` **e** `b` (interseção) |
| `!a` | NOT | inverte todos os bits (complemento) |

### `contains` — o bit `value` está ligado?

```rust
pub fn contains(&self, value: u8) -> bool {
    self.0 & (1 << value) != 0
}
```

`1 << value` é uma máscara com só aquele bit. O `&` zera todo o resto; se sobrar algo
diferente de 0, o bit estava ligado.

### `count` — quantos candidatos?

```rust
pub fn count(&self) -> u32 {
    self.0.count_ones()
}
```

**`count_ones()`** é um método nativo dos inteiros: conta quantos bits estão em 1. (Muitas
CPUs fazem isso numa única instrução — `popcount`.) É o tamanho do conjunto de graça.

### `values` — listar os dígitos ligados

```rust
pub fn values(&self) -> Vec<u8> {
    (1..=9).filter(|&v| self.contains(v)).collect()
}
```

Itera 1..=9, mantém só os que `contains` aprova, e coleta num `Vec<u8>`. O **`|&v|`** no
filtro é desestruturação: o `filter` entrega `&u8`, e o `&v` já desreferencia para `u8`.

## `candidates_for` — montando o conjunto

```rust
pub fn candidates_for(grid: &Grid, index: usize) -> CandidateSet {
    if let Cell::Filled(_) = grid.get(index) {
        return CandidateSet(0);          // célula cheia: nenhum candidato
    }
    let (r, c, b) = (row_of(index), col_of(index), box_of(index));
    let mut used: u16 = 0;
    for i in 0..81 {
        if let Cell::Filled(v) = grid.get(i) {
            if row_of(i) == r || col_of(i) == c || box_of(i) == b {
                used |= 1 << v;          // marca o dígito v como "já usado"
            }
        }
    }
    let all: u16 = 0b11_1111_1110;       // bits 1..=9 ligados
    CandidateSet(all & !used)            // todos MENOS os usados
}
```

1. Se a célula já está preenchida, não há candidatos → conjunto vazio (`CandidateSet(0)`).
2. Senão, percorre o tabuleiro acumulando em `used` (com `|=`) todo dígito que aparece na
   mesma linha/coluna/box.
3. **`all & !used`** = "todos os dígitos" **interseção** "não-usados" = exatamente os que
   ainda cabem. Aqui o trio `&`, `!` e a máscara `all` trabalham juntos.

- **`0b11_1111_1110`** — literal **binário** (`0b`), com `_` só pra legibilidade (como vírgula
  de milhar). São os bits 1 a 9 ligados, o bit 0 desligado.
- **`used |= 1 << v`** — `|=` é "OR e atribui" (liga o bit `v` sem mexer nos outros), igual a
  `used = used | (1 << v)`.

## Por que isso importa

`candidates_for` é a base das **técnicas de resolução**: um *Naked Single* é uma célula com
exatamente **um** candidato (`count() == 1`); um *Hidden Single* é um dígito que só tem um
lugar possível numa unidade. As próximas tasks (7 e 8) constroem isso em cima daqui.
