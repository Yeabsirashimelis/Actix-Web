🦀 Learn Actix Web Framework — Core Concepts

This repository is a structured walkthrough of Actix Web, one of the fastest and most powerful web frameworks in Rust.
It contains well-documented examples demonstrating core server concepts, state management, routing, guards, configuration, multi-threading, and more.

 Overview
Each section in the code explains a fundamental Actix Web feature with practical examples and detailed comments.
This is ideal for learners who want to understand how Actix Web works under the hood — not just how to write routes.

 Topics Covered
1, Basic Routing
Learn to define routes using:
#[get("/")], #[post("/echo")] macros
Manual route registration with .route()
Using impl Responder for flexible return types

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

2, Application Scopes (App and web::scope)
Scopes group routes under common prefixes — like namespaces for APIs.
App::new().service(
    web::scope("/api")
        .route("index", web::get().to(index)),
);

3, Application State (web::Data)
Learn how to:
Store global data (like app name)
Share state safely across routes and threads using Arc + Mutex
struct AppState {
    app_name: String,
}

4, Shared Mutable State
Actix creates one App per thread.
To share data between them, use web::Data (Arc) and optionally Mutex for mutation.
struct AppStateWithCounter {
    counter: Mutex<i32>,
}

5, Application Scopes & Composition
Use web::scope() to group routes logically, similar to express.Router() in Node.js.
let scope = web::scope("/users").service(show_users);
App::new().service(scope);

6, Route Guards & Virtual Hosting
Guards let routes respond only under certain conditions (e.g., host header, method, headers).
web::scope("/")
    .guard(guard::Host("www.rust-lang.org"))
    .route("", web::to(|| async { HttpResponse::Ok().body("www") }));

7, Configuration & Modular Setup
Organize routes into reusable modules using .configure().
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app").route(web::get().to(|| async { "app" })),
    );
}

8, Multi-Threading
Each Actix Web server spawns multiple worker threads (default = CPU cores).
You can set it manually with .workers(4).
Avoid blocking calls (std::thread::sleep) — use async equivalents like:
tokio::time::sleep(Duration::from_secs(5)).await;

9, Keep-Alive Connections
Configure how long HTTP connections stay open using .keep_alive():
HttpServer::new(app)
    .keep_alive(Duration::from_secs(75)); // 75s timeout

You can also explicitly close connections with:
resp.head_mut().set_connection_type(http::ConnectionType::Close);

10, Path Extractors (web::Path)
Extract dynamic route segments as tuples or structs (with serde::Deserialize).
Tuple Example:
#[get("/users/{user_id}/{friend}")]
async fn index(path: web::Path<(u32, String)>) -> Result<String> {
    let (user_id, friend) = path.into_inner();
    Ok(format!("Welcome {}, user_id {}!", friend, user_id))
}

Struct Example:
#[derive(Deserialize)]
struct Info { user_id: u32, friend: String }
#[get("/users/{user_id}/{friend}")]
async fn index(info: web::Path<Info>) -> Result<String> {
    Ok(format!("Welcome {}, user_id {}!", info.friend, info.user_id))
}

 Key Takeaways
App = main entry point for routes and state
web::scope = organize APIs by prefix
web::Data = share state safely across workers
guard = conditionally match routes
configure = modularize setup
Avoid blocking threads — use async/await
Control connection lifetime with Keep-Alive

 Run the Examples
  Requirements
    Rust
    Cargo
    Actix Web 4.x

Run
cargo run // but before running the code uncomment the sectoion of code you wanna learn and explore

Then open in your browser or any other API tester (CLI or GUI):
http://127.0.0.1:8080

 📚 References
   Actix Web Documentation - https://actix.rs/docs/
   Actix Web on docs.rs - https://docs.rs/actix-web/latest/actix_web/
   Tokio Async Runtime - https://tokio.rs/
   Serde — Serialization Framework - https://serde.rs/
   The Rust Programming Language Book - https://doc.rust-lang.org/book/

Author
 Yeabsira Shimelis
   Learning Rust backend development and Actix Web fundamentals.
