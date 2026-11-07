# 49 — Nomes salvos: um module de dentro do crate, `pub(crate)` e dependência entre irmãos

> Referente à Task 49 do plano do refactor do salvamento. Commit: `refactor: move saved level names into their own module`.

O nível aparece escrito por extenso em dois formatos salvos: na foto (`SavedGame`) e no resultado
salvo (`SavedResult`). As duas funções que fazem a tradução, `difficulty_name` e
`parse_difficulty`, moravam em `save.rs`, o module da foto. Por isso o `result.rs` dependia do
module da foto, sem ter nada a ver com ela. Esta task dá aos nomes um module próprio, `names.rs`.
É o primeiro dos dois passos do refactor que junta todo o formato da foto num lugar só (spec
`2026-11-07-save-module-design.md`).

## Um arquivo só vira module quando alguém o declara

```rust
// sudoku-game/src/lib.rs
mod config;
mod game;
mod hint;
mod names;   // novo
mod notes;
```

Criar `names.rs` não basta: o Cargo nem compila o arquivo enquanto o module pai (aqui, a raiz do
crate, `lib.rs`) não escrever `mod names;`. Quem monta a árvore de modules é o código, não a pasta.

Em JavaScript/TypeScript é o contrário: cada arquivo já é um módulo, e ele entra no programa quando
alguém o importa. Em Rust, `use` não carrega nada; ele só encurta um caminho que a árvore já tem:

```rust
// sudoku-game/src/result.rs
use crate::names::{difficulty_name, parse_difficulty};
```

`crate::` é a raiz do crate atual, como um caminho absoluto. `mod names;` sem `pub` deixa o module
privado: ninguém de fora do `sudoku-game` escreve `sudoku_game::names`, e o `lib.rs` não o
reexporta.

## `pub(crate)` nas funções, `mod` privado no module

As funções continuam `pub(crate)`, como eram em `save.rs` (o `pub(crate)` foi apresentado na
[página 37](37-anotacoes-e-bitmask.md)). São duas camadas de visibilidade:

- o **module** `names` é privado na raiz, então só o próprio crate chega até ele;
- as **funções** são `pub(crate)`, então qualquer module do crate pode chamá-las.

Com o module privado, até um `pub fn` ficaria invisível de fora. O `pub(crate)` serve de aviso
para quem lê: "isto é interno ao crate de propósito". Se alguém um dia escrever
`pub use names::difficulty_name` no `lib.rs`, o compilador recusa, porque um item `pub(crate)`
não pode virar público por reexportação.

A comparação com TypeScript mais próxima é um arquivo que tem `export`, mas que o `index.ts` do
pacote não reexporta. A diferença é que em TS isso é só convenção, e quem quiser importa o arquivo
pelo caminho. Em Rust, a regra é do compilador.

## Por que um module irmão não deveria importar de outro por conveniência

Antes:

```
result.rs ──difficulty_name──▶ save.rs (a foto)
```

Funcionava, mas dizia algo falso sobre o design: que o resultado depende da foto. Na próxima task,
`save.rs` vira filho de `game` (`game/save.rs`). Se os nomes tivessem ficado lá, o import viraria
`crate::game::save::difficulty_name`, e o resultado passaria a depender do module da partida.

Depois:

```
save.rs ───┐
           ├──▶ names.rs
result.rs ─┘
```

Os dois formatos dependem de um terceiro module que só sabe escrever e ler o nome de um nível.
Quando os dois precisam da mesma coisa e ela não pertence a nenhum, o lugar dela é fora dos dois.
Os nomes de desfecho (`"Won"`, `"OutOfLives"`...) continuam em `result.rs`, porque só o resultado
os usa.

## Mover um teste também exige vê-lo falhar

O teste `nome_do_nivel_vai_e_volta` foi junto com as funções, sem mudar uma linha. Como ele já
passava, a regra do `CLAUDE.md` para teste que passa de primeira vale aqui também: quebrar o código
de propósito no lugar novo. Trocar `"Medio"` por `"Medio2"` em `parse_difficulty` deu:

```
assertion `left == right` failed
  left: None
 right: Some(Medio)
```

Depois de desfazer, o teste voltou a passar. Isso prova que, no arquivo novo, o teste continua
testando a função certa. E as contagens de `test result` ficaram iguais às de antes do refactor,
com e sem `--all-features`: nenhum teste sumiu nem duplicou.
