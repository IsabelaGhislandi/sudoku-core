# 08 — Closures, `impl Fn` e `Option` como acumulador

> Referente à **Task 8** do plano. Commit: `feat: add HiddenSingle technique`.

A segunda técnica é o **Hidden Single**: um dígito que, dentro de uma **unidade** (linha,
coluna ou box), só tem **um lugar possível** — mesmo que aquela célula ainda aceite vários
outros dígitos. Implementá-la traz o conceito mais "funcional" do Rust até aqui: **closures**
passadas como argumento.

## Naked vs. Hidden Single

As duas técnicas olham para a mesma grade, mas por ângulos opostos:

| | Pergunta | Onde olha |
|---|---|---|
| **Naked Single** (Task 7) | "esta **célula** só aceita um dígito?" | uma célula |
| **Hidden Single** (Task 8) | "este **dígito** só cabe numa célula?" | uma unidade inteira |

O teste deixa isso explícito antes de chamar a técnica:

```rust
assert!(candidates_for(&grid, 0).count() > 1);
```

O índice 0 tem **vários** candidatos — então não é naked single. Ainda assim o 5 não cabe em
mais nenhuma célula da linha 0, e é isso que o Hidden Single enxerga. "Hidden" (escondido)
justamente porque a dedução está encoberta pelos outros candidatos da célula.

## Closures: funções como valor

```rust
sole_spot(grid, value, |i| row_of(i) == unit)
sole_spot(grid, value, |i| col_of(i) == unit)
sole_spot(grid, value, |i| box_of(i) == unit)
```

`|i| row_of(i) == unit` é uma **closure**: uma função anônima escrita na hora. A sintaxe
`|args| corpo` é o equivalente do `(i) => ...` do JS. Os tipos não aparecem porque o
compilador os deduz do uso.

A mesma função `sole_spot` serve para linha, coluna e box — a única coisa que muda é **como se
decide se um índice pertence à unidade**, e isso vira um parâmetro. Sem closures, seria
preciso ou três funções quase idênticas, ou um enum `Unit` com um `match` dentro do loop.

### "Closure" = fecha em cima do ambiente

O que diferencia uma closure de uma função comum é que ela **captura** variáveis do escopo
onde foi escrita. Aqui, `|i| row_of(i) == unit` usa `unit`, que é a variável do loop externo —
ela não é parâmetro da closure, veio "de fora". Cada volta do loop cria uma closure que
carrega o `unit` daquela volta. É desse "fechar em volta do ambiente" que vem o nome.

Como `unit` é um `usize` (tipo `Copy`), a captura é uma cópia barata — nada de borrow checker
reclamando de tempo de vida.

## `impl Fn(usize) -> bool` no parâmetro

```rust
fn sole_spot(grid: &Grid, value: u8, in_unit: impl Fn(usize) -> bool) -> Option<usize>
```

Toda closure em Rust tem um tipo **anônimo e único** — não dá para escrever
`in_unit: ClosureDeLinha`, porque esse nome não existe. O que se declara é a **trait** que ela
satisfaz:

- **`Fn(usize) -> bool`** — "algo chamável que recebe um `usize` e devolve `bool`".
- **`impl Trait` em posição de argumento** — açúcar para um genérico:
  `fn sole_spot<F: Fn(usize) -> bool>(..., in_unit: F)`. Diz "aceito **qualquer** tipo que
  implemente essa trait".

Isso é resolvido em **tempo de compilação**: o compilador gera uma cópia especializada de
`sole_spot` para cada closure passada e inlineia a chamada. Custo zero em runtime — não há
ponteiro de função nem alocação, diferente de um callback de JS.

### A família `Fn`, `FnMut`, `FnOnce`

| Trait | A closure… | Pode ser chamada |
|-------|-----------|------------------|
| `Fn` | só **lê** o que capturou | várias vezes |
| `FnMut` | **modifica** o que capturou | várias vezes |
| `FnOnce` | **consome** o que capturou | uma vez só |

Nossas closures só leem `unit` e são chamadas 81 vezes dentro do loop — por isso `Fn`, a mais
permissiva para quem chama. Regra prática: peça a menos restritiva que resolve
(`FnOnce` < `FnMut` < `Fn` em exigência sobre a closure).

## `Option` como acumulador de estado

```rust
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

`found` guarda três estados em uma variável só, sem flag booleana auxiliar:

1. `None` no fim do loop → **nenhum** lugar possível (não é dedução).
2. `Some(i)` no fim do loop → **exatamente um** lugar → é o Hidden Single.
3. Achou um segundo enquanto `found.is_some()` → **`return None` na hora**, curto-circuito.

Note que os casos 1 e 3 devolvem o mesmo `None`, e está certo: em ambos não há nada a deduzir.

- **`if !in_unit(i) { continue; }`** — *guard clause*: descarta cedo o que não interessa em vez
  de aninhar mais um `if` em volta do resto.
- **`found.is_some()`** — pergunta se o `Option` está preenchido sem desembrulhar o valor.
  O par é `is_none()`.
- Um detalhe que o código **não** precisa tratar: se o dígito já estiver preenchido na
  unidade, nenhuma célula vazia dela terá esse candidato (o `candidates_for` da Task 6 já
  exclui os usados), então `found` fica `None` naturalmente.

## Detalhes menores de sintaxe

- **`for value in 1..=9u8`** — o sufixo `u8` no literal fixa o tipo de todo o range. Sem ele o
  compilador inferiria `i32` e o `contains(value)` (que espera `u8`) não bateria.
- **`fn deduction(...)`** e **`fn sole_spot(...)`** são funções **livres** do módulo, fora de
  qualquer `impl`. Sem `pub`, são privadas ao arquivo `hidden_single.rs` — helpers internos que
  não poluem a API. A `deduction` existe só para não repetir o literal `"Hidden Single"` e o
  nível em três `return` diferentes.

## Sobre eficiência

`apply` percorre 9 dígitos × 9 unidades × 3 tipos, e cada `sole_spot` varre as 81 células
chamando `candidates_for`, que por sua vez varre 81 de novo. É bastante trabalho repetido —
mas o tabuleiro é minúsculo e fixo, e a prioridade agora é **clareza**. Otimizar antes de ter
o solver rodando seria otimização prematura; se um dia o gerador ficar lento, o lugar a mexer
é cachear os candidatos.

## Por que isso importa

`NakedSingle` e `HiddenSingle` agora implementam o **mesmo** contrato `Technique`. A Task 9
monta o `LogicalSolver`, que guarda uma lista dessas técnicas e aplica a primeira que
funcionar — e é aí que o `pub use` da Task 7 finalmente vai ser consumido, fazendo os warnings
de `unused import` desaparecerem.
