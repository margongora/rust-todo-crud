# rust-todo-crud
Howdy!<br>
This is a basic Rust todo-app using Axum and SQLx to implement basic CRUD functionality.<br>
The app utilizes Postgres as the database backend, with Axum serving as the backend server to handle requests.<br>
<br>
Example usage is as follows: <br>
## Create
```
curl -H "Content-Type: application/json" -d '{"content": "Add HTML to the project!", "done": false}' localhost:42069/todos
```
## Read (Multiple)
```
curl localhost:42069/todos
```
## Read (Single by ID)
```
curl localhost:42069/todos/1
```
## Update
```
curl -H "Content-Type: application/json" -d '{"content": "Add CSS to the project!", "done": false}' -X "PUT" localhost:42069/todos/1
```
## Delete
```
curl -X "DELETE" localhost:42069/1
```
<br>
I plan to eventually add HTML and CSS to the project. Askama could be used for rendering the todos. I also plan on adding a journal functionality, so as to make a minor productivity app for myself.
<br><br>
Thank you for viewing!
