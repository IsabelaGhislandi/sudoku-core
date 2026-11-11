# Glossário do domínio

Os nomes que o código, os specs e a wiki usam para as coisas do Sudoku. Nomes de código em inglês;
entre parênteses, o nome que aparece na conversa e nos docs em português. O design está em
[`docs/architecture.md`](docs/architecture.md); o formato deste arquivo, em
[`.claude/skills/improve-codebase-architecture/CONTEXT-FORMAT.md`](.claude/skills/improve-codebase-architecture/CONTEXT-FORMAT.md).

## Tabuleiro (`sudoku-core`)

- **`Grid`** (grade): as 81 células de um tabuleiro, endereçadas por **índice** de 0 a 80, linha por
  linha. Não sabe de quem é cada número: enunciado e jogada são iguais para ela.
  `sudoku-core/src/grid.rs`.
- **`Cell`** (célula): `Empty` ou `Filled(u8)`, com valor de 1 a 9. É a célula do core; a da
  partida é o `CellState`.
- **Unidade** (row, column, box): linha, coluna ou caixa 3×3. Toda regra de Sudoku é "sem repetir
  na unidade". `row_of`, `col_of`, `box_of` dão a unidade de um índice.
- **`Conflict`** (conflito): dois números iguais na mesma unidade. Resultado de `validate`.
- **Linha de texto**: a grade em 81 caracteres (`Grid::from_line` / `to_line`). Formato de testes e
  de transporte, não de tela.

## Candidatos e dedução (`sudoku-core`)

- **Candidato**: um valor que ainda pode ir numa célula vazia sem repetir na unidade.
  **`CandidateSet`** guarda os de uma célula; **`CandidateGrid`**, os de todas.
  `sudoku-core/src/candidates.rs`.
- **`Technique`** (técnica): uma regra lógica de resolução (naked single, x-wing...), no pattern
  Strategy. Cada uma tem um nível. `sudoku-core/src/techniques/`.
- **`Deduction`** (dedução): o que uma técnica achou: uma **`Action`** (`Place` um valor ou
  `Eliminate` candidatos), o nome da técnica e o nível dela.
- **`LogicalSolver`** (solver): aplica as técnicas da mais simples para a mais difícil até travar ou
  resolver. Devolve um **`SolveResult`** com os passos. Não chuta; quem chuta é o backtracking.
- **Backtracking**: busca por força bruta, usada só para contar soluções (`count_solutions`). Não
  diz nada sobre dificuldade.

## Puzzle e dificuldade (`sudoku-core`)

- **`Difficulty`** (nível): `Facil`, `Medio`, `Dificil`, `MuitoDificil`, em ordem. Os nomes em
  português são exceção consciente à regra de nomes em inglês (`CLAUDE.md`).
- **Rating** (classificação): o nível de um puzzle é o da técnica mais difícil que o solver precisou
  (`rate`). Não é contagem de pistas.
- **`Puzzle`**: `givens` (o enunciado), `solution`, `difficulty` e a `seed` de fato usada. Tem
  solução única.
- **Seed**: o `u64` que, com o nível, determina o puzzle. **Determinismo** é contrato: mesma seed e
  mesmo nível dão o mesmo puzzle, entre versões. `generate`, `GenError::AttemptsExhausted`.

## Partida (`sudoku-game`)

- **`Game`** (partida): um puzzle e tudo o que a pessoa fez nele. Recebe o tempo de fora (`now`),
  nunca lê relógio. `sudoku-game/src/game.rs`.
- **`GameConfig`** (configuração, regras): vidas (`NonZeroU8`) e dicas. O padrão do solo é 3 e 2.
- **`CellState`** (estado da célula): `Empty`, `Given` (enunciado), `Placed` (jogada certa) ou
  `Hinted` (revelada por dica). Os três últimos são travados. Não existe estado "errado".
- **Jogada** (`place`): colocar um valor. Certa trava a célula; errada (**erro**, `mistakes`) custa
  uma **vida** e não fica no tabuleiro.
- **`Notes`** (anotações): os valores que a pessoa marcou a lápis numa célula. Não são candidatos:
  o jogo não as calcula. Quando um número certo trava, apaga as anotações da célula e a anotação
  desse número na linha, na coluna e na caixa dela.
- **Dica** (`hint`): revela o valor certo de uma célula vazia escolhida de forma determinística pela
  seed e pelo número da dica.
- **`Status`**: `Playing`, `Won` ou `Lost`. Os dois últimos são finais.
- **Tempo jogado** (`elapsed_ms`): soma dos trechos entre `resume` e `pause`, no relógio monotônico
  da casca. Partida nova começa pausada.
- **Reiniciar** (`restart`, `Restart`): partida nova com as mesmas regras; se a antiga não tinha
  terminado, ela vira um abandono.

## Salvamento e histórico (`sudoku-game`)

- **`SavedGame`** (foto): a partida em campos simples, versionada (`SAVE_VERSION`), sem garantia
  nenhuma até passar por `Game::restore`. `sudoku-game/src/game/save.rs`.
- **`GameResult`** (resultado): o registro de uma partida terminada ou abandonada: nível, seed,
  **`Outcome`** (`Won`, `OutOfLives`, `Abandoned`), tempo, erros, dicas, configuração e data. Só
  nasce da `Game` ou de um registro salvo. `sudoku-game/src/result.rs`.
- **`SavedResult`** (resultado salvo): o resultado em campos simples, versionado (`RESULT_VERSION`).
- **Data** (`finished_at_ms`): relógio de **parede** da casca, em UTC. Não confundir com o `now`
  monotônico das jogadas, que mede duração.
- **Histórico**: a lista de `GameResult` que a casca guarda. O core não guarda lista nenhuma.
- **`LevelStats`** (estatísticas): partidas, vitórias, melhor tempo e porcentagem de vitórias de um
  nível. Abandono conta como partida não vencida. `sudoku-game/src/stats.rs`.
- **Ranking pessoal** (`personal_ranking`): as vitórias de um nível, do menor tempo para o maior.

## Ponte para o navegador (`sudoku-wasm`)

- **`WebGame`**: a `Game` vista do JavaScript. Casca fina, sem regra própria: converte texto e
  números do JS e repassa para a `Game`. `sudoku-wasm/src/lib.rs`.
- **`GameView`** (retrato): tudo o que a tela desenha num instante (nível, status, pausa, tempo,
  vidas, dicas e as 81 células com anotações), em JSON. `sudoku-wasm/src/view.rs`.

## Fora do núcleo

- **Casca** (shell): quem usa o núcleo e faz o IO: o app Tauri, o navegador, o servidor do PvP. Lê
  relógio, sorteia seed, grava foto e histórico. Ver `docs/architecture.md`, seção 4.1.
- **PvP**: o modo online futuro, duas pessoas no mesmo puzzle. Ver `docs/architecture.md`, seção 10.

## Termos evitados

- *board*, *tabuleiro* no código: diga `Grid` (core) ou `Game` (partida).
- *square*, *cell value*: diga célula e valor.
- *errado* como estado de célula: não existe; um erro é um evento (`PlaceOutcome::Wrong`).
- *save* para o resultado: a foto é `SavedGame`; o resultado salvo é `SavedResult`.
