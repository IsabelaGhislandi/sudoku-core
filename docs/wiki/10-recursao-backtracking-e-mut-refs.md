# 10 — Recursão, backtracking e referências `&mut`

> Referente à **Task 10** do plano. Commit: `feat: add count_solutions via backtracking for uniqueness`.

Até aqui o crate só resolvia por **lógica**: se nenhuma técnica se aplica, o `LogicalSolver`
empaca. Agora entra a **força bruta**, que sempre chega ao fim — não para resolver puzzles
para o jogador, mas para responder uma pergunta que a lógica não responde: **este tabuleiro
tem exatamente uma solução?**

## Por que dois solvers?

| | `LogicalSolver` (Task 9) | `count_solutions` (Task 10) |
|---|---|---|
| Como | aplica técnicas dedutivas | tenta 1–9 e volta atrás |
| Pergunta que responde | "dá para resolver **pensando**?" | "**quantas** soluções existem?" |
| Serve para | dicas, passo-a-passo, classificar dificuldade | garantir unicidade do puzzle |
| Pode empacar | sim (`solved: false`) | não |

Um Sudoku de verdade precisa ter **solução única** — senão o jogador chuta e "acerta" um
tabuleiro diferente do gabarito. Nenhuma técnica lógica prova unicidade; só a busca exaustiva
prova. É por isso que os dois convivem, e o gerador (Task 12) vai usar os dois: o backtracking
para garantir que a solução é única, e o solver lógico para garantir que ela é **alcançável
por dedução** no nível Fácil.

## Backtracking: tentar, recursar, desfazer

```rust
for value in 1..=9u8 {
    if grid.can_place(index, value) {
        grid.set(index, Cell::Filled(value));   // 1. tenta
        solve_recursive(grid, limit, count);    // 2. explora o resto
        grid.set(index, Cell::Empty);           // 3. DESFAZ
        if *count >= limit {
            return;
        }
    }
}
```

O coração do algoritmo são essas três linhas. Escolhe-se a próxima célula vazia, tenta-se cada
dígito **legal** ali, e para cada tentativa resolve-se recursivamente o resto do tabuleiro.
Quando a recursão volta, a linha 3 **desfaz** a tentativa antes de testar o próximo dígito.

Sem o passo 3, o tabuleiro ficaria sujo com as tentativas fracassadas e os ramos seguintes da
busca partiriam de um estado errado. Esse "desfazer ao voltar" é literalmente o *backtrack* do
nome.

A recursão precisa de um **caso base**, e aqui ele é achar a grade cheia:

```rust
let next = (0..81).find(|&i| matches!(grid.get(i), Cell::Empty));
match next {
    None => *count += 1,     // não há célula vazia => é uma solução completa
    Some(index) => { /* ...tenta os dígitos... */ }
}
```

- **`.find(...)`** é um método de iterador: devolve `Option` com o **primeiro** elemento que
  satisfaz o predicado, ou `None`. Ele é *lazy* — para na primeira célula vazia, não percorre
  as 81 à toa.
- **`matches!(expr, Padrão)`** é uma macro que devolve `bool`: "esse valor casa com esse
  padrão?". É o atalho para `match expr { Padrão => true, _ => false }`, útil quando só
  interessa a variante e não os dados dela.
- Como `can_place` (Task 5) já filtra jogadas ilegais, chegar ao fim do tabuleiro significa
  necessariamente que todas as 81 casas estão preenchidas **e** válidas.

## `&mut`: emprestar para modificar

As assinaturas mudam de tom nesta task:

```rust
pub fn count_solutions(grid: &Grid, limit: usize) -> usize        // empresta só-leitura
fn solve_recursive(grid: &mut Grid, limit: usize, count: &mut usize)  // empresta p/ modificar
```

**`&T`** é um empréstimo imutável; **`&mut T`** é um empréstimo **exclusivo** que permite
mutar. A regra do borrow checker: podem existir **vários** `&T` ao mesmo tempo, **ou** um
único `&mut T`, nunca os dois. É isso que impede, em tempo de compilação, que dois pontos do
código mexam no mesmo dado sem coordenação.

Na recursão isso funciona bem porque o `&mut Grid` é repassado **um nível por vez**: enquanto
a chamada filha o tem, a mãe está parada esperando. Um único tabuleiro atravessa toda a árvore
de busca — nada de clonar 81 células a cada nó.

### Por que `count: &mut usize` em vez de um valor de retorno?

`solve_recursive` não retorna nada (`()`); ela **escreve** no contador compartilhado. Como o
mesmo contador é visto por todos os ramos da recursão, qualquer ramo pode ver que o limite já
foi atingido e abortar imediatamente. Com um retorno somado a posteriori, os ramos não teriam
como saber que o trabalho já acabou.

- **`*count += 1`** — o `*` é o **dereference**: `count` é um *ponteiro* para o número, e `*count`
  é o número em si. Ler também exige: `if *count >= limit`.
- **`&mut count`** na chamada de `count_solutions` cria o empréstimo mutável a partir da
  variável local `let mut count = 0;`.

## A poda pelo `limit`

```rust
if *count >= limit {
    return;
}
```

Essa checagem aparece **duas vezes**: na entrada da função e depois de cada recursão. É o que
transforma "conte todas as soluções" em "conte até `limit` e pare".

Isso não é detalhe de performance — é o que torna a função **utilizável**. Uma grade vazia tem
~6,67 × 10²¹ soluções; contar todas é impossível. Com `limit = 2` a pergunta vira decidível e
barata:

| Resultado de `count_solutions(g, 2)` | Significa |
|---|---|
| `0` | tabuleiro impossível (contradição) |
| `1` | **solução única** — puzzle válido |
| `2` | ambíguo (≥2 soluções) — cavou demais |

O teste `grade_vazia_tem_muitas_solucoes` espera exatamente `2`: não "duas soluções existem",
e sim "parei ao encontrar a segunda". É por isso que o parâmetro é um limite, e não um `bool`
de unicidade — a Task 12 vai chamar isso dentro do laço do gerador, muitas vezes.

## Mutabilidade em Rust, resumida

Três coisas diferentes usam a mesma palavra `mut`:

| Forma | Onde | Significa |
|---|---|---|
| `let mut x` | variável local | posso reatribuir/mutar **esta** variável |
| `&mut T` | parâmetro/empréstimo | empresto **exclusivamente** para quem for mutar |
| `&mut self` | método | o método muta a instância |

Note que `count_solutions` recebe `&Grid` (imutável) e faz `grid.clone()` antes de mutar — o
mesmo cuidado do `solve` na Task 9. Quem chama nunca tem seu tabuleiro alterado por baixo; a
mutação fica **contida** dentro da função.

## Por que isso importa

Com `count_solutions` no lugar, o crate consegue **validar** um puzzle. Falta classificá-lo:
a Task 11 usa o `LogicalSolver` para decidir o nível de dificuldade, e a Task 12 junta as
duas peças no gerador — cavar células enquanto a solução seguir única e o puzzle seguir
resolvível por lógica simples.
