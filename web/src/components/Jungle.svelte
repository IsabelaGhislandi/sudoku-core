<script lang="ts">
  import { onMount, untrack } from 'svelte'
  import {
    CANVAS_H,
    CANVAS_W,
    createAir,
    createBackground,
    drawJungle,
    stepAir,
  } from '../lib/jungle/draw'
  import { createJungle, dropBananas, step } from '../lib/jungle/sim'

  // `drops` conta quantas vezes uma jogada fechou linha ou coluna. Cada número novo derruba uma
  // banana para cada um. O valor de quando a tela abre não conta: abrir uma partida salva não
  // derruba nada.
  let { drops }: { drops: number } = $props()

  let canvas: HTMLCanvasElement
  // "Reduzir movimento" ligado: a cena fica parada e a banana aparece direto na mão.
  const calm = window.matchMedia('(prefers-reduced-motion: reduce)').matches
  const jungle = createJungle({ calm })
  const air = createAir(Math.random)

  // De propósito, só o valor inicial: é a linha de base para saber quando chegou um novo.
  let seen = untrack(() => drops)
  $effect(() => {
    if (drops > seen) dropBananas(jungle)
    seen = drops
  })

  // ~30 quadros por segundo bastam para pixel art e poupam bateria. Com a aba escondida, o
  // navegador para o `requestAnimationFrame` sozinho.
  const FRAME_MS = 1000 / 30

  onMount(() => {
    const g = canvas.getContext('2d')
    if (!g) return
    const background = createBackground()
    let last = performance.now()
    let t = 0
    let frame = 0
    const loop = (now: number) => {
      frame = requestAnimationFrame(loop)
      if (now - last < FRAME_MS) return
      const dt = Math.min(0.1, (now - last) / 1000)
      last = now
      if (!calm) {
        t += dt
        stepAir(air, dt, Math.random)
      }
      step(jungle, dt)
      drawJungle(g, background, jungle, air, t)
    }
    frame = requestAnimationFrame(loop)
    return () => cancelAnimationFrame(frame)
  })
</script>

<div class="jungle" role="img" aria-label="Selva com os orangotangos Beni e Valentino">
  <canvas bind:this={canvas} width={CANVAS_W} height={CANVAS_H} aria-hidden="true"></canvas>
</div>

<style>
  /* A moldura de madeira em volta da cena. */
  .jungle {
    border-radius: 6px;
    overflow: hidden;
    box-shadow:
      0 0 0 3px #3b2414,
      0 0 0 5px #1a0f08;
  }
  canvas {
    display: block;
    width: 100%;
    height: auto;
    image-rendering: pixelated;
  }
</style>
