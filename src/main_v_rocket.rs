#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel;
use diesel::{prelude::*, table, Insertable, Queryable};
use rocket::{fairing::AdHoc, serde::json::Json, State};
use rocket_sync_db_pools::database;
use serde::{Deserialize, Serialize};

table! {
    todos (id) {
        id -> Int4,
        title -> Varchar,
        body -> Text,
    }
}

#[database("project-todo")]
pub struct Db(diesel::PgConnection);

#[derive(Serialize, Deserialize, Clone, Queryable, Debug, Insertable)]
#[table_name = "todos"]
struct Todo {
    id: i32,
    title: String,
    body: String,
}

#[derive(Deserialize)]
struct Config {
    name: String,
    age: u8,
}

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/random")]
fn get_random_to_do() -> Json<Todo> {
    Json(Todo {
        id: 1,
        title: "First TODO".to_string(),
        body: "This is my description for 1st TIDI".to_string(),
    })
}

#[get("/<id>")]
fn get_single_todo(id: i32) -> Json<Todo> {
    Json(Todo {
        id,
        title: "Some title".to_string(),
        body: "Some body".to_string(),
    })
}

#[post("/", data = "<todo>")]
async fn create_todo(connection: Db, todo: Json<Todo>) -> Json<Todo> {
    connection
        .run(move |c| {
            diesel::insert_into(todos::table)
                .values(&todo.into_inner())
                .get_result(c)
        })
        .await
        .map(Json)
        .expect("boo")
}

#[get("/")]
async fn get_all_todos(connection: Db) -> Json<Vec<Todo>> {
    connection
        .run(|c| todos::table.load(c))
        .await
        .map(Json)
        .expect("Failed to fetch todos")
}

#[get("/config")]
fn custom(config: &State<Config>) -> String {
    format!("Hello, {}! You are {} years old.", config.name, config.age)
}

#[launch]
fn rocket() -> _ {
    let rocket = rocket::build();

    rocket
        .attach(Db::fairing())
        .attach(AdHoc::config::<Config>())
        .mount("/", routes![index, custom])
        .mount(
            "/todos",
            routes![
                get_random_to_do,
                get_single_todo,
                get_all_todos,
                create_todo
            ],
        )
}