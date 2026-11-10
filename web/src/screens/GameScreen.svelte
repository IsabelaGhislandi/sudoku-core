<script lang="ts">
  import { onDestroy } from 'svelte'
  import Board from '../components/Board.svelte'
  import NumberPad from '../components/NumberPad.svelte'
  import { formatClock, monotonicNow } from '../lib/clock'
  import { levelLabel } from '../lib/levels'
  import type { GameView } from '../lib/types'
  import type { WebGame } from '../lib/wasm-pkg/sudoku_wasm'

  // `onExit` entra no tipo agora e ganha uso na Task 54, com a seta de voltar.
  let { game, onFinished }: { game: WebGame; onExit: () => void; onFinished: () => void } =
    $props()

  // O retrato é a única fonte do que a tela mostra. Depois de cada ação, ele é pedido de novo ao
  // WASM: a tela nunca calcula sozinha vidas, erros ou o fim da partida.
  let view: GameView = $state(readView())
  let selected: number | null = $state(null)
  let wrong: number | null = $state(null)
  let wrongTimer: ReturnType<typeof setTimeout> | undefined

  function readView(): GameView {
    return JSON.parse(game.view(monotonicNow()))
  }

  function refresh() {
    view = readView()
    if (view.status !== 'playing') onFinished()
  }

  // O relógio da barra anda sozinho: um retrato novo por segundo.
  const tick = setInterval(() => {
    view = readView()
  }, 1000)
  onDestroy(() => {
    clearInterval(tick)
    clearTimeout(wrongTimer)
  })

  function place(value: number) {
    if (selected === null) return
    const index = selected
    try {
      if (game.place(index, value, monotonicNow()) === 'wrong') flashWrong(index)
    } catch {
      // Jogada recusada (célula ocupada, partida terminada): não muda nada na partida.
    }
    refresh()
  }

  function flashWrong(index: number) {
    wrong = index
    clearTimeout(wrongTimer)
    wrongTimer = setTimeout(() => (wrong = null), 600)
  }

  const isOver = $derived(view.status !== 'playing')
</script>

<section class="game">
  <header class="bar">
    <span>{levelLabel(view.difficulty)}</span>
    <span>⏱ {formatClock(view.elapsedMs)}</span>
    <span aria-label={`${view.livesLeft} vidas`}>{'❤'.repeat(view.livesLeft)}</span>
  </header>

  <Board cells={view.cells} {selected} {wrong} onSelect={(i) => (selected = i)} />

  <NumberPad disabled={isOver || selected === null} onNumber={place} />
</section>

<style>
  .game {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    min-height: 44px;
  }
</style>
