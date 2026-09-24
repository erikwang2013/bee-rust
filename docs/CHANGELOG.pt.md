# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.1.5] — 2026-09-25

### Adicionado
- READMEs condensados para o crates.io em mais 11 idiomas: `docs/crates-readme.{ja,ko,ru,de,fr,es,pt,hi,ar,bn,id}.md`. O seletor de idioma da página do crates.io agora aponta para esses arquivos em vez dos READMEs completos do repositório, então cada idioma recebe o mesmo documento curto voltado ao desenvolvedor.

## [1.1.4] — 2026-09-25

### Alterado
- A página do crates.io (`docs/crates-readme.md`) agora é bilíngue: inglês em cima, chinês abaixo. O crates.io renderiza apenas um README por crate e não oferece seletor de idioma na página, então o inglês fica na primeira tela e o texto em chinês permanece na mesma página.

## [1.1.3] — 2026-09-25

### Adicionado
- A página do crates.io (`docs/crates-readme.md`) agora traz um seletor de 13 idiomas que aponta para os READMEs do repositório

### Corrigido
- `docs/api.*`: 11 traduções não tinham link para o original em chinês (`api.md`)
- `docs/CHANGELOG.zh.md` e `docs/CONTRIBUTING.zh.md` eram os únicos arquivos da sua família sem link para si mesmos

## [1.1.2] — 2026-09-24

### Adicionado
- `docs/crates-readme.md`: um README dedicado às páginas do crates.io — instalação, exemplo executável (retirado de `examples/hello`, coberto pelos seus testes E2E), tabela de feature flags e índice de sub-crates
- `scripts/publish.sh`: publica os crates do workspace um a um, ignora os já enviados e, em caso de limitação de taxa, aguarda o horário de nova tentativa devolvido pelo crates.io

### Corrigido
- A suíte de testes não compilava (`cargo test --workspace` falhava na main): no teste de integração do `bee_router` faltava `Serialize` em `Submit` (devolvido como resposta `Json`), faltava `Router::build()` em dois helpers e faltava `&` em oito chamadas a `status_line`
- Os testes do exemplo `hello` referenciavam um nome `CARGO_BIN_EXE` incorreto — o target binário mantém o hífen (`CARGO_BIN_EXE_hello-bee`)
- O teste de autoescape de template do `hello` verificava `&#39;` enquanto o tera emite `&#x27;`; o comportamento de escape em si estava correto
- Lints do Clippy `manual_split_once` e `manual_range_contains` no `bee_cli`; um import não utilizado no teste de integração do `bee_router`

### Alterado
- Todos os crates agora trazem metadados `readme` e `repository` — antes nenhuma página do crates.io renderizava um README e seis crates não tinham `repository`
- `examples/hello` está marcado com `publish = false`: permanece no workspace como harness de testes E2E, mas já não é publicado no crates.io

## [1.0.6] — 2026-08-07

### Adicionado
- Implementações reais do `bee_cli`: `new` (scaffolding de projetos), `generate controller/model`, `run` com hot reload via `--watch`, `pack` (build de release + cópia para `dist/`)
- Testes unitários da CLI para scaffolding e geração de código (7 novos testes)

### Corrigido
- `bee_rust::init()` agora fica atrás da feature `logs` — builds com features reduzidas (ex.: `--no-default-features --features kv`) voltam a compilar
- Lint `unnecessary_map_or` do Clippy em `bee_kv::InMemoryKvStore::exists`
- `rustfmt.toml` removeu opções exclusivas do nightly que eram silenciosamente ignoradas no stable; o workspace agora passa em `cargo fmt --all --check`
- `doc = false` no binário `bee_cli` para eliminar a colisão do nome do arquivo de saída do rustdoc com o `bee_rust`
- A porta do exemplo `hello` agora é configurável pela variável de ambiente `PORT`

### Alterado
- `bee-rust migrate` reporta "not implemented" e sai com código de erro diferente de zero (planejado)
- README / README.en atualizados para descrever o comportamento real da CLI

## [1.0.4] — 2026-07-29

### Adicionado
- Filtro de detecção de ataques de segurança via `security-rust` (27 detectores)
- `SecurityFilter` com cobertura para XSS, injeção de SQL, injeção de comandos e path traversal
- Feature flag `security` em `bee_rust` e `bee_router`

### Alterado
- README atualizado com a documentação do recurso de segurança
- README atualizado com a seção de apoio por pagamento (WeChat Pay / Alipay)

### Corrigido
- Sintaxe de identificador raw do Tera em `bee_template` para a edição 2024 do Rust

## [1.0.3] — 2026-07-29

### Adicionado
- Estrutura inicial do workspace com 13 crates
- Roteamento MVC com o trait `Controller` e o `Router`
- ORM com o builder `QuerySet` e a macro derivada `Model`
- Abstração de trait KV/Cache com os backends Redis e Memory
- Gerenciamento de sessão com os backends Memory/Redis
- Gerenciamento de configuração com suporte a INI/YAML/ENV e hot reload
- Renderização de templates via Tera
- Logs com integração com tracing
- Scaffolding de CLI e geração de código
- Stubs de traits para os mecanismos de busca, grafos e séries temporais (drivers planejados)

[1.0.4]: https://github.com/erikwang2013/bee-rust/compare/v1.0.3...v1.0.4
[1.0.3]: https://github.com/erikwang2013/bee-rust/releases/tag/v1.0.3
