# Fundamentos — consolidado das Tarefas 56 a 60

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 56 a
> 60. As páginas por-task ([56](../56-historico.md) … [60](../60-selva-viva.md)) contam a história na ordem do
> código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 51 a 55](rust-base-tarefas-51-a-55.md).

Este bloco fecha o frontend jogável e começa a enfeitá-lo. O histórico (56) e o CI do app (57) terminam o caminho
do jogo. O destaque de números iguais (58) e a selva com o Beni e o Valentino (59, 60) são as primeiras coisas
**só da tela**: nenhuma delas toca no Rust. O fio que liga as cinco é **onde cada decisão mora**. A lógica fica em
funções puras e testáveis, dos dois lados da ponte, e a casca (Svelte, canvas, CI) só junta as peças.

Sumário:
1. [Entrada de fora se confere item a item](#1-entrada-de-fora-se-confere-item-a-item)
2. [Lockfile como contrato, no Cargo e no npm](#2-lockfile-como-contrato-no-cargo-e-no-npm)
3. [O que pode ser calculado não se guarda](#3-o-que-pode-ser-calculado-não-se-guarda)
4. [Eventos que a ponte não manda: comparar antes e depois](#4-eventos-que-a-ponte-não-manda-comparar-antes-e-depois)
5. [Serviços entram de fora: relógio, seed e sorteio](#5-serviços-entram-de-fora-relógio-seed-e-sorteio)
6. [União discriminada é o `enum` com dados](#6-união-discriminada-é-o-enum-com-dados)
7. [Provar o vermelho](#7-provar-o-vermelho)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. Entrada de fora se confere item a item

O histórico vem do `localStorage`, e qualquer coisa que vem de lá pode ter sido gravada por uma versão antiga ou
editada à mão. A ponte lê o array como `Vec<Value>` (JSON genérico) e só depois confere cada elemento:

```rust
records
    .into_iter()
    .filter_map(|record| serde_json::from_value::<SavedResult>(record).ok())
    .filter_map(|saved| GameResult::restore(&saved).ok())
```

Ler direto como `Vec<SavedResult>` faria um único registro ruim derrubar o histórico inteiro
([56](../56-historico.md)). O `filter_map` com `.ok()` diz "tente converter; o que falhar, ignore". No TypeScript,
o equivalente é um `flatMap` que devolve `[]` para descartar. A lição vale para qualquer fronteira: a forma do
todo não pode depender de cada parte estar certa.

## 2. Lockfile como contrato, no Cargo e no npm

O CI do Rust já usava `--locked`. O job `web` usa `npm ci`, que tem o mesmo papel: instala exatamente o que está no
`package-lock.json` e falha se ele estiver desatualizado ([57](../57-ci-do-frontend.md)). Sem isso, um PR poderia
passar no CI com versões diferentes das de quem desenvolveu. O lockfile entra no git, e mudar uma dependência é
uma mudança revisável como qualquer outra.

O job roda num sistema só, porque `.wasm` e JavaScript são os mesmos bytes em qualquer lugar. Os testes do Rust
rodam em três porque o compilador nativo muda de um para outro.

## 3. O que pode ser calculado não se guarda

O número destacado ([58](../58-destacar-numeros-iguais.md)) não é estado: é um `$derived` a partir do retrato, da
seleção e do botão. A selva ([59](../59-selva-e-bananas.md)) também não guarda "quais linhas já fecharam": conta as
linhas completas no retrato de agora e compara com o de antes.

Guardar um valor que dá para calcular abre espaço para ele ficar velho. Alguém precisaria lembrar de atualizá-lo
em cada caminho que muda a origem, e bastaria esquecer um. No React, seria um `useMemo` ou uma variável no corpo do
componente. No Rust, um método que calcula na hora, em vez de um campo mantido em dia.

## 4. Eventos que a ponte não manda: comparar antes e depois

O WASM não avisa "uma linha fechou". A tela tem dois retratos e decide sozinha:

```ts
export function closesLine(before: CellView[], after: CellView[]): boolean {
  return completeLines(after) > completeLines(before)
}
```

O `>` resolve casos que pareciam pedir código especial: linha e coluna fechando juntas viram um `true` só,
recomeçar esvazia o tabuleiro e dá `false`, e abrir uma partida salva não passa por comparação nenhuma. Derivar o
evento da mudança de estado, em vez de pedir à fonte que o emita, mantém a partida em Rust sem saber de selva.

Na tela, o evento vira um **contador** (`drops`). Um booleano não serviria: a segunda linha fechada não mudaria
nada. Na versão em SVG, um `{#key drops}` recriava as bananas para reiniciar a animação CSS (59). Na versão em
canvas, um `$effect` chama `dropBananas` quando o número sobe (60).

## 5. Serviços entram de fora: relógio, seed e sorteio

A regra do `sudoku-core`, "o core recebe dados, não serviços", reaparece no TypeScript:

| Onde | O serviço | Como entra |
|---|---|---|
| `sudoku-core` | aleatoriedade do gerador | a `seed` (e o `ChaCha12Rng` montado a partir dela) |
| `sudoku-game` | relógio | `now` como parâmetro de cada ação |
| `lib/jungle/sim.ts` | sorteio do que eles fazem | `random: () => number` em `createJungle` |

Com o sorteio injetado, o teste passa um gerador com seed e a "vida" do Beni e do Valentino se repete igual a
cada execução ([60](../60-selva-viva.md)). O mesmo vale para o desenho: o `sim.ts` não conhece canvas, e por isso
roda no Vitest, sem tela.

## 6. União discriminada é o `enum` com dados

O plano de cada orangotango é uma fila de passos:

```ts
type Step =
  | { type: 'walk'; x: number; speed?: number }
  | { type: 'climb'; y: number }
  | { type: 'eat'; t: number }
  // ...
```

É o `enum Step { Walk { x, speed }, Climb { y }, Eat { t } }` do Rust. O `switch (step.type)` estreita o tipo em
cada `case`, como o `match` desestrutura cada variante. O `Record<Level, string>` dos rótulos do histórico
(56) é o outro lado da mesma ideia: um nível novo sem rótulo não compila, como um `match` sem a variante nova.

## 7. Provar o vermelho

Três páginas deste bloco têm um teste que passou de primeira e precisou de prova:

- os registros ruins do histórico (56): a ponte passou a aceitar versão desconhecida de propósito, o teste
  falhou (3 entradas em vez de 2), e a quebra foi desfeita;
- o CI (57): um teste com o valor esperado trocado e uma constante com tipo errado fizeram o `test` e o `check`
  falharem, cada um no erro que devia pegar;
- o "não fecha linha" (59): o `>` virou `!==` e `>=`, e nos dois o teste falhou.

Um teste que nunca foi visto falhando pode estar testando nada. A prova custa um minuto e transforma "passou"
em "pega o erro".

## 8. Tabela-resumo dos conceitos

| Conceito | Rust | TypeScript / Svelte | Onde apareceu |
|---|---|---|---|
| Descartar itens inválidos | `filter_map` + `.ok()` | `flatMap` devolvendo `[]` | 56 |
| Ordem decrescente | `sort_by_key(\|e\| Reverse(e.x))` | `sort((a, b) => b.x - a.x)` | 56 |
| Exaustividade | `match` sem `_` | `Record<Union, T>`, `switch` estreitando | 56, 60 |
| Lockfile no CI | `cargo --locked` | `npm ci` | 57 |
| Valor derivado | método que calcula | `$derived`, `useMemo` | 58 |
| Evento a partir do estado | comparar dois `&[T]` | `closesLine(before, after)` | 59 |
| Reiniciar animação | — | `{#key}` (Svelte), `key` (React) | 59 |
| Serviço injetado | `impl Rng`, `now: u64` | `random: () => number` | 60 |
| Laço com tempo real | `dt` em segundos | `requestAnimationFrame` + `dt` | 60 |
| Valor inicial de propósito | — | `untrack(() => x)` | 60 |
