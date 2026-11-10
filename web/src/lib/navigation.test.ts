import { describe, expect, it } from 'vitest'
import { nextScreen } from './navigation'

describe('navigation', () => {
  it('vai do início ao jogo pelo nível', () => {
    expect(nextScreen('home', { type: 'play' })).toBe('levels')
    expect(nextScreen('levels', { type: 'levelChosen' })).toBe('game')
    expect(nextScreen('home', { type: 'continue' })).toBe('game')
    expect(nextScreen('home', { type: 'openHistory' })).toBe('history')
  })

  it('volta e termina no histórico', () => {
    expect(nextScreen('levels', { type: 'back' })).toBe('home')
    expect(nextScreen('game', { type: 'back' })).toBe('home')
    expect(nextScreen('history', { type: 'back' })).toBe('home')
    expect(nextScreen('game', { type: 'gameOver' })).toBe('history')
    expect(nextScreen('history', { type: 'home' })).toBe('home')
    expect(nextScreen('levels', { type: 'gameOver' })).toBe('levels')
  })
})
