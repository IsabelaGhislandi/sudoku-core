<script lang="ts" module>
  // A cena é desenhada em "pixels" de verdade: cada retângulo do SVG é um quadradinho de 1×1 (ou
  // uma faixa) numa grade de 112×60, ampliada sem suavizar. Tudo é calculado uma vez, quando o
  // módulo carrega: a cena não muda, só as bananas e as bocas se mexem.
  interface Px {
    x: number
    y: number
    w: number
    h: number
    c: string
  }

  const WIDTH = 112
  const HEIGHT = 60

  /** Converte um desenho em texto (um caractere por pixel) em retângulos, pela paleta. */
  function sprite(rows: string[], palette: Record<string, string>, ox: number, oy: number): Px[] {
    const out: Px[] = []
    rows.forEach((row, y) =>
      [...row].forEach((ch, x) => {
        const c = palette[ch]
        if (c) out.push({ x: ox + x, y: oy + y, w: 1, h: 1, c })
      }),
    )
    return out
  }

  function background(): Px[] {
    const out: Px[] = []
    const add = (x: number, y: number, w: number, h: number, c: string) => out.push({ x, y, w, h, c })
    // Céu em faixas, do mais claro para o mais escuro.
    ;['#9fe0d0', '#86d4c0', '#6cc6b0', '#55b49c'].forEach((c, i) => add(0, i * 8, WIDTH, 8, c))
    // Folhagem ao fundo, em duas camadas de alturas "aleatórias" fixas.
    for (let x = 0; x < WIDTH; x += 7) {
      const h = 10 + ((x * 7) % 9)
      add(x, 30 - h, 8, h + 20, '#2d7a5a')
    }
    for (let x = 3; x < WIDTH; x += 9) {
      const h = 6 + ((x * 5) % 7)
      add(x, 38 - h, 7, h + 14, '#236448')
    }
    // Raios de luz atravessando a copa.
    for (let y = 0; y < 40; y++) {
      add(20 + Math.floor(y / 2), y, 2, 1, 'rgba(255,255,220,.25)')
      add(80 + Math.floor(y / 3), y, 2, 1, 'rgba(255,255,220,.2)')
    }
    // A árvore das bananas.
    add(51, 12, 10, 40, '#5a3418')
    add(53, 12, 3, 40, '#7a4a24')
    add(59, 12, 2, 40, '#3e2210')
    add(34, 3, 44, 11, '#2f9a4a')
    add(38, 0, 36, 4, '#46b85e')
    add(30, 8, 52, 5, '#46b85e')
    add(40, 12, 32, 3, '#1f7a38')
    // Cipós.
    for (const [x, h] of [
      [26, 26],
      [86, 22],
    ]) {
      for (let y = 10; y < h + 10; y++) add(x + (y % 8 < 4 ? 0 : 1), y, 1, 1, '#1f6a30')
      add(x - 1, h + 9, 3, 2, '#46b85e')
    }
    // Chão: grama e terra.
    add(0, 50, WIDTH, 2, '#6ccf4a')
    add(0, 52, WIDTH, 8, '#6b4220')
    for (let x = 0; x < WIDTH; x += 5) add(x, 54 + (x % 3), 2, 1, '#8a5a2c')
    for (let x = 2; x < WIDTH; x += 9) add(x, 48, 1, 2, '#6ccf4a')
    return out
  }

  // Orangotango: cabeça larga, rosto claro, braços compridos até o chão. A cabeça (5 primeiras
  // linhas) fica parada; o resto, da boca para baixo, é a parte que mastiga.
  const APE = [
    '...oooooo...',
    '..oOOOOOOo..',
    '.oOffffffOo.',
    '.oOfkffkfOo.',
    '.oOffffffOo.',
    '..oOfmmfOo..',
    '.ooOOOOOOoo.',
    'oOOOOOOOOOOo',
    'oOo.OOOO.oOo',
    'oOo.OOOO.oOo',
    'oOo.OOOO.oOo',
    'ff..OO.OO.ff',
    '....oo.oo...',
  ]
  const APE_PALETTE = { o: '#6a2808', O: '#c4561a', f: '#f0b88a', k: '#1a0a00', m: '#4a1604' }
  const APE_TOP = 37
  const HEAD_ROWS = 5

  const BANANA = ['..y', '.yy', 'yy.', 'Y..']
  const BANANA_PALETTE = { y: '#ffd83a', Y: '#6a4a10' }
  const BANANA_TOP = 10

  // Fonte própria de 5 pixels de altura, só com as letras dos dois nomes.
  const GLYPHS: Record<string, string[]> = {
    A: ['010', '101', '111', '101', '101'],
    B: ['110', '101', '110', '101', '110'],
    E: ['111', '100', '110', '100', '111'],
    I: ['111', '010', '010', '010', '111'],
    L: ['100', '100', '100', '100', '111'],
    N: ['1001', '1101', '1011', '1001', '1001'],
    O: ['010', '101', '101', '101', '010'],
    T: ['111', '010', '010', '010', '010'],
    V: ['101', '101', '101', '101', '010'],
  }

  /** O nome em pixels, centrado em `cx`. */
  function label(text: string, cx: number, y: number): Px[] {
    const glyphs = [...text].map((ch) => GLYPHS[ch] ?? [])
    const width = glyphs.reduce((sum, g) => sum + (g[0]?.length ?? 0), 0) + glyphs.length - 1
    let x = cx - Math.floor(width / 2)
    const out: Px[] = []
    for (const g of glyphs) {
      out.push(...sprite(g, { '1': '#fff3c4' }, x, y))
      x += (g[0]?.length ?? 0) + 1
    }
    return out
  }

  /** Um dos dois: onde ele fica e onde a banana dele cai. */
  function ape(name: string, x: number) {
    return {
      name,
      head: sprite(APE.slice(0, HEAD_ROWS), APE_PALETTE, x, APE_TOP),
      body: sprite(APE.slice(HEAD_ROWS), APE_PALETTE, x, APE_TOP + HEAD_ROWS),
      banana: sprite(BANANA, BANANA_PALETTE, x + 5, BANANA_TOP),
      label: label(name.toUpperCase(), x + 6, 54),
    }
  }

  const SCENE = background()
  const APES = [ape('Beni', 31), ape('Valentino', 69)]
</script>

<script lang="ts">
  // `drops` conta quantas vezes uma linha ou coluna fechou. Cada número novo derruba uma banana
  // para cada um: o `{#key}` recria as bananas e as animações recomeçam do zero. Começa em 0, sem
  // banana nenhuma: abrir uma partida salva não derruba nada.
  let { drops }: { drops: number } = $props()
</script>

{#snippet pixels(list: Px[])}
  {#each list as p, i (i)}
    <rect x={p.x} y={p.y} width={p.w} height={p.h} fill={p.c} />
  {/each}
{/snippet}

<div class="jungle" role="img" aria-label="Selva com os orangotangos Beni e Valentino">
  <svg viewBox="0 0 {WIDTH} {HEIGHT}" shape-rendering="crispEdges">
    {@render pixels(SCENE)}
    {#key drops}
      {#each APES as a (a.name)}
        <g class:munching={drops > 0}>
          {@render pixels(a.head)}
          <g class="jaw">{@render pixels(a.body)}</g>
        </g>
        {#if drops > 0}
          <g class="banana">{@render pixels(a.banana)}</g>
        {/if}
      {/each}
    {/key}
    {#each APES as a (a.name)}
      {@render pixels(a.label)}
    {/each}
  </svg>
</div>

<style>
  .jungle {
    border-radius: 6px;
    overflow: hidden;
  }
  svg {
    display: block;
    width: 100%;
    height: auto;
  }
  /* A queda anda em degraus (`steps`), como num jogo antigo: nada de movimento suave. */
  .banana {
    animation: fall 1.4s steps(8) forwards;
  }
  .munching .jaw {
    animation: munch 1.4s steps(1) forwards;
  }
  @keyframes fall {
    0% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(30px);
      opacity: 1;
    }
    51%,
    100% {
      transform: translateY(30px);
      opacity: 0;
    }
  }
  @keyframes munch {
    0%,
    50% {
      transform: translateY(0);
    }
    58%,
    74%,
    90% {
      transform: translateY(1px);
    }
    66%,
    82%,
    100% {
      transform: translateY(0);
    }
  }
  /* Com "reduzir movimento" ligado, a banana não cai: aparece já com eles e some. */
  @media (prefers-reduced-motion: reduce) {
    .banana {
      animation: appear 1s steps(1) forwards;
    }
    .munching .jaw {
      animation: none;
    }
    @keyframes appear {
      0% {
        transform: translateY(30px);
        opacity: 1;
      }
      100% {
        transform: translateY(30px);
        opacity: 0;
      }
    }
  }
</style>
