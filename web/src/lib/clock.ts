// O relógio e o sorteio da partida. O `sudoku-game` não tem relógio nem sorteio por contrato:
// quem fornece os dois é a casca, aqui.

/** O tempo de jogo no formato da barra: `mm:ss`, ou `h:mm:ss` a partir de uma hora. */
export function formatClock(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor(totalSeconds / 60) % 60
  const seconds = totalSeconds % 60
  const pad = (n: number) => String(n).padStart(2, '0')
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(seconds)}`
    : `${pad(minutes)}:${pad(seconds)}`
}

/** Uma seed aleatória de 64 bits, em decimal. */
export function randomSeed(): string {
  // Texto, e não number: o number do JS só guarda inteiros exatos até 2^53 (ver wiki 51).
  return crypto.getRandomValues(new BigUint64Array(1))[0].toString()
}

/** O `now` das jogadas: monotônico, não volta quando o relógio do sistema muda. */
export function monotonicNow(): number {
  return performance.now()
}

/** A data do resultado: relógio de parede, milissegundos desde a época Unix. */
export function wallClockNow(): number {
  return Date.now()
}
