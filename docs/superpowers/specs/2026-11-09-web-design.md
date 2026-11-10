# Spec — Módulo Frontend: o jogo no navegador

- **Data:** 2026-11-09
- **Status:** aprovada.
- **Módulo:** 3 de 5 (Frontend/UI). Depende do [módulo 2, Game](2026-11-04-game-design.md), que
  depende do [módulo 1, engine](2026-06-14-sudoku-engine-design.md). O módulo 4 (Tauri/APK) vem
  depois e tem spec própria.
- **Visão relacionada:** [`docs/architecture.md`](../../architecture.md), seções 2, 10 (PvP) e 11.

## 1. Objetivo e motivo

O `sudoku-core` e o `sudoku-game` estão prontos, mas até hoje ninguém jogou uma partida: só os
testes. Este módulo põe o jogo na frente de uma pessoa, **no navegador do PC**, com o mínimo de
telas para jogar do começo ao fim.

Por que o navegador primeiro, e não direto o APK: é o caminho mais curto para validar que o core
funciona de verdade nas mãos de alguém. Os dois destinos (navegador e APK) precisam ser
validados; o APK é o módulo 4 e reaproveita este app inteiro.

**Pronto quer dizer:** no navegador do PC, a pessoa joga uma partida inteira, fecha a aba no
meio, volta, continua de onde parou, termina, e vê a partida no histórico.

## 2. O jogo, do ponto de vista de quem joga

Decidido em conversa com a pessoa responsável pelo projeto, salvo onde está marcado como
*escolha do autor da spec*.

**Tela inicial.** O nome do jogo e um botão **Jogar**. Quando existe uma partida guardada, aparece
também um botão **Continuar**. *Escolha do autor da spec:* um botão **Histórico**, para ver o
histórico sem precisar terminar uma partida.

**Escolha de nível.** Quatro botões: Fácil, Médio, Difícil, Muito Difícil, e um jeito de voltar à
tela inicial. Escolher um nível começa uma partida nova, com o seed sorteado na hora (a pessoa
não vê nem escolhe o seed).

**Tela do jogo**, de cima para baixo:

- barra com o nível, o tempo correndo com botão de **pausa**, e as **vidas**;
- o tabuleiro 9×9;
- ferramentas: **Anotar** (liga/desliga o modo anotação), **Dica** (com quantas restam),
  **Recomeçar**;
- teclado com os números de 1 a 9.

A pessoa toca numa célula e depois num número. Com o modo anotação ligado, o número vira
anotação na célula em vez de jogada. As regras (o que é erro, quanto custa uma vida, o que a dica
revela, quando a partida acaba) são as do `sudoku-game` e **não são recalculadas no frontend**.

**Pausa.** Pausar para o tempo. *Escolha do autor da spec:* com o jogo pausado, o tabuleiro fica
escondido, para a pausa não virar tempo grátis de raciocínio.

**Recomeçar.** Começa uma partida nova do mesmo nível, com outro seed (`Game::restart`). A partida
anterior entra no histórico como largada.

**Fim da partida.** Completar o tabuleiro ou perder a última vida termina a partida: o resultado é
gravado e o jogo vai direto para a tela do **histórico**.

**Histórico.** Lista das partidas, mais recentes no topo. Cada linha mostra nível, tempo jogado e
data, por exemplo `Médio — 10 min — 13/06/2026`. Vencer não ganha rótulo; os outros finais ganham:

| Como terminou (`Outcome`) | Rótulo na linha |
|---------------------------|-----------------|
| `Won`                     | nenhum          |
| `OutOfLives`              | **Derrota**     |
| `Abandoned`               | **Largada** (*escolha do autor da spec*) |

No fim da tela, um botão **Início**, que volta à tela inicial. Vencer x perder não é o foco desta
fase; ganha peso com o PvP.

**Sair no meio.** Sair da partida pela seta de voltar ou fechando a aba guarda a partida para
continuar depois. Existe **uma partida guardada por vez**: começar outra (Jogar → nível) larga a
guardada, que entra no histórico como `Abandoned`. Continuar devolve a partida pausada, com o tempo
de onde parou: o tempo em que a aba ficou fechada não conta.

**Seta de voltar.** A tela do jogo tem uma seta de voltar na barra de cima (o `← Médio` do esboço
aprovado), que leva à tela inicial. *Escolha do autor da spec:* o botão voltar do navegador (e,
no módulo 4, o do Android) faz o mesmo em todas as telas: as telas entram no histórico do
navegador (`history.pushState`/`popstate`), para o voltar trocar de tela em vez de sair do site.

**Confirmação.** *Escolha do autor da spec:* largar uma partida em andamento pede confirmação
antes, porque o `Abandoned` é gravado no histórico e não tem volta. Vale para Jogar → nível com uma
partida guardada e para Recomeçar no meio de uma partida. Sair pela seta não pede, porque não larga
nada: só guarda.

## 3. Metas

- Jogar uma partida inteira no navegador do PC, em qualquer um dos quatro níveis.
- Toda regra do jogo vem do Rust, compilado para WebAssembly. O frontend só mostra e repassa.
- Partida em andamento sobrevive a fechar a aba e recarregar a página.
- Histórico sobrevive a fechar o navegador.
- Usável com mouse no PC e já pensado para o dedo: alvos de toque grandes o bastante, layout em
  coluna que cabe numa tela de celular em pé. O mesmo app vira o APK no módulo 4.

## 4. Fora de escopo

- **Design caprichado.** Visual simples e funcional; o design é melhorado depois, numa fase própria.
- **APK e Tauri.** Módulo 4.
- **Estatísticas por nível e ranking pessoal.** A lógica existe no `sudoku-game`
  (`level_stats`, `personal_ranking`) e fica sem tela até o online.
- **Online, PvP, contas, sincronizar histórico entre aparelhos.**
- **Escolher ou digitar o seed**, compartilhar puzzle por código.
- **Publicar num endereço** (GitHub Pages ou outro). Decidido em conversa: nesta fase o jogo roda
  só localmente, com `npm run dev`; o endereço público vem depois. O CI compila o app, mas não
  publica.
- **Mudar regras do `sudoku-game` ou do `sudoku-core`.** Se uma regra parecer errada jogando, vira
  anotação para uma fatia do Game, não ajuste no frontend.

## 5. O que não pode acontecer

- **Regra duplicada no TypeScript.** Se o frontend decide sozinho se um número está certo, se a
  vida cai ou se a partida acabou, as regras vão divergir quando o PvP chegar. O frontend pergunta
  ao WASM.
- **Perder a partida em andamento.** Fechar a aba em qualquer momento, inclusive logo depois de uma
  jogada, não pode perder a jogada.
- **Histórico corrompido derrubar o app.** Um registro que não passa no `GameResult::restore` (ou
  uma partida guardada que não passa no `Game::restore`) é descartado sem travar a tela; o resto do
  histórico continua aparecendo.
- **Tela travada sem retorno** enquanto o puzzle é gerado (ver seção 7, tempo de geração).
- **`sudoku-core` ou `sudoku-game` ganharem dependência de navegador.** Continuam puros (sem IO,
  sem relógio, sem `wasm-bindgen`).

## 6. Arquitetura

Decidido em conversa: **a ponte entre Rust e o navegador é WebAssembly**, e não os comandos do
Tauri. Motivos:

- serve aos dois destinos: o navegador agora, e o APK depois (o Tauri empacota o mesmo app, e o
  WebView do Android roda o mesmo `.wasm`);
- o PvP previsto (`architecture.md`, seção 10) é no navegador;
- os comandos do Tauri só funcionam dentro do Tauri, não num navegador comum.

Isso muda o papel do Tauri, que passa de "ponte Rust↔Svelte" para "empacotamento". O
`architecture.md` é atualizado no mesmo trabalho (seção 2, linha do módulo 4, e o que mais citar a
ponte).

```
sudoku-core  ──►  sudoku-game  ──►  sudoku-wasm  ──►  web/ (Svelte)
 (engine)         (regras)          (ponte JS)        (telas, localStorage, relógio)
```

**`sudoku-wasm`** (crate novo, membro do workspace). Casca fina: expõe a `Game` para JavaScript
com `wasm-bindgen` e converte tipos. Sem regra de jogo própria. É o único crate que conhece o
navegador.

**`web/`** (pasta nova na raiz). App **Svelte 5 + TypeScript**, servido pelo **Vite**
(`npm run dev`), que importa o pacote gerado do `sudoku-wasm`. Responsável pelo que o core não
faz por contrato:

- **Relógio.** Fornece o `now` das jogadas (monotônico, em ms) e o `finished_at_ms` (relógio de
  parede, ms desde a época Unix) dos resultados.
- **Seed.** Sorteia um `u64` com `crypto.getRandomValues`.
- **Persistência.** Guarda no `localStorage` a partida em andamento (o `SavedGame` em JSON) e o
  histórico (lista de `SavedResult` em JSON). Os dois formatos já existem, versionados, com a
  feature `serde` do `sudoku-game`.

*Escolhas do autor da spec:* a pasta `web/` e as ferramentas (Svelte 5, TypeScript, Vite) são as
padrão do ecossistema Svelte hoje e cobrem o que a fase pede.

## 7. Decisões de comportamento

- **Quando salvar a partida.** Depois de cada ação que muda a partida (jogada, anotação, dica,
  pausa) e ao sair da tela do jogo. Não depender só do evento de fechar a aba: ele não é
  confiável em todo navegador, e no celular a aba pode morrer sem aviso.
- **Ao voltar para a aba.** *Escolha do autor da spec:* se a aba ficar oculta
  (`visibilitychange`), a partida pausa e é salva. Assim o tempo não corre com a pessoa fora.
- **Continuar.** `Game::restore` devolve a partida pausada; a tela do jogo abre pausada e a pessoa
  retoma pelo botão de pausa. *Escolha do autor da spec.*
- **Partida guardada inválida** (falha no `restore`, versão desconhecida): é descartada, o botão
  Continuar some, e nada entra no histórico.
- **Partida guardada já terminada** não existe: ao terminar, a partida guardada é apagada no mesmo
  passo em que o resultado é gravado no histórico.
- **Tempo de geração.** Medido em release nativo: até ~800 ms no Muito Difícil, e o WASM tende a
  ser mais lento. A geração não pode congelar a tela sem retorno: no mínimo um estado de
  "gerando..." visível antes de chamar o gerador. Se no WASM passar de ~1 s com frequência,
  mover a geração para um Web Worker. O builder mede e decide.
- **Duas abas ao mesmo tempo.** *Escolha do autor da spec:* não é tratado nesta fase. Duas abas
  na mesma partida guardada podem sobrescrever uma à outra ou duplicar uma linha do histórico. Fica
  registrado como limitação conhecida; o PvP vai pedir outra solução de qualquer forma.
- **Formato do tempo no histórico.** Minutos arredondados (`10 min`); abaixo de 1 minuto,
  `< 1 min`. Data no formato `dd/mm/aaaa`, no fuso do navegador. *Escolha do autor da spec.*

## 8. Restrições

- `sudoku-core` e `sudoku-game` não mudam de comportamento. Se a ponte precisar de algo que a API
  pública não dá, isso é discutido antes, e não contornado no `sudoku-wasm`.
- Dependências novas no Rust (`wasm-bindgen` e o que mais a ponte pedir) seguem a skill
  `adding-dependencies`: verificar o crate e pedir aprovação antes do `cargo add`. `cargo deny
  check` continua passando. Dependências do npm também são apresentadas antes de instalar, com
  o motivo, e ficam no mínimo (Svelte, Vite, TypeScript, Vitest e o que o build do WASM pedir).
- O workspace continua passando nos comandos do `CLAUDE.md` em Linux, Windows e macOS. O
  `sudoku-wasm` precisa compilar no target nativo também (para `cargo test --workspace` e o
  `clippy`), mesmo que só faça sentido em `wasm32-unknown-unknown`.
- **TDD vale para o código com lógica:**
  - `sudoku-wasm`: testes inline com `cargo test`, como os outros crates, para as conversões.
  - `web/`: Vitest para a lógica fora dos componentes: leitura e gravação no `localStorage`
    (inclusive registro corrompido), formatação das linhas do histórico, e a navegação entre telas
    (qual tela vem depois de qual evento).
  - Componentes visuais não ganham teste nesta fase: o design ainda vai mudar.
- **CI** passa a: compilar o `sudoku-wasm` para `wasm32-unknown-unknown`, instalar as dependências
  do `web/` com lockfile (`npm ci`), rodar o type-check, os testes do Vitest e o build do app.
- Código, nomes e comentários seguem `docs/convencoes-de-codigo.md`, inclusive no TypeScript
  (nomes em inglês, comentários em português). Uma página de wiki por task, como sempre, agora
  também com conceitos de WASM e da ponte Rust↔JS.
- O disco `C:` desta máquina está quase cheio (18 GB livres). O target `wasm32` e o
  `node_modules` ocupam espaço; vale conferir antes de instalar.

## 9. Fatias de implementação

Sugestão; o plano decide o corte final.

1. **Ponte e tela do jogo.** `sudoku-wasm` mínimo, `web/` com Vite e Svelte, a tela do jogo
   jogável (célula, número, vidas, fim). Já dá para jogar uma partida no navegador.
2. **Ferramentas e telas.** Anotar, dica, pausa, recomeçar; tela inicial e escolha de nível.
3. **Salvamento e histórico.** Partida guardada (Continuar), histórico no `localStorage`, ida
   automática ao histórico no fim, botão Início.
4. **CI e docs.** Job do WASM e do frontend no CI, `architecture.md` e `CONTEXT.md` atualizados.
   (Pode ser distribuído pelas fatias anteriores, se fizer mais sentido.)

## 10. Em aberto, para o builder

- Como gerar o pacote JS do WASM: `wasm-pack` ou `wasm-bindgen-cli` direto, e como o Vite o
  importa (plugin ou pacote local).
- Como os `u64` atravessam a ponte. O `wasm-bindgen` mapeia `u64` para `BigInt`; o seed pode ir
  como `BigInt` ou texto, e o `now` como `number` convertido no Rust. Escolher o que deixar a API
  JS mais simples, sem perder precisão do seed.
- Formato da API exposta: uma classe `Game` no JS espelhando os métodos da Rust, ou funções que
  devolvem um "retrato" da partida pronto para a tela. O que deixar os componentes mais simples.
- Onde mora o estado no Svelte (runes, store) e como a tela sabe que precisa redesenhar depois de
  uma jogada.
- Como o Vitest roda a lógica que depende do WASM, se depender.
