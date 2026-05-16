# Webby

`Webby` is a high-performance backend sandbox built to master asynchronous web services using **Axum** and the **Tokio** runtime in Rust. 
Rather than building a generic tutorial app, this repository serves as a laboratory for implementing backend patterns, concurrent state management, and system infrastructure.

## Features

*   **Asynchronous Shared State:** Implemented thread-safe, in-memory user data management using concurrent state patterns across asynchronous worker threads.
*   **Production-Grade Middleware:** Integrated request tracing and diagnostics utilizing `tower-http` and `TraceLayer` for structured logging.
*   **Type-Safe Routing:** Designed robust, nested REST API endpoints with strict JSON payload extraction and type-safe `ApiResponse` enums.
*   **Infrastructure Resilience:** Engineered cross-platform graceful shutdown logic to capture system signals (SIGINT/SIGTERM) and cleanly drain active connections.
*   **Automated Testing:** Built a decoupled unit-testing suite (`test.rs`) to validate API endpoint behaviors and status codes.

## The Tech Stack

*   **Language:** Rust
*   **Web Framework:** Axum
*   **Async Runtime:** Tokio
*   **Observability:** Tower-HTTP (Tracing/Logging)

## Learning Objectives Achieved

This project was built to transition theoretical knowledge from *Rustlings* (specifically smart pointers, and concurrency) into a practical, bulletproof backend service. Key takeaways included managing the borrow checker across async boundaries and handling thread synchronization without data races.

## Getting Started

Ensure you have the Rust toolchain installed, then spin up the server locally:

```bash
# The server will initialize and listen at http://localhost:3000
cargo run
