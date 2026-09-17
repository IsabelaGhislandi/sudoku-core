# 07 — Traits, o pattern Strategy e módulos em pasta

> Referente à **Task 7** do plano. Commit: `feat: add Technique trait, Deduction and NakedSingle`.

A partir daqui o crate ganha **técnicas de resolução**. Cada técnica olha a grade e, se
conseguir, devolve **uma jogada dedutível**. Como todas têm a mesma "forma", elas são
intercambiáveis — isso é o pattern **Strategy**, e em Rust ele se escreve com uma **trait**.

## Módulo em pasta: `techniques/mod.rs`

Até agora cada módulo era um arquivo (`grid.rs`, `candidates.rs`). Um módulo com **filhos**
vira uma pasta com um `mod.rs` dentro:

```
src/
  lib.rs                    → mod techniques;
  techniques/
    mod.rs                  → mod naked_single;  +  pub use naked_single::NakedSingle;
    naked_single.rs
```

- `mod techniques;` no `lib.rs` diz "existe um módulo `techniques`" — o compilador procura
  `techniques.rs` **ou** `techniques/mod.rs`.
- `mod naked_single;` dentro do `mod.rs` declara o submódulo. Sem essa linha o arquivo
  `naked_single.rs` simplesmente **não é compilado** (em Rust não existe "auto-import" por
  pasta, como o `index.js`/`barrel` do JS).
- **`pub use naked_single::NakedSingle;`** é um **re-export**: quem estiver de fora escreve
  `techniques::NakedSingle` em vez de `techniques::naked_single::NakedSingle`. A estrutura
  interna de arquivos deixa de vazar na API.

> Enquanto ninguém de fora usa esse re-export, o compilador avisa `unused import`. É
> esperado: some na Task 9, quando o `LogicalSolver` consumir a técnica.

## `Deduction`: o resultado de uma dedução

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deduction {
    pub index: usize,
    pub value: u8,
    pub technique: &'static str,
    pub level: Difficulty,
}
```

"Na célula `index` cabe o dígito `value`, deduzido pela técnica `technique`, que é de nível
`level`." É o que alimenta tanto o solver quanto a **dica** (hint) para o jogador.

### `&'static str` — a lifetime que dura o programa todo

`technique` não é `String` (dona, alocada no heap) e sim `&'static str`: uma **referência** a
um texto que vive **enquanto o programa viver**. Literais como `"Naked Single"` estão
embutidos no binário, então já são `&'static str` — dá pra guardar a referência sem alocar
nada e sem o borrow checker reclamar, porque `'static` nunca "expira".

Regra prática: nome fixo, conhecido em tempo de compilação → `&'static str`. Texto montado em
runtime → `String`.

## A trait `Technique`

```rust
pub trait Technique {
    fn apply(&self, grid: &Grid) -> Option<Deduction>;
}
```

Uma **trait** é um contrato de comportamento — o parente mais próximo de uma `interface` de
TypeScript/Java. Aqui ela declara a assinatura de `apply` **sem corpo**: quem implementar a
trait é obrigado a escrever o corpo.

Duas diferenças importantes vindo do mundo web:

1. A implementação fica **fora** do tipo, num bloco `impl Trait for Tipo`. O tipo não precisa
   "saber" das traits que implementa — dá pra implementar uma trait sua para um tipo alheio.
2. `&self` é explícito: o método **empresta** a instância (não a consome, não a modifica).

O retorno é **`Option<Deduction>`**: `Some(d)` quando a técnica achou uma jogada, `None`
quando ela não se aplica àquela grade. Não é erro não achar nada — por isso `Option`, e não
`Result`.

## `NakedSingle`: uma unit struct

```rust
pub struct NakedSingle;
```

Struct **sem nenhum campo** (*unit struct*) — ela não guarda estado, só existe para ter um
tipo em que pendurar o `impl`. O nome dela é, ao mesmo tempo, o **valor**: por isso o teste
escreve `NakedSingle.apply(&grid)` — não há `::new()` nem `{}` a construir.

## A implementação

```rust
impl Technique for NakedSingle {
    fn apply(&self, grid: &Grid) -> Option<Deduction> {
        for index in 0..81 {
            if let Cell::Empty = grid.get(index) {
                let cands = candidates_for(grid, index);
                if cands.count() == 1 {
                    let value = cands.values()[0];
                    return Some(Deduction {
                        index,
                        value,
                        technique: "Naked Single",
                        level: Difficulty::Facil,
                    });
                }
            }
        }
        None
    }
}
```

Um **Naked Single** ("único nu") é a dedução mais simples do Sudoku: uma célula vazia que só
tem **um** candidato. Se sobrou um só, é ele — sem escolha, sem chute. Por isso o nível é
`Difficulty::Facil`.

O algoritmo: varre as 81 posições, e na primeira célula vazia cujo `CandidateSet` tem
`count() == 1`, devolve a jogada. Reaproveita inteiro o `candidates_for` da Task 6 — a trait
não precisou saber nada de bitmask.

- **`if let Cell::Empty = ...`** — o mesmo `if let` da Task 5, agora numa variante **sem
  dados**. Como `Cell::Empty` não carrega nada, aqui daria pra escrever
  `if grid.get(index) == Cell::Empty` (o `Cell` deriva `PartialEq`); o `if let` mantém o
  estilo usado no resto do crate e continua valendo se a variante ganhar campos.
- **`return Some(...)` no meio do loop** — sai da função na primeira dedução encontrada. O
  `None` solto no fim é a última expressão do corpo: sem `;`, ele **é** o valor de retorno
  quando o loop termina sem achar nada.
- **`index,` e `value,` sem `: valor`** — *field init shorthand*: quando a variável tem o
  mesmo nome do campo, basta escrever o nome (igual ao `{ index }` do JS).

## Por que `Eq`/`PartialEq` em `Deduction`

O teste faz `assert_eq!(NakedSingle.apply(&grid), None)`. Comparar dois `Option<Deduction>`
com `==` exige que `Deduction` saiba se comparar — daí o `#[derive(PartialEq, Eq)]`. E o
`assert_eq!`, quando falha, precisa **imprimir** os dois lados: daí o `Debug`.

## Por que isso importa

A trait é o ponto de extensão de todo o solver. A Task 8 acrescenta `HiddenSingle`
implementando o **mesmo** contrato, e a Task 9 monta o `LogicalSolver`, que percorre uma
lista de técnicas em ordem de dificuldade e aplica a primeira que der resultado. Nenhuma
dessas peças precisa saber *como* cada técnica funciona por dentro — só que ela tem `apply`.
