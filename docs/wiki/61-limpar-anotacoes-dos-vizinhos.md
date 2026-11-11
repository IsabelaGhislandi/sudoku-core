# 61 — Limpar anotações dos vizinhos: captura disjunta em closures, `?` dentro de `find_map` e uma regra que muda de ideia

> Referente à Task 61 do plano de regras de anotação. Commit: `feat: clear a placed number from its neighbours' notes`.

Quando você coloca um número certo, ou uma dica revela um, a anotação desse número some de todas as
células da mesma linha, da mesma coluna e da mesma caixa. Se o 7 entrou na célula do canto, o 7
anotado a lápis em qualquer outra célula da linha 1, da coluna 1 ou da primeira caixa já estava
errado, e o jogo o apaga. As outras anotações dessas células ficam como estavam.

## Uma regra que muda de ideia

Até aqui o jogo fazia o contrário, e de propósito. Havia um teste com o nome
`numero_certo_limpa_so_as_anotacoes_da_propria_celula`, e o glossário dizia que o jogo "só apaga
as de uma célula quando ela trava". A `architecture.md` deixava a limpeza automática como questão
em aberto.

O teste não foi apagado: ele foi **reescrito** para dizer a regra nova
(`numero_certo_limpa_a_anotacao_dos_vizinhos`), no mesmo commit que muda o código, o glossário
(`CONTEXT.md`) e a arquitetura. Num projeto com TDD, os testes são a lista das decisões. Mudar uma
decisão é mudar o teste junto, com uma mensagem de commit que explica o porquê. Um teste antigo que
"atrapalha" uma mudança não é obstáculo: é o lembrete de que alguém escolheu o contrário antes.

A regra mora no `sudoku-game`, e não na tela, porque é regra do jogo: no PvP, o servidor roda a
mesma `Game`, e os dois jogadores têm o mesmo comportamento sem ninguém reimplementar nada em
TypeScript.

## Quem enxerga quem: uma função livre com closures

```rust
/// `true` se as células `a` e `b` são diferentes e dividem linha, coluna ou caixa.
fn sees(a: usize, b: usize) -> bool {
    let (row, col) = (|i: usize| i / 9, |i: usize| i % 9);
    let block = |i: usize| (row(i) / 3, col(i) / 3);
    a != b && (row(a) == row(b) || col(a) == col(b) || block(a) == block(b))
}
```

Três detalhes:

- **Função livre, não método.** `sees` não lê nada da partida: só faz contas com dois índices.
  Por isso fica fora do `impl Game`, sem `&self`. É a mesma ideia de exportar uma função solta num
  módulo de TypeScript em vez de pendurá-la numa classe.
- **Closures como funções locais.** `row`, `col` e `block` são closures guardadas em variáveis,
  como `const row = (i: number) => Math.floor(i / 9)` no JS. A diferença é que no Rust a divisão
  de inteiros já descarta o resto, sem `Math.floor`.
- **Tupla compara por inteiro.** `block` devolve `(linha da caixa, coluna da caixa)`, e `==` entre
  tuplas compara campo a campo. No JS, `[1, 2] === [1, 2]` é `false` (são dois arrays
  diferentes); no Rust, `(1, 2) == (1, 2)` é `true`, porque tuplas são valores.

O `sudoku-core` tem helpers parecidos (`row_of`, `box_of`), mas eles são privados do crate. Abrir
a API pública do core por causa de três contas de uma linha seria pior que repeti-las.

## Mexer em `self.notes` dentro de um laço sobre índices

```rust
fn lock(&mut self, index: usize, state: CellState) {
    self.cells[index] = state;
    self.notes[index] = Notes::default();
    if let Some(value) = state.value() {
        for i in (0..81).filter(|&i| sees(index, i)) {
            self.notes[i].remove(value);
        }
    }
}
```

O laço percorre um iterador (`(0..81).filter(...)`) e, dentro dele, altera `self.notes`. O
borrow checker aceita porque a closure do `filter` não toma nada emprestado de `self`: ela usa só
`index`, um `usize` copiado, e a função livre `sees`.

Se a condição precisasse ler a partida, por exemplo `filter(|&i| self.cells[i] == CellState::Empty)`,
ainda funcionaria. Desde a edition 2021, uma closure captura **só o campo** que usa
(`self.cells`), e não o `self` inteiro. Isso se chama captura disjunta. Como `self.cells` e
`self.notes` são campos diferentes, o empréstimo de leitura de um convive com a escrita no outro.

O que **não** compila é chamar um método dentro da closure, como `filter(|&i| self.is_empty(i))`.
Um método recebe `&self` inteiro, e aí o laço teria uma leitura de `self` viva enquanto o corpo
escreve em `self.notes`. No JS nada disso existe; no Rust, é um dos motivos para preferir funções
livres e campos a métodos "de conveniência" em código que muda o estado.

## Desligar um bit sem saber se estava ligado

As anotações de uma célula são uma máscara de bits (`Notes(u16)`, wikis 06 e 53). Já existia o
`toggle`, que inverte o bit com XOR. Para limpar, inverter seria um bug: se o vizinho **não**
tinha o 7 anotado, o XOR o ligaria. Por isso entrou um método novo:

```rust
/// Desliga o dígito, estivesse ligado ou não.
pub(crate) fn remove(&mut self, value: u8) {
    self.0 &= !(1 << value);
}
```

`1 << value` é a máscara com só aquele bit; `!` inverte todos os bits (no JS seria `~`, porque o
`!` do JS é o "não" lógico e transforma o número em `false`); o `&=` mantém todos os bits, menos
aquele. Chamar duas vezes dá o mesmo resultado que chamar uma, e é isso que a limpeza precisa.

O teste `limpeza_so_apaga_o_numero_colocado` passou de primeira, porque o `remove` já estava
certo. Pela regra do projeto, foi preciso provar que ele pega o erro: trocamos o `remove` por
`Notes::default()` (apagar tudo), o teste falhou com `left: []`, `right: [6]`, e desfizemos.

## Achar um cenário de teste com `?` dentro de `find_map`

O teste precisa de uma célula vazia `a` e, para o número certo dela, de vizinhos vazios na linha,
na coluna e só na caixa, mais uma célula longe. Montar isso à mão para a seed 42 seria frágil.
Por isso o helper procura:

```rust
empty.iter().find_map(|&a| {
    let v = solution_value(game, a);
    let pick = |keep: &dyn Fn(usize) -> bool| { /* primeira vazia que passa em `keep` */ };
    Some(Spread {
        a,
        v,
        row: pick(&|i| i / 9 == a / 9)?,
        col: pick(&|i| i % 9 == a % 9)?,
        // ...
    })
})
```

- `find_map` testa cada `a` e para no primeiro que devolve `Some`. Seria um
  `for` com `return` no JS, ou um `.find()` seguido de um `.map()`, só que numa passada.
- O `?` funciona com `Option` dentro da closure. Se faltar o vizinho de coluna, `pick(...)?`
  devolve `None` na hora, e o `find_map` passa para a próxima célula. É o mesmo `?` do `Result`
  (wiki 36), aplicado ao "não achei".
- `&dyn Fn(usize) -> bool` deixa o `pick` receber closures diferentes. Cada closure no Rust tem um
  tipo próprio, então para aceitar várias pela mesma variável é preciso um **trait object**: uma
  referência a "qualquer coisa que pode ser chamada assim". No TypeScript, um tipo
  `(i: number) => boolean` já cobre todas.

Os testes também calculam linha, coluna e caixa por conta própria (`same_house`), em vez de usar o
`sees`. Um teste que usa o código testado para montar a resposta esperada não pega erro nenhum
nesse código.
