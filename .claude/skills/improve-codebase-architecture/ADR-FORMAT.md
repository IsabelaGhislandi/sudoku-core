# Formato dos ADRs

ADR (*Architecture Decision Record*) é um registro curto de **uma** decisão de arquitetura e do
motivo dela. Serve para que uma revisão futura não proponha de novo o que já foi pensado e recusado.

## Onde e como

- Pasta `docs/adr/`, criada quando o primeiro ADR for escrito.
- Nome do arquivo: `NNNN-titulo-em-kebab-case.md`, numeração sequencial com quatro dígitos
  (`0001-core-sem-relogio.md`).
- Em português, como o resto de `docs/`.
- Um ADR aceito não é editado para mudar de ideia: escreva um ADR novo que o **substitui** e marque
  o antigo como `Substituído por ADR-NNNN`.
- Indexe o ADR novo na seção 9 do `docs/architecture.md`, e cite-o na seção do documento que ele
  afeta.

## Modelo

```markdown
# ADR-NNNN: <decisão em uma frase>

- **Status:** Aceito | Substituído por ADR-NNNN
- **Data:** AAAA-MM-DD

## Contexto

O problema e as forças em jogo. O que torna a escolha não óbvia.

## Decisão

O que foi decidido, no imperativo: "O `sudoku-core` não lê relógio."

## Alternativas consideradas

- **<alternativa>:** por que não.

## Consequências

O que fica mais fácil, o que fica mais difícil, e o que uma revisão de arquitetura **não** deve
sugerir de novo sem fato novo.
```

## Quando vale um ADR

Só quando o motivo seria necessário para alguém, no futuro, não repetir a sugestão. Motivos
passageiros ("agora não dá tempo") e óbvios não viram ADR. Decisões que já estão explicadas no
`docs/architecture.md` (seção 3, por exemplo) também não precisam virar ADR, a não ser que uma
revisão volte a questioná-las.
