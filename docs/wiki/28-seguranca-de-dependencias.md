# 28 — Segurança de dependências: `cargo-deny`, features e uma skill para a IA

> Referente à task de segurança de dependências. Commits: `chore: check dependencies with cargo-deny in CI`,
> `fix: drop unused rand default features` e `docs: add adding-dependencies skill`.

Base: a [OWASP Secure Coding with AI Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secure_Coding_with_AI_Cheat_Sheet.html).
Ela lista riscos que aparecem quando quem escreve o código é um agente de IA. Três deles são sobre dependências:

- **Crates inventados:** a IA sugere um nome que não existe, ou que existe mas é de outra pessoa (typosquatting).
- **Versões vulneráveis:** a memória da IA não conhece alertas recentes.
- **Código que roda no build:** em Rust, `build.rs` e proc-macros executam na máquina de quem compila.

## Três camadas

| Camada | Onde | O que pega |
|---|---|---|
| Skill `adding-dependencies` | `.claude/skills/` | o agente verifica o crate e **pede aprovação** antes do `cargo add` |
| `cargo deny check` local | `deny.toml` (raiz; era `sudoku-core/deny.toml` antes do workspace) | alertas, crates retirados, licenças e origem, antes do commit |
| Workflow `Dependências` | `.github/workflows/dependencias.yml` | o mesmo, em todo PR e **toda segunda-feira** |

A execução semanal existe porque vulnerabilidades novas são publicadas sem que nada mude no repositório. Um
`Cargo.lock` que passou ontem pode falhar hoje.

## `deny.toml`, seção por seção

- **`[advisories]`:** consulta a [RustSec Advisory Database](https://rustsec.org). Crates retirados do crates.io
  (`yanked`) e sem manutenção também falham, porque não vão receber correção.
- **`[licenses]`:** só MIT e Apache-2.0. Uma licença nova vira decisão consciente, e não um efeito colateral. O
  próprio `sudoku-core` declara `license = "MIT"` e passa pela mesma checagem (página 29).
- **`[bans]`:** duas versões do mesmo crate geram aviso; versão `*` falha.
- **`[sources]`:** só o crates.io. Uma dependência puxada direto de um repositório git falha.

## O primeiro achado: um crate retirado

A primeira execução falhou:

```
error[yanked]: detected yanked crate (try `cargo update -p chacha20`)
  ├ chacha20 v0.10.0
    └── rand v0.10.1
```

A sugestão do próprio cargo-deny seria atualizar o `chacha20`. Mas, antes, valia perguntar **por que ele estava
lá**. O `cargo tree -e features -i chacha20` mostrou que ele vinha das **features padrão** do `rand`, que trazem o
`StdRng`, o `ThreadRng` e a aleatoriedade do sistema operacional (`getrandom`). O core não usa nada disso: ele é
determinístico, e o gerador é o `ChaCha12Rng`, do `rand_chacha`.

```toml
rand = { version = "0.10", default-features = false }
```

## Features: o que um crate traz depende do que você liga

Uma **feature** em Cargo é um interruptor de compilação: liga partes opcionais do crate e as dependências que elas
precisam. Todo crate tem um conjunto `default`, ligado automaticamente, a menos que você diga
`default-features = false`.

Desligar o padrão tirou 8 crates da árvore (`chacha20`, `getrandom`, `cpufeatures`, `cfg-if`, `wasip2`, `wasip3` e
duas versões de `wit-bindgen`), e 357 linhas do `Cargo.lock`. Tudo isso era código compilado e confiado, mas
nunca usado. Menos dependências significa menos superfície de ataque e menos alertas para acompanhar.

A saída do gerador não mudou: o `shuffle` e o `ChaCha12Rng` são os mesmos. Os golden tests de Fácil, Médio e
Difícil confirmam isso.

> **Lição geral:** quando uma ferramenta acusa uma dependência, a primeira pergunta é se ela precisa estar lá. A
> correção mais segura é a que remove código, não a que troca a versão.

## Instalando o `cargo-deny` no Windows

O `cargo install cargo-deny` falhou neste PC: uma dependência dele (`zstd-sys`) compila código C, e o compilador da
Microsoft instalado estava sem os headers (`vcruntime.h`). A saída foi o **binário oficial** da release do GitHub,
conferido com o SHA-256 publicado junto:

```bash
sha256sum cargo-deny-0.20.2-x86_64-pc-windows-msvc.tar.gz   # tem que bater com o .sha256 da release
```

No CI, que roda em Linux, o `cargo install` funciona normalmente. Lá ele usa versão fixa e `--locked`, em vez de
uma action de terceiros, para não acrescentar mais código externo ao pipeline.
