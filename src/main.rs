mod routes;
mod state;
mod forms;
mod db;
mod security;

use std::{fs, io, fmt::Display};
use rocket::{launch, routes, fs::{FileServer, Options}, response::{self, Responder}, Response, http::Status};

use rocket_dyn_templates::Template;
use serde::Deserialize;

use sqlx::Error as SqlxError;

use crate::{state::SessionManager, routes::*, db::MainDB};

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
	let env = Env::read_env();

	println!("{:?}", env);

	let db_url = env.expect(".env must be supplied, containing a url to the database in the form 'url = \"mysql://user:pass@host:port\"'")
		.url.expect(".env was in the incorrect format: It should contain a database url in the form");

	let figment = rocket::Config::figment()
		.merge(("databases.main_db", rocket_db_pools::Config {
			url: db_url,
			min_connections: None,
			max_connections: 128,
			connect_timeout: 5,
			idle_timeout: Some(180)
		}));

	rocket::custom(figment)
		.mount("/", routes![index, tasks, set_task, remove_task, signup, login, create_user, login_user, logout])
		.mount("/assets/", FileServer::new("assets", Options::None))
		.attach(Template::fairing())
		.attach(MainDB::stage())
		.manage(SessionManager::new())
}