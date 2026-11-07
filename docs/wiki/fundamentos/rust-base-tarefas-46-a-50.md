# Fundamentos de Rust — consolidado das Tarefas 46 a 50

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 46 a
> 50. As páginas por-task ([46](../46-abandonar-e-self.md) … [50](../50-module-filho-e-privacidade.md)) contam a
> história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 41 a 45](rust-base-tarefas-41-a-45.md).

Este bloco fecha a fase Game e arruma a casa antes do Frontend. As tasks 46 a 48 terminam o histórico de partidas:
abandonar, estatísticas e o resultado salvável. As tasks 49 e 50 são o primeiro **refactor** do projeto, sem
comportamento novo: o formato da foto passa a morar num lugar só. O fio que liga as cinco é **quem é dono de quê**:
quem é dono de um valor (`self`, `&self`, referências presas a uma lista) e quem é dono de um pedaço de código
(a árvore de modules e a visibilidade que ela dá).

Sumário:
1. [`self`, `&self` e `&mut self`: o tipo diz o que acontece com a partida](#1-self-self-e-mut-self-o-tipo-diz-o-que-acontece-com-a-partida)
2. [`Option` para "não existe", não para "deu erro"](#2-option-para-não-existe-não-para-deu-erro)
3. [Iteradores preguiçosos](#3-iteradores-preguiçosos)
4. [Ordenação estável e referências presas à lista](#4-ordenação-estável-e-referências-presas-à-lista)
5. [O mesmo padrão de DTO, com versão por registro](#5-o-mesmo-padrão-de-dto-com-versão-por-registro)
6. [A árvore de modules e a visibilidade](#6-a-árvore-de-modules-e-a-visibilidade)
7. [Refactor com prova](#7-refactor-com-prova)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. `self`, `&self` e `&mut self`: o tipo diz o que acontece com a partida

```rust
pub fn result(&self, finished_at_ms: u64) -> Option<GameResult>            // lê
pub fn place(&mut self, index: usize, value: u8, now: u64) -> ...           // altera
pub fn abandon(self, now: u64, finished_at_ms: u64) -> Option<GameResult>   // consome
```

A forma do primeiro parâmetro é uma promessa que o compilador cobra. `abandon(self)` **move** a partida para dentro
do método, e depois dele a variável não existe mais: uma jogada numa partida abandonada não compila. Não existe
status `Abandoned` nem `if` em cada ação. Em JavaScript, o objeto seguiria vivo, e só uma checagem em tempo de
execução impediria o uso ([46](../46-abandonar-e-self.md)).

`restart(&self)` continua só lendo, porque "jogar de novo" vale também com a tela de fim ainda aberta: quem chama
continua dono da partida antiga e decide quando trocá-la.

## 2. `Option` para "não existe", não para "deu erro"

O bloco usa `Option` em lugares onde, em JS, apareceria `undefined`, `NaN` ou um `0` enganoso:

- `Game::result` é `None` enquanto a partida não terminou ([45](../45-resultado-da-partida.md),
  [46](../46-abandonar-e-self.md));
- `(status == Playing).then(|| ...)` cria o `Some` só quando a condição vale, e a closure nem roda no outro caso
  ([46](../46-abandonar-e-self.md));
- `best_ms` é `None` sem vitória, e `win_percentage()` é `None` sem partidas. Isso evita a divisão por zero e
  também o "0%", que diria "perdeu todas" ([47](../47-estatisticas-e-iteradores.md));
- `parse_difficulty` devolve `Option`, e cada formato transforma o `None` no **seu** erro (`RestoreError` ou
  `ResultError`) ([48](../48-resultado-salvavel.md), [49](../49-modules-de-dentro-do-crate.md)).

A regra prática: `Option` quando a ausência é um estado normal, `Result` quando ela é uma falha que precisa de
motivo. E o helper genérico devolve `Option`, para não decidir o erro de quem chama.

## 3. Iteradores preguiçosos

```rust
let level = results.iter().filter(|r| r.difficulty == difficulty);
let wins = level.clone().filter(|r| r.outcome == Outcome::Won);
```

`filter` não percorre nada, só descreve. Quem percorre é o consumidor (`count`, `min`, `collect`), e ele **gasta**
o iterador: recebe `self`, como o `abandon`. Clonar um iterador copia a receita, e não os dados. É um *generator*
do JS, e não o `Array.prototype.filter`, que monta um array na hora ([47](../47-estatisticas-e-iteradores.md)).

## 4. Ordenação estável e referências presas à lista

`sort_by_key(|r| (r.elapsed_ms, r.finished_at_ms))` usa a tupla como chave, comparada campo a campo, e é
**estável**: empate total mantém a ordem da lista. `personal_ranking` devolve `Vec<&GameResult>`, referências e não
cópias, e o compilador prende a vida delas à da lista original. Um `drop(results)` com o ranking ainda em uso não
compila. Em JS, o coletor de lixo manteria os objetos vivos sozinho; em Rust, nenhuma referência aponta para
memória liberada ([47](../47-estatisticas-e-iteradores.md)).

## 5. O mesmo padrão de DTO, com versão por registro

O `SavedResult` repete, de propósito, o desenho do `SavedGame`: campos simples, `u64` como texto por causa do
limite de 2^53 do JSON em JS, e um `restore` que confere tudo antes de devolver o tipo de verdade. A diferença é
a versão. A foto é um arquivo só, sobrescrito a cada salvamento, então uma versão por arquivo basta. O histórico é
uma lista que só cresce, então **cada registro** leva a sua. Os testes de tabela usam closures sem captura como
ponteiros de função (`fn(&mut SavedResult)`), o `test.each` do Jest ([48](../48-resultado-salvavel.md)).

## 6. A árvore de modules e a visibilidade

Em JS, cada arquivo já é um módulo e entra no programa quando alguém o importa. Em Rust, quem monta a árvore é o
código:

```rust
// lib.rs
mod names;        // sem isto, names.rs nem compila
// game.rs
mod save;         // game/save.rs vira game::save
pub use save::{RestoreError, SAVE_VERSION, SavedGame};
```

As regras de visibilidade que este bloco usou ([37](../37-anotacoes-e-bitmask.md),
[49](../49-modules-de-dentro-do-crate.md), [50](../50-module-filho-e-privacidade.md)):

| Marca | Quem enxerga |
|---|---|
| (nada) | o module onde nasceu **e todos os descendentes** |
| `pub(super)` | o module pai (e, portanto, os descendentes dele) |
| `pub(crate)` | o crate inteiro, nunca quem usa o crate |
| `pub` | todo mundo que alcança o caminho |

Dessa tabela saem as duas decisões do refactor:

- **Os nomes salvos (`names.rs`) ficam fora da foto e do resultado**, porque não pertencem a nenhum dos dois.
  Um module irmão importar de outro por conveniência (`result → save`) dizia algo falso sobre o design.
- **A foto é filha da partida**, porque precisa dos campos privados da `Game`. Como filha, ela os enxerga sem
  que nenhum vire `pub(crate)`. Os métodos `save` e `restore` continuam na `Game`: um tipo pode ter blocos `impl`
  em vários arquivos do mesmo crate, e o `pub fn` vale pelo método, e não pelo module do bloco.

`pub use` reexporta o que importa e deixa o caminho interno inacessível. É o `index.ts` de um pacote, com o
compilador garantindo que ninguém importe por fora.

## 7. Refactor com prova

TDD pede ver o teste falhar. Num refactor, os testes já passam, então a prova muda de forma
([49](../49-modules-de-dentro-do-crate.md), [50](../50-module-filho-e-privacidade.md)):

1. **Contagem igual** antes e depois, com e sem features. Um teste esquecido ou duplicado aparece aí.
2. **Quebrar de propósito no lugar novo** e ver o teste movido falhar. Na Task 50, a primeira quebra escolhida
   não derrubou nada: a regra quebrada era coberta por outra conferência. Isso não é falha da prova, é informação:
   mostrou uma lacuna de teste (dicas fora de ordem) que já existia antes do refactor.
3. **Comparar os corpos**: as linhas que saíram e as que entraram só diferem em `use`, cabeçalhos e visibilidade.

## 8. Tabela-resumo dos conceitos

| Conceito | Onde aparece | Equivalente aproximado em JS/TS |
|---|---|---|
| `self` que consome | `Game::abandon` | sem equivalente; só checagem em runtime |
| `bool::then` | `abandoned_result` | `cond ? valor : undefined` |
| `Option` no lugar de `NaN`/`0` | `win_percentage`, `best_ms` | `number \| undefined` |
| Iterador preguiçoso | `level_stats` | generator |
| `sort_by_key` estável com tupla | `personal_ranking` | `sort` estável (ES2019) com comparador |
| Referência presa à lista | `Vec<&GameResult>` | sem equivalente; o GC mantém vivo |
| Versão por registro | `SavedResult.version` | campo `schemaVersion` em cada item |
| Ponteiro de função em tabela | testes do `SavedResult` | `test.each` |
| `mod` declarado pelo pai | `mod names;`, `mod save;` | arquivo vira módulo ao ser importado |
| Privado para a árvore | `game::save` lê campos da `Game` | `#campo` é privado só para a classe |
| `impl` em vários arquivos | `impl Game` em `game/save.rs` | *declaration merging* (raro) |
| `pub use` | `lib.rs` | `index.ts` reexportando |
| Prova de refactor | contagem + quebra proposital | snapshot de cobertura antes e depois |
