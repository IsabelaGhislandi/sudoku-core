<script lang="ts">
  import { tick } from 'svelte'
  import GameScreen from './screens/GameScreen.svelte'
  import History from './screens/History.svelte'
  import Home from './screens/Home.svelte'
  import Levels from './screens/Levels.svelte'
  import { randomSeed } from './lib/clock'
  import { nextScreen, type NavEvent, type Screen } from './lib/navigation'
  import type { Level } from './lib/types'
  import { WebGame } from './lib/wasm-pkg/sudoku_wasm'

  let screen: Screen = $state('home')
  let game: WebGame | null = $state(null)
  let generating = $state(false)

  // Cada troca de tela entra no histórico do navegador: assim o botão voltar (do navegador e, no
  // APK, do Android) troca de tela em vez de sair do site.
  history.replaceState({ screen: 'home' }, '')

  function navigate(event: NavEvent) {
    const next = nextScreen(screen, event)
    if (next === screen) return
    screen = next
    history.pushState({ screen }, '')
  }

  window.addEventListener('popstate', () => {
    screen = nextScreen(screen, { type: 'back' })
  })

  async function chooseLevel(level: Level) {
    // Gerar um puzzle difícil pode levar centenas de milissegundos e bloqueia a thread. Antes de
    // gerar, a tela mostra "Gerando..." e cede um quadro, para não congelar sem retorno.
    generating = true
    await tick()
    await new Promise(requestAnimationFrame)
    game?.free()
    game = new WebGame(level, randomSeed())
    generating = false
    navigate({ type: 'levelChosen' })
  }
</script>

<main>
  {#if generating}
    <p class="generating">Gerando...</p>
  {:else if screen === 'home'}
    <Home onPlay={() => navigate({ type: 'play' })} onHistory={() => navigate({ type: 'openHistory' })} />
  {:else if screen === 'levels'}
    <Levels onChoose={chooseLevel} onBack={() => navigate({ type: 'back' })} />
  {:else if screen === 'game' && game}
    <GameScreen
      {game}
      onExit={() => navigate({ type: 'back' })}
      onFinished={() => navigate({ type: 'gameOver' })}
    />
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
