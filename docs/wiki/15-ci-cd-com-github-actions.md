# 15 — CI/CD com GitHub Actions

> Fora da numeração do plano da fatia Fácil: infraestrutura, não código do engine.
> Spec: [`2026-09-19-ci-design.md`](../superpowers/specs/2026-09-19-ci-design.md).
> Commits: `chore: normalize line endings…`, `chore: format codebase with rustfmt`,
> `chore: add GitHub Actions CI…`.

Até aqui, as regras do `CLAUDE.md` ("testes verdes", "build sem warnings", "clippy limpo")
eram um **combinado**: dependiam de alguém lembrar de rodar os comandos. CI transforma o
combinado em **verificação automática**: o GitHub roda tudo sozinho a cada pull request e
marca o PR com ✅ ou ❌ antes do merge.

## CI vs CD

| Sigla | Nome | Pergunta que responde | Aqui no projeto |
|---|---|---|---|
| **CI** | Continuous Integration | "Esse código pode entrar no `master`?" | ✅ agora: fmt, clippy, build, testes |
| **CD** | Continuous Delivery / Deployment | "Como isso chega em quem usa?" | ⏳ com o Tauri: APK gerado a cada release |

**Delivery** = o pipeline produz um artefato pronto (o APK) e alguém decide publicar.
**Deployment** = publica sozinho, sem humano no meio. Para um app de loja, *delivery* é o
normal: você ainda revisa antes de subir na Play Store.

Por que não CD agora? `sudoku-core` é uma biblioteca pura — não há nada para "entregar". Montar
pipeline de release antes de existir o produto seria resolver um problema que ainda não temos.

> Paralelo web: é o mesmo papel de um `npm run lint && npm test` rodando no GitHub Actions ou
> Vercel antes do deploy. A diferença é só o que se roda.

## Anatomia do workflow

O arquivo é `.github/workflows/ci.yml`. O GitHub lê **qualquer** YAML dessa pasta.

```yaml
on:
  push:
    branches: [master]
  pull_request:
```

**Gatilhos (`on`)**: quando rodar. Em PR (qualquer branch) e em push no `master`. Um push numa
branch de feature *sem* PR não roda — e um push numa branch *com* PR roda uma vez só (pelo
`pull_request`), não duas.

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

**Concorrência**: dois pushes seguidos no mesmo PR → a execução do primeiro é cancelada, porque
o resultado dela já não importa. `${{ … }}` é a sintaxe de expressões do Actions; `github.ref`
identifica o PR/branch.

```yaml
env:
  RUSTFLAGS: -D warnings
```

**Variáveis de ambiente** valem para todos os passos. `RUSTFLAGS` é lida pelo `cargo` e passada
ao `rustc`; `-D warnings` = *deny warnings* — todo warning vira erro. É assim que "build sem
warnings" deixa de depender de alguém olhar a saída.

### Jobs, steps e runners

```
workflow (CI)
├── job fmt        → 1 máquina Ubuntu
│   ├── step: checkout
│   ├── step: instala Rust + rustfmt
│   └── step: cargo fmt --check
└── job test       → 3 máquinas (matriz), em paralelo
    ├── step: checkout
    ├── step: instala Rust + clippy
    ├── step: cache
    └── steps: clippy → build → test
```

- **Job** = uma máquina virtual novinha (o *runner*). Jobs rodam **em paralelo** por padrão.
- **Step** = um comando (`run:`) ou uma action pronta (`uses:`). Steps de um job rodam em
  sequência, na mesma máquina; se um falha, os seguintes não rodam.
- **Action** = um passo reutilizável publicado por alguém (`dono/repo@versão`):
  - `actions/checkout@v7` — baixa o repositório (a máquina começa vazia!).
  - `dtolnay/rust-toolchain@stable` — instala o Rust estável mais recente.
  - `Swatinem/rust-cache@v2` — cache de dependências compiladas.

### Matriz

```yaml
strategy:
  fail-fast: false
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
runs-on: ${{ matrix.os }}
```

Um job vira **três**, um por valor de `os`. `fail-fast: false` impede que uma falha no Windows
cancele Linux e macOS — você quer ver o quadro todo para saber se o problema é do código ou do
sistema.

Uma biblioteca pura como a nossa quase nunca se comporta diferente entre sistemas. A matriz
paga de verdade quando entrar o Tauri (caminhos de arquivo, WebView, toolchain Android), mas
já fica pronta — e já pegou um caso real: finais de linha (veja abaixo).

## As verificações, uma a uma

| Comando | Garante |
|---|---|
| `cargo fmt --check` | Código no formato padrão do `rustfmt`. Não altera nada, só falha se algo estiver fora. |
| `cargo clippy --all-targets -- -D warnings` | Nenhum aviso do clippy, incluindo código de teste (`--all-targets`). |
| `cargo build --locked` | Compila sem warnings (via `RUSTFLAGS`). |
| `cargo test --locked` | Testes unitários + integração, inclusive o golden test da seed 42. |

### `--locked` e o `Cargo.lock`

`Cargo.toml` diz `rand = "0.10"` — qualquer 0.10.x serve. `Cargo.lock` registra **qual**
exatamente foi usada (ex.: 0.10.3). Sem o lock versionado, o CI resolveria as versões do zero
e poderia compilar com uma diferente da sua máquina.

`--locked` vai além: se o `Cargo.lock` não bater com o `Cargo.toml` (você adicionou uma
dependência e esqueceu de commitar o lock), o comando **falha** em vez de atualizar em silêncio.

> Paralelo web: `Cargo.lock` ≈ `package-lock.json`; `cargo build --locked` ≈ `npm ci`.

Aqui isso é ainda mais importante por causa do **contrato de determinismo** (página
[14](14-rng-reproduzivel-e-golden-tests.md)): o golden test roda no CI com as versões travadas.
Se um dia alguém atualizar uma dependência e a saída do gerador mudar, o CI fica vermelho.

### Cache

Compilar `rand` e companhia do zero em cada execução custa minutos. O `rust-cache` salva
`~/.cargo` e a pasta `target/` no fim do job e restaura no começo do próximo, com a chave
derivada do `Cargo.lock` + versão do Rust. Mudou o lock ou o Rust? Cache novo.

## Formatação: `rustfmt`

`cargo fmt` reescreve o código no estilo oficial da comunidade. Rodamos **uma vez** no projeto
inteiro (commit `chore: format codebase with rustfmt`) e, a partir daí, o CI só confere.

Hábito sugerido: rodar `cargo fmt --manifest-path sudoku-core/Cargo.toml` antes de cada commit
(ou ativar "format on save" do rust-analyzer no editor).

Exemplo do que ele muda — ordem de imports (tipos antes de funções) e quebra de cadeias longas:

```rust
// antes
use crate::grid::{box_of, col_of, row_of, Cell, Grid};
let vazias = (0..81).filter(|&i| puzzle.givens.get(i) == Cell::Empty).count();

// depois
use crate::grid::{Cell, Grid, box_of, col_of, row_of};
let vazias = (0..81)
    .filter(|&i| puzzle.givens.get(i) == Cell::Empty)
    .count();
```

### `.git-blame-ignore-revs`

Um commit de formatação toca muitas linhas sem mudar nada. Sem cuidado, o `git blame` passaria
a dizer que **todas** foram escritas por ele. O arquivo `.git-blame-ignore-revs` lista esse
commit; o GitHub o respeita automaticamente, e localmente basta uma vez:

```bash
git config blame.ignoreRevsFile .git-blame-ignore-revs
```

## Finais de linha: `.gitattributes`

Windows usa `CRLF` (`\r\n`) no fim de cada linha; Linux e macOS usam `LF` (`\n`). Com
`core.autocrlf=true`, o Git no Windows convertia os arquivos para CRLF no checkout — daí os
avisos *"LF will be replaced by CRLF"*.

```gitattributes
* text=auto eol=lf
```

Isso fixa **LF em todo lugar**, independente da configuração de cada máquina. Sem isso, o
runner Windows veria bytes diferentes dos outros, e ferramentas como o `rustfmt --check`
poderiam discordar entre sistemas.

## Proteção do `master` (passo manual)

O CI só **informa**; quem **impede** o merge de código vermelho é a proteção de branch. Não dá
para versionar isso no repositório — é configuração do GitHub:

1. No repositório: **Settings → Branches → Add branch ruleset** (ou *Add rule*).
2. Alvo: `master`.
3. Marque **Require status checks to pass** e adicione os checks:
   `Formatação`, `Testes (ubuntu-latest)`, `Testes (windows-latest)`, `Testes (macos-latest)`.
   (Eles só aparecem na busca depois que o workflow rodou ao menos uma vez.)
4. Opcional: **Require a pull request before merging** — ninguém faz push direto no `master`.

## Quando o CI fica vermelho

1. Abra o PR → aba **Checks** (ou **Actions** no repositório) → clique no job com ❌.
2. Expanda o step que falhou; a saída é a mesma que você veria no terminal.
3. Reproduza localmente com o **mesmo comando** (todos estão no `ci.yml`), corrija, faça push.

Um caso esperado: usamos o Rust `stable` **mais recente**. Quando sai uma versão nova, às
vezes o clippy ganha um lint novo e o CI fica vermelho **sem você ter mudado nada**. Não é
bug: é o preço de estar sempre na versão nova. Atualize local (`rustup update`), corrija o
aviso e commite como `chore`.

## Olhando adiante: o CD do APK

Quando o módulo Tauri existir, entra um segundo workflow, algo como `release.yml`:

```yaml
on:
  push:
    tags: ["v*"]      # roda ao criar uma tag, ex.: git tag v0.2.0 && git push --tags
```

Ele instalaria Android SDK/NDK, rodaria `tauri android build`, assinaria o APK com uma chave
guardada em **GitHub Secrets** (nunca no repositório) e anexaria o arquivo a uma GitHub
Release. O CI de hoje continua igual — o CD só roda depois que o código já passou por ele.

## Resumo

- **CI** = verificações automáticas em cada PR; **CD** = entrega automática do artefato.
- Workflow → jobs (máquinas, em paralelo) → steps (comandos, em sequência).
- `-D warnings` e `--locked` transformam convenções em regras que falham.
- Matriz roda o mesmo job em vários sistemas; `fail-fast: false` mostra todos.
- `rustfmt` padroniza; `.git-blame-ignore-revs` preserva o histórico de autoria.
- `.gitattributes` fixa LF para todos os sistemas verem os mesmos bytes.
- Proteção de branch é o que realmente bloqueia o merge — configurada no site do GitHub.
