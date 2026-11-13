# 60 — Selva viva: simulação separada do desenho, sorteio injetado e laço de animação no canvas

> Referente a um pedido feito depois da Task 59 do plano do frontend. Commit: `feat: bring the jungle to life with wandering orangutans`.

A selva da wiki 59 era uma cena parada, em que só a banana se mexia. Agora ela vive o tempo todo.
O Beni e o Valentino, dois orangotangos jovens, passeiam, sobem no tronco, sentam no galho, acenam,
coçam a cabeça, bocejam e dançam. Quando uma linha fecha, eles levam um susto, correm até a banana
(ou pegam ali mesmo, se estiverem na árvore), comem e jogam a casca fora. Às vezes a casca cai na
cabeça. Folhas caem, nuvens passam, um pássaro cruza o céu e tem poeirinha subindo nos raios de sol.

A regra "fechou uma linha?" (`closesLine`) e o contador `drops` da wiki 59 não mudaram. Mudou só
quem recebe o contador.

## Três arquivos, três papéis

```
web/src/lib/jungle/
  sim.ts      o que eles fazem: estado e regras, sem desenho
  sprites.ts  os desenhos, em texto
  draw.ts     pinta um quadro no canvas, mais o vento, as folhas, as nuvens
```

O `sim.ts` não importa nada de canvas nem de DOM, e por isso roda no Vitest, que não tem tela. O
`draw.ts` só lê o estado e pinta: não decide nada. É a mesma divisão que o projeto inteiro já
segue, só que em miniatura. O `sudoku-game` decide e a tela desenha. Aqui, o `sim.ts` decide e o
`draw.ts` desenha.

Em Rust, seria um `struct Jungle` com `fn step(&mut self, dt: f64)` num módulo, e o desenho em
outro, recebendo `&Jungle`. O `&` (só leitura) deixaria explícito, na assinatura, que desenhar não
muda o estado. No TypeScript isso fica só na convenção.

## O sorteio vem de fora

```ts
export function createJungle(options: { random?: () => number; calm?: boolean } = {}): Jungle
```

O que os dois fazem é sorteado: subir na árvore, uma palhaçada ou um passeio. Se o `sim.ts` chamasse
`Math.random()` direto, nenhum teste conseguiria repetir a mesma "vida" deles. Com o sorteio
injetado, o app passa `Math.random` e o teste passa um gerador com seed:

```ts
function seeded(seed: number): () => number { /* mulberry32 */ }

const jungle = createJungle({ random: seeded(7) })
```

É a regra do `sudoku-core`, "o core recebe dados, não serviços", aplicada ao frontend. Lá, a seed
do gerador e o `now` do relógio entram como parâmetros, e aqui entra o sorteio. No Rust seria um
`impl Rng` passado de fora, como o `ChaCha12Rng` do gerador.

## O plano: uma fila de passos com união discriminada

Cada um tem uma fila de passos e executa o primeiro:

```ts
type Step =
  | { type: 'walk'; x: number; speed?: number }
  | { type: 'climb'; y: number }
  | { type: 'wait'; dur: number; pose: 'front' | 'sit'; t: number }
  | { type: 'gag'; kind: Gag; dur: number; emote?: EmoteKind; t: number }
  | { type: 'await'; banana: Banana }
  | { type: 'eat'; t: number }
```

Subir na árvore vira uma sequência simples: andar até o tronco, subir, andar no galho, sentar,
voltar, descer. Quando a banana cai, a fila inteira é trocada por outra. Por isso a banana
"interrompe" qualquer coisa sem nenhum caso especial.

A união discriminada é o `enum` com dados do Rust. O `switch (current.type)` estreita o tipo em
cada `case`: dentro de `case 'climb'`, o TypeScript sabe que existe `current.y`. Em Rust seria um
`match` com `Step::Climb { y } => ...`, e o compilador também reclamaria de um passo esquecido.

## Testar comportamento, não quadros

Os cinco testes rodam a selva por alguns segundos simulados e conferem o resultado:

```ts
it('sentado no galho, ele pega a banana ali mesmo, sem descer', () => {
  // ...
  dropBananas(jungle)
  run(jungle, 4, () => {
    if (beni.pose === 'eat') eatingAt.push({ x: beni.x, y: beni.y })
  })
  for (const spot of eatingAt) expect(spot).toEqual({ x: beni.branchX, y: BRANCH_Y })
})
```

Nenhum teste diz "no quadro 37 ele está em x 52". Isso quebraria a cada ajuste de velocidade. Os
testes dizem o que importa: ele comeu, comeu sem sair do galho, a banana sumiu, a casca sumiu, cada
um ficou do seu lado do tronco.

O vermelho desse teste foi de asserção, como pede o projeto: antes do caso da árvore existir, o Beni
"andava no ar" até a banana, e o teste mostrou `expected { x: 41, y: 20 } to deeply equal { x: 44,
y: 20 }`.

## O laço: `requestAnimationFrame`, `dt` e 30 quadros por segundo

```ts
const loop = (now: number) => {
  frame = requestAnimationFrame(loop)
  if (now - last < FRAME_MS) return
  const dt = Math.min(0.1, (now - last) / 1000)
  last = now
  step(jungle, dt)
  drawJungle(g, background, jungle, air, t)
}
```

Três decisões aqui:

- **`dt` em segundos, não "um passo por quadro".** As velocidades são em pixels por segundo, então
  num celular lento, que desenha menos quadros, eles andam a mesma distância, só que com menos
  quadros no caminho.
- **`Math.min(0.1, ...)`.** Se a aba ficou escondida e voltou, o `dt` seria enorme e eles
  "teletransportariam". O teto evita isso.
- **30 quadros por segundo.** Pixel art não precisa de 60, e desenhar metade das vezes poupa
  bateria. Com a aba escondida, o navegador para o `requestAnimationFrame` sozinho.

O `onMount` devolve uma função que cancela o laço. Sair da tela do jogo para a animação. É o mesmo
papel do retorno de um `useEffect` no React.

## Canvas em vez de SVG

Na wiki 59, a cena era SVG: cada pixel, um `<rect>`. Funcionava porque quase nada se mexia. Agora a
copa balança linha por linha a cada quadro, e isso dá uns 2.000 pixels redesenhados. No SVG, seriam
2.000 elementos no DOM mudando. No canvas, são 2.000 `fillRect` numa imagem só, sem DOM nenhum.

O mundo é uma grade de 160×80, e o CSS estica o canvas até a largura da tela:

```css
canvas { width: 100%; image-rendering: pixelated; }
```

O `image-rendering: pixelated` faz o navegador ampliar sem borrar, e cada pixel vira um quadrado
nítido. O fundo que não se mexe (céu, selva distante, chão, tronco) é desenhado uma vez numa imagem
à parte e colado a cada quadro com um único `drawImage`.

## Menores, sem perder detalhe: duas escalas no mesmo canvas

Na primeira versão, cada pixel do desenho dos orangotangos era um pixel da grade, e eles ficaram
grandes demais na cena. Redesenhar menor jogaria fora o rosto, o anel claro em volta dos olhos e as
bochechas. A saída foi desenhar o canvas três vezes maior que a grade (480×240) e usar dois tamanhos
de pixel:

```ts
export const SCENE_PX = 3                               // o cenário: blocos de 3
const APE_PX = Math.round(SCENE_PX * APE_SCALE)         // os macacos: blocos de 2
```

O `setTransform` do canvas troca a escala antes de cada parte:

```ts
g.setTransform(SCENE_PX, 0, 0, SCENE_PX, 0, 0)          // cenário na grade do mundo
// ...
g.setTransform(APE_PX, 0, 0, APE_PX, toCanvas(x), toCanvas(y)) // macaco na grade do desenho
```

Com isso, o código que desenha um macaco não sabe onde ele está nem de que tamanho ficou: desenha
sempre a partir do (0,0), em pixels do desenho. É o mesmo que um `transform: translate() scale()` no
CSS, aplicado a um grupo de elementos.

A proporção mora numa constante do `sim.ts`, `APE_SCALE = 2 / 3`, porque a simulação também precisa
dela: a altura deles no mundo (`APE_H = 20 * APE_SCALE`) decide onde ficam o chão e o galho, e a mão
que segura a banana fica em `12 * APE_SCALE`. Mudar o tamanho deles de novo é mudar essa constante,
e o resto acompanha.

## O valor inicial de propósito: `untrack`

```ts
let seen = untrack(() => drops)
$effect(() => {
  if (drops > seen) dropBananas(jungle)
  seen = drops
})
```

O `svelte-check` avisou que `let seen = drops` "só captura o valor inicial". Aqui é exatamente o que
se quer: o valor de quando a tela abre é a linha de base, e só um número **maior** derruba banana.
O `untrack` diz isso ao Svelte em código, e o aviso some sem esconder um erro de verdade.

## "Reduzir movimento"

Com `prefers-reduced-motion` ligado no sistema, o `calm` vira `true`. Ninguém passeia, o vento e as
folhas param, e a banana aparece direto na mão. A regra está no `sim.ts`, e por isso tem teste.
