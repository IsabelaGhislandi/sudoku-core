# 31 — XY-Wing: `let ... else`, pré-filtrar e uma função `sees`

> Referente à Task 2 do plano da fatia Muito Difícil. Commit: `feat: add XY-Wing technique`.

Três células com exatamente 2 candidatos cada formam um "Y":

```
pivô  {x, y}  ── enxerga ──  asa 1 {x, z}
   │
enxerga
   │
asa 2 {y, z}
```

O pivô vale x ou y. Se valer x, a asa 1 não pode ser x e vira **z**. Se valer y, a asa 2 vira **z**. Em qualquer
caso, uma das asas é z. Então z sai de toda célula que enxerga **as duas asas**: qualquer que seja a asa que vire
z, essa célula estaria na mesma unidade que ela.

As asas podem estar em unidades diferentes (uma na linha do pivô, outra no box) e não precisam se enxergar. Se se
enxergarem, elas mesmas também "enxergam as duas asas", mas não podem perder o z: o filtro
`i != wing_x && i != wing_y` existe para isso, e o teste `asas_que_se_enxergam_nao_perdem_o_digito` o prova.

## `sees`: uma regra do Sudoku numa linha

```rust
fn sees(a: usize, b: usize) -> bool {
    a != b && (row_of(a) == row_of(b) || col_of(a) == col_of(b) || box_of(a) == box_of(b))
}
```

`&&` e `||` são **curto-circuito**: se `a == b`, o lado direito nem é avaliado. A função é privada do módulo
(sem `pub`), porque só o XY-Wing precisa dela por enquanto.

## Pré-filtrar para cortar combinações

Um XY-Wing é um trio de células. Testar todos os trios seria 81 × 81 × 81, mais de 500 mil combinações, a cada
passo do solver. Mas só células **bivalores** (com 2 candidatos) participam, e elas são poucas:

```rust
let bivalue: Vec<usize> = (0..81)
    .filter(|&i| candidates.get(i).count() == 2)
    .collect();
for &pivot in &bivalue { ... }
```

O `Vec` é montado **uma vez por chamada**, fora dos laços. Montá-lo dentro de um laço seria o tipo de alocação
repetida que a página 26 mostrou ser caro.

`for &pivot in &bivalue` itera por referência (`&usize`) e o padrão `&pivot` já desfaz a referência, então `pivot`
é um `usize` comum. Em `bivalue.iter().filter(|&&w| ...)`, são duas camadas: `iter()` dá `&usize`, e `filter`
passa uma referência a esse item, `&&usize`. O padrão `&&w` desfaz as duas.

## `let ... else`: extrair ou pular

```rust
let mut digits = candidates.get(pivot).digits();
let (Some(x), Some(y)) = (digits.next(), digits.next()) else {
    continue;
};
```

`let ... else` tenta casar um padrão. Se casar, as variáveis (`x`, `y`) ficam disponíveis **no resto do bloco**. Se
não casar, o `else` roda, e ele é obrigado a sair dali (`continue`, `return`, `break` ou `panic!`). É o jeito de
dizer "pegue isso ou desista" sem aninhar um `if let` e empurrar o resto do código para a direita.

Aqui o `else` nunca roda, porque toda célula bivalor tem 2 dígitos. Mas o compilador não sabe disso, e o `let ...
else` deixa o caso impossível explícito sem `unwrap()`.

O mesmo padrão pega o z da asa: `let Some(z) = wing.digits().find(|&v| v != x) else { continue; };`. O `find`
devolve o primeiro dígito diferente de x, como `Option<u8>`.

## Os puzzles de teste

Os testes de solver e rating usam dois puzzles achados no experimento da spec: um que só sai com XY-Wing e outro
que exige Swordfish. O teste `swordfish_entra_na_resolucao` confere que o Swordfish, que só tinha testes com
candidatos montados à mão, também aparece numa resolução de verdade.

Um terceiro puzzle, de Arto Inkala, tem solução única mas nenhuma das técnicas implementadas o resolve. Ele
continua `MuitoDificil`, o topo da escala. Esse teste já passaria antes da task: ele é uma **trava** de um
comportamento que a spec decidiu manter, não a prova de um código novo.

> **Lição geral:** antes de combinar elementos em laços aninhados, filtre o que pode participar. O filtro quase
> sempre corta mais trabalho do que qualquer otimização dentro do laço.
