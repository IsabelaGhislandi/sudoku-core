# 11 — Rating: cadeias de iteradores, `max` e `Option` como resposta

> Referente à **Task 11** do plano. Commit: `feat: add rate to classify difficulty by required techniques`.

Com o `LogicalSolver` (Task 9) e o `count_solutions` (Task 10) prontos, dá para responder a
pergunta que dá nome ao jogo: **qual é a dificuldade deste tabuleiro?** A função `rate` não
inventa nada novo — ela **compõe** as duas peças anteriores.

## A regra de classificação

```rust
pub fn rate(grid: &Grid) -> Option<Difficulty> {
    if count_solutions(grid, 2) != 1 {
        return None;
    }
    let result = LogicalSolver::new().solve(grid);
    if result.solved {
        let level = result
            .steps
            .iter()
            .map(|s| s.level)
            .max()
            .unwrap_or(Difficulty::Facil);
        Some(level)
    } else {
        Some(Difficulty::MuitoDificil)
    }
}
```

Lida em português:

1. **Não tem solução única?** Então não é um puzzle de verdade → `None`.
2. **A lógica resolve?** O nível é o da **técnica mais avançada** usada em algum passo.
3. **A lógica empaca?** Só a força bruta resolve → `MuitoDificil`.

A ideia central é que a dificuldade de um Sudoku não é "quantas pistas tem", e sim **o passo
mais difícil que você é obrigado a dar**. Um puzzle com 80 passos triviais e um único X-Wing é
um puzzle Difícil.

## Guard clause: `return` cedo

```rust
if count_solutions(grid, 2) != 1 {
    return None;
}
```

Em Rust a última expressão de um bloco já é o valor de retorno (sem `;`), então `return`
explícito aparece quase só em **saídas antecipadas**. Esse padrão — checar o caso inválido e
sair logo — evita aninhar o resto da função dentro de um `if`. É o mesmo "early return" que se
usa em JavaScript, só que aqui o compilador garante que **todos os caminhos devolvem o tipo
certo** (`Option<Difficulty>`).

Repare que o `if result.solved { ... } else { ... }` no fim **não tem `return`**: o `if/else`
inteiro é uma expressão, e cada ramo termina em `Some(...)` sem ponto-e-vírgula.

## A cadeia de iteradores

```rust
result.steps     // Vec<Deduction>
    .iter()      // Iterator<Item = &Deduction>
    .map(|s| s.level)   // Iterator<Item = Difficulty>
    .max()       // Option<Difficulty>
    .unwrap_or(Difficulty::Facil)  // Difficulty
```

Acompanhar o **tipo** a cada elo é o jeito mais seguro de ler essas cadeias:

| Elo | O que faz | Tipo resultante |
|---|---|---|
| `.iter()` | empresta cada item sem consumir o `Vec` | `&Deduction` |
| `.map(\|s\| s.level)` | projeta só o campo que interessa | `Difficulty` |
| `.max()` | maior elemento, se houver algum | `Option<Difficulty>` |
| `.unwrap_or(x)` | tira do `Option`, com valor padrão | `Difficulty` |

Três detalhes de Rust estão escondidos aí:

- **`.map` é lazy.** Nada é calculado até o `.max()` puxar os valores. Não há `Vec`
  intermediário, ao contrário de `arr.map(...)` em JS, que cria um array novo.
- **`s.level` sai de um `&Deduction` sem erro de borrow** porque `Difficulty` é `Copy`
  (Task 2). O valor é copiado bit a bit; se fosse um tipo não-`Copy` (tipo `String`), o
  compilador reclamaria de "move out of borrowed content" e seria preciso `.clone()`.
- **`.max()` só existe porque `Difficulty: Ord`.** O `derive(PartialOrd, Ord)` da Task 2 ordena
  as variantes **pela ordem de declaração** — `Facil < Medio < Dificil < MuitoDificil`. Aquele
  "truque" agora paga a conta: o nível mais difícil é literalmente o `max`.

### Por que `.max()` devolve `Option`?

Porque o iterador pode estar **vazio** — e aí não existe maior elemento. Rust não tem `-Infinity`
nem `undefined` para mascarar isso: o tipo força você a decidir o que acontece.

Quando isso ocorre aqui? Com uma grade **já completa**: `solve` devolve `solved: true` com
`steps` vazio. O `unwrap_or(Difficulty::Facil)` diz "um puzzle sem nada a fazer é Fácil".

| Método | Se for `None` | Quando usar |
|---|---|---|
| `.unwrap()` | **panic** | só em testes ou quando é impossível ser `None` |
| `.unwrap_or(v)` | devolve `v` (sempre avaliado) | padrão barato, como uma variante de enum |
| `.unwrap_or_else(\|\| ...)` | chama a closure | padrão caro de calcular |
| `?` | propaga o `None` para quem chamou | dentro de função que devolve `Option` |

## `Option` como resposta de domínio

A assinatura `fn rate(&Grid) -> Option<Difficulty>` é, ela mesma, documentação: **nem toda
grade tem dificuldade**. Uma grade vazia, ou uma com duas soluções, não é um Sudoku — e o tipo
de retorno obriga quem chama a tratar esse caso.

A alternativa seria inventar uma variante `Difficulty::Invalido`. Seria pior: todo `match` sobre
`Difficulty` no resto do app (tela de seleção, histórico de partidas) teria que lidar com um
"nível" que nunca devia existir ali. Manter a ausência **fora** do enum deixa `Difficulty`
limpo e empurra a validação para a borda.

## A ordem das checagens importa

`count_solutions` vem **antes** do solver lógico, por dois motivos:

1. **Correção.** O `LogicalSolver` só faz jogadas forçadas, então ele nunca "resolve" uma grade
   ambígua — mas também pode empacar numa grade **sem solução** e isso viraria um falso
   `MuitoDificil`. Checar unicidade primeiro elimina os dois casos.
2. **Semântica.** "Muito Difícil" significa "tem resposta única, mas a lógica atual não
   alcança". Só faz sentido afirmar isso depois de provar que a resposta existe.

## Limitação consciente desta fatia

Hoje o solver só conhece Naked Single e Hidden Single, ambos `Facil`. Logo `rate` só devolve
`Facil`, `MuitoDificil` ou `None` — nunca `Medio` nem `Dificil`. Um puzzle que precisa de um
simples Naked Pair aparece como `MuitoDificil`.

Isso **não exige mudar `rate`** nas próximas fatias: basta registrar técnicas novas no
`LogicalSolver`, cada uma com seu `level`. O `max()` sobre os passos se ajusta sozinho. É o
pattern Strategy (Task 7) rendendo de novo.

## Por que isso importa

`rate` é o **juiz** do gerador. Na Task 12, a cada célula cavada, o gerador precisa saber se o
puzzle continua único e continua no nível-alvo — exatamente as duas perguntas que `rate`
responde.
