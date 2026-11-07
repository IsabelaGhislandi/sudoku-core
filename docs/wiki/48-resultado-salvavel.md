# 48 — Resultado salvável: o mesmo padrão de DTO, versão por registro e o fim da fatia 3

> Referente à Task 4 do plano da fatia 3 do Game. Commit: `feat: make game results savable with a versioned format`.

O histórico só existe se a casca conseguir gravar os resultados e lê-los de volta meses depois.
Esta task dá ao `GameResult` um formato salvável, no mesmo molde da foto da partida (página 41).

## O mesmo padrão, de propósito

| | Foto da partida | Resultado |
|---|---|---|
| Tipo do domínio | `Game` | `GameResult` |
| DTO de campos simples | `SavedGame` | `SavedResult` |
| Ida | `game.save(now)` | `result.save()` |
| Volta, com conferência | `Game::restore` → `RestoreError` | `GameResult::restore` → `ResultError` |
| `serde` | só no DTO, com a feature | só no DTO, com a feature |

Repetir uma forma que já funciona tem valor próprio: quem leu a página 41 entende esta task sem
aprender nada novo, e a casca trata os dois arquivos do mesmo jeito. Seed e data vão como texto
(limite de 2^53 do JavaScript), e os tipos do `sudoku-core` continuam sem `serde`.

## `Option` no helper, erro no chamador

```rust
pub(crate) fn parse_difficulty(name: &str) -> Option<Difficulty>
```

Antes, `parse_difficulty` devolvia `Result<Difficulty, RestoreError>`, o erro da foto. Agora os dois
DTOs usam o mesmo nome de nível, mas cada um tem o seu erro. O helper passou a dizer só "achei" ou
"não achei", e cada chamador escolhe o erro com `ok_or`:

```rust
parse_difficulty(&saved.difficulty).ok_or(ResultError::UnknownDifficulty)?;
```

É uma regra geral: uma função reaproveitada em lugares diferentes devolve o dado mínimo
(`Option`, ou um erro próprio dela), e quem chama traduz para o seu contexto. Como o helper é
`pub(crate)`, a mudança não afeta ninguém fora do crate.

## Versão em cada registro

A foto é um arquivo só, sobrescrito a cada salvamento: uma versão para o arquivo basta. O histórico
é uma **lista que só cresce**, e pode guardar registros gravados por versões diferentes do app.
Com `version` em cada registro, uma versão 2 do formato convive com os registros antigos na mesma
lista, e cada um é lido pela regra da sua versão.

O golden test (`tests/result_json.rs`) trava o JSON da versão 1. Se a forma mudar, o teste quebra,
e a mudança precisa ser consciente: versão nova, com leitura da antiga.

## Testes de tabela com ponteiros de função

```rust
type Break = fn(&mut SavedResult);
let cases: [(Break, ResultError); 6] = [
    (|s| s.version = 2, ResultError::UnknownVersion(2)),
    (|s| s.seed = "-1".into(), ResultError::InvalidSeed),
    // ...
];
for (break_it, expected) in cases {
    let mut saved = valid_saved();
    break_it(&mut saved);
    assert_eq!(GameResult::restore(&saved), Err(expected));
}
```

Uma closure que **não captura** nada do ambiente pode virar um ponteiro de função (`fn(...)`), e
assim todas cabem no mesmo array, mesmo sendo closures diferentes. Cada caso estraga um campo de um
registro válido e diz o erro esperado. É o `test.each` do Jest. O `type Break` existe porque o
clippy acha `[(fn(&mut SavedResult), ResultError); 6]` complexo demais para ler (`type_complexity`).

## Fim da fatia 3

Com esta task, o módulo Game tem as três fatias da spec: a partida, o tempo e o salvamento, e os
resultados. O que falta para o jogo chegar às mãos de alguém é a casca: a ponte do Tauri, que
chama estas funções, guarda a foto e o histórico no disco e entrega os números para a tela em
Svelte.
