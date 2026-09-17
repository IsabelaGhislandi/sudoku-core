# 12 — Gerador: RNG com seed, traits em escopo e a heurística MRV

> Referente à **Task 12** do plano. Commits: `perf: branch on most-constrained cell in count_solutions`
> e `feat: add deterministic Easy puzzle generator`.

Até aqui o crate só **analisava** tabuleiros prontos. Agora ele **cria** tabuleiros. O
gerador junta as peças anteriores: backtracking para montar uma grade cheia, `count_solutions`
para garantir unicidade e o `LogicalSolver` para garantir que o puzzle continua Fácil.

## O algoritmo em duas fases

```
seed ──► StdRng ──► full_solution ──► dig_facil ──► Puzzle { givens, solution, .. }
                    (grade cheia)     (cava células)
```

1. **`full_solution`**: preenche uma grade vazia com backtracking, tentando os dígitos em
   **ordem embaralhada**. Mesmo algoritmo do `count_solutions` (Task 10), mas para no
   primeiro sucesso (`-> bool`) e a aleatoriedade faz cada seed dar uma grade diferente.
2. **`dig_facil`**: percorre as 81 células em ordem aleatória e tenta esvaziar cada uma. A
   remoção **fica** só se o puzzle continuar com solução única **e** resolvível pelo solver
   Fácil. Senão, a célula é restaurada.

```rust
for index in order {
    let removed = puzzle.get(index);
    puzzle.set(index, Cell::Empty);
    let unique = count_solutions(&puzzle, 2) == 1;
    let logical = solver.solve(&puzzle).solved;
    if !(unique && logical) {
        puzzle.set(index, removed); // restaura
    }
}
```

É o mesmo "tenta, verifica, desfaz" do backtracking, só que **guloso**: nunca volta para
reconsiderar uma célula. Por isso o puzzle final é *minimal* em relação àquela ordem (nenhuma
pista restante pode sair sem quebrar as regras), mas não necessariamente o puzzle com menos
pistas possível.

`removed` só pode ser guardado e reposto assim porque `Cell` é `Copy`: `puzzle.get(index)`
devolve uma **cópia** da célula, não uma referência para dentro do `puzzle`. Se fosse
referência, o borrow checker bloquearia o `puzzle.set(...)` logo em seguida.

## Determinismo: RNG com seed

```rust
let mut rng = StdRng::seed_from_u64(seed);
```

Um gerador pseudoaleatório é uma **função pura disfarçada**: dado o mesmo estado inicial
(a seed), produz sempre a mesma sequência de números. É o que torna estes testes possíveis:

```rust
let a = generate(Difficulty::Facil, 7).unwrap();
let b = generate(Difficulty::Facil, 7).unwrap();
assert_eq!(a, b);
```

Em JavaScript, `Math.random()` não aceita seed, e seria preciso uma lib. Em Rust o `rand`
separa **o algoritmo** (`StdRng`) de **como ele é iniciado** (`seed_from_u64`, ou a partir da
entropia do SO para aleatoriedade real).

O RNG é passado como **`&mut StdRng`** para `fill` e `dig_facil`: gerar um número **avança o
estado interno**, então é uma mutação. Passar o mesmo `rng` adiante (em vez de criar outro)
garante uma única sequência determinística para a geração inteira. A ordem das chamadas faz
parte do contrato: embaralhar algo a mais no meio mudaria todos os puzzles.

> ℹ️ A primeira versão desta task usava o `StdRng`, e isso foi corrigido logo em seguida —
> veja a [página 14](14-rng-reproduzivel-e-golden-tests.md). Resumo: o `rand` se reserva o
> direito de trocar o algoritmo do `StdRng` entre versões, então o determinismo não
> sobreviveria a um upgrade da lib. O gerador usa hoje o `ChaCha12Rng`, de algoritmo fixo.

## Traits precisam estar em escopo

```rust
use rand::rngs::StdRng;
use rand::seq::SliceRandom;   // traz .shuffle() para slices
use rand::SeedableRng;        // traz StdRng::seed_from_u64
```

Nenhuma dessas duas últimas linhas é usada **pelo nome** no código, mas sem elas não compila:

```
error[E0599]: no method named `shuffle` found for struct `Vec<u8>` in the current scope
```

Em Rust, um método definido por um **trait** só pode ser chamado se o trait estiver importado
no módulo. `Vec` não tem `shuffle`; é o trait `SliceRandom` que **adiciona** esse método a
todo slice (`impl<T> SliceRandom for [T]`). É o jeito do Rust de fazer "extension methods"
sem monkey-patching: a extensão existe, mas só fica visível onde você pede.

Isso evita colisões. Se duas bibliotecas definirem um `.shuffle()` para `Vec`, só o trait
importado vale naquele arquivo. Em JS, quem modifica `Array.prototype` afeta o programa inteiro.

O compilador costuma ajudar: o erro acima vem com `help: trait SliceRandom which provides
shuffle is implemented but not in scope; perhaps you want to import it`.

## `Result` e um erro que é um enum

```rust
pub enum GenError {
    Unsupported(Difficulty),
}

pub fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>
```

Nesta fatia só existe o Fácil, e pedir outro nível é uma **falha esperada**, não um bug. Por
isso é `Result`, e não `panic!`. O erro carrega **qual** nível foi pedido
(`Unsupported(Difficulty::Medio)`), para quem chama poder montar uma mensagem útil.

Quando as próximas fatias implementarem o Médio, `Unsupported` deixa de ocorrer para ele, mas
o enum continua útil. Uma variante futura provável é "não consegui atingir o nível alvo em N
tentativas".

## A lição de performance: MRV

A primeira versão rodou os testes do gerador em **~220 segundos**. A medição mostrou onde:

| Chamada | Pior tempo (build de debug) |
|---|---|
| `full_solution` | < 1 ms |
| `solver.solve` | ~60 ms |
| `count_solutions` | **33 s** |

O `count_solutions` da Task 10 ramificava sempre na **primeira** célula vazia. Numa grade
esparsa ela pode ter 7–9 candidatos, e o erro só aparece muitos níveis abaixo, o que torna a
árvore de busca gigantesca.

A correção é a heurística clássica **MRV** (*Minimum Remaining Values*): ramificar na célula
vazia com **menos candidatos**.

```rust
let next = (0..81)
    .filter(|&i| matches!(grid.get(i), Cell::Empty))
    .min_by_key(|&i| candidates_for(grid, i).count());
```

- Se alguma célula tem **0** candidatos, ela é escolhida e o `for` não roda: o ramo morre
  **imediatamente**, em vez de dezenas de níveis depois.
- Se tem **1**, não há ramificação: é uma jogada forçada (o mesmo raciocínio do Naked Single).
- Só se ramifica de verdade quando não há escolha melhor.

Resultado: suíte inteira em **~8 s**, e um puzzle de 17 pistas (o mínimo possível para
solução única) que antes não terminava em 60 s agora resolve na hora. Esse puzzle virou teste
(`puzzle_de_17_pistas_eh_unico`) para proteger contra regressão.

Dois pontos de Rust nesse trecho:

- **`min_by_key`** devolve `Option<usize>`: `None` se o `filter` não deixou nada (grade
  cheia), que é exatamente o caso base da recursão. O `match` existente continuou igual.
- A closure empresta `grid` **imutavelmente**, e logo depois o código faz `grid.set(...)`
  (empréstimo **mutável**). Compila porque o iterador é totalmente consumido pelo
  `min_by_key` antes do `match`: o empréstimo imutável termina ali. O borrow checker olha
  **até onde** cada empréstimo é usado, não o escopo léxico inteiro.

> **Lição geral:** medir antes de otimizar. As duas verificações rodam 81 vezes por puzzle,
> e só a medição mostrou que uma delas custava ~500× mais que a outra.

## Por que isso importa

O crate agora gera, resolve, valida e classifica. Falta só a **Task 13**: decidir o que é
público (`pub use` em `lib.rs`), adicionar `next_hint` e escrever um teste de integração que
use o crate **de fora**, como o módulo Game vai usar.
