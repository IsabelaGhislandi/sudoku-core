# 39 — Testes de integração entre crates: a API vista de fora

> Referente à Task 6 do plano da fatia 1 do Game. Commits: `test: add sudoku-game public API integration tests`,
> `feat: re-export Cell and Grid from sudoku-game` e `chore: allow workspace path dependencies in cargo-deny`.

A última task da fatia jogou duas partidas inteiras, uma até a vitória e outra até a derrota, **de
fora** do crate, do jeito que o app vai usar. O teste passou de primeira, como o plano previa, mas
mostrou uma lacuna na API que os testes de dentro não tinham como ver.

## Cada crate tem a sua pasta `tests/`

```
sudoku-game/
├── src/        testes unitários inline (#[cfg(test)] mod tests)
└── tests/
    └── api.rs  testes de integração
```

Cada arquivo em `tests/` é compilado como um **crate à parte**, que depende do `sudoku-game` como
qualquer outro usuário. Ele só enxerga o que é `pub` e exportado pelo `lib.rs` (wiki 13). Num
workspace, cada membro tem a sua pasta, e os comandos escolhem qual rodar:

```bash
cargo test -p sudoku-game --test api   # só o tests/api.rs do sudoku-game
cargo test -p sudoku-core --test api   # o do core, que também se chama api.rs
cargo test --workspace                 # tudo
```

Os dois arquivos se chamarem `api.rs` não dá conflito, porque `-p` escolhe o pacote.

## O que o teste de fora pegou

O teste precisava do valor da solução para jogar certo. `game.puzzle().solution` é uma `Grid`, e
`Grid::get` devolve um `Cell`. Dentro do crate, `use sudoku_core::Cell` resolve. Mas o arquivo de
integração foi escrito **sem** `use sudoku_core`, de propósito: a spec diz que o app depende só do
`sudoku-game`. Sem acesso ao nome `Cell`, o único jeito era um contorno, `solution.to_line()` e
conversão de byte para número.

Um contorno no teste é um aviso sobre a API: o app teria o mesmo problema. A correção foi reescrever
o helper do jeito natural, com `match` em `Cell::Filled(v)`, ver o teste falhar
(`cannot find type Cell`) e re-exportar o que faltava:

```rust
pub use sudoku_core::{Cell, Difficulty, Grid, Puzzle};
```

A regra prática: **todo tipo que aparece numa assinatura pública precisa estar acessível para quem
usa**. `Puzzle` tem campos `pub` do tipo `Grid`, e `Grid::get` devolve `Cell`, então os três vêm
juntos. Os testes unitários não pegariam isso, porque de dentro do crate tudo está acessível.

## O que o CI pegaria: `cargo deny` e dependência por caminho

A verificação final rodou também o `cargo deny check`, que acusou:

```
error[wildcard]: found 1 wildcard dependency for crate 'sudoku-game'
sudoku-core = { path = "../sudoku-core" }
```

O `deny.toml` tem `wildcards = "deny"`, que proíbe dependência sem versão definida, como
`rand = "*"`, porque ela aceita qualquer versão futura. Uma dependência por `path` sem `version`
cai na mesma regra. O plano tinha suposto o contrário, e a suposição só caiu porque o comando foi
rodado. A opção `allow-wildcard-paths = true` libera exatamente esse caso: dependência por caminho
em crate com `publish = false`. Curingas do crates.io continuam proibidos.

A outra saída seria escrever `{ path = "../sudoku-core", version = "0.1.0" }`. Isso só faz sentido
se o `sudoku-game` for publicado um dia, porque o crates.io troca o caminho pela versão. Como os
dois crates são privados, a opção do cargo-deny diz a intenção com menos coisa para manter
sincronizada.

## Fim da fatia 1

| Crate | Unitários | Integração | Ignorados |
|---|---|---|---|
| `sudoku-core` | 123 | 4 | 3 (desempenho) |
| `sudoku-game` | 27 | 2 | 0 |

Os 123 + 4 do core são os mesmos de antes do workspace, golden tests incluídos. A spec continua com
duas fatias: **tempo e salvamento** (pausar, retomar, salvar, reiniciar) e **resultados** (histórico,
estatísticas, ranking pessoal).
