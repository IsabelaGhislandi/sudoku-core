# 17 — Enums com dados e o solver limitado por nível

> Referente à Task 2 do plano da fatia Médio.
> Commit: `feat: represent eliminations in Deduction and cap the solver by level`.

Esta task não adiciona técnica nenhuma. Ela troca o "encanamento" para que as técnicas de Médio caibam
nele, sem mudar a saída de nenhum puzzle Fácil.

## Um enum que carrega dados diferentes

```rust
pub enum Action {
    Place { index: usize, value: u8 },
    Eliminate(Vec<(usize, u8)>),
}
```

Antes, `Deduction` tinha os campos `index` e `value`. Uma eliminação não tem um valor a colocar, então
esses campos ficariam sem sentido. As alternativas seriam piores:

| Alternativa | Problema |
|---|---|
| `value: Option<u8>` + `removidos: Vec<...>` | permite estados sem sentido, como os dois preenchidos |
| Duas structs separadas | `Vec<Deduction>` deixaria de guardar os dois tipos juntos |

Com um enum, **estados inválidos nem podem ser escritos**. Paralelo em TypeScript:
`{ type: 'place', index, value } | { type: 'eliminate', pairs }`, sem precisar do campo `type`, porque
a variante já é a etiqueta.

As duas formas de variante têm usos diferentes: com campos nomeados (`Place { index, value }`) quando
há vários dados que se confundiriam, e como tupla (`Eliminate(lista)`) quando há um só.

## `match` em uma referência

```rust
match &d.action {
    Action::Place { index, value } => {
        work.set(*index, Cell::Filled(*value));
    }
    Action::Eliminate(pairs) => { /* ... */ }
}
steps.push(d);
```

O `&` importa. Como `Action` tem um `Vec`, ela não é mais `Copy`, e um `match d.action` sem `&` **moveria**
a lista para fora de `d`. Aí o `steps.push(d)` não compilaria.

Com `&d.action`, cada variável do padrão vira uma referência (`index: &usize`), recurso chamado *match
ergonomics*. Por isso aparece o `*index`: o `*` lê o valor. No laço, o padrão faz o inverso:

```rust
for &(index, value) in pairs { /* ... */ }
```

`pairs` entrega `&(usize, u8)`, e o `&` no padrão desembrulha o par. Só funciona porque os dois campos
são `Copy`.

## Mudar uma trait pública

```rust
pub trait Technique {
    fn level(&self) -> Difficulty;
    fn apply(&self, grid: &Grid, candidates: &CandidateGrid) -> Option<Deduction>;
}
```

Mudar a assinatura de uma trait quebra **todo** `impl` dela, em qualquer crate. Aqui não há problema
porque nada de fora depende do crate ainda, mas numa biblioteca publicada isso exigiria subir a versão
*major*. O compilador vira uma lista de tarefas: cada erro aponta um lugar a atualizar.

## Limitando por nível: `filter` em uma lista de traits

```rust
all_techniques()
    .into_iter()
    .filter(|t| t.level() <= level)
    .collect()
```

- `into_iter()` consome a `Vec` e entrega cada `Box<dyn Technique>`, sem clonar.
- `t.level()` funciona direto: a chamada de método atravessa as camadas de referência e o `Box`, e chega
  na vtable (*auto-deref*).
- `<=` só compila porque `Difficulty` deriva `PartialOrd`.
- `collect()` descobre que deve criar uma `Vec` pelo tipo do campo `techniques`.

É isso que protege o Fácil: o `dig_facil` usa `up_to(Difficulty::Facil)`, então as técnicas novas nunca
entram na decisão de cavar uma célula.

## Por que `next_hint` saiu

A dica do jogo sorteia uma célula vazia e copia o valor da `solution`, então não precisa do solver. Essa
lógica fica no módulo Game. Remover API sem uso é mais barato agora do que depois que ela tiver
consumidores.

## A guarda de `rate`

```rust
if !validate(grid).is_empty() || count_solutions(grid, 2) != 1 {
    return None;
}
```

Numa grade quase vazia com conflito, `count_solutions` poderia passar muito tempo tentando provar que não
há solução, e `validate` responde na hora. O `||` tem **curto-circuito**: se a primeira condição for
verdadeira, a segunda nem roda.

## Como garantir que nada mudou

O golden test só cobre a seed 42. Nesta task, gravamos os puzzles Fáceis das seeds 0 a 49 **antes** de
mexer no código e comparamos os bytes depois: idênticos. É um teste temporário, de refatoração: útil
enquanto a mudança acontece, e sem motivo para ficar na suíte.
