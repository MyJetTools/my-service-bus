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