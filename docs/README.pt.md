<!-- Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz -->
# Beerust

[简体中文](../README.md) · [English](README.en.md) · [한국어](README.ko.md) · [Русский](README.ru.md) · [Deutsch](README.de.md) · [Français](README.fr.md) · [Español](README.es.md) · [Português](README.pt.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [বাংলা](README.bn.md) · [Bahasa Indonesia](README.id.md) · [日本語](README.ja.md)

O Beerust é um framework web de produção escrito em Rust, cuja filosofia de design é originária do framework Beego de Go, reexpressa com os traits, macros e o sistema de tipos idiomáticos de Rust.

## Mascote do projeto: Rusty

<img src="rusty.svg" width="360" alt="Rusty pairando de frente, asas abertas em círculo, patas dianteiras segurando a caixa de ferramentas cor de ferrugem laranja">

### Ficha do personagem

**Nome** Rusty (Ferrugento) — a cor da ferrugem de Rust, e também a ferrugem esfregada no seu macacão de trabalho.

**Aparência**: uma abelha engenheira peluda. A pelagem âmbar-dourada cobre o abdômen redondo e volumoso, com três listras pretas como carvão que parecem suspensórios de macacão tortos; quatro asas transparentes tão finas que deixam passar a luz, zumbindo em rastro fantasma ao pairar; os olhos compostos são duas contas de vidro de obsidiana, cada uma com um pontinho de brilho; as patas dianteiras seguram uma caixa de ferramentas cor de ferrugem laranja, bem polida pelo uso — essa é a sua cesta de pólen, que nunca carrega pólen, e sim dependências.

**Personalidade**: workaholic, otimista, levemente compulsiva.

- Ao consertar a colmeia, verifica célula por célula com as antenas, tão rigorosa quanto rodar um lint
- Trava firmemente as versões das dependências na cesta de pólen; quem mexer nelas, enfrenta a fúria dela
- Ao caçar bugs, fica imóvel pairando no ar, com as asas tão rápidas que resta apenas um círculo de rastro
- No dia do lançamento de uma nova versão, dança a dança do oito das abelhas na entrada da colmeia

### Especificações visuais

| Elemento | Especificação |
|------|------|
| Corpo | Âmbar-dourado `#F5B301`, pelo abdominal curto e denso, com um contorno de brilho na borda |
| Listras | Preto-carvão `#1F1A17`, três, apenas no abdômen |
| Asas | Azul-claro transparente `#CFE8F5`, 40% de opacidade, com nervuras visíveis |
| Olhos compostos | Marrom-escuro-preto `#2A1E16`, ovais, com um pontinho de brilho no canto superior esquerdo |
| Detalhes | Ferrugem-laranja `#B7410E` (caixa de ferramentas, gravata-borboleta); o tom ferrugem é a sua assinatura |

### Catálogo de expressões

| Expressão | Visual | Quando |
|------|------|---------|
| Cheia de energia | `(^o^)` pelagem eriçada, asas vibrando em alta frequência | quando todos os testes passam |
| Debug pairando | `o(°▽°)o` pairando no ar, restando apenas o rastro das asas | ao caçar um bug difícil |
| Retorno com carga total | `(≧▽≦)` caixa de ferramentas recheada, voando torta | quando um novo driver é integrado com sucesso |
| Cochilando | `(-_-)zZ` agachada na entrada da colmeia, antenas caídas | tarde da noite, sem tráfego |
| Irritada | `(#°益°)` ferrão eriçado | quando o filtro de segurança bloqueia um ataque |

### Movimentos de assinatura

1. **Debug pairando** — ao encontrar um bug, paira sobre o código, com as asas zumbindo em rastro, olhando fixamente até você corrigir.
2. **Dança do oito** — a cada lançamento de nova versão, dá uma volta completa ao redor da colmeia, anunciando para toda a colmeia: "as flores desabrocharam". As abelhas de verdade usam isso para transmitir a localização da fonte de flores; a Rusty usa isso para transmitir o changelog.
3. **Alisar a pelagem** — esfrega as antenas repetidamente com as patas dianteiras. Você pensa que é vaidade; na verdade ela está verificando se a configuração foi recarregada a quente.

### Onde ela aparece

- **Logo**: posição de pairar de frente, asas abertas em anel, circundando exatamente o nome.
- **Página 404**: pairando sobre um campo de flores vazio, girando confusa — "e o mel?"
- **CLI**: uma pequena abelha na linha do banner de inicialização, agachada ao lado supervisionando enquanto os testes rodam.
- **Anúncio de lançamento**: aquela em que dança o oito.

## Objetivos de design

| Objetivo | Métrica |
|------|------|
| **Experiência de desenvolvimento** | De `bee-rust new` à primeira requisição em < 30 s |
| **Desempenho** | Sobrecarga da camada de controladores < 5% (comparado ao axum puro), latência de roteamento P99 < 100 µs |
| **Velocidade de compilação** | Compilação completa do meta crate < 60 s (release), compilação incremental < 5 s |
| **Tamanho do binário** | Aplicação mínima (apenas router) < 5 MB (strip + LTO) |
| **Segurança** | 0 código de negócio com unsafe; todo FFI encapsulado em crates `*-sys` separados |
| **Compatibilidade** | MSRV Rust 1.80+, acompanhando o canal stable |

## Princípios de design

1. **Filosofia do Beego, expressão em Rust** — MVC, namespaces, cadeia de filtros, implementados com traits + macros
2. **Aprimoramento progressivo** — o núcleo mínimo depende apenas de axum + tokio; todo o resto fica atrás de feature gates
3. **Explícito melhor que implícito** — registro de rotas, mapeamento de modelos e ordem de middlewares são todos declarados explicitamente no código
4. **Abstrações de custo zero** — despacho estático via traits, expansão em tempo de compilação via macros, sem sobrecarga de funções virtuais
5. **Independência dos mecanismos de armazenamento** — cada trait de mecanismo pode ter sua implementação trocada isoladamente, sem afetar a lógica de negócio acima
6. **Observabilidade embutida** — instrumentação de tracing + metrics em todo o framework, logs estruturados ativados por padrão

## Arquitetura

### Topologia de crates

```
bee_rust/           # Meta crate, re-export + feature flags
bee_router/         # Roteamento + controladores + Context + cadeia de filtros
bee_orm/            # ORM — trait Model + QuerySet
bee_kv/             # Abstração unificada KV/Cache — Redis / Memcached
bee_search/         # Mecanismo de busca/análise — Elasticsearch + OpenSearch + ClickHouse
bee_graph/          # Banco de dados de grafos — Neo4j + NebulaGraph + ArangoDB
bee_tsdb/           # Banco de dados de séries temporais — InfluxDB + Apache IoTDB + QuestDB
bee_config/         # Gerenciamento de configuração — INI/YAML/ENV + hot reload
bee_cache/          # Abstração de cache — Memory / Redis / Memcache
bee_session/        # Session — backends Memory, Redis (implementado) / Cookie, Database (planejado)
bee_logs/           # Logs — log em múltiplos níveis + integração com tracing
bee_template/       # Renderização de templates — baseado em tera
bee_cli/            # CLI — scaffolding/geração de código/execução de desenvolvimento/empacotamento (migrate init/run incluído)
```

### Diagrama de arquitetura

```
                          ┌───────────────────────┐
                          │  bee_rust  (meta)      │
                          │  re-export + features  │
                          └───────────┬───────────┘
                                      │
              ┌───────────────────────┼───────────────────────┐
              │                       │                       │
     ┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
     │    Web Layer     │    │   Data Layer    │    │   Tool Layer    │
     └────────┬────────┘    └────────┬────────┘    └────────┬────────┘
              │                       │                       │
  ┌───────────┼───────────┐  ┌───────┼───────┐  ┌───────────┼───────────┐
  │ bee_router            │  │ bee_orm        │  │ bee_cli               │
  │  - route register     │  │  - Model/Query  │  │  - scaffolding        │
  │  - controller trait   │  │  - QuerySet    │  │  - hot reload          │
  │  - filter chain       │  │  - Connection   │  │  - code generation    │
  │  - param extract      │  │                 │  │                       │
  ├────────────────────────┤  ├────────────────┤  ├───────────────────────┤
  │ bee_template           │  │ bee_config     │  │ bee_logs              │
  │  - template render     │  │  - INI/YAML/ENV│  │  - multi-level log    │
  │  - HTML/JSON           │  │  - hot reload   │  │  - tracing integrate  │
  ├────────────────────────┤  ├────────────────┤  └───────────────────────┘
  │ bee_session            │  │ bee_cache      │
  │  - session management  │  │  - cache trait  │
  │  - Memory/Redis        │  │  - Mem/Redis    │
  └────────────────────────┘  └────────────────┘

  ┌─────────────────────────────────────────────────────────┐
  │                   Storage Engine Layer                   │
  ├──────────────────┬──────────────────────────────────────┤
  │ bee_kv           │  Redis / Memcached                   │
  │ bee_search       │  Elasticsearch + OpenSearch + ClickHouse │
  │ bee_graph        │  Neo4j + NebulaGraph + ArangoDB      │
  │ bee_tsdb         │  InfluxDB + Apache IoTDB + QuestDB   │
  └──────────────────┴──────────────────────────────────────┘
```

### Dependências entre crates

```
bee_config (sem dependências)
bee_logs   (sem dependências)
bee_cache  → bee_config
bee_kv     → bee_config
bee_session → bee_cache, bee_config
bee_template (sem dependências)
bee_orm    → bee_config, bee_cache
bee_search → bee_config
bee_graph  → bee_config
bee_tsdb   → bee_config
bee_router → bee_session, bee_template, bee_config, bee_logs
bee_cli    → bee_router, bee_orm
bee_rust   → todos os crates acima (re-export)
```

### Bancos de dados suportados

| Categoria | Banco de dados | Crate correspondente | Feature Flag |
|------|--------|-----------|-------------|
| **Relacional** | SQLite | `bee_orm` | `sqlite` (bee_rust: `orm-sqlite`) |
| | PostgreSQL | `bee_orm` | `postgres` / `postgres-tls` (bee_rust: `orm-postgres` / `orm-postgres-tls`) |
| | MySQL 5.7 / 8.0 / 8.4 / 9.x · MariaDB 10.11+ | `bee_orm` | `mysql` (bee_rust: `orm-mysql`) |
| | TiDB | `bee_orm` | `mysql` |
| **Tipos de campo** | Data / Decimal | `bee_orm` | `chrono` / `rust_decimal` (bee_rust: `orm-chrono` / `orm-rust_decimal`) |
| **KV / Cache** | Redis | `bee_kv` / `bee_cache` | `redis` / `redis-tls` / `redis-sentinel` / `redis-cluster` |
| | Memcached | `bee_kv` / `bee_cache` | `memcached` / `memcache` |
| **Busca / Análise** | Elasticsearch | `bee_search` | `elasticsearch` |
| | OpenSearch | `bee_search` | `opensearch` |
| | ClickHouse | `bee_search` | `clickhouse` |
| **Banco de grafos** | Neo4j | `bee_graph` | `neo4j` |
| | NebulaGraph | `bee_graph` | `nebulagraph` |
| | ArangoDB | `bee_graph` | `arangodb` |
| **Séries temporais** | InfluxDB | `bee_tsdb` | `influxdb` |
| | Apache IoTDB | `bee_tsdb` | `iotdb` |
| | QuestDB | `bee_tsdb` | `questdb` |

### Cadeia de filtros de requisição

```
Requisição → [SecurityFilter detecção de ataques] → [Restauração de Session] → [Validação de parâmetros] → [gancho prepare] → [tratamento handle] → [gancho finish] → Resposta
                  ↓ Qualquer etapa pode ser interrompida (como o Abort do Beego)
```

## Recursos

A visão geral dos recursos está abaixo; para o uso detalhado de cada módulo, exemplos de código e descrições da API, consulte a [Referência da API](api.pt.md).

### Núcleo Web (bee_router)

Controladores MVC: o trait `Controller` + `Context`, com suporte a namespaces de rotas, registro de métodos RESTful e cadeia de filtros de requisição.

- Saída de resposta: `ctx.json()` / `ctx.text()` / `ctx.html()`
- Redirecionamento / interrupção: `ctx.redirect()` / `ctx.abort()`
- Sessão e parâmetros: `ctx.session` / `ctx.params`
- Orçamento de latência (opcional): `Router::latency(LatencyConfig::new(cache))` concede a GET/HEAD um orçamento total padrão de 6 ms (4 ms de handler + 2 ms de reserva de fallback de cache); respostas públicas são armazenadas em cache e reproduzidas no timeout com `X-Bee-Cache: stale`, ou a requisição retorna 504. Escritas mantêm o timeout de 30 s; `LatencyStats` expõe P50/P99.

### Detecção de segurança (feature `security`)

Filtro de detecção de ataques baseado em [security-rust](https://crates.io/crates/security-rust), cobrindo 27 tipos de ataque, incluindo XSS, injeção de SQL, injeção de comandos e SSRF, ativado com uma linha:

```rust
let security = SecurityFilter::new();  // todos os 27 detectores ativados
```

### ORM (bee_orm)

Macro derivada `#[derive(Model)]` (com `#[bee(table / column / pk / auto / ignore / auto_now_add / soft_delete / shard_key)]`) + consultas encadeadas e execução com QuerySet (`all` / `one` / `count` / `exists` / `update` / `delete` / `filter_in`, agregados `sum` / `avg` / `min` / `max`) + migrações não destrutivas (`migrate::create_table` / `add_missing_columns` / `sync`) + leituras de relações (`belongs_to` / has_many `children*` / many-to-many `m2m`) + colunas JSON (`serde_json::Value`) + pools de conexões para os três backends (com transações e timeouts), com suporte a SQLite, PostgreSQL, MySQL e TiDB; sharding de banco/tabela com chave obrigatória (`ShardedPools` + `HashRouter` — `#[bee(shard_key)]` / `for_model` roteiam diretamente a partir de uma instância; a chave de shard deve ser fornecida pelo chamador — nunca um campo auto/carimbo de tempo — e NULL não é roteável; toda consulta deve levar uma chave de shard, e consultas e transações entre shards não são suportadas). Há suporte a MySQL 5.7–9.x e MariaDB 10.11+; `sync_all_parallel` migra todos os shards em paralelo, e `all_shards()` / `pools()` expõem os pools configurados para operações. Um `PoolStatus` / `pool_status()` compartilhado e o wrapper `TimedDb` de consultas lentas cobrem todos os backends; falhas de sharding têm seu próprio `OrmError::ShardError`, e `OrmError` é `#[non_exhaustive]`.

### Gerenciamento de configuração (bee_config)

Macro derivada `#[derive(Config)]`, com suporte a carregamento de INI / YAML / ENV e hot reload.

### Mecanismos de armazenamento

KV / Cache (Redis / Memcached), mecanismo de busca (Elasticsearch / OpenSearch / ClickHouse), banco de dados de grafos (Neo4j / NebulaGraph / ArangoDB) e banco de dados de séries temporais (InfluxDB / IoTDB / QuestDB) — todos com abstrações unificadas via trait, drivers compilados sob feature gate. Os backends Redis usam um pool de conexões real (`deadpool-redis`), e `bee_kv::RedisLock` fornece um lock distribuído (aquisição `SET NX PX` + liberação em Lua, espera limitada em `acquire` e renovação automática com `try_lock_with_watchdog`). Os pools Redis expõem `status()` / `RedisPoolStatus` e `RedisOptions` com tempo limite de resposta configurável; `redis-tls` habilita `rediss://` e credenciais ACL na URL; `redis-sentinel` / `redis-cluster` adicionam backends KV/cache Sentinel e Cluster. A primeira URL do Sentinel também fornece as credenciais ACL, o banco de dados, o protocolo e TLS da conexão com o master; as conexões do pool continuam sendo recicladas por PING sem verificação de papel, então um antigo master rebaixado que permaneça acessível pode continuar atendendo. Os comandos de Cluster não são limitados pelo tempo limite de resposta.

### Session, logs e templates

- Session: Memory, Redis (implementado) / Cookie, Database (planejados)
- Logs: log em múltiplos níveis + integração com tracing
- Templates: renderização baseada em tera

### Ferramenta CLI

```bash
bee-rust new my-app            # cria o scaffolding do projeto
bee-rust generate controller user
bee-rust run --watch           # execução de desenvolvimento (hot reload)
bee-rust pack                  # empacotamento para implantação
```

## Como usar

### Requisitos de ambiente

- Rust 1.80+
- Cargo

### Instalação

```bash
# Clonar o projeto
git clone https://github.com/erikwang2013/bee-rust.git
cd bee-rust

# Compilar
cargo build --workspace

# Executar os testes
cargo test --workspace
```

### Início rápido

```bash
# Criar um novo projeto com a CLI
cargo run -p bee_cli -- new hello
cd hello

# Executar o servidor de desenvolvimento
cargo run
```

### Usando no seu projeto

```toml
[dependencies]
bee_rust = { version = "1.6.1", features = ["full"] }
```

## Notas técnicas

### Stack de tecnologia

| Camada | Tecnologia |
|----|------|
| Base HTTP | axum 0.8 + tower 0.5 |
| Runtime assíncrono | tokio 1.x |
| Serialização | serde + serde_json |
| Mecanismo de templates | tera 1.x |
| Camada de logs | tracing + tracing-subscriber |
| CLI | clap 4 |
| Parsing de configuração | toml / serde_yaml / INI próprio |
| Tratamento de erros | thiserror |
| Macros processuais | syn + quote + proc-macro2 |

### Padrões de design

| Padrão | Aplicação |
|------|------|
| **Builder** | Logger, Router, QuerySet |
| **Abstração por trait** | Cache, KvStore, SearchEngine, GraphDB, TimeSeriesDB |
| **Macros derivadas** | `#[derive(Model)]`, `#[derive(Config)]` |
| **Feature Gate** | implementações de drivers compiladas sob demanda (redis, memcached, elasticsearch, etc.) |
| **Filter Chain** | cadeia de filtros de requisição, equivalente ao Filter do Beego |

### Lista de crates

| Crate | Função | Equivalente no Beego |
|-------|------|-----------|
| `bee_rust` | Meta crate, ponto de entrada unificado | — |
| `bee_router` | Roteamento + controladores + Context + filtros | `server/web`, `context` |
| `bee_orm` | ORM + QuerySet | `client/orm` |
| `bee_kv` | Abstração unificada KV/Cache | `client/cache` (estendido) |
| `bee_search` | Mecanismo de busca/análise | — (novo) |
| `bee_graph` | Banco de dados de grafos | — (novo) |
| `bee_tsdb` | Banco de dados de séries temporais | — (novo) |
| `bee_config` | Gerenciamento de configuração + hot reload | `client/config` |
| `bee_cache` | Abstração de cache | `client/cache` |
| `bee_session` | Gerenciamento de Session (Memory/Redis implementado; Cookie/Database planejado) | `server/web/session` |
| `bee_logs` | Logs | `logs` |
| `bee_template` | Renderização de templates | — (aprimorado) |
| `bee_cli` | Ferramenta CLI | ferramenta `bee` |

### Cobertura de testes

Os 475 testes de todo o repositório passam:

| Crate | Número de testes |
|-------|--------|
| bee_config | 8 |
| bee_cache | 27 |
| bee_template | 2 |
| bee_logs | 3 |
| bee_kv | 46 |
| bee_search | 21 |
| bee_graph | 19 |
| bee_tsdb | 19 |
| bee_orm | 211 |
| bee_orm_macro | 26 |
| bee_session | 2 |
| bee_router | 61 |
| bee_cli | 30 |

## Apoie o projeto

Se este projeto foi útil para você, fique à vontade para escanear o QR code e fazer uma doação de apoio. Obrigado!

**WeChat Pay**

<img src="weixinpay.png" width="160" height="175" alt="WeChat Pay">

**Alipay**

<img src="alipay.png" width="160" height="175" alt="Alipay">

### Licença

Apache-2.0
