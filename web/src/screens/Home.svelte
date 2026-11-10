<script lang="ts">
  let {
    canContinue,
    onContinue,
    onPlay,
    onHistory,
  }: {
    canContinue: boolean
    onContinue: () => void
    onPlay: () => void
    onHistory: () => void
  } = $props()
</script>

<section class="home">
  <h1>Sudoku</h1>
  {#if canContinue}
    <button class="primary" onclick={onContinue}>Continuar</button>
  {/if}
  <button class:primary={!canContinue} onclick={onPlay}>Jogar</button>
  <button onclick={onHistory}>Histórico</button>
</section>

<style>
  .home {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 20vh;
  }
  h1 {
    text-align: center;
    font-size: 2.4rem;
    margin: 0 0 24px;
  }
  .primary {
    background: var(--accent);
    border-color: var(--accent);
    color: #fff;
    font-weight: 700;
  }
</style>
