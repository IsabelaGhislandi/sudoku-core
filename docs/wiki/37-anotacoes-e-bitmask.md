# 37 — Anotações: newtype sobre `u16`, XOR e `pub(crate)`

> Referente à Task 4 do plano da fatia 1 do Game. Commit: `feat: add per-cell pencil notes`.

As anotações são os números pequenos que a pessoa escreve numa célula ("pode ser 3 ou 7"). Pela
spec, cada célula tem as suas e **nada muda sozinho**. A única exceção é a própria célula: quando o
número certo trava nela, as anotações dela somem.

## Um newtype sobre `u16`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Notes(u16);
```

Nove dígitos cabem em 16 bits: o bit n ligado quer dizer "n anotado". É a mesma ideia do
`CandidateSet` do core (wiki 06). Um **newtype** é uma struct de um campo só, que embrulha outro
tipo para lhe dar um nome e uma API própria. O `u16` de dentro é privado: de fora, ninguém faz
`notes.0 = 0xFFFF` nem soma duas anotações como números. O newtype não custa nada em tempo de
execução, porque `Notes` ocupa exatamente os 2 bytes do `u16`.

- **`Copy`**: copiar 2 bytes é mais barato que emprestar, então `notes(index)` devolve uma cópia em
  vez de `&Notes`. Os métodos recebem `self` por valor pelo mesmo motivo.
- **`Default`**: com derive, vale `Notes(0)`, que é o que se quer aqui: nenhum dígito anotado. É ele
  que monta o array inicial, `[Notes::default(); 81]`. A sintaxe `[x; 81]` só funciona porque
  `Notes` é `Copy`.

## Por que não reaproveitar o `CandidateSet`?

Os dois são máscaras de 9 bits, mas significam coisas diferentes. **Candidatos** são o que o
solver deduz a partir das regras. **Anotações** são o que a pessoa escreveu, e podem estar
erradas. Se fossem o mesmo tipo, nada impediria passar anotações para uma técnica do solver como se
fossem candidatos verdadeiros. Tipos separados deixam essa troca fora de alcance. Além disso, o
`CandidateSet` não tem um método público para ligar um dígito, e acrescentar um mexeria na API do
core, que a spec mantém como está.

## Ligar e desligar com XOR

```rust
pub(crate) fn toggle(&mut self, value: u8) {
    self.0 ^= 1 << value;
}
```

`1 << value` é a máscara só com o bit do dígito. O **XOR** (`^`) inverte exatamente os bits ligados
na máscara: se o bit estava em 0 vira 1, e se estava em 1 vira 0. Os outros não mudam. Uma operação
só faz o "liga ou desliga" do botão de anotação, sem `if`.

| Operação | Expressão | Efeito no bit |
|---|---|---|
| Ligar | `x \| m` | sempre 1 |
| Desligar | `x & !m` | sempre 0 (o `remove` do `CandidateSet`) |
| Inverter | `x ^ m` | troca |
| Testar | `x & m != 0` | lê |

## `pub(crate)`: a regra fica com a `Game`

`toggle` é `pub(crate)`: o crate inteiro enxerga, mas quem usa o `sudoku-game` de fora não. Do
lado de fora, um `Notes` só pode ser **lido** (`contains`, `values`, `is_empty`). O único jeito de
mudar uma anotação é `game.toggle_note(index, value)`, que passa pelas mesmas recusas de `place`.
Partida terminada, índice inválido, dígito fora de 1..=9 e célula com número caem todos no
`check_empty_cell` da wiki 36. Por isso a regra "anotação só em célula vazia" não tem como ser
contornada, nem mesmo pelo app.

O `toggle` também confia que `value` está em 1..=9 e não confere. Isso é seguro porque só a `Game`
chama, e ela já validou. Com `value = 20`, `1 << value` estouraria o `u16`. Deixar o método
interno é o que torna essa confiança aceitável.

## Um lugar só para travar um número

```rust
fn lock(&mut self, index: usize, state: CellState) {
    self.cells[index] = state;
    self.notes[index] = Notes::default();
}
```

Jogada certa e dica (Task 5) travam um número do mesmo jeito. Juntar as duas coisas num método
garante a regra "célula com número nunca tem anotação", que é um invariante como o do
`CandidateGrid` (wiki 33): só existe um caminho que escreve um número numa célula, e ele limpa as
anotações. Uma jogada **errada** não passa por `lock`, porque a célula continua vazia. Por isso as
anotações dela ficam, como o teste `numero_errado_mantem_as_anotacoes` confere.
