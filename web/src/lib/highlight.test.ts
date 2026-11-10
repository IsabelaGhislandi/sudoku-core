import { describe, expect, it } from 'vitest'
import { highlightedValue } from './highlight'
import type { CellView } from './types'

const given = (value: number): CellView => ({ kind: 'given', value, notes: [] })
const placed = (value: number): CellView => ({ kind: 'placed', value, notes: [] })
const hinted = (value: number): CellView => ({ kind: 'hinted', value, notes: [] })
const empty = (notes: number[] = []): CellView => ({ kind: 'empty', value: null, notes })

describe('highlight', () => {
  it('destaca o número da célula preenchida selecionada', () => {
    const cells = [given(8), placed(3), hinted(5), empty([8])]

    expect(highlightedValue(cells, 0, true)).toBe(8)
    expect(highlightedValue(cells, 1, true)).toBe(3)
    expect(highlightedValue(cells, 2, true)).toBe(5)
  })

  it('não destaca célula vazia, nada selecionado ou com o botão desligado', () => {
    const cells = [given(8), empty([8])]

    expect(highlightedValue(cells, 1, true)).toBeNull()
    expect(highlightedValue(cells, null, true)).toBeNull()
    expect(highlightedValue(cells, 0, false)).toBeNull()
  })
})
