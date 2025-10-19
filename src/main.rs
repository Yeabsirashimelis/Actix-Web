use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

/*
/*
 Notice that some of these handlers have routing information attached directly using the
 built-in macros. These allow you to specify the method and path that the handler should respond to.
  You will see below how to register manual_hello (i.e. routes that do not use a routing macro).
*/

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
 */

/////////////////////////////////////////////////////////////////
/*
   Actix Web: App & Scopes
1, App Instance
    This is the central object for your server.
It holds:
    Registered routes
    Middleware
    Shared application state
Every request is evaluated against the App’s routing table.

2, Scopes (App::scope)
    A scope is like a namespace for routes.
*/

async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            // prefixes all resources and routes attached to it
            web::scope("/api")
                // ...so this handle requests fri "get /app/index.html"
                .route("index", web::get().to(index)),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
