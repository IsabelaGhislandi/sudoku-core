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
  // Modo anotar é estado só da tela: a partida não sabe dele, só recebe `place` ou `toggleNote`.
  let noting = $state(false)
  let wrongTimer: ReturnType<typeof setTimeout> | undefined

  function readView(): GameView {
    return JSON.parse(game.view(monotonicNow()))
  }

  function refresh() {
    view = readView()
    if (view.status !== 'playing') onFinished()
  }

  // Uma ação recusada pela partida (célula ocupada, pausada, sem dicas) não muda nada nela; a tela
  // só desenha de novo.
  function act(action: () => void) {
    try {
      action()
    } catch {
      // recusada: nada a fazer
    }
    refresh()
  }

  // O relógio da barra anda sozinho: um retrato novo por segundo.
  const tick = setInterval(() => {
    view = readView()
  }, 1000)

  // Trocar de aba ou minimizar pausa a partida: o tempo não corre com a pessoa fora.
  function onVisibilityChange() {
    if (document.visibilityState === 'hidden') act(() => game.pause(monotonicNow()))
  }
  document.addEventListener('visibilitychange', onVisibilityChange)

  onDestroy(() => {
    clearInterval(tick)
    clearTimeout(wrongTimer)
    document.removeEventListener('visibilitychange', onVisibilityChange)
  })

  function onNumber(value: number) {
    if (selected === null) return
    const index = selected
    act(() => {
      if (noting) {
        game.toggleNote(index, value)
      } else if (game.place(index, value, monotonicNow()) === 'wrong') {
        flashWrong(index)
      }
    })
  }

  function flashWrong(index: number) {
    wrong = index
    clearTimeout(wrongTimer)
    wrongTimer = setTimeout(() => (wrong = null), 600)
  }

  function togglePause() {
    act(() => (view.paused ? game.resume(monotonicNow()) : game.pause(monotonicNow())))
  }

  const isOver = $derived(view.status !== 'playing')
  const isBlocked = $derived(isOver || view.paused)
</script>

<section class="game">
  <header class="bar">
    <span>{levelLabel(view.difficulty)}</span>
    <span class="clock">
      ⏱ {formatClock(view.elapsedMs)}
      <button
        class="pause"
        disabled={isOver}
        aria-label={view.paused ? 'Continuar' : 'Pausar'}
        onclick={togglePause}>{view.paused ? '▶' : '⏸'}</button
      >
    </span>
    <span aria-label={`${view.livesLeft} vidas`}>{'❤'.repeat(view.livesLeft)}</span>
  </header>

  {#if view.paused && !isOver}
    <!-- Pausado, o tabuleiro some: a pausa não vira tempo grátis para pensar (spec, seção 2). -->
    <button class="paused" onclick={togglePause}>Pausado — toque para continuar</button>
  {:else}
    <Board cells={view.cells} {selected} {wrong} onSelect={(i) => (selected = i)} />
  {/if}

  <div class="tools">
    <button class:active={noting} disabled={isBlocked} onclick={() => (noting = !noting)}>
      ✏ Anotar
    </button>
    <button
      disabled={isBlocked || view.hintsLeft === 0}
      onclick={() => act(() => game.hint(monotonicNow()))}
    >
      💡 Dica ({view.hintsLeft})
    </button>
  </div>

  <NumberPad disabled={isBlocked || selected === null} {onNumber} />
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
  .clock {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .pause {
    border: none;
  }
  .paused {
    aspect-ratio: 1;
    width: 100%;
    border: 2px solid var(--line-strong);
    border-radius: 0;
    color: var(--muted);
  }
  .tools {
    display: flex;
    justify-content: space-around;
    gap: 8px;
  }
  .tools button {
    flex: 1;
  }
  .tools button.active {
    background: var(--selected);
    border-color: var(--accent);
  }
</style>
