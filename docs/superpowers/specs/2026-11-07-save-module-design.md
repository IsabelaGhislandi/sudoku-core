# Spec — Refactor: a foto mora no module de salvamento

- **Data:** 2026-11-07
- **Status:** implementada (Tasks 49 e 50 do [plano](../plans/2026-11-07-save-module.md)).
- **Módulo:** 2 de 5 (Game/Sessão), depois do fim da fase. Refactor interno, sem comportamento novo.
- **Origem:** candidato A das revisões de arquitetura de 2026-11-06 e 2026-11-07 (skill
  `improve-codebase-architecture`), já listado como opcional na
  [spec do Game](2026-11-04-game-design.md), seção 12.6.

## 1. Objetivo e motivo

Hoje a foto (`SavedGame`) está partida em dois lugares:

- `sudoku-game/src/save.rs` só **declara** a foto: os campos, o `RestoreError`, o `SAVE_VERSION` e
  os nomes de nível. É um module shallow: a interface dele é quase tudo o que ele tem.
- `sudoku-game/src/game.rs` **monta e confere** a foto: `Game::save`, `Game::restore` e
  `parse_notes`, cerca de 155 linhas, mais uns 25 testes, misturados com as regras da partida.

E o `result.rs` busca os nomes de nível (`difficulty_name`, `parse_difficulty`) dentro do
`save.rs`, embora o resultado não tenha nada a ver com a foto.

O objetivo é que o formato da foto (escrever, ler, conferir e, no futuro, ler uma versão antiga)
fique **num lugar só**, e que a `game.rs` volte a ter só as regras da partida. Isso prepara a
`SAVE_VERSION` 2, que o Frontend deve pedir: ela muda um arquivo, e não dois.

## 2. O que muda

### 2.1 A foto vira um module filho da partida

`save` deixa de ser irmão de `game` e vira filho dele:

```
sudoku-game/src/
├── game.rs          regras da partida; declara `mod save;`
└── game/
    └── save.rs      SavedGame, RestoreError, SAVE_VERSION, parse_notes,
                     e o bloco `impl Game { pub fn save, pub fn restore }`
```

Por que filho: em Rust, um item privado é visível no module onde nasce **e nos descendentes**.
Como filho, o `save.rs` lê e monta os campos privados da `Game` sem que nenhum campo mude de
visibilidade. Com `pub(crate)` nos campos, qualquer module do crate (`stats`, `result`) poderia
mexer no estado da partida. Com uma struct intermediária, sobraria código só de passagem.

`Game::save` e `Game::restore` continuam sendo métodos da `Game`, com a mesma assinatura: só o
bloco `impl` muda de arquivo. Os métodos privados que o `restore` usa (`from_puzzle`,
`solution_value`) continuam na `game.rs`, e o filho os enxerga.

### 2.2 Os nomes salvos ganham um module próprio

Um module novo, `sudoku-game/src/names.rs`, `pub(crate)`, recebe `difficulty_name` e
`parse_difficulty`. A foto e o resultado passam a importar dele, e acaba a dependência
`result → save`.

Os nomes de desfecho (`outcome_name`, `parse_outcome`) **ficam** em `result.rs`, porque só o
resultado os usa. Se a foto um dia precisar deles, eles se mudam para `names.rs`.

### 2.3 Os testes acompanham o código

Os testes de foto e restauração saem do `mod tests` da `game.rs` e vão para o `mod tests` do
`game/save.rs`. Os helpers que os dois blocos usam (`easy_game`, `started`, `first_empty`,
`solution_value`...) ficam em `game.rs` como `pub(super)`, visíveis para os descendentes de
`game`. O teste `nome_do_nivel_vai_e_volta`, hoje em `save.rs`, vai para `names.rs`.

## 3. O que não pode mudar

- A **interface pública** do `sudoku-game`: os mesmos itens no `lib.rs`, com as mesmas
  assinaturas. Os testes de `sudoku-game/tests/` passam **sem edição**.
- A **foto versão 1**: o golden test de `tests/save_json.rs` passa sem edição, e o
  `SAVE_VERSION` continua 1.
- O **comportamento**: nenhum teste muda de asserção. A **quantidade de testes** do workspace é a
  mesma antes e depois, com e sem `--all-features`.
- O `sudoku-core`.
- Nenhuma dependência nova.

## 4. Como provar que é só refactor

Um refactor não tem vermelho de asserção próprio: os testes já existem e passam. Por isso a prova
é outra, e entra no plano como passo:

1. **Antes:** anotar as linhas `test result:` de `cargo test --workspace` e de
   `cargo test --workspace --all-features`.
2. **Depois:** as mesmas linhas, com as mesmas contagens.
3. **Os testes movidos ainda pegam erro:** quebrar de propósito uma conferência do `restore` já no
   lugar novo (por exemplo, aceitar dica repetida), ver o teste correspondente falhar e desfazer.
   É a regra do `CLAUDE.md` para teste que passa de primeira.

## 5. Tasks

Duas tasks `refactor`, cada uma com um commit `refactor` e um commit `docs` (wiki):

1. **Task 49 — Nomes salvos num module próprio** (`names.rs`). Vem primeiro, para que a mudança
   seguinte não arraste a dependência `result → save` para dentro de `game/`.
2. **Task 50 — A foto vira module filho da partida** (`game/save.rs`). Fecha com o consolidado das
   tasks 46 a 50 em `docs/wiki/fundamentos/`.

Atualizações de docs, no commit de cada task:

- `docs/architecture.md`, seção 4.1: a árvore do `sudoku-game`.
- `CONTEXT.md`: o caminho do `SavedGame` passa a ser `sudoku-game/src/game/save.rs`.
- Wiki 49 e 50, indexadas no `docs/wiki/README.md`.

## 6. Fora deste refactor

- O candidato D da revisão de 2026-11-06 (derivar vidas, dicas e status em vez de guardá-los).
- O `Puzzle` com garantia própria no core (candidato B da revisão de 2026-11-07).
- A leitura única da partida para a casca (candidato C): fica para o spec do Frontend.
