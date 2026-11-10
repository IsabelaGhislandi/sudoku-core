// O que o Beni e o Valentino fazem, quadro a quadro: passear, subir na árvore, palhaçadas e
// buscar a banana. Só estado e regras, sem desenho: quem desenha é o `draw.ts`. O sorteio vem de
// fora (`random`), para os testes poderem repetir a mesma "vida" dos dois.

export const WIDTH = 160
export const HEIGHT = 80
export const GROUND = 60
/**
 * Um pixel do desenho dos macacos, em unidades do mundo. Os desenhos têm 18×20 pixels; com 2/3,
 * eles ocupam 12×13⅓ do mundo, menores que um pixel do cenário mas com todo o detalhe.
 */
export const APE_SCALE = 2 / 3
export const APE_W = 18 * APE_SCALE
export const APE_H = 20 * APE_SCALE
/** O `y` do topo de um macaco em pé no chão. */
export const GROUND_Y = GROUND - APE_H
export const BRANCH = 40
/** O `y` do topo de um macaco sentado no galho. */
export const BRANCH_Y = BRANCH - APE_H
export const TRUNK_L = 72
export const TRUNK_R = 88

export type ApeName = 'Beni' | 'Valentino'
export type Gag = 'wave' | 'scratch' | 'yawn' | 'dance'
export type Pose = 'front' | 'walk' | 'climb' | 'climbStill' | 'sit' | 'lookUp' | 'eat' | Gag
export type EmoteKind = 'heart' | 'z' | 'bang' | 'what' | 'note'

export interface Banana {
  x: number
  y: number
  targetY: number
  vy: number
  landed: boolean
}

export interface Peel {
  x: number
  y: number
  vx: number
  vy: number
  life: number
}

type Step =
  | { type: 'walk'; x: number; speed?: number }
  | { type: 'climb'; y: number }
  | { type: 'wait'; dur: number; pose: 'front' | 'sit'; t: number }
  | { type: 'gag'; kind: Gag; dur: number; emote?: EmoteKind; t: number }
  | { type: 'await'; banana: Banana }
  | { type: 'eat'; t: number }

export interface Ape {
  name: ApeName
  x: number
  y: number
  /** -1: o lado esquerdo do tronco (Beni); 1: o direito (Valentino). */
  side: -1 | 1
  face: -1 | 1
  pose: Pose
  /** Tempo na animação atual, para escolher o quadro. */
  anim: number
  plan: Step[]
  emote: { kind: EmoteKind; t: number } | null
  peelOnHead: number
  /** Quanto da banana ainda falta comer, de 1 a 0. */
  eatLeft: number
  trunkX: number
  branchX: number
  range: [number, number]
}

export interface Jungle {
  apes: Ape[]
  bananas: Banana[]
  peels: Peel[]
  /** "Reduzir movimento": ninguém passeia e a banana aparece direto na mão. */
  calm: boolean
  random: () => number
}

function makeApe(name: ApeName, x: number, side: -1 | 1): Ape {
  return {
    name,
    x,
    y: GROUND_Y,
    side,
    face: side < 0 ? 1 : -1,
    pose: 'front',
    anim: 0,
    plan: [],
    emote: null,
    peelOnHead: 0,
    eatLeft: 0,
    trunkX: side < 0 ? TRUNK_L - 2 : TRUNK_R - APE_W + 2,
    branchX: side < 0 ? 48 : 100,
    range: side < 0 ? [2, 54] : [98, 144],
  }
}

export function createJungle(options: { random?: () => number; calm?: boolean } = {}): Jungle {
  return {
    apes: [makeApe('Beni', 30, -1), makeApe('Valentino', 120, 1)],
    bananas: [],
    peels: [],
    calm: options.calm ?? false,
    random: options.random ?? Math.random,
  }
}

const WALK_SPEED = 11
const CLIMB_SPEED = 8
const EMOTE_S = 1.4
/** Onde fica a mão que segura a banana, no desenho (pixel 11–12, linha 10), em unidades do mundo. */
const HAND_X = 12 * APE_SCALE
const HAND_Y = 10 * APE_SCALE
const EAT_S = 1.8
const PEEL_S = 2.5
/** Em pixels por segundo², para a banana e a casca. */
const GRAVITY = 120

const GAGS: { kind: Gag; dur: number; emote?: EmoteKind }[] = [
  { kind: 'wave', dur: 2.2 },
  { kind: 'scratch', dur: 1.8, emote: 'what' },
  { kind: 'yawn', dur: 2.2, emote: 'z' },
  { kind: 'dance', dur: 2.4, emote: 'note' },
]

function between(random: () => number, min: number, max: number): number {
  return min + random() * (max - min)
}

/** Escolhe o que fazer agora: subir na árvore, uma palhaçada ou um passeio. */
function newPlan(ape: Ape, random: () => number): Step[] {
  const roll = random()
  if (roll < 0.3) {
    return [
      { type: 'walk', x: ape.trunkX },
      { type: 'climb', y: BRANCH_Y },
      { type: 'walk', x: ape.branchX, speed: 7 },
      { type: 'wait', dur: between(random, 3, 6), pose: 'sit', t: 0 },
      { type: 'walk', x: ape.trunkX, speed: 7 },
      { type: 'climb', y: GROUND_Y },
      { type: 'wait', dur: 0.6, pose: 'front', t: 0 },
    ]
  }
  if (roll < 0.65) {
    const gag = GAGS[Math.floor(random() * GAGS.length)] ?? GAGS[0]
    return [
      { type: 'gag', ...gag, t: 0 },
      { type: 'wait', dur: between(random, 0.5, 1.5), pose: 'front', t: 0 },
    ]
  }
  return [
    { type: 'walk', x: between(random, ape.range[0], ape.range[1]) },
    { type: 'wait', dur: between(random, 1, 3), pose: random() < 0.3 ? 'sit' : 'front', t: 0 },
  ]
}

/** Com "reduzir movimento", ele só fica parado de frente (por um minuto, e de novo). */
const standStill = (): Step[] => [{ type: 'wait', dur: 60, pose: 'front', t: 0 }]

function stepApe(jungle: Jungle, ape: Ape, dt: number): void {
  if (ape.plan.length === 0) {
    ape.plan = jungle.calm ? standStill() : newPlan(ape, jungle.random)
  }
  const current = ape.plan[0]
  if (!current) return
  ape.anim += dt
  if (ape.emote) {
    ape.emote.t += dt
    if (ape.emote.t > EMOTE_S) ape.emote = null
  }
  switch (current.type) {
    case 'walk': {
      const speed = current.speed ?? WALK_SPEED
      const d = current.x - ape.x
      if (d !== 0) ape.face = d > 0 ? 1 : -1
      ape.pose = 'walk'
      if (Math.abs(d) <= speed * dt) {
        ape.x = current.x
        ape.plan.shift()
      } else ape.x += Math.sign(d) * speed * dt
      break
    }
    case 'climb': {
      const d = current.y - ape.y
      ape.pose = 'climb'
      if (Math.abs(d) <= CLIMB_SPEED * dt) {
        ape.y = current.y
        ape.plan.shift()
      } else ape.y += Math.sign(d) * CLIMB_SPEED * dt
      break
    }
    case 'wait':
      ape.pose = current.pose
      current.t += dt
      if (current.t >= current.dur) ape.plan.shift()
      break
    case 'gag':
      if (current.t === 0 && current.emote) ape.emote = { kind: current.emote, t: 0 }
      ape.pose = current.kind
      current.t += dt
      if (current.t >= current.dur) ape.plan.shift()
      break
    case 'await':
      ape.pose = isClimbing(ape) ? 'climbStill' : 'lookUp'
      if (current.banana.landed) {
        jungle.bananas.splice(jungle.bananas.indexOf(current.banana), 1)
        ape.plan.shift()
      }
      break
    case 'eat':
      ape.pose = isClimbing(ape) ? 'climbStill' : 'eat'
      current.t += dt
      ape.eatLeft = 1 - current.t / EAT_S
      if (current.t >= EAT_S) {
        ape.plan.shift()
        ape.emote = { kind: 'heart', t: 0 }
        throwPeel(jungle, ape)
      }
      break
  }
}

/** A casca voa para longe... ou, às vezes, cai na cabeça dele e fica lá um tempinho. */
function throwPeel(jungle: Jungle, ape: Ape): void {
  if (isOnGround(ape) && jungle.random() < 0.4) {
    ape.peelOnHead = PEEL_S
    return
  }
  jungle.peels.push({
    x: ape.x + HAND_X,
    y: ape.y + HAND_Y,
    vx: ape.side * between(jungle.random, 18, 30),
    vy: -38,
    life: PEEL_S,
  })
}

function stepPeels(jungle: Jungle, dt: number): void {
  for (const peel of jungle.peels) {
    peel.vy += GRAVITY * dt
    peel.x += peel.vx * dt
    peel.y = Math.min(peel.y + peel.vy * dt, GROUND - 3)
    if (peel.y >= GROUND - 3) peel.vx *= 0.9 // no chão, vai parando
    peel.life -= dt
  }
  jungle.peels = jungle.peels.filter((peel) => peel.life > 0)
  for (const ape of jungle.apes) if (ape.peelOnHead > 0) ape.peelOnHead -= dt
}

/** Avança a selva `dt` segundos. */
export function step(jungle: Jungle, dt: number): void {
  for (const banana of jungle.bananas) {
    if (banana.landed) continue
    banana.vy += GRAVITY * dt
    banana.y += banana.vy * dt
    if (banana.y >= banana.targetY) {
      banana.y = banana.targetY
      banana.landed = true
    }
  }
  stepPeels(jungle, dt)
  for (const ape of jungle.apes) stepApe(jungle, ape, dt)
}

/** Uma linha fechou: cai uma banana para cada um. */
export function dropBananas(jungle: Jungle): void {
  for (const ape of jungle.apes) {
    if (jungle.calm) {
      // Sem queda nem corrida: a banana já aparece na mão.
      ape.plan = [{ type: 'eat', t: 0 }]
      continue
    }
    ape.emote = { kind: 'bang', t: 0 }
    const banana = isOnGround(ape) ? bananaToFetch(ape) : bananaToCatch(ape)
    jungle.bananas.push(banana)
  }
}

function isOnGround(ape: Ape): boolean {
  return ape.y === GROUND_Y
}

/** No meio do tronco: nem no chão, nem no galho. */
function isClimbing(ape: Ape): boolean {
  return !isOnGround(ape) && ape.y !== BRANCH_Y
}

/** No chão: a banana cai da copa, no ponto mais perto dele, e ele corre buscar. */
function bananaToFetch(ape: Ape): Banana {
  const landX = Math.min(116, Math.max(44, ape.x + APE_W / 2))
  const banana: Banana = { x: landX - 1, y: 22, targetY: GROUND - 4, vy: 0, landed: false }
  ape.plan = [
    { type: 'walk', x: landX - HAND_X, speed: 30 },
    { type: 'await', banana },
    { type: 'eat', t: 0 },
    { type: 'gag', kind: 'dance', dur: 1.6, t: 0 },
  ]
  return banana
}

/** Na árvore: a banana cai bem em cima dele, que pega ali mesmo e depois desce. */
function bananaToCatch(ape: Ape): Banana {
  const banana: Banana = {
    x: ape.x + HAND_X,
    y: Math.max(ape.y - 8, 2),
    targetY: ape.y + HAND_Y,
    vy: 0,
    landed: false,
  }
  const down: Step[] =
    ape.y === BRANCH_Y
      ? [
          { type: 'wait', dur: 1, pose: 'sit', t: 0 },
          { type: 'walk', x: ape.trunkX, speed: 7 },
          { type: 'climb', y: GROUND_Y },
        ]
      : [{ type: 'climb', y: GROUND_Y }]
  ape.plan = [{ type: 'await', banana }, { type: 'eat', t: 0 }, ...down]
  return banana
}
