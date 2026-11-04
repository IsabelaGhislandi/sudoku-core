# 30 — Swordfish: generalizar o X-Wing e extrair um helper comum

> Referente à Task 1 do plano da fatia Muito Difícil. Commit: `feat: add Swordfish technique`.

O Swordfish é o X-Wing com **3 linhas** em vez de 2. Em três linhas, um dígito só cabe em 2 ou 3 posições de
cada, e todas essas posições caem nas **mesmas 3 colunas**. Cada linha precisa do dígito uma vez, e nenhuma coluna
pode ter o dígito duas vezes. Então as três linhas ocupam, juntas, uma célula em cada uma das 3 colunas. Logo, o
dígito sai dessas colunas em todas as **outras** linhas.

```
         col 1  col 4  col 7
linha 0   5?     5?     .       ← cada linha tem o 5 em 2 ou 3 dessas colunas
linha 4   .      5?     5?
linha 8   5?     .      5?
          │      │      │
          └ o 5 sai das colunas 1, 4 e 7 nas outras 6 linhas
```

É a primeira técnica de nível `MuitoDificil`, e entra depois do X-Wing no solver. Como nenhum gerador antigo usa
técnicas desse nível, nenhum puzzle Fácil, Médio ou Difícil mudou. A referência de 150 puzzles e os três golden
tests confirmam isso.

## Um helper para as duas técnicas

O X-Wing montava, para cada linha, uma máscara `u16` com as posições onde o dígito cabe. O Swordfish precisa
exatamente da mesma coisa. Em vez de copiar o código, ele virou uma função em `techniques/mod.rs`:

```rust
pub(crate) fn line_masks(
    candidates: &CandidateGrid,
    value: u8,
    cell: fn(usize, usize) -> usize,
) -> [u16; 9]
```

- **`pub(crate)`** deixa a função visível no crate inteiro, mas fora da API pública. Quem usa a biblioteca não
  enxerga um detalhe interno das técnicas.
- **`fn(usize, usize) -> usize`** é o mesmo truque da página 25: a closure que diz o que é "linha" e o que é
  "posição" não captura nada, então vira um ponteiro de função. Uma função atende as duas direções.
- Os testes do X-Wing passaram sem nenhuma mudança depois da extração. Isso prova que ele continua se comportando
  igual.

## Três linhas com `a < b < c`

```rust
let fits = |m: u16| (2..=3).contains(&m.count_ones());
for a in (0..9).filter(|&a| fits(masks[a])) {
    for b in ((a + 1)..9).filter(|&b| fits(masks[b])) {
        if (masks[a] | masks[b]).count_ones() > 3 { continue; }
        for c in ((b + 1)..9).filter(|&c| fits(masks[c])) {
            let columns = masks[a] | masks[b] | masks[c];
            if columns.count_ones() != 3 { continue; }
            ...
```

- **`b` começa em `a + 1`, e `c` em `b + 1`.** Assim cada trio de linhas aparece uma vez só, sempre na mesma
  ordem. Isso evita trabalho repetido e mantém a varredura determinística.
- **`|` junta as máscaras.** A união das posições das três linhas é um OU bit a bit, e `count_ones` conta quantas
  colunas ela ocupa. Exatamente 3 é o padrão.
- **Cortar cedo.** Se duas linhas já ocupam mais de 3 colunas, nenhuma terceira resolve, e o `continue` pula o
  laço de dentro inteiro.
- **`(2..=3).contains(&n)`** lê como "n está entre 2 e 3". Uma linha com o dígito numa posição só fica de fora: ali
  o dígito já está decidido (é um Hidden Single), e contá-la geraria eliminações erradas. O teste
  `retorna_none_com_linha_de_uma_posicao` cobre isso.

## Os testes

O helper de montagem `five_in_rows_at(&[(linha, &[colunas])])` recebe uma fatia de tuplas. Cada linha tem suas
próprias colunas, o que permite montar o caso em que uma das linhas tem só 2 posições
(`aceita_linha_com_duas_posicoes`). Os pares esperados foram calculados à mão e conferidos por igualdade exata,
não com `contains`. Assim, uma eliminação a mais também faria o teste falhar.

> **Lição geral:** quando uma técnica nova é a generalização de uma antiga, o primeiro passo é extrair o que as
> duas compartilham. Os testes da antiga, sem mudança, provam que a extração não quebrou nada.
