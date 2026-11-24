# MY SERVICE BUS

## Overview

**My Service Bus** is a high-performance, distributed message broker and service bus written in Rust. It provides a pub/sub messaging system with persistent message storage, designed for building scalable microservices architectures.

### Core Functionality

- **Topics & Queues**: Organizes messages into topics (channels) and queues (subscription groups)
- **Publish/Subscribe**: Publishers send messages to topics; subscribers consume messages from queues
- **Message Persistence**: Messages are persisted to a separate persistence service via gRPC for durability
- **Multiple Queue Types**: Supports permanent queues, temporary queues (delete on disconnect), and single-connection queues
- **Dual Protocol Support**: 
  - TCP server (port 6421) for high-performance binary protocol communication
  - HTTP server for REST API access and web-based management UI
  - Optional Unix socket support for local inter-process communication
- **Message Delivery**: Reliable message delivery with delivery confirmation, retry mechanisms, and dead subscriber detection
- **Background Processing**: Automated garbage collection, message persistence, metrics collection, and health monitoring
- **Web UI**: Built-in web interface for monitoring topics, queues, sessions, and system metrics
- **Prometheus Integration**: Exposes metrics for monitoring and observability
- **Message Paging**: Efficient memory management through message pagination and sub-page organization

### Architecture

The application consists of:
- **Main Node** (`my-service-bus`): Handles message routing, delivery, and client connections
- **Persistence Service** (`my-service-bus-persistence`): Separate service for durable message storage (must be running before starting the main node)

### Key Features

- High-throughput message processing
- At-least-once delivery guarantees
- Automatic message persistence with configurable delays
- Session management for TCP and HTTP connections
- Dead subscriber detection and cleanup
- Graceful shutdown with message drain
- Real-time metrics and monitoring
- Compressed message storage option

## Run  

You should run my-service-bus-persistence before running my-service-bus

Enusure that environment variable "**HOME**" exists.
It should point to location with **.myservicebus** file!

**.myservicebus** content:
`
GrpcUrl: http://127.0.0.1:7124 // my-service-bus-persistence should run on this url
EventuallyPersistenceDelay: 00:00:05
QueueGcTimeout: 00:00:20
DebugMode: true
MaxDeliverySize: 4194304
`

Install rust: https://www.rust-lang.org/tools/install
execute: **cargo run --release**

`listen_ip` (optional) is the address the TCP port (6421) and the HTTP port (6123) bind to;
absent, `0.0.0.0`. **Neither port asks who is calling** - publishing, subscribing, deleting a
topic or a queue, moving a queue's message id, reading messages over `/mcp` and opening the MCP
write window are open to whoever reaches them. `127.0.0.1` keeps the node to its own machine.

## Running on Windows

The node builds and runs on Windows (`x86_64-pc-windows-msvc`), in the console or as a native
Windows service - the command line and the install checks of my-reverse-proxy's Windows service:

```powershell
.\my-service-bus-main-node.exe --settings C:\my-sb\bus\config\settings.yaml   # the console
.\my-service-bus-main-node.exe --install --settings C:\my-sb\bus\config\settings.yaml
.\my-service-bus-main-node.exe --start      # --uninstall to remove it
```

- `--settings <file>` is read instead of `%HOME%\.myservicebus` (HOME, or USERPROFILE, at run
  time - it used to be the HOME of the machine the binary was built on); its folder is what `~`
  stands for.
- `--install` registers the service `my-service-bus`: auto-start, restarted 10 s after a failure,
  running as `NT SERVICE\my-service-bus` - an account with the rights of a plain user, given the
  settings folder and the exe folder to read and `logs\` of the settings folder to write. It is
  refused when a folder of the exe or of the settings - or one above them - can be changed by an
  account other than the system and the administrators; the message gives the `icacls` commands
  which close it.
- A service has no console: what the node prints goes to `logs\my-service-bus.log` next to the
  settings (the previous start's as `.prev.log`).
- Stopping the service is the graceful shutdown: the node hands what it still holds to the
  persistence (at most 20 s per topic when the persistence does not answer), then saves the
  topics and queues.
- A release build does not run under a debugger and closes its process to everybody but the
  system, the administrators and its own account - the messages are in its memory.
- There are no unix sockets on Windows: `listen_unix_socket` is ignored, and says so.

`tools\windows\install.ps1` (elevated) puts the node and my-service-bus-persistence on one
machine together: closed folders, both services, the bus depending on the persistence, both on
`127.0.0.1`, an inbound firewall block on their ports. Run again, it replaces the exes.
`tools\windows\uninstall.ps1` removes the services and keeps the data.


## Changes
### 2.2.4
* Grpc Client now have timeouts
* Backgrounds are implemented using timers which means now they have one minute timeout in case of long running tasks;
* Added Metric - topic size in memory
* Highlited PageId within MessageID on UI

### 2.2.5
* Pages Support
* GC works as fast as it can
* Added Visualisation - how many messages are on the delivery
* UI Shows amount of Sessions
* Bug Fixed - immediate persistence made to send a lot of data to console.

### 2.2.6
* Immediately persist case is signle threaded
* Added ability to send messages to persist uncompressed way (Settings Parameter PersistCompressed)
* BugFIX: When we delete a queue - we remove topic_queue_size from prometheus

### 2.2.7-rc01
* Updated Library versions


### 2.2.7
* TCP ReadLoop now has timeout as well as write loop

### 2.2.8
* GRPC Optimization
* Libraries Updates

### 2.4.0
* Namespaces. Every topic now lives inside a namespace; a topic name is unique only within one, and namespaces share nothing.
* A client which names no namespace works in `default` — the pre-namespace behaviour, byte for byte on the wire.
* TCP: new `SetNamespace` packet (id 16). It is sent only when the namespace is not `default`, and is refused once the connection has published or subscribed.
* HTTP: publishers and subscribers take the namespace from the session fixed at `/Greeting`; the admin/read surface takes it from the `ns` header (with a `?ns=` fallback). New `GET /api/Namespaces/List`.
* Persistence: `Namespace` is sent in every gRPC request and in each topics-snapshot record. The default namespace is sent as an absent field, so an un-upgraded persistence keeps working.
* Prometheus topic metrics gained a `namespace` label; MCP tools gained an optional `namespace` argument; the sessions list reports the namespace of each connection.
* UI: namespace selector in the topbar (shown once more than one namespace exists), remembered in localStorage.
* MCP write tools are gated behind an enable window. `mysb_set_topic_persist`, `mysb_delete_queue` and `mysb_delete_topic` are refused unless a human opens it; the window lasts 10 minutes, pressing Enable again while it is open adds another 10, and a node restart always leaves it closed.
* New MCP tools: `mysb_delete_queue` (irreversible) and `mysb_delete_topic` (soft delete, `hard_delete_after_seconds` defaults to 24h, restorable until then). Both report what the queue/topic still held at the moment of deletion.
* New endpoints `GET`/`POST /api/Mcp/Writes`; `/api/Status` carries `mcpWritesRemainingSecs`, and the UI status bar gained an `MCP W` switch next to the DEBUG badge.
