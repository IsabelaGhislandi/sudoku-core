# 26 — Gerador de Difícil: medir, achar o gargalo e otimizar sem mudar a saída

> Referente à Task 5 do plano da fatia Difícil. Commits: `perf: speed up solver techniques and candidate setup`
> e `feat: generate Hard puzzles with bounded attempts`.

O gerador de Difícil é o mesmo laço do Médio, com outro nível e outro limite de tentativas. O código novo é
pequeno; a história desta task é o **desempenho**.

## Um laço para dois níveis

```rust
Difficulty::Medio   => generate_with_attempts(difficulty, MAX_ATTEMPTS_MEDIUM, seed, &mut rng),
Difficulty::Dificil => generate_with_attempts(difficulty, MAX_ATTEMPTS_HARD, seed, &mut rng),
```

O laço que era do Médio virou a função `generate_with_attempts`, que recebe o RNG como `&mut ChaCha12Rng`. Para o Médio, a ordem
das chamadas ao RNG não mudou, e por isso os puzzles Médios continuaram idênticos.

## A primeira medição: 7,4 segundos

| | Mediana | Pior caso | Acima de 1 s |
|---|---|---|---|
| Primeira versão | 782 ms | 7,43 s | 80 de 200 seeds |
| Aceite sem busca de unicidade, tabela de unidades fixa | 240 ms | 2,14 s | 11 |
| Sem `Vec` no caminho quente | 140 ms | 1,15 s | 3 |
| `from_grid` em uma passada, Hidden Triple filtrado | **91 ms** | **0,79 s** | **0** |

A causa principal não era lentidão do código, e sim **raridade**: só 3,4% das tentativas saíam Difíceis (65% saíam
Fáceis e 32% Médios). Uma seed chegou a precisar de 201 tentativas. Sem como mudar essa taxa sem mudar o
algoritmo, o caminho foi baratear cada tentativa.

## Perfil antes de otimizar

Em vez de adivinhar, um teste temporário mediu o tempo de cada técnica, com `Instant::now()` em volta de cada
`apply`:

```
HP: 30.7µs/chamada    HT: 46.8µs/chamada    BL: 33.0µs/chamada    PP: 16.4µs/chamada
```

As quatro mais lentas tinham algo em comum: criavam um `Vec` pequeno muitas vezes por chamada. Cada `Vec` novo é
uma **alocação no heap**, um pedido de memória ao sistema operacional. Isso é barato uma vez, mas caro dezenas de
milhares de vezes.

## As otimizações, uma a uma

**1. Não repetir o que já foi provado.** O aceite chamava `rate`, que roda `count_solutions` (busca por força bruta)
em todo puzzle. Mas `dig` só devolve puzzles que o solver resolve com deduções forçadas, então eles já são únicos;
é o mesmo argumento da página 21. Agora o aceite pega o nível máximo dos passos do próprio solver.

**2. Uma tabela calculada na compilação.**

```rust
static UNITS: [[usize; 9]; 27] = monta_unidades();
const fn monta_unidades() -> [[usize; 9]; 27] { /* laços while */ }
```

- **`const fn`** pode ser executada pelo compilador. O resultado vai pronto para dentro do binário, e nada é
  calculado em tempo de execução.
- Dentro de `const fn`, o Rust ainda não aceita `for` nem iteradores. Por isso a montagem usa `while`.
- **`static`** é um valor com endereço fixo, que vive o programa inteiro. Por isso `units()` pode devolver
  `&'static [[usize; 9]; 27]`: uma referência que nunca expira.

**3. Iterar sem alocar.**

```rust
pub(crate) fn digits(self) -> impl Iterator<Item = u8> {
    (1..=9).filter(move |&v| self.contains(v))
}
```

`values()` continua existindo na API pública, mas agora é só `self.digits().collect()`. As técnicas usam
`digits()`, que devolve um iterador **preguiçoso**: nenhum `Vec` é criado, e cada dígito é calculado quando o laço
pede. O `move` copia `self` para dentro da closure, porque o iterador vive mais do que a chamada.

**4. Um array na pilha no lugar de um `Vec`.** O Pointing Pair só se interessa por 2 ou 3 células, então
`[0usize; 3]` e um contador bastam. A Box/Line conta e confere o box enquanto percorre, sem guardar a lista.

**5. Uma passada em vez de 81.** `from_grid` chamava `candidates_for` para cada célula, e cada chamada lia a grade
inteira: 81 × 81 leituras. Agora uma passada monta uma máscara de "dígitos usados" por unidade, e cada célula é um
OU de três máscaras. O tempo de montar candidatos caiu de 18 ms para 1,2 ms no perfil.

## Como saber que nada mudou

Cada otimização tinha que dar **exatamente** a mesma saída, porque a ordem de varredura faz parte do contrato.
Três redes conferiram isso:

- os golden tests de Fácil, Médio e Difícil para a seed 42;
- uma referência de 100 puzzles Fáceis e Médios, gravada antes de qualquer mudança e comparada com `diff`;
- testes de equivalência, como `unidades_batem_com_linha_coluna_e_box` e `construcao_bate_com_candidates_for`.

De quebra, o Médio ficou cerca de 5 vezes mais rápido: o pior caso caiu para 44 ms.

## O limite de tentativas

O máximo observado foi 201 tentativas, e o plano mandava usar cerca de 4 vezes isso: 800. A revisão final mostrou
que a conta não fechava. Rodando o mesmo teste três vezes, o pior caso deu 1,08 s, 1,01 s e 0,98 s, e numa faixa
maior de seeds uma precisou de 254 tentativas. Com 800, uma seed azarada levaria mais de 3 s no PC.

O tempo de uma chamada é, no fundo, **número de tentativas × custo de uma tentativa**. O custo já tinha sido
otimizado; restava limitar o número. Com `MAX_ATTEMPTS_HARD = 250`, nenhuma chamada passa de cerca de 1 s com o
PC livre. Em troca, cerca de 1 seed em 5.000 devolve `AttemptsExhausted`, e o Game sorteia outra:

```rust
assert_eq!(
    generate(Difficulty::Dificil, 459),
    Err(GenError::AttemptsExhausted(Difficulty::Dificil))
);
```

O erro também é determinístico: a seed 459 sempre esgota, em qualquer aparelho. É isso que um limite por
tentativas dá e um limite por relógio não daria.

**Medir em PC compartilhado:** com outros programas usando 40% da CPU, a mesma medição chegou a dobrar. Testes de
tempo dizem algo sobre a máquina naquele momento, não só sobre o código. Por isso ficam `#[ignore]`, fora do CI.

> **Lição geral:** meça antes de mexer, e mexa onde o perfil aponta. Aqui, o gargalo não era o algoritmo das
> técnicas, e sim alocações pequenas e repetidas, que nenhuma leitura do código denunciava.
