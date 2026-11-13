# 59 — Selva e bananas: comparar antes e depois, `{#key}` para reiniciar animação e pixel art com SVG

> Referente a um pedido feito depois da Task 58 do plano do frontend. Commit: `feat: drop bananas for the orangutans when a line completes`.

Embaixo do teclado de números agora tem uma selva em pixel art, com uma árvore e dois orangotangos,
o **Beni** e o **Valentino**. Toda jogada que completa uma linha ou uma coluna derruba uma banana
da árvore para cada um, e eles comem. Não tem placar: a banana cai, é comida e pronto.

Como na wiki 58, é estado só da tela. A partida em Rust não sabe de selva nenhuma, e o
`sudoku-game` e o `sudoku-wasm` não mudaram uma linha.

## "Fechou uma linha?" é uma comparação entre dois retratos

A tela não recebe um evento "linha completa" do WASM. O que ela tem é o retrato do tabuleiro antes
e depois de cada ação. Então a pergunta vira uma função pura:

```ts
export function closesLine(before: CellView[], after: CellView[]): boolean {
  return completeLines(after) > completeLines(before)
}
```

`completeLines` conta quantas linhas e colunas estão cheias. Como número errado não fica no
tabuleiro (ele custa uma vida e sai), cheia quer dizer certa: não é preciso conferir a solução.

O `>` resolve, de uma vez, três casos que pareciam exigir código especial:

- **Uma jogada que fecha linha e coluna ao mesmo tempo** sobe a contagem em 2, mas é um `true` só.
  Cai uma banana para cada um, não duas.
- **Recomeçar** esvazia o tabuleiro: a contagem desce, e `>` dá `false`.
- **Abrir uma partida salva** não passa por `closesLine` nenhuma: o primeiro retrato já é o
  "antes" da próxima jogada.

Por isso o terceiro teste foi escrito depois de a função já estar pronta, e passou de primeira.
Para provar que ele pega o erro, o `>` foi trocado de propósito por `!==` e por `>=`: nos dois
casos o teste ficou vermelho, e o `>` voltou.

Em Rust, a mesma ideia seria um `fn closes_line(before: &[CellView], after: &[CellView]) -> bool`.
Os dois `&[...]` deixam claro, já pela assinatura, que a função só lê, e é isso que o TypeScript
não consegue dizer: um `CellView[]` poderia ser modificado lá dentro.

## Quem chama: guardar o "antes" numa variável local

```ts
function refresh() {
  const before = view.cells
  view = readView()
  if (closesLine(before, view.cells)) drops++
}
```

`readView()` devolve um objeto novo, montado a partir do JSON. O `before` continua apontando para o
array antigo, que ninguém mais modifica. No JS isso funciona porque arrays são passados por
referência e o retrato antigo nunca é alterado no lugar. Em Rust seria o contrário: o compilador
exigiria um `clone()` (ou um `std::mem::replace`) para manter o antigo vivo enquanto o novo é
atribuído, porque `view` não pode ser reescrita enquanto `before` empresta dela.

Toda ação passa por `refresh`, inclusive a dica. Então uma dica que fecha uma linha também derruba
banana, sem nenhuma linha a mais de código.

## `drops` é um contador, e `{#key}` reinicia a animação

A cena recebe um número, não um booleano:

```svelte
<Jungle {drops} />
```

```svelte
{#key drops}
  {#each APES as a (a.name)}
    ...
    {#if drops > 0}
      <g class="banana">{@render pixels(a.banana)}</g>
    {/if}
  {/each}
{/key}
```

Uma animação CSS só começa quando o elemento entra na página. Se a tela só ligasse uma classe
`falling`, a segunda linha fechada não animaria nada: a classe já estaria lá. O `{#key drops}` diz
ao Svelte que destrua e recrie o bloco sempre que `drops` mudar. Os elementos novos nascem com a
animação no começo, e a banana cai de novo.

No React, o equivalente é trocar a `key` de um componente (`<Bananas key={drops} />`): mudar a
`key` faz o React desmontar e montar de novo.

`drops` começa em 0, e com 0 não tem banana: abrir a tela não derruba nada.

## Pixel art com retângulos de SVG

Nenhuma imagem: cada personagem é um desenho em texto, um caractere por pixel, convertido em
retângulos de 1×1.

```ts
const APE = [
  '...oooooo...',
  '..oOOOOOOo..',
  '.oOffffffOo.',
  '.oOfkffkfOo.',
  // ...
]
const APE_PALETTE = { o: '#6a2808', O: '#c4561a', f: '#f0b88a', k: '#1a0a00', m: '#4a1604' }
```

O SVG usa um `viewBox` de 112×60 e é esticado até a largura da tela. O
`shape-rendering="crispEdges"` impede o navegador de suavizar as bordas, e os pixels continuam
quadrados em qualquer tamanho. Os nomes usam uma fonte própria de 5 pixels de altura, com só as
letras de BENI e VALENTINO.

Tudo isso fica no `<script module>`, que roda **uma vez**, quando o arquivo é importado, e não a
cada vez que o componente aparece. A cena não muda, então calcular os retângulos de novo a cada
partida seria trabalho jogado fora. É o mesmo papel de uma constante no topo de um módulo JS, ou de
um `const`/`static` em Rust.

As cinco listas de retângulos são desenhadas por um único snippet do Svelte 5:

```svelte
{#snippet pixels(list: Px[])}
  {#each list as p, i (i)}
    <rect x={p.x} y={p.y} width={p.w} height={p.h} fill={p.c} />
  {/each}
{/snippet}
```

Um snippet é um pedaço de template com parâmetros. Funciona como um componente pequeno, mas sem
arquivo próprio e sem props. Tem a mesma função de uma função que devolve JSX no React.

## A queda em degraus e o "reduzir movimento"

```css
.banana { animation: fall 1.4s steps(8) forwards; }
```

`steps(8)` faz a banana pular de posição em 8 degraus, em vez de deslizar: é o movimento de jogo
antigo, que combina com os pixels. O `forwards` deixa a animação parada no último quadro (a banana
já comida, invisível).

Quem liga "reduzir movimento" no sistema vê a banana aparecer direto com os orangotangos, sem
queda, pela media query `prefers-reduced-motion`.

## Custa alguma coisa no servidor?

Não. O site é estático, e o servidor só entrega arquivos. A cena roda inteira no navegador, e o
que cresce é o pacote JS em alguns KB, porque o desenho é código e não imagem. No celular, a
animação só roda quando uma banana cai, e mexe em poucos retângulos.
