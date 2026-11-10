import { describe, expect, it } from 'vitest'
import { formatClock, randomSeed } from './clock'

describe('clock', () => {
  it('seed aleatória é um u64 em decimal', () => {
    const seeds = Array.from({ length: 50 }, () => randomSeed())
    for (const seed of seeds) {
      expect(seed).toMatch(/^\d+$/)
      expect(BigInt(seed) <= 18446744073709551615n).toBe(true)
    }
    expect(new Set(seeds).size).toBeGreaterThan(1)
  })

  it('formata o relógio da barra', () => {
    expect(formatClock(0)).toBe('00:00')
    expect(formatClock(222_000)).toBe('03:42')
    expect(formatClock(59_999)).toBe('00:59')
    expect(formatClock(3_723_000)).toBe('1:02:03')
  })
})
