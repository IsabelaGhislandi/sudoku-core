import { describe, expect, it } from 'vitest'
import { closesLine } from './lines'
import type { CellView } from './types'

const filled = (value: number): CellView => ({ kind: 'placed', value, notes: [] })
const empty = (): CellView => ({ kind: 'empty', value: null, notes: [] })

/** Um tabuleiro vazio com as células de `indices` preenchidas. */
function boardWith(indices: number[]): CellView[] {
  return Array.from({ length: 81 }, (_, i) => (indices.includes(i) ? filled((i % 9) + 1) : empty()))
}

const range = (from: number, count: number, step = 1) =>
  Array.from({ length: count }, (_, k) => from + k * step)

describe('lines', () => {
  it('fechar a última célula de uma linha fecha a linha', () => {
    const row = range(18, 9) // a terceira linha
    const before = boardWith(row.slice(0, 8))
    const after = boardWith(row)

    expect(closesLine(before, after)).toBe(true)
  })

  it('fechar a última célula de uma coluna fecha a coluna', () => {
    const column = range(4, 9, 9) // a quinta coluna
    const before = boardWith(column.slice(1))
    const after = boardWith(column)

    expect(closesLine(before, after)).toBe(true)
  })

  it('jogada que não completa nada, ou tabuleiro que esvazia, não fecha linha', () => {
    const row = range(0, 9)
    const complete = boardWith(row)

    expect(closesLine(complete, boardWith([...row, 40]))).toBe(false)
    expect(closesLine(boardWith([10]), boardWith([10, 11]))).toBe(false)
    expect(closesLine(complete, boardWith([]))).toBe(false)
  })
})
