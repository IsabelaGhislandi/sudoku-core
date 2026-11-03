# Spec — Engine, fatia de nível Médio

- **Data:** 2026-11-01
- **Status:** implementado (PR #10)
- **Módulo:** 1 de 5 (Core/Engine), continuação da [spec do Engine](2026-06-14-sudoku-engine-design.md)
- **Fatia anterior:** [plano da fatia Fácil](../plans/2026-06-14-engine-facil-vertical-slice.md)

Esta spec prevalece sobre a spec do Engine nos pontos em que as duas divergem, como `next_hint`.

## 1. Objetivo e motivo

Fazer o `sudoku-core` **gerar e classificar puzzles de nível Médio**. Hoje, `generate` só aceita
Fácil, e um puzzle que precisa de um simples Naked Pair é classificado como `MuitoDificil`, porque
o solver só conhece singles.

As técnicas de Médio não colocam valores: elas **removem candidatos**. O solver atual não tem como
representar isso, e essa é a principal mudança de design desta fatia.

## 2. Metas

1. `generate(Difficulty::Medio, seed)` devolve um puzzle de solução única e `rate == Some(Medio)`.
2. `rate` reconhece Médio, usando as técnicas Naked Pair, Hidden Pair e Pointing Pair.
3. `Deduction` representa tanto colocar um valor quanto eliminar candidatos.
4. A saída de `generate(Facil, seed)` não muda para nenhuma seed.
5. Gerar um Médio cabe em **5 segundos no celular**.

## 3. Fora de escopo

- Técnicas e geração de Difícil e Muito Difícil: `generate` continua devolvendo
  `GenError::Unsupported` para esses níveis.
- Módulo Game, incluindo a implementação da dica (seção 7).
- Ajuste fino da distribuição de puzzles Médios, como quantas técnicas cada um deve exigir.

## 4. O que não pode acontecer

- **Mudar qualquer puzzle Fácil.** O golden test da seed 42 não pode ser alterado nesta fatia.
- **Mostrar eliminações para quem joga.** Elas são detalhe interno do solver e da classificação.
- **Limitar a geração por tempo de relógio.** Isso violaria o determinismo e a regra de que o core
  não usa relógio.
- **Travar a geração.** Ela sempre termina, com um puzzle ou com erro.

## 5. Decisões

**Tomadas em conversa com a pessoa responsável pelo projeto:**

- A dica preenche uma célula vazia **sorteada** com o valor da `solution`, mesmo que a célula ainda
  não pudesse ser deduzida. Ela nunca mostra eliminações.
- O limite de espera para gerar um Médio é de 5 segundos no celular.
- O desenho das seções 6 a 9 foi revisado e aprovado em conversa.

**Regras herdadas do projeto:**

- Mesma seed + mesmo nível → mesmo puzzle, inclusive entre versões.
- O nível de um puzzle é o da **técnica mais avançada exigida**: basta precisar de um único Naked
  Pair para ser Médio.
- `sudoku-core` continua puro, e as dependências continuam sendo só `rand` e `rand_chacha`.
- TDD, wiki por task e demais convenções do `CLAUDE.md`.

## 6. Representação: `Deduction` e candidatos

### 6.1 `Deduction`

```rust
pub enum Action {
    Place { index: usize, value: u8 },
    Eliminate(Vec<(usize, u8)>), // pares (célula, dígito) a remover
}

pub struct Deduction {
    pub action: Action,
    pub technique: &'static str,
    pub level: Difficulty,
}
```

`Action` é exportado em `lib.rs`. `rate` continua igual em essência, calculando o máximo de `level`
sobre todos os passos.

### 6.2 Candidatos guardados

Novo tipo público `CandidateGrid`, em `candidates.rs`, com um `CandidateSet` por célula:

- é construído a partir de uma `Grid`, usando `candidates_for`;
- ao colocar um valor, limpa a célula e remove o dígito das células vizinhas (linha, coluna, box);
- ao eliminar, remove o par (célula, dígito).

`CandidateSet` ganha o que for preciso para isso, como remover um dígito. `candidates_for` continua
público, pois é a visão a partir da grade pura.

### 6.3 Trait `Technique`

```rust
pub trait Technique {
    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction>;
}
```

`NakedSingle` e `HiddenSingle` passam a ler de `candidates`. Sem eliminações, os candidatos guardados
são idênticos aos recalculados, então a saída do nível Fácil não muda. O golden test comprova isso.

**Invariante de progresso:** uma técnica só devolve `Eliminate` se **ao menos um** par removeria um
candidato ainda presente, e a lista contém só esses pares. Sem essa regra, o solver repetiria a mesma
dedução para sempre.

## 7. Dica: `next_hint` sai do core

`next_hint` é removido da API pública, junto com seu trecho em `tests/api.rs`. O sorteio de uma célula
vazia e a leitura de `puzzle.solution` são simples e ficam no módulo Game, que tem acesso público a
ambos. Nada além dos testes depende de `next_hint` hoje.

Registrar em `docs/architecture.md` que a dica pertence ao Game, para entrar na spec desse módulo. Como o
valor vem da `solution`, jogadas erradas na grade atual não afetam a dica.

## 8. Solver e técnicas

### 8.1 Solver por nível

```rust
LogicalSolver::new()          // todas as técnicas implementadas
LogicalSolver::up_to(level)   // só as técnicas com nível <= level
```

- `dig_facil` passa a usar `up_to(Difficulty::Facil)`. É isso que protege a meta 4.
- `rate` usa `new()` e devolve `None` de imediato se `validate` apontar conflitos, para não gastar uma
  busca longa em uma grade quase vazia que não pode ter solução.
- `solve` mantém um `CandidateGrid` e aplica cada `Action`. Quando nenhuma técnica se aplica, empaca.
- `SolveResult.grid` e `.solved` mantêm o significado atual; `.steps` passa a incluir eliminações.

### 8.2 Ordem das técnicas

Fixa, da mais simples para a mais difícil: Naked Single, Hidden Single, Naked Pair, Hidden Pair,
Pointing Pair. O solver sempre tenta do início da lista, então só recorre a uma técnica de Médio quando
os singles empacam. A varredura interna de cada técnica também tem ordem fixa: unidades e células em
ordem crescente de índice, sem nenhum `HashMap`.

### 8.3 Técnicas novas

Todas têm nível `Medio` e devolvem `Action::Eliminate`. Uma unidade é uma linha, coluna ou box.

| Técnica | Padrão | Elimina |
|---|---|---|
| **Naked Pair** | duas células da mesma unidade têm exatamente os mesmos 2 candidatos | esses 2 dígitos das outras células da unidade |
| **Hidden Pair** | dois dígitos só aparecem nas mesmas 2 células de uma unidade | os demais candidatos dessas 2 células |
| **Pointing Pair** | dentro de um box, todas as células candidatas a um dígito (2 ou 3) estão na mesma linha ou coluna | esse dígito do resto da linha ou coluna, fora do box |

Cada técnica fica em seu próprio arquivo em `techniques/`, como as atuais.

## 9. Gerador

```
generate(Medio, seed):
  rng = ChaCha12Rng(seed)
  repetir até MAX_TENTATIVAS_MEDIO vezes:
    solution = full_solution(rng)
    givens   = dig(solution, up_to(Medio), rng)
    se rate(givens) == Some(Medio): devolve Puzzle
  devolve Err(GenError::AttemptsExhausted(Medio))
```

- Um único RNG atravessa todas as tentativas, então a saída continua determinística.
- `dig_facil` é generalizado para receber o solver. O fluxo de Fácil não usa o laço de tentativas e
  deve consumir o RNG exatamente como hoje.
- `GenError` ganha a variante `AttemptsExhausted(Difficulty)`.
- **Decisão tomada na implementação:** `dig` não chama mais `count_solutions`. Um puzzle que o solver
  resolve só com deduções forçadas já tem solução única, e a busca custava cerca de 80% do tempo. A
  primeira medição deu 2,24 s no pior caso; sem a busca, ficou abaixo de 0,5 s. `rate` continua conferindo
  a unicidade de todo Médio aceito.
- O valor de `MAX_TENTATIVAS_MEDIO` faz parte do contrato de determinismo. Reduzi-lo pode transformar
  um puzzle em erro. Aumentá-lo só transforma erro em puzzle, sem alterar os que já existem.

### Limite de tempo

Ainda não há app para medir no celular. Na fatia, medimos o build de **release** neste PC, com um
teste `#[ignore]` em `tests/` (testes podem usar `std::time`; a biblioteca não). A meta é **até 1 s no
pior caso, em 200 seeds consecutivas**.

Essa meta é uma **estimativa**, baseada na suposição de um celular até 5 vezes mais lento que o PC. Deve
ser confirmada em um aparelho real quando o módulo Tauri existir. Uma seed que devolva `AttemptsExhausted`
nessa medição conta como falha. Se a meta não for atingida, a implementação para e traz os números
medidos, em vez de relaxar a meta sozinha.

## 10. Testes

- **Técnicas:** cada uma com um `CandidateGrid` montado em que ela se aplica, conferindo os pares
  eliminados exatos; um caso sem padrão (`None`); e um caso em que o padrão existe mas não há nada a
  eliminar (`None`, pela invariante de progresso).
- **Candidatos:** colocar limpa os vizinhos; eliminar remove só o par pedido; a construção bate com
  `candidates_for`.
- **Solver:** `up_to(Facil)` empaca em um puzzle que exige par; `new()` resolve o mesmo puzzle, chegando
  à solução correta.
- **Eliminações corretas:** em um conjunto de puzzles Médios gerados, nenhuma eliminação remove o dígito
  que a `solution` tem naquela célula.
- **Rating:** um puzzle conhecido que exige par é classificado como `Medio`; os testes atuais continuam.
- **Gerador:** Médio com solução única e `rate == Some(Medio)`; mesma seed gera o mesmo puzzle; novo golden
  test de Médio; `Dificil` continua `Unsupported`, substituindo `Medio` no teste atual.
- **Contrato do Fácil:** `puzzle_da_seed_42_e_reproduzivel` passa **sem alteração**.
- **Integração:** `tests/api.rs` cobre o fluxo de Médio pela API pública.

Os testes do gerador rodam em build de debug no CI, então devem usar poucas seeds.

## 11. Documentação

- Atualizar `docs/architecture.md`: tabela de módulos, árvore de arquivos, API pública, seção de dica,
  e a nota de que `rate` só cobre Fácil.
- Atualizar os doc comments que ficarem desatualizados, como a nota em `rate`.
- Uma página de wiki por task, continuando a partir da 16, e o consolidado a cada 5 tasks.

## 12. Decisões deixadas para quem implementa

- Nomes internos e forma exata dos helpers de unidade, e como evitar duplicação entre as técnicas.
- Forma de `next_step` (`LogicalSolver`), que precisa do `CandidateGrid`: pode mudar de assinatura ou
  deixar de ser pública, já que seu principal uso público era `next_hint`.
- Valor de `MAX_TENTATIVAS_MEDIO`, escolhido com base em medição, respeitando o limite de tempo.
