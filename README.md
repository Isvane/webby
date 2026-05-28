# Webby

Webby is a learning-focused backend sandbox built to learn asynchronous web services in Rust using Axum, Tokio, and Tower. 
Instead of reading tutorials passively, this repository serves as a hands-on laboratory to transition from basic Rust syntax to building actual backend systems. It documents the journey of learning how to manage asynchronous shared state, integrate databases, handle type-safe routing, and configure web middleware.

## Core Learning Features

* **Learning Asynchronous Shared State & Database Integration**: Moving from basic in-memory state to managing thread-safe database connection handles across asynchronous worker threads using `Arc<AppState>` coupled with the **Toasty** ORM executing queries against a local SQLite instance.
* **Understanding Middleware and Layers**: Implementing production-style diagnostics and traffic control by configuring Tower and Tower-HTTP. This includes `TraceLayer` for structured request logging, `TimeoutLayer` to drop requests exceeding 10 seconds, and a localized `ConcurrencyLimitLayer` to bound simultaneous requests.
* **Type-Safe Routing and Extraction**: Learning how Axum utilizes extractors to automatically parse incoming request data, specifically using `Path` for URL variables, `Query` for URL pagination parameters, and `Json` for request bodies.
* **Input Validation**: Discovering how to chain the `validator` crate into the Axum pipeline to check string constraints and email formats before the handler logic runs.
* **Declarative Error Handling**: Implementing custom `IntoResponse` traits for enums to transform internal application logic and database errors directly into structured HTTP Status Codes (e.g., 200 OK, 201 Created, 400 Bad Request, 404 Not Found, and 422 Unprocessable Entity).
* **Static Files and Fallbacks**: Learning how to serve physical assets from the file system using `ServeDir` and configuring fallback services to cleanly redirect unmatched traffic to a Single Page Application (SPA) entrypoint (`index.html`).
* **Graceful Shutdowns**: Writing cross-platform logic that listens for OS signals like SIGINT (Ctrl+C) and SIGTERM (Unix process termination) to stop the Tokio runtime cleanly without dropping active client connections abruptly.

## The Tech Stack

* **Language**: Rust
* **Web Framework**: Axum (Routing and request handling)
* **Async Runtime**: Tokio (Asynchronous execution foundation)
* **Database / ORM**: Toasty (An experimental ORM configured with a persistent SQLite backend)
* **Middleware**: Tower and Tower-HTTP (Service abstractions, concurrency limiting, logging, and timeouts)
  
## API Endpoints Matrix

| Method | Endpoint | Description | Extractors / Middleware Used |
| :--- | :--- | :--- | :--- |
| GET | `/` | Root Index (Returns 202 Accepted) | None |
| GET | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| GET | `/users/` | User section introduction (with 2s artificial delay) | Concurrency Limited (Max 5) |
| GET | `/users/list` | Asynchronous retrieval of all users from DB | `State<Arc<AppState>>`, Concurrency Limited |
| POST | `/users/create` | Validated user payload submission and insertion | `State<Arc<AppState>>`, `Json<CreateUser>`, Concurrency Limited |
| DELETE| `/users/delete/{id}` | Database removal of a specific user by ID | `State<Arc<AppState>>`, `Path<u64>`, Concurrency Limited |
| GET | `/users/greet/{name}`| Dynamic path variable injection | `Path<String>`, Concurrency Limited |
| ANY | `/assets/*` | Static file server delivery from `./public` | `ServeDir` |
| ANY | * (Fallback) | Catches all unmatched paths | `ServeDir` + `ServeFile("public/index.html")` |

## Learning Objectives Achieved

This project was built to bridge the gap between finishing introductory materials like Rustlings and writing real-world services. Key takeaways included:
* Learning how to satisfy the borrow checker when passing application state across async boundaries using atomic reference counting (`Arc`).
* Interfacing an async web server with an ORM (`toasty`), automating directory creation, managing schema migrations (`push_schema`), and performing safe asynchronous CRUD operations.
* Discovering how Tower acts as the underlying service abstraction beneath Axum routers, allowing layers to cleanly wrap inner routes.

## Getting Started

Ensure you have the Rust toolchain installed, then spin up the server locally:

```bash
# The server will initialize directories, sync the SQLite schema, and listen at http://localhost:3000
cargo run
