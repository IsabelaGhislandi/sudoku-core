import type { CellView } from './types'

const SIZE = 9

/**
 * Se a jogada que levou de `before` para `after` fechou alguma linha ou coluna. É o que derruba as
 * bananas da selva. Só da tela: a partida não sabe de selva nenhuma.
 */
export function closesLine(before: CellView[], after: CellView[]): boolean {
  return completeLines(after) > completeLines(before)
}

/** Quantas linhas e colunas estão completas, somadas. */
function completeLines(cells: CellView[]): number {
  const isFilled = (index: number) => cells[index]?.value != null
  let count = 0
  for (let n = 0; n < SIZE; n++) {
    const row = Array.from({ length: SIZE }, (_, k) => n * SIZE + k)
    const column = Array.from({ length: SIZE }, (_, k) => k * SIZE + n)
    if (row.every(isFilled)) count++
    if (column.every(isFilled)) count++
  }
  return count
}
