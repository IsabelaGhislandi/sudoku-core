# Fundamentos de Rust — consolidado das Tarefas 31 a 35

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 31 a
> 35. As páginas por-task ([31](../31-xy-wing.md) … [35](../35-crate-sudoku-game-e-nonzero.md)) contam a história
> na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 26 a 30](rust-base-tarefas-26-a-30.md).

Este bloco fecha o engine (XY-Wing, gerador de Muito Difícil e a otimização dele) e abre o módulo Game (workspace
e o crate `sudoku-game`). O fio que liga as duas metades é **deixar o compilador garantir as regras**: padrões que
precisam casar, `match` que precisa ser completo, campos privados que protegem um cache, e tipos em que um valor
inválido nem pode ser escrito.

Sumário:
1. [Padrões que extraem ou desistem: `let ... else`](#1-padrões-que-extraem-ou-desistem-let--else)
2. [`match` exaustivo como alarme](#2-match-exaustivo-como-alarme)
3. [Closures: captura, tipo anônimo e `fn`](#3-closures-captura-tipo-anônimo-e-fn)
4. [Invariantes protegidos por privacidade](#4-invariantes-protegidos-por-privacidade)
5. [Tipos que proíbem valores: `NonZeroU8`](#5-tipos-que-proíbem-valores-nonzerou8)
6. [Medir em trabalho, não em tempo](#6-medir-em-trabalho-não-em-tempo)
7. [Workspace, dependência por caminho e re-export](#7-workspace-dependência-por-caminho-e-re-export)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. Padrões que extraem ou desistem: `let ... else`

```rust
let (Some(x), Some(y)) = (digits.next(), digits.next()) else {
    continue;
};
```

`let ... else` tenta casar um padrão. Se casar, as variáveis ficam disponíveis no resto do bloco. Se não casar, o
`else` roda e **é obrigado a sair** dali (`continue`, `return`, `break`, `panic!`): o compilador recusa um `else`
que deixa o código seguir. Comparado a `if let`, o caminho feliz não fica aninhado; comparado a `unwrap()`, o caso
impossível fica explícito e não vira pânico (página 31).

## 2. `match` exaustivo como alarme

Um `match` sem `_` precisa cobrir todas as variantes. Quando o gerador ganhou o Muito Difícil, o braço
`_ => Err(Unsupported)` saiu junto com a variante `Unsupported` (página 32). Agora, se um quinto nível surgir, o
código não compila até alguém decidir como gerá-lo. Com o `_`, o nível novo cairia em silêncio num erro genérico.

O mesmo mecanismo trabalha ao contrário: remover uma variante tornou um braço **inalcançável**, e o compilador
avisou. Na Task 35, `Game::new` faz `match` em `GenError` sem `_`: se o core ganhar outro erro, o Game para de
compilar em vez de ignorá-lo.

## 3. Closures: captura, tipo anônimo e `fn`

- **Captura.** `|givens, solution| Puzzle { givens, solution, difficulty, seed }` usa `difficulty` e `seed` do
  ambiente (página 32). Como são `Copy`, a closure copia os valores e pode ser chamada várias vezes.
- **Tipo anônimo.** Cada closure tem um tipo próprio, que ninguém consegue escrever. Duas closures com o mesmo
  corpo ainda têm tipos diferentes, e por isso não cabem juntas num array ou numa tupla (página 33).
- **Ponteiro de função.** Uma closure que **não captura nada** pode virar `fn(usize, usize) -> usize`. Anotar o
  primeiro elemento com esse tipo faz o segundo seguir, e os dois cabem no mesmo array.
- **Montar arrays.** `std::array::from_fn(|i| ...)` chama a closure para cada índice e devolve um `[T; N]`, sem
  valores provisórios (página 35).

## 4. Invariantes protegidos por privacidade

Um **invariante** é uma regra que precisa ser verdade o tempo todo. No `CandidateGrid`, a regra é "`positions` é
sempre consequência de `cells`" (página 33). O Rust ajuda porque os campos são privados: só os métodos do próprio
tipo mexem neles, então basta conferir esses métodos. `place` foi reescrito em cima de `eliminate` para existir
um lugar só que atualiza as duas coisas.

A `Game` da Task 35 segue o mesmo desenho: `cells`, `lives_left` e `status` são privados, e a tela só lê por
métodos (`cell`, `lives_left`, `status`). Nas próximas tasks, as jogadas vão ser os únicos caminhos para mudar o
estado, e as regras (número certo trava, vida perdida não volta) ficam garantidas por construção.

## 5. Tipos que proíbem valores: `NonZeroU8`

`GameConfig.lives` é um `NonZeroU8`: um `u8` que não pode ser zero (página 35). O valor só nasce por
`NonZeroU8::new(n)`, que devolve `None` para zero, ou por constantes como `NonZeroU8::MIN`. Depois disso,
ninguém precisa checar: uma partida com zero vidas não pode ser descrita.

É a ideia de **tornar estados inválidos irrepresentáveis**, prima do `match` exaustivo: em vez de lembrar de
validar, escolha um tipo em que o erro não compila. De bônus, `Option<NonZeroU8>` cabe em 1 byte, porque o zero
proibido representa o `None`.

## 6. Medir em trabalho, não em tempo

Para otimizar o Muito Difícil, a página 33 mediu antes de mexer, com um acumulador `thread_local!` por técnica, e
provou depois que nenhum puzzle mudou (golden tests). Para escolher as constantes do gerador, contou **cavações**
em vez de milissegundos. A contagem é determinística e não depende de o PC estar ocupado, e por isso os tetos
(`MAX_REFINEMENTS_VERY_HARD` e companhia) viraram **contrato**: mudar um deles muda puzzles.

O Game herda a mesma postura: `Game::new` resolve uma seed esgotada tentando a seguinte, de forma determinística,
e um teste trava o caso conhecido (15 → 16).

## 7. Workspace, dependência por caminho e re-export

- **Workspace** (página 34): vários crates com um `Cargo.lock`, um `target/` e um `deny.toml` na raiz. Campos
  comuns ficam em `[workspace.package]` e são herdados com `edition.workspace = true`. Os comandos usam
  `--workspace`, `-p <crate>` e `fmt --all`, de qualquer pasta.
- **Dependência por caminho** (página 35): `sudoku-core = { path = "../sudoku-core" }`. Compila a pasta local, sem
  versão nem download, e só enxerga o que é `pub`.
- **Re-export entre crates:** `pub use sudoku_core::{Difficulty, Puzzle};` republica tipos do core. São os mesmos
  tipos, não cópias, e quem usa o `sudoku-game` não precisa depender do core.

## 8. Tabela-resumo dos conceitos

| Conceito | Onde apareceu | Em uma frase |
|---|---|---|
| `let ... else` | 31 | Casa um padrão ou sai do bloco, sem aninhar e sem `unwrap`. |
| `match` exaustivo | 32, 35 | Sem `_`, uma variante nova quebra a compilação até ser tratada. |
| Captura de closure | 32 | A closure leva valores do ambiente; com `Copy`, pode ser chamada várias vezes. |
| Closure como `fn` | 33 | Sem captura, vira ponteiro de função e cabe num array com outras. |
| `thread_local!` | 33 | Variável global por thread, útil para medir sem interferência entre testes. |
| Invariante com campos privados | 33, 35 | Só os métodos do tipo mudam o estado, então só eles precisam garantir a regra. |
| Constantes como contrato | 33 | Tetos medidos em trabalho determinístico fazem parte da saída. |
| Cargo workspace | 34 | Vários crates com lock, `target/` e regras compartilhados. |
| `[workspace.package]` | 34 | Campos comuns declarados uma vez e herdados explicitamente. |
| Dependência por `path` | 35 | Um crate local do workspace, compilado da pasta. |
| `pub use` de outro crate | 35 | Republica tipos, que continuam sendo os mesmos. |
| `NonZeroU8` | 35 | Um tipo em que o valor inválido (zero) não pode ser escrito. |
| `impl Default` à mão | 35 | Valores padrão documentados num lugar só. |
| `std::array::from_fn` | 35 | Monta um array fixo a partir do índice. |
