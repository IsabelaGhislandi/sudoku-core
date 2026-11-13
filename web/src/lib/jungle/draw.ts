// Desenha a selva num canvas de 160×80 "pixels", ampliado sem suavizar pelo CSS. O fundo parado é
// desenhado uma vez numa imagem à parte; a cada quadro só o que se mexe é redesenhado por cima.
import {
  APE_SCALE,
  BRANCH,
  BRANCH_Y,
  GROUND,
  HEIGHT,
  TRUNK_L,
  TRUNK_R,
  WIDTH,
  type Ape,
  type Jungle,
} from './sim'
import {
  APE_PALETTES,
  APE_SPRITES,
  BANANA,
  BANANA_PALETTE,
  EMOTES,
  GLYPHS,
  PEEL,
  PEEL_PALETTE,
  type Palette,
  type Sprite,
} from './sprites'

type Ctx = CanvasRenderingContext2D

function rect(g: Ctx, x: number, y: number, w: number, h: number, color: string | undefined) {
  if (!color) return
  g.fillStyle = color
  g.fillRect(Math.round(x), Math.round(y), w, h)
}

function sprite(g: Ctx, rows: Sprite, palette: Palette, x: number, y: number, flip = false) {
  const left = Math.round(x)
  const top = Math.round(y)
  const width = rows[0]?.length ?? 0
  rows.forEach((row, j) => {
    for (let i = 0; i < row.length; i++) {
      const color = palette[row[i] ?? '.']
      if (!color) continue
      g.fillStyle = color
      g.fillRect(flip ? left + width - 1 - i : left + i, top + j, 1, 1)
    }
  })
}

// ---------- o ar: nuvens, folhas, poeirinha, pássaro e borboleta ----------

interface Air {
  clouds: { x: number; y: number; w: number }[]
  leaves: { x: number; y: number; vx: number; phase: number; color: string }[]
  motes: { x: number; y: number; phase: number }[]
  bird: { x: number; y: number; dir: number } | null
  nextBird: number
  butterfly: number
}

export function createAir(random: () => number): Air {
  const between = (a: number, b: number) => a + random() * (b - a)
  return {
    clouds: [
      { x: 10, y: 5, w: 18 },
      { x: 95, y: 11, w: 14 },
      { x: 150, y: 3, w: 22 },
    ],
    leaves: [],
    motes: Array.from({ length: 14 }, () => ({ x: between(20, 140), y: between(10, 56), phase: between(0, 6) })),
    bird: null,
    nextBird: 3,
    butterfly: 0,
  }
}

export function stepAir(air: Air, dt: number, random: () => number): void {
  const between = (a: number, b: number) => a + random() * (b - a)
  for (const c of air.clouds) {
    c.x += dt * (c.w > 16 ? 1.5 : 2.4)
    if (c.x > WIDTH + 4) c.x = -c.w - 4
  }
  if (random() < dt * 1.2 && air.leaves.length < 10) {
    air.leaves.push({
      x: between(40, 120),
      y: between(8, 22),
      vx: between(-2, 2),
      phase: between(0, 6),
      color: random() < 0.3 ? '#e8c840' : '#5cc46a',
    })
  }
  for (const leaf of air.leaves) {
    leaf.y += dt * 6
    leaf.phase += dt * 2.5
    leaf.x += Math.sin(leaf.phase) * dt * 6 + leaf.vx * dt
  }
  air.leaves = air.leaves.filter((leaf) => leaf.y <= GROUND)
  for (const m of air.motes) {
    m.phase += dt
    m.y -= dt * 1.2
    m.x += Math.sin(m.phase) * dt * 1.5
    if (m.y < 6) {
      m.y = between(40, 58)
      m.x = between(20, 140)
    }
  }
  air.nextBird -= dt
  if (!air.bird && air.nextBird <= 0) {
    const dir = random() < 0.5 ? 1 : -1
    air.bird = { x: dir > 0 ? -4 : WIDTH + 4, y: between(4, 16), dir }
  }
  if (air.bird) {
    air.bird.x += air.bird.dir * 22 * dt
    if (air.bird.x < -6 || air.bird.x > WIDTH + 6) {
      air.bird = null
      air.nextBird = between(5, 10)
    }
  }
  air.butterfly += dt
}

// ---------- o fundo parado ----------

export function createBackground(): HTMLCanvasElement {
  const canvas = document.createElement('canvas')
  canvas.width = WIDTH
  canvas.height = HEIGHT
  const g = canvas.getContext('2d')
  if (!g) return canvas
  const p = (x: number, y: number, w: number, h: number, c: string) => rect(g, x, y, w, h, c)
  ;['#b7ecdc', '#a2e3d1', '#8cd8c3', '#76cbb4', '#62bca3'].forEach((c, i) => p(0, i * 9, WIDTH, 9, c))
  // A selva do fundo, em três camadas de alturas "aleatórias" fixas.
  for (let x = 0; x < WIDTH; x += 6) {
    const h = 12 + ((x * 7) % 11)
    p(x, 36 - h, 7, h + 30, '#3a8a66')
    p(x + 2, 36 - h, 2, 1, '#4ea07a')
  }
  for (let x = 3; x < WIDTH; x += 8) {
    const h = 8 + ((x * 5) % 9)
    p(x, 46 - h, 7, h + 20, '#2a6d50')
  }
  for (let x = 0; x < WIDTH; x += 11) {
    const h = 4 + ((x * 3) % 6)
    p(x, 52 - h, 9, h + 10, '#1f5a40')
  }
  // Chão.
  p(0, GROUND, WIDTH, 3, '#6ccf4a')
  p(0, GROUND + 3, WIDTH, HEIGHT - GROUND - 3, '#6b4220')
  for (let x = 0; x < WIDTH; x += 5) p(x, GROUND + 6 + (x % 4), 2, 1, '#8a5a2c')
  for (let x = 3; x < WIDTH; x += 7) p(x, GROUND + 12 + (x % 3), 1, 1, '#4a2c14')
  for (const [x, c] of [
    [20, '#9a9a9a'],
    [132, '#8a8a8a'],
  ] as const) {
    p(x, GROUND - 2, 5, 2, c)
    p(x + 1, GROUND - 3, 3, 1, c)
  }
  for (const [x, c] of [
    [12, '#ff6b9a'],
    [40, '#ffd83a'],
    [118, '#ff6b9a'],
    [146, '#b98cff'],
  ] as const) {
    p(x, GROUND - 2, 1, 2, '#2f8a3a')
    p(x - 1, GROUND - 3, 3, 1, c)
  }
  // Tronco e galhos.
  p(TRUNK_L, 16, TRUNK_R - TRUNK_L, GROUND - 16, '#5a3418')
  p(TRUNK_L + 3, 16, 3, GROUND - 16, '#7a4a24')
  p(TRUNK_R - 3, 16, 2, GROUND - 16, '#3e2210')
  for (let y = 20; y < GROUND; y += 7) p(TRUNK_L + 6 + (y % 3), y, 4, 1, '#3e2210')
  p(TRUNK_L - 3, GROUND - 2, TRUNK_R - TRUNK_L + 6, 2, '#5a3418')
  p(44, BRANCH, TRUNK_L - 44, 3, '#5a3418')
  p(44, BRANCH, TRUNK_L - 44, 1, '#7a4a24')
  p(TRUNK_R, BRANCH, 116 - TRUNK_R, 3, '#5a3418')
  p(TRUNK_R, BRANCH, 116 - TRUNK_R, 1, '#7a4a24')
  return canvas
}

// ---------- o que se mexe ----------

function drawCanopy(g: Ctx, t: number) {
  for (let r = 0; r < 26; r++) {
    const k = (r - 13) / 13.5
    const half = Math.round(44 * Math.sqrt(Math.max(0, 1 - k * k)) + Math.sin(r * 1.7) * 2)
    // As linhas de cima balançam mais: é o vento na copa.
    const sway = Math.round(Math.sin(t * 1.3 + r * 0.35) * (r < 10 ? 1.4 : 0.7))
    for (let i = -half; i <= half; i++) {
      let c = r < 5 ? '#5cc46a' : r > 20 ? '#1f6a34' : '#3a9f4f'
      if (((i + 50) * 7 + r * 3) % 11 === 0) c = '#2a7a3c'
      if (r < 14 && ((i + 50) * 5 + r * 11) % 13 === 0) c = '#8be08a'
      rect(g, 80 + i + sway, r - 2, 1, 1, c)
    }
  }
  for (const [x, y] of [
    [66, 23],
    [92, 23],
    [80, 22],
  ] as const) {
    rect(g, x, y, 1, 1, '#3a2a10')
    rect(g, x - 1, y + 1, 3, 2, '#ffd83a')
    rect(g, x, y + 3, 1, 1, '#e8b820')
  }
}

function drawVines(g: Ctx, t: number) {
  for (const [x0, len] of [
    [46, 30],
    [114, 26],
    [56, 18],
    [104, 20],
  ] as const) {
    for (let y = 18; y < 18 + len; y++) {
      const x = x0 + Math.round(Math.sin(t * 1.4 + y * 0.18 + x0) * 2 * ((y - 18) / len))
      rect(g, x, y, 1, 1, '#1f6a30')
      if (y % 5 === 0) rect(g, x + (y % 10 === 0 ? 1 : -1), y, 1, 1, '#46b85e')
    }
  }
}

function drawAir(g: Ctx, air: Air, t: number) {
  for (const c of air.clouds) {
    const col = 'rgba(255,255,255,.85)'
    rect(g, c.x, c.y + 2, c.w, 3, col)
    rect(g, c.x + 3, c.y, c.w - 8, 2, col)
    rect(g, c.x + 6, c.y - 1, c.w - 14, 1, col)
  }
  const a = 0.1 + 0.05 * Math.sin(t * 0.8)
  for (let y = 0; y < GROUND; y++) {
    rect(g, 24 + y * 0.45, y, 3, 1, `rgba(255,255,220,${a})`)
    rect(g, 122 + y * 0.3, y, 2, 1, `rgba(255,255,220,${a * 0.8})`)
  }
  for (const m of air.motes) if (Math.sin(m.phase * 3) > -0.2) rect(g, m.x, m.y, 1, 1, 'rgba(255,255,230,.8)')
  if (air.bird) {
    const { x, y } = air.bird
    if (Math.floor(t * 8) % 2) {
      rect(g, x, y, 1, 1, '#1a2a2a')
      rect(g, x + 1, y + 1, 1, 1, '#1a2a2a')
      rect(g, x + 2, y, 1, 1, '#1a2a2a')
    } else rect(g, x, y + 1, 3, 1, '#1a2a2a')
  }
}

function drawGrassAndButterfly(g: Ctx, air: Air, t: number) {
  for (let x = 1; x < WIDTH; x += 3) {
    const tip = Math.round(Math.sin(t * 2.2 + x * 0.45) * 0.7)
    rect(g, x, GROUND - 1, 1, 1, '#4fb83a')
    rect(g, x + tip, GROUND - 2, 1, 1, '#6ccf4a')
  }
  const p = air.butterfly
  const bx = 34 + Math.sin(p * 0.7) * 22
  const by = 52 + Math.sin(p * 1.9) * 4
  const open = Math.floor(p * 10) % 2
  rect(g, bx, by, 1, 1, '#3a1a3a')
  rect(g, bx - 1, by - open, 1, 1, '#ff8ac8')
  rect(g, bx + 1, by - open, 1, 1, '#ff8ac8')
  for (const leaf of air.leaves) rect(g, leaf.x, leaf.y, 1, 1, leaf.color)
}

// ---------- os dois ----------

function raisedArm(g: Ctx, ape: Ape, y: number, side: -1 | 1, lift: number) {
  const pal = APE_PALETTES[ape.name]
  const c = side > 0 ? 16 : 0
  for (let r = 4; r <= 11; r++) {
    rect(g, c, y + r, 1, 1, (side > 0 ? pal.O : pal.o))
    rect(g, c + 1, y + r, 1, 1, (side > 0 ? pal.o : pal.O))
  }
  rect(g, c + (side > 0 ? 0 : 1), y + 3, 1, 1, pal.h)
  rect(g, c, y + 2 - lift, 2, 2, pal.s)
}

function scratchArm(g: Ctx, ape: Ape, y: number, dy: number) {
  const pal = APE_PALETTES[ape.name]
  for (let r = 3; r <= 11; r++) rect(g, 16, y + r, 1, 1, pal.O)
  rect(g, 15, y + 2, 2, 1, pal.O)
  rect(g, 12, y + dy, 3, 2, pal.s)
}

function happyEyes(g: Ctx, ape: Ape, y: number) {
  const pal = APE_PALETTES[ape.name]
  rect(g, 6, y + 6, 2, 2, pal.p)
  rect(g, 11, y + 6, 2, 2, pal.p)
  for (const [dx, dy] of [
    [6, 7],
    [7, 6],
    [12, 7],
    [11, 6],
  ] as const)
    rect(g, dx, y + dy, 1, 1, pal.k)
}

function closedEyes(g: Ctx, ape: Ape, y: number) {
  const pal = APE_PALETTES[ape.name]
  rect(g, 6, y + 6, 2, 2, pal.p)
  rect(g, 11, y + 6, 2, 2, pal.p)
  rect(g, 6, y + 7, 2, 1, pal.k)
  rect(g, 11, y + 7, 2, 1, pal.k)
}

function openMouth(g: Ctx, ape: Ape, y: number, big: boolean) {
  const pal = APE_PALETTES[ape.name]
  rect(g, 7, y + 9, 6, 2, pal.p)
  rect(g, (big ? 8 : 9), y + 9, big ? 4 : 2, 2, pal.m)
  if (big) rect(g, 9, y + 10, 2, 1, '#e85a6a') // a linguinha
}

/**
 * Desenha um deles na própria grade (0,0 é o canto do desenho, 1 = um pixel do desenho) e devolve
 * quanto ele subiu (dança, passo), para a etiqueta e o emote irem junto.
 */
function drawApe(g: Ctx, ape: Ape, t: number): number {
  const pal = APE_PALETTES[ape.name]
  const s = APE_SPRITES[ape.name]
  let y = 0
  const beat = Math.floor(ape.anim * 4) % 2
  const frame = Math.floor(ape.anim * 5) % 2
  switch (ape.pose) {
    case 'walk':
      sprite(g, frame ? s.side[1] : s.side[0], pal, 0, y - frame, ape.face < 0)
      break
    case 'climb':
      sprite(g, s.climb, pal, 0, y, frame === 1)
      break
    case 'climbStill':
      sprite(g, s.climb, pal, 0, y)
      break
    case 'sit':
      sprite(g, s.sit, pal, 0, y + (ape.y === BRANCH_Y ? 1 : 0))
      break
    case 'lookUp':
      sprite(g, s.front, pal, 0, y)
      rect(g, 7, y + 6, 1, 1, pal.w)
      rect(g, 12, y + 6, 1, 1, pal.w)
      openMouth(g, ape, y, false)
      break
    case 'eat': {
      sprite(g, s.front, pal, 0, y)
      if (Math.floor(ape.anim * 8) % 2) openMouth(g, ape, y, false)
      else happyEyes(g, ape, y)
      const left = Math.ceil(ape.eatLeft * 4)
      if (left > 0) sprite(g, BANANA.slice(4 - left), BANANA_PALETTE, 11, y + 10 + (4 - left))
      break
    }
    case 'wave':
      sprite(g, s.noRight, pal, 0, y)
      raisedArm(g, ape, y, 1, beat)
      happyEyes(g, ape, y)
      break
    case 'scratch':
      sprite(g, s.noRight, pal, 0, y)
      scratchArm(g, ape, y, Math.floor(ape.anim * 10) % 2)
      break
    case 'yawn':
      sprite(g, s.front, pal, 0, y)
      closedEyes(g, ape, y)
      if (ape.anim % 2.2 > 0.3) openMouth(g, ape, y, true)
      break
    case 'dance':
      y -= beat * 2
      sprite(g, s.noArms, pal, 0, y)
      raisedArm(g, ape, y, 1, beat)
      raisedArm(g, ape, y, -1, 1 - beat)
      happyEyes(g, ape, y)
      break
    case 'front':
      sprite(g, s.front, pal, 0, y)
      // pisca de vez em quando, cada um no seu tempo
      if ((t + (ape.side < 0 ? 0 : 1.7)) % 4 < 0.15) closedEyes(g, ape, y)
      break
  }
  if (ape.peelOnHead > 0) sprite(g, PEEL, PEEL_PALETTE, 6, y - 2)
  return y
}

/** Etiqueta simples com o nome, em cima da cabeça (na grade do desenho, presa à borda da cena). */
function drawLabel(g: Ctx, ape: Ape, dy: number) {
  const glyphs = [...ape.name.toUpperCase()].map((ch) => GLYPHS[ch] ?? [])
  const width = glyphs.reduce((n, q) => n + (q[0]?.length ?? 0), 0) + glyphs.length - 1
  const boxW = width + 4
  const left = Math.min(
    Math.max(toCanvas(ape.x) + (9 - boxW / 2) * APE_PX, APE_PX),
    CANVAS_W - (boxW + 1) * APE_PX,
  )
  const top = Math.max(toCanvas(ape.y) + (dy - (ape.peelOnHead > 0 ? 12 : 9)) * APE_PX, APE_PX)
  g.setTransform(APE_PX, 0, 0, APE_PX, Math.round(left), Math.round(top))
  rect(g, 0, 0, boxW, 7, 'rgba(30,16,6,.72)')
  let cx = 2
  for (const q of glyphs) {
    sprite(g, q, { 1: '#fff3c4' }, cx, 1)
    cx += (q[0]?.length ?? 0) + 1
  }
}

function drawEmote(g: Ctx, ape: Ape, dy: number) {
  if (!ape.emote) return
  const { sprite: rows, palette } = EMOTES[ape.emote.kind]
  onApeGrid(g, ape.x, ape.y)
  sprite(g, rows, palette, ape.side < 0 ? 17 : -5, dy + 2 - Math.round(ape.emote.t * 4))
}

// A cena é desenhada num canvas SCENE_PX vezes maior que a grade do mundo (160×80). O cenário usa
// pixels de SCENE_PX; os orangotangos, a banana e as etiquetas usam pixels menores, de APE_PX.
// Assim eles ficam menores (APE_PX / SCENE_PX do tamanho) sem perder nenhum detalhe do desenho.
export const SCENE_PX = 3
const APE_PX = Math.round(SCENE_PX * APE_SCALE)
export const CANVAS_W = WIDTH * SCENE_PX
export const CANVAS_H = HEIGHT * SCENE_PX

const toCanvas = (world: number) => Math.round(world * SCENE_PX)

/** Passa a desenhar na grade do desenho, com o canto em (x, y) do mundo. */
function onApeGrid(g: Ctx, x: number, y: number) {
  g.setTransform(APE_PX, 0, 0, APE_PX, toCanvas(x), toCanvas(y))
}

/** Um quadro inteiro da cena. */
export function drawJungle(g: Ctx, background: HTMLCanvasElement, jungle: Jungle, air: Air, t: number) {
  g.imageSmoothingEnabled = false
  g.setTransform(SCENE_PX, 0, 0, SCENE_PX, 0, 0)
  g.drawImage(background, 0, 0)
  drawAir(g, air, t)
  drawVines(g, t)
  drawCanopy(g, t)
  const lifts = jungle.apes.map((ape) => {
    onApeGrid(g, ape.x, ape.y)
    return drawApe(g, ape, t)
  })
  for (const banana of jungle.bananas) {
    onApeGrid(g, banana.x, banana.y)
    sprite(g, BANANA, BANANA_PALETTE, 0, 0)
  }
  g.setTransform(SCENE_PX, 0, 0, SCENE_PX, 0, 0)
  drawGrassAndButterfly(g, air, t)
  for (const peel of jungle.peels) {
    onApeGrid(g, peel.x, peel.y)
    sprite(g, PEEL, PEEL_PALETTE, 0, 0)
  }
  jungle.apes.forEach((ape, i) => {
    const dy = lifts[i] ?? 0
    drawLabel(g, ape, dy)
    drawEmote(g, ape, dy)
  })
  g.setTransform(1, 0, 0, 1, 0, 0)
}
