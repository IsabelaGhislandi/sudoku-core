import { describe, expect, it } from 'vitest'
import { afterEach, vi } from 'vitest'
import {
  appendResult,
  browserStorage,
  clearSavedGame,
  HISTORY_KEY,
  loadHistory,
  loadSavedGame,
  storeSavedGame,
} from './storage'

/** Um `Storage` em memória, com a mesma interface do `localStorage`. */
function memoryStorage(initial: Record<string, string> = {}): Storage {
  const data = new Map(Object.entries(initial))
  return {
    get length() {
      return data.size
    },
    clear: () => data.clear(),
    getItem: (key) => data.get(key) ?? null,
    key: (index) => [...data.keys()][index] ?? null,
    removeItem: (key) => void data.delete(key),
    setItem: (key, value) => void data.set(key, String(value)),
  }
}

describe('storage', () => {
  it('acrescenta resultados ao histórico', () => {
    const storage = memoryStorage()
    expect(loadHistory(storage)).toBe('[]')

    appendResult(storage, '{"n":1}')
    appendResult(storage, '{"n":2}')
    expect(JSON.parse(loadHistory(storage))).toEqual([{ n: 1 }, { n: 2 }])

    for (const garbage of ['não é json', '{"a":1}']) {
      const broken = memoryStorage({ [HISTORY_KEY]: garbage })
      appendResult(broken, '{"n":3}')
      expect(JSON.parse(loadHistory(broken))).toEqual([{ n: 3 }])
    }
  })

  it('guarda, lê e apaga a partida', () => {
    const storage = memoryStorage()
    expect(loadSavedGame(storage)).toBeNull()
    storeSavedGame(storage, '{"version":1}')
    expect(loadSavedGame(storage)).toBe('{"version":1}')
    clearSavedGame(storage)
    expect(loadSavedGame(storage)).toBeNull()
  })

  it('ignora Storage que lança', () => {
    const fail = () => {
      throw new Error('QuotaExceededError')
    }
    const broken: Storage = { ...memoryStorage(), getItem: fail, setItem: fail, removeItem: fail }

    expect(() => storeSavedGame(broken, '{}')).not.toThrow()
    expect(() => clearSavedGame(broken)).not.toThrow()
    expect(() => appendResult(broken, '{}')).not.toThrow()
    expect(loadSavedGame(broken)).toBeNull()
    expect(loadHistory(broken)).toBe('[]')
  })

  afterEach(() => vi.unstubAllGlobals())

  it('usa um armazenamento que não guarda quando o localStorage é bloqueado', () => {
    vi.stubGlobal('localStorage', undefined)
    Object.defineProperty(globalThis, 'localStorage', {
      configurable: true,
      get() {
        throw new Error('SecurityError')
      },
    })

    const storage = browserStorage()
    storeSavedGame(storage, '{}')
    expect(loadSavedGame(storage)).toBeNull()
  })
})
