# Webby

Webby is an asynchronous backend sandbox built to master building web services in Rust using Axum, Tokio, and Tower. It serves as a hands-on laboratory for managing shared state, integrating databases, and writing custom type-safe middleware.

## The Tech Stack

* **Language & Framework**: Rust + Axum
* **Runtime & Ecosystem**: Tokio + Tower + Tower-HTTP
* **Database & Configuration**: Toasty ORM (SQLite) + Dotenvy

---

## Core Learning Highlights

* **State & DB Management**: Thread-safe database connection sharing via `Arc<AppState>` using the **Toasty** ORM against SQLite.
* **JWT Authentication**: Custom `FromRequestParts` extractor to decode, validate, and secure routes using JSON Web Tokens (HS256).
* **Traffic Control & Middleware**: Structured logging (`TraceLayer`), request timeouts (`TimeoutLayer`), concurrency bounds (`ConcurrencyLimitLayer`), and rate limiting (`GovernorLayer`).
* **Input Validation & Errors**: Chaining the `validator` crate into the Axum pipeline and transforming internal app logic into structured HTTP responses via custom `IntoResponse` enums.
* **SPA Routing**: Serving physical assets with `ServeDir` and catching unmatched traffic with an `index.html` fallback.
* **Graceful Shutdowns**: Listening for cross-platform OS signals (`SIGINT`/`SIGTERM`) to drop the runtime safely.

---

## API Endpoints Matrix

| Method | Endpoint | Description | Auth / Extractors / Middleware |
| :--- | :--- | :--- | :--- |
| **GET** | `/` | Root Index | None |
| **GET** | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| **POST**| `/login` | Authenticate user and issue JWT | `Json<AuthPayload>` |
| **GET** | `/users` | User section about (2s delay) | Concurrency Limited (Max 5) |
| **GET** | `/users/list` | Asynchronously fetch all users | **Requires JWT (`Claims`)** |
| **POST**| `/users/create` | Validate and insert new user | `Json<CreateUser>`, Concurrency Limited |
| **PATCH**| `/users/update/{id}`| Update user profile | **Requires JWT (`Claims`)**, `Path<u64>`, `Json<UpdateUser>` |
| **DELETE**| `/users/delete/{id}`| Remove a specific user by ID | **Requires JWT (`Claims`)**, `Path<u64>`, Concurrency Limited |
| **GET** | `/users/greet/{name}`| Dynamic path injection | `Path<String>`, Concurrency Limited |
| **ANY** | `/assets/*` / Fallback| Static asset server / SPA catch-all | `ServeDir` + `ServeFile` |

---

## Getting Started

### 1. Setup Environment
```env
JWT_SECRET=your_super_secret_key_here
```

### 2. Run the Server
```bash
# Automatically builds SQLite schema and listens at http://localhost:3000
cargo run
```
### 3. Run Test
```bash
JWT_SECRET=test_secret cargo test
```
