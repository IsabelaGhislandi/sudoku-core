# Fundamentos de Rust — consolidado das Tarefas 21 a 25

> Documento de aprendizado **intensivo**, organizado **por conceito** (não por arquivo), a partir das Tasks 21 a
> 25. As páginas por-task ([21](../21-gerador-medio-e-tentativas.md) … [25](../25-x-wing.md)) contam a história na
> ordem do código; **este** documento ensina os fundamentos por trás.
> O bloco anterior está em [Tarefas 16 a 20](rust-base-tarefas-16-a-20.md).

O bloco anterior evoluiu o design para aceitar eliminações. Este **reaproveita** esse design: fecha o gerador de
Médio e acrescenta quatro técnicas de Difícil sem mudar nenhuma peça central. O assunto, no fundo, é **iteração**:
como percorrer coleções, combiná-las e reduzi-las de forma barata e com ordem previsível.

Sumário:
1. [Laços com estado compartilhado](#1-laços-com-estado-compartilhado)
2. [Fatias e empréstimos de arrays](#2-fatias-e-empréstimos-de-arrays)
3. [Referências e valores em iteradores](#3-referências-e-valores-em-iteradores)
4. [Reduzindo iteradores](#4-reduzindo-iteradores)
5. [Combinações sem alocar](#5-combinações-sem-alocar)
6. [Closures, `fn` e genéricos](#6-closures-fn-e-genéricos)
7. [`self` por valor em tipos `Copy`](#7-self-por-valor-em-tipos-copy)
8. [Testes que provam alguma coisa](#8-testes-que-provam-alguma-coisa)
9. [Tabela-resumo dos conceitos](#9-tabela-resumo-dos-conceitos)

---

## 1. Laços com estado compartilhado

O gerador de Médio repete tentativas com **um único** RNG, emprestado como `&mut` a cada volta:

```rust
let mut rng = ChaCha12Rng::seed_from_u64(seed);
for _ in 0..MAX_TENTATIVAS_MEDIO {
    let solution = full_solution(&mut rng);
    ...
}
```

`&mut rng` é um empréstimo exclusivo que termina ao fim de cada chamada, então o mesmo RNG pode ser emprestado de
novo na volta seguinte. Cada tentativa continua a sequência da anterior. O `return` de dentro do laço sai da função
inteira; o valor depois do laço só é usado se nenhuma tentativa deu certo.

## 2. Fatias e empréstimos de arrays

```rust
let unidades = units();          // [[usize; 9]; 27], dono dos dados
for linha in &unidades[..18] {}  // &[[usize; 9]], só uma visão
```

- **Array** (`[T; N]`): tamanho no tipo, vive na pilha, é dono dos dados.
- **Fatia** (`&[T]`): referência a um trecho contíguo, com tamanho em tempo de execução. Não copia nada.
- **Intervalos**: `a..b`, `..b`, `a..`, `..`. São os mesmos tipos usados em `for`.

Diferente do `slice` do JavaScript, a fatia do Rust **não copia**. Ela é um empréstimo, e o borrow checker garante
que o array original continua vivo enquanto ela existir.

## 3. Referências e valores em iteradores

| Expressão | Produz | Uso típico |
|---|---|---|
| `arr.iter()` | `&T` | só ler |
| `arr.iter().copied()` | `T` (se `T: Copy`) | números, máscaras |
| `arr.into_iter()` em array | `T` | consumir ou copiar |
| `.enumerate()` | `(usize, item)` | precisar da posição |

O padrão do parâmetro da closure acompanha o tipo do item: `|&i|` desestrutura um `&usize` e entrega um `usize`;
`|i|` mantém a referência. Escolha o que dispensa conversões. `contains(&x)` pede referência, então um `i: &usize`
serve direto.

## 4. Reduzindo iteradores

| Método | Faz | JavaScript |
|---|---|---|
| `fold(inicial, f)` | acumula um valor | `reduce(f, inicial)` |
| `any(pred)` | `true` se algum vale; para cedo | `some` |
| `all(pred)` | `true` se todos valem; para cedo | `every` |
| `count()` | quantos itens | `length` depois de `filter` |
| `collect()` | monta uma coleção | o próprio array |

Com bitmasks, `fold(0, |m, p| m | (1 << p))` transforma uma lista de posições num `u16`, e `count_ones()` conta os
bits. Comparar dois `u16` é muito mais barato do que comparar duas listas.

## 5. Combinações sem alocar

```rust
for x in 0..n { for y in (x + 1)..n { for z in (y + 1)..n { ... } } }
```

Cada laço começa depois do anterior: cada combinação aparece **uma vez** e **sempre na mesma ordem**. A ordem fixa
é contrato, porque o gerador depende dela para ser determinístico. Nada é alocado: os índices só avançam, e o trio
é um array `[usize; 3]` na pilha.

## 6. Closures, `fn` e genéricos

| Tipo | Aceita | Custo |
|---|---|---|
| `fn(A) -> B` | funções nomeadas e closures **sem captura** | ponteiro, sem genérico |
| `impl Fn(A) -> B` | qualquer closure, uma por vez | genérico, resolvido na compilação |
| `Box<dyn Fn(A) -> B>` | qualquer closure, misturadas | heap e chamada dinâmica |

Cada closure tem um tipo próprio e anônimo, por isso duas closures não cabem num mesmo array. Uma closure sem
captura pode virar `fn`, e aí cabem. Foi o que permitiu ao X-Wing iterar `[por_linha, por_coluna]` e usar uma única
função `procura` para as duas direções.

## 7. `self` por valor em tipos `Copy`

```rust
pub fn union(self, other: CandidateSet) -> CandidateSet
```

Receber `self` por valor normalmente **consome** o objeto. Num tipo `Copy` pequeno, como um `u16` embrulhado, o Rust
copia, e o original continua usável. Isso deixa o encadeamento limpo: `a.union(b).union(c)`. Para tipos grandes ou
sem `Copy`, prefira `&self`.

## 8. Testes que provam alguma coisa

- **Golden test e referência**: os golden tests travam a seed 42, e uma referência de 100 puzzles, gravada antes de
  qualquer mudança, conferiu que Fácil e Médio continuaram idênticos.
- **Medir antes de otimizar**: a medição do Médio mostrou que 80% do tempo ia para uma checagem redundante (página
  21).
- **Ver o teste certo falhar**: um erro de compilação só prova que o código não existia. No Hidden Triple, a
  implementação foi escrita primeiro sem a regra do dígito já colocado, para ver o teste daquela regra falhar
  (página 24).

## 9. Tabela-resumo dos conceitos

| Conceito | Onde apareceu | Ideia central |
|---|---|---|
| `&mut` reemprestado em laço | gerador de Médio | um estado atravessa todas as tentativas |
| Fatia `&arr[..n]` | Box/Line Reduction | visão sem cópia |
| `copied()` | Box/Line, Naked Triple | de `&T` para `T` em tipos `Copy` |
| `fold`, `any` | Hidden Triple, X-Wing | reduzir sem `mut` |
| Três laços crescentes | Naked e Hidden Triple | combinações em ordem fixa, sem alocar |
| Closure → `fn` | X-Wing | duas direções, uma função |
| `std::array::from_fn` | X-Wing | array fixo calculado por índice |
| `sort_unstable` | X-Wing | ordem determinística dos pares |
| `self` por valor | `CandidateSet::union` | encadear sem `&` em tipos `Copy` |
