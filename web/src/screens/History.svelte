<script lang="ts">
  // Provisório (Task 54): a lista de partidas entra na Task 56.
  let { onHome }: { onHome: () => void } = $props()
</script>

<section class="history">
  <h2>Histórico</h2>
  <button onclick={onHome}>Início</button>
</section>

<style>
  .history {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  h2 {
    text-align: center;
  }
</style>
