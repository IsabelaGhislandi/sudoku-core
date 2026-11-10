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
