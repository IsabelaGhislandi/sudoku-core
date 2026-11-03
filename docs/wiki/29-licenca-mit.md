# 29 — Licença MIT: o que ela permite e como o Cargo a declara

> Referente à escolha da licença do projeto. Commit: `docs: license the project under MIT`.

Sem licença, um repositório público é "todos os direitos reservados": qualquer um pode ler o código, mas ninguém
tem permissão para copiar, modificar ou reaproveitar. Como o projeto é portfólio e estudo, a ideia é o contrário:
que o código possa ser lido **e** reaproveitado.

## O que a MIT diz

Em uma frase: qualquer pessoa pode usar, copiar, modificar, distribuir e até vender o código, **desde que mantenha o
aviso de copyright e o texto da licença**. E o software vem "como está", sem garantia.

| Pode | Precisa | Não tem |
|---|---|---|
| usar, modificar, vender, fechar o código de um derivado | manter o aviso de copyright e a licença | garantia nem responsabilidade de quem escreveu |

Comparada às alternativas:

- **Apache-2.0** tem as mesmas permissões, mais uma cláusula explícita sobre patentes, e é bem mais longa. Boa parte
  do ecossistema Rust usa `MIT OR Apache-2.0` (o `rand`, por exemplo) para quem prefere uma ou outra.
- **GPL-3.0** obriga quem distribuir um derivado a abrir o código dele também.
- A MIT foi escolhida por ser a mais curta e direta para um projeto de portfólio.

## Onde ela aparece

```toml
# sudoku-core/Cargo.toml
[package]
license = "MIT"
```

O campo `license` usa a sintaxe **SPDX**, um padrão de identificadores de licença. `"MIT"`, `"Apache-2.0"` e
expressões como `"MIT OR Apache-2.0"` são entendidos por ferramentas como o crates.io e o `cargo-deny`.

O texto completo fica no arquivo `LICENSE`, na raiz do repositório, que o GitHub reconhece e mostra na página do
projeto.

## E o `cargo-deny`?

Antes, o crate não declarava licença, e o `cargo-deny` acusava `sudoku-core = 0.1.0 is unlicensed`. A solução
provisória era pular a checagem de crates privados (`[licenses.private]`). Com `license = "MIT"`, a exceção saiu:
o próprio projeto passa pela mesma regra que as dependências, e MIT está na lista `allow`.
