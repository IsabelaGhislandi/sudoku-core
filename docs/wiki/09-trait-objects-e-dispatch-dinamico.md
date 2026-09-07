# 09 — Trait objects, `Box<dyn Trait>` e dispatch dinâmico

> Referente à **Task 9** do plano. Commit: `feat: add LogicalSolver with solve and next_step`.

O `LogicalSolver` guarda uma **lista de técnicas** e aplica a primeira que der resultado,
repetindo até a grade fechar ou empacar. Guardar tipos **diferentes** (`NakedSingle`,
`HiddenSingle`) na mesma `Vec` exige o outro lado das traits: o **trait object**.

## O problema: `Vec` é homogênea

```rust
techniques: Vec<Box<dyn Technique>>
```

Uma `Vec<T>` guarda elementos de **um único tipo** `T`, todos do mesmo tamanho em memória —
é assim que ela consegue indexar por deslocamento. Mas `NakedSingle` e `HiddenSingle` são
tipos distintos. `Vec<NakedSingle>` não aceita o outro; não existe um "tipo comum" entre eles.

O que eles têm em comum é a **trait**. `dyn Technique` significa "algum tipo, decidido em
runtime, que implementa `Technique`".

## Por que `Box`?

`dyn Technique` é um tipo **unsized**: o compilador não sabe quanto espaço ele ocupa (pode ser
uma struct vazia ou uma com 200 bytes de estado). E Rust precisa saber o tamanho de tudo que
vai na stack.

**`Box<T>`** resolve isso: é um ponteiro dono de um valor no **heap**. O `Box` em si tem tamanho
conhecido, então `Vec<Box<dyn Technique>>` é perfeitamente homogênea — uma lista de ponteiros.
Quando o `Box` é descartado, o valor no heap é liberado junto (RAII, sem GC).

Um `Box<dyn Trait>` é um **fat pointer**, dois ponteiros do tamanho de uma palavra:

```
Box<dyn Technique>  ─┬─→  o dado (a struct NakedSingle, no heap)
                     └─→  a vtable (onde fica o código de `apply` desse tipo)
```

## Dispatch estático vs. dinâmico

Vale comparar com o `impl Fn` da Task 8 — são as **duas** formas de polimorfismo do Rust:

| | `impl Trait` / genéricos | `dyn Trait` |
|---|---|---|
| Quando o tipo é decidido | compilação | runtime |
| Como | **monomorfização**: uma cópia do código por tipo | **vtable**: uma busca de ponteiro por chamada |
| Custo | zero, e pode inlinear | um salto indireto |
| Binário | cresce (uma cópia por tipo) | uma versão só |
| Serve para lista heterogênea? | não | **sim** |

Aqui o dinâmico é o certo: a lista de técnicas é heterogênea e vai **crescer** nas fatias
Médio/Difícil (X-Wing, Pointing Pairs…) sem que `LogicalSolver` mude uma linha. O custo do
salto indireto é irrelevante perto do trabalho que cada `apply` faz por dentro.

## `new()`: função associada, não método

```rust
impl LogicalSolver {
    pub fn new() -> Self {
        LogicalSolver {
            techniques: vec![Box::new(NakedSingle), Box::new(HiddenSingle)],
        }
    }
```

- Sem `self` no primeiro parâmetro → é uma **função associada** (o "método estático" de outras
  linguagens), chamada com `::` — `LogicalSolver::new()`. Com `&self`, seria um **método**,
  chamado com `.` sobre uma instância.
- **`Self`** (maiúsculo) é um apelido para o tipo do bloco `impl`. Dá para escrever
  `LogicalSolver` no lugar; `Self` é o idiomático e sobrevive a renomeações.
- **`Box::new(NakedSingle)`** — `NakedSingle` aqui é o **valor** da unit struct (Task 7), movido
  para o heap. A conversão de `Box<NakedSingle>` para `Box<dyn Technique>` é automática
  (*unsized coercion*), porque o tipo do `vec!` já está declarado no campo.
- `new()` **não** é palavra-chave nem construtor especial: é só uma convenção da comunidade.

## `impl Default`

```rust
impl Default for LogicalSolver {
    fn default() -> Self {
        Self::new()
    }
}
```

`Default` é a trait de "valor padrão óbvio" da std. A regra idiomática (e um lint do Clippy,
`new_without_default`): se um tipo tem `new()` sem argumentos, ele deveria implementar
`Default` também — assim ele funciona em qualquer lugar que peça um `T: Default`, como
`unwrap_or_default()`.

## `next_step`: iterar emprestando

```rust
pub fn next_step(&self, grid: &Grid) -> Option<Deduction> {
    for technique in &self.techniques {
        if let Some(d) = technique.apply(grid) {
            return Some(d);
        }
    }
    None
}
```

O **`&`** em `&self.techniques` é essencial: itera **emprestando** a `Vec`, e cada `technique`
é um `&Box<dyn Technique>`. Sem o `&`, o `for` **consumiria** a `Vec` (a moveria para dentro do
loop) — e o compilador barraria, já que ela é emprestada de `&self`, que não é dono.

A ordem do `vec!` é a ordem de tentativa: `NakedSingle` primeiro por ser a dedução mais barata
e mais simples de explicar numa dica. Trocar a prioridade das técnicas é só reordenar a lista.

## `solve`: clonar para não mutar o argumento

```rust
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
```

- **`grid.clone()`** — a assinatura recebe `&Grid` (empréstimo só-leitura), então não dá para
  mutar o original. `clone()` faz uma cópia própria, e quem chamou fica com a grade intacta.
  Isso vem do `#[derive(Clone)]` da Task 3; como `Grid` é um array fixo de 81 `Cell`, a cópia é
  barata e cabe na stack.
- **`loop`** é o laço infinito explícito do Rust (não `while true`). Sair dele é sempre
  intencional: aqui, dois `return` — grade completa (resolvida) ou nenhuma técnica aplicável
  (empacou).
- **`match self.next_step(&work)`** — trata os dois braços do `Option` de forma exaustiva. O
  compilador **exige** cobrir `Some` e `None`; esquecer um caso é erro de compilação, não um
  `undefined` silencioso.
- **`SolveResult { grid: work, steps, solved: true }`** — `steps` usa o *field init shorthand*
  (Task 7); `grid: work` precisa da forma completa porque a variável tem outro nome.
- `steps` acumula as deduções **na ordem em que foram aplicadas** — é o passo-a-passo que a UI
  vai usar para explicar a resolução, não só o resultado final.

## O que isso destrava

`solved: false` não é erro: significa "as técnicas desta fatia não bastam". É exatamente essa
informação que a **Task 11** (`rate`) usa para classificar a dificuldade, e que o **gerador**
(Task 12) usa para garantir que um puzzle "Fácil" seja resolvível só com lógica simples. Para
saber se um puzzle tem solução **única**, porém, lógica não basta — daí a Task 10, com
backtracking.

E, como previsto na Task 8, os warnings de `unused import` do `pub use` sumiram: o solver
finalmente consome `NakedSingle` e `HiddenSingle`.
