<script lang="ts">
  import type { CellView } from '../lib/types'

  // O tabuleiro só desenha: quem decide o que é certo ou errado é o WASM.
  let {
    cells,
    selected,
    wrong,
    conflicting,
    highlighted,
    onSelect,
  }: {
    cells: CellView[]
    selected: number | null
    /** A célula que acabou de receber um número errado, para piscar em vermelho. */
    wrong: number | null
    /** As células cujo número impediu uma anotação: piscam algumas vezes. */
    conflicting: number[]
    /** O número em destaque: as células com ele ganham outro fundo. */
    highlighted: number | null
    onSelect: (index: number) => void
  } = $props()
</script>

<div class="board" role="grid" aria-label="Tabuleiro">
  {#each cells as cell, i (i)}
    <button
      class="cell"
      class:given={cell.kind === 'given'}
      class:hinted={cell.kind === 'hinted'}
      class:highlighted={highlighted !== null && cell.value === highlighted}
      class:selected={selected === i}
      class:wrong={wrong === i}
      class:conflict={conflicting.includes(i)}
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
  .cell.highlighted {
    background: var(--highlight);
  }
  .cell.selected {
    background: var(--selected);
  }
  .cell.wrong {
    background: var(--wrong);
  }
  /* Três piscadas em 900 ms, o mesmo tempo que a tela mantém a classe. */
  .cell.conflict {
    animation: blink 300ms steps(1, end) 3;
    color: var(--wrong-text);
  }
  @keyframes blink {
    0% {
      background: var(--wrong);
    }
    50% {
      background: transparent;
    }
  }
  /* Quem pediu menos movimento ao sistema vê o fundo vermelho parado, sem piscar. */
  @media (prefers-reduced-motion: reduce) {
    .cell.conflict {
      animation: none;
      background: var(--wrong);
    }
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
