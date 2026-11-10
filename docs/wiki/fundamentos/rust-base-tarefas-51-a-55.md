# Fundamentos de Rust — consolidado das Tarefas 51 a 55

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 51 a
> 55. As páginas por-task ([51](../51-ponte-wasm-bindgen.md) … [55](../55-guardar-e-continuar.md)) contam a
> história na ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 46 a 50](rust-base-tarefas-46-a-50.md).

Este bloco abre a fase Frontend. Pela primeira vez, o Rust sai dos testes e roda na frente de uma pessoa: compilado
para WebAssembly, chamado por um app Svelte no navegador. O fio que liga as cinco tasks é **a fronteira**: o que
cruza a ponte entre Rust e JavaScript, em que forma, e o que cada lado garante. O Rust traz garantias de compilação
(tipos, posse, `Result`); o JS não tem várias delas, e a ponte decide como cada garantia sobrevive à travessia.

Sumário:
1. [Um crate, dois alvos: `cdylib`, `rlib` e `wasm32-unknown-unknown`](#1-um-crate-dois-alvos-cdylib-rlib-e-wasm32-unknown-unknown)
2. [O que o `#[wasm_bindgen]` gera, e o que custa atravessar](#2-o-que-o-wasm_bindgen-gera-e-o-que-custa-atravessar)
3. [Texto como contrato: seeds, JSON e o limite de 2^53](#3-texto-como-contrato-seeds-json-e-o-limite-de-253)
4. [Como cada tipo do Rust aparece no JavaScript](#4-como-cada-tipo-do-rust-aparece-no-javascript)
5. [Posse através da ponte: ponteiros, `free()` e `self` que consome](#5-posse-através-da-ponte-ponteiros-free-e-self-que-consome)
6. [A casca fornece o mundo: relógios, sorteio e armazenamento](#6-a-casca-fornece-o-mundo-relógios-sorteio-e-armazenamento)
7. [Lógica pura dos dois lados da ponte](#7-lógica-pura-dos-dois-lados-da-ponte)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. Um crate, dois alvos: `cdylib`, `rlib` e `wasm32-unknown-unknown`

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

O mesmo código compila para dois produtos. O `cdylib`, no alvo `wasm32-unknown-unknown`, é o `.wasm` que o
navegador carrega: uma interface plana, sem os metadados do Rust. O `rlib` é a biblioteca Rust comum, que o
`cargo test` usa para rodar os testes no PC, sem navegador ([51](../51-ponte-wasm-bindgen.md)).

O alvo diz muito: `wasm32` (WebAssembly de 32 bits, então `usize` tem 4 bytes lá), `unknown` (nenhum fabricante),
`unknown` (nenhum sistema operacional). Sem sistema, não existe arquivo, relógio nem rede. Isso combina com o que o
`sudoku-core` e o `sudoku-game` já eram por escolha: puros. Eles compilaram para WASM sem mudar uma linha, porque
nunca dependeram de nada que o alvo não tem. A regra "o core recebe dados, não serviços" pagou aqui.

O `wasm-pack` junta o build, a geração da cola (`wasm-bindgen`) e a otimização (`wasm-opt`) num comando só
([52](../52-wasm-no-navegador-com-vite-e-svelte.md)). O resultado ficou com 86 KB.

## 2. O que o `#[wasm_bindgen]` gera, e o que custa atravessar

O WebAssembly só troca números com quem o chama. Strings, objetos e exceções não existem na fronteira. O
`#[wasm_bindgen]` escreve, dos dois lados, a cola que copia uma string do JS para a memória do WASM, chama a função
e copia a resposta de volta ([51](../51-ponte-wasm-bindgen.md)).

Cada travessia custa cópias. Por isso a ponte é **larga e rara**: um retrato inteiro da partida (`GameView`, em
JSON) por ação, em vez de 81 chamadas de "me dá a célula i" ([52](../52-wasm-no-navegador-com-vite-e-svelte.md)).
É o mesmo raciocínio de uma API HTTP: um endpoint que devolve o recurso inteiro, e não um por campo.

O nome também é decidido na ponte: `#[wasm_bindgen(js_name = toggleNote)]` exporta `toggle_note` em `camelCase`,
e o `#[serde(rename_all = "camelCase")]` faz o mesmo com os campos do JSON ([53](../53-anotacoes-dica-e-pausa.md)).
Cada linguagem fica com a sua convenção.

## 3. Texto como contrato: seeds, JSON e o limite de 2^53

O `number` do JavaScript é um float de 64 bits: inteiros exatos só até 2^53 − 1. Um seed `u64` passa disso, e dois
seeds diferentes poderiam virar o mesmo puzzle. A ponte resolve igual à foto e ao resultado (wiki 41): o seed
viaja como **texto em decimal**. O `wasm-bindgen` ofereceria `BigInt`, mas ele espalharia `42n` pelo app e não
passa pelo `JSON.stringify` ([51](../51-ponte-wasm-bindgen.md)).

```rust
pub fn parse_seed(text: &str) -> Result<u64, String> {
    text.parse().map_err(|_| "InvalidSeed".to_string())
}
```

O `randomSeed` do lado do JS fecha o ciclo: `crypto.getRandomValues(new BigUint64Array(1))[0].toString()` sorteia
64 bits de verdade e entrega texto ([52](../52-wasm-no-navegador-com-vite-e-svelte.md)).

Tudo que é mais que um número simples cruza como JSON nos formatos que já existiam: `SavedGame` e `SavedResult`
(com `serde_json`), e o `GameView` novo ([55](../55-guardar-e-continuar.md)). O formato de transporte é um contrato
separado do tipo de domínio. O `Notes(u16)` do Rust é um bitset, e na ponte ele vira `[3, 7]`. A representação
interna pode mudar sem o TypeScript perceber ([53](../53-anotacoes-dica-e-pausa.md)).

## 4. Como cada tipo do Rust aparece no JavaScript

| Rust | JavaScript | O que se perde |
|---|---|---|
| `Result<T, String>` | retorno normal ou `throw` | o tipo não diz que lança |
| `Option<String>` | `string \| undefined` | nada: o `strict` obriga a checar |
| `fn new(...)` com `#[wasm_bindgen(constructor)]` | `new WebGame(...)` | — |
| `fn restore(json: &str)` (sem `self`) | `WebGame.restore(json)`, estático | — |
| `f64` | `number` | — |
| `u64` | `bigint` | por isso o seed vai como texto |
| `usize`, `u8` | `number` | — |

O `Result` é o caso mais delicado ([51](../51-ponte-wasm-bindgen.md)). Em Rust, o compilador obriga a tratar o `Err`.
No TS, as exceções não aparecem no tipo, e só a disciplina avisa. A tela adota uma regra só para isso: uma ação
recusada (o `MoveError`) não muda nada na partida, então a tela engole a exceção e pede o retrato de novo
([53](../53-anotacoes-dica-e-pausa.md)).

O `as` de float para inteiro também é uma decisão de fronteira: ele **satura** (negativo e `NaN` viram `0`). O
`performance.now()` recomeça do zero a cada recarga, e isso não pode virar underflow ([51](../51-ponte-wasm-bindgen.md)).

## 5. Posse através da ponte: ponteiros, `free()` e `self` que consome

O `WebGame` que o JS segura é um **ponteiro** para uma `Game` na memória linear do WASM. O coletor de lixo do JS
não vê essa memória, por isso existe o `free()` ([52](../52-wasm-no-navegador-com-vite-e-svelte.md)).

As três formas de receber `self` sobrevivem à travessia, cada uma do seu jeito:

| Rust | No JS | Quem garante |
|---|---|---|
| `&self` (`view`, `save`) | método comum | — |
| `&mut self` (`place`, `restart`) | método comum | o JS é single-thread: nunca há duas chamadas ao mesmo tempo |
| `self` (`abandon`) | método que zera o ponteiro | uma checagem em tempo de execução |

Depois de `abandon`, chamar qualquer método lança `"null pointer passed to rust"` ([55](../55-guardar-e-continuar.md)).
O que em Rust é erro de compilação, no JS vira exceção. Por isso o `App.svelte` faz `game = null` logo depois: o
que o compilador fazia, ali é feito à mão.

O `restart` mostra a outra escolha: ele troca a partida **dentro** do mesmo `WebGame` (`self.game = restart.game`),
em vez de devolver um objeto novo. O ponteiro que a tela segura continua válido, e não há um segundo objeto para
liberar ([54](../54-telas-e-navegacao.md)).

## 6. A casca fornece o mundo: relógios, sorteio e armazenamento

O core não tem relógio, sorteio nem disco, por contrato. Quem fornece é a casca, e agora ela existe:

- **Dois relógios** ([52](../52-wasm-no-navegador-com-vite-e-svelte.md)): o `performance.now()`, monotônico, mede a
  duração; o `Date.now()`, relógio de parede, dá a data do resultado. É a separação que o `GameResult` já fazia
  (wiki 45).
- **Sorteio**: `crypto.getRandomValues`, a fonte de entropia do navegador.
- **Armazenamento** ([55](../55-guardar-e-continuar.md)): o `localStorage`, síncrono, gravado depois de cada ação. O
  `beforeunload` não é confiável, e no celular o sistema mata abas sem aviso
  ([53](../53-anotacoes-dica-e-pausa.md)).

O armazenamento é o lugar onde o JS precisa compensar o que o Rust daria de graça. O `localStorage` lança em
situações normais (aba anônima, cota cheia, dados bloqueados), e nenhuma dessas pode impedir alguém de jogar. O
módulo promete que nenhuma função dele lança. Em Rust, isso estaria no tipo (devolver `T`, e não `Result<T, E>`);
em TS, fica num comentário e num teste com um `Storage` falso que lança. O `Storage` como parâmetro, em vez do
global, é o mesmo princípio do core: receber o serviço, e não buscá-lo.

## 7. Lógica pura dos dois lados da ponte

A regra "lógica pura de um lado, casca do outro" se repetiu dentro do próprio frontend:

- `nextScreen(tela, evento) → tela` é um redutor puro, testado sem DOM ([54](../54-telas-e-navegacao.md)). O
  `NavEvent` é uma união discriminada, o mais perto que o TS chega de um `enum` com dados, e a exaustividade vem do
  tipo de retorno declarado (o `match` do Rust a dá de graça).
- `formatClock`, `randomSeed` e o módulo de armazenamento ficam em `src/lib/`, testados com Vitest.
- Os componentes `.svelte` só desenham e repassam. Eles nunca calculam vidas, erros ou fim de partida: pedem o
  retrato ao WASM depois de cada ação ([52](../52-wasm-no-navegador-com-vite-e-svelte.md)).

E a divisão de estado ([53](../53-anotacoes-dica-e-pausa.md)): o que muda o resultado ou precisa sobreviver a fechar
a aba mora na `Game` (anotações, pausa, tempo). O que só muda o que se vê agora mora na tela (célula selecionada,
modo anotar, o vermelho piscando).

Os testes nativos do `sudoku-wasm` funcionam porque nenhum método cria um `JsValue`: com a ponte em texto e
números simples, o `#[wasm_bindgen]` deixa as funções como Rust comum fora do alvo WASM
([51](../51-ponte-wasm-bindgen.md)). Uma ponte simples também deixa os testes baratos.

## 8. Tabela-resumo dos conceitos

| Conceito | Onde aparece | Equivalente aproximado em JS/TS |
|---|---|---|
| `crate-type = ["cdylib", "rlib"]` | `sudoku-wasm/Cargo.toml` | pacote com build de browser e de Node |
| `#[wasm_bindgen]` | `WebGame` | a cola que um bundler gera para um módulo nativo |
| `Result<T, String>` na ponte | `place`, `restore` | `throw` |
| `Option<String>` na ponte | `result`, `abandon` | `string \| undefined` |
| `js_name` | `toggleNote` | — |
| `#[serde(rename_all = "camelCase")]` | `GameView` | — |
| Seed como texto | `parse_seed`, `randomSeed` | evitar `number` acima de `Number.MAX_SAFE_INTEGER` |
| `as` que satura | `to_ms` | `Math.max(0, Math.trunc(x)) \|\| 0` |
| Ponteiro + `free()` | `WebGame` | recurso que o GC não vê (`URL.revokeObjectURL`) |
| `self` que consome | `abandon` | ponteiro zerado, checagem em runtime |
| Função associada exportada | `WebGame.restore` | método estático (`Array.from`) |
| `init()` assíncrono | `main.ts` | `await` no topo do módulo |
| Redutor puro | `nextScreen` | `useReducer` |
| União discriminada | `NavEvent` | `enum` com dados do Rust |
| `pushState` / `popstate` | `App.svelte` | o que um router faz por baixo |
| `Storage` como parâmetro | `storage.ts` | injeção de dependência |
| `unknown` no lugar de `any` | `readHistory` | o "confira antes de usar" do Rust |
