// Espelho, em TypeScript, do JSON que a ponte (`sudoku-wasm`) devolve. Os nomes e os valores
// possíveis vêm do Rust; se um mudar lá, muda aqui.

/** O nível, pelo nome do Rust. */
export type Level = 'Facil' | 'Medio' | 'Dificil' | 'MuitoDificil'

/** Uma célula do retrato da partida. */
export interface CellView {
  kind: 'empty' | 'given' | 'placed' | 'hinted'
  value: number | null
  notes: number[]
}

/** O retrato da partida, como `WebGame.view` devolve. */
export interface GameView {
  difficulty: Level
  status: 'playing' | 'won' | 'lost'
  paused: boolean
  elapsedMs: number
  livesLeft: number
  hintsLeft: number
  cells: CellView[]
}
