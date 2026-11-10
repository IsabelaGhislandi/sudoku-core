import { describe, expect, it } from 'vitest'
import { BRANCH_Y, TRUNK_L, createJungle, dropBananas, step } from './sim'
import type { Jungle } from './sim'

/** Sorteio repetível (mulberry32), para a "vida" dos dois ser sempre a mesma no teste. */
function seeded(seed: number): () => number {
  let a = seed
  return () => {
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}

const FRAME = 1 / 30

/** Roda a selva por `seconds` e chama `each` a cada quadro. */
function run(jungle: Jungle, seconds: number, each: () => void = () => {}) {
  for (let t = 0; t < seconds; t += FRAME) {
    step(jungle, FRAME)
    each()
  }
}

describe('jungle', () => {
  it('sozinhos, eles passeiam e cada um fica do seu lado do tronco', () => {
    const jungle = createJungle({ random: seeded(7) })
    const [beni, valentino] = jungle.apes
    const beniXs = new Set<number>()
    const valentinoXs = new Set<number>()

    run(jungle, 120, () => {
      beniXs.add(Math.round(beni.x))
      valentinoXs.add(Math.round(valentino.x))
    })

    expect(beniXs.size).toBeGreaterThan(5)
    expect(valentinoXs.size).toBeGreaterThan(5)
    expect(Math.max(...beniXs)).toBeLessThan(TRUNK_L)
    expect(Math.min(...valentinoXs)).toBeGreaterThan(TRUNK_L - 1)
    expect(Math.max(...valentinoXs)).toBeLessThan(160 - 16)
  })

  it('no chão, cada um corre até a banana que caiu e come', () => {
    const jungle = createJungle({ random: seeded(1) })
    const poses = jungle.apes.map(() => new Set<string>())

    dropBananas(jungle)
    run(jungle, 6, () => jungle.apes.forEach((ape, i) => poses[i]?.add(ape.pose)))

    for (const seen of poses) expect(seen).toContain('eat')
    expect(jungle.bananas).toHaveLength(0)
  })

  it('sentado no galho, ele pega a banana ali mesmo, sem descer', () => {
    const jungle = createJungle({ random: seeded(3) })
    const beni = jungle.apes[0]!
    beni.x = beni.branchX
    beni.y = BRANCH_Y
    const eatingAt: { x: number; y: number }[] = []

    dropBananas(jungle)
    run(jungle, 4, () => {
      if (beni.pose === 'eat') eatingAt.push({ x: beni.x, y: beni.y })
    })

    expect(eatingAt.length).toBeGreaterThan(0)
    for (const spot of eatingAt) expect(spot).toEqual({ x: beni.branchX, y: BRANCH_Y })
  })

  it('com reduzir movimento, ninguém sai do lugar e a banana vai direto para a mão', () => {
    const jungle = createJungle({ random: seeded(5), calm: true })
    const start = jungle.apes.map((ape) => ape.x)

    run(jungle, 30)
    dropBananas(jungle)
    step(jungle, FRAME)

    expect(jungle.apes.map((ape) => ape.x)).toEqual(start)
    for (const ape of jungle.apes) expect(ape.pose).toBe('eat')
  })

  it('depois de comer, a casca voa longe ou cai na cabeça, e depois some', () => {
    const jungle = createJungle({ random: seeded(11) })
    let peelSeen = 0

    dropBananas(jungle)
    run(jungle, 6, () => {
      peelSeen += jungle.peels.length + jungle.apes.filter((ape) => ape.peelOnHead > 0).length
    })
    run(jungle, 5)

    expect(peelSeen).toBeGreaterThan(0)
    expect(jungle.peels).toHaveLength(0)
    for (const ape of jungle.apes) expect(ape.peelOnHead).toBeLessThanOrEqual(0)
  })
})
