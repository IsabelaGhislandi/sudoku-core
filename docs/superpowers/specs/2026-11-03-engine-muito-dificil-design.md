# Spec — Engine, fatia de nível Muito Difícil

- **Data:** 2026-11-03
- **Status:** implementado. Medido em 200 seeds, release, neste PC: mediana de 119 ms, p90 de
  411 ms, pior caso de 793 ms, 1 seed esgotada (a 15), dentro das metas de 1 s e 10 esgotadas.
  O que mudou em relação ao experimento da seção 8:
  - **Constantes:** K = 8 (e não 6), 80 refinamentos e 5 soluções, teto de 405 cavações. Na
    varredura, K = 8 deu a menor mediana e a cauda mais curta; K = 4, 6 e 10 ficaram atrás.
    `MAX_SOLUCOES = 6` também passaria, mas nenhuma das 200 seeds chega ao teto dele, que custaria
    cerca de 1 s; com 5, a seed esgotada mostra o custo do teto inteiro.
  - **Desempenho:** sem otimizar, o pior caso era de 5,7 s. Dois ganhos, sem mudar a saída de
    nenhum nível: o gerador pergunta ao solver por `hardest_step` (singles em lote, sem lista de
    passos), e o `CandidateGrid` mantém as posições de cada dígito por unidade, que as técnicas
    de unidade antes recalculavam a cada chamada.
- **Módulo:** 1 de 5 (Core/Engine), continuação da [spec do Difícil](2026-11-02-engine-dificil-design.md)
- **Fatia anterior:** [plano da fatia Difícil](../plans/2026-11-02-engine-dificil.md)

## 1. Objetivo e motivo

Fazer o `sudoku-core` **gerar puzzles de nível Muito Difícil** e fechar a escala de quatro níveis.
Hoje, `generate(MuitoDificil, _)` devolve `Unsupported`, e `rate` chama de `MuitoDificil` qualquer
puzzle que as técnicas do Difícil não resolvem.

O Muito Difícil tem duas características. Ele **continua sendo um puzzle de lógica**: quem joga
consegue resolvê-lo só pensando, sem chutar. E ele **parece mais vazio** que os outros níveis: no
máximo 22 números no início, contra cerca de 25 nos demais.

## 2. Metas

1. `generate(Difficulty::MuitoDificil, seed)` devolve um puzzle de solução única com
   `rate == Some(MuitoDificil)`, que o solver lógico resolve por completo e que tem **no máximo 22
   números dados**.
2. O solver ganha as técnicas **Swordfish** e **XY-Wing**, de nível `MuitoDificil`.
3. A saída de `generate` para Fácil, Médio e Difícil não muda para nenhuma seed.
4. Gerar um Muito Difícil cabe em **5 segundos no celular**.

## 3. Fora de escopo

- Técnicas além de Swordfish e XY-Wing (Jellyfish, XYZ-Wing, cadeias, coloração etc.).
- Mudar quantos números os outros níveis deixam no tabuleiro. A ideia de deixar Fácil e Médio mais
  cheios ficou registrada como possível fatia futura, porque mudaria os golden tests.
- Módulo Game, inclusive sortear outra seed quando a geração esgota o trabalho.
- Precisão fina da classificação. Um puzzle Muito Difícil às vezes parecer Difícil é aceitável; os
  apps comerciais também são assim.

## 4. O que não pode acontecer

- **Gerar um Muito Difícil que exija chute.** O puzzle precisa sair inteiro pelo solver lógico.
- **Mudar qualquer puzzle Fácil, Médio ou Difícil.** Os três golden tests da seed 42 não podem ser
  alterados.
- **Limitar a geração por tempo de relógio.** O limite é de trabalho, contado de forma determinística.
- **Travar a geração.** Ela sempre termina, com um puzzle ou com erro.

## 5. Decisões

**Tomadas em conversa com a pessoa responsável pelo projeto:**

- O Muito Difícil **sempre se resolve por lógica**. Isso substitui a frase do design original
  "pode exigir backtracking/chute" para o que o gerador produz.
- As técnicas novas são **Swordfish e XY-Wing**, as do design original.
- O Muito Difícil tem **no máximo 22 números dados**. A pessoa responsável escolheu esse limite
  sabendo que ele exige um gerador novo (seção 8).
- O limite de espera continua **5 segundos no celular**.
- Para limitar a espera, a geração tem um **teto de trabalho**, como o teto de tentativas do Difícil:
  ao atingi-lo, devolve `AttemptsExhausted(MuitoDificil)`, e o Game sorteia outra seed. Além disso, o
  plano inclui uma **etapa de desempenho** para que o teto seja atingido raramente.
- **`GenError::Unsupported` sai da API.** Com os quatro níveis suportados, nenhum caminho o devolve
  mais. Removê-lo agora não quebra ninguém, porque ainda não há código fora do `sudoku-core` que o
  use. `GenError` fica só com `AttemptsExhausted(Difficulty)`, e o `match` de `generate` passa a
  cobrir os quatro níveis sem o braço `_`, para o compilador avisar se surgir um quinto nível.
- `rate` continua classificando como `MuitoDificil` um puzzle de solução única que nem as técnicas
  novas resolvem (um que só sai por tentativa e erro). Ele fica no topo da escala, como "pelo menos
  Muito Difícil". O gerador nunca produz esse tipo de puzzle; o caso só aparece com puzzles vindos de
  fora.

**Regras herdadas do projeto:** determinismo por seed, `sudoku-core` puro, só `rand` e `rand_chacha`,
TDD, wiki por task e demais convenções do `CLAUDE.md`. A invariante de progresso de `Eliminate`
(seção 6.3 da spec do Médio) vale para as técnicas novas.

## 6. Por que a saída de Fácil, Médio e Difícil não muda

O argumento é o mesmo da fatia Difícil (seção 6 daquela spec):

- **Cada gerador cava com `up_to(nível)`**, que filtra por `level() <= nível`. As técnicas novas têm
  nível `MuitoDificil` e ficam de fora de todos os geradores existentes.
- **As técnicas novas vão no fim da lista** de `all_techniques`. Em um puzzle que `up_to(Dificil)`
  resolve, `new()` faz exatamente os mesmos passos, então `rate` não muda para esses puzzles.
- O código novo do gerador fica só no ramo `MuitoDificil` de `generate`.
- Os três golden tests comprovam isso.

## 7. Técnicas novas

As duas têm nível `MuitoDificil` e devolvem `Action::Eliminate`. Cada uma fica em seu arquivo em
`techniques/`, e as duas são expostas pela API pública, como as demais.

| Técnica | Padrão | Elimina |
|---|---|---|
| **Swordfish** | em três linhas, um dígito é candidato em 2 ou 3 posições de cada uma, e as posições das três linhas juntas caem em exatamente 3 colunas | esse dígito dessas 3 colunas, nas outras linhas; e o mesmo trocando linha por coluna |
| **XY-Wing** | uma célula pivô com exatamente os candidatos {x, y} enxerga duas células "asa", uma com exatamente {x, z} e outra com exatamente {y, z} (z diferente de x e de y) | z de toda célula, exceto as asas, que enxerga as duas asas |

"Enxergar" quer dizer estar na mesma linha, coluna ou box, sendo outra célula. As asas não precisam
se enxergar.

O Swordfish é a generalização do X-Wing para 3 linhas. Uma linha com o dígito em só 2 posições
também conta. Isso permite que um X-Wing apareça "dentro" de um Swordfish, mas o X-Wing vem antes na
lista e é achado primeiro.

**Ordem final do solver**, fixa: Naked Single, Hidden Single, Naked Pair, Hidden Pair, Pointing Pair,
Box/Line Reduction, Naked Triple, Hidden Triple, X-Wing, **Swordfish, XY-Wing**. Dentro de cada
técnica, a varredura segue ordem crescente (dígitos, linhas, células), linha antes de coluna, sem
`HashMap`. No XY-Wing, a ordem é: pivô por índice crescente, depois a primeira asa, depois a segunda,
as duas também por índice crescente; os pares eliminados saem em ordem de célula.

*Chamada minha:* Swordfish antes de XY-Wing segue a ordem do design original e foi a ordem medida no
experimento. Como a ordem faz parte do contrato de determinismo, ela fica fixa a partir desta fatia.

## 8. Gerador

### Por que o laço do Difícil não basta

Um experimento descartável (código cru, sem testes, numa cópia do crate) mediu o laço do Difícil
aplicado ao Muito Difícil: repetir "solução nova → cavar com `up_to(MuitoDificil)` → aceitar se
exigiu técnica de Muito Difícil". Build de release, neste PC:

| Regra de aceite | Seeds | Sem puzzle | Mediana | p90 | Pior |
|---|---|---|---|---|---|
| só exigir técnica de Muito Difícil | 200 | 0 | 70 ms | 226 ms | 455 ms |
| + no máximo 23 números | 50 | 1 (teto de 500 tentativas) | 0,5 s | 1,6 s | 2,0 s |
| + no máximo 22 números | 50 | 21 (teto de 500 tentativas) | 1,1 s | 3,0 s | 4,3 s |

Sem limite de números, o Muito Difícil sai fácil: o XY-Wing apareceu nos 200 puzzles, e o Swordfish
foi exigido em 10. O problema é o limite de 22 números. O `dig` atual para quando nenhum número pode
sair, e isso costuma acontecer com 22 a 27 números. Puzzles que chegam perto são descartados inteiros.

### Estratégia: refinar em vez de recomeçar

Quando o puzzle cavado não atende o aceite, em vez de descartá-lo, o gerador o **perturba e recava**:
devolve alguns números da solução em células vazias e tenta de novo tirar números, agora só entre as
células preenchidas. Recavar custa bem menos que cavar do zero, porque só testa as cerca de 25
células preenchidas, e não as 81.

```
generate(MuitoDificil, seed):
  rng    = ChaCha12Rng(seed)
  solver = up_to(MuitoDificil)
  repetir até MAX_SOLUCOES vezes:
    solution = full_solution(rng)
    puzzle   = dig(solution, solver, rng)
    se aceita(puzzle): devolve Puzzle
    repetir até MAX_REFINAMENTOS vezes:
      candidato = puzzle com K números da solução devolvidos a células vazias (sorteadas pelo rng)
      recavar(candidato, solver, rng)      // tenta tirar cada número dado, em ordem sorteada
      se aceita(candidato): devolve Puzzle
      se candidato tem no máximo tantos números quanto puzzle: puzzle = candidato
  devolve Err(GenError::AttemptsExhausted(MuitoDificil))

aceita(p) = p tem no máximo 22 números dados
            e o solver resolve p por completo
            e o solver exigiu ao menos uma técnica de Muito Difícil
```

*Chamada minha:* o "resolve por completo" no aceite é redundante na prática, porque `dig` e
`recavar` só tiram um número se o solver continua resolvendo. Mas ele custa nada e protege a regra
de nunca exigir chute contra um erro futuro no refinamento.

Resultado do experimento com essa estratégia, 50 seeds, no máximo 22 números:

| K números devolvidos | Refinamentos por solução | Sem puzzle | Mediana | p90 | Pior |
|---|---|---|---|---|---|
| 3 | 10 | 0 | 0,89 s | 3,9 s | 5,8 s |
| 4 | 40 | 0 | 0,50 s | 1,7 s | 3,1 s |
| 5 | 40 | 0 | 0,55 s | 1,4 s | 1,7 s |
| **6** | **80** | **0** | **0,47 s** | **1,2 s** | **2,2 s** |
| 8 | 150 | 0 | 0,36 s | 1,5 s | 3,1 s |

Com refinamento, todas as seeds acharam puzzle, com 20 a 22 números. O caso típico cabe na meta, mas
cerca de 1 seed em 10 passa de 1 s neste PC.

- **Unicidade:** `recavar` só aceita tirar um número se o solver ainda resolve o puzzle por completo,
  como o `dig`. O argumento das fatias anteriores vale: se o solver resolve só com deduções forçadas,
  a solução é única. Devolver números da própria solução nunca quebra isso.
- **Aceite sem repetir a busca de unicidade**, como no Difícil. Os testes conferem a unicidade com
  `count_solutions`.
- **Determinismo:** todo sorteio vem do mesmo `rng`, na ordem do pseudocódigo. `K`,
  `MAX_REFINAMENTOS` e `MAX_SOLUCOES` fazem parte do contrato de determinismo, como
  `MAX_ATTEMPTS_HARD`.

### Teto de trabalho e limite de tempo

O teto é contado em trabalho, não em tempo de relógio: cada chamada faz no máximo
`MAX_SOLUCOES × (1 + MAX_REFINAMENTOS)` cavações (um `dig` ou um `recavar` cada). Com `K` e
`MAX_REFINAMENTOS` fixados, `MAX_SOLUCOES` é o teto e é escolhido por medição para que **cada chamada
fique em até cerca de 1 s neste PC**, com o PC livre. Uma chamada que atinge o teto devolve
`AttemptsExhausted(MuitoDificil)`.

Mesmo método do Difícil: build de **release** neste PC, teste `#[ignore]` em `tests/desempenho.rs`,
**200 seeds consecutivas**. A meta é **pior caso de até 1 s por chamada**, termine ela com um puzzle
ou com `AttemptsExhausted`. Como no Difícil, as seeds esgotadas são aceitas e contadas, e a medição
imprime quantas foram.

### Etapa de desempenho

Antes de fixar o teto, o plano inclui uma etapa para acelerar a geração e reduzir quantas seeds o
atingem. Ela não pode mudar a saída de Fácil, Médio e Difícil. Caminhos prováveis, sem obrigação:
evitar alocações no caminho quente das técnicas novas, e fazer o `recavar` parar o solver assim que
ele empaca.

*Chamada minha:* a meta é **no máximo 1 seed esgotada em 20** (5%) nas 200 seeds. Se não for
atingida, a implementação para e traz os números medidos, em vez de relaxar a meta, o critério ou o
limite de 22 números sozinha. A pessoa responsável decide.

## 9. Testes

- **Swordfish:** um `CandidateGrid` montado em que ele se aplica, conferindo os pares eliminados
  exatos; testado nas duas direções (linhas e colunas); um caso com uma das três linhas com só 2
  posições; um caso sem padrão (`None`); um caso com padrão mas nada a eliminar (`None`).
- **XY-Wing:** um caso com as asas em unidades diferentes (por exemplo, uma na linha do pivô e outra
  no box), conferindo os pares eliminados exatos; um caso com as asas se enxergando; um caso em que a
  segunda asa tem z diferente (`None`); um caso com padrão mas nada a eliminar (`None`).
- **Solver:** `up_to(Dificil)` empaca em um puzzle que exige técnica de Muito Difícil; `new()` o
  resolve, chegando à solução correta. `up_to(Dificil)` não usa nenhuma técnica de Muito Difícil.
- **Eliminações corretas:** em alguns Muito Difíceis gerados, nenhuma eliminação remove o dígito da
  `solution`.
- **Rating:** um puzzle conhecido que exige técnica de Muito Difícil é classificado como
  `MuitoDificil`; um puzzle de solução única que nem o solver completo resolve continua
  `MuitoDificil`.
- **Gerador:** Muito Difícil único, com `rate == Some(MuitoDificil)`, resolvido por completo pelo
  solver lógico e com no máximo 22 números; mesma seed, mesmo puzzle; seeds extremas (`0`,
  `u64::MAX`), aceitando `AttemptsExhausted` só se a medição mostrar que é o caso dessas seeds; novo
  golden test de Muito Difícil para a seed 42; uma seed conhecida que esgota o teto devolve
  `AttemptsExhausted(MuitoDificil)`, se existir uma entre as medidas. O teste
  `niveis_nao_suportados_ainda_retornam_erro` sai junto com a variante `Unsupported`.
- **Contratos:** os golden tests de Fácil, Médio e Difícil passam **sem alteração**.
- **Integração:** `tests/api.rs` cobre o fluxo de Muito Difícil pela API pública.
- **Desempenho:** `muito_dificil_cabe_na_meta_de_tempo` em `tests/desempenho.rs`, no molde do Difícil.

Os testes do gerador rodam em debug no CI, então devem usar poucas seeds.

## 10. Documentação

- Atualizar `docs/architecture.md`: status do módulo (engine completo), tabela de níveis (Muito
  Difícil sempre resolvível por lógica, no máximo 22 números), técnicas, API (`generate` aceita os
  quatro níveis, `GenError` sem `Unsupported`, técnicas novas expostas) e fluxo de geração.
- Atualizar as menções a `Unsupported` na wiki 12 e no consolidado das tasks 11 a 15 com uma nota de
  que a variante saiu nesta fatia, sem reescrever a história da página. Na seção 10, remover "a próxima fatia
  adiciona as técnicas de Muito Difícil" e registrar a ideia de encher os níveis fáceis.
- Atualizar o `Status` da spec original do engine e a nota em `rate` sobre quais níveis existem.
- Uma página de wiki por task, continuando a partir da 30, e o consolidado das tasks 26 a 30 em
  `docs/wiki/fundamentos/` quando chegar à 30.

## 11. Decisões deixadas para quem implementa

- Valores de `K`, `MAX_REFINAMENTOS` e `MAX_SOLUCOES`, por medição. O ponto de
  partida é o melhor do experimento: K = 6 e 80 refinamentos por solução.
- Como sortear as K células devolvidas (com ou sem repetição), desde que seja determinístico.
- Se `recavar` reaproveita o `dig` (por exemplo, recebendo a lista de células a tentar) ou é uma
  função à parte.
- Helpers compartilhados entre X-Wing e Swordfish, como as máscaras de posição por linha.
- Os nomes em inglês das constantes, seguindo o `CLAUDE.md` (por exemplo, `MAX_REFINEMENTS_VERY_HARD`).
