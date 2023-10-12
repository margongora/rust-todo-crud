use axum::response::Html;
use sqlx::*;
use axum::{
    extract::State, extract::Path, extract::Multipart,
    response::Response, response::Result, response::IntoResponse,
    http::StatusCode,
    routing::get,
    Router, Json,
};
use serde::*;
use serde_json::*;
use std::net::SocketAddr;
use std::env;
use askama::Template;

#[derive(Template)]
#[template(path="index.html")]
struct Index {}

#[derive(Template)]
#[template(path="todos.html")]
struct Todos {
    todos: Vec<Todo>
}

pub enum ApiError {
    DatabaseError(sqlx::Error),
    PayloadError(anyhow::Error),
    HtmlError(askama::Error)
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError::DatabaseError(e)
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(e: anyhow::Error) -> Self {
        ApiError::PayloadError(e)
    }
}

impl From<askama::Error> for ApiError {
    fn from(e: askama::Error) -> Self {
        ApiError::HtmlError(e)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DatabaseError(_) => 
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected exception has occured").into_response(),
            ApiError::PayloadError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected exception has occured").into_response(),
            ApiError::HtmlError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected exception has occured").into_response()  
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Todo {
    id: i32,
    content: String,
    done: bool
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateTodo {
    content: String,
    done: bool
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    
    // create db pool
    let pool = sqlx::postgres::PgPoolOptions::new()
                .min_connections(5)
                .connect(&env::var("DATABASE_URL").unwrap())
                .await?;

    // setup app router with pool as state
    let app = Router::new()
        .route("/", get(handle_idx))
        .route("/todos", get(read_todos)
            .post(create_todo))
        .route("/todos/:id", get(read_todo)
            .put(update_todo)
            .delete(delete_todo))
        .with_state(pool);

    // run on 127.0.0.1:42069
    let addr = SocketAddr::from(([127, 0, 0, 1], 42069));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    // if no errors, return Ok
    Ok(())
}

async fn handle_idx() -> impl IntoResponse {
    let idx = Index{};
    let reply_html = idx.render().unwrap();

    (StatusCode::OK, Html(reply_html))
}

async fn create_todo(State(pool): State<PgPool>, mut req: Multipart) -> Result<Html<String>, ApiError> {

    let mut i = 0;
    let mut todo: CreateTodo = CreateTodo{ content: String::new(), done: false };

    while let Some(field) = req.next_field().await.unwrap() {
        if i == 0 {
            todo.content = field.text().await.unwrap();
        }
        i += 1;
    }

    if i == 2 { todo.done = true }

    if todo.content.is_empty() {
        return Err(ApiError::PayloadError(anyhow::anyhow!("Won't use empty payloads")))
    }

    query!("INSERT INTO todos (content, done) VALUES($1, $2)", todo.content, todo.done).execute(&pool).await?;

    let todos: Vec<Todo> = query_as!(Todo, "SELECT * FROM todos ORDER BY id").fetch_all(&pool).await?;
    let temp = Todos{ todos: todos };
    let reply = temp.render().unwrap();

    Ok(Html(reply))
}

async fn read_todo(Path(id): Path<i32>, State(pool): State<PgPool>) -> Result<Json<serde_json::Value>, ApiError> {
    // grab todo by id
    let todo: Todo = query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id).fetch_one(&pool).await?;

    // send back result
    Ok(Json(json!(&todo)))
}

async fn read_todos(State(pool): State<PgPool>) -> Result<Html<String>, ApiError> {
    // query todos from db
    let todos: Vec<Todo> = query_as!(Todo, "SELECT * FROM todos ORDER BY id").fetch_all(&pool).await?;
    let temp = Todos{ todos: todos };
    let reply = temp.render().unwrap();

    Ok(Html(reply))
    // send back result
    //Ok(Json(json!(&todos)))
}

async fn update_todo(Path(id): Path<i32>, State(pool): State<PgPool>) -> Result<Html<String>, ApiError> {

    // get todo based on id
    let mut todo: Todo = query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id).fetch_one(&pool).await?;

    // invert doneness of todo
    todo.done = !todo.done;

    // update todo
    query!("UPDATE todos SET done = $1 WHERE id = $2", todo.done, todo.id).execute(&pool).await?;    

    let todos: Vec<Todo> = query_as!(Todo, "SELECT * FROM todos ORDER BY id").fetch_all(&pool).await?;
    let temp = Todos{ todos: todos };
    let reply = temp.render().unwrap();

    Ok(Html(reply))
}

async fn delete_todo(Path(id): Path<i32>, State(pool): State<PgPool>) -> Result<Html<String>, ApiError> {
    // Delete based on id
    query!("DELETE FROM todos WHERE id = $1", id).execute(&pool).await?;

    let todos: Vec<Todo> = query_as!(Todo, "SELECT * FROM todos ORDER BY id").fetch_all(&pool).await?;
    let temp = Todos{ todos: todos };
    let reply = temp.render().unwrap();

    Ok(Html(reply))
}
