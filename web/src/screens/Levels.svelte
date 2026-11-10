<script lang="ts">
  import { LEVELS } from '../lib/levels'
  import type { Level } from '../lib/types'

  let { onChoose, onBack }: { onChoose: (level: Level) => void; onBack: () => void } = $props()
</script>

<section class="levels">
  <h2>Escolha o nível</h2>
  {#each LEVELS as level (level.id)}
    <button onclick={() => onChoose(level.id)}>{level.label}</button>
  {/each}
  <button class="back" onclick={onBack}>← voltar</button>
</section>

<style>
  .levels {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-top: 10vh;
  }
  h2 {
    text-align: center;
  }
  .back {
    border: none;
    color: var(--muted);
    margin-top: 24px;
  }
</style>
