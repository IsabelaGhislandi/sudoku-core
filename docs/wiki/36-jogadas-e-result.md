# 36 — Jogadas: `Result` para regras do jogo, enums com dados e `?`

> Referente à Task 3 do plano da fatia 1 do Game. Commit: `feat: add number placement with lives, win and loss`.

A partida passou a aceitar jogadas: `game.place(index, value)` confere o número com a solução, trava
o número certo, tira uma vida no errado e termina a partida em vitória ou derrota.

## Erro de regra não é pânico

```rust
pub fn place(&mut self, index: usize, value: u8) -> Result<PlaceOutcome, MoveError>
```

Uma jogada pode ser recusada: a partida já acabou, o índice não existe, o número não é de 1 a 9, ou
a célula já tem número. Nenhum desses casos é bug. São coisas que uma tela, um toque duplo ou uma
mensagem do celular para o servidor podem pedir. Por isso `place` devolve um `Result`, e não usa
`panic!` nem `unwrap`. Quem chama decide o que fazer, por exemplo ignorar o toque ou mostrar um
aviso. No servidor do PvP isso importa ainda mais: uma mensagem maliciosa com `index = 500` não
pode derrubar o processo.

A fronteira ficou assim:

| Situação | Tratamento | Por quê |
|---|---|---|
| Jogada recusada | `Err(MoveError::...)` | Entrada de quem joga: esperada e recuperável |
| `game.cell(500)` | pânico (índice de array) | Erro de programação de quem lê o estado, como `Grid::get` |
| Solução com célula vazia | `unreachable!` | Impossível: `Game::from_puzzle` recusa um `Puzzle` assim (ver abaixo) |

Na web, é a diferença entre responder `400 Bad Request` e deixar uma exceção derrubar o servidor.

## Conferir na porta de entrada: `from_puzzle -> Result`

A primeira versão de `Game::from_puzzle` aceitava qualquer `Puzzle`. A revisão da branch mostrou o
problema: os campos de `Puzzle` são públicos, então dá para montar um à mão, ou recebê-lo de um
arquivo ou da rede. Com um enunciado sem célula vazia, a partida nunca terminava e a dica dividia
por zero. Com uma solução incompleta, o `unreachable!` acima seria alcançável.

A correção foi conferir **uma vez, na entrada**: `from_puzzle` devolve
`Result<Game, PuzzleError>` e recusa solução incompleta, solução com conflito, número dado
diferente da solução e enunciado sem célula vazia. Depois disso, o resto do código pode confiar no
puzzle. `Game::new` usa um construtor interno sem a conferência, porque o gerador sempre entrega
um puzzle válido. É o padrão *parse, don't validate*: em vez de checar em todo lugar, o tipo
`Game` só existe para puzzles que passaram pela porta.

## Duas camadas de resposta: `Result` com um enum dentro

```rust
pub enum PlaceOutcome {
    Correct,
    Wrong { index: usize, value: u8 },
}
```

Uma jogada errada **não é um `Err`**. Ela é aceita, segue a regra (custa uma vida) e muda a partida.
Por isso fica no `Ok`, como um resultado do jogo. O `Err` fica para o que **não aconteceu**. Essa
separação faz uma garantia aparecer no tipo: um `Err` nunca mudou nada, e o teste
`indice_ou_valor_invalido_nao_conta_erro` confere isso.

`Wrong { index, value }` é uma variante com **campos nomeados**, como uma struct dentro do enum. A
spec pede que a jogada errada devolva o número tentado, porque a célula já volta vazia e a tela
precisa saber o que pintar de vermelho. Com os dados dentro da variante, `Correct` não carrega
campos inúteis, e não dá para ler `value` de uma jogada certa por engano.

## O operador `?` e um helper de validação

```rust
pub fn place(&mut self, index: usize, value: u8) -> Result<PlaceOutcome, MoveError> {
    self.check_empty_cell(index, value)?;
    // daqui em diante, a jogada é válida
}
```

`check_empty_cell` devolve `Result<(), MoveError>`: `Ok(())` quer dizer "pode seguir". O `?` depois
da chamada desembrulha o `Ok` ou **retorna na hora** com o `Err`, como um `return` antecipado. É o
equivalente de um *guard clause* (`if (!valid) return error`), mas sem repetir o `if` em cada
chamada. A Task 4 (anotações) vai reusar o mesmo helper, e as duas ações recusam as mesmas coisas na
mesma ordem.

A **ordem** das verificações é parte da regra e está documentada no `///` de `place`: partida
terminada → índice → número → célula ocupada. Ela decide qual erro sai quando há mais de um. Uma
jogada com índice inválido numa partida terminada devolve `GameOver`, porque depois do fim nada
mais importa.

## `&mut self` e por que a vida não volta

`place` recebe `&mut self`, uma referência **exclusiva** à partida: enquanto ela roda, nenhum outro
código pode ler nem mudar a `Game`. Os campos `lives_left` e `mistakes` são privados, e nenhum
método os aumenta. Com isso, "uma vida perdida não volta" não depende de disciplina: não existe
caminho no código que faça isso.

O `lives_left -= 1` também é seguro. Um `u8` em 0 entraria em pânico ao subtrair em modo debug,
mas isso nunca acontece: com zero vidas, o status vira `Lost`, e a próxima jogada para no
`GameOver` antes de chegar à subtração.

## Testes com helpers em inglês

Os testes usam helpers como `solution_value`, `wrong_value` e `make_mistakes`. Seguindo o
`CLAUDE.md`, só o nome das funções `#[test]` fica em português. `wrong_value` escolhe "o número
certo + 1, ou 1 se o certo for 9", e assim é sempre errado e sempre de 1 a 9.
