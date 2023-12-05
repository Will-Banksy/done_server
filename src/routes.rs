use std::ops::Deref;

use rocket::{get, State, response::Redirect, form::Form, post, uri};
use rocket_dyn_templates::{Template, context};

use crate::{state::Tasks, forms::TaskForm};

#[get("/")]
pub fn index() -> Template {
	Template::render("index", context! {})
}

#[get("/login")]
pub fn login() -> Template {
	Template::render("login", context! {})
}

#[get("/tasks")]
pub fn tasks(tasks: &State<Tasks>) -> Template {
	let tasks = tasks.inner().tasks.read().unwrap();
	Template::render("tasks", context! { tasks: tasks.deref() })
}

#[post("/set_task", data = "<task_form>")]
pub fn set_task(task_form: Form<TaskForm<'_>>, tasks: &State<Tasks>) -> Redirect {
	eprintln!("/set_task: Recieved task data: {:?}", task_form);
	tasks.inner().tasks.write().unwrap().insert(task_form.id.to_string(), task_form.task.to_string());
	Redirect::to(uri!("/tasks"))
}

#[post("/remove_task", data = "<task_form>")]
pub fn remove_task(task_form: Form<TaskForm<'_>>, tasks: &State<Tasks>) -> Redirect {
	eprintln!("/remove_task: Recieved task data: {:?}", task_form);
	tasks.inner().tasks.write().unwrap().remove(&task_form.id.to_string());
	Redirect::to(uri!("/tasks"))
}

#[get("/signup")]
pub fn signup() -> Template {
	Template::render("signup", context! {})
}