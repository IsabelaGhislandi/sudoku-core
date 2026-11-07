# Formato do CONTEXT.md

O `CONTEXT.md` na raiz do repositório é o glossário do **domínio**: os nomes que o código, os specs e
a wiki usam para as coisas do Sudoku. Não é documentação de arquitetura (isso é o
`docs/architecture.md`) nem tutorial de Rust (isso é a wiki).

## Regras

- **Um termo por entrada**, com o nome exato do código em inglês (`Puzzle`, `Deduction`) e, se a
  conversa usa outro nome em português, ele entre parênteses ("foto", "casca").
- **Uma ou duas frases** dizendo o que o termo *é* e o que ele *não é*. A fronteira importa mais
  que a definição: `Given` não é `Placed`, embora os dois sejam números travados.
- **Onde mora:** o crate e o arquivo (`sudoku-game/src/result.rs`).
- **Agrupe por crate** (`sudoku-core`, `sudoku-game`, futuros) e, dentro dele, por conceito.
- **Termos evitados** ficam numa seção própria, com o substituto: "não diga *board*, diga *Grid*".
- Português no texto, nomes de código em inglês, como no resto do projeto.

## Quando mexer

- Um módulo novo, ou aprofundado, ganhou nome de conceito que não está aqui: acrescente.
- Um termo ficou ambíguo numa conversa e a ambiguidade foi resolvida: reescreva a entrada na hora.
- Um tipo foi renomeado ou removido no código: atualize no mesmo commit.

## Exemplo de entrada

```markdown
- **`GameResult`** (resultado): o registro de uma partida terminada ou abandonada. Só nasce da
  `Game` ou de um `SavedResult` válido, nunca montado à mão pela casca.
  `sudoku-game/src/result.rs`.
```
