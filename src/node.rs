use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(root))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/bar", get(foo_bar));

    // run our app with hyper, listening globally on port 3000
    let addr = "0.0.0.0:3000".parse().unwrap();
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    async fn root() -> &'static str {
        "Hello, I'm root!"
    }

    async fn get_foo() -> &'static str {
        "Hello, I'm GET /foo!"
    }

    async fn post_foo() -> &'static str {
        "Hello, I'm POST /foo!"
    }

    async fn foo_bar() -> &'static str {
        "Hello, I'm /foo/bar!"
    }
}
