# 54 — Telas como máquina de estados: união discriminada no TS, `enum` no Rust e o histórico do navegador

> Referente à Task 54 do plano do frontend. Commit: `feat: add home, level and game screens with navigation`.

O app ganha o caminho que o jogo pede: Início → Escolher nível → Jogo → Histórico. Em vez de
espalhar "se clicou aqui, mostra aquilo" pelos componentes, as trocas de tela moram numa função
pura, `nextScreen`, testada com Vitest. O `App.svelte` só aplica o resultado.

## União discriminada: o `enum` com dados do TypeScript

```ts
export type Screen = 'home' | 'levels' | 'game' | 'history'

export type NavEvent =
  | { type: 'play' }
  | { type: 'continue' }
  | { type: 'openHistory' }
  | { type: 'levelChosen' }
  | { type: 'back' }
  | { type: 'gameOver' }
  | { type: 'home' }
```

O `NavEvent` é uma **união discriminada**: cada variante é um objeto com um campo `type` diferente,
e o TypeScript usa esse campo para saber qual variante está na mão. É o mais perto que o TS chega de
um `enum` do Rust com dados. A mesma ideia, em Rust, seria:

```rust
enum NavEvent {
    Play,
    Continue,
    OpenHistory,
    LevelChosen,
    Back,
    GameOver,
    Home,
}
```

As diferenças aparecem quando uma variante carrega dados. Em Rust, `PlaceOutcome::Wrong { index,
value }` (wiki 36) é um tipo só, e o compilador garante que os campos existem. Em TS, a união é
de objetos soltos com uma "etiqueta" combinada. Funciona igual na prática, mas a etiqueta é uma
string que você escreve à mão em todo lugar.

## Exaustividade: `match` × `switch`

```ts
export function nextScreen(screen: Screen, event: NavEvent): Screen {
  switch (screen) {
    case 'home':
      if (event.type === 'play') return 'levels'
      if (event.type === 'continue') return 'game'
      if (event.type === 'openHistory') return 'history'
      return screen
    case 'levels': ...
    case 'game': ...
    case 'history': ...
  }
}
```

O `match` do Rust é exaustivo por regra da linguagem: esquecer uma variante é erro de compilação.
O `switch` do TS não é. Aqui, ele fica exaustivo por um efeito colateral: a função declara que
devolve `Screen`, e com o `strict` ligado o TS reclama ("Function lacks ending return statement")
se algum `case` de `Screen` ficar de fora, porque o fim da função devolveria `undefined`. Uma quinta
tela que alguém acrescente ao tipo quebra a compilação, como no Rust.

Quando isso não basta, o truque comum em TS é um `default` com `never`:

```ts
default: {
  const unreachable: never = screen // erro de tipo se sobrar alguma tela
  return unreachable
}
```

Evento sem sentido na tela (`gameOver` estando na escolha de nível) devolve a mesma tela. O teste
trava isso:

```ts
expect(nextScreen('levels', { type: 'gameOver' })).toBe('levels')
```

## Por que uma função pura

A navegação podia morar dentro dos componentes, e cada botão faria `screen = 'levels'`. Funcionaria,
mas o caminho entre as telas ficaria espalhado e só daria para testá-lo abrindo o navegador.

Como função pura, `(tela, evento) → tela`, ela é um **redutor**, o mesmo padrão do `useReducer` do
React ou do Redux. Testar é chamar e comparar, sem DOM nenhum. É a mesma escolha do `sudoku-core`:
lógica pura de um lado, a casca (aqui, o `App.svelte`) do outro.

## O histórico do navegador: `pushState` e `popstate`

Um app de uma página só tem um problema: para o navegador, ele é uma página só. O botão voltar
sairia do site em vez de voltar uma tela. O spec pede o contrário, e no APK o botão voltar do
Android vai ser o principal jeito de sair de uma tela.

```ts
history.replaceState({ screen: 'home' }, '')

function navigate(event: NavEvent) {
  const next = nextScreen(screen, event)
  if (next === screen) return
  screen = next
  history.pushState({ screen }, '') // uma entrada nova no histórico do navegador
}

window.addEventListener('popstate', () => {
  screen = nextScreen(screen, { type: 'back' })
})
```

- O `pushState` acrescenta uma entrada no histórico **sem recarregar a página**. Para o navegador,
  agora existe um "voltar" possível.
- O `popstate` dispara quando a pessoa aperta voltar. O app trata como o evento `back` da própria
  máquina de estados. Assim, o voltar do navegador e a seta `←` da tela fazem a mesma coisa.

Frameworks de rotas, como o SvelteKit e o React Router, fazem isso com URLs (`/jogo`, `/historico`).
Aqui não há URL por tela: uma partida não faz sentido aberta direto por link, porque ela depende do
que está guardado no navegador. Quatro telas e um `pushState` resolvem sem dependência nova.

## Recomeçar: a ponte devolve o abandono

```rust
pub fn restart(&mut self, seed: &str, now: f64, finished_at_ms: f64)
    -> Result<Option<String>, String>
{
    let seed = convert::parse_seed(seed)?;
    let restart = self.game.restart(seed, convert::to_ms(now), convert::to_ms(finished_at_ms));
    self.game = restart.game;
    Ok(restart.abandoned.map(|result| save_result(&result)))
}
```

O `Game::restart` recebe `&self` e devolve um valor novo (wiki 44): a partida nova e, se a antiga
estava em andamento, o resultado do abandono. A ponte troca a partida **dentro** do mesmo `WebGame`
(`self.game = restart.game`), então o objeto que a tela segura continua válido, e não há um segundo
ponteiro para liberar com `free()`.

O tipo de volta empilha dois "talvez": `Result` (a seed pode ser inválida) por fora e `Option` (pode
não haver abandono) por dentro. No JS, isso vira: lança, devolve `undefined`, ou devolve o JSON do
`SavedResult`. A tela pede confirmação antes, porque o abandono vai para o histórico e não tem volta:

```ts
if (!isOver && !confirm('Largar esta partida e começar outra?')) return
```

O `confirm` nativo é feio, mas o design vem depois, e ele bloqueia do jeito certo: nada acontece
até a pessoa responder.
