// Os desenhos da selva em texto: um caractere por pixel, e a paleta diz a cor de cada letra
// ("." é transparente). Personagens originais, desenhados para este jogo.
import type { ApeName, EmoteKind } from './sim'

export type Sprite = readonly string[]
export type Palette = Readonly<Record<string, string>>

// Orangotangos jovens (18×20): cabelo espetado, rosto escuro (s) com o anel de pele clara (p) em
// volta dos olhos e da boca, olhos grandes com brilho (k + w), bochecha rosada (r), sorriso (m),
// barriga de pele (b) e braços compridos até o chão.
const FACE = [
  '...oOpppssspppOo..',
  '...oOpkwsspkwpOo..',
  '...oOpkksspkkpOo..',
  '...oOrppppppprOo..',
  '....ospmppppmpo...',
  '.....oOpmmmmpOo...',
]

const HAIR: Record<ApeName, string[]> = {
  // Beni: dois espetinhos no topo.
  Beni: [
    '.......h..h.......',
    '......hOhhOh......',
    '.....oOOOOOOo.....',
    '....oOOdOOdOOo....',
    '...oOOssssssOOo...',
  ],
  // Valentino: cabelo jogado para o lado.
  Valentino: [
    '.........hhh......',
    '......oOOhhhh.....',
    '.....oOOOOOOhh....',
    '....oOOdOOOOOo....',
    '...oOOssssssOOo...',
  ],
}

const FRONT_BODY = [
  '..ooOOOOOOOOOOoo..',
  '.oOOdOObbbbOOdOOo.',
  'oOOdOObbbbbbOOdOOo',
  'oOdo.ObbbbbbO.odOo',
  'oOdo.OObbbbOO.odOo',
  'Ohh..oOOOOOOo..hhO',
  'ss...oOO..OOo...ss',
  'ss...dOO..OOd...ss',
  '.....sss..sss.....',
]

const SIT_BODY = [
  '..ooOOOOOOOOOOoo..',
  '.oOOdOObbbbOOdOOo.',
  'oOOdOObbbbbbOOdOOo',
  'oOdoOObbbbbbOOodOo',
  'ssdOOOOOOOOOOOOdss',
  '..oOOOss..ssOOOo..',
]

function sideHead(name: ApeName): string[] {
  const top =
    name === 'Valentino'
      ? ['.........hhh......', '.......oOhhhh.....', '......oOOOOOhh....', '.....oOOOOOOOo....']
      : ['........h.h.......', '.......hOhOh......', '......oOOOOOo.....', '.....oOOOOOOOo....']
  return [
    ...top,
    '.....oOOOsspppo...',
    '.....oOOsspkwp....',
    '.....oOOsspkkpp...',
    '.....oOOOsrpppps..',
    '......oOOssppmp...',
    '.......oOssmms....',
    '......ooOOOOo.....',
    '.....oOOOOOOOo....',
    '....oOdOObbOOOo...',
    '...oOdOObbbOOhOo..',
  ]
}

// Andando em pé, de lado: dois quadros de pernas e braços balançando.
const SIDE_LEGS = [
  [
    '...Od.oObbbOO.hO..',
    '..Od..oOOOOOo..hO.',
    '..ss...oO..Oo...ss',
    '.......oO...Oo....',
    '......dO.....Od...',
    '.....sss.....sss..',
  ],
  [
    '....OdoObbbOOhO...',
    '....OdoOOOOOOhO...',
    '....ss.oOOOOo.ss..',
    '.......oOOOo......',
    '.......dOOOd......',
    '.......ss.sss.....',
  ],
]

// Subindo no tronco, de costas. O segundo quadro é o mesmo espelhado.
function climbFrame(name: ApeName): string[] {
  const hair = name === 'Valentino' ? '..OO.....hhh......' : '..OO...h.h........'
  const empty = '..................'
  return [
    '..ss..............',
    '..OO..............',
    hair,
    '..OO..hOhOh.......',
    '...OooOOOOOo......',
    '...oOOOOOOOOo.....',
    '....oOOdOOOdOo....',
    '....oOOOOOOOOOOss.',
    '....oOOdOOOOOOOss.',
    '....oOOOOOOOOo....',
    '....oOdOOOOdOo....',
    '.....oOOOOOOo.....',
    '.....OOO..OOO.....',
    '.....OO....OO.....',
    '.....OO....OO.....',
    '.....OO....ss.....',
    '.....ss...........',
    empty,
    empty,
    empty,
  ]
}

/** Tira os braços pendurados do desenho de frente, para desenhar braços levantados no lugar. */
function stripArms(rows: string[], left: boolean, right: boolean): string[] {
  return rows.map((row, j) => {
    if (j < 12 || j > 18) return row
    const chars = [...row]
    if (left) chars.fill('.', 0, 4)
    if (right) chars.fill('.', 14, 18)
    return chars.join('')
  })
}

export interface ApeSprites {
  front: Sprite
  /** De frente, sem o braço direito: para acenar e coçar a cabeça. */
  noRight: Sprite
  /** De frente, sem os dois braços: para dançar. */
  noArms: Sprite
  sit: Sprite
  side: [Sprite, Sprite]
  climb: Sprite
}

function apeSprites(name: ApeName): ApeSprites {
  const head = [...HAIR[name], ...FACE]
  const front = [...head, ...FRONT_BODY]
  const blank = '..................'
  const [walkA = [], walkB = []] = SIDE_LEGS
  return {
    front,
    noRight: stripArms(front, false, true),
    noArms: stripArms(front, true, true),
    sit: [blank, blank, blank, ...head, ...SIT_BODY],
    side: [
      [...sideHead(name), ...walkA],
      [...sideHead(name), ...walkB],
    ],
    climb: climbFrame(name),
  }
}

export const APE_SPRITES: Record<ApeName, ApeSprites> = {
  Beni: apeSprites('Beni'),
  Valentino: apeSprites('Valentino'),
}

/** Cores de cada um: o Beni tem o pelo mais claro, o Valentino mais escuro. */
export const APE_PALETTES: Record<ApeName, Palette> = {
  Beni: {
    o: '#4a1806', O: '#c8561e', h: '#f0904a', d: '#8e3412', s: '#5a4440', p: '#e8c0a0',
    k: '#0e0604', w: '#ffffff', m: '#4a2418', b: '#a8786a', r: '#f08a8a',
  },
  Valentino: {
    o: '#3e1404', O: '#a8441a', h: '#d6783a', d: '#742a0e', s: '#4e3a36', p: '#dcb090',
    k: '#0e0604', w: '#ffffff', m: '#4a2418', b: '#9a6c5e', r: '#e88080',
  },
}

export const BANANA: Sprite = ['..y', '.yy', 'yy.', 'Y..']
export const BANANA_PALETTE: Palette = { y: '#ffd83a', Y: '#6a4a10' }

export const PEEL: Sprite = ['y.y.y', 'yyyyy', '.yYy.']
export const PEEL_PALETTE: Palette = { y: '#ffd83a', Y: '#a07a20' }

export const EMOTES: Record<EmoteKind, { sprite: Sprite; palette: Palette }> = {
  heart: { sprite: ['.r.r.', 'rrrrr', 'rrrrr', '.rrr.', '..r..'], palette: { r: '#ff4a6a' } },
  z: { sprite: ['zzz', '..z', '.z.', 'z..', 'zzz'], palette: { z: '#ffffff' } },
  bang: { sprite: ['y', 'y', 'y', '.', 'y'], palette: { y: '#ffd83a' } },
  what: { sprite: ['yyy', '..y', '.yy', '...', '.y.'], palette: { y: '#ffffff' } },
  note: { sprite: ['.nnn', '.n.n', '.n.n', 'nn.n', 'nn..'], palette: { n: '#ffffff' } },
}

/** Fonte própria de 5 pixels de altura, só com as letras dos dois nomes. */
export const GLYPHS: Readonly<Record<string, Sprite>> = {
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
