# Changelog

[简体中文](CHANGELOG.zh.md) · [English](CHANGELOG.md) · [한국어](CHANGELOG.ko.md) · [Русский](CHANGELOG.ru.md) · [Deutsch](CHANGELOG.de.md) · [Français](CHANGELOG.fr.md) · [Español](CHANGELOG.es.md) · [Português](CHANGELOG.pt.md) · [हिन्दी](CHANGELOG.hi.md) · [العربية](CHANGELOG.ar.md) · [বাংলা](CHANGELOG.bn.md) · [Bahasa Indonesia](CHANGELOG.id.md) · [日本語](CHANGELOG.ja.md)

## [1.2.2] — 2026-10-06

### Adicionado
- `bee_orm`: `Model::create() -> Result<Self>` — insere e devolve a instância completa, com a chave primária atribuída pelo banco já preenchida (`INSERT … RETURNING *` no sqlite / postgres, `LAST_INSERT_ID()` + releitura no mysql); `insert()` não muda e continua devolvendo linhas afetadas
- `bee_orm`: o atributo `#[bee(crate = "…")]` — aponta a derive para o caminho do crate ORM (ex.: `#[bee(crate = "bee_rust::bee_orm")]` quando só o metacrate `bee_rust` é dependência); omiti-lo mantém a expansão byte a byte idêntica
- configuração de build do docs.rs completada para os oito crates com módulos públicos atrás de feature (`all-features = true`) — a documentação de features de backend e a tabela de atributos da derive estavam invisíveis no docs.rs

## [1.2.1] — 2026-10-06

### Adicionado
- `bee_orm`: campos de data e Decimal atrás de features opt-in — `chrono` (`NaiveDate` / `NaiveDateTime` / `DateTime<Utc>`) e `rust_decimal` (`Decimal`) mapeiam para os tipos SQL `Date` / `DateTime` / `DateTimeTz` / `Decimal` nos três backends; o `bee_rust` encaminha como `orm-chrono` / `orm-rust_decimal` (fora do `full`)
- `bee_orm`: limites honestos desses tipos — sqlite guarda como TEXT (`typeof` = `text`; declarar DECIMAL daria afinidade NUMERIC e viraria `"1.50"` em REAL 1.5 sem aviso), mysql mantém microssegundos em `datetime(6)` / `timestamp(6)` (TIMESTAMP volta como UTC, DATETIME segue naive) e no pg um `numeric` acima de 29 dígitos significativos volta `NULL`; `NaiveTime` / `DateTime<Local>` / `FixedOffset` / nanossegundos ficam fora e exigem `#[bee(sql_type = "…")]`, e com a feature desligada o modelo não compila (E0277)

### Corrigido
- `bee_orm`: o aviso de relação m2m ausente agora escreve o ident do tipo (`#[bee(m2m(Author))]`) em vez do nome da tabela — a sugestão compila mesmo com a tabela renomeada
- CI: `rust-toolchain` ganhou o input explícito `toolchain: stable`; os dois arquivos de teste que quebravam `clippy --all-targets` sem features receberam gates cfg de arquivo (bee_cache / bee_kv) e o doctest m2m do `bee_orm` ficou agnóstico de backend; o teste ponta a ponta de `migrate` do CLI agora compila o crate scratch online (offline só passava com cache local quente) e roda no CI via `BEE_CLI_E2E=1`

## [1.2.0] — 2026-10-06

### Adicionado
- `bee_orm`: o ORM agora executa de ponta a ponta — parâmetros tipados `Value` (Null / Bool / Int / Float / Text / Bytes), `#[derive(Model)]` com `#[bee(table / column / pk / auto / ignore)]`, `insert` / `update` / `delete` no modelo e execução do QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete`)
- `bee_orm`: pools de conexões para os três backends (`pool::{sqlite, postgres, mysql}::Pool`) — `connect(dsn, max_size)`, `get()` → `CheckedConn`, `query` / `execute`, `status()` e transações via `begin` / `commit` / `rollback`; uma conexão descartada no meio da transação é revertida no sqlite e no postgres
- `bee_orm`: nova feature `postgres-tls` (PostgreSQL via TLS, raízes Mozilla embutidas) e pools reforçados — timeouts de 30 s / 10 s e cache de instruções preparadas no pool postgres
- `bee_orm`: ciclo de vida do modelo — carimbos de tempo com `#[bee(auto_now_add)]` / `#[bee(auto_now)]`, exclusão lógica com `#[bee(soft_delete)]` (delete inverte a flag; `with_deleted()` / `hard_delete()` a contornam), hooks (before/after de insert/update/delete), `Model::insert_many` (lotes de 999 parâmetros, não transacional) e `QuerySet::filter_in` com agregados `sum` / `avg` / `min` / `max`
- `bee_orm`: migrações não destrutivas — `migrate::{create_table, add_missing_columns, sync}` geram DDL conforme o dialeto (sqlite `AUTOINCREMENT` / postgres `IDENTITY` / mysql `AUTO_INCREMENT`), criando tabelas e adicionando colunas ausentes sem nunca remover nem alterar
- `bee_orm`: relações por chave estrangeira — `#[bee(fk = Target)]` (com `#[bee(sql_type = "…")]`) emite DDL de FK e `rel::{fk_column_to, belongs_to, children, children_for}` as leem; a aplicação depende do backend: postgres e sqlite (build bundled) aplicam a referência, mysql ignora `REFERENCES` inline (lacuna para o round 5+)
- `bee_orm`: relações many-to-many — `#[bee(m2m(Target))]` declara a relação, `m2m::{attach, detach, related, related_for, related_ids}` a leem e escrevem, e `create_table` / `sync` criam a tabela de junção (`add_missing_columns` nunca a toca)
- `bee_orm`: colunas JSON — campos `serde_json::Value` mapeiam para `TEXT` / `JSONB` / `JSON` conforme o backend, com semântica de NULL honesta: SQL `NULL` → `None`, um documento JSON `null` armazenado → `Some(Json::Null)`
- `bee_orm`: chaves estrangeiras em nível de tabela no MySQL (opt-in) — `MigrateOptions { table_level_fk }` com `sync_with` / `create_table_with` / `add_missing_columns_with`; desligadas por padrão, e linhas órfãs preexistentes fazem `ADD CONSTRAINT` falhar em vez de serem omitidas em silêncio
- `bee_orm`: `Pool::connect_tls_with` para uma `rustls::ClientConfig` própria, mais um reexport de `bee_orm::rustls` para a versão sempre bater; `connect_tls` mantém as raízes webpki embutidas
- `bee_rust`: quatro features de encaminhamento — `orm-sqlite` / `orm-postgres` / `orm-postgres-tls` / `orm-mysql` encaminham um backend do `bee_orm` através do `bee_rust` (nenhuma está em `full`)
- `bee_cli`: `bee-rust migrate init` gera `src/bin/bee_migrate.rs` (recusa sobrescrever um arquivo existente) e `bee-rust migrate run` o executa via `cargo run --bin bee_migrate`
- `bee_kv` / `bee_cache`: os backends Redis sobrevivem a conexões derrubadas — o `ConnectionManager` por baixo reconecta sob demanda (sem thread em segundo plano) com backoff exponencial e jitter (o comando que esbarra na queda falha; o próximo espera a conexão nova)
- `bee_kv` / `bee_cache`: backends memcached (`MemcacheStore` no bee_kv, `MemcacheCache` no bee_cache); `incr` cria o contador antes do delta, contadores sem sinal param em 0, e TTL 0 exclui a chave

### Alterado
- As migrações e relações do bee_orm (incluindo many-to-many), o subcomando `bee-rust migrate`, os backends Redis / Memcached e as features de encaminhamento ORM do `bee_rust` deixaram de ser marcados como planejados na documentação
- os drivers de busca / grafos / séries temporais deixaram de ser marcados nos docs como planejados ou como stub de trait, passando a implementados (features opt-in); os exemplos antigos foram corrigidos (nomes de tipos inexistentes `ElasticsearchEngine` / `Neo4jDB`, esquema `bolt://` incorreto)

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
