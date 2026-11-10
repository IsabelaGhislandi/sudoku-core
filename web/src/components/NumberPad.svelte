<script lang="ts">
  let { disabled, onNumber }: { disabled: boolean; onNumber: (value: number) => void } = $props()
</script>

<div class="pad">
  {#each [1, 2, 3, 4, 5, 6, 7, 8, 9] as n (n)}
    <button {disabled} onclick={() => onNumber(n)}>{n}</button>
  {/each}
</div>

<style>
  .pad {
    display: grid;
    grid-template-columns: repeat(9, 1fr);
    gap: 4px;
  }
  button {
    min-width: 0;
    font-weight: 700;
    font-size: 1.1rem;
  }
</style>
