# 32 — Gerador de Muito Difícil: refinar em vez de recomeçar, e um `match` exaustivo

> Referente à Task 3 do plano da fatia Muito Difícil. Commit: `feat: generate Very Hard puzzles by refining near misses`.

O Muito Difícil tem duas regras: precisa de Swordfish ou XY-Wing, e começa com **no máximo 22 números**. A
primeira regra sai fácil. A segunda, não: o `dig` para quando nenhum número pode sair sem quebrar o puzzle, e isso
acontece com 22 a 27 números. No experimento da spec, o laço do Difícil (solução nova a cada tentativa) deixou 21
de 50 seeds sem puzzle em 500 tentativas.

## Refinar: busca local com estado mutável

Em vez de jogar fora um puzzle que parou em 24 números, o gerador o **perturba**: devolve 6 números da solução em
células vazias sorteadas e tenta tirar números de novo, agora só entre as células preenchidas.

```rust
for _ in 0..MAX_REFINEMENTS_VERY_HARD {
    let mut candidate = refill(&puzzle, &solution, REFILL_VERY_HARD, rng);
    redig(&mut candidate, &solver, rng);
    if accepts_very_hard(&candidate, &solver) {
        return Ok(found(candidate, solution));
    }
    if given_count(&candidate) <= given_count(&puzzle) {
        puzzle = candidate;
    }
}
```

- **`refill` devolve um `Grid` novo** e não mexe no original. Se o candidato sair pior, o puzzle atual continua
  intacto. `Grid` é um array de 81 células, então o `clone` é uma cópia barata na pilha, sem heap.
- **`redig(&mut candidate, ...)`** recebe um empréstimo mutável: altera o candidato no lugar, sem devolver nada.
- **`puzzle = candidate`** *move* o candidato para a variável do laço. O valor antigo de `puzzle` é descartado ali
  mesmo. É uma busca local: o estado só anda para um puzzle com o mesmo número de números ou menos.
- **`<=` e não `<`**: aceitar um empate deixa a busca "andar de lado" por puzzles diferentes com a mesma
  quantidade, em vez de ficar presa no primeiro.

Recavar testa só as cerca de 25 células preenchidas, e não as 81. Cada refinamento custa uma fração de uma
tentativa nova.

## Um laço para `dig` e `redig`

```rust
fn dig(solution: &Grid, solver: &LogicalSolver, rng: &mut ChaCha12Rng) -> Grid {
    let mut puzzle = solution.clone();
    remove_where_possible(&mut puzzle, (0..81).collect(), solver, rng);
    puzzle
}

fn redig(puzzle: &mut Grid, solver: &LogicalSolver, rng: &mut ChaCha12Rng) {
    let filled = (0..81).filter(|&i| puzzle.get(i) != Cell::Empty).collect();
    remove_where_possible(puzzle, filled, solver, rng);
}
```

O laço de remoção virou `remove_where_possible`, que recebe a lista de células a tentar. O `collect()` de `redig`
não diz o tipo: o Rust o deduz pelo parâmetro de `remove_where_possible`, que pede `Vec<usize>`.

**Cuidado com o determinismo:** o `dig` antigo embaralhava `(0..81)` e depois tentava cada célula. A versão nova
faz exatamente isso, na mesma ordem de uso do RNG. Se a extração tivesse mudado essa ordem, todos os puzzles de
todos os níveis mudariam. A referência de 150 puzzles Fáceis, Médios e Difíceis, gravada antes da fatia, saiu
idêntica, e os três golden tests passaram sem mudança.

## Uma closure que monta o resultado

```rust
let found = |givens: Grid, solution: Grid| Puzzle { givens, solution, difficulty, seed };
```

A closure **captura** `difficulty` e `seed` do ambiente e recebe o que muda. Os dois `return Ok(found(...))` ficam
curtos. Como `Difficulty` e `u64` são `Copy`, a captura copia os valores, e a closure pode ser chamada quantas
vezes for preciso.

## O `match` exaustivo a favor

Antes:

```rust
match difficulty {
    Difficulty::Facil => ...,
    Difficulty::Medio => ...,
    Difficulty::Dificil => ...,
    _ => Err(GenError::Unsupported(difficulty)),
}
```

Agora os quatro níveis têm braço próprio, e o `_` saiu junto com a variante `Unsupported`. Sem o `_`, o `match`
precisa ser **exaustivo**: se um dia surgir um quinto nível, o código deixa de compilar até alguém decidir como
gerá-lo. Com o `_`, o nível novo cairia em silêncio num erro genérico.

Remover a variante também mostrou um efeito em cascata: em `tests/desempenho.rs`, o braço `Err(e) => panic!(...)`
virou **inalcançável**, porque `AttemptsExhausted` passou a ser o único erro possível. O compilador avisou
(`unreachable pattern`), e o braço saiu. É o mesmo mecanismo do `match` exaustivo, trabalhando nos dois sentidos.

Remover uma variante pública quebra quem usa a API. Aqui foi seguro porque ainda não existe código fora do
`sudoku-core`: o módulo Game vem depois.

## Testes rápidos para o CI

O CI roda os testes em debug, que é bem mais lento que release. Uma medição temporária, em release, mostrou o
tempo de cada seed (de 23 ms a 1,4 s), e os testes usam as mais rápidas: 15 nos testes unitários e 14 no teste de
integração. Os testes conferem as regras do Muito Difícil uma a uma:

- no máximo 22 números, solução única, `rate == Some(MuitoDificil)`;
- o solver lógico resolve tudo e chega na `solution`, sem chute;
- todo número dado bate com a `solution`, o que protege o `refill` de usar a solução errada;
- nenhuma eliminação remove a resposta.

> **Lição geral:** quando recomeçar do zero é caro, guarde o que já deu certo e mude só um pouco. E, numa API,
> prefira um `match` sem `_`: o compilador passa a avisar de cada caso novo.
