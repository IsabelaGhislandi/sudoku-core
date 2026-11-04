# 33 — Desempenho do Muito Difícil: medir, cachear com invariante e constantes como contrato

> Referente à Task 4 do plano da fatia Muito Difícil. Commit: `perf: speed up Very Hard generation and fix its constants`.

O gerador de Muito Difícil da Task 3 funcionava, mas a primeira medição em 200 seeds, em release, deu pior caso
de **5,7 s**, e a meta é 1 s. Esta task levou o pior caso a **0,79 s** sem mudar nenhum puzzle de Fácil, Médio ou
Difícil.

| | Mediana | p90 | Pior caso | Esgotadas |
|---|---|---|---|---|
| Linha de base (K = 6, 80 refinamentos, 20 soluções) | 652 ms | 2,04 s | 5,73 s | 0 |
| + `hardest_step` (singles em lote) | 195 ms | 597 ms | 1,56 s | 0 |
| + constantes novas (K = 8, 4 soluções) | 165 ms | 587 ms | 976 ms | 5 |
| + posições por unidade no `CandidateGrid` | 111 ms | 383 ms | 647 ms | 5 |
| **Final (5 soluções)** | **119 ms** | **411 ms** | **793 ms** | **1** |

## Medir antes de otimizar

O perfil usou o mesmo truque da wiki 26, com uma diferença: em vez de imprimir dentro do `apply`, os tempos
ficaram num acumulador por thread.

```rust
thread_local! {
    static PROFILE: RefCell<[(u128, u64); 11]> = const { RefCell::new([(0, 0); 11]) };
}
// em next_step, em volta de cada técnica:
PROFILE.with(|p| { let mut p = p.borrow_mut(); p[k].0 += t.elapsed().as_nanos(); p[k].1 += 1; });
```

- **`thread_local!`** cria uma variável global por thread. Como cada teste roda numa thread, os números de um
  teste não se misturam com os de outro, e não é preciso `Mutex`.
- **`RefCell`** dá mutabilidade interior: `with` só empresta `&RefCell`, e o `borrow_mut()` checa em tempo de
  execução que ninguém mais está mexendo.
- Era instrumentação **temporária**: saiu antes do commit, porque a biblioteca não lê relógio.

O perfil mostrou que nenhuma técnica sozinha dominava. O custo vinha de duas fontes: 1,46 milhão de passos de
single, cada um voltando ao começo da lista, e as cascatas de falha. Cada célula que o `redig` *não* consegue
tirar termina num empacamento que roda as nove técnicas, e quase todas recalculavam do zero as mesmas máscaras.

## Um caminho rápido com a mesma resposta

O gerador só pergunta duas coisas ao solver: "resolve?" e "qual o nível mais alto usado?". `solve` responde
montando a lista de passos, que o jogo vai usar nas dicas e por isso não pode mudar. A resposta foi um método
novo, visível só dentro da crate:

```rust
pub(crate) fn hardest_step(&self, grid: &Grid) -> Option<Difficulty> {
    // ...
    loop {
        place_singles(&mut work, &mut candidates);
        if work.is_complete() {
            return Some(hardest);
        }
        let deduction = self.techniques[2..]
            .iter()
            .find_map(|t| t.apply(&work, &candidates))?;
        hardest = hardest.max(deduction.level);
        // aplica a dedução...
    }
}
```

- **`pub(crate)`** deixa o método fora da API pública: é um detalhe de implementação do gerador.
- **`find_map(...)?`**: `find_map` devolve a primeira técnica que achou algo, e o `?` num `Option` sai da função
  com `None` se nenhuma achou. Uma linha diz "se empacou, devolve `None`".
- **`self.techniques[2..]`** pula os dois singles, que `place_singles` acabou de esgotar. O comentário em
  `all_techniques` registra que eles abrem a lista em todo nível.

Por que a resposta é **a mesma** de `solve`? Numa grade que é parte de uma solução válida (tudo que o gerador
cava), um single disponível continua disponível até ser colocado. Então qualquer ordem de colocar os singles
chega à mesma grade. E `solve` só passa da terceira técnica quando os dois singles falham: as técnicas
difíceis veem, nos dois caminhos, exatamente as mesmas grades. O teste `hardest_step_concorda_com_solve` compara
os dois em dezenas de grades, umas que resolvem e outras que empacam, nos quatro níveis de solver.

## Truques de bits

`place_singles` acha os Hidden Singles de uma unidade sem contar dígito por dígito:

```rust
let (mut once, mut twice) = (0u16, 0u16);
for &i in unit {
    let bits = candidates.get(i).bits();
    twice |= once & bits; // já tinha aparecido: agora apareceu duas vezes
    once |= bits;
}
let mut hidden = once & !twice; // apareceu exatamente uma vez
while hidden != 0 {
    let digit = hidden.isolate_lowest_one(); // só o bit mais baixo
    hidden &= hidden - 1;                    // desliga o bit mais baixo
    // ...
}
```

- **`isolate_lowest_one()`** devolve só o bit mais baixo ligado. A primeira versão usava o truque clássico
  `x & x.wrapping_neg()` (o negativo em complemento de dois, sem checagem de overflow; num `u16`, `-x` nem
  compilaria). O clippy do Rust 1.99 passou a recusar o truque (`manual_isolate_lowest_one`) e a sugerir o
  método, que diz a intenção no nome. O CI usa o Rust `stable` mais novo e pegou isso, mas o PC local ainda
  estava no 1.91, cujo clippy não conhecia o lint: "passou local" só vale com a mesma versão do CI.
- **`x & (x - 1)`** desliga o bit mais baixo; **`trailing_zeros()`** diz qual é a posição dele.
- O laço `while bits != 0` percorre só os bits ligados, em ordem crescente. É o que mantém a ordem dos pares
  igual à das varreduras antigas.

## Um cache com invariante

A segunda otimização guarda no `CandidateGrid` a tabela que as técnicas recalculavam:

```rust
pub struct CandidateGrid {
    cells: [CandidateSet; 81],
    /// `positions[u][d]`: bit `p` ligado quer dizer que `units()[u][p]` tem `d` candidato.
    positions: [[u16; 10]; 27],
}
```

Um cache só é seguro se nunca fica velho. A regra (o **invariante**) é: `positions` é sempre consequência de
`cells`. O Rust ajuda a mantê-la porque os campos são privados: só `from_grid`, `place` e `eliminate` mexem neles,
e os três foram escritos juntos.

```rust
pub fn eliminate(&mut self, index: usize, value: u8) {
    if self.cells[index].contains(value) {
        self.cells[index].remove(value);
        for (u, pos) in SLOTS[index] {
            self.positions[u][value as usize] &= !(1 << pos);
        }
    }
}
```

O `if contains` importa: sem ele, eliminar duas vezes o mesmo candidato seria inofensivo para `cells`, mas
deixaria de ser verdade que "um bit muda junto com o outro". `place` passou a ser escrito em cima de
`eliminate`, então só existe um lugar que atualiza as duas coisas.

`SLOTS` é uma tabela `static` montada por uma `const fn` (como `units()`, wiki 25): para cada célula, as três
unidades e a posição dela em cada uma. O `#[derive(PartialEq)]` continua correto, porque duas grades com as
mesmas células sempre têm as mesmas posições. O teste antigo `colocar_equivale_a_reconstruir_da_grade` agora
confere o cache também, de graça.

## Closures diferentes num mesmo array

X-Wing e Swordfish passaram a percorrer pares `(cell, first_unit)`:

```rust
let by_row: fn(usize, usize) -> usize = |line, pos| line * 9 + pos;
let by_column = |column: usize, pos: usize| pos * 9 + column;
for (cell, first_unit) in [(by_row, 0), (by_column, 9)] { ... }
```

Sem a anotação em `by_row`, o compilador recusa: cada closure tem um **tipo próprio e anônimo**, e um array
precisa de elementos do mesmo tipo. Em `[by_row, by_column]` (wiki 25) o Rust converte as duas para `fn`
sozinho, mas dentro de tuplas essa conversão não acontece. Anotar o primeiro elemento como ponteiro de função
resolve: closures que não capturam nada viram `fn`, e o segundo segue o tipo do primeiro.

## Constantes como contrato: medir em trabalho, não em tempo

Para escolher `REFILL_VERY_HARD` (K) e `MAX_REFINEMENTS_VERY_HARD` (R), um hook temporário rodou o mesmo laço
sem teto e contou **cavações** por seed, não milissegundos. A contagem é determinística, então não depende de o PC
estar ocupado:

| K | R | Mediana | p90 | p99 | Máximo |
|---|---|---|---|---|---|
| 6 | 80 | 75 | 222 | 547 | 567 |
| **8** | **80** | **59** | **212** | **377** | **434** |
| 10 | 80 | 67 | 275 | 533 | 587 |

Com K e R fixos, o teto é `MAX_SOLUTIONS_VERY_HARD × (1 + R)` cavações, e daí saem as seeds esgotadas para
cada valor: 12 com 3 soluções, 5 com 4, 1 com 5, 0 com 6. Por que 5, e não 6, se 6 também passava em 754 ms?
Porque nenhuma das 200 seeds chega ao teto de 6 (486 cavações), então os 754 ms não dizem quanto ele custa,
e a estimativa é cerca de 1 s. Com 5, a seed 15 esgota, e o tempo dela (793 ms) **é** o custo do teto inteiro:
o pior caso fica medido, não estimado.

As três constantes entram no contrato de determinismo, e o golden test da seed 42 as trava. Uma consequência
pequena: a seed 15, usada nos testes da Task 3, passou a esgotar. Ela virou o teste
`muito_dificil_desiste_depois_do_teto_de_trabalho`, e os outros testes passaram a usar a seed 19.

## Provar que nada mudou

Cada otimização foi conferida contra referências gravadas antes da fatia: 50 puzzles de cada nível antigo e 50
de Muito Difícil, comparados com `diff`. As 200 linhas saíram idênticas depois de cada mudança. Para separar a
otimização da troca de constantes, as constantes voltaram aos valores antigos durante a comparação.

> **Lição geral:** meça antes, e meça a coisa certa (aqui, cavações em vez de milissegundos). Um cache é seguro
> quando campos privados e um único ponto de atualização garantem o invariante. E um caminho rápido só pode
> substituir o lento quando dá para explicar por que a resposta é a mesma, e um teste confere isso.
