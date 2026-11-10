import { describe, expect, it } from 'vitest'
import { historyAction, nextScreen } from './navigation'

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

  it('mantém o histórico do navegador a uma entrada do início', () => {
    expect(historyAction('home', 'levels')).toBe('push')
    expect(historyAction('home', 'history')).toBe('push')
    expect(historyAction('levels', 'game')).toBe('replace')
    expect(historyAction('game', 'history')).toBe('replace')
    expect(historyAction('game', 'home')).toBe('back')
    expect(historyAction('history', 'home')).toBe('back')
    expect(historyAction('home', 'home')).toBe('none')
  })
})
