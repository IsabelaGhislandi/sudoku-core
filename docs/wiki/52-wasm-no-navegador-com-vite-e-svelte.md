# 52 — WASM no navegador: `wasm-pack`, `init` assíncrono e runes do Svelte

> Referente à Task 52 do plano do frontend. Commit: `feat: play a game on the board in the browser`.

A Task 51 criou a ponte em Rust. Esta task põe o tabuleiro na tela: um app Svelte 5 em `web/`,
servido pelo Vite, que carrega o `sudoku-wasm` compilado. Por enquanto ele começa direto uma partida
Fácil. Você toca numa célula e depois num número, e um número errado custa uma vida.

## O que o `wasm-pack` gera

```json
"build:wasm": "wasm-pack build ../sudoku-wasm --target web --out-dir ../web/src/lib/wasm-pkg"
```

O `wasm-pack` junta três passos que, sem ele, seriam feitos à mão:

1. `cargo build --release --target wasm32-unknown-unknown`: o `.wasm` cru, com a interface plana
   da wiki 51.
2. `wasm-bindgen`: lê os `#[wasm_bindgen]` e escreve a cola em JavaScript (`sudoku_wasm.js`) e os
   tipos em TypeScript (`sudoku_wasm.d.ts`). A versão da ferramenta precisa ser **exatamente** a da
   biblioteca no `Cargo.lock`, e o `wasm-pack` baixa a certa sozinho.
3. `wasm-opt`: encolhe o binário. O resultado ficou com 86 KB (38 KB com gzip).

O `--target web` gera um módulo ES puro, que o navegador importa sem bundler especial. O pacote vai
para `web/src/lib/wasm-pkg/`, fora do git, porque é gerado. Os scripts `dev` e `build` sempre geram de
novo antes, então o app nunca roda com uma ponte velha.

O `.d.ts` gerado é o que faz o TypeScript conhecer a classe `WebGame`:

```ts
export class WebGame {
  free(): void;
  constructor(difficulty: string, seed: string);
  view(now: number): string;
  place(index: number, value: number, now: number): string;
  resume(now: number): void;
}
```

O `Result<WebGame, String>` do Rust aparece como um construtor comum. O tipo não diz que ele pode
lançar, e esse é um limite do TypeScript, que não tem exceções tipadas. Em Rust, o `Result` obriga
quem chama a tratar o erro. No TS, só a disciplina (e o comentário na ponte) avisa.

## Por que o `init()` é `await`

```ts
// web/src/main.ts
import init from './lib/wasm-pkg/sudoku_wasm'

await init()

const app = mount(App, { target: document.getElementById('app')! })
```

Um módulo JS comum está pronto assim que é importado. Um módulo WASM, não: o navegador precisa
baixar o `.wasm`, compilar para código de máquina e instanciar, com a memória e as funções. O
`init()` faz isso e devolve uma `Promise`. Antes de ela resolver, `new WebGame(...)` falharia, porque a
memória do WASM ainda não existe.

Por isso o `await` vem antes de montar o app. O `await` no topo do arquivo, fora de função, funciona
porque o `main.ts` é um módulo ES (o "top-level await" que o Vite suporta). Em Rust, não existe nada
parecido: um binário já começa com tudo carregado.

## Um `WebGame` é um ponteiro

O objeto `WebGame` que o JS guarda não contém a partida. Ele guarda um número: o endereço da `Game`
dentro da memória linear do WASM, que é um grande `ArrayBuffer`. Cada `game.view(now)` passa esse
endereço para o Rust, que lê a `Game` lá dentro e copia a resposta (uma `String`) de volta para uma
string do JS.

Duas consequências:

- **Memória que o GC não vê.** O coletor de lixo do JS não sabe quanto o `WebGame` ocupa do lado do
  Rust. O método `free()` libera explicitamente. Nas próximas tasks, quando uma partida for trocada
  por outra, a antiga recebe `free()`.
- **Atravessar a ponte custa uma cópia.** Por isso a tela pede **um** retrato inteiro em JSON por
  ação, em vez de 81 chamadas de "me dá a célula i".

## Runes: `$state`, `$derived` e `$props`

O Svelte 5 marca o que é reativo com "runes", funções especiais que o compilador entende:

```svelte
<script lang="ts">
  let { game, onFinished }: { game: WebGame; onExit: () => void; onFinished: () => void } =
    $props()

  let view: GameView = $state(readView())
  let selected: number | null = $state(null)

  const isOver = $derived(view.status !== 'playing')

  function place(value: number) {
    if (selected === null) return
    game.place(selected, value, monotonicNow())
    view = readView() // atribuir redesenha o que depende de `view`
  }
</script>
```

Comparando com React:

| Svelte 5 | React | O que faz |
|---|---|---|
| `let x = $state(0)` | `const [x, setX] = useState(0)` | estado local; no Svelte, `x = 1` já basta |
| `const y = $derived(x * 2)` | `const y = useMemo(() => x * 2, [x])` | valor calculado; o Svelte descobre as dependências sozinho |
| `let { a } = $props()` | `function C({ a })` | as props do componente |

A diferença de fundo: o React roda a função do componente de novo a cada mudança e compara o
resultado. O Svelte compila o componente e sabe, já na compilação, qual pedaço do DOM depende de
`view`. Atribuir `view = ...` atualiza só esse pedaço.

A regra do projeto aparece aqui também: a tela **não calcula regra nenhuma**. Depois de cada
jogada, ela pede o retrato de novo ao WASM. Ela nunca faz algo como `livesLeft -= 1` por conta
própria. Se fizesse, as regras teriam duas versões, uma em Rust e outra em TS, e elas iriam divergir
no PvP.

## Dois relógios no JavaScript

```ts
export function monotonicNow(): number { return performance.now() }
export function wallClockNow(): number { return Date.now() }
```

É a mesma separação da wiki 45, agora do lado da casca. O `performance.now()` é **monotônico**: só
anda para frente, mesmo se alguém mudar a hora do sistema. Ele mede a duração da partida. O
`Date.now()` é o relógio de parede: dá a data do resultado, mas pode pular para trás. O primeiro
recomeça do zero a cada recarga da página, e o `sudoku-game` aceita isso porque uma partida
restaurada volta pausada.

## A medição do tempo de geração

O gerador roda síncrono, na mesma thread que desenha a tela. Antes de gerar, o app mostra
"Gerando..." e cede um quadro (`await tick()` + `requestAnimationFrame`), para o texto aparecer antes
de a thread travar. Medido no V8 (o motor do Chrome), 20 seeds por nível:

| Nível | Mediana | Pior caso |
|---|---|---|
| Fácil | 1 ms | 8 ms |
| Médio | 3 ms | 12 ms |
| Difícil | 21 ms | 196 ms |
| Muito Difícil | 154 ms | 676 ms |

O spec pedia para mover a geração para um Web Worker (outra thread) se a mediana passasse de 1 s. Não
passou, então o código fica simples. O WASM ficou cerca de 30% mais lento que o build nativo da
wiki do Muito Difícil (mediana de 119 ms), o que é o esperado. Num celular, a conta é outra: ela
volta no módulo do Tauri, num aparelho de verdade.
