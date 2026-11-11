# 62 — Anotação em conflito: variante nova num enum público, consulta separada da ação e iteradores preguiçosos

> Referente à Task 62 do plano de regras de anotação. Commit: `feat: refuse a note that clashes with its row, column or box`.

No modo Anotar, tentar anotar um número que já está na linha, na coluna ou na caixa da célula não
faz nada no tabuleiro, a não ser mostrar o porquê: as células que já têm esse número piscam três
vezes em vermelho. Apagar uma anotação continua sempre permitido.

## Uma variante nova na `MoveError`

A recusa ganhou um nome no enum de recusas da partida:

```rust
pub enum MoveError {
    GameOver,
    // ...
    Paused,
    /// O número já está na linha, na coluna ou na caixa da célula, então não pode ser anotado
    /// ali. `Game::conflicts` diz em quais células ele está.
    NoteConflict,
}
```

A `MoveError` é pública e **não** tem `#[non_exhaustive]` (a `Restart` tem, wiki 44). Isso quer
dizer que quem estiver fora do crate e fizer um `match` cobrindo todas as variantes deixa de
compilar quando entra uma nova: o compilador avisa que falta tratar `NoteConflict`. Para uma
biblioteca publicada, isso é uma mudança que quebra quem usa (pediria versão nova "major"). Aqui
quem usa é o próprio workspace, e ninguém faz um `match` completo na `MoveError`, então nada quebrou.

Foi uma escolha: deixar o enum exaustivo obriga cada casca a decidir o que fazer com cada recusa.
No TypeScript, uma união como `'GameOver' | 'Paused'` ganha a mesma checagem com um `switch` e um
`never` no `default`, mas só se alguém escrever o `never`. No Rust, a checagem vem de graça.

Do outro lado da ponte, nada precisou mudar. O `sudoku-wasm` transforma qualquer recusa em texto
com o `Debug` (`format!("{error:?}")`), então a variante nova já chega ao JS como a string
`"NoteConflict"`.

## A consulta separada da ação

A tela precisa saber **quais** células piscar. A primeira ideia seria a recusa trazer a lista:
`NoteConflict(Vec<usize>)`. Só que a `MoveError` é `Copy`, um valor pequeno que se copia sem
custo, e um `Vec` não pode ser `Copy` porque aponta para memória alocada. Tirar o `Copy` do enum
afetaria todo o código que o usa.

A solução foi uma consulta pública separada, que a própria ação também usa:

```rust
pub fn conflicts(&self, index: usize, value: u8) -> Vec<usize> {
    assert!(index < 81, "célula {index} fora do tabuleiro");
    (0..81)
        .filter(|&i| sees(index, i) && self.cells[i].value() == Some(value))
        .collect()
}

pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), MoveError> {
    self.check_empty_cell(index, value)?;
    if !self.notes[index].contains(value) && !self.conflicts(index, value).is_empty() {
        return Err(MoveError::NoteConflict);
    }
    self.notes[index].toggle(value);
    Ok(())
}
```

É a separação entre **comando** (muda o estado e diz se deu certo) e **consulta** (só lê e
responde). Como o `toggle_note` decide usando o próprio `conflicts`, a regra existe num lugar só:
a tela pisca exatamente as células que fizeram a partida recusar. No React, seria parecido com
uma função pura `getConflicts(board, index, value)` usada tanto pelo reducer quanto pelo
componente.

## `assert!` contra uma resposta errada em silêncio

Na primeira versão, `conflicts(81, 1)` não dava erro nenhum. A conta de coluna (`81 % 9 == 0`)
achava a coluna 0, e o método devolvia células de verdade para uma célula que não existe. Por isso
entrou o `assert!` com mensagem, e um teste `#[should_panic(expected = "fora do tabuleiro")]` que
falhou antes dele existir.

A regra do projeto é `Result` para falha esperada e pânico para erro de programação. Pedir os
conflitos de uma célula inexistente é erro de programação, como `cell(81)`, que também entra em
pânico. Pior do que quebrar seria responder errado e seguir em frente.

## Desligar é sempre permitido: fotos antigas

A checagem só vale para **ligar** a anotação (`!self.notes[index].contains(value)`). Uma partida
salva antes desta regra pode ter uma anotação em conflito, e quem abrir essa foto precisa
conseguir apagá-la. O teste `apagar_anotacao_em_conflito_eh_permitido` monta exatamente isso: tira
a foto, escreve uma anotação em conflito no campo `notes` (que é público no `SavedGame`, wiki 41),
restaura e apaga. Ele passou de primeira, e a prova foi tirar a condição: sem ela, veio
`Err(NoteConflict)`.

O golden test da foto (wiki 43) anotava um 5 numa célula que agora conflita. O **formato** da foto
não mudou; mudou o cenário: o teste anota um 2, e o texto travado mudou só nesse caractere. A
regra do golden é "mudou? É bug ou decisão consciente no mesmo commit", e a mensagem do commit
registra isso.

## Iteradores preguiçosos: `filter` com efeito e `take(2)`

O teste da ponte precisava de dois números que a célula aceita anotar, e a ponte não tem um
`can_note`. O jeito foi tentar:

```rust
let noted: Vec<u8> = (1..=9)
    .filter(|&v| game.toggle_note(index, v).is_ok())
    .take(2)
    .collect();
```

Isso só funciona porque os iteradores do Rust são **preguiçosos**: o `filter` não roda sozinho, só
quando alguém pede o próximo item. O `take(2)` para de pedir depois do segundo aceito, então o
`toggle_note` roda até achar dois números, e não nos nove.

No JS, `[1, ..., 9].filter(...)` é **ansioso**: percorre o array inteiro antes do `.slice(0, 2)`, e
o mesmo código anotaria todos os números aceitos. Para ter o comportamento do Rust seria preciso
um gerador (`function*`) ou os iterator helpers novos (`Iterator.prototype.filter`).

Mesmo assim, `filter` com efeito colateral não é um hábito para código de produção. Num teste
curto, com um comentário dizendo o que acontece, ele economiza um helper.

## Na tela: o erro do WASM vira `throw`, e a animação recomeça

Um `Result<(), String>` exportado com `wasm-bindgen` vira, no JS, uma função que **lança** a
string quando dá `Err`. Por isso a tela usa `try/catch` e compara com o nome:

```ts
function noteOrFlash(index: number, value: number) {
  try {
    game.toggleNote(index, value)
  } catch (error) {
    if (error !== 'NoteConflict') throw error
    flashConflicts(JSON.parse(game.conflicts(index, value)))
  }
}
```

As outras recusas são relançadas e caem no `act`, que já as ignorava. Os conflitos chegam como
array JSON, mantendo a regra da ponte de que tudo que cruza é texto (wiki 51).

Para piscar, o `Board` ganha a classe `conflict` com uma animação CSS de três ciclos em 900 ms.
Um detalhe: se a pessoa tenta de novo enquanto ainda pisca, a classe já está lá e a animação não
recomeça. O `flashConflicts` então esvazia a lista e só a preenche no quadro seguinte, com
`requestAnimationFrame`. Assim o navegador vê a classe sair e voltar. A wiki 59 resolveu o mesmo
problema com `{#key}`.

Quem ativou "reduzir movimento" no sistema (`prefers-reduced-motion: reduce`) vê o fundo vermelho
parado, sem piscar.
