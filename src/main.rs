use std::{collections::HashMap, sync::RwLock, ops::Deref};

use rocket::{launch, routes, get, fs::{FileServer, Options}, post, FromForm, form::Form, response::Redirect, uri, State};
use rocket_dyn_templates::{Template, context};

#[derive(FromForm, Debug)]
struct TaskForm<'a> {
    id: &'a str,
    task: &'a str,
}

struct Tasks {
    pub tasks: RwLock<HashMap<String, String>>
}

#[get("/")]
fn index() -> Template {
    Template::render("index", context! {})
}

#[get("/login")]
fn login() -> Template {
    Template::render("login", context! {})
}

#[get("/tasks")]
fn tasks(tasks: &State<Tasks>) -> Template {
    let tasks = tasks.inner().tasks.read().unwrap();
    Template::render("tasks", context! { tasks: tasks.deref() })
}

#[post("/set_task", data = "<task_form>")]
fn set_task(task_form: Form<TaskForm<'_>>, tasks: &State<Tasks>) -> Redirect {
    println!("Recieved task data: {:?}", task_form);
    tasks.inner().tasks.write().unwrap().insert(task_form.id.to_string(), task_form.task.to_string());
    Redirect::to(uri!("/tasks"))
}

#[post("/remove_task", data = "<task_form>")]
fn remove_task(task_form: Form<TaskForm<'_>>, tasks: &State<Tasks>) -> Redirect {
    tasks.inner().tasks.write().unwrap().remove(&task_form.id.to_string());
    Redirect::to(uri!("/tasks"))
}

#[get("/signup")]
fn signup() -> Template {
    Template::render("signup", context! {})
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, tasks, set_task, remove_task, signup, login])
        .mount("/assets/", FileServer::new("assets", Options::None))
        .attach(Template::fairing())
        .manage(Tasks { tasks: RwLock::new(HashMap::new()) })
}