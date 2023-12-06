use std::ops::Deref;

use rocket::{get, State, response::{Redirect, Flash}, form::Form, post, uri, http::CookieJar};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{Template, context};

use crate::{state::{Tasks, SessionManager}, forms::{TaskForm, SignupForm, LoginForm}, db::{MainDB, reprs::User}, security, Error};

const SESSID_COOKIE: &'static str = "done_sessid";

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Template {
	let sessid = cookies.get_private(SESSID_COOKIE);
	if let Some(sessid) = sessid {
		if let Some(user_id) = sman.query_session(sessid.value()) {
			if let Ok(user) = MainDB::get_user(&mut db, user_id).await {
				return Template::render("index", context! {
					user_name: user.name
				})
			}
		}
	}

	Template::render("index", context! {})
}

#[get("/login")]
pub fn login() -> Template {
	Template::render("login", context! {})
}

#[post("/login_user", data = "<login_form>")]
pub async fn login_user(login_form: Form<LoginForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Result<Flash<Redirect>, Error> {
	let user = MainDB::find_user(&mut db, login_form.username).await?;

	if security::password_verify(login_form.password, &user.pass) {
		let sessid = sman.start_session(user.id);

		cookies.add_private((SESSID_COOKIE, sessid));

		Ok(Flash::success(Redirect::to(uri!("/")), "Login successful"))
	} else {
		Err(Error::AuthenticationError("Password did not match".to_string()))
	}
}

#[get("/signup")]
pub fn signup() -> Template {
	Template::render("signup", context! {})
}

#[post("/create_user", data = "<signup_form>")]
pub async fn create_user(signup_form: Form<SignupForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Result<Flash<Redirect>, Error> {
	let user = User {
		id: 0, // id is ignored when inserting into db
		name: signup_form.username.into(),
		pass: security::password_hash(signup_form.password),
		tasks: None
	};

	let user_id = MainDB::create_user(&mut db, user).await?;

	let sessid = sman.start_session(user_id);

	cookies.add_private((SESSID_COOKIE, sessid));

	Ok(Flash::success(Redirect::to(uri!("/")), "Login successful"))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Redirect {
	let sessid = cookies.get_private(SESSID_COOKIE);
	if let Some(sessid) = sessid {
		sman.stop_session(sessid.value());
		cookies.remove_private(SESSID_COOKIE);
	}

	Redirect::to("/")
}

#[get("/tasks")]
pub async fn tasks(tasks: &State<Tasks>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Template {
	let sessid = cookies.get_private(SESSID_COOKIE);
	if let Some(sessid) = sessid {
		if let Some(user_id) = sman.query_session(sessid.value()) {
			if let Ok(user) = MainDB::get_user(&mut db, user_id).await {
				let tasks = tasks.inner().tasks.read().unwrap();
				return Template::render("tasks", context! {
					user_name: user.name,
					tasks: tasks.deref()
				})
			}
		}
	}

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