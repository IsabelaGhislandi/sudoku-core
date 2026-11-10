<script lang="ts">
  import { onDestroy } from 'svelte'
  import Board from '../components/Board.svelte'
  import Jungle from '../components/Jungle.svelte'
  import NumberPad from '../components/NumberPad.svelte'
  import { formatClock, monotonicNow, randomSeed, wallClockNow } from '../lib/clock'
  import { highlightedValue } from '../lib/highlight'
  import { closesLine } from '../lib/lines'
  import { levelLabel } from '../lib/levels'
  import {
    appendResult,
    browserStorage,
    clearSavedGame,
    storeSavedGame,
  } from '../lib/storage'
  import type { GameView } from '../lib/types'
  import type { WebGame } from '../lib/wasm-pkg/sudoku_wasm'

  let {
    game,
    onExit,
    onFinished,
  }: { game: WebGame; onExit: () => void; onFinished: () => void } = $props()

  // O retrato é a única fonte do que a tela mostra. Depois de cada ação, ele é pedido de novo ao
  // WASM: a tela nunca calcula sozinha vidas, erros ou o fim da partida.
  let view: GameView = $state(readView())
  let selected: number | null = $state(null)
  let wrong: number | null = $state(null)
  // Modo anotar é estado só da tela: a partida não sabe dele, só recebe `place` ou `toggleNote`.
  let noting = $state(false)
  // Destacar números iguais também é só da tela. Começa ligado.
  let highlighting = $state(true)
  // Quantas vezes uma jogada fechou linha ou coluna: cada uma derruba bananas na selva.
  let drops = $state(0)
  let wrongTimer: ReturnType<typeof setTimeout> | undefined
  // As células que piscam quando uma anotação é recusada por conflito. Quem diz quais são é o
  // WASM (`conflicts`); a tela só pisca.
  let conflicting: number[] = $state([])
  let conflictTimer: ReturnType<typeof setTimeout> | undefined
  let conflictFrame: number | undefined
  // Vira `true` no instante em que o resultado é gravado. Depois disso, nada mais é salvo nem
  // gravado: a partida terminada não volta a ser "partida guardada", nem entra duas vezes no
  // histórico.
  let recorded = false
  const storage = browserStorage()

  function readView(): GameView {
    return JSON.parse(game.view(monotonicNow()))
  }

  // A foto é gravada depois de **cada** ação, e não só ao fechar a aba: o fechamento não é
  // confiável, e no celular a aba pode morrer sem aviso (spec, seção 7).
  function persist() {
    if (!recorded) storeSavedGame(storage, game.save(monotonicNow()))
  }

  function refresh() {
    const before = view.cells
    view = readView()
    if (closesLine(before, view.cells)) drops++
    if (view.status === 'playing') {
      persist()
      return
    }
    if (recorded) return
    // Fim da partida: o resultado vai para o histórico e a partida guardada some, nessa ordem.
    const result = game.result(wallClockNow())
    if (result !== undefined) appendResult(storage, result)
    clearSavedGame(storage)
    recorded = true
    onFinished()
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
    // Sair da tela (seta, voltar do navegador) guarda a partida pausada, para o Continuar.
    if (!recorded) {
      game.pause(monotonicNow())
      persist()
    }
    clearInterval(tick)
    clearTimeout(wrongTimer)
    clearTimeout(conflictTimer)
    if (conflictFrame !== undefined) cancelAnimationFrame(conflictFrame)
    document.removeEventListener('visibilitychange', onVisibilityChange)
  })

  function onNumber(value: number) {
    if (selected === null) return
    const index = selected
    act(() => {
      if (noting) {
        noteOrFlash(index, value)
      } else if (game.place(index, value, monotonicNow()) === 'wrong') {
        flashWrong(index)
      }
    })
  }

  // A anotação recusada por conflito não entra: os números que já ocupam a linha, a coluna ou a
  // caixa piscam. As outras recusas (pausada, célula ocupada) seguem para o `act`, como antes.
  function noteOrFlash(index: number, value: number) {
    try {
      game.toggleNote(index, value)
    } catch (error) {
      if (error !== 'NoteConflict') throw error
      flashConflicts(JSON.parse(game.conflicts(index, value)))
    }
  }

  function flashConflicts(cells: number[]) {
    // Limpa e marca no quadro seguinte: tentar de novo enquanto ainda pisca recomeça a animação.
    conflicting = []
    clearTimeout(conflictTimer)
    if (conflictFrame !== undefined) cancelAnimationFrame(conflictFrame)
    conflictFrame = requestAnimationFrame(() => {
      conflicting = cells
      conflictTimer = setTimeout(() => (conflicting = []), 900)
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

  function restart() {
    // Recomeçar no meio larga a partida, e o abandono vai para o histórico sem volta: pede
    // confirmação antes (spec, seção 2). Partida já terminada recomeça direto.
    if (!isOver && !confirm('Largar esta partida e começar outra?')) return
    act(() => {
      const abandoned = game.restart(randomSeed(), monotonicNow(), wallClockNow())
      if (abandoned !== undefined) appendResult(storage, abandoned)
    })
    selected = null
  }

  const isOver = $derived(view.status !== 'playing')
  const isBlocked = $derived(isOver || view.paused)
  const highlighted = $derived(highlightedValue(view.cells, selected, highlighting))
</script>

<section class="game">
  <header class="bar">
    <button class="back" onclick={onExit}>← {levelLabel(view.difficulty)}</button>
    <span class="clock">
      ⏱ {formatClock(view.elapsedMs)}
      <button
        class="pause"
        disabled={isOver}
        onclick={togglePause}>{view.paused ? 'Continuar' : 'Pausar'}</button
      >
    </span>
    <span aria-label={`${view.livesLeft} vidas`}>{'❤'.repeat(view.livesLeft)}</span>
  </header>

  {#if view.paused && !isOver}
    <!-- Pausado, o tabuleiro some: a pausa não vira tempo grátis para pensar (spec, seção 2). -->
    <button class="paused" onclick={togglePause}>
      {view.elapsedMs === 0 ? 'Toque para começar' : 'Pausado — toque para continuar'}
    </button>
  {:else}
    <Board
      cells={view.cells}
      {selected}
      {wrong}
      {conflicting}
      {highlighted}
      onSelect={(i) => (selected = i)}
    />
  {/if}

  <div class="tools">
    <button class:active={noting} disabled={isBlocked} onclick={() => (noting = !noting)}>
      Anotar
    </button>
    <button
      disabled={isBlocked || view.hintsLeft === 0}
      onclick={() => act(() => game.hint(monotonicNow()))}
    >
      Dica ({view.hintsLeft})
    </button>
    <button
      class:active={highlighting}
      aria-pressed={highlighting}
      onclick={() => (highlighting = !highlighting)}
    >
      Destacar
    </button>
    <button onclick={restart}>Recomeçar</button>
  </div>

  <NumberPad disabled={isBlocked || selected === null} {onNumber} />

  <Jungle {drops} />
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
  .back {
    border: none;
    padding: 0 8px 0 0;
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
    font-size: 0.9rem;
    padding: 0 4px;
  }
  .tools button.active {
    background: var(--selected);
    border-color: var(--accent);
  }
</style>
