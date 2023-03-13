use std::{
    net::SocketAddr,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, RwLock,
    },
};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Json, Router, Server,
};
use devtools_api::types::{
    claims::Claims,
    create_todo::CreateTodo,
    login_request::LoginRequest,
    login_response::LoginResponse,
    todo::{Todo, TodoStore},
};
use jsonwebtoken as jwt;
use serde::{Deserialize, Serialize};

const SECRET: &[u8] = b"secret";
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() {
    let store = TodoStore {
        items: Arc::new(RwLock::new(vec![Todo {
            id: 0,
            user_id: 0,
            title: "Learn Rust".to_string(),
            completed: false,
        }])),
    };
    let app = Router::new()
        .route("/", get(index_handler))
        .route(
            "/todos",
            get(todos_handler)
                .post(create_todo_handler)
                .layer(Extension(store)),
        )
        .route("/login", post(login_handler));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    println!("Listening on http://{addr}");

    let _ = Server::bind(&addr).serve(app.into_make_service()).await;
}

async fn index_handler() -> Html<&'static str> {
    Html("Hello, world!")
}

async fn todos_handler(Extension(store): Extension<TodoStore>) -> impl IntoResponse {
    match store.items.read() {
        Ok(items) => {
            let res: Vec<Todo> = items.iter().cloned().collect();

            Json(Ok(res))
        }
        Err(_) => Json(Err(HttpError::Internal)),
    }
}

// Axum 的 Handler trait 定义中要求 T, S, B 参数的顺序是 T 在前，S 在中间，B 在最后，这个顺序非常重要，因为 Axum 在处理请求时，首先需要对请求进行解析，然后调用业务逻辑处理函数，最后将响应信息写入到 HTTP 响应中。因此，这三个参数的顺序也决定了它们的含义和作用。

// 通常，请求的 body 信息是在解析请求时被读取并转换成相应的数据类型，最后传递给业务逻辑处理函数作为参数。因此，Axum 推荐将 body 参数放在函数参数列表的最后面，以方便读取请求的其他参数并进行处理，最后处理 body 参数。如果将 body 参数放在前面，可能会导致其他参数还没有准备好就开始处理 body，进而导致错误。

// 如果 Json(todo): Json<CreateTodo>, 不放在最后一个参数，路由就会报错。

/// 在Axum中，Handler trait 的定义如下：
/// pub trait Handler<ReqBody>: Clone + Send + Sized + 'static {
///     type Fut: Future<Output = Response<Body>> + Send + 'static;
///     fn call(&self, req: Request<ReqBody>) -> Self::Fut;
///     More methods...
/// }
/// 在这个trait中，call方法需要接收一个Request对象，这个对象在定义中的泛型参数为ReqBody，表示这个请求的body的类型。Axum在实现Handler trait的时候使用了Rust的特性推导和impl Trait功能，可以自动推导ReqBody的类型，但是要求在call方法之前必须出现所有的泛型参数，这样才能正确地推导出ReqBody的类型。

///对于函数1和3，它们都满足这个要求，因为在call方法之前，Extension(store)和Json(todo)这两个泛型参数都已经出现了。而对于函数2，Extension(store)和Json(todo)出现在了claims之后，无法正确推导ReqBody的类型，因此不符合Handler trait的trait bound。

async fn create_todo_handler(
    Extension(store): Extension<TodoStore>,
    claims: Claims,
    Json(todo): Json<CreateTodo>,
) -> impl IntoResponse {
    match store.items.write() {
        Ok(mut guard) => {
            let todo = Todo {
                id: get_next_id(),
                user_id: claims.id,
                title: todo.title,
                completed: false,
            };
            guard.push(todo.clone());
            Json(Ok(todo))
        }
        Err(_) => Json(Err(HttpError::Internal)),
    }
}

async fn login_handler(Json(login): Json<LoginRequest>) -> impl IntoResponse {
    let claims = Claims {
        id: 1,
        name: login.email,
        exp: get_epoch() + 14 * 24 * 60 * 60,
    };
    let key = jwt::EncodingKey::from_secret(SECRET);
    let token = jwt::encode(&jwt::Header::default(), &claims, &key).unwrap();

    Json(LoginResponse { token })
}

fn get_epoch() -> usize {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
}

#[derive(Debug, Serialize, Deserialize)]
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

fn get_next_id() -> usize {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}
