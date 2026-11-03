# 21 — Gerador de Médio: tentativas, medição e uma checagem redundante

> Referente à Task 6 do plano da fatia Médio. Commit: `feat: generate Medium puzzles with bounded attempts`.

Cavar com as técnicas de Médio não garante um puzzle Médio: às vezes o resultado ainda se resolve só com singles.
Por isso o gerador **tenta de novo** até `rate` confirmar o nível.

## Um laço com um RNG compartilhado

```rust
let mut rng = ChaCha12Rng::seed_from_u64(seed);
for _ in 0..MAX_ATTEMPTS_MEDIUM {
    let solution = full_solution(&mut rng);
    let givens = dig(&solution, &solver, &mut rng);
    if rate(&givens) == Some(Difficulty::Medio) {
        return Ok(Puzzle { givens, solution, difficulty, seed });
    }
}
Err(GenError::AttemptsExhausted(difficulty))
```

- **Um RNG só, emprestado como `&mut`:** cada tentativa continua a sequência de onde a anterior parou. Criar um RNG
  novo a cada volta, com a mesma seed, repetiria sempre a mesma tentativa fracassada.
- **`for _ in`:** `_` diz que o número da volta não importa. Usar um nome sem uso geraria warning.
- **`return` de dentro do laço:** sai da função inteira. Se o laço acabar sem sucesso, a expressão depois dele vira o
  valor da função.

## Por que o limite é por tentativas, e não por tempo

Um "pare depois de 5 s" daria resultados diferentes em aparelhos diferentes: o celular lento desistiria antes do rápido.
A mesma seed deixaria de dar o mesmo puzzle, e o core teria que ler o relógio, o que ele não faz.

O limite é a constante `MAX_ATTEMPTS_MEDIUM = 50`, e ela **faz parte do contrato**:

| Mudança | Efeito |
|---|---|
| Aumentar | só transforma erros em puzzles; os puzzles que já existiam não mudam |
| Diminuir | pode transformar um puzzle em erro |

Nas 200 seeds medidas, o máximo foi de **13 tentativas** e a média, de cerca de 3. O valor 50 dá uma folga de quase 4 vezes.

## Medindo em release

```rust
#[test]
#[ignore]
fn medio_cabe_na_meta_de_tempo() { /* Instant::now() + elapsed() */ }
```

```bash
cargo test --release --manifest-path sudoku-core/Cargo.toml --test desempenho -- --ignored --nocapture
```

- **`--release`:** o build de debug não otimiza e pode ser dezenas de vezes mais lento. Medir tempo nele não diz nada
  sobre o app.
- **`#[ignore]`:** o teste fica fora do `cargo test` normal e do CI, onde o build é de debug e as máquinas variam. Para
  rodá-lo, é preciso pedir com `--ignored`.
- **`std::time` nos testes, nunca na biblioteca:** um teste em `tests/` é um crate à parte, então pode usar o relógio sem
  violar a regra de que o core é puro.
- **`--nocapture`:** mostra os `println!` dos testes, que o Cargo esconde por padrão.

## A medição que falhou

A primeira versão ficou **acima da meta**:

| | Mediana | Pior caso | Acima de 1 s |
|---|---|---|---|
| Com checagem de unicidade no `dig` | 341 ms | 2,24 s | 30 de 200 seeds |
| Sem a checagem | 39 ms | 0,25 s | nenhuma |

O plano mandava parar e trazer os números, em vez de relaxar a meta. Um diagnóstico com contadores de tempo mostrou
que cerca de **80%** do tempo ia para `count_solutions`, chamado a cada célula cavada.

A checagem era **redundante**. O solver lógico só faz jogadas **forçadas**: se ele resolve o puzzle inteiro, cada valor
era o único possível, e a solução é única. `unique && logical` sempre valia o mesmo que `logical`. Tirar a busca não
mudou nenhum puzzle Fácil, conferido em 50 seeds, e de quebra acelerou também o gerador de Fácil.

O custo da decisão: agora a unicidade depende de as técnicas estarem **corretas**. Uma técnica com bug poderia gerar um
puzzle ambíguo. Duas redes continuam: `rate` confere a unicidade de todo Médio aceito, e há testes de técnica e um teste
de que as eliminações nunca removem a resposta.

> **Lição geral:** antes de otimizar o código lento, pergunte se ele precisa rodar. A checagem mais rápida é a que você
> prova ser desnecessária.

## Golden test do Médio

`puzzle_medio_da_seed_42_e_reproduzivel` trava a saída de `generate(Medio, 42)`, como o golden do Fácil. Ele também
protege detalhes que viraram contrato sem parecer: a **ordem das técnicas**, a **ordem de varredura** de cada uma e o valor
de `MAX_ATTEMPTS_MEDIUM`. Qualquer mudança neles altera os puzzles de Médio, e o teste avisa.

A mesma grade virou fixture de `rate` e do solver: um puzzle **conhecido** que os singles não resolvem e o Médio resolve.
Como esse valor saiu da implementação, os testes de propriedade (único, sem conflitos, eliminações corretas) é que provam
que ele é válido.
