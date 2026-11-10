import { levelLabel } from './levels'
import type { HistoryEntry } from './types'

/**
 * A linha do histórico: `Médio — 10 min — 13/06/2026`, mais o rótulo de como terminou. Vencer não
 * ganha rótulo (spec, seção 2). `timeZone` existe para os testes; na tela, vale o fuso do navegador.
 */
export function formatEntry(
  entry: HistoryEntry,
  timeZone?: string,
): { text: string; label: string | null } {
  const date = new Intl.DateTimeFormat('pt-BR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    timeZone,
  }).format(entry.finishedAtMs)
  const text = `${levelLabel(entry.difficulty)} — ${formatMinutes(entry.elapsedMs)} — ${date}`
  return { text, label: OUTCOME_LABELS[entry.outcome] }
}

const OUTCOME_LABELS: Record<HistoryEntry['outcome'], string | null> = {
  Won: null,
  OutOfLives: 'Derrota',
  Abandoned: 'Largada',
}

/** O tempo jogado em minutos arredondados; abaixo de um minuto, `< 1 min`. */
function formatMinutes(elapsedMs: number): string {
  return elapsedMs < 60_000 ? '< 1 min' : `${Math.round(elapsedMs / 60_000)} min`
}
