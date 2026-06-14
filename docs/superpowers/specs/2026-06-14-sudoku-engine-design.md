# Spec — Módulo Core/Engine (`sudoku-core`)

- **Data:** 2026-06-14
- **Status:** Aprovado (design) — pronto para virar plano de implementação
- **Módulo:** 1 de 5 (Core/Engine)
- **Stack do projeto:** Tauri + Rust (backend) / Svelte (frontend) — alvo final: APK Android
- **Metodologia:** Spec-driven, modular, TDD (teste antes da implementação)

---

## 1. Contexto e objetivo

Projeto de um jogo de Sudoku para celular (APK Android via Tauri), feito também como
projeto de aprendizado de Rust por alguém com background em desenvolvimento web.

O desenvolvimento é **modular**: cada módulo tem seu próprio ciclo
`spec → plano → implementação`. Os módulos previstos:

| # | Módulo | Responsabilidade | Status |
|---|--------|------------------|--------|
| 1 | **Core/Engine** | Gerar, resolver, validar e classificar tabuleiros. Lógica pura. | **este spec** |
| 2 | Game/Sessão | Estado de uma partida: jogadas, anotações, erros, timer, dicas. | futuro |
| 3 | Frontend/UI | Tabuleiro, input, seleção de dificuldade, telas (Svelte). | futuro |
| 4 | Tauri/Empacotamento | Ponte Rust↔Svelte e build do APK. | futuro |
| 5 | Online | Partida por código com outro player. | futuro distante |

Este documento cobre **apenas o módulo 1 (Core/Engine)**.

### Objetivo do módulo

Uma biblioteca Rust **pura** (`sudoku-core`) que gera, resolve, valida e classifica
tabuleiros de Sudoku 9×9. É a fundação testável de todo o resto do app.

---

## 2. Escopo

### Dentro do escopo

- Geração de tabuleiros 9×9 com **solução única garantida**.
- Geração **determinística por seed** (mesma seed → mesmo tabuleiro).
- Quatro níveis de dificuldade: Fácil, Médio, Difícil, Muito Difícil.
- Classificação de dificuldade por **técnicas de resolução** (solver "humano").
- Solver lógico (passo a passo) e solver por backtracking (verificação de unicidade).
- Validação de conflitos (linha/coluna/box).
- Função de "próxima dica" (`next_hint`) reutilizando as técnicas — alimenta a dica do jogo no futuro.

### Fora do escopo (outros módulos)

- Qualquer UI, render ou interação.
- Qualquer dependência de Tauri.
- Estado de partida, timer, contagem de erros, persistência (módulo Game).
- Multiplayer/online.

---

## 3. Decisões de design

| Decisão | Escolha | Motivo |
|---------|---------|--------|
| Tamanho da grade | 9×9 clássico | Padrão do Sudoku |
| Solução | Única garantida | Sudoku "de verdade" tem uma resposta só |
| Geração | Determinística por seed | Testes reprodutíveis + futuro "compartilhar puzzle por código" |
| Critério de dificuldade | Técnicas de resolução | Mais justo e preciso que contar pistas |
| Muito Difícil | Pode exigir backtracking/chute | Quando a lógica pura não basta |
| Organização | Crate Rust único `sudoku-core` | Simples, testável isolado, fácil de evoluir depois |
| Dependências externas | Apenas `rand` (RNG com seed) | Mínimo necessário; nada de Tauri/IO/UI |

### Abordagem escolhida vs. alternativas

- **Escolhida — crate único `sudoku-core`:** biblioteca pura, testada com `cargo test`,
  sem dependências de Tauri/UI/IO. Fácil de promover a workspace multi-crate depois, se necessário.
- Rejeitada — workspace multi-crate desde já: separação prematura, fricção sem ganho (YAGNI).
- Rejeitada — engine dentro do app Tauri: acopla lógica ao app, dificulta teste isolado.

---

## 4. Mapeamento de dificuldade (técnica → nível)

Cada nível **inclui** as técnicas dos níveis anteriores. O nível de um tabuleiro é
definido pela **técnica mais avançada necessária** para resolvê-lo.

| Nível | Técnica mais avançada necessária |
|-------|----------------------------------|
| **Fácil** | Naked Single, Hidden Single |
| **Médio** | + Naked/Hidden Pairs, Pointing Pairs |
| **Difícil** | + Naked/Hidden Triples, Box-Line Reduction, X-Wing |
| **Muito Difícil** | + Swordfish, XY-Wing — e/ou exige backtracking quando a lógica não basta |

---

## 5. Arquitetura — submódulos

Cada submódulo é uma unidade isolada, com propósito único e testável independentemente.

### 5.1 `grid`
Tipos base e representação do tabuleiro.
- `Grid` — a grade 9×9.
- `Cell` — `Empty` ou `Filled(1..=9)`.
- `Position` (e newtypes `Row`/`Col`) — coordenadas seguras.
- Candidatos por célula como bitmask (`u16`).
- Parsing/serialização via string de 81 chars (`.` = vazio).
- Detecção de conflitos linha/coluna/box.

### 5.2 `techniques`
Cada técnica de resolução é uma unidade isolada implementando o trait `Technique`,
que examina o estado da grade e devolve uma `Deduction` (célula resolvida ou
candidatos eliminados) + nome + nível.
- Técnicas: NakedSingle, HiddenSingle, Naked/Hidden Pairs, PointingPairs,
  Naked/Hidden Triples, BoxLineReduction, X-Wing, Swordfish, XY-Wing.

### 5.3 `solver`
- `LogicalSolver` — aplica técnicas em ordem de dificuldade e produz a **lista de passos**
  (serve para o rating **e** para a dica).
- `BacktrackingSolver` — resolve por força bruta e **conta soluções** (verifica unicidade).

### 5.4 `rating`
- Roda o `LogicalSolver`, registra a técnica mais avançada usada → mapeia para `Difficulty`.
- Se a lógica empacar e só o backtracking resolver → marca **MuitoDificil**.

### 5.5 `generator`
- Gera grade completa válida (backtracking randomizado **com seed**).
- Remove células mantendo solução única (verificado pelo `BacktrackingSolver`),
  mirando o nível pedido (verificado pelo `rating`).

---

## 6. API pública

```rust
pub enum Difficulty { Facil, Medio, Dificil, MuitoDificil }

pub struct Puzzle {
    givens: Grid,
    solution: Grid,
    difficulty: Difficulty,
    seed: u64,
}

/// Gera um puzzle determinístico no nível pedido.
fn generate(difficulty: Difficulty, seed: u64) -> Result<Puzzle, GenError>;

/// Resolve logicamente, retornando os passos (técnicas aplicadas).
fn solve(grid: &Grid) -> SolveResult;

/// Classifica a dificuldade de um tabuleiro.
fn rate(grid: &Grid) -> Difficulty;

/// Lista conflitos atuais (linha/coluna/box).
fn validate(grid: &Grid) -> Vec<Conflict>;

/// Próxima jogada logicamente dedutível — reutiliza `techniques`.
/// Alimenta a dica do jogo no módulo Game (futuro).
fn next_hint(grid: &Grid) -> Option<Deduction>;
```

---

## 7. Fluxo de geração

```
seed → RNG → grade resolvida completa
           → remover células (respeitando unicidade via BacktrackingSolver)
           → rate (classificar nível)
           → bate o nível alvo?
                 sim → devolve Puzzle
                 não → tenta outro padrão de remoção
                       (até N tentativas; senão GenError)
```

---

## 8. Tratamento de erros

- Entrada de grade inválida → `Result` com erro descritivo.
- Geração: número limitado de tentativas. Se não atingir o nível exato em N tentativas,
  retorna `GenError` (não trava).
- Determinismo é contrato: mesma seed + mesmo nível → puzzle idêntico, sempre.

---

## 9. Estratégia de testes (TDD)

O teste vem **antes** da implementação em cada unidade.

- **Técnicas:** cada uma com grade-fixture onde **só ela** se aplica → deduz o esperado.
- **Solver lógico:** puzzles conhecidos → solução conhecida + sequência de passos esperada.
- **Unicidade:** grades com múltiplas soluções são detectadas pelo `BacktrackingSolver`.
- **Generator:**
  - todo puzzle gerado tem solução única;
  - o rating do puzzle bate com o nível pedido;
  - mesma seed + nível → puzzle byte-a-byte idêntico (determinismo).
- **Validação:** conflitos em linha/coluna/box detectados corretamente.
- **`grid`:** parsing/serialização round-trip (string de 81 chars).

---

## 10. Notas didáticas (patterns ↔ equivalente web)

Esta seção liga cada parte do Engine a um pattern e ao equivalente do mundo web,
como guia de aprendizado durante a implementação.

### Strategy — organiza as técnicas (`techniques` + `solver`)
Cada técnica é uma estratégia intercambiável via **trait** `Technique`. O solver guarda
`Vec<Box<dyn Technique>>` e tenta uma por uma. Adicionar técnica nova **não toca no solver**
— só registra na lista (Open/Closed na prática).
- **Web:** `interface Technique { apply(grid) }` com várias classes, ou um array de funções
  com a mesma assinatura. Diferença: o compilador Rust força implementação completa.

### Algebraic Data Types (enum + match) — modelagem de dados (`grid`)
`Cell` é `Empty | Filled(u8)`; `Difficulty` é um enum fechado. `match` exige tratar
todos os casos (exaustividade).
- **Web:** discriminated unions do TS (`{type:'empty'} | {type:'filled', n}`), mas sem
  `default:` esquecido — o compilador garante a exaustividade.

### Result / Option — ausência e falha (toda a API)
Rust não tem exceptions nem null.
- `Option<T>` (`Some`/`None`) no lugar de `value | null` → ex.: `next_hint`.
- `Result<T, E>` (`Ok`/`Err`) no lugar de `throw`/`try-catch` → ex.: `generate`.

### Newtype — segurança barata (`grid`)
`struct Row(u8)`, `struct Col(u8)` evitam trocar linha por coluna sem perceber.
- **Web:** "branded types" do TS, mas de graça e à prova de erro em compilação.

### Iterator — varredura de células (em todo lugar)
`grid.cells().filter(...).map(...)` — lazy e sem custo.
- **Web:** métodos de array (`.filter`/`.map`), que você já conhece — só que zero-cost.

### Patterns deliberadamente evitados
Factory/Singleton/DI containers, herança profunda e "classes manager" do mundo OO não
entram: Rust resolve composição com traits + funções. Importá-los aqui só adicionaria
cerimônia (YAGNI).

**Espinha dorsal:** Strategy (traits) organiza as técnicas, enums+match modelam os dados,
Result/Option modelam ausência e falha.

---

## 11. Considerações futuras (fora deste spec)

- **Módulo Game:** consumirá `validate` e `next_hint`; guardará estado de partida, timer,
  contagem de erros e anotações.
- **`next_hint`** já é desenhado pensando na feature de dica do jogo.
- **Seed pública:** a geração determinística por seed habilita "compartilhar puzzle por
  código" no futuro (inclusive base para o multiplayer).
- Promoção para workspace multi-crate só se/quando houver necessidade real.
