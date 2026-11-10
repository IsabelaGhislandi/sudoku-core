import { describe, expect, it } from 'vitest'
import { formatEntry } from './history'
import type { HistoryEntry } from './types'

const TZ = 'America/Sao_Paulo'

function entry(overrides: Partial<HistoryEntry>): HistoryEntry {
  return {
    difficulty: 'Medio',
    outcome: 'Won',
    elapsedMs: 600_000,
    finishedAtMs: Date.UTC(2026, 5, 13, 15),
    ...overrides,
  }
}

describe('history', () => {
  it('formata a linha do histórico', () => {
    expect(formatEntry(entry({}), TZ)).toEqual({
      text: 'Médio — 10 min — 13/06/2026',
      label: null,
    })
    expect(formatEntry(entry({ outcome: 'OutOfLives' }), TZ).label).toBe('Derrota')
    expect(formatEntry(entry({ outcome: 'Abandoned' }), TZ).label).toBe('Largada')
    expect(formatEntry(entry({ elapsedMs: 59_000 }), TZ).text).toBe('Médio — < 1 min — 13/06/2026')
    expect(formatEntry(entry({ elapsedMs: 90_000 }), TZ).text).toBe('Médio — 2 min — 13/06/2026')
  })
})
