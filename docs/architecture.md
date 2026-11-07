# Arquitetura — Sudoku

> Visão de **design** do projeto. Para os aprendizados de Rust durante a implementação,
> veja a [wiki](wiki/README.md). Para o detalhe completo de cada decisão, veja a
> [spec do Core/Engine](superpowers/specs/2026-06-14-sudoku-engine-design.md), a
> specs das fatias [Médio](superpowers/specs/2026-11-01-engine-medio-design.md),
> [Difícil](superpowers/specs/2026-11-02-engine-dificil-design.md) e
> [Muito Difícil](superpowers/specs/2026-11-03-engine-muito-dificil-design.md), e os planos das fatias
> [Fácil](superpowers/plans/2026-06-14-engine-facil-vertical-slice.md),
> [Médio](superpowers/plans/2026-11-01-engine-medio.md),
> [Difícil](superpowers/plans/2026-11-02-engine-dificil.md) e
> [Muito Difícil](superpowers/plans/2026-11-03-engine-muito-dificil.md).

## 1. Visão geral

Jogo de Sudoku para celular (APK Android via **Tauri**), com backend em **Rust** e
frontend em **Svelte**. Desenvolvimento **modular** e **spec-driven**: cada módulo segue
o ciclo `spec → plano → implementação` com **TDD**.

## 2. Decomposição em módulos

| # | Módulo | Responsabilidade | Status |
|---|--------|------------------|--------|
| 1 | **Core/Engine** (`sudoku-core`) | Gerar, resolver, validar e classificar tabuleiros. Lógica pura. | **concluído: os quatro níveis** |
| 2 | **Game/Sessão** (`sudoku-game`) | Estado de uma partida: jogadas, anotações, erros, **timer**, dicas, **histórico de partidas**. Lógica pura. | **concluído: fatias 1, 2 e 3** (partida básica; tempo e salvamento; resultados) |
| 3 | Frontend/UI | Tabuleiro, input, seleção de dificuldade, telas (Svelte). | futuro |
| 4 | Tauri/Empacotamento | Ponte Rust↔Svelte e build do APK. | futuro |
| 5 | Online | Partida por código com outro player. | futuro distante |

A fronteira é dura: **`sudoku-core` é uma biblioteca pura** — sem UI, sem IO, sem Tauri.
Isso a torna testável isoladamente com `cargo test` e reaproveitável por todos os outros módulos.
O `sudoku-game` segue a mesma regra (ver seção 4.1).

## 3. Decisões de design (Core/Engine)

| Decisão | Escolha | Motivo |
|---------|---------|--------|
| Tamanho da grade | 9×9 clássico | Padrão do Sudoku |
| Solução | Única garantida | Sudoku "de verdade" tem uma resposta só |
| Geração | Determinística por seed | Testes reprodutíveis + futuro "puzzle por código" |
| Licença | MIT (`LICENSE` na raiz) | Portfólio e estudo: uso livre, mantendo o aviso de copyright |
| Critério de dificuldade | Técnicas de resolução (solver "humano") | Mais justo que contar pistas |
| Muito Difícil | Sempre resolvível por lógica, no máximo 22 números | Difícil sem exigir chute; poucos números o fazem parecer mais vazio |
| Organização | Cargo workspace: `sudoku-core` + `sudoku-game` | Começou como crate único (YAGNI); virou workspace no módulo Game, quando o app e o servidor passaram a precisar das mesmas regras de partida |
| Dependências | `rand` (sem features padrão) + `rand_chacha` | Mínimo necessário; nada de Tauri/IO/UI nem aleatoriedade do SO |

**Padrão espinha-dorsal:** *Strategy* via trait `Technique` organiza as técnicas de
resolução; `enum` + `match` modelam os dados; `Result`/`Option` modelam falha e ausência.

## 4. Arquitetura interna do `sudoku-core`

O repositório é um **Cargo workspace**. Na raiz ficam o `Cargo.toml` do workspace (membros e
campos comuns: edition, MSRV, licença), o `Cargo.lock` e o `deny.toml` (regras do cargo-deny:
RustSec, licenças, origem), compartilhados por todos os crates.

Dentro do `sudoku-core`, submódulos isolados, cada um com propósito único e testável de forma independente:

```
sudoku-core/
├── Cargo.toml          publish = false, deps: rand 0.10 (sem default), rand_chacha 0.10
└── src/
    ├── lib.rs          declara os módulos e re-exporta a API pública
    ├── difficulty.rs   enum Difficulty (Facil < Medio < Dificil < MuitoDificil)
    ├── grid.rs         Grid, Cell; parsing/serialização; can_place; validate; helpers de unidade
    ├── candidates.rs   CandidateSet (bitmask u16), candidates_for, CandidateGrid (+ posições por unidade)
    ├── techniques/
    │   ├── mod.rs          trait Technique, Deduction, Action, units() (tabela fixa), helpers de linha
    │   ├── naked_single.rs     Fácil
    │   ├── hidden_single.rs    Fácil
    │   ├── naked_pair.rs       Médio
    │   ├── hidden_pair.rs      Médio
    │   ├── pointing_pair.rs    Médio
    │   ├── box_line_reduction.rs  Difícil
    │   ├── naked_triple.rs     Difícil
    │   ├── hidden_triple.rs    Difícil
    │   ├── x_wing.rs           Difícil
    │   ├── swordfish.rs        Muito Difícil
    │   └── xy_wing.rs          Muito Difícil
    ├── solver.rs       LogicalSolver (new, up_to, solve, next_step; hardest_step interno)
    ├── backtracking.rs count_solutions (unicidade por força bruta)
    ├── rating.rs       rate (classifica pela técnica mais avançada exigida)
    └── generator.rs    generate, Puzzle, GenError (grade completa → cava células)
```

Testes de integração em `sudoku-core/tests/`: `api.rs` (fluxos dos quatro níveis pela API pública) e
`desempenho.rs` (medição dos geradores de Médio, Difícil e Muito Difícil, `#[ignore]`, só em release).

Responsabilidades:

- **`grid`** — tipos base (`Grid`, `Cell = Empty | Filled(1..=9)`), parsing/serialização
  (string de 81 chars, `.` = vazio), `can_place`, detecção de conflitos linha/coluna/box.
- **`candidates`** — candidatos de uma célula como bitmask `u16` (bit *n* = dígito *n* possível).
  `CandidateGrid` guarda os candidatos de todas as células durante uma resolução, porque as
  eliminações das técnicas de Médio e Difícil não aparecem na grade. Guarda também, por unidade e
  dígito, a máscara das posições onde o dígito ainda cabe, atualizada em `place` e `eliminate`:
  as técnicas de unidade a leem em vez de recalculá-la a cada chamada.
- **`techniques`** — cada técnica implementa o trait `Technique` e devolve uma `Deduction`
  (uma `Action`, `Place` ou `Eliminate`, mais o nome da técnica e o nível). Uma técnica só
  devolve `Eliminate` se remover ao menos um candidato presente, para o solver não entrar em
  loop. Adicionar técnica nova **não toca em `solve`**.
- **`solver`** — `LogicalSolver` aplica as técnicas em ordem, da mais simples para a mais
  difícil, e produz a **lista de passos** usada pelo rating. `up_to(nível)` limita as técnicas;
  cada gerador usa `up_to(nível)`, e as técnicas novas entram no **fim** da lista: é isso que
  mantém inalterados os puzzles dos níveis já existentes. O gerador usa `hardest_step`, interno:
  a mesma resposta de `solve` (resolve? qual o nível mais alto?), mas colocando os singles em lote
  e sem montar a lista de passos.
- **`backtracking`** — `count_solutions` resolve por força bruta (ramificando na célula com
  menos candidatos — heurística MRV) e conta soluções até um
  limite (limite 2 ⇒ testa unicidade).
- **`rating`** — roda o solver lógico e mapeia a técnica mais avançada usada → `Difficulty`;
  cai para `MuitoDificil` quando só o backtracking resolve.
- **`generator`** — gera grade completa válida (backtracking randomizado **com seed**) e
  remove células enquanto o solver do nível alvo ainda resolve, o que também garante solução
  única. No Médio e no Difícil, repete com o mesmo RNG até o puzzle exigir o nível pedido, até
  um número fixo de tentativas (`MAX_ATTEMPTS_MEDIUM` = 50, `MAX_ATTEMPTS_HARD` = 250,
  parte do contrato de determinismo). O aceite equivale a `rate`, sem repetir a busca de
  unicidade. No Muito Difícil, em vez de recomeçar, **refina**: devolve 8 números da solução e
  recava, até 80 vezes por solução e 5 soluções (`REFILL_VERY_HARD`, `MAX_REFINEMENTS_VERY_HARD`,
  `MAX_SOLUTIONS_VERY_HARD`, também contrato).

### 4.1 Arquitetura do `sudoku-game`

Camadas no estilo *functional core, imperative shell*: o núcleo (`sudoku-core` e `sudoku-game`)
recebe dados e devolve dados; a casca (app Tauri, navegador, servidor do PvP) lê o relógio, grava
no disco, fala com a rede e chama o núcleo. Spec: `docs/superpowers/specs/2026-11-04-game-design.md`.

```
app Tauri + Svelte   ─┐
navegador (Wasm)     ─┼─→  sudoku-game  ─→  sudoku-core
servidor PvP         ─┘     (regras)         (gerar, resolver)
```

```
sudoku-game/
├── Cargo.toml          publish = false, deps: sudoku-core (path); serde opcional (feature `serde`)
├── src/
│   ├── lib.rs          re-exporta a API e os tipos do core que ela usa (Difficulty, Puzzle, Grid, Cell)
│   ├── config.rs       GameConfig { lives: NonZeroU8, hints: u8 }, padrão 3 vidas e 2 dicas
│   ├── game.rs         Game (new, from_puzzle, place, toggle_note, hint, pause, resume,
│   │                   elapsed_ms, save, restore, restart, result, abandon), CellState, Status,
│   │                   Restart, PlaceOutcome, HintOutcome, MoveError, PuzzleError
│   ├── notes.rs        Notes: anotações de uma célula (bitmask u16)
│   ├── save.rs         SavedGame (foto versão 1, campos simples), RestoreError, SAVE_VERSION
│   ├── result.rs       GameResult, Outcome, SavedResult (registro versão 1), ResultError, RESULT_VERSION
│   ├── stats.rs        LevelStats, level_stats e personal_ranking: funções puras sobre os resultados
│   └── hint.rs         splitmix64 e pick: sorteio determinístico da célula da dica
└── tests/
    ├── api.rs          partidas completas pela API pública, inclusive sair, voltar e o histórico
    ├── save_json.rs    ida e volta em JSON e golden test do formato (só com a feature serde)
    └── result_json.rs  golden test do JSON do resultado (só com a feature serde)
```

Regras da fatia 1: número certo trava; número errado tira uma vida e **sai do tabuleiro** (a mesma
regra do PvP); sem desfazer; zero vidas termina em derrota; tabuleiro completo termina em vitória;
2 dicas, sorteadas de forma determinística (`splitmix64(seed ^ número da dica) % vazias`); anotações
só em célula vazia, apagadas quando um número trava nela. Seed esgotada no gerador: o Game tenta a
seguinte. Ações recusadas devolvem `MoveError` e não mudam nada. `from_puzzle` confere o puzzle (solução
completa e válida, dados iguais à solução, pelo menos uma vazia) e devolve `PuzzleError` se não passar.

Regras da fatia 2: o tempo entra como `now: u64` (milissegundos de um relógio monotônico da casca).
Uma partida nova começa pausada, e "começar" é o primeiro `resume`. Pausada, ela recusa jogadas,
anotações e dicas com `MoveError::Paused`. `place` e `hint` recebem `now` para que a vitória ou a
derrota parem o relógio sozinhas. `save(now)` tira uma foto (`SavedGame`, versão 1) com campos
simples: a seed vai em texto, por causa do limite de 2^53 do JSON em JavaScript, e o status não vai,
porque é derivado. `restore` confere a foto inteira e devolve `RestoreError` se ela for incoerente, e
a partida restaurada volta pausada. Ele **não** confere se o puzzle vem da seed: no solo, quem edita o
arquivo só engana a si mesmo, e no PvP quem confere é o servidor. `restart(seed)` começa outro puzzle
do mesmo nível e com a mesma configuração. A feature opcional `serde` deixa o `SavedGame` serializável,
e um golden test trava o JSON da versão 1.

Regras da fatia 3: toda partida termina num `GameResult` (nível, seed, desfecho, tempo, erros,
dicas usadas, configuração e data). O desfecho é `Won`, `OutOfLives` ou `Abandoned`, e abandonar
conta como derrota. `result(finished_at_ms)` dá o resultado de uma partida terminada; é repetível, e
a casca o registra quando a partida termina, antes de salvar a foto. `abandon(self, now, data)`
consome a partida, e `restart(seed, now, data)` devolve em `abandoned` o resultado de uma partida
não terminada; uma terminada não dá um segundo resultado. A data vem do relógio de parede da casca
(milissegundos desde a época Unix, UTC), separada do `now` monotônico. `level_stats` (jogadas,
vitórias, porcentagem e melhor tempo) e `personal_ranking` (só vitórias, menor tempo primeiro, empate
para a mais antiga) são funções puras sobre a lista que a casca guarda. `SavedResult` (versão 1, um
número de versão por registro) guarda o resultado em campos simples, e um golden test trava o JSON.

## 5. Mapeamento dificuldade → técnica

Cada nível **inclui** as técnicas dos anteriores. O nível de um tabuleiro é a **técnica
mais avançada necessária** para resolvê-lo.

| Nível | Técnica mais avançada necessária |
|-------|----------------------------------|
| **Fácil** | Naked Single, Hidden Single |
| **Médio** | + Naked/Hidden Pairs, Pointing Pairs |
| **Difícil** | + Naked/Hidden Triples, Box-Line Reduction, X-Wing |
| **Muito Difícil** | + Swordfish, XY-Wing; no máximo 22 números |

> Os quatro níveis estão concluídos. O Pointing Pair também vale para 3 células alinhadas no
> box. A Box/Line Reduction fica no Difícil, e não no Médio, porque no Médio mudaria todos os
> puzzles Médios já gerados. O gerador de Muito Difícil só entrega puzzles que o solver lógico
> resolve por completo; já o `rate` continua chamando de `MuitoDificil` um puzzle de solução
> única que nem o solver completo resolve (só o backtracking).

## 6. API pública

```rust
pub enum Difficulty { Facil, Medio, Dificil, MuitoDificil }

pub struct Puzzle { givens: Grid, solution: Grid, difficulty: Difficulty, seed: u64 }

fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>;
fn rate(grid: &Grid) -> Option<Difficulty>;
fn validate(grid: &Grid) -> Vec<Conflict>;
fn count_solutions(grid: &Grid, limit: usize) -> usize;
fn candidates_for(grid: &Grid, index: usize) -> CandidateSet;

pub enum Action { Place { index: usize, value: u8 }, Eliminate(Vec<(usize, u8)>) }
pub struct Deduction { action: Action, technique: &'static str, level: Difficulty }
pub enum GenError { AttemptsExhausted(Difficulty) }
```

`generate` aceita os quatro níveis. O único erro é `AttemptsExhausted`, quando a seed esgota o
teto de trabalho do nível (o Game sorteia outra).

Tipos expostos: `Grid`, `Cell`, `Difficulty`, `Puzzle`, `Deduction`, `Action`, `Conflict`,
`CandidateSet`, `CandidateGrid`, `ParseError`, `GenError`, `LogicalSolver`, `SolveResult`, e as
técnicas `NakedSingle`, `HiddenSingle`, `NakedPair`, `HiddenPair`, `PointingPair`,
`BoxLineReduction`, `NakedTriple`, `HiddenTriple`, `XWing`, `Swordfish` e `XYWing` via trait
`Technique`.

**A dica do jogo não fica no core.** Ela sorteia uma célula vazia e copia o valor de
`Puzzle.solution`, então pertence ao módulo Game. O antigo `next_hint` foi removido.

## 7. Fluxo de geração

```
seed → RNG → grade resolvida completa
           → remover células enquanto o solver do nível resolve (o que garante unicidade)
           → classificar o nível (mesmo resultado de rate)
           → bate o nível alvo? sim → Puzzle | não → nova tentativa, até o limite → GenError

Muito Difícil (refina em vez de recomeçar):
seed → RNG → grade resolvida → cavar
           → ≤ 22 números e exige Swordfish/XY-Wing? sim → Puzzle
           → não → devolver 8 números da solução, recavar (até 80 vezes)
                   → ficou com ≤ números? vira o puzzle atual
           → nova solução, até 5 → GenError
```

Determinismo é contrato: **mesma seed + mesmo nível → puzzle idêntico, sempre** — inclusive
entre versões da lib, porque o RNG é o `ChaCha12Rng` (algoritmo fixo) e não o `StdRng`, cujo
algoritmo o `rand` pode trocar. Um *golden test* por nível trava a saída da seed 42 para
flagrar qualquer mudança acidental.

## 8. Stack e convenções

- **Rust** edition **2024**, num **Cargo workspace** (resolver 3) com os crates `sudoku-core` e `sudoku-game`. Dependências: **`rand` 0.10**, sem as
  features padrão (o core não usa StdRng nem a aleatoriedade do SO), e **`rand_chacha` 0.10** (`ChaCha12Rng`, de algoritmo fixo, para o determinismo da seed).
- Testes unitários **inline** em cada módulo (`#[cfg(test)] mod tests`); teste de
  integração da API pública em `sudoku-core/tests/`.
- Comandos rodam de qualquer pasta do repo, sem `--manifest-path`: `cargo test --workspace`,
  `cargo test -p sudoku-core`, `cargo fmt --all`, `cargo deny check`.
- Commits, branches e PRs em **inglês**, padrão Conventional Commits. Nomes no código em inglês
  (testes em português), pelas Rust API Guidelines; SOLID traduzido para Rust, erros e API pública:
  [`docs/convencoes-de-codigo.md`](convencoes-de-codigo.md).
- Código formatado com `rustfmt` (configuração padrão); finais de linha LF via `.gitattributes`.
- **CI** (GitHub Actions, `.github/workflows/ci.yml`): fmt, clippy, build e testes em
  Ubuntu/Windows/macOS a cada PR, com `-D warnings` e `--locked`. **CD** fica para o módulo
  Tauri: build do APK a cada tag `v*`, anexado a uma GitHub Release.
  Spec: `docs/superpowers/specs/2026-09-19-ci-design.md`.
- **Dependências** (`.github/workflows/dependencias.yml`): `cargo deny check` com as regras de
  `deny.toml` (raiz) a cada PR e toda segunda-feira. Ao adicionar uma dependência, o agente segue a
  skill `.claude/skills/adding-dependencies` (verificar o crate e pedir aprovação antes do `cargo add`).

## 9. Processo e skills

Metodologia **spec-driven + modular + TDD**. As skills que de fato conduzem o fluxo:

| Quando | Skill | Onde mora | Papel |
|--------|-------|-----------|-------|
| Cada fatia | `superpowers:brainstorming` | plugin superpowers | Do pedido ao spec em `docs/superpowers/specs/`. |
| Cada fatia | `superpowers:writing-plans` | plugin superpowers | Do spec ao plano em `docs/superpowers/plans/`. |
| Cada task | `superpowers:executing-plans` / `subagent-driven-development` | plugin superpowers | Executa o plano, task por task. |
| Cada task | `superpowers:test-driven-development` | plugin superpowers | Red-Green-Refactor; as regras do projeto (testes inline, nomes em português) estão no `CLAUDE.md`. |
| Fim da fatia | `superpowers:finishing-a-development-branch` | plugin superpowers | Verifica, faz o merge ou abre o PR. |
| Dependência nova | `adding-dependencies` | `.claude/skills/` | Verifica o crate e pede aprovação antes do `cargo add`. |
| Fim de cada fase | `improve-codebase-architecture` | `.claude/skills/` | Revisão de arquitetura: acha módulos rasos e propõe aprofundá-los, antes do spec da fase seguinte. |

As skills do projeto ficam em `.claude/skills/`, a pasta que o Claude Code carrega. A
`improve-codebase-architecture` é uma adaptação da de [mattpocock/skills](https://github.com/mattpocock/skills);
a seção "Neste repositório" do `SKILL.md` diz o que mudou. Ela usa dois artefatos do repositório:

- **[`CONTEXT.md`](../CONTEXT.md):** glossário do domínio (Grid, Deduction, Game, GameResult...).
  Atualizado quando um conceito ganha nome novo ou muda.
- **`docs/adr/`:** registros de decisões de arquitetura recusadas ou tomadas numa revisão, para não
  serem propostas de novo. Criada com o primeiro ADR; nenhum existe ainda.

Removidas em 2026-11-07, porque o Claude Code não as carregava e nunca rodaram: `agile-tdd`
(voltada para JS/Bun, mesmo papel do `superpowers:test-driven-development`) e `agile-roadmap`
(os specs e planos já fazem o sequenciamento).

Artefatos de processo do Engine:
- **Spec:** `docs/superpowers/specs/2026-06-14-sudoku-engine-design.md`
- **Plano (fatia Fácil):** `docs/superpowers/plans/2026-06-14-engine-facil-vertical-slice.md`
- **Spec (fatia Médio):** `docs/superpowers/specs/2026-11-01-engine-medio-design.md`
- **Plano (fatia Médio):** `docs/superpowers/plans/2026-11-01-engine-medio.md`
- **Spec (fatia Difícil):** `docs/superpowers/specs/2026-11-02-engine-dificil-design.md`
- **Plano (fatia Difícil):** `docs/superpowers/plans/2026-11-02-engine-dificil.md`
- **Spec (fatia Muito Difícil):** `docs/superpowers/specs/2026-11-03-engine-muito-dificil-design.md`
- **Plano (fatia Muito Difícil):** `docs/superpowers/plans/2026-11-03-engine-muito-dificil.md`

Artefatos de processo do Game:
- **Spec:** `docs/superpowers/specs/2026-11-04-game-design.md`
- **Plano (fatia 1, partida básica):** `docs/superpowers/plans/2026-11-04-game-partida.md`
- **Plano (fatia 2, tempo e salvamento):** `docs/superpowers/plans/2026-11-05-game-tempo-e-salvamento.md`
- **Plano (fatia 3, resultados):** `docs/superpowers/plans/2026-11-06-game-resultados.md`

## 10. Visão do módulo Online (PvP)

> Decisões de produto já tomadas, **ainda sem spec**. Quando o módulo começar, a spec parte
> daqui. O módulo Game/Sessão deve ser desenhado para rodar também no servidor: lógica pura,
> com o tempo entrando como parâmetro, nunca lido de um relógio interno.

O jogo também roda no **navegador**: o `sudoku-core` compila para WebAssembly e a UI Svelte é
a mesma do app. O PvP precisa de um **servidor autoritativo** (por exemplo, axum + WebSockets),
que também usa o `sudoku-core`.

| Tema | Decisão |
|------|---------|
| Sala | Exatamente 2 jogadores, entrada por código |
| Configuração | **Quem cria a sala** escolhe o nível e o limite de erros; o convidado entra já sabendo |
| Puzzle | O mesmo para os dois. O servidor gera, envia **só as pistas** e guarda a solução |
| Vitória | Quem completar primeiro, pelo **relógio do servidor** |
| Privacidade | Ninguém vê as jogadas do oponente: o servidor não as repassa |
| Progresso do oponente | Barra com células corretas ÷ células vazias no início, calculada pelo servidor |
| Erro | Número diferente da solução. Ao atingir o limite (ex.: 3), o jogador perde |
| Feedback de erro | Aviso na hora, e o número errado é removido do tabuleiro |
| Anotações (lápis) | Livres, nunca contam como erro. Na v1 ficam só no cliente; sincronizar depois |

Máquina de estados da sala:

```
Aguardando (1 jogador) → Contagem regressiva → Jogando → Finalizada
                                                  ↘ Abandonada (desconexão)
```

Em aberto: limpeza automática de anotações (opção do jogador?), tempo de espera por
reconexão antes do W.O. e defesas contra trapaça (quem tem as pistas pode usar um solver).

**Persistência.** A v1 não tem banco de dados: as salas vivem na memória do servidor e somem
ao terminar. Contas, ranking e histórico online ficam para uma versão futura, e é só aí que
entra um banco.

**Observabilidade.** O servidor é instrumentado com logs estruturados, métricas e traces (por
exemplo, o crate `tracing` exportando via OpenTelemetry). O `sudoku-core` continua puro, sem
instrumentação: o servidor mede o que acontece *em volta* das chamadas ao core (por exemplo,
o tempo de `generate` por nível).

**Geração fora do loop assíncrono.** `generate` é trabalho pesado de CPU (o Muito Difícil é o
mais caro). No servidor, ele roda em `tokio::task::spawn_blocking` (ou num pool dedicado), para
que a geração de uma sala não atrase as jogadas das outras salas, que dividem as mesmas
threads do runtime. Como a sala já gera o puzzle na criação, o convidado normalmente encontra
o puzzle pronto ao entrar.

## 11. Evolução futura

- Módulo Game trata `AttemptsExhausted` sorteando outra seed: cerca de 1 seed em 5.000 no
  Difícil e 1 em 200 no Muito Difícil.
- Módulo Game consumirá `validate` e implementará a dica, sorteando uma célula vazia e usando
  `Puzzle.solution`. Como o valor vem da solução, jogadas erradas não afetam a dica.
- A meta de tempo (até 5 s no celular, estimada como 1 s neste PC) foi medida só neste PC, em
  200 seeds, build de release. Depois da otimização da fatia Muito Difícil: pior caso de 15 ms no
  Médio e 431 ms no Difícil; no Muito Difícil, mediana de 119 ms e pior caso de 793 ms, que é o
  custo do teto inteiro (a seed 15 o esgota). Confirmar em um aparelho real quando o módulo Tauri
  existir.
- **Ideia: encher Fácil e Médio.** Hoje os três primeiros níveis param em cerca de 25 números,
  então visualmente quase não se distinguem. Devolver números da solução depois de cavar (como o
  refinamento faz ao contrário) deixaria os níveis fáceis mais cheios. Muda a saída desses níveis,
  então seria decisão consciente, com golden tests novos.
- **Módulo Game — timer + histórico de partidas:** cronometrar quanto o usuário leva para
  resolver e manter um histórico de partidas concluídas, ex.:
  - `Jogo Médio — 10 min — 13/06/2026`
  - `Jogo Muito Difícil — 17 min — 13/06/2026`

  O engine permanece **puro** (sem relógio nem IO): tempo, data e persistência são
  responsabilidade do módulo Game, que já recebe a `Difficulty` e o `seed` do `Puzzle`.
  Entra na **spec do módulo Game** quando ela for escrita.
- Seed pública habilita "compartilhar puzzle por código" (base do futuro multiplayer).
- ~~Promoção a workspace multi-crate só se houver necessidade real.~~ Feita no módulo Game (wiki 34).
