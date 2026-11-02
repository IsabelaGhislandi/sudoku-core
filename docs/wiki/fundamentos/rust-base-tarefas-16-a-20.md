# Fundamentos de Rust — consolidado das Tarefas 16 a 20

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 16 a
> 20. As páginas por-task ([16](../16-candidategrid-e-estado-mutavel.md) … [20](../20-pointing-pair.md)) contam a
> história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 11 a 15](rust-base-tarefas-11-a-15.md).

Nos blocos anteriores, modelamos dados, depois comportamento, e depois compusemos e protegemos o conjunto. Este
bloco trata de **evoluir um design que já existe**: acrescentar um conceito novo, a eliminação de candidatos, sem
mudar o que já funciona.

Sumário:
1. [Estado vs. valor derivado](#1-estado-vs-valor-derivado)
2. [Enums que carregam dados](#2-enums-que-carregam-dados)
3. [Padrões: referências, `match` e `let else`](#3-padrões-referências-match-e-let-else)
4. [Cadeias com `flat_map` e `move`](#4-cadeias-com-flat_map-e-move)
5. [Funções como valores](#5-funções-como-valores)
6. [`Option` encadeado](#6-option-encadeado)
7. [Conversão numérica e máscaras](#7-conversão-numérica-e-máscaras)
8. [Mudando sem quebrar](#8-mudando-sem-quebrar)
9. [Tabela-resumo dos conceitos](#9-tabela-resumo-dos-conceitos)

---

## 1. Estado vs. valor derivado

Candidatos eram **derivados** da grade. As técnicas de Médio removem candidatos sem colocar valor, informação que a
grade não guarda, e isso os transformou em **estado**.

Estado guardado é uma segunda fonte da verdade, que pode ficar desatualizada. Por isso existe um teste só para a
sincronia: `colocar_equivale_a_reconstruir_da_grade`. Prefira derivar e, ao guardar, teste a equivalência.

**Armadilha com `Copy`:** `let mut x = self.cells[i]; x.remove(v);` altera só a cópia. Já
`self.cells[i].remove(v)` altera o elemento.

## 2. Enums que carregam dados

```rust
pub enum Action {
    Place { index: usize, value: u8 },
    Eliminate(Vec<(usize, u8)>),
}
```

Cada variante tem os seus próprios dados, então estados sem sentido nem podem ser escritos. Com `Option`s soltos,
nada impediria uma dedução que coloca **e** elimina ao mesmo tempo. O equivalente em TypeScript é a *discriminated
union*.

Um `Vec` em uma variante tem uma consequência: o enum inteiro deixa de poder ser `Copy`.

## 3. Padrões: referências, `match` e `let else`

Sem `Copy`, um `match d.action` moveria a lista para fora de `d`. Daí o `match &d.action`, em que cada variável vira
uma referência:

```rust
match &d.action {
    Action::Place { index, value } => work.set(*index, Cell::Filled(*value)),
    Action::Eliminate(pairs) => { /* ... */ }
}
```

Em padrões, `&` faz o caminho inverso, **desembrulhando** a referência. O número de `&` segue o que a closure recebe:

| Trecho | Recebe | Padrão |
|---|---|---|
| `for ... in pairs` | `&(usize, u8)` | `&(index, value)` |
| `unit.iter().filter(...)` | `&&usize` | `&&i` |
| `.enumerate().flat_map(...)` | `(usize, &usize)` | `(_, &i)` |

`let ... else` desembrulha uma variante ou sai do bloco. O `else` precisa divergir, e é assim que o compilador garante
que a variável existe no restante da função.

## 4. Cadeias com `flat_map` e `move`

`flat_map` transforma cada item em vários, em um único iterador plano. A closure interna precisa de `move` porque
sobrevive à externa:

```rust
.flat_map(|&i| par.values().into_iter().map(move |v| (i, v)))
```

Sem `move`, ela guardaria uma referência a `i`, uma variável que já teria acabado.

## 5. Funções como valores

| Tipo | Aceita | Quando usar |
|---|---|---|
| `fn(usize) -> usize` | função comum, ou closure que não captura nada | basta passar uma função pronta |
| `impl Fn(usize) -> bool` | qualquer closure | a closure captura algo |
| `Box<dyn Technique>` | tipos diferentes juntos, em runtime | lista heterogênea |

`row_of` e `col_of` entram em `fora_do_box` como `fn`. As closures do `sole_spot` capturam `unit`, então usam `impl Fn`.

## 6. `Option` encadeado

| Método | Faz | Paralelo JS |
|---|---|---|
| `a.or_else(\|\| b())` | só calcula `b` se `a` for `None` | `a ?? b()` |
| `cond.then_some(x)` | `Some(x)` se verdadeiro; `x` já foi calculado | `cond ? x : null` |
| `cond.then(\|\| x())` | igual, mas só calcula se verdadeiro | `cond ? x() : null` |

## 7. Conversão numérica e máscaras

Rust nunca converte tipos numéricos sozinho. Índice é `usize`, daí o `v as usize`. Cuidado: `as` de um tipo maior para
um menor corta bits em silêncio.

Um inteiro pode representar qualquer conjunto pequeno. `CandidateSet` é o conjunto de **dígitos** de uma célula;
`posicoes[d]`, no Hidden Pair, é o conjunto de **células** de um dígito. Comparar dois conjuntos vira comparar dois
inteiros.

## 8. Mudando sem quebrar

Três técnicas do bloco protegeram o código existente:

- **Restringir o que muda:** `up_to(Difficulty::Facil)` mantém as técnicas novas fora do gerador de Fácil.
- **Invariante de progresso:** agora o solver pode dar um passo sem colocar valor. Uma técnica que respondesse sem
  mudar nada causaria loop infinito, então ela só responde se tiver algo a remover.
- **Comparar antes e depois:** o golden test só cobre a seed 42. Durante a mudança, comparamos os puzzles das seeds 0 a
  49 byte a byte.

Mudar a assinatura de uma trait quebra todo `impl`, e os erros do compilador viram a lista de tarefas.

## 9. Tabela-resumo dos conceitos

| Conceito | Essência | Exemplo nosso |
|----------|----------|---------------|
| Estado vs. derivado | guardar só o que não dá para recalcular | `CandidateGrid` |
| `array::from_fn` | monta array fixo por índice | `units()` |
| Enum com dados | cada variante com seus dados | `Action` |
| `match` em referência | olha sem mover | `match &d.action` |
| `&` em padrão | desembrulha a referência | `for &(i, v) in pairs` |
| `let else` | desembrulha ou diverge | testes de `NakedPair` |
| `flat_map` + `move` | um para muitos; closure leva cópia | pares de eliminação |
| `fn` vs. `impl Fn` | ponteiro de função vs. closure | `fora_do_box`, `sole_spot` |
| `or_else`, `then_some` | fluxo com `Option` | `PointingPair` |
| `as usize` | conversão sempre explícita | `posicoes[v as usize]` |
| Faixa com `contains` | faixa é um valor | `(2..=3).contains(...)` |
| Invariante de progresso | só responde se mudar algo | todas as técnicas de Médio |
