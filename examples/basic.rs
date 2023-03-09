use std::net::SocketAddr;

use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router, Server, TypedHeader,
};
use jsonwebtoken as jwt;
use jwt::Validation;
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

    let _ = Server::bind(&addr).serve(app.into_make_service()).await;
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

async fn create_todo_handler(_claims: Claims, Json(todo): Json<CreateTodo>) -> impl IntoResponse {
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

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(token)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| HttpError::Auth)?;

        let key = jwt::DecodingKey::from_secret(SECRET);

        let token = jwt::decode::<Claims>(token.token(), &key, &Validation::default())
            .map_err(|_| HttpError::Internal)?;

        Ok(token.claims)
    }
}

#[derive(Debug)]
enum HttpError {
    Auth,
    Internal,
}

impl IntoResponse for HttpError {
    fn into_response(self) -> axum::response::Response {
        let (code, msg) = match self {
            HttpError::Auth => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            HttpError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR"),
        };
        (code, msg).into_response()
    }
}
