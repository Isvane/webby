# Webby

An asynchronous backend sandbox built to master production-grade web services in `Rust`. 

## Why Webby?
Reading documentation only takes you so far. `Webby` is a hands-on laboratory designed to bridge the gap between "Hello World" tutorials and production realities. It serves as a playground to wrestle with `async` state, write type-safe middleware, and understand how the `Tokio` and `Tower` ecosystem handles traffic under the hood.

---

## Core Learning Takeaways

* **Thread-Safe State & Static Keys:** Sharing database connection pools via `Arc<AppState>` combined with modern `std::sync::LazyLock` for zero-overhead, safe lazy initialization of cryptographic keys.
* **Type-Safe Extraction & RBAC:** Leveraging `axum_extra::TypedHeader` to extract Bearer tokens directly from request parts, parsing them into verifiable `Claims` before routes are executed, paired with granular role-matching middleware (`require_role`).
* **Socketless Integration Testing:** Testing the entire HTTP pipeline natively in-memory without binding to a physical TCP port. Uses `tower::Service` utilities (`oneshot`/`call`) combined with an ephemeral `sqlite::memory:` schema instantiation for instantaneous, deterministic testing.
* **Defensive Traffic Control:** Layering Tower middleware to handle global rate-limiting (`GovernorLayer`), request timeouts (`TimeoutLayer`), and concurrency limits.
* **Declarative Payload Validation:** Binding the `validator` crate directly to incoming deserialization pipelines to sanitize names, string boundaries, and email patterns before hitting domain logic.

---

## API Endpoints Matrix

| Method | Endpoint | Description | Auth / Extractors / Middleware |
| :--- | :--- | :--- | :--- |
| **GET** | `/` | Root Index | None |
| **GET** | `/pages` | Query-driven list pagination | `Query<Pagination>` |
| **POST**| `/login` | Authenticate user and issue JWT | `Json<AuthPayload>` |
| **GET** | `/users/` | User section about | Concurrency Limited (Max 5) |
| **POST**| `/users/create` | Validate and insert new user | `Json<CreateUser>`, Concurrency Limited (Max 5) |
| **PATCH**| `/users/update/{id}` | Update user profile | **Requires JWT (`Claims`)**, `Path<u64>`, `Json<UpdateUser>`, Concurrency Limited (Max 5) |
| **DELETE**| `/users/delete/{id}` | Remove a specific user by ID | **Requires JWT (`Claims`)**, `Path<u64>`, Concurrency Limited (Max 5) |
| **GET** | `/users/greet/{name}` | Dynamic path injection | `Path<String>`, Concurrency Limited (Max 5) |
| **GET** | `/admin/list` | Asynchronously fetch all users | **Requires JWT (`Claims`)**, `Query<Pagination>`, Admin Role Middleware |
| **PATCH**| `/admin/{id}/role` | Modify a user's access role level | **Requires JWT (`Claims`)**, `Path<u64>`, `Json<ChangeRolePayload>`, Admin Role Middleware |
| **ANY** | `/assets/*` / Fallback | Static asset server / SPA catch-all | `ServeDir` ("public") + `ServeFile` ("public/index.html") |

> **Global Middleware Layers Applied:** 
> * **Rate Limiting:** `GovernorLayer` (2 req/sec, burst size 5) using client IP tracking.
> * **Timeouts:** `TimeoutLayer` enforcing a strict 10-second request termination limit.
> * **Observability:** `TraceLayer` capture via `tracing` for structured HTTP request metrics.

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
cargo test
```
