use std::net::SocketAddr;

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router, Server,
};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

const SECRET: &[u8] = b"secret";

#[derive(Debug, Deserialize, Serialize)]
struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginResponse {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Todo {
    pub id: usize,
    pub title: String,
    pub completed: bool,
}

impl Todo {
    pub fn new(id: usize, title: String) -> Self {
        Self {
            id,
            title,
            completed: false,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodo {
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    id: usize,
    name: String,
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/todos", get(todos_handler).post(create_todo_handler))
        .route("/login", post(login_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("Listening on http://{addr}");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html("Hello, world!")
}

async fn todos_handler() -> impl IntoResponse {
    Json(vec![
        Todo::new(1, "Todo 1".to_string()),
        Todo::new(2, "Todo 2".to_string()),
    ])
}

async fn create_todo_handler(Json(todo): Json<CreateTodo>) -> impl IntoResponse {
    println!("{todo:?}");
    StatusCode::CREATED
}

async fn login_handler(Json(login): Json<LoginRequest>) -> impl IntoResponse {
    let claims = Claims {
        id: 1,
        name: login.email,
    };
    let key = jwt::EncodingKey::from_secret(SECRET);
    let token = jwt::encode(&jwt::Header::default(), &claims, &key).unwrap();

    Json(LoginResponse { token })
}
