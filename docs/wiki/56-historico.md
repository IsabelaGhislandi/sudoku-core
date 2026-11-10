# 56 — Histórico: filtrar o que vem de fora, `filter_map` e `Intl` no lugar de formatar à mão

> Referente à Task 56 do plano do frontend. Commit: `feat: show the game history`.

A tela do histórico fecha o caminho do jogo: toda partida terminada, perdida ou largada aparece numa
lista, da mais recente para a mais antiga, com o nível, o tempo e a data. Uma partida perdida ganha
o rótulo **Derrota**, uma largada ganha **Largada**, e uma vencida não ganha rótulo nenhum.

O trabalho se divide pela regra de sempre. O Rust confere e ordena, porque sabe o que é um
registro válido. O TypeScript formata, porque a data depende do fuso do navegador.

## Conferir cada registro sozinho

```rust
pub fn history_entries(json: &str) -> String {
    let records: Vec<Value> = serde_json::from_str(json).unwrap_or_default();
    let mut entries: Vec<HistoryEntry> = records
        .into_iter()
        .filter_map(|record| serde_json::from_value::<SavedResult>(record).ok())
        .filter_map(|saved| GameResult::restore(&saved).ok())
        .map(|result| HistoryEntry { ... })
        .collect();
    entries.sort_by_key(|entry| Reverse(entry.finished_at_ms));
    serde_json::to_string(&entries).expect("as entradas só têm campos simples")
}
```

O histórico mora no `localStorage`, e qualquer coisa que vem de lá é **entrada de fora**: pode ter
sido gravada por uma versão antiga, corrompida ou editada à mão pelo DevTools. O spec é claro: um
registro ruim não pode esconder os bons.

Por isso a leitura acontece em duas etapas:

1. **`Vec<Value>`, e não `Vec<SavedResult>`.** Se o array inteiro fosse lido direto como
   `Vec<SavedResult>`, um único registro ruim faria o `from_str` falhar, e o histórico todo sumiria.
   Lido como `Vec<Value>` (JSON genérico), o array sempre entra, e cada elemento é conferido depois,
   sozinho.
2. **Cada `Value` vira `SavedResult` e depois `GameResult`.** O primeiro passo confere a forma
   (os campos existem e têm o tipo certo). O segundo confere o conteúdo (versão conhecida, nível
   válido, seed que cabe num `u64`...). As duas conferências já existiam: a ponte só as aplica em
   sequência.

O `unwrap_or_default()` cobre o caso extremo: se o texto nem for um array (`"{}"`, lixo), sai um
`Vec` vazio. Ele é um dos poucos `unwrap_*` permitidos fora de testes, porque não pode entrar em
pânico: ele **troca** o erro por um valor padrão, em vez de explodir.

## `filter_map`: filtrar e transformar num passo só

```rust
.filter_map(|record| serde_json::from_value::<SavedResult>(record).ok())
```

O `filter_map` recebe uma closure que devolve `Option`. O `Some(x)` passa adiante como `x`, e o
`None` é descartado. Junto com `.ok()`, que transforma `Result<T, E>` em `Option<T>` (jogando o
erro fora), a linha diz: "tente converter; o que falhar, ignore".

Em JavaScript, seriam duas passadas, ou um `flatMap` com array vazio:

```ts
records
  .map(r => { try { return parse(r) } catch { return undefined } })
  .filter(r => r !== undefined)

// ou
records.flatMap(r => { try { return [parse(r)] } catch { return [] } })
```

Jogar o erro fora com `.ok()` é uma decisão consciente. Aqui, o motivo de um registro ser ruim não
interessa à tela: ela não tem o que fazer com ele. Em outro lugar, como o `restore` da foto, o erro
importa e volta inteiro.

Como os iteradores do Rust são preguiçosos (wiki 47), as três etapas (`filter_map`, `filter_map`,
`map`) passam **uma vez** pela lista, elemento por elemento, sem criar vetores intermediários. Só o
`collect` materializa o resultado.

## Ordenar do mais recente: `Reverse`

```rust
entries.sort_by_key(|entry| Reverse(entry.finished_at_ms));
```

O `sort_by_key` ordena do menor para o maior. O `std::cmp::Reverse` é um embrulho que inverte a
comparação: o maior `finished_at_ms` (o mais recente) vem primeiro. Em JS, seria
`sort((a, b) => b.finishedAtMs - a.finishedAtMs)`. O `Reverse` evita o comparador escrito à mão,
onde é fácil trocar `a` e `b`.

A ordenação é **estável** (wiki 47): dois resultados com a mesma data ficam na ordem em que foram
gravados.

## A prova de um teste que passou de primeira

O teste dos registros ruins passou logo na primeira execução, porque o `filter_map` com `.ok()` já
descartava tudo que falhava. A regra do projeto pede uma prova nesse caso: quebrar o código de
propósito e ver o teste falhar.

A quebra escolhida foi aceitar registros de versão desconhecida, tentando de novo com
`version: 1`. O teste falhou (3 entradas em vez de 2), e a quebra foi desfeita. Isso mostra que o
teste pega o comportamento, e não só passa por acaso.

## `Intl.DateTimeFormat`: a data no fuso de quem joga

```ts
const date = new Intl.DateTimeFormat('pt-BR', {
  day: '2-digit',
  month: '2-digit',
  year: 'numeric',
  timeZone,
}).format(entry.finishedAtMs)
```

O resultado guarda a data em UTC (o `finished_at_ms`, wiki 45). Mostrar "13/06/2026" depende do fuso
de quem está olhando: uma partida terminada às 23h em São Paulo já é dia 14 em UTC. O
`Intl.DateTimeFormat` faz essa conversão com a base de fusos do próprio navegador. Escrever à mão
(`getDate()`, `padStart`) também funcionaria, mas o `Intl` já sabe o formato `pt-BR` e os horários
de verão.

O `timeZone` é parâmetro só para o teste: sem ele, o teste dependeria do fuso da máquina e falharia
no CI, que roda em UTC. Na tela, ele fica `undefined`, e vale o fuso do navegador. É o mesmo
princípio de receber `now` em vez de ler o relógio, agora aplicado ao fuso.

## `Record` para o rótulo: exaustividade no TS

```ts
const OUTCOME_LABELS: Record<HistoryEntry['outcome'], string | null> = {
  Won: null,
  OutOfLives: 'Derrota',
  Abandoned: 'Largada',
}
```

O `Record<'Won' | 'OutOfLives' | 'Abandoned', ...>` obriga o objeto a ter as três chaves. Se o Rust
ganhar um quarto desfecho e o tipo `HistoryEntry` for atualizado, este objeto deixa de compilar até
alguém escolher o rótulo. É o mesmo efeito do `match` exaustivo do Rust, conseguido com um tipo.
