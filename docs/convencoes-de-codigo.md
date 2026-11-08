# Convenções de código

Como escrever código, nomes, commits e PRs neste projeto. O [`CLAUDE.md`](../CLAUDE.md) resume as
regras para o agente; este documento explica cada uma, com exemplos do próprio código. Quando os
dois divergirem, corrija o que estiver errado no mesmo commit.

Base: as [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), o
[Conventional Commits](https://www.conventionalcommits.org/) e os princípios SOLID, traduzidos
para o que faz sentido em Rust (seção 4).

## 1. Idioma de cada coisa

| O quê | Idioma | Exemplo |
|---|---|---|
| Mensagens de commit | inglês | `feat: add per-level statistics and personal ranking` |
| Nomes de branch | inglês | `feat/game-results`, `chore/skills-cleanup` |
| Título e descrição de PR | inglês | mesmo formato do commit |
| Tipos, funções, variáveis, constantes, módulos, helpers de teste | inglês | `CandidateGrid`, `level_stats`, `MAX_ATTEMPTS_HARD` |
| Nome da função `#[test]` | **português** (exceção) | `fn partida_completa_ate_a_vitoria()` |
| Comentários, doc comments (`///`), mensagens de `assert!` e `expect` | português | `expect("3 não é zero")` |
| Specs, planos, wiki, `architecture.md`, `CONTEXT.md` | português | |
| `CHANGELOG.md`, `CONTRIBUTING.md`, `README.md` | inglês | públicos, voltados a quem chega de fora |

**Por que o teste é exceção:** o nome de um teste é uma frase de comportamento lida na saída do
`cargo test` e citada na wiki, não um identificador que alguém chama. Variáveis e helpers dentro
do teste seguem a regra geral (inglês).

**Exceção de API:** as variantes de `Difficulty` (`Facil`, `Medio`, `Dificil`, `MuitoDificil`).
Elas estão no formato salvo (`SavedGame`, `SavedResult`), então mudar o nome é mudança de formato,
com versão nova. Ficam como estão até uma decisão consciente.

Sem misturar idiomas num mesmo nome: nada de `calcular_score` ou `fatia-results`.

## 2. Git: commits, branches e PRs

**Commits** seguem o Conventional Commits:

```
type: short summary in the imperative

Optional body explaining why, wrapped at ~72 columns.
```

- Tipos: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`.
- O assunto diz o que o commit **faz**: "add rate", não "added rate" nem "adding rate".
- Minúsculas depois do tipo, sem ponto final, até ~72 caracteres.
- O corpo explica o **porquê**; o *como* está no diff.
- Um commit por unidade lógica: o código num `feat`/`fix`, a página da wiki num `docs`.

**Branches:** `<tipo>/<descricao-curta-em-kebab-case>`, com o mesmo tipo do commit principal:
`feat/game-results`, `fix/win-rate-rounding`, `chore/skills-cleanup`. Sempre a partir do
`master` atualizado.

**PRs:** o título segue o formato do commit (`feat: add game results`); a descrição, em inglês,
segue o template em `.github/pull_request_template.md`.

## 3. Nomes (Rust API Guidelines)

### Caixa

| Item | Caixa | Exemplo do projeto |
|---|---|---|
| Tipos, traits, variantes de enum | `UpperCamelCase` | `LogicalSolver`, `Technique`, `Outcome::OutOfLives` |
| Funções, métodos, variáveis, módulos | `snake_case` | `count_solutions`, `elapsed_ms`, `box_line_reduction` |
| Constantes e `static` | `SCREAMING_SNAKE_CASE` | `SAVE_VERSION`, `RESULT_VERSION` |
| Parâmetros genéricos | uma letra maiúscula ou nome curto | `T`, `R: Rng` (ainda não há genéricos no projeto) |

Siglas contam como uma palavra: `JsonFormat`, não `JSONFormat` (exemplo).

### Prefixos e formas que dizem o que a função faz

| Forma | Significado | Exemplo do projeto |
|---|---|---|
| `new` | construtor principal | `Game::new`, `LogicalSolver::new` |
| `from_*` | constrói a partir de outra coisa (pode falhar e devolver `Result`) | `Grid::from_line`, `CandidateGrid::from_grid`, `Game::from_puzzle` |
| `to_*` | conversão que cria um valor novo, sem consumir | `Grid::to_line` |
| `as_*` | visão barata, sem cópia | — |
| `into_*` | conversão que consome `self` | — |
| sem `get_` | getter tem o nome do campo | `lives_left()`, `hints_left()`, `status()` |
| `is_*`, `has_*`, `can_*` | devolve `bool` | `is_complete`, `is_paused`, `can_place` |
| verbo no imperativo | ação que muda estado | `place`, `toggle_note`, `pause`, `resume` |
| `*Error` | tipo de erro de uma operação | `ParseError`, `MoveError`, `RestoreError` |

### Clareza

- Nome pelo **conceito do domínio** (ver [`CONTEXT.md`](../CONTEXT.md)), não pela implementação:
  `hints_left`, não `counter2`.
- Sem abreviações fora as consagradas (`id`, `ms`, `rng`). Unidade no nome quando houver:
  `elapsed_ms`, `finished_at_ms`.
- Nomes de uma letra só em escopo curto e óbvio (`|v| v + 1`, índices de laço).

## 4. Princípios de design: SOLID em Rust

Rust não tem herança, e boa parte do SOLID foi pensada para ela. O que sobra, traduzido:

| Princípio | Em Rust, neste projeto |
|---|---|
| **S** — responsabilidade única | Um módulo por motivo de mudança. A foto (`game/save.rs`), o resultado (`result.rs`) e as estatísticas (`stats.rs`) mudam por motivos diferentes e moram em arquivos diferentes. Mudar o formato da foto mexe só em `game/save.rs`. |
| **O** — aberto/fechado | Conjunto **aberto** → trait: uma técnica nova implementa `Technique` e entra na lista do solver, sem mexer nas outras. Conjunto **fechado** → `enum` com `match` exaustivo (`Outcome`, `CellState`): o compilador aponta cada lugar que precisa tratar a variante nova. Escolha pelo tipo de conjunto, não por gosto. |
| **L** — substituição de Liskov | Toda implementação de trait cumpre o contrato **documentado** na trait. Ex.: uma `Deduction` com `Action::Eliminate` nunca vem vazia, senão o solver entra em laço. Contrato que o tipo não garante vai no `///`. |
| **I** — segregação de interfaces | Traits pequenas, com o mínimo de métodos. E trait só quando há (ou vai haver logo) **duas** implementações; com uma só, uma função ou um tipo concreto basta. |
| **D** — inversão de dependência | O núcleo depende de **dados**, não de serviços: o tempo entra como `now: u64`, a aleatoriedade como `seed: u64`. Isso dispensa uma trait `Clock` ou `Random` e mantém o core puro e testável. Trait de serviço só quando a casca precisar trocar a implementação de verdade. |

E os princípios que, em Rust, pesam tanto quanto o SOLID:

- **Estados inválidos não compilam.** `GameConfig.lives` é `NonZeroU8`: uma partida sem vidas
  não existe. `CellState` não tem variante "errado": um erro é evento, não estado.
- **Parse, don't validate.** Dado de fora (`SavedGame`, `SavedResult`) é um tipo de transporte
  sem garantia; só vira tipo de domínio por uma função que confere tudo (`Game::restore`,
  `GameResult::restore`). Depois disso, o resto do código confia no tipo.
- **A assinatura diz o que acontece com o valor.** `&self` lê, `&mut self` muda, `self` consome:
  `Game::abandon(self, ...)` não deixa usar a partida abandonada.
- **Composição em vez de herança.** Struct que contém struct; comportamento comum via trait ou
  função livre.
- **Genérico (`impl Trait`, `<T: Trait>`) por padrão; `dyn Trait` quando a lista é heterogênea**
  (`Vec<Box<dyn Technique>>` no solver).
- **YAGNI.** Abstração nasce da segunda necessidade, não da primeira. "One adapter = hypothetical
  seam, two adapters = real seam" (skill `improve-codebase-architecture`).

## 5. Erros e panics

- **Falha esperada → `Result`**, com um `enum` de erro por operação, cada variante dizendo o
  motivo (`MoveError::Paused`, `RestoreError::...`). Propague com `?`.
- **Ausência normal → `Option`**, não valor mágico: `LevelStats` devolve `None` sem partidas, em
  vez de 0%.
- **`panic` só para invariante interna violada**, que seria bug do próprio crate, e sempre com
  `expect("<por que isso não acontece>")` ou `unreachable!("<motivo>")` em português. Hoje o
  código de biblioteca tem dois casos, ambos assim: `config.rs` e `game.rs`.
- **Sem `unwrap()` fora de testes.** Em teste, `unwrap` e `expect` são livres.
- Ação recusada não muda nada: `MoveError` volta com o estado intacto.

## 6. Visibilidade e API pública

- `pub` só no que a casca usa. Ajuda interna é `pub(crate)` (`hint::pick`, `save::difficulty_name`).
- Campos **privados** quando há invariante a proteger (`Game`, `Grid`); **públicos** em tipos de
  transporte sem garantia (`SavedGame`, `SavedResult`).
- `#[non_exhaustive]` em structs de leitura e enums que podem crescer sem quebrar quem usa
  (`GameResult`, `Restart`).
- Todo item `pub` tem `///` em português dizendo o que é, o que garante e quando falha. Quem
  cobra é o `#![warn(missing_docs)]` no `lib.rs` de cada crate: com o `-D warnings` do clippy,
  um item público sem doc quebra o CI. A doc diz o que o nome não diz (invariante, unidade,
  quando dá panic); `/// Célula vazia.` em `Cell::Empty` só repete o nome.
- Todo módulo começa com `//!` dizendo do que ele cuida. A exceção são os arquivos de
  `techniques/`: cada um tem um único tipo, e o `///` dele já explica a técnica.
- Mudou a API pública? Testes de integração em `tests/` e o `CHANGELOG.md` no mesmo PR.

## 7. Quem garante o quê

| Ferramenta | Garante |
|---|---|
| `cargo fmt --all` | formatação; não se discute estilo em review |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | idiomas e armadilhas comuns; warning é erro |
| `cargo test --workspace --all-features` | comportamento e formatos (golden tests) |
| `cargo deny check` | licenças, alertas de segurança e origem das dependências |
| review | nomes, design e o resto deste documento |

## 8. Pendências conhecidas

O código ainda não cumpre estas regras das API Guidelines. Ficam registradas para uma task
`refactor` própria, não para ajustes soltos:

- **Os enums de erro não implementam `Display` nem `std::error::Error`** (`ParseError`,
  `GenError`, `MoveError`, `PuzzleError`, `RestoreError`, `ResultError`). Sem isso, a casca não
  consegue usar `?` com `Box<dyn Error>` nem mostrar a mensagem direto
  ([C-GOOD-ERR](https://rust-lang.github.io/api-guidelines/interoperability.html#c-good-err)).
- **Nenhuma função usa `#[must_use]`.** Candidatas: `Game::result`, `Game::save`,
  `GameResult::save`, `level_stats`, `generate`, cujo retorno ignorado é quase sempre bug.
