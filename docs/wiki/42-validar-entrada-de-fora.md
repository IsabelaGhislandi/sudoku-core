# 42 — Validar o que vem de fora: "parse, don't validate", índices conferidos e `let ... else`

> Referente à Task 3 do plano da fatia 2 do Game. Commit: `feat: reject inconsistent snapshots on restore`.

A foto da partida (wiki 41) vem do disco, e um arquivo pode estar corrompido ou ter sido editado
à mão. Esta task fez `Game::restore` recusar toda foto incoerente: vida "recuperada", número errado
travado, anotação em célula preenchida, dica fora do tabuleiro.

## "Parse, don't validate"

Há dois jeitos de lidar com dado de fora:

1. **Validar:** uma função `is_valid(&foto) -> bool`, e o resto do código torce para alguém ter
   chamado antes.
2. **Converter (parse):** uma função `restore(&foto) -> Result<Game, RestoreError>`, que ou entrega
   uma `Game` pronta, ou um erro.

O segundo jeito é o do Rust. Como os campos da `Game` são privados, o único caminho de uma foto
até uma partida passa pelas checagens. O resto do código nunca recebe uma `Game` "talvez válida".
A frase "parse, don't validate" vem de um artigo de Alexis King e resume isso: em vez de checar e
jogar a informação fora, transforme o dado num tipo que só existe se a checagem passou.

## Dado de fora nunca indexa um array sem conferir

Antes desta task, uma foto com `"hinted": [81]` derrubava o programa:

```
panicked at sudoku-core\src\grid.rs:20:9: index out of bounds: the len is 81 but the index is 81
```

Em Rust, indexar fora do array não corrompe a memória como em C: ele **para o programa** com
pânico. Isso é seguro, mas, para quem joga, um arquivo ruim fecharia o app. Por isso todo índice
que vem da foto é conferido antes de ser usado:

```rust
let i = usize::from(i);
if i >= 81 || previous.is_some_and(|p| i <= p) {
    return Err(RestoreError::InvalidHints);
}
```

O teste que trava isso tenta `81` e `255`, o maior `u8`.

## `let ... else` para exigir um estado

Uma dica só pode estar numa célula que, pelo tabuleiro, foi preenchida por jogada:

```rust
let CellState::Placed(v) = game.cells[i] else {
    return Err(RestoreError::InvalidHints);
};
game.cells[i] = CellState::Hinted(v);
```

`let ... else` (wiki 31) casa o padrão e extrai o `v`, ou sai da função. É mais direto que um
`match` com dois braços, um deles só para devolver o erro.

## Underflow evitado com `&&`

As vidas fecham se `vidas_restantes = vidas_iniciais − erros`. Escrito sem cuidado, isso daria
underflow com uma foto que tenha mais vidas restantes que iniciais:

```rust
let lives_ok = saved.lives_left <= saved.lives
    && saved.mistakes == u32::from(saved.lives - saved.lives_left);
```

O `&&` é de **curto-circuito**: se a primeira parte for falsa, a segunda nem roda, e a subtração só
acontece quando é segura. É o mesmo comportamento do `&&` do JavaScript, usado aqui como guarda.

## Erro que embrulha outro erro

O puzzle da foto passa pelo mesmo `from_puzzle` da fatia 1, que tem o seu próprio erro. Em vez de
duplicar as variantes, o `RestoreError` embrulha o outro:

```rust
pub enum RestoreError {
    Puzzle(PuzzleError),
    BoardMismatch,
    // ...
}

Game::from_puzzle(puzzle, config).map_err(RestoreError::Puzzle)?;
```

`RestoreError::Puzzle` usado sem argumentos é uma **função** `PuzzleError -> RestoreError`, então
pode ser passado direto para o `map_err`, sem closure.

## Testes de adulteração campo a campo

Cada regra ganhou um teste que parte de uma foto válida, estraga **um** campo e confere o erro
exato:

```rust
let mut saved = valid_save();
saved.lives_left += 1;
assert_rejected(&saved, RestoreError::InvalidLives);
```

Mudar um campo só garante que o teste prova aquela regra, e não outra que falhou antes. E o teste
`foto_valida_restaura_sem_erro` garante que a foto de partida está mesmo válida. Sem ele, todos os
outros poderiam passar por acidente, recusando uma foto que já nascia ruim.
