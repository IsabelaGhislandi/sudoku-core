<script lang="ts">
  import { tick } from 'svelte'
  import GameScreen from './screens/GameScreen.svelte'
  import { monotonicNow, randomSeed } from './lib/clock'
  import { WebGame } from './lib/wasm-pkg/sudoku_wasm'

  // Provisório (Task 52): começa direto uma partida Fácil. As telas de início e de nível entram
  // na Task 54.
  let game: WebGame | null = $state(null)

  async function start() {
    // Gerar um puzzle difícil pode levar centenas de milissegundos e bloqueia a thread. Antes de
    // gerar, a tela mostra "Gerando..." e cede um quadro, para não congelar sem retorno.
    await tick()
    await new Promise(requestAnimationFrame)
    const next = new WebGame('Facil', randomSeed())
    next.resume(monotonicNow())
    game = next
  }

  start()
</script>

<main>
  {#if game}
    <GameScreen {game} onExit={() => {}} onFinished={() => {}} />
  {:else}
    <p class="generating">Gerando...</p>
  {/if}
</main>

<style>
  .generating {
    text-align: center;
    margin-top: 40vh;
  }
</style>
