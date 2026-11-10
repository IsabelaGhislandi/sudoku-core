<script lang="ts">
  import type { CellView } from '../lib/types'

  // O tabuleiro só desenha: quem decide o que é certo ou errado é o WASM.
  let {
    cells,
    selected,
    wrong,
    onSelect,
  }: {
    cells: CellView[]
    selected: number | null
    /** A célula que acabou de receber um número errado, para piscar em vermelho. */
    wrong: number | null
    onSelect: (index: number) => void
  } = $props()
</script>

<div class="board" role="grid" aria-label="Tabuleiro">
  {#each cells as cell, i (i)}
    <button
      class="cell"
      class:given={cell.kind === 'given'}
      class:hinted={cell.kind === 'hinted'}
      class:selected={selected === i}
      class:wrong={wrong === i}
      class:box-right={i % 9 === 2 || i % 9 === 5}
      class:box-bottom={Math.floor(i / 9) === 2 || Math.floor(i / 9) === 5}
      aria-label={`Linha ${Math.floor(i / 9) + 1}, coluna ${(i % 9) + 1}`}
      onclick={() => onSelect(i)}
    >
      {#if cell.value !== null}
        {cell.value}
      {:else if cell.notes.length > 0}
        <span class="notes">
          {#each [1, 2, 3, 4, 5, 6, 7, 8, 9] as n (n)}
            <span>{cell.notes.includes(n) ? n : ''}</span>
          {/each}
        </span>
      {/if}
    </button>
  {/each}
</div>

<style>
  .board {
    display: grid;
    grid-template-columns: repeat(9, 1fr);
    border: 2px solid var(--line-strong);
    aspect-ratio: 1;
    width: 100%;
  }
  .cell {
    min-width: 0;
    min-height: 0;
    border: 1px solid var(--line);
    border-radius: 0;
    font-size: 1.2rem;
    color: var(--accent);
    padding: 0;
  }
  .cell.given {
    color: var(--text);
    font-weight: 700;
  }
  .cell.hinted {
    font-style: italic;
  }
  .cell.selected {
    background: var(--selected);
  }
  .cell.wrong {
    background: var(--wrong);
  }
  .box-right {
    border-right: 2px solid var(--line-strong);
  }
  .box-bottom {
    border-bottom: 2px solid var(--line-strong);
  }
  .notes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    font-size: 0.55rem;
    line-height: 1;
    color: var(--muted);
    width: 100%;
    height: 100%;
    align-items: center;
  }
</style>
