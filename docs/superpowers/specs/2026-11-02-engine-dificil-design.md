# Spec — Engine, fatia de nível Difícil

- **Data:** 2026-11-02
- **Status:** em revisão
- **Módulo:** 1 de 5 (Core/Engine), continuação da [spec do Médio](2026-11-01-engine-medio-design.md)
- **Fatia anterior:** [plano da fatia Médio](../plans/2026-11-01-engine-medio.md)

## 1. Objetivo e motivo

Fazer o `sudoku-core` **gerar e classificar puzzles de nível Difícil**. Hoje, um puzzle que precisa
de um X-Wing é classificado como `MuitoDificil`, porque o solver para nas técnicas de Médio, e
`generate(Dificil, _)` devolve `Unsupported`.

A fatia Médio já criou toda a base: `Action::Eliminate`, `CandidateGrid`, `LogicalSolver::up_to` e o
laço de tentativas do gerador. Esta fatia é, em essência, **mais técnicas + mais um nível no gerador**.
Não há mudança de design na representação.

## 2. Metas

1. `generate(Difficulty::Dificil, seed)` devolve um puzzle de solução única e `rate == Some(Dificil)`.
2. `rate` reconhece Difícil, usando as técnicas Box/Line Reduction, Naked Triple, Hidden Triple e X-Wing.
3. A saída de `generate(Facil, seed)` e de `generate(Medio, seed)` não muda para nenhuma seed.
4. Gerar um Difícil cabe em **5 segundos no celular**.

## 3. Fora de escopo

- Técnicas e geração de Muito Difícil: `generate(MuitoDificil, _)` continua devolvendo
  `GenError::Unsupported`. `rate` continua devolvendo `MuitoDificil` para o que o solver não resolve
  mas tem solução única.
- Módulo Game.
- Ajuste fino da distribuição de puzzles Difíceis.

## 4. O que não pode acontecer

- **Mudar qualquer puzzle Fácil ou Médio.** Os dois golden tests da seed 42 não podem ser alterados.
- **Limitar a geração por tempo de relógio.**
- **Travar a geração.** Ela sempre termina, com um puzzle ou com erro.

## 5. Decisões

**Tomadas em conversa com a pessoa responsável pelo projeto:**

- A fatia cobre **só o Difícil**. Muito Difícil fica para a próxima.
- O nível é definido **só pela técnica mais avançada exigida**. A quantidade de casas preenchidas não
  entra como critério: obrigar técnicas mais avançadas é o que torna o puzzle interessante.
- As quatro técnicas da seção 7 entram todas nesta fatia.
- O limite de espera continua **5 segundos no celular**.

**Regras herdadas do projeto:** determinismo por seed, `sudoku-core` puro, só `rand` e `rand_chacha`,
TDD, wiki por task e demais convenções do `CLAUDE.md`. A invariante de progresso de `Eliminate`
(seção 6.3 da spec do Médio) vale para as técnicas novas.

## 6. Por que a saída de Fácil e Médio não muda

- **Fácil e Médio cavam com `up_to(nível)`**, que filtra por `level() <= nível`. As técnicas novas têm
  nível `Dificil` e ficam de fora.
- **`rate` usa `new()`**, que passa a incluir as técnicas novas. Elas vão **no fim da lista**. O solver
  sempre tenta do início, então só chega a uma técnica de Difícil quando todas as de Fácil e Médio
  empacam. Em um puzzle que `up_to(Medio)` resolve, isso nunca acontece: a sequência de passos, e
  portanto o nível, é idêntica. O aceite `rate == Some(Medio)` do gerador de Médio dá o mesmo resultado.
- Os golden tests comprovam as duas coisas.

## 7. Técnicas novas

Todas têm nível `Dificil` e devolvem `Action::Eliminate`. Cada uma em seu arquivo em `techniques/`.

| Técnica | Padrão | Elimina |
|---|---|---|
| **Box/Line Reduction** | numa linha (ou coluna), todas as células candidatas a um dígito estão no mesmo box | esse dígito das outras células do box |
| **Naked Triple** | três células da mesma unidade cujos candidatos, juntos, são exatamente 3 dígitos (cada célula tem 2 ou 3 deles) | esses 3 dígitos das outras células da unidade |
| **Hidden Triple** | três dígitos que, numa unidade, só aparecem dentro das mesmas 3 células (e cada um aparece ao menos uma vez) | os demais candidatos dessas 3 células |
| **X-Wing** | em duas linhas, um dígito só é candidato nas mesmas 2 colunas | esse dígito dessas 2 colunas, nas outras linhas; e o mesmo trocando linha por coluna |

**Ordem final do solver**, fixa: Naked Single, Hidden Single, Naked Pair, Hidden Pair, Pointing Pair,
Box/Line Reduction, Naked Triple, Hidden Triple, X-Wing. Dentro de cada técnica, a varredura segue
ordem crescente de índice (unidades, células, dígitos), linha antes de coluna, sem `HashMap`.

Box/Line Reduction costuma ser listada junto do Pointing Pair, como técnica de Médio. Aqui ela fica
no Difícil porque colocá-la no Médio mudaria a saída de `generate(Medio, _)`, violando a meta 3.

## 8. Gerador

```
generate(Dificil, seed):
  rng = ChaCha12Rng(seed)
  repetir até MAX_TENTATIVAS_DIFICIL vezes:
    solution = full_solution(rng)
    givens   = dig(solution, up_to(Dificil), rng)
    se rate(givens) == Some(Dificil): devolve Puzzle
  devolve Err(GenError::AttemptsExhausted(Dificil))
```

- É o mesmo laço do Médio, com outro nível e outro limite. O laço pode ser generalizado, desde que o
  Médio consuma o RNG exatamente como hoje e `MAX_TENTATIVAS_MEDIO` continue 50.
- `dig` continua sem `count_solutions`. As técnicas novas também só fazem deduções forçadas, então o
  argumento do Médio vale: se o solver resolve, a solução é única. `rate` confere a unicidade de todo
  Difícil aceito.
- `MAX_TENTATIVAS_DIFICIL` é escolhido por medição e faz parte do contrato de determinismo, como o do
  Médio.

### Limite de tempo

Mesmo método do Médio: build de **release** neste PC, teste `#[ignore]` em `tests/desempenho.rs`,
meta de **até 1 s no pior caso, em 200 seeds consecutivas**, como estimativa de 5 s no celular. Uma seed
que devolva `AttemptsExhausted` conta como falha. Se a meta não for atingida, a implementação para e
traz os números medidos, em vez de relaxar a meta ou o critério sozinha.

## 9. Testes

- **Técnicas:** cada uma com um `CandidateGrid` montado em que ela se aplica, conferindo os pares
  eliminados exatos; um caso sem padrão (`None`); e um caso com padrão mas nada a eliminar (`None`).
  X-Wing testado nas duas direções (linhas e colunas). Naked Triple testado com uma célula de só 2
  candidatos.
- **Solver:** `up_to(Medio)` empaca em um puzzle que exige técnica de Difícil; `new()` o resolve,
  chegando à solução correta. `up_to(Medio)` não usa nenhuma técnica de Difícil.
- **Eliminações corretas:** em alguns Difíceis gerados, nenhuma eliminação remove o dígito da `solution`.
- **Rating:** um puzzle conhecido que exige técnica de Difícil é classificado como `Dificil`.
- **Gerador:** Difícil único e com `rate == Some(Dificil)`; mesma seed, mesmo puzzle; seeds extremas
  (`0`, `u64::MAX`); novo golden test de Difícil para a seed 42; `MuitoDificil` continua `Unsupported`.
- **Contratos:** os golden tests de Fácil e Médio passam **sem alteração**.
- **Integração:** `tests/api.rs` cobre o fluxo de Difícil pela API pública.

Os testes do gerador rodam em debug no CI, então devem usar poucas seeds.

## 10. Documentação

- Atualizar `docs/architecture.md`: status do módulo, técnicas, API e fluxo de geração.
- Atualizar doc comments desatualizados, como a nota em `rate` sobre quais níveis existem.
- Uma página de wiki por task, continuando a partir da 22, e o consolidado das tasks 21 a 25 quando
  chegar a 25.

## 11. Decisões deixadas para quem implementa

- Helpers compartilhados entre as técnicas (por exemplo, iterar combinações de 3 células ou dígitos
  sem alocar), desde que a ordem de varredura seja determinística.
- Se o laço de tentativas vira uma função comum a Médio e Difícil.
- Valor de `MAX_TENTATIVAS_DIFICIL`, com base em medição, respeitando o limite de tempo.
