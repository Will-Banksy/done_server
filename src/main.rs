mod routes;
mod state;
mod forms;
mod db;
mod security;

use std::{collections::BTreeMap, sync::RwLock, fs, io, fmt::Display};
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
		let status = match self {
			Error::Sql(_) => Status::InternalServerError,
			Error::UserExists(_) => Status::Conflict,
			Error::UserDoesNotExist(_) => Status::Unauthorized,
			Error::AuthenticationError(_) => Status::Unauthorized,
		};
		let msg = self.to_string();
		resp.set_status(status);
		resp.set_sized_body(msg.len(), io::Cursor::new(msg));
		Ok(resp)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(
			&match self {
				Error::Sql(sqlx_e) => format!("Database/SQL error: {}", sqlx_e.to_string()),
				Error::UserExists(uname) => format!("Error: User \"{}\" already exists", uname),
				Error::UserDoesNotExist(uid_or_uname) => format!("Error: User {} does not exist", uid_or_uname),
				Error::AuthenticationError(msg) => format!("Authentication error: {}", msg),
			}
		)
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