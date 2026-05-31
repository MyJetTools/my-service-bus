# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Layout

This repo root contains two separate Rust crates (it is **not** a single Cargo workspace):

- `my-service-bus/` — the main broker node (`my-service-bus-main-node`, the binary). All backend work happens here.
- `my-service-bus-ui/` — a Dioxus 0.7 WASM SPA that is the web dashboard. It is built separately and its output is copied into `my-service-bus/wwwroot/`, which the node serves via `StaticFilesMiddleware`.
- `docs/` — design notes; see `docs/HOW_IT_WORKS.md` (GC layers) and `docs/gc-recovery.md`.

## Build / Run / Test

All `cargo` commands for the broker must run from inside `my-service-bus/`:

```bash
cd my-service-bus
cargo build --release          # build the node
cargo run --release            # run the node (needs persistence service + settings, see below)
cargo test                     # run all tests
cargo test <name>              # run a single test by substring match
```

The UI is built from `my-service-bus-ui/` with the Dioxus CLI. Use the provided script — it builds the SPA and copies the bundle (including raw `app.css`/`styled.css`) into the node's `wwwroot/`, which is the only step needed to publish a UI change:

```bash
cd my-service-bus-ui
./build.sh                     # dx build --release --platform web, then copy to ../my-service-bus/wwwroot
```

### Runtime prerequisites

- The `my-service-bus-persistence` service must already be running — the node persists messages to it over gRPC and will try to connect on startup.
- A settings file is required. Its path comes from `get_settings_filename()` in `src/settings.rs` (resolves under `HOME`, file `.myservicebus`). It is **YAML** (despite the README's older C#-style example). Keys map to `SettingsModelYaml`: `persistence_grpc_url`, `queue_gc_timeout`, `debug_mode`, `max_delivery_size`, `delivery_timeout` (optional), `auto_create_topic_on_publish` (optional), `auto_create_topic_on_subscribe` (optional), `listen_unix_socket` (optional).

### Ports

- TCP binary protocol: `6421`
- HTTP (REST API, Swagger, web UI, `/mcp`): `6123`
- Optional Unix socket: `listen_unix_socket` setting

### Proto files

`build.rs` downloads the persistence proto from a remote GitHub repo (`my-sb-proto-files`) at build time via `ci_utils::sync_and_build_proto_file`, then `tonic` generates the `persistence` module (`tonic::include_proto!("persistence")` in `main.rs`). Local `proto/` files mirror the contracts. Network access is needed for a clean build.

## Architecture

### Core domain model (everything hangs off `AppContext`)

`AppContext` (`src/app/app_ctx.rs`) is the single shared `Arc` passed everywhere. Key fields: `topic_list`, `sessions`, `persistence_client`, `persist_executor` (a `BackgroundExecutor`), `prometheus`, `restore_page_scheduler`, `settings`, `states` (`AppStates` from rust-extensions, drives init/shutdown).

The data hierarchy:

- **Topic** (`src/topics/`) — a channel messages are published to. `Topic` wraps a `parking_lot::Mutex<TopicInner>`; **all topic state is behind that one mutex**. Access it via `topic.get_access()` (returns `TopicDataAccess`) or `topic.get_topic_info(|inner| ...)`. `TopicInner` owns the queues, publishers, the message id counter, and the in-memory message pages. Do not hold the lock across `.await`.
- **Queue** (`src/queues/`) — a subscription group on a topic. Queue types: permanent, temporary (deleted on disconnect), single-connection. Tracks per-queue delivery cursors and `delivery_bucket`s.
- **Messages paging** (`src/messages_page/`, `src/sub_page/`) — messages are stored in memory in pages subdivided into sub-pages (`SubPage`/`SubPageId`). Pages are loaded on demand from persistence and garbage-collected when no longer referenced. `MessagesPageList` holds them.
- **QueueSubscribers** (`src/queue_subscribers/`) — live subscribers attached to a queue, each with a delivery cursor and id from `SubscriberIdGenerator`.
- **Sessions** (`src/sessions/`) — connected clients (`SessionsList`), split into `tcp/` and `http/` variants; carry connection metrics.

### Operations layer

`src/operations/` holds the use-case functions that mutate the domain: `publisher.rs`, `subscriber.rs`, `delivery/`, `delivery_confirmation.rs`, `create_topic_if_not_exists.rs`, `delete_topic.rs`, `persist_*.rs`, `initialization.rs` (startup restore, spawned from `main`), `page_loader/`. Prefer adding logic here rather than in protocol handlers.

### Protocol surfaces (entry points)

- **TCP** (`src/tcp/`) — `TcpServerEvents` (`socket_events.rs`) handles the binary MySB protocol via `my-tcp-sockets` + `MySbSerializerFactory` from the `my-service-bus` SDK. Same handler is reused for the Unix socket server.
- **HTTP** (`src/http/`) — `start_up::setup_server` wires up middlewares in order: Swagger, `AuthMiddleware` (`src/http/auth/`), MCP middleware, the controllers, then static files (SPA). REST controllers live under `src/http/controllers/` (publisher, subscribers, topics, queues, sessions, status, prometheus, logs, debug, greeting).
- **MCP** (`src/mcp/`) — read-only MCP server mounted at `/mcp` (built in `mcp::build_middleware`). Tools: overview, list topics, get topic, list sessions, get message. Register new tools in `src/mcp/mod.rs`.

### Persistence client

`src/grpc_client/` — `PersistenceGrpcService` is the gRPC client to `my-service-bus-persistence` (built with `my-grpc-extensions`). `create_production_instance` is used in `main`; `messages_pages_mock_repo.rs` backs tests.

### Background jobs

Started in `main.rs` using `rust_extensions::MyTimer` (note timers carry a ~1 minute task timeout) and the `persist_executor` `BackgroundExecutor`:

- `MetricsTimer` (1s) — collects metrics incl. TCP/HTTP connection counts and thread stats.
- `GcTimer` + `DeadSubscribersKickerTimer` (3s) — message- and page-level GC (see `docs/HOW_IT_WORKS.md`) and dead-subscriber kicking.
- `GcDeletedTopicsTimer` (60s).
- `PersistJob` (on `persist_executor`) — flushes queued messages to persistence.
- `RestoreSubPagesEventLoop` (non-test only) — restores message pages from persistence on demand.

Shutdown is cooperative: `app.states.wait_until_shutdown()` then `app::shutdown::execute` drains messages (1s grace).

## Conventions

- **Locking**: topic state lives behind a single `Mutex<TopicInner>`; go through `Topic`'s accessor methods. `arc-swap` and `parking_lot` are used for low-contention shared state. Never `.await` while holding a sync mutex.
- **Dependencies**: most MyJetTools crates are pinned to git tags in `Cargo.toml` (e.g. `my-service-bus` SDK, `my-http-server`, `my-tcp-sockets`, `rust-extensions`, `my-grpc-extensions`). Consult the development-best-practices MCP docs before using these APIs — they evolve and signatures change between tags.
- **Allocator**: `mimalloc` is the global allocator (set in `main.rs`).
- **Versioning**: `APP_VERSION` comes from `CARGO_PKG_VERSION` (the crate version in `my-service-bus/Cargo.toml`); bump the changelog in `README.md`.
- Tests use `#[cfg(test)]` mocks (`src/test_tools.rs`, `SubPageLoaderSchedulerMock`, the grpc mock repo) so the node can run without the persistence service.
