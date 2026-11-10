<script lang="ts">
  import { tick } from 'svelte'
  import GameScreen from './screens/GameScreen.svelte'
  import History from './screens/History.svelte'
  import Home from './screens/Home.svelte'
  import Levels from './screens/Levels.svelte'
  import { monotonicNow, randomSeed, wallClockNow } from './lib/clock'
  import { historyAction, nextScreen, type NavEvent, type Screen } from './lib/navigation'
  import { appendResult, browserStorage, clearSavedGame, loadSavedGame } from './lib/storage'
  import type { Level } from './lib/types'
  import { WebGame } from './lib/wasm-pkg/sudoku_wasm'

  const storage = browserStorage()

  let screen: Screen = $state('home')
  // A partida aberta nesta aba, se houver. Fora da tela do jogo, ela fica pausada e guardada.
  let game: WebGame | null = $state(null)
  let generating = $state(false)

  // Cada troca de tela entra no histórico do navegador: assim o botão voltar (do navegador e, no
  // APK, do Android) troca de tela em vez de sair do site.
  history.replaceState({ screen: 'home' }, '')

  function navigate(event: NavEvent) {
    const next = nextScreen(screen, event)
    const action = historyAction(screen, next)
    screen = next
    if (action === 'push') history.pushState({ screen: next }, '')
    else if (action === 'replace') history.replaceState({ screen: next }, '')
    // Voltar ao início desfaz a única entrada acima dele; o `popstate` que vem depois só confirma
    // a tela que já está aqui.
    else if (action === 'back') history.back()
  }

  window.addEventListener('popstate', () => {
    screen = nextScreen(screen, { type: 'back' })
  })

  /** A partida que o Continuar abriria: a desta aba, ou a guardada no navegador. */
  function currentGame(): WebGame | null {
    if (game) return game
    const saved = loadSavedGame(storage)
    if (saved === null) return null
    try {
      return WebGame.restore(saved)
    } catch {
      // Foto ilegível (corrompida, versão desconhecida): descartada, sem entrar no histórico.
      clearSavedGame(storage)
      return null
    }
  }

  function continueGame() {
    game = currentGame()
    if (game) navigate({ type: 'continue' })
  }

  async function chooseLevel(level: Level) {
    const previous = currentGame()
    if (previous) {
      // Começar outra larga a guardada, e o abandono vai para o histórico sem volta.
      if (!confirm('Largar a partida guardada?')) return
      const abandoned = previous.abandon(monotonicNow(), wallClockNow())
      if (abandoned !== undefined) appendResult(storage, abandoned)
      clearSavedGame(storage)
      game = null
    }
    // Gerar um puzzle difícil pode levar centenas de milissegundos e bloqueia a thread. Antes de
    // gerar, a tela mostra "Gerando..." e cede um quadro, para não congelar sem retorno.
    generating = true
    await tick()
    await new Promise(requestAnimationFrame)
    game = new WebGame(level, randomSeed())
    generating = false
    navigate({ type: 'levelChosen' })
  }

  function finishGame() {
    // O GameScreen já gravou o resultado e apagou a foto; a partida terminada sai da memória.
    game?.free()
    game = null
    navigate({ type: 'gameOver' })
  }
</script>

<main>
  {#if generating}
    <p class="generating">Gerando...</p>
  {:else if screen === 'home'}
    <Home
      canContinue={game !== null || loadSavedGame(storage) !== null}
      onContinue={continueGame}
      onPlay={() => navigate({ type: 'play' })}
      onHistory={() => navigate({ type: 'openHistory' })}
    />
  {:else if screen === 'levels'}
    <Levels onChoose={chooseLevel} onBack={() => navigate({ type: 'back' })} />
  {:else if screen === 'game' && game}
    <GameScreen {game} onExit={() => navigate({ type: 'back' })} onFinished={finishGame} />
  {:else if screen === 'history'}
    <History onHome={() => navigate({ type: 'home' })} />
  {/if}
</main>

<style>
  .generating {
    text-align: center;
    margin-top: 40vh;
  }
</style>
