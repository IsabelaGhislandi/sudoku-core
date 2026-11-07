# 47 — Estatísticas: iteradores preguiçosos, `Option` no lugar de dividir por zero e ordenação estável

> Referente à Task 3 do plano da fatia 3 do Game. Commit: `feat: add per-level statistics and personal ranking`.

As estatísticas e o ranking são **funções puras** sobre a lista de resultados que a casca guarda:
recebem `&[GameResult]` e devolvem números. Não guardam nada e não leem relógio, então os testes
montam uma lista à mão e conferem a resposta.

## Iteradores preguiçosos e `clone` de iterador

```rust
let level = results.iter().filter(|r| r.difficulty == difficulty);
let wins = level.clone().filter(|r| r.outcome == Outcome::Won);
LevelStats {
    played: level.count() as u32,
    won: wins.clone().count() as u32,
    best_ms: wins.map(|r| r.elapsed_ms).min(),
}
```

Um iterador do Rust é **preguiçoso**: `filter` não percorre nada, só descreve o filtro. Quem
percorre é o consumidor no fim (`count`, `min`, `collect`). É como um *generator* do JavaScript, e
não como `Array.prototype.filter`, que monta um array novo na hora.

Os consumidores **gastam** o iterador (recebem `self`, como o `abandon` da página 46). Para contar
duas vezes, o código clona o iterador antes, o que copia só a "receita" (a posição na fatia e a
closure), e não os resultados. Nenhum `Vec` intermediário é criado.

`min()` devolve `Option<u64>`: sem vitória, não existe melhor tempo, e o tipo já diz isso.

## `Option` no lugar de dividir por zero

```rust
pub fn win_percentage(&self) -> Option<u8> {
    if self.played == 0 {
        return None;
    }
    let (won, played) = (u64::from(self.won), u64::from(self.played));
    Some(((won * 100 + played / 2) / played) as u8)
}
```

Em JavaScript, `0 / 0` dá `NaN`, e um `NaN` na tela vira "NaN%". Em Rust, dividir um inteiro por
zero **entra em pânico**. Nenhum dos dois serve. A spec quer que um nível sem partidas não mostre
porcentagem nenhuma (0% diria "perdeu todas"), e o `Option` deixa essa diferença no tipo.

Duas escolhas de conta:

- **Inteiros, sem `f64`.** Somar metade do divisor antes de dividir arredonda para o mais próximo:
  2 de 3 dá `(200 + 1) / 3 = 67`. Sem ponto flutuante, o resultado é igual em qualquer plataforma.
- **`u64::from` antes de multiplicar.** `won * 100` em `u32` estouraria com mais de 42 milhões de
  vitórias. Improvável, mas `u64::from` é conversão sem perda e custa nada. O `as u8` no fim é
  seguro porque o resultado é no máximo 100.

## Ordenação estável com tupla como chave

```rust
wins.sort_by_key(|r| (r.elapsed_ms, r.finished_at_ms));
```

Tuplas se comparam campo a campo, como palavras no dicionário: primeiro o tempo, e só no empate a
data. Assim, "empate no tempo vai para a mais antiga" cabe numa linha.

`sort_by_key` é **estável**: elementos com a mesma chave mantêm a ordem em que estavam. Por isso
nem precisa de um terceiro critério para o empate total. O `sort` do JavaScript também é estável
desde o ES2019. Em Rust, quem não precisa dessa garantia usa `sort_unstable_by_key`, que é um pouco
mais rápido.

## `Vec<&GameResult>`: referências presas à lista

`personal_ranking` devolve **referências** para os resultados, e não cópias. O compilador liga o
tempo de vida da saída ao da entrada (a regra de *lifetime elision*: uma referência de entrada, a
saída vive o mesmo tanto). Na prática, a lista original não pode ser descartada enquanto o ranking
estiver em uso:

```rust
let ranking = personal_ranking(&results, Difficulty::Medio);
drop(results); // erro: `results` ainda está emprestado pelo `ranking`
```

Em JavaScript, o ranking manteria os objetos vivos sozinho, por causa do coletor de lixo. Em Rust,
é o compilador que garante que nenhuma referência aponte para algo que já foi liberado.
