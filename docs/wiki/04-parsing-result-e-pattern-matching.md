# 04 — Parsing: Result, enum de erro e pattern matching

> Referente à **Task 4** do plano. Commit: `feat: add Grid parsing/serialization (from_line/to_line)`.

Representamos um tabuleiro como uma **string de 81 chars** (`.` ou `0` = vazio, `1..9` =
dígito). `from_line` desserializa; `to_line` serializa.

## `Result<T, E>` — sucesso ou erro, sem exceptions

```rust
pub fn from_line(line: &str) -> Result<Grid, ParseError>
```

Rust não tem `throw`/`try-catch`. Uma função que pode falhar devolve `Result<T, E>`:
`Ok(valor)` no sucesso, `Err(erro)` na falha. Quem chama é **obrigado** pelo compilador a
lidar com os dois casos — via `match`, via `?` (propaga o erro), ou `.unwrap()` (assume
sucesso e dá *panic* se for erro; usamos só em teste).

| Web | Rust |
|-----|------|
| `throw new Error()` | `return Err(...)` |
| `try { } catch (e) { }` | `match resultado { Ok(v) => ..., Err(e) => ... }` |
| `value \| null` | `Option<T>` (`Some`/`None`) |

## Erro como **dado** (`enum ParseError`)

```rust
pub enum ParseError {
    WrongLength(usize),   // carrega o tamanho recebido
    InvalidChar(char),    // carrega o char inválido
}
```

Em vez de uma mensagem de texto, o erro é um `enum` cujas variantes **carregam dados**.
Isso deixa o teste comparar com `==` (`Err(ParseError::WrongLength(3))`) e, no futuro, a UI
decidir o que mostrar para cada caso. Por isso ele deriva `PartialEq`.

## Pattern matching sobre `char`

```rust
let cell = match ch {
    '.' | '0' => Cell::Empty,                  // | = "ou"
    '1'..='9' => Cell::Filled(ch as u8 - b'0'),// ..= = range inclusivo
    other     => return Err(ParseError::InvalidChar(other)), // captura o resto
};
```

O `match` é **exaustivo**: o compilador exige cobrir todos os casos (o braço `other` fecha
o resto). É o equivalente seguro de um `switch`, sem o risco de esquecer um caso.

## Char ↔ dígito: o truque do `b'0'`

`b'0'` é o **byte** do caractere `'0'` (48). Como os dígitos são contíguos na tabela ASCII:

- desserializar: `ch as u8 - b'0'` → `'7'` vira `7`.
- serializar: `(b'0' + n) as char` → `7` vira `'7'`.

## `.map().collect()` para montar a `String`

```rust
self.cells.iter().map(|c| /* char */).collect()
```

`collect()` consome o iterador e monta a coleção-alvo — aqui uma `String`, inferida pelo
tipo de retorno da função. Mesma ideia de `array.map(...).join("")` no JS, mas zero-cost.

## Round-trip: a propriedade que protege

O teste `to_line_faz_roundtrip` garante que `from_line` e `to_line` são **inversos**:
serializar e desserializar devolve o original. É a base segura para salvar/carregar e,
no futuro, "compartilhar puzzle por código".
