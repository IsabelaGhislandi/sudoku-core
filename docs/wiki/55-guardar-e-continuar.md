# 55 — Guardar e continuar: `serde_json` na ponte, `self` que consome visto do JS e `try/catch` no armazenamento

> Referente à Task 55 do plano do frontend. Commit: `feat: save the game in progress and continue it later`.

Fechar a aba no meio de uma partida não perde mais nada. A tela inicial ganha um botão **Continuar**,
que traz a partida de volta, pausada, com o tempo de onde parou. O `sudoku-game` já sabia tirar a
"foto" da partida (`SavedGame`, wiki 41) e registrar o resultado (`SavedResult`, wiki 48). Esta task
liga esses formatos ao `localStorage` do navegador, passando pela ponte.

## A foto atravessa a ponte como JSON

```rust
pub fn save(&self, now: f64) -> String {
    serde_json::to_string(&self.game.save(convert::to_ms(now)))
        .expect("o SavedGame só tem campos simples")
}

pub fn restore(json: &str) -> Result<WebGame, String> {
    let saved: SavedGame =
        serde_json::from_str(json).map_err(|_| "InvalidJson".to_string())?;
    let game = Game::restore(&saved).map_err(convert::error_name)?;
    Ok(WebGame { game })
}
```

Os dois lados têm papéis diferentes:

- **`save` não falha.** Transformar um `SavedGame` (só strings, números e listas) em JSON sempre dá
  certo. O `expect` documenta por quê, conforme a regra do projeto: `panic` só para invariante
  interna, com o motivo escrito.
- **`restore` falha de dois jeitos.** O texto pode nem ser JSON (`"InvalidJson"`), ou pode ser JSON
  de uma foto que não fecha: versão desconhecida, tabuleiro que não bate com a solução... Esses vêm
  do `RestoreError` do `sudoku-game` (wiki 42). A ponte só dá nome ao erro, sem conferir nada por
  conta própria.

O `restore` não tem `&self`: é uma função associada, o "construtor alternativo". O `wasm-bindgen`
exporta isso como um método **estático**: `WebGame.restore(json)` no JS, como
`Date.parse` ou `Array.from`.

## `abandon(self)`: o objeto deixa de valer no JS

```rust
pub fn abandon(self, now: f64, finished_at_ms: f64) -> Option<String> {
    self.game
        .abandon(convert::to_ms(now), convert::to_ms(finished_at_ms))
        .map(|result| save_result(&result))
}
```

No Rust, `self` (sem `&`) **consome** o valor: depois de `abandon`, a partida não existe mais, e o
compilador recusa qualquer uso dela (wiki 46). O JavaScript não tem esse conceito, então o
`wasm-bindgen` imita: ele libera a memória do lado do Rust e zera o ponteiro do objeto JS. Uma
chamada depois disso lança `"null pointer passed to rust"`.

```ts
const abandoned = previous.abandon(monotonicNow(), wallClockNow())
previous.view(0) // lança: o objeto já foi consumido
```

A garantia do Rust vira uma checagem em tempo de execução do lado do JS. Por isso o `App.svelte`
zera a referência logo depois (`game = null`): o que o compilador fazia por você, aqui você faz à mão.

## `Option<String>` vira `string | undefined`

```ts
const result = game.result(wallClockNow())
if (result !== undefined) appendResult(storage, result)
```

O `Option` do Rust atravessa a ponte como `undefined` no lugar de `None`. O `.d.ts` gerado diz
`result(finished_at_ms: number): string | undefined`, e o `strict` do TypeScript obriga a checar
antes de usar, que é o mais perto do `match` que o TS oferece.

## Salvar depois de cada ação

```ts
// A foto é gravada depois de **cada** ação, e não só ao fechar a aba: o fechamento não é
// confiável, e no celular a aba pode morrer sem aviso (spec, seção 7).
function persist() {
  if (!recorded) storeSavedGame(storage, game.save(monotonicNow()))
}
```

O `localStorage` é **síncrono**: quando o `setItem` volta, o dado já está gravado. Uma foto tem uns
poucos kilobytes, então gravar a cada jogada não pesa. A alternativa seria esperar o `beforeunload`,
que não dispara quando o sistema mata a aba (wiki 53).

O `recorded` é a trava contra o caso contrário: depois que a partida termina e o resultado é
gravado, nada mais pode salvá-la como "partida guardada", nem gravar o resultado de novo. É um
estado de "já aconteceu" que o Rust expressaria consumindo a partida. No TS, é uma flag.

## `try/catch` no armazenamento: nada daqui lança

```ts
/** Roda `action` e devolve `fallback` se o armazenamento lançar. */
function attempt<T>(action: () => T, fallback: T): T {
  try {
    return action()
  } catch {
    return fallback
  }
}

export function loadSavedGame(storage: Storage): string | null {
  return attempt(() => storage.getItem(SAVED_GAME_KEY), null)
}
```

O `localStorage` lança em situações normais: aba anônima em alguns navegadores, cota cheia
(`QuotaExceededError`), dados do site bloqueados. Em alguns casos, até **ler a propriedade**
`localStorage` lança. Nenhuma dessas falhas pode impedir alguém de jogar. Por isso o módulo de
armazenamento promete que nenhuma função dele lança, e o `browserStorage()` troca um `localStorage`
inacessível por um armazenamento que não guarda nada.

Em Rust, essa promessa estaria no tipo: as funções devolveriam `T`, e não `Result<T, E>`, e quem
chama saberia que não precisa tratar erro. Em TS, as exceções não aparecem no tipo. A promessa está
no comentário do módulo e num teste:

```ts
it('ignora Storage que lança', () => {
  const fail = () => { throw new Error('QuotaExceededError') }
  const broken: Storage = { ...memoryStorage(), getItem: fail, setItem: fail, removeItem: fail }

  expect(() => storeSavedGame(broken, '{}')).not.toThrow()
  expect(loadHistory(broken)).toBe('[]')
})
```

O `Storage` falso é um objeto literal que implementa a interface do DOM. É o equivalente a
implementar um trait para um tipo de teste em Rust: quem recebe um `Storage` não sabe nem se
importa se é o do navegador ou um `Map` em memória. Foi por isso que as funções recebem o `storage`
como parâmetro, em vez de usar o `localStorage` global direto. É o mesmo princípio do core
receber `now` e `seed` em vez de ler o relógio.

## Histórico ilegível: começar de novo

```ts
function readHistory(storage: Storage): unknown[] {
  return attempt(() => {
    const parsed: unknown = JSON.parse(storage.getItem(HISTORY_KEY) ?? '[]')
    return Array.isArray(parsed) ? parsed : []
  }, [])
}
```

O `unknown` é o tipo honesto para o que vem de fora: o TypeScript não deixa usar o valor antes de
conferir o que ele é. O `any` deixaria. Aqui, a única conferência é "é uma lista?". Se o valor
guardado não for um array (corrompido, editado à mão), o próximo resultado começa um histórico novo.
Conferir **cada registro** é trabalho do Rust, na Task 56, com o `GameResult::restore`, que já sabe
o que é um registro válido.
