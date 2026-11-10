// O que o app guarda no navegador: a partida em andamento (o `SavedGame` em JSON) e o histórico
// (um array de `SavedResult` em JSON). Os formatos são do Rust; aqui só se guarda e se lê texto.
//
// Nenhuma função daqui lança. O `localStorage` pode falhar (aba anônima, cota cheia, site
// bloqueado), e isso não pode impedir ninguém de jogar: o jogo continua, só não fica guardado.

/** Onde fica a partida guardada. Só existe uma por vez. */
export const SAVED_GAME_KEY = 'sudoku.savedGame'
/** Onde fica o histórico. */
export const HISTORY_KEY = 'sudoku.history'

/** A partida guardada, ou `null` se não houver. */
export function loadSavedGame(storage: Storage): string | null {
  return attempt(() => storage.getItem(SAVED_GAME_KEY), null)
}

/** Guarda a partida, no lugar da anterior. */
export function storeSavedGame(storage: Storage, json: string): void {
  attempt(() => storage.setItem(SAVED_GAME_KEY, json), undefined)
}

/** Apaga a partida guardada. */
export function clearSavedGame(storage: Storage): void {
  attempt(() => storage.removeItem(SAVED_GAME_KEY), undefined)
}

/** O histórico como texto de um array JSON. Sem histórico, `'[]'`. */
export function loadHistory(storage: Storage): string {
  return JSON.stringify(readHistory(storage))
}

/**
 * Acrescenta um resultado (`SavedResult` em JSON) ao fim do histórico. Um histórico que não é um
 * array (corrompido, editado à mão) é trocado por um novo: perder um histórico ilegível é melhor
 * do que deixar de gravar todos os próximos.
 */
export function appendResult(storage: Storage, resultJson: string): void {
  attempt(() => {
    const history = readHistory(storage)
    history.push(JSON.parse(resultJson))
    storage.setItem(HISTORY_KEY, JSON.stringify(history))
  }, undefined)
}

/**
 * O histórico como array. Cada registro é conferido depois, pelo Rust (`historyEntries`): aqui só
 * se garante que é uma lista.
 */
function readHistory(storage: Storage): unknown[] {
  return attempt(() => {
    const parsed: unknown = JSON.parse(storage.getItem(HISTORY_KEY) ?? '[]')
    return Array.isArray(parsed) ? parsed : []
  }, [])
}

/**
 * O `localStorage` do navegador. Se até ler a propriedade lançar (dados do site bloqueados), devolve
 * um armazenamento que não guarda nada, para o jogo seguir sem salvar.
 */
export function browserStorage(): Storage {
  return attempt(() => globalThis.localStorage ?? NOWHERE, NOWHERE)
}

/** Um `Storage` que não guarda nada: toda leitura volta vazia. */
const NOWHERE: Storage = {
  length: 0,
  clear: () => {},
  getItem: () => null,
  key: () => null,
  removeItem: () => {},
  setItem: () => {},
}

/** Roda `action` e devolve `fallback` se o armazenamento lançar. */
function attempt<T>(action: () => T, fallback: T): T {
  try {
    return action()
  } catch {
    return fallback
  }
}
