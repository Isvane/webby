# Webby

Webby is a learning-focused backend sandbox built to learn asynchronous web services in Rust using Axum, Tokio, and Tower. 
Instead of reading tutorials passively, this repository serves as a hands-on laboratory to transition from basic Rust syntax to building actual backend systems. It documents the journey of learning how to manage asynchronous shared state, integrate databases, handle type-safe routing, and configure web middleware.

## Core Learning Features

* **Learning Asynchronous Shared State & Database Integration**: Moving from basic in-memory state to managing thread-safe database connection handles across asynchronous worker threads using `Arc<AppState>` coupled with the **Toasty** ORM executing queries against an in-memory SQLite instance.
* **Understanding Middleware and Layers**: Implementing production-style diagnostics by configuring Tower and Tower-HTTP. This includes `TraceLayer` for structured logging with `tracing-subscriber` and `TimeoutLayer` to automatically drop requests that exceed 10 seconds.
* **Type-Safe Routing and Extraction**: Learning how Axum utilizes extractors to automatically parse incoming request data, specifically using `Path` for URL variables, `Query` for URL pagination parameters, and `Json` for request bodies.
* **Input Validation**: Discovering how to chain the `validator` crate into the Axum pipeline to check string constraints and email formats before the handler logic even runs.
* **Declarative Error Handling**: Implementing custom `IntoResponse` traits for enums to transform internal application logic and errors directly into structured HTTP Status Codes like 200 OK, 201 Created, 400 Bad Request, 404 Not Found, and 422 Unprocessable Entity.
* **Static Files and Fallbacks**: Learning how to serve physical assets from the file system using `ServeDir` and configuring fallback routes to redirect unmatched traffic to a single-page-application `index.html`.
* **Graceful Shutdowns**: Writing cross-platform logic that listens for OS signals like SIGINT and SIGTERM to stop the Tokio runtime cleanly without terminating active client connections abruptly.

## The Tech Stack

* **Language**: Rust
* **Web Framework**: Axum (Routing and request handling)
* **Async Runtime**: Tokio (Asynchronous execution foundation)
* **Database / ORM**: Toasty (with an in-memory SQLite backend)
* **Middleware**: Tower and Tower-HTTP (Service abstractions, logging, and timeouts)
  
## API Endpoints Matrix

| Method | Endpoint | Description | Extractors Used |
| :--- | :--- | :--- | :--- |
| GET | / | Root Index | None (Returns 202) |
| GET | /pages | Query-driven list pagination | `Query<Pagination>` |
| GET | /users/ | User section introduction | None |
| GET | /users/list | Asynchronous retrieval of all users from DB | `State<Arc<AppState>>` |
| POST | /users/create | Validated user payload submission and insertion | `State<Arc<AppState>>`, `Json<CreateUser>` |
| DELETE| /users/delete/{id} | Database removal of a specific user by ID | `State<Arc<AppState>>`, `Path<u64>` |
| GET | /users/greet/{name}| Dynamic path variable injection | `Path<String>` |
| ANY | /assets/* | Static file server delivery | `ServeDir` |

## Learning Objectives Achieved

This project was built to bridge the gap between finishing introductory materials like Rustlings and writing real-world services. Key takeaways included:
* Learning how to satisfy the borrow checker when passing application state across async boundaries using atomic reference counting (`Arc`).
* Interfacing an async web server with an ORM (`toasty`), schema initialization (`push_schema`), and performing asynchronous CRUD operations.
* Discovering how Tower acts as the underlying service abstraction beneath Axum routers and middleware layers.

## Getting Started

Ensure you have the Rust toolchain installed, then spin up the server locally:

```bash
# The server will initialize, build the SQLite schema, and listen at http://localhost:3000
cargo run
```
