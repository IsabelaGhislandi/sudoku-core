# 23 — Naked Triple: combinações sem alocar e métodos que consomem `self`

> Referente à Task 2 do plano da fatia Difícil. Commit: `feat: add Naked Triple technique`.

Três células de uma unidade cujos candidatos, **juntos**, são exatamente 3 dígitos. Os três dígitos ficam com
essas três células, em alguma ordem, então saem de todas as outras células da unidade.

```
linha 0:  {1,2}  {2,3}  {1,3}  |  ...  as outras células perdem 1, 2 e 3
```

A pegadinha: nenhuma célula precisa ter os três dígitos. `{1,2}`, `{2,3}` e `{1,3}` formam um triple, mesmo sem
nenhum `{1,2,3}`.

## Combinações de 3 com três laços

```rust
for x in 0..celulas.len() {
    for y in (x + 1)..celulas.len() {
        for z in (y + 1)..celulas.len() {
            let trio = [celulas[x], celulas[y], celulas[z]];
```

- Começar cada laço **depois** do anterior (`x + 1`, `y + 1`) faz cada combinação aparecer uma vez só, e
  sempre na mesma ordem. Essa ordem fixa faz parte do determinismo do gerador.
- Nenhuma lista de combinações é montada: os índices andam, e `trio` é um array de 3 posições na pilha. Numa
  unidade de 9 células, são no máximo 84 combinações.
- Em JavaScript, a tentação seria uma função `combinations(arr, 3)` que devolve um array de arrays. Em Rust ela
  também existe (no crate `itertools`), mas aqui não vale uma dependência nova.

## `union(self, other)`: método que consome o valor

```rust
pub fn union(self, other: CandidateSet) -> CandidateSet {
    CandidateSet(self.0 | other.0)
}
```

O método recebe `self` **por valor**, e não `&self`. Normalmente isso "consumiria" o valor, que não poderia
mais ser usado depois. Mas `CandidateSet` é `Copy` (é só um `u16`), então o Rust copia o valor e o original
continua disponível. Isso permite encadear sem `&` nem `clone`:

```rust
let digitos = candidates.get(a).union(candidates.get(b)).union(candidates.get(c));
```

A união de bitmasks é um `|` (OU bit a bit): o dígito está no resultado se estiver em qualquer um dos lados.

## `contains` com referência

```rust
.filter(|i| !trio.contains(i))
```

Dentro do `filter` de um `iter()`, `i` já é `&usize`, que é exatamente o tipo que `contains` pede. Por isso aqui
não se usa o padrão `|&i|`: desestruturar e depois emprestar de novo seria trabalho à toa.

## Um par não é triple

Se duas células têm `{1,2}` e as outras estão cheias, só as duas do par são elegíveis (com 2 ou 3 candidatos), e
não há combinação de 3. Esse caso é do Naked Pair, que vem antes na lista. O teste
`uniao_de_dois_digitos_nao_eh_triple` deixa isso registrado.
