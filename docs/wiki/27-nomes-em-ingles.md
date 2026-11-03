# 27 — Nomes em inglês: uma regra de idioma e um renomeio seguro

> Referente à padronização feita no fim da fatia Difícil. Commit: `refactor: use English names for all code identifiers`.

O projeto usa dois idiomas de propósito: o histórico do git e o código seguem a convenção da indústria (inglês), e a
documentação de estudo fica em português. Até aqui, a regra só cobria commits, comentários e nomes de teste. Os
nomes no código ficaram sem regra e acabaram misturados: `sole_spot` ao lado de `fora_do_box`, `value` ao lado de
`celulas`.

## A regra

| O quê | Idioma |
|---|---|
| Commits e PRs | inglês |
| Nomes no código (tipos, funções, constantes, variáveis, helpers de teste) | **inglês** |
| Nome da função `#[test]` | português |
| Comentários, doc comments e mensagens de `assert!` | português |
| Specs, planos e wiki | português |

A exceção são as variantes de `Difficulty` (`Facil`, `Medio`, `Dificil`, `MuitoDificil`). Elas são API pública, e
trocá-las é uma decisão à parte.

## Por que renomear é seguro aqui

Nomes locais e privados não existem para quem usa a biblioteca: o compilador os resolve e descarta. Trocar
`celulas` por `cells` não muda nenhuma instrução executada, e por isso não muda nenhum puzzle. Os golden tests de
Fácil, Médio e Difícil passaram sem alteração, comprovando isso.

O Rust ajuda de duas formas:

- **Sem sombra silenciosa:** se o nome novo colidisse com outro no mesmo escopo, o código mudaria de sentido ou
  deixaria de compilar. A suíte inteira e o clippy pegam os dois casos.
- **Capturas em strings de formatação:** `"pior caso: {pior:?}"` lê a variável `pior` **de dentro da string**. O
  renomeio automático pulou as strings, para não traduzir as mensagens, e o compilador acusou
  `cannot find value pior`. A correção foi trocar só a captura: `"pior caso: {worst:?}"`.

## Principais trocas

| Antes | Depois |
|---|---|
| `MAX_TENTATIVAS_MEDIO`, `MAX_TENTATIVAS_DIFICIL` | `MAX_ATTEMPTS_MEDIUM`, `MAX_ATTEMPTS_HARD` |
| `tentar` | `generate_with_attempts` |
| `fora_do_box`, `linha_de` | `outside_box`, `line_of` |
| `monta_unidades`, `unidades` | `build_units`, `all_units` |
| `procura`, `por_linha`, `por_coluna`, `mascaras` | `search`, `by_row`, `by_column`, `masks` |
| `celulas`, `pares`, `posicoes`, `onde` | `cells`, `pairs`, `positions`, `spots` |
| `TODOS` | `ALL_DIGITS` |

As páginas anteriores da wiki contam a história com os nomes da época. Só as constantes e `generate_with_attempts`
foram atualizadas nelas, por serem os nomes mais procurados.
