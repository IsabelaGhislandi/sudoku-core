import type { Level } from './types'

/** Os quatro níveis, na ordem da tela de escolha, com o rótulo mostrado na tela. */
export const LEVELS: { id: Level; label: string }[] = [
  { id: 'Facil', label: 'Fácil' },
  { id: 'Medio', label: 'Médio' },
  { id: 'Dificil', label: 'Difícil' },
  { id: 'MuitoDificil', label: 'Muito Difícil' },
]

/** O rótulo de um nível. */
export function levelLabel(level: Level): string {
  return LEVELS.find((l) => l.id === level)?.label ?? level
}
