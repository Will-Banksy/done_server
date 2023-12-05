mod routes;
mod state;
mod forms;
mod db;

use std::{collections::BTreeMap, sync::RwLock, fs};
use rocket::{launch, routes, fs::{FileServer, Options}};

use rocket_dyn_templates::Template;
use serde::Deserialize;

use crate::{state::Tasks, routes::*, db::MainDB};

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
		.mount("/", routes![index, tasks, set_task, remove_task, signup, login])
		.mount("/assets/", FileServer::new("assets", Options::None))
		.attach(Template::fairing());

	rocket_build = if use_db {
		rocket_build.attach(MainDB::stage())
	} else {
		rocket_build
	};

	rocket_build
		.manage(Tasks { tasks: RwLock::new(BTreeMap::new()) })
}