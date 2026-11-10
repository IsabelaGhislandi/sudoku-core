# 57 — CI do frontend: um job para o WASM, `npm ci` e o lockfile como contrato

> Referente à Task 57 do plano do frontend. Commit: `chore: build and test the web frontend in CI`.

O CI (wiki 15) conferia só o Rust: fmt, clippy, build e testes em três sistemas. Com o app em `web/`,
um PR podia quebrar o TypeScript, um teste do Vitest ou o build do Vite sem ninguém ver. Esta task
acrescenta um job `web` que roda os mesmos comandos de quem desenvolve.

## O job

```yaml
web:
  name: App web
  runs-on: ubuntu-latest
  defaults:
    run:
      working-directory: web
  steps:
    - uses: actions/checkout@v7
    - uses: dtolnay/rust-toolchain@stable
      with:
        targets: wasm32-unknown-unknown
    - uses: Swatinem/rust-cache@v2
    - uses: taiki-e/install-action@v2
      with:
        tool: wasm-pack
    - uses: actions/setup-node@v7
      with:
        node-version: 24
        cache: npm
        cache-dependency-path: web/package-lock.json
    - run: npm ci
    - run: npm run build:wasm
    - run: npm run check
    - run: npm test
    - run: npm run build
```

Algumas escolhas:

- **Um sistema só.** Os testes do Rust rodam em três sistemas porque o compilador nativo muda de um
  para outro. O `.wasm` e o JavaScript, não: são os mesmos bytes em qualquer lugar. Ubuntu basta.
- **`defaults.run.working-directory`.** Todos os `run` executam dentro de `web/`, sem um `cd` em
  cada passo.
- **O target `wasm32-unknown-unknown`** entra pelo `targets:` da action do Rust. Sem ele, o
  `wasm-pack` falharia logo no `cargo build`.
- **`wasm-pack` pronto.** O `taiki-e/install-action` baixa o binário já compilado, em segundos. Um
  `cargo install wasm-pack` levaria alguns minutos a cada execução. É a mesma action que o job de
  cobertura usa para o `cargo-llvm-cov`.
- **Dois caches.** O `rust-cache` guarda o `target/` (o build do WASM). O `cache: npm` do
  `setup-node` guarda os pacotes baixados, pela chave do `package-lock.json`.

## `npm ci`: o `--locked` do npm

```yaml
# npm ci instala exatamente o que está no package-lock.json, e falha se ele estiver
# desatualizado: o mesmo papel do --locked do Cargo.
- run: npm ci
```

O `package.json` diz **faixas** de versão (`"svelte": "^5.57.3"` aceita qualquer `5.x` a partir
daquela). O `package-lock.json` diz a versão **exata** de cada pacote, inclusive dos indiretos. É o
mesmo par do Rust: `Cargo.toml` com faixas, `Cargo.lock` com versões exatas.

| | Rust | npm |
|---|---|---|
| Faixas | `Cargo.toml` | `package.json` |
| Versões exatas | `Cargo.lock` | `package-lock.json` |
| Instalar respeitando o lock | `cargo build --locked` | `npm ci` |
| Instalar e talvez atualizar o lock | `cargo build` | `npm install` |

O `npm install` pode resolver uma faixa para uma versão mais nova e reescrever o lock. No CI, isso
seria ruim: o build testado não seria o que está no repositório. O `npm ci` apaga o `node_modules`,
instala exatamente o lock e falha se o lock não bater com o `package.json`.

Esse "apaga o `node_modules`" apareceu na prática nesta task. No PC, um servidor do Vite ainda
aberto segurava um arquivo nativo dentro de `node_modules`. O `npm ci` começou apagando, não
conseguiu apagar aquele arquivo, e parou no meio, com a pasta quase vazia. Um `npm install` refez
tudo sem mudar o lock. No CI, onde cada execução começa numa máquina limpa, esse problema não
existe, e é por isso que o `npm ci` é o comando certo lá.

## A ordem dos passos

1. **`build:wasm` antes do `check`.** O TypeScript só conhece a classe `WebGame` pelo `.d.ts` que o
   `wasm-pack` gera. Sem o pacote, o `svelte-check` falharia com "módulo não encontrado", e o erro
   esconderia qualquer problema de verdade.
2. **`check`, depois `test`.** Um erro de tipo aparece antes, com mensagem mais clara que a de um
   teste quebrando.
3. **`build` por último.** Ele roda o `build:wasm` de novo (o script é `npm run build:wasm && vite
   build`). Com o cache do Cargo, a segunda vez só confere que nada mudou.

## A prova do vermelho

Um job de CI que nunca falha não prova nada, assim como um teste. Antes do commit, os mesmos
comandos rodaram localmente com dois erros de propósito: um teste do Vitest com o valor esperado
trocado e uma constante com tipo errado.

```
ERROR "src\lib\levels.ts" 15:14 "Type 'string' is not assignable to type 'number'."
× formata o relógio da barra
Tests  1 failed | 8 passed (9)
```

O `check` e o `test` falharam, cada um no erro que devia pegar. Desfeitos os erros, tudo voltou a
passar. A prova definitiva é o próprio PR: o job roda lá, numa máquina limpa, com o `npm ci` de
verdade.
