use std::collections::BTreeMap;

use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD as base64_eng};
use rocket::{post, serde::{json::Json, Serialize}, request::{FromRequest, Outcome}, Request, http::Status};
use rocket_db_pools::Connection;

use crate::{db::MainDB, security};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetTasksResponseJson {
	success: bool,
	tasks: Vec<TaskJson>
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SetTaskRequestJson {
	tasks: BTreeMap<i32, String>
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SetTaskResponseJson {
	success: bool
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct TaskJson {
	user_task_id: i32,
	task: String
}

pub struct AuthHeader {
	username: String,
	password: String,
}

#[derive(Debug)]
pub enum AuthHeaderError {
	Missing,
	Invalid
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthHeader {
	type Error = AuthHeaderError;

	async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
		if let Some(hdr) = req.headers().get_one("Authorization") {
			if let Ok(decoded) = base64_eng.decode(hdr) {
				if let Ok(auth_str) = String::from_utf8(decoded) {
					let mut auth_str_iter = auth_str.split(':').map(|sref| sref.to_string());
					let username = auth_str_iter.next();
					let password = auth_str_iter.next();
					if let Some(username) = username {
						if let Some(password) = password {
							return Outcome::Success(AuthHeader { username, password});
						}
					}
				}
			}
			return Outcome::Error((Status::BadRequest, AuthHeaderError::Invalid))
		}

		Outcome::Error((Status::BadRequest, AuthHeaderError::Missing))
	}
}

#[post("/api/get_tasks")]
pub async fn get_tasks(mut db: Connection<MainDB>, auth: AuthHeader) -> Json<GetTasksResponseJson> {
	match MainDB::find_user(&mut db, &auth.username).await {
		Ok(user) => {
			if security::password_verify(&auth.password, &user.pass) {
				if let Ok(tasks) = MainDB::get_user_tasks(&mut db, user.id).await {
					return Json(GetTasksResponseJson {
						success: true,
						tasks: tasks.into_iter().map(|dbt| TaskJson { user_task_id: dbt.user_task_id, task: dbt.text}).collect()
					})
				}
			}

			Json(GetTasksResponseJson { success: false, tasks: Vec::new() })
		},
		Err(_) => Json(GetTasksResponseJson { success: false, tasks: Vec::new() })
	}
}
