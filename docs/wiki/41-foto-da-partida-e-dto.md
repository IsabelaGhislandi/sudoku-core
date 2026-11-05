# 41 — Foto da partida: um DTO, `parse` com `Result` e o limite de 2^53

> Referente à Task 2 do plano da fatia 2 do Game. Commit: `feat: add SavedGame snapshot with save and restore`.

Fechar o app no meio da partida não pode custar nada. Esta task criou a "foto" da partida, que a
casca grava no disco, e o caminho de volta: `game.save(now)` tira a foto e `Game::restore(&foto)`
remonta a partida.

## Por que um tipo separado (DTO)

O jeito mais curto seria marcar a própria `Game` como serializável. O problema é que a `Game` tem
**regras**: campos privados, vida que não volta, número certo que trava. Se ela pudesse ser lida
direto de um arquivo, qualquer arquivo viraria partida, inclusive um com 99 vidas.

Por isso existe o `SavedGame`, um **DTO** (*data transfer object*): um tipo só para transporte, com
campos públicos e simples, que não promete nada.

```rust
pub struct SavedGame {
    pub version: u32,
    pub difficulty: String,   // "Facil", "Medio"...
    pub seed: String,         // u64 em decimal
    pub givens: String,       // 81 caracteres, como Grid::to_line
    pub board: String,
    pub hinted: Vec<u8>,
    pub notes: Vec<String>,   // "", "37"...
    // vidas, dicas, erros, tempo...
}
```

O caminho de volta é **a única porta**: `Game::restore` devolve `Result<Game, RestoreError>`, então
a `Game` só existe se a foto passar. No mundo web é parecido com separar o JSON que chega numa API
(o "body" do request) do modelo de domínio, e validar na fronteira.

De quebra, os tipos do `sudoku-core` (`Grid`, `Puzzle`, `Difficulty`) não precisam saber serializar:
a foto usa o formato de texto que a `Grid` já tinha (`to_line`/`from_line`).

## Converter texto com `Result`

Toda conversão de texto pode falhar, e o Rust deixa isso no tipo:

```rust
let seed: u64 = saved.seed.parse().map_err(|_| RestoreError::InvalidSeed)?;
let givens = Grid::from_line(&saved.givens).map_err(|_| RestoreError::InvalidGrid)?;
```

- `str::parse::<u64>()` devolve `Result<u64, ParseIntError>`. `"abc"`, `""` e `"-1"` falham.
- `map_err` troca o erro de baixo nível por um erro do domínio, que a casca entende.
- O `?` (wiki 36) sai da função no primeiro erro.

O nível também é texto. A conversão para `Difficulty` é um `match` **exaustivo** nos dois sentidos:
se o core ganhar um quinto nível, `difficulty_name` deixa de compilar até alguém dar um nome a ele
(o mesmo alarme da wiki 32).

## O limite de 2^53 do JSON

Em JavaScript, todo número é um `f64`, que representa inteiros exatos só até 2^53 (cerca de 9
quatrilhões). A seed é um `u64`, que vai até 2^64. Um JSON com `"seed": 18446744073709551615`, lido
por `JSON.parse` no Svelte, viraria outro número, e a partida restaurada seria **outro puzzle**.

Gravar a seed como texto (`"seed": "18446744073709551615"`) evita o problema. O tempo jogado
(`elapsed_ms`) continua número, porque 2^53 milissegundos são 285 mil anos.

## O que a foto não guarda

- **O status.** Ele é derivado: tabuleiro completo é vitória, zero vidas é derrota, senão a partida
  está em andamento. Se fosse salvo, uma foto poderia dizer "vitória" com células vazias. Sem o
  campo, esse tipo de incoerência nem pode ser escrito.
- **O relógio rodando.** A partida restaurada volta sempre pausada. Salvar no meio do jogo inclui o
  trecho até `now` no tempo, mas não pausa a partida original: `save` recebe `&self`, então nem
  conseguiria mudá-la.

## Determinismo depois de restaurar

A célula de uma dica depende só da seed e de quantas dicas já foram usadas (wiki 38). Os dois vão
na foto, então a próxima dica de uma partida restaurada cai na mesma célula que cairia na
original. O teste `dica_depois_de_restaurar_cai_na_mesma_celula` trava isso.

A próxima task fecha a porta de vez: hoje, uma foto com um número errado travado ainda passaria.
