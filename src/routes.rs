use rocket::{get, State, response::{Redirect, Flash}, form::Form, post, uri, http::{CookieJar, Status}, request::FlashMessage};
use rocket_db_pools::Connection;
use rocket_dyn_templates::{Template, context};

use crate::{state::SessionManager, forms::{TaskForm, SignupForm, LoginForm}, db::{MainDB, reprs::{User, Task}}, security, Error};

const SESSID_COOKIE: &'static str = "done_sessid";

#[get("/")]
pub async fn index(flash: Option<FlashMessage<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Template {
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
pub fn login(flash: Option<FlashMessage<'_>>) -> Template {
	if let Some(msg) = flash {
		Template::render("login", context! {
			error_msg: msg.message()
		})
	} else {
		Template::render("login", context! {})
	}
}

#[post("/login_user", data = "<login_form>")]
pub async fn login_user(login_form: Form<LoginForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Flash<Redirect> {
	match MainDB::find_user(&mut db, login_form.username).await {
		Ok(user) => {
			if security::password_verify(login_form.password, &user.pass) {
				let sessid = sman.start_session(user.id);

				cookies.add_private((SESSID_COOKIE, sessid));

				Flash::success(Redirect::to(uri!("/")), "Login successful")
			} else {
				Flash::error(Redirect::to(uri!("/login")), Error::AuthenticationError("Password did not match".to_string()).to_string())
			}
		}
		Err(e) => {
			let msg = {
				if let Error::Sql(sqlx_e) = e {
					match sqlx_e {
						sqlx::Error::RowNotFound => format!("Error: Username \"{}\" not found", login_form.username),
						_ => sqlx_e.to_string(),
					}
				} else {
					e.to_string()
				}
			};
			Flash::error(Redirect::to(uri!("/login")), msg)
		}
	}
}

#[get("/signup")]
pub fn signup(flash: Option<FlashMessage<'_>>) -> Template {
	if let Some(msg) = flash {
		Template::render("signup", context! {
			error_msg: msg.message()
		})
	} else {
		Template::render("signup", context! {})
	}
}

#[post("/create_user", data = "<signup_form>")]
pub async fn create_user(signup_form: Form<SignupForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Flash<Redirect> {
	if signup_form.username.is_empty() {
		return Flash::error(Redirect::to(uri!("/signup")), "Username cannot be empty");
	} else if signup_form.password.is_empty() {
		return Flash::error(Redirect::to(uri!("/signup")), "Password cannot be empty");
	}

	eprintln!("signup_form.username: {} (length: {} bytes)", signup_form.username, signup_form.username.len());
	eprintln!("signup_form.password: {} (length: {} bytes)", signup_form.password, signup_form.password.len());

	let user = User {
		id: 0, // id is ignored when inserting into db
		name: signup_form.username.into(),
		pass: security::password_hash(signup_form.password),
		tasks: None
	};

	match MainDB::create_user(&mut db, user).await {
		Ok(user_id) => {
			let sessid = sman.start_session(user_id);

			cookies.add_private((SESSID_COOKIE, sessid));

			Flash::success(Redirect::to(uri!("/")), "User creation successful")
		}
		Err(e) => {
			Flash::error(Redirect::to("/signup"), format!("{}", e))
		}
	}
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>, sman: &State<SessionManager>) -> Redirect {
	let sessid = cookies.get_private(SESSID_COOKIE);
	if let Some(sessid) = sessid {
		sman.stop_session(sessid.value());
		cookies.remove_private(SESSID_COOKIE);
	}

	Redirect::to("/")
}

#[get("/tasks")]
pub async fn tasks(cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> Template {
	let sessid = cookies.get_private(SESSID_COOKIE);
	if let Some(sessid) = sessid {
		if let Some(user_id) = sman.query_session(sessid.value()) {
			if let Ok(user) = MainDB::get_user(&mut db, user_id).await {
				if let Ok(tasks) = MainDB::get_user_tasks(&mut db, user.id).await {
					eprintln!("Successfully retrieved user tasks");

					return Template::render("tasks", context! {
						user_name: user.name,
						tasks
					})
				}
			}
		}
	}

	// let tasks = tasks.inner().tasks.read().unwrap();
	Template::render("tasks", context! {})
}

#[post("/set_task", data = "<task_form>")]
pub async fn set_task(task_form: Form<TaskForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> (Status, String) {
	eprintln!("/set_task: Recieved task data: {:?}", task_form);

	if let Some(sessid) = cookies.get_private(SESSID_COOKIE) {
		if let Some(user_id) = sman.query_session(sessid.value()) {
			let task = Task {
				id: 0,
				text: task_form.task.to_string(),
				user_id,
				user_task_id: match task_form.id[5..].parse() {
					Ok(res) => res,
					Err(_) => { return (Status::BadRequest, "Task id must be in form \"task-<numeric_id>\"".to_string()) }
				}
			};

			return match MainDB::set_task(&mut db, task).await {
				Ok(()) => (Status::Ok, "SUCCESS".to_string()),
				Err(e) => (Status::InternalServerError, e.to_string()),
			};
		}
	}

	// tasks.inner().tasks.write().unwrap().insert(task_form.id.to_string(), task_form.task.to_string());
	(Status::Unauthorized, "Error: Not logged in".to_string())
}

#[post("/remove_task", data = "<task_form>")]
pub async fn remove_task(task_form: Form<TaskForm<'_>>, cookies: &CookieJar<'_>, mut db: Connection<MainDB>, sman: &State<SessionManager>) -> (Status, String) {
	eprintln!("/remove_task: Recieved task data: {:?}", task_form);

	if let Some(sessid) = cookies.get_private(SESSID_COOKIE) {
		if let Some(user_id) = sman.query_session(sessid.value()) {
			let task = Task {
				id: 0,
				text: task_form.task.to_string(),
				user_id,
				user_task_id: match task_form.id[5..].parse() {
					Ok(res) => res,
					Err(_) => { return (Status::BadRequest, "Task id must be in form \"task-<numeric_id>\"".to_string()) }
				}
			};

			return match MainDB::remove_task(&mut db, task).await {
				Ok(()) => (Status::Ok, "SUCCESS".to_string()),
				Err(e) => (Status::InternalServerError, e.to_string()),
			};
		}
	}

	// tasks.inner().tasks.write().unwrap().remove(&task_form.id.to_string());
	(Status::Unauthorized, "Error: Not logged in".to_string())
}