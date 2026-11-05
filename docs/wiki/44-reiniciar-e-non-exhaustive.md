# 44 — Reiniciar: `#[non_exhaustive]`, `&self` que devolve um valor novo e o fim da fatia 2

> Referente à Task 5 do plano da fatia 2 do Game. Commit: `feat: add restart with a new seed at the same level`.

A última task da fatia deu à partida um atalho para "jogar de novo": outro puzzle, do mesmo nível e
com as mesmas regras (vidas e dicas), com uma seed nova que vem da casca.

## `&self` que devolve um valor novo

```rust
pub fn restart(&self, seed: u64) -> Restart
```

Reiniciar não **muda** a partida atual. Ele lê o nível e a configuração dela e cria outra. Por isso
recebe `&self`, e não `&mut self`. Quem chama decide o que fazer com as duas: trocar uma pela outra,
ou, na fatia 3, guardar o resultado da antiga no histórico antes de jogá-la fora.

É o mesmo estilo dos métodos de `String` como `to_uppercase()`, que devolvem um valor novo em vez de
alterar o original. Em JavaScript, `array.map()` (valor novo) e `array.sort()` (altera o array) são
os dois estilos. Em Rust, a assinatura (`&self` ou `&mut self`) já diz qual é qual.

## Por que devolver uma struct, e não a `Game` direto

Pela spec, reiniciar uma partida em andamento **conta como derrota por abandono**, mas registrar essa
derrota é da fatia 3. Se `restart` devolvesse `Game` hoje, a fatia 3 teria de mudar a assinatura para
algo como `(Game, Option<GameResult>)`, e todo código que chama `restart` quebraria.

A saída foi devolver uma struct desde já:

```rust
#[non_exhaustive]
pub struct Restart {
    pub game: Game,
}
```

A fatia 3 vai acrescentar um campo, por exemplo `abandoned: Option<GameResult>`, e quem só lê
`.game` continua funcionando.

## `#[non_exhaustive]`: avisar que vai crescer

Acrescentar um campo a uma struct pública normalmente **quebra** quem está fora do crate, de dois
jeitos:

```rust
let r = Restart { game };          // montar: falta o campo novo
let Restart { game } = r;          // desestruturar: o padrão não cobre o campo novo
```

`#[non_exhaustive]` proíbe as duas coisas **fora do crate** desde já. Quem usa só consegue ler
campos (`r.game`) ou desestruturar com `..`:

```rust
let Restart { game: new_game, .. } = game.restart(4);
```

O `..` diz "e o que mais houver". Assim, um campo novo não quebra ninguém. O teste
`reiniciar_pela_api_publica` usa essa forma de propósito, para provar que a API pública aceita.

O atributo também existe em enums. Lá, ele obriga quem faz `match` de fora a ter um braço `_`, para
que uma variante nova não quebre a compilação. É o oposto do `match` exaustivo da wiki 32, e a
escolha depende de quem deve reagir à mudança: dentro do projeto, queremos o alarme; numa API para
outros, queremos poder crescer.

## Fim da fatia 2

| Crate | Unitários | Integração | Ignorados |
|---|---|---|---|
| `sudoku-core` | 123 | 4 | 3 (desempenho) |
| `sudoku-game` | 67 | 4 (`api.rs`) + 3 (`save_json.rs`, com a feature `serde`) | 0 |

A partida agora tem relógio, foto salvável e conferida, JSON opcional e reinício. Os números do core
são os mesmos de antes, golden tests incluídos. Falta a fatia 3, **resultados**: o registro de cada
partida terminada (inclusive o abandono), o histórico, as estatísticas por nível e o ranking pessoal.
