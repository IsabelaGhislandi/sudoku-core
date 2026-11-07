# 50 — A foto como module filho: privacidade por árvore, `impl` em outro arquivo e `git mv`

> Referente à Task 50 do plano do refactor do salvamento. Commit: `refactor: move the saved game codec into a child module of game`.

Até aqui, a foto estava dividida em duas partes. O `save.rs` declarava o `SavedGame`, e a `game.rs`
escrevia, lia e conferia a foto (`save`, `restore`, `parse_notes`, cerca de 150 linhas, mais 24
testes). Esta task junta tudo num lugar só: `sudoku-game/src/game/save.rs`. A interface pública não
muda: `Game::save` e `Game::restore` continuam sendo métodos da `Game`, com a mesma assinatura, e
nenhum teste de `tests/` foi editado.

## Privado em Rust é "privado para a árvore"

O `restore` precisa escrever campos privados da `Game` (`cells`, `lives_left`, `status`...). Um
module irmão não os enxerga, e por isso o código tinha ficado na `game.rs`. A regra do Rust é:

> Um item privado é visível no module onde foi declarado **e em todos os descendentes dele**.

Então basta pôr a foto **dentro** de `game`:

```rust
// sudoku-game/src/game.rs
mod save;
pub use save::{RestoreError, SAVE_VERSION, SavedGame};
```

Como filho de `game`, o `save.rs` lê e monta os campos privados da `Game` sem que nenhum precise
virar `pub(crate)`. As alternativas eram piores:

- **campos `pub(crate)`:** qualquer module do crate (`stats`, `result`...) poderia alterar o estado
  da partida e pular as regras;
- **uma struct intermediária:** `into_parts` e `from_parts`, código só de passagem.

Em JavaScript, o equivalente a um campo privado é o `#campo` de classe. Ele é privado *para a
classe*, e nenhum outro arquivo o enxerga, nem um módulo "filho". O `private` do TypeScript é mais
frouxo: some na compilação, e `(obj as any).campo` passa. A privacidade do Rust não é nem uma coisa
nem outra: ela segue a **árvore de modules**, e quem decide a árvore é você.

## Um tipo, blocos `impl` em vários arquivos

```rust
// sudoku-game/src/game/save.rs
impl Game {
    pub fn save(&self, now: u64) -> SavedGame { /* ... */ }
    pub fn restore(saved: &SavedGame) -> Result<Game, RestoreError> { /* ... */ }
}
```

A `Game` é declarada na `game.rs`, mas um bloco `impl Game` pode morar em qualquer module do mesmo
crate. Para quem usa, é tudo a mesma `Game`: `game.save(now)` funciona igual.

E o module `save` é privado. Isso não esconde os métodos? Não. A visibilidade de um método vem do
`pub` **do método**, e não do module onde está o `impl`. Se o tipo é público, um `pub fn` dele
também é, onde quer que o bloco esteja.

Em TypeScript, uma classe fica num arquivo só. Para espalhar métodos, seria preciso *declaration
merging* ou mexer no `prototype`, e nenhum dos dois é comum. Em Rust, dividir os `impl` por assunto
é um jeito normal de dar a cada arquivo uma responsabilidade.

## `game.rs` e a pasta `game/`, sem `mod.rs`

```
src/
├── game.rs        o module `game`, que declara `mod save;`
└── game/
    └── save.rs    o module `game::save`
```

Desde a edition 2018, o filho de `game.rs` mora em `game/save.rs`, e não é preciso transformar
`game.rs` em `game/mod.rs`. O jeito antigo ainda funciona, mas deixava dezenas de abas chamadas
`mod.rs` no editor.

## `pub use` de dentro de um module privado

A API pública não mudou de forma. O `lib.rs` continua exportando `SavedGame`, `RestoreError` e
`SAVE_VERSION`, agora por `pub use game::{...}`. O caminho interno (`game::save::SavedGame`) é
detalhe de implementação: quem usa o crate escreve `sudoku_game::SavedGame`, como antes. É o
"barrel file" (`index.ts`) do TypeScript, com uma diferença: em Rust, o caminho interno é
inacessível de fora, e não só desencorajado.

## `pub(super)` nos helpers de teste

Os 24 testes da foto foram junto para o `mod tests` do `save.rs`, e alguns usam helpers que os
testes da partida também usam (`started`, `easy_game`, `solution_value`, `empties`, `wrong_value`,
`make_mistakes`). Eles ficaram na `game.rs`, marcados `pub(super)`:

```rust
// game.rs, dentro de `mod tests`
pub(super) fn easy_game() -> Game { /* ... */ }

// game/save.rs, dentro de `mod tests`
use crate::game::tests::{easy_game, empties, make_mistakes, solution_value, started, wrong_value};
```

`pub(super)` quer dizer "visível no module pai". O pai de `game::tests` é `game`, e o que `game`
enxerga os descendentes dele também enxergam. Por isso `game::save::tests` alcança os helpers, e o
resto do crate não.

## O que o `git mv` faz (e o que não faz)

`git mv sudoku-game/src/save.rs sudoku-game/src/game/save.rs` é só um atalho para `git rm` mais
`git add`. O Git **não guarda renomeações**: ele as deduz depois, comparando o conteúdo. Como o
arquivo cresceu de 74 para mais de 500 linhas, o `git show --stat` deste commit mostra um arquivo
apagado e outro criado. Para seguir a história do formato da foto, `git log --follow` ajuda, mas
num caso assim pode parar no commit do refactor.

## Como provar que um refactor não mudou nada

Um refactor não tem teste vermelho próprio, então a prova foi outra:

1. **Contagem igual:** as linhas `test result` de `cargo test --workspace`, com e sem
   `--all-features`, ficaram idênticas às de antes (123, 4, 83, 6... e 1 e 3 só com `serde`). A
   `game.rs` foi de 68 testes para 44, e o `save.rs` ficou com 24.
2. **Os testes movidos ainda pegam erro:** a primeira quebra planejada (aceitar dica repetida)
   **não** derrubou teste nenhum. A dica repetida já era recusada por outra conferência, porque na
   segunda visita a célula está `Hinted`, e não `Placed`. Quebrar a ordem das anotações em
   `parse_notes` (`d > previous` → `d >= previous`) derrubou
   `anotacao_repetida_ou_fora_de_ordem_eh_recusada`, como devia.
3. **Os corpos são os mesmos:** as linhas tiradas da `game.rs` e do `save.rs` antigo, comparadas
   com o `save.rs` novo, só diferem nos `use`, nos cabeçalhos e no `pub(super)` dos helpers.

O passo 2 também mostrou uma lacuna antiga: nenhum teste cobre dicas **fora de ordem** na foto
(`hinted = [b, a]`). Ela fica registrada para uma task própria, porque um refactor não muda testes.
