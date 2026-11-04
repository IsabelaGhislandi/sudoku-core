# Spec — Módulo Game/Sessão

- **Data:** 2026-11-04
- **Status:** aprovada, fatia 1 em planejamento
- **Módulo:** 2 de 5 (Game/Sessão). Depende do [módulo 1, engine](2026-06-14-sudoku-engine-design.md).
- **Visão relacionada:** [`docs/architecture.md`](../../architecture.md), seções 10 (PvP) e 11.

## 1. Objetivo e motivo

O `sudoku-core` gera, resolve e classifica puzzles, mas ainda não existe **uma partida**: alguém
colocando números, errando, pedindo dica, saindo e voltando. Este módulo cria essa camada.

As regras da partida precisam valer **igual em todo lugar onde o jogo roda**: no app Android
(Tauri), no navegador (WebAssembly) e, no PvP futuro, no servidor. Se cada lugar
reimplementasse as regras, elas divergiriam. Por isso elas ficam num crate Rust puro, usado por
todos.

## 2. A partida, do ponto de vista de quem joga

Decidido em conversa com a pessoa responsável pelo projeto, salvo onde está marcado como
*escolha do autor da spec*.

**Começo.** A pessoa escolhe um nível e recebe um puzzle novo, aleatório. A partida começa com
**3 vidas** e **2 dicas**.

**Colocar um número.** A pessoa escolhe uma célula e um número. O jogo confere **na hora** com a
solução:

- **Certo:** o número fica na célula e **trava**. Ele não pode mais ser apagado, trocado nem
  desfeito, como se fizesse parte do enunciado.
- **Errado:** o número aparece **em vermelho** com um aviso, a pessoa **perde 1 vida**, e o número
  **sai do tabuleiro**: a célula volta a ficar vazia. É a mesma regra do PvP (ver
  `architecture.md`, seção 10). Quanto tempo o vermelho fica na tela é decisão da UI.

**Sem desfazer.** Não há botão de desfazer: números certos travam, e os errados já saem sozinhos.
Uma vida perdida não volta.

**Fim por erros.** Quando as vidas chegam a zero, a partida **termina** como derrota. Não há como
continuar.

**Dicas.** São **2 por partida**. Uma dica revela o valor certo de uma célula, copiado da solução,
e esse número trava como uma jogada certa. A célula é sorteada pelo jogo, entre as vazias, e a
pessoa não escolhe qual.

**Anotações.** São os números pequenos de "pode ser 3 ou 7". **Cada célula tem as suas**, e a
pessoa liga e desliga cada uma. **Nada muda sozinho:** colocar um número numa célula não apaga
anotações das outras células.

**Vitória.** A partida termina como vitória quando as 81 células têm o número certo.

**Sair e voltar.** Se a pessoa fecha o app no meio da partida, ela fica salva como estava, com o
**tempo pausado**. Ao voltar, aparece um aviso perguntando se ela quer **continuar** ou
**reiniciar**. Reiniciar começa **outro puzzle aleatório**, do mesmo nível (*escolha do autor da
spec*: a pessoa não disse o nível).

**Histórico.** Toda partida terminada, vitória ou derrota, vira um registro, por exemplo
`Jogo Médio — 10 min — 13/06/2026`. **Reiniciar ou abandonar uma partida no meio conta como
derrota**: ela entra no histórico e baixa a porcentagem de vitórias, para que fugir de uma partida
ruim não proteja a estatística.

**Estatísticas e ranking.** Por nível: **melhor tempo** e **porcentagem de vitórias**. O ranking,
por enquanto, compara a pessoa **com ela mesma**: suas melhores partidas. O desenho deve deixar
espaço para um **ranking online** no futuro.

## 3. Metas

1. Um crate novo, **`sudoku-game`**, implementa todas as regras da seção 2.
2. O `sudoku-game` é **puro**, como o `sudoku-core`: não lê relógio, não grava arquivo, não acessa
   rede e não tem UI. Compila para Android, desktop e WebAssembly.
3. A mesma partida, com as mesmas entradas, dá sempre o mesmo resultado (inclusive a célula
   escolhida por uma dica), para que o servidor do PvP possa conferir o que o celular diz.
4. O estado de uma partida em andamento pode ser **salvo e restaurado** sem perder nada: tabuleiro,
   anotações, vidas, dicas, erros cometidos e tempo acumulado.
5. Uma partida terminada vira um **resultado** completo o bastante para o histórico, as
   estatísticas, o ranking pessoal e um ranking online futuro.

## 4. Fora de escopo

- **UI**, telas e o aviso de "continuar ou reiniciar" em si. O Game só dá o que a tela precisa
  para mostrar.
- **Gravar no disco** e o banco do histórico. Quem grava é a casca (app ou servidor).
- **O app Tauri, o build para o navegador e o servidor.** São os módulos 3, 4 e 5.
- **Ranking online**, contas e qualquer coisa de rede. Só se deixa o espaço para eles.
- **Regras específicas do PvP** (sala, contagem regressiva, vitória pelo relógio do servidor).
  Ficam para a spec do módulo Online.
- **Anotações automáticas** (preencher ou limpar candidatos sozinho).
- Mudar o `sudoku-core`: sua API pública e a saída do gerador ficam como estão.

## 5. O que não pode acontecer

- **Regra com comportamento diferente entre plataformas.** As regras existem num lugar só.
- **Relógio, arquivo ou rede dentro do `sudoku-game`.** O tempo e a data entram como parâmetro.
- **Uma partida terminada continuar aceitando jogadas.**
- **Recuperar uma vida perdida**, ou destravar um número certo.
- **Um número errado ficar no tabuleiro** depois da jogada.
- **Perder o progresso ao restaurar** uma partida salva.
- **Mudar algum puzzle gerado.** Os golden tests da seed 42 do `sudoku-core` não mudam.

## 6. Arquitetura

Decidido em conversa: **arquitetura em camadas**, no estilo *functional core, imperative shell*.

```
app Tauri + Svelte   ─┐
navegador (Wasm)     ─┼─→  sudoku-game  ─→  sudoku-core
servidor PvP         ─┘     (regras)         (gerar, resolver)
(relógio, disco, rede)
```

- O **núcleo** (`sudoku-core` e `sudoku-game`) recebe dados e devolve dados.
- A **casca** (app, navegador, servidor) lê o relógio, grava no disco, fala com a rede e chama o
  núcleo.

**O repositório vira um Cargo workspace.** O `architecture.md` dizia para fazer isso "só com
necessidade real", e ela apareceu: o app e o servidor vão usar a mesma lógica de partida.

```
Cargo.toml          [workspace] com members = ["sudoku-core", "sudoku-game"]
Cargo.lock          um só, na raiz
sudoku-core/
sudoku-game/        depende de sudoku-core pelo caminho (path)
```

Os crates `src-tauri/` e `server/` entram depois, nos seus módulos.

**No PvP, a solução fica no servidor.** No modo solo, a partida inteira roda no aparelho, com a
solução junto. No PvP, a mesma partida roda **no servidor**: o celular manda a jogada e recebe
"certo" ou "errado", sem nunca receber a solução. Por isso o `sudoku-game` não pode depender de
nada que só existe no aparelho.

## 7. Decisões de comportamento

**Tomadas em conversa:** tudo da seção 2.

**Escolhas do autor da spec** (o plano pode revê-las, dizendo por quê):

- **Tempo.** A casca informa instantes (por exemplo, milissegundos de um relógio monotônico) nos
  eventos de começar, pausar e retomar. O Game guarda o **tempo acumulado** e calcula o tempo
  total a partir dele. A data do registro no histórico também vem da casca.
- **Sorteio da dica.** É determinístico: vem de um RNG semeado a partir da seed do puzzle e do
  número da dica, no mesmo espírito do contrato de determinismo do gerador. A célula sorteada é
  uma das **vazias**.
- **Jogada errada devolve o número tentado.** A tela precisa saber qual número mostrar em vermelho
  e em qual célula, mas o estado da partida já sai com a célula vazia.
- **Anotações** só existem em células **vazias**. Uma jogada errada não mexe nelas: a célula volta
  vazia com as mesmas anotações. Quando o número certo trava numa célula (por jogada ou dica), as
  anotações **dessa** célula somem; as das outras continuam.
- **Seed esgotada.** Se `generate` devolver `AttemptsExhausted`, o Game tenta outra seed derivada
  de forma determinística da primeira (por exemplo, a seguinte), até conseguir. Acontece em cerca
  de 1 seed em 5.000 no Difícil e 1 em 200 no Muito Difícil.
- **A seed inicial vem da casca.** O Game não sorteia seeds sozinho, porque isso exigiria
  aleatoriedade do sistema.
- **Configuração da partida.** Vidas (3) e dicas (2) são parâmetros com esses valores padrão. A
  visão do PvP já prevê que quem cria a sala escolhe o limite de erros.
- **Ranking pessoal** ordena **só vitórias**, pelo **menor tempo**, por nível.
- **Estatísticas e ranking** são funções puras sobre uma lista de resultados que a casca guarda e
  entrega. O Game não mantém essa lista.

**Resultado de uma partida.** Tem, no mínimo: nível, seed, vitória ou derrota (com o motivo da
derrota: vidas esgotadas ou abandono, para o histórico poder mostrar a diferença), tempo total,
erros cometidos, dicas usadas e a data informada pela casca. Guardar a **seed** permite que um
ranking online compare de forma justa quem jogou o mesmo puzzle, e como o cálculo é puro, o
servidor pode refazer a partida em vez de confiar no número que o celular manda.

## 8. Restrições

- **Rust edition 2024**, nomes no código em inglês, comentários e testes em português, TDD com
  testes inline e testes de integração da API pública em `sudoku-game/tests/`. Valem as regras do
  `CLAUDE.md`.
- **Dependências mínimas.** Cada uma passa pela skill `adding-dependencies` e pelo `cargo deny`.
  Salvar o estado provavelmente pede serialização (por exemplo, `serde`), e o Tauri também vai
  pedir na ponte com o Svelte. O plano decide entre `serde` e um formato próprio e, se for `serde`,
  segue a skill antes do `cargo add`.
- **Virar workspace mexe na infraestrutura.** CI (`ci.yml`, `dependencias.yml`, caminho do cache),
  `deny.toml`, Dependabot, os comandos do `CLAUDE.md`, o README e o `CONTRIBUTING.md` precisam
  apontar para o workspace. O `Cargo.lock` passa para a raiz.
- **A dependência por git do README continua funcionando.** O Cargo acha o pacote `sudoku-core`
  dentro de um workspace num repositório git.

## 9. Fatias de implementação

Como no engine, a implementação vem em fatias, cada uma com o seu plano:

1. **Workspace e partida básica.** Workspace, crate `sudoku-game`, CI ajustado. Começar partida,
   colocar números, vidas, dicas, anotações, vitória e derrota.
2. **Tempo e salvamento.** Pausar e retomar, salvar e restaurar o estado, reiniciar com outro
   puzzle.
3. **Resultados.** Resultado da partida, histórico, estatísticas por nível e ranking pessoal.

## 10. Em aberto, para o builder

- Os nomes dos tipos e funções da API pública.
- Como representar o estado de cada célula (vazia, dada, certa por jogada, certa por dica) e as
  anotações.
- A estratégia exata de derivar a seed da dica e a seed seguinte, desde que seja determinística e
  documentada.
- O formato de salvamento, dentro da restrição da seção 8.
- A página de wiki de cada task e o consolidado das tasks 31 a 35, que cai durante este módulo.

## 11. Ainda em aberto no projeto

- **Nome do repositório.** Ele se chama `sudoku-core`, mas vai abrigar o app inteiro. Renomear (por
  exemplo, para `sudoku`) é decisão da pessoa responsável, fora desta spec.
