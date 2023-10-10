use sqlx::*;
use axum::{
    extract::State, extract::Path,
    response::Response, response::Result, response::IntoResponse,
    http::StatusCode,
    routing::get,
    Router, Json
};
use serde::*;
use serde_json::*;
use std::net::SocketAddr;
use std::env;

pub enum ApiError {
    DatabaseError(sqlx::Error),
    PayloadError(anyhow::Error),
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

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::DatabaseError(_) => 
                (StatusCode::INTERNAL_SERVER_ERROR, "An unexpected exception has occured").into_response(),
            ApiError::PayloadError(_) =>
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

async fn create_todo(State(pool): State<PgPool>, Json(payload): Json<CreateTodo>) -> Result<StatusCode, ApiError> {
    if payload.content.is_empty() {
        return Err(ApiError::PayloadError(anyhow::anyhow!("Won't use empty payloads")))
    }

    // use payload to create todo
    query!("INSERT INTO todos (content, done) VALUES($1, $2)", payload.content, payload.done).execute(&pool).await?;    

    Ok(StatusCode::CREATED)
}

async fn read_todo(Path(id): Path<i32>, State(pool): State<PgPool>) -> Result<Json<serde_json::Value>, ApiError> {
    // grab todo by id
    let todo: Todo = query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id).fetch_one(&pool).await?;

    // send back result
    Ok(Json(json!(&todo)))
}

async fn read_todos(State(pool): State<PgPool>) -> Result<Json<serde_json::Value>, ApiError> {
    // query todos from db
    let todos: Vec<Todo> = query_as!(Todo, "SELECT * FROM todos").fetch_all(&pool).await?;

    // send back result
    Ok(Json(json!(&todos)))
}

async fn update_todo(Path(id): Path<i32>, State(pool): State<PgPool>, Json(payload): Json<CreateTodo>) -> Result<StatusCode, ApiError> {
    if payload.content.is_empty() {
        return Err(ApiError::PayloadError(anyhow::anyhow!("Won't use empty todo")))
    }
    
    // get todo based on id
    let mut todo: Todo = query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id).fetch_one(&pool).await?;

    // transfer payload contents over
    todo.content = payload.content;
    todo.done = payload.done;

    // update todo
    query!("UPDATE todos SET content = $1, done = $2 WHERE id = $3", todo.content, todo.done, todo.id).execute(&pool).await?;    

    Ok(StatusCode::OK)
}

async fn delete_todo(Path(id): Path<i32>, State(pool): State<PgPool>) -> Result<StatusCode, ApiError> {
    // Delete based on id
    query!("DELETE FROM todos WHERE id = $1", id).execute(&pool).await?;

    // send back result
    Ok(StatusCode::OK)
}
