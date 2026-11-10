// As telas do app e como se vai de uma para outra. Uma função pura, testável sem navegador: o
// `App.svelte` só aplica o resultado.

/** As telas do app. */
export type Screen = 'home' | 'levels' | 'game' | 'history'

/** O que aconteceu na tela atual e pode mudar de tela. */
export type NavEvent =
  | { type: 'play' }
  | { type: 'continue' }
  | { type: 'openHistory' }
  | { type: 'levelChosen' }
  | { type: 'back' }
  | { type: 'gameOver' }
  | { type: 'home' }

/** A tela que vem depois de `event` na tela `screen`. Evento sem sentido na tela não muda nada. */
export function nextScreen(screen: Screen, event: NavEvent): Screen {
  switch (screen) {
    case 'home':
      if (event.type === 'play') return 'levels'
      if (event.type === 'continue') return 'game'
      if (event.type === 'openHistory') return 'history'
      return screen
    case 'levels':
      if (event.type === 'levelChosen') return 'game'
      if (event.type === 'back') return 'home'
      return screen
    case 'game':
      if (event.type === 'gameOver') return 'history'
      if (event.type === 'back') return 'home'
      return screen
    case 'history':
      if (event.type === 'back' || event.type === 'home') return 'home'
      return screen
  }
}

/** O que fazer no histórico do navegador ao trocar de tela. */
export type HistoryAction = 'push' | 'replace' | 'back' | 'none'

/**
 * Como a troca de `from` para `to` aparece no histórico do navegador. O histórico fica com no
 * máximo uma entrada acima do início, para o voltar do navegador (e, no APK, do Android) sempre
 * levar ao início, e do início sair do site.
 */
export function historyAction(from: Screen, to: Screen): HistoryAction {
  if (from === to) return 'none'
  if (from === 'home') return 'push'
  if (to === 'home') return 'back'
  return 'replace'
}
