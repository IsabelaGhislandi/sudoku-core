# 53 — Anotações, dica e pausa: um bitset visto do JS, eventos da página e estado só da tela

> Referente à Task 53 do plano do frontend. Commit: `feat: add notes, hints and pause to the board`.

Esta task liga na tela três ferramentas que o `sudoku-game` já tinha: anotar, pedir dica e pausar.
Do lado do Rust, são três métodos curtos no `WebGame`. O que vale estudar é onde fica cada pedaço
de estado e como um tipo do Rust aparece do outro lado da ponte.

## `Notes`: um bitset que vira `number[]`

No `sudoku-game`, as anotações de uma célula são um `Notes(u16)`, um bitset: o bit 3 ligado quer
dizer "o 3 está anotado". Ele é compacto e barato de copiar (wiki 30). Só que o JavaScript não
deveria precisar saber disso. Na ponte, o retrato converte para a forma que a tela usa:

```rust
// sudoku-wasm/src/view.rs
CellView {
    kind: ...,
    value: state.value(),
    notes: game.notes(index).values(), // Vec<u8>, em ordem crescente
}
```

No JSON, sai `"notes": [3, 7]`. A tela só pergunta `cell.notes.includes(n)`. A representação
interna do Rust pode mudar (para um `u16` com outro layout, por exemplo) sem que o TypeScript
perceba. É o mesmo princípio de um DTO (wiki 41): o formato de transporte é um contrato à parte do
tipo de domínio.

O teste confere o contrato pelo lado de fora, lendo o JSON:

```rust
game.toggle_note(index, 3)?;
game.toggle_note(index, 7)?;
assert_eq!(view_json(&game, 0.0)["cells"][index]["notes"], serde_json::json!([3, 7]));
```

O `json!` do `serde_json` monta um `Value` com a sintaxe do próprio JSON, e a comparação vira uma
comparação de árvores. Em JS, seria um `expect(view.cells[i].notes).toEqual([3, 7])`.

## `js_name`: o nome de cada lado

```rust
#[wasm_bindgen(js_name = toggleNote)]
pub fn toggle_note(&mut self, index: usize, value: u8) -> Result<(), String> { ... }
```

Cada linguagem segue a sua convenção: `snake_case` no Rust, `camelCase` no JS. O `js_name` escolhe
o nome exportado sem mudar o nome em Rust. No TS, a chamada fica `game.toggleNote(i, 3)`, como
qualquer método JavaScript. O `Result<(), String>` vira um método que devolve `undefined` ou lança.

## Estado da partida × estado da tela

O modo "Anotar" **não** está na `Game`:

```svelte
// Modo anotar é estado só da tela: a partida não sabe dele, só recebe `place` ou `toggleNote`.
let noting = $state(false)

function onNumber(value: number) {
  ...
  if (noting) game.toggleNote(index, value)
  else if (game.place(index, value, monotonicNow()) === 'wrong') flashWrong(index)
}
```

A regra para decidir onde um estado mora:

- **Na `Game` (Rust):** o que muda o resultado da partida ou precisa sobreviver a fechar a aba.
  Vidas, anotações, tempo, se está pausada.
- **Na tela (Svelte):** o que só muda o que se vê agora. A célula selecionada, o modo anotar, o
  vermelho piscando por 600 ms.

Se o modo anotar estivesse na `Game`, a foto da partida (`SavedGame`) teria de guardá-lo e o PvP
teria de sincronizá-lo, sem motivo nenhum. Em React, é a mesma distinção entre estado do servidor
(ou de uma store global) e estado local de componente.

## Uma ação, um retrato novo

As três ferramentas passam pelo mesmo caminho:

```ts
function act(action: () => void) {
  try {
    action()
  } catch {
    // recusada: nada a fazer
  }
  refresh()
}
```

Uma dica pedida sem dicas, ou uma jogada numa célula ocupada, volta como exceção (o `Err` do Rust).
Como uma ação recusada não muda nada na partida (é o contrato do `MoveError`, wiki 32), a tela pode
engolir a exceção e simplesmente desenhar de novo. O retrato novo é a verdade, tenha a ação sido
aceita ou não.

O botão "Dica" ainda fica desabilitado quando `hintsLeft === 0`, mas por **conveniência**, não por
regra. Quem decide se há dica é o Rust. O botão desabilitado só evita um clique inútil.

## `visibilitychange` no lugar de `beforeunload`

```ts
function onVisibilityChange() {
  if (document.visibilityState === 'hidden') act(() => game.pause(monotonicNow()))
}
document.addEventListener('visibilitychange', onVisibilityChange)
onDestroy(() => document.removeEventListener('visibilitychange', onVisibilityChange))
```

Trocar de aba, minimizar ou bloquear o celular dispara `visibilitychange` com `hidden`. Esse é o
evento confiável para "a pessoa saiu". O `beforeunload` (fechar a aba) não dispara em vários casos.
No celular, o sistema pode matar uma aba em segundo plano sem aviso nenhum. Por isso a pausa entra
aqui, e na Task 55 o salvamento acontece depois de cada ação, sem esperar o fechamento.

O `removeEventListener` no `onDestroy` é o equivalente ao retorno de limpeza de um `useEffect`.
Sem ele, um ouvinte de uma partida que já saiu da tela continuaria pausando um `WebGame` antigo.

## Pausado, o tabuleiro some

```svelte
{#if view.paused && !isOver}
  <button class="paused" onclick={togglePause}>Pausado — toque para continuar</button>
{:else}
  <Board cells={view.cells} ... />
{/if}
```

Com o relógio parado, a pessoa poderia olhar o tabuleiro e pensar à vontade, e o tempo final não
contaria esse raciocínio. Esconder o tabuleiro é o que os apps de Sudoku costumam fazer. O `{#if}` do
Svelte remove o componente do DOM: não é só esconder com CSS, e o tabuleiro nem existe na página
enquanto está pausado.
