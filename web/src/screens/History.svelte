<script lang="ts">
  import { formatEntry } from '../lib/history'
  import { browserStorage, loadHistory } from '../lib/storage'
  import type { HistoryEntry } from '../lib/types'
  import { historyEntries } from '../lib/wasm-pkg/sudoku_wasm'

  let { onHome }: { onHome: () => void } = $props()

  // O Rust confere cada registro guardado e ordena do mais recente para o mais antigo; a tela só
  // formata as linhas.
  const entries: HistoryEntry[] = JSON.parse(historyEntries(loadHistory(browserStorage())))
  const lines = entries.map((entry) => formatEntry(entry))
</script>

<section class="history">
  <h2>Histórico</h2>
  {#if lines.length === 0}
    <p class="empty">Nenhuma partida ainda.</p>
  {:else}
    <ul>
      {#each lines as line, i (i)}
        <li>
          <span>{line.text}</span>
          {#if line.label}<span class="label">{line.label}</span>{/if}
        </li>
      {/each}
    </ul>
  {/if}
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
  ul {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  li {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 0;
    border-bottom: 1px solid var(--line);
  }
  .label {
    color: var(--muted);
    font-size: 0.9rem;
  }
  .empty {
    text-align: center;
    color: var(--muted);
  }
</style>
