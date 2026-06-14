# 02 — Enums e `derive` (e o truque do `Ord`)

> Referente à **Task 2** do plano da fatia Fácil. Commit: `feat: add orderable Difficulty enum`.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Difficulty {
    Facil,
    Medio,
    Dificil,
    MuitoDificil,
}
```

## `enum` é um tipo "soma"

Um `enum` em Rust significa que o valor é **exatamente uma** das variantes. Diferente de C,
não é só um número disfarçado: combinado com `match`, o compilador **obriga** a tratar todas
as variantes (exaustividade). É o equivalente — porém à prova de erro — das *discriminated
unions* do TypeScript, mas sem o risco de esquecer um `default:`.

## `#[derive(...)]` gera implementações de graça

`derive` pede ao compilador para gerar implementações automáticas, sem escrever código:

| Derive | O que habilita | Por que aqui |
|--------|----------------|--------------|
| `Debug` | imprimir com `{:?}` | usado pelo `assert_eq!` nos testes |
| `Clone` | cópia explícita (`.clone()`) | base para `Copy` |
| `Copy` | cópia **implícita** e barata | o valor é pequeno; não há `move` ao passá-lo |
| `PartialEq`, `Eq` | operador `==` | comparar níveis |
| `PartialOrd`, `Ord` | `<`, `>`, `.max()`, ordenação | comparar/ordenar dificuldades |

> `Copy` vs `Clone`: por padrão Rust **move** valores (transfere posse). `Copy` diz "esse
> tipo é pequeno o bastante para ser duplicado automaticamente", então usar o valor não o
> invalida. `Clone` é a cópia explícita; `Copy` exige `Clone`.

## O truque do `Ord`: ordem de declaração

O `Ord` derivado ordena as variantes pela **ordem em que são declaradas**. Como declaramos
`Facil, Medio, Dificil, MuitoDificil` nessa sequência, já vale:

```rust
Difficulty::Facil < Difficulty::Medio          // true
niveis.iter().max()                            // pega o nível mais alto
```

Nenhum código manual de comparação — só a ordem certa no `enum`. É exatamente disso que o
`rating` vai precisar depois: pegar a **técnica mais avançada** usada com `.max()`.

## Sem fase "vê falhar" no TDD

Esta task não teve o passo *red* clássico do TDD: não há lógica manual para quebrar — todo
o comportamento (`==`, `<`, `.max()`) vem dos `derive`. O teste serve para **fixar o contrato**
(a ordem dos níveis) e proteger contra alguém reordenar as variantes sem querer.
