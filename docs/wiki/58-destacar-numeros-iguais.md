# 58 — Destacar números iguais: estado derivado, `$derived` e uma função pura para a tela

> Referente a um pedido feito depois da Task 57 do plano do frontend. Commit: `feat: highlight every cell with the selected number`.

Um botão novo na tela do jogo, **Destacar**, que começa ligado. Com ele ligado, tocar numa célula
preenchida (um 8 do enunciado, um 8 que você colocou ou um revelado por dica) destaca todos os 8 do
tabuleiro. Tocar numa célula vazia tira o destaque. Ajuda a ver de relance onde um número já está e
onde ele ainda falta.

## Só da tela, de novo

A regra da wiki 53 decide onde mora o estado: na `Game` (Rust) fica o que muda o resultado ou
precisa sobreviver a fechar a aba; na tela fica o que só muda o que se vê agora. O destaque é do
segundo tipo. Ele não muda nenhuma regra, não precisa estar na foto da partida e não interessa ao
PvP. Por isso o `sudoku-wasm` e o `sudoku-game` não mudaram uma linha.

## Estado mínimo, o resto é derivado

A tela guarda um estado novo só, o do botão:

```ts
let highlighting = $state(true)   // o botão
// `selected` (a célula tocada) já existia
```

O número destacado **não** é um estado. Ele é calculado a partir do que já existe:

```ts
const highlighted = $derived(highlightedValue(view.cells, selected, highlighting))
```

Isso evita uma classe inteira de bugs. Se o número destacado fosse guardado à parte, alguém teria
de lembrar de atualizá-lo quando a seleção muda, quando o botão desliga, quando uma dica preenche a
célula... Bastaria esquecer um caso para o destaque mostrar um número que não está mais
selecionado. Com o `$derived`, o Svelte recalcula sempre que `view`, `selected` ou `highlighting`
mudam, e o valor nunca fica velho.

É o mesmo princípio de "uma fonte da verdade" que a tela já seguia com o retrato do WASM: o que
pode ser calculado não deve ser guardado. No React, seria um `useMemo` (ou só uma variável no corpo
do componente). No Rust, seria um método que calcula na hora, em vez de um campo que precisa ser
mantido em dia.

## Uma função pura, testada antes

```ts
export function highlightedValue(
  cells: CellView[],
  selected: number | null,
  enabled: boolean,
): number | null {
  if (!enabled || selected === null) return null
  return cells[selected]?.value ?? null
}
```

Três linhas, mas cada uma decide um caso do pedido: botão desligado, nada selecionado, célula vazia
(o `value` dela é `null`). Os dois testes do Vitest foram escritos antes, um de cada vez, e vistos
falhando:

```ts
expect(highlightedValue(cells, 0, true)).toBe(8)       // célula do enunciado
expect(highlightedValue(cells, 1, true)).toBeNull()    // célula vazia, mesmo com 8 anotado
expect(highlightedValue(cells, 0, false)).toBeNull()   // botão desligado
```

O `?.` e o `??` são o `Option` do TypeScript em versão curta. `cells[selected]?.value` dá
`undefined` se o índice não existir, em vez de lançar, e o `?? null` normaliza para `null`. Em Rust,
seria `cells.get(selected).and_then(|c| c.value)`: o `get` devolve `Option` em vez de entrar em
pânico com índice fora da faixa.

## O destaque é "vivo"

O tabuleiro só compara:

```svelte
class:highlighted={highlighted !== null && cell.value === highlighted}
```

Como o retrato é pedido de novo depois de cada jogada, um 8 colocado com o destaque do 8 ligado já
aparece destacado no mesmo instante. Nenhum código especial foi preciso para isso: é consequência
de o destaque ser derivado do retrato atual, e não guardado.

## Ordem das regras de CSS

```css
.cell.highlighted { background: var(--highlight); }
.cell.selected    { background: var(--selected); }
.cell.wrong       { background: var(--wrong); }
```

A célula selecionada também tem o número destacado, e as duas classes valem ao mesmo tempo. Com a
mesma especificidade, quem vem depois no CSS ganha. A ordem acima garante que a célula tocada
continue com a cor de selecionada, e o vermelho de um erro aparece por cima de tudo. As cores ficam
como variáveis em `:root`, com outra versão no tema escuro, como as demais.
