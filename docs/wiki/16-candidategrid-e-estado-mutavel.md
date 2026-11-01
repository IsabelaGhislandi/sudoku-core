# 16 — `CandidateGrid`: quando um valor calculado vira estado

> Primeira task da fatia Médio. Spec:
> [`2026-11-01-engine-medio-design.md`](../superpowers/specs/2026-11-01-engine-medio-design.md).
> Commit: `feat: add CandidateGrid to keep candidates during a solve`.

Até aqui, os candidatos de uma célula eram **sempre calculados** a partir da grade, com
`candidates_for`. As técnicas de Médio quebram esse modelo: um Naked Pair conclui "o 5 não pode
ficar aqui" **sem colocar nenhum valor**. Essa conclusão não existe na grade e, se não for guardada,
some no passo seguinte.

## Derivado vs. estado

| | Valor derivado (`candidates_for`) | Estado (`CandidateGrid`) |
|---|---|---|
| Fonte da verdade | a grade | ele mesmo |
| Pode ficar desatualizado? | não | sim, se alguém esquecer de atualizar |
| Guarda eliminações? | não | sim |

Paralelo web: é a diferença entre um valor computado do Svelte (`$derived`) e um estado próprio
(`$state`). Prefira derivar sempre que puder, porque estado guardado é uma segunda fonte da verdade
que precisa ser mantida em dia.

Por isso existe um teste só para essa sincronia, `colocar_equivale_a_reconstruir_da_grade`. Ele
afirma que `place` produz exatamente o mesmo resultado que recalcular tudo, e é essa equivalência que
mantém os puzzles Fáceis idênticos aos de antes.

## `std::array::from_fn`

```rust
cells: std::array::from_fn(|i| candidates_for(grid, i)),
```

Monta um array de tamanho fixo chamando a closure uma vez por índice. O tamanho e o tipo vêm do
destino, o campo `[CandidateSet; 81]`, por inferência. Sem isso, seria preciso criar um array com um
valor inicial e sobrescrever em um laço, o que exige `mut`.

Como `[T; N]` não aloca nada no heap, um `CandidateGrid` ocupa 162 bytes (81 × 2) direto onde estiver
guardado.

## Método que muta: `&mut self`

```rust
pub fn remove(&mut self, value: u8) {
    self.0 &= !(1 << value);
}
```

`a & !b` é a **diferença de conjuntos** vista no bloco de bitmask: `1 << value` cria a máscara, `!`
inverte e `&=` desliga só aquele bit. `CandidateSet` é `Copy`, e isso traz uma armadilha:

```rust
let mut copia = self.cells[i];   // COPIA o valor
copia.remove(value);             // muda só a cópia; o array continua igual

self.cells[i].remove(value);     // muda o elemento do próprio array
```

Na segunda forma, o Rust faz o empréstimo mutável direto do elemento (*auto-ref*). Com tipos `Copy`,
atribuir a uma variável sempre cria um novo valor independente, sem nenhum aviso do compilador.

## Por que `place` passa pela própria célula

```rust
for i in 0..81 {
    if row_of(i) == r || col_of(i) == c || box_of(i) == b {
        self.cells[i].remove(value);
    }
}
self.cells[index] = CandidateSet(0);
```

A célula colocada também satisfaz a condição do laço, mas perder só aquele dígito não basta: ela está
preenchida e não pode ter candidato nenhum. Daí a última linha. É o mesmo contrato de
`candidates_for`, que devolve o conjunto vazio para células preenchidas.

## Por que isso importa

Essa é a base das próximas tasks. Na Task 17, o solver passa a manter um `CandidateGrid` e as técnicas
passam a recebê-lo, sem mudar nenhum puzzle Fácil.
