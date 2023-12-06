mod routes;
mod state;
mod forms;
mod db;
mod security;

use std::{collections::BTreeMap, sync::RwLock, fs, io};
use rocket::{launch, routes, fs::{FileServer, Options}, response::{self, Responder}, Response, http::Status};

use rocket_dyn_templates::Template;
use serde::Deserialize;

use sqlx::Error as SqlxError;

use crate::{state::{Tasks, SessionManager}, routes::*, db::MainDB};

#[derive(Debug, Deserialize)]
struct Env {
	pub url: Option<String>
}

impl Env {
	fn read_env() -> Option<Env> {
		let env_str = fs::read_to_string(".env").ok()?;
		toml::from_str(&env_str).ok()
	}
}

pub enum Error {
	Sql(SqlxError),
	UserExists(String),
	UserDoesNotExist(String),
	AuthenticationError(String)
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
	fn respond_to(self, _: &'r rocket::Request<'_>) -> response::Result<'o> {
		let mut resp = Response::new();
		let (status, msg) = match self {
			Error::Sql(sqlx_e) => (Status::InternalServerError, sqlx_e.to_string()),
			Error::UserExists(uname) => (Status::Conflict, format!("Error: User \"{}\" already exists", uname)),
			Error::UserDoesNotExist(uid_or_uname) => (Status::Unauthorized, format!("User {} does not exist", uid_or_uname)),
			Error::AuthenticationError(msg) => (Status::Unauthorized, format!("Authentication error: {}", msg)),
		};
		resp.set_status(status);
		resp.set_sized_body(msg.len(), io::Cursor::new(msg));
		Ok(resp)
	}
}

#[launch]
fn rocket() -> _ {
	// rocket::build()
	// 	.mount("/", routes![index, tasks, set_task, remove_task, signup, login])
	// 	.mount("/assets/", FileServer::new("assets", Options::None))
	// 	.attach(Template::fairing())
	// 	.attach(MainDB::init())
	// 	.manage(Tasks { tasks: RwLock::new(BTreeMap::new()) })

	let env = Env::read_env();

	println!("{:?}", env);

	let mut use_db = false;

	let mut rocket_build = if let Some(env) = env {
		if let Some(url) = env.url {
			let figment = rocket::Config::figment()
				.merge(("databases.main_db", rocket_db_pools::Config {
					url,
					min_connections: None,
					max_connections: 128,
					connect_timeout: 5,
					idle_timeout: Some(180)
				}));

			use_db = true;

			rocket::custom(figment)
		} else {
			rocket::build()
		}
	} else {
		rocket::build()
	};

	rocket_build = rocket_build
		.mount("/", routes![index, tasks, set_task, remove_task, signup, login, create_user, login_user, logout])
		.mount("/assets/", FileServer::new("assets", Options::None))
		.attach(Template::fairing());

	rocket_build = if use_db {
		rocket_build.attach(MainDB::stage())
	} else {
		rocket_build
	};

	rocket_build
		.manage(SessionManager::new())
		.manage(Tasks { tasks: RwLock::new(BTreeMap::new()) })
}