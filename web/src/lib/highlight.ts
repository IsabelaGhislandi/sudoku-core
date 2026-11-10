import type { CellView } from './types'

/**
 * O número que a tela destaca: o da célula selecionada, se ela estiver preenchida e o destaque
 * estiver ligado. Só da tela: a partida não sabe de destaque nenhum.
 */
export function highlightedValue(
  cells: CellView[],
  selected: number | null,
  enabled: boolean,
): number | null {
  if (!enabled || selected === null) return null
  return cells[selected]?.value ?? null
}
