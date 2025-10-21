use std::{sync::Mutex, time::Duration};

use actix_web::{
    get, guard, http::{self, KeepAlive}, post, web::{self, head}, App, HttpRequest, HttpResponse, HttpServer, Responder
};

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
 */

////////////////////////////////////////////////////////

/*
/*
State
 application state is shared with all routes and resources within the same scope. state can be
  accesses with the web::Data<T> extractor where T is the type of the state. State is also accessible
   for middleware
*/

//LET'S WRITE A SIMPLE APPLICATION AND STORE THE APPLICATION NAME IN THE STATE

// this struct represents state
struct AppState {
    app_name: String,
}

#[get("/")]
async fn index(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name;
    format!("Hello {}!", app_name) // response with app_name
}

// NEXT PASS IN THE STATE WHEN INITIALIZING THE APP AND START THE APPLICATION
//    any number of state types could be registered within the application
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix Web"),
            }))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
 */

///////////////////////////////////////////////////////////////////
/*
/*
  Shared Mutable State
  HttpServer accpets an application factory rather than an application instance.
  An httpServer constructs an application instance for each thread. threfore application data must be
   constructed multiple times. if you want to share data between different threads, a shareable object
   should be used, eg: Send + Sync

  internally, web::Data uses Arc. So inorder to avoid creating two ArcS, we shoulf create
   our data before regsitering it using App::app_data()

   Why We Use Arc / web::Data in Actix Web

By default, HttpServer creates one App per thread (usually one per CPU core).
So if we put data directly in App::new(), it would be copied (or re-created) for every thread.

    - To avoid copying and instead share a single instance of that data:
    We create the state once (outside the server factory).
    Then we wrap it in web::Data, which internally uses Arc.
    - Arc (Atomic Reference Counted pointer):
        Allows multiple threads to share the same data.
        Cloning it is cheap — it just increments the reference count.
        The actual data is stored only once in memory.
        Safe for concurrent access if combined with Mutex or RwLock.

🧠 In short:
We use web::Data (i.e. Arc<T>) so all worker threads share one piece of application state, without making multiple copies.
*/

struct AppStateWithCounter {
    counter: Mutex<i32>, // mutex is necessary to mutate safely accross threads
}

async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // get counter's MutexGuard
    *counter += 1; // access counter inside MutexGuard

    format!("Request number: {counter}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // NOTE: web::Data created outside HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()
            /*
            Notes: Why We Use .clone() with web::Data
                web::Data uses Arc internally
                → This makes the state safe to share across threads (Send + Sync).
                HttpServer::new runs the closure once per thread
                → Each thread builds its own App instance.
                If we don’t clone the data, it would move into the first closure call,
                leaving nothing for the next thread → ❌ compile-time ownership error.
                .clone() on web::Data only clones the Arc,
                not the actual data — so:
                ✅ No data duplication
                ✅ Safe shared access across threads
                ✅ Cheap operation (just increments reference count)
                In short:
                .clone() gives each worker thread a handle to the same shared state.
                        */
            .app_data(counter.clone()) // register the crated data
            .route("/", web::get().to(index))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
 */

////////////////////////////////////////////////////////////////
/*
/*
   USING AN APPLICATION SCOPE TO COMPOSE APPLICATIONS

   🔹 Purpose
    web::scope() defines a URL path prefix for a group of routes.
    It’s a way to namespace routes (e.g. /users, /admin, /api).
    All routes inside that scope automatically get the prefix prepended.

🔹 Why It’s Useful
    Helps organize routes logically (by feature or module).
    Allows mounting existing route groups under different paths.
    Keeps route names consistent when generating URLs with url_for().

*/

#[get("/show")]
async fn show_users() -> impl Responder {
    "User list"
}

#[actix_web::main]
async fn main() {
    let scope = web::scope("/users").service(show_users);
    App::new().service(scope);
}
 */

/////////////////////////////////////////////////////////////////
/*
/*
   APPLICATION GUARDS AND VIRTUAL HOSTING
    you can think of a guard as a simple function that accepts a request object reference and returns true or false
     Formally, a guard is any object that implements the Guard trait
     Actix Web provides several guards

    One of the provided guards is Host. it can be used as a filter based on request header info

    If the guard returns true, the route’s handler runs;
    if it returns false, Actix continues searching for another matching route.

    🧠 Think of it like:
    "A route will only activate if all its guards say yes.

    //common built in guards
    | Guard                                   | Purpose                    |
| --------------------------------------- | -------------------------- |
| `guard::Get()`                          | Matches only GET requests  |
| `guard::Post()`                         | Matches only POST requests |
| `guard::Header("Header-Name", "value")` | Matches based on a header  |
| `guard::Host("example.com")`            | Matches a specific host    |
| `guard::Any(...)` / `guard::All(...)`   | Combine multiple guards    |


You can also create your own:
use actix_web::{dev::ServiceRequest, guard::Guard};

struct CustomGuard;

impl Guard for CustomGuard {
    fn check(&self, req: &ServiceRequest) -> bool {
        req.path().starts_with("/admin")
    }
}

Then use it like:
.route("/admin", web::get().guard(CustomGuard).to(admin_handler))

*/

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        /*
        If the request’s Host header is example.com, it hits the first handler.
        If it’s test.com, it hits the second.
        THIS ALLOWS MULTI-DOMAIN SUPPORT WITHIN THE SAME APP
                 */
        App::new()
            .service(
                web::scope("/")
                    .guard(guard::Host("www.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("www") })),
            )
            .service(
                web::scope("/")
                    .guard(guard::Host("users.rust-lang.org"))
                    .route("", web::to(|| async { HttpResponse::Ok().body("user") })),
            )
            .route("/", web::to(HttpResponse::Ok))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
 */

/////////////////////////////////////////////////////////////////////////////
/*
/*
Configure
 for simplicity and reuseability both App and web::Scope provide the configure method.
  this function is useful form oving parts of the configuration to a different module or even library.
  for eg, some of the resource's configuration could be moved to a different module

 App::configure() and web::Scope::configure() let you modularize route and service setup by moving
  them into separate functions or modules.

    Instead of registering all routes directly in main.rs,
    you can group related ones (like /users or /posts) into reusable config functions.
 */

//this function could be located in a different module
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

//this function could be located in a different module
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .configure(config) // applies to App-level routes
            .service(web::scope("/api").configure(scoped_config)) //like app.use("/users" userRouter) in express.js
            .route(
                "/",
                web::get().to(|| async { HttpResponse::Ok().body("/") }),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
 */

////////////////////////////////////////////////////
/*
/*
MULTI-THREADING
 httpServer automatically starts a number of HTTP workers, by default this numbers is
 equal to the number of physical CPUS in the system. This number can be overriden with
  the HttpServer::workers() method
*/

#[actix_web::main]
async fn main() {
    let _ = HttpServer::new(|| App::new().route("/", web::get().to(HttpResponse::Ok))).workers(4);
    //  <- Start 4 workers
}
 */

/*
   since each worker thread processes its requests sequentially, handlers which block the 
    current thread will cause the current worker to stop processing new requests
    
        fn my_handler() -> impl Responder {
            std::thread::sleep(Duration::from_secs(5)); // <-- Bad practice! Will cause the current worker thread to hang!
        "response"

    for this reason, any long, non-cpu-bound operation (eg: database operations) should be expresses as futures or asynchronous operations
      Async handlers get executed concurrently by worker threads andd thus don't block execution

        async fn my_handler() -> impl Responder {
           tokio::time::sleep(Duration::from_secs(5)).await // <-- Ok. Worker thread will handle other requests here
           "response"
        }
}
*/

////////////////////////////////////////////////////////////////
/*
/*
  KEEP-ALIVE
    actix web keeps connections open to wait for subsquent requests
      keep alive connection behaviour is defined by server settings
      - Duration::from_secs(75) or keepAlive::Timeout(75) : enables 75 second keep-alive timer
      - KeepAlive::Os: uses Os keep-alive
      - Noe or KeepAlive::Disabled: disables keep-alive
*/

#[actix_web::main]
// async fn main() -> std::io::Result<()>{
//     // Set keep-alive to 75 seconds
//     let _one = HttpServer::new(app).keep_alive(Duration::from_secs(75));

//     // use OS's keep-alive (usually quite long)
//     let _two = HttpServer::new(app).keep_alive(KeepAlive::Os);

//     // Disable keep-alive
//     let _three = HttpServer::new(app).keep_alive(None)

//     Ok(())

//     /*
//       if the first option above is selected, then keep-alive is enabled for HTTP/1.1 requests
//        if the response does not explicitly disallow it by, for eg, setting the connection type for 
//         Close or Upgrade. force closing a connection can be done with the force_close() method on HTTPResponseBuilder

//         Keep-alive is off for HTTP/1.0 and is on for HTTP/1.1 and HTTP/2.0.
//      */
// }
      
  // Keep-alive is off for HTTP/1.0 and is on for HTTP/1.1 and HTTP/2.0.
async fn index(_req: HttpRequest) -> HttpResponse {
    let mut resp = HttpResponse::Ok()
    .force_close() // <- close connection on the HttpRepsonseBuilder
    .finish();

    // Alternatively close connection on the HttpResponse struct
    resp.head_mut().set_connection_type(http::ConnectionType::Close);

    resp
}
 */
fn main() {}