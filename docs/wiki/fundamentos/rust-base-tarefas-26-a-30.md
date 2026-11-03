# Fundamentos de Rust — consolidado das Tarefas 26 a 30

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 26 a
> 30. As páginas por-task ([26](../26-gerador-dificil.md) … [30](../30-swordfish.md)) contam a história na ordem
> do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 21 a 25](rust-base-tarefas-21-a-25.md).

Este bloco tem pouco código novo e muito cuidado com o que já existe: otimizar o gerador sem mudar nenhum puzzle,
renomear tudo para inglês, enxugar dependências, declarar a licença e generalizar o X-Wing. O assunto, no fundo,
é **mudar código com segurança**.

Sumário:
1. [Custo de memória: pilha, heap e alocação](#1-custo-de-memória-pilha-heap-e-alocação)
2. [Calcular na compilação: `const fn` e `static`](#2-calcular-na-compilação-const-fn-e-static)
3. [Iteradores preguiçosos no lugar de coleções](#3-iteradores-preguiçosos-no-lugar-de-coleções)
4. [Mudar sem mudar a saída](#4-mudar-sem-mudar-a-saída)
5. [Nomes: o que o compilador vê e o que ele descarta](#5-nomes-o-que-o-compilador-vê-e-o-que-ele-descarta)
6. [Cargo: features, licença e a árvore de dependências](#6-cargo-features-licença-e-a-árvore-de-dependências)
7. [Visibilidade e helpers compartilhados](#7-visibilidade-e-helpers-compartilhados)
8. [Tabela-resumo dos conceitos](#8-tabela-resumo-dos-conceitos)

---

## 1. Custo de memória: pilha, heap e alocação

Um array de tamanho fixo (`[usize; 3]`, `[u16; 9]`) vive na **pilha**: reservar espaço para ele é só mover um
ponteiro, praticamente de graça. Um `Vec` guarda os elementos no **heap**: criar um pede memória ao alocador, e
descartá-lo devolve essa memória. Uma vez, isso é barato. Dezenas de milhares de vezes por segundo, num laço que o
gerador chama sem parar, vira o gargalo (página 26).

Regra prática: num caminho quente, se você sabe o tamanho máximo, prefira um array e um contador a um `Vec`.

## 2. Calcular na compilação: `const fn` e `static`

```rust
static UNITS: [[usize; 9]; 27] = build_units();
const fn build_units() -> [[usize; 9]; 27] { /* laços while */ }
```

- **`const fn`** pode ser executada pelo compilador, e o resultado entra pronto no binário.
- Dentro dela, ainda não dá para usar `for` nem iteradores, por isso os laços são `while`.
- **`static`** tem endereço fixo e vive o programa inteiro, então dá para devolver `&'static` para ele.

## 3. Iteradores preguiçosos no lugar de coleções

```rust
pub(crate) fn digits(self) -> impl Iterator<Item = u8> {
    (1..=9).filter(move |&v| self.contains(v))
}
```

`impl Iterator` devolve "algum tipo que itera `u8`", sem nomeá-lo. O iterador é **preguiçoso**: nada é calculado
até alguém pedir o próximo item, e nenhuma coleção é criada. O `move` copia `self` para dentro da closure, porque
o iterador sobrevive à chamada. `values()` continua na API, como `self.digits().collect()`.

## 4. Mudar sem mudar a saída

O gerador é **contrato**: mesma seed, mesmo puzzle. Toda mudança interna (otimizar, renomear, extrair helper,
desligar features) precisa provar que não tocou nisso. As redes usadas no projeto:

| Rede | O que prova |
|---|---|
| Golden tests da seed 42 | o caso do contrato não mudou |
| Referência gravada antes e comparada com `diff` | dezenas de seeds de cada nível não mudaram |
| Testes de equivalência (`construcao_bate_com_candidates_for`) | a versão rápida dá o mesmo que a lenta |
| Testes antigos sem edição | a extração de um helper não mudou o comportamento |

Também ficou claro que **limitar por trabalho** (tentativas) é determinístico, e limitar por relógio não seria: a
seed 459 do Difícil esgota em qualquer aparelho.

## 5. Nomes: o que o compilador vê e o que ele descarta

Nomes locais e privados somem na compilação. Trocar `celulas` por `cells` não muda nenhuma instrução, e por isso
renomear foi seguro (página 27). O Rust ainda ajuda: sem sombra silenciosa, uma colisão de nomes vira erro, e as
capturas em strings de formatação (`"{worst:?}"`) também são checadas pelo compilador. Nomes **públicos** são
outra história: as variantes de `Difficulty` ficaram em português porque trocá-las quebra quem usa a API.

## 6. Cargo: features, licença e a árvore de dependências

- **Features** são interruptores de compilação. `default-features = false` desliga o conjunto padrão; no `rand`,
  isso tirou 8 crates da árvore sem mudar nenhum puzzle (página 28).
- **`cargo tree -e features -i <crate>`** mostra quem puxa um crate e por qual feature. É a pergunta a fazer antes
  de atualizar uma dependência acusada: ela precisa estar lá?
- **`license = "MIT"`** no `Cargo.toml` usa a sintaxe SPDX, entendida pelo crates.io e pelo `cargo-deny`
  (página 29).

## 7. Visibilidade e helpers compartilhados

| Modificador | Visível em |
|---|---|
| (nenhum) | só no módulo e nos filhos dele |
| `pub(crate)` | no crate inteiro, mas não para quem usa a biblioteca |
| `pub` | em todo lugar, se o caminho até ele também for público |

O `line_masks` do Swordfish (página 30) é `pub(crate)`: X-Wing e Swordfish o usam, mas ele não vira API. Recebe a
direção como `fn(usize, usize) -> usize`, o ponteiro de função em que uma closure sem captura se converte.

## 8. Tabela-resumo dos conceitos

| Conceito | Para quê | Exemplo no projeto |
|---|---|---|
| Array na pilha | evitar alocação num laço quente | `[0usize; 3]` no Pointing Pair |
| `const fn` + `static` | tabela pronta no binário | `UNITS` |
| `impl Iterator` + `move` | iterar sem criar coleção | `CandidateSet::digits` |
| Referência gravada + `diff` | provar que a saída não mudou | `referencia.txt` |
| Limite por trabalho | erro determinístico | `MAX_ATTEMPTS_HARD = 250` |
| `default-features = false` | trazer só o que se usa | `rand` sem `std_rng` |
| SPDX em `license` | licença legível por ferramentas | `license = "MIT"` |
| `pub(crate)` | helper interno compartilhado | `line_masks` |
| Laços `a < b < c` | cada combinação uma vez, em ordem | Swordfish |
