use base64::{Engine, engine::general_purpose::STANDARD as base64_eng};
use rocket::{post, serde::{json::Json, Serialize, Deserialize}, request::{FromRequest, Outcome}, Request, http::Status};
use rocket_db_pools::Connection;

use crate::{db::{MainDB, reprs::{User, Task}}, security, Error};

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct GetTasksResponseJson {
	success: bool,
	tasks: Vec<TaskJson>
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct TaskJson {
	user_task_id: i32,
	task: String
}

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct SuccessJson {
	success: bool
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
			let hdr = &hdr[6..];
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

async fn get_user_with_verify(db: &mut Connection<MainDB>, auth: AuthHeader) -> Result<User, Error> {
	match MainDB::find_user(&mut *db, &auth.username).await {
		Ok(user) => {
			if security::password_verify(&auth.password, &user.pass) {
				Ok(user)
			} else {
				Err(Error::AuthenticationError("Password did not match".to_string()))
			}
		}
		Err(e) => Err(e)
	}
}

#[post("/tasks/get")]
pub async fn get_tasks(mut db: Connection<MainDB>, auth: AuthHeader) -> Json<GetTasksResponseJson> {
	if let Ok(user) = get_user_with_verify(&mut db, auth).await {
		if let Ok(tasks) = MainDB::get_user_tasks(&mut db, user.id).await {
			return Json(GetTasksResponseJson {
				success: true,
				tasks: tasks.into_iter().map(|dbt| TaskJson { user_task_id: dbt.user_task_id, task: dbt.text}).collect()
			});
		}
	}

	Json(GetTasksResponseJson { success: false, tasks: Vec::new() })
}

#[post("/tasks/set", format = "json", data = "<tasks>")]
pub async fn set_tasks(tasks: Json<Vec<TaskJson>>, mut db: Connection<MainDB>, auth: AuthHeader) -> Json<SuccessJson> {
	if let Ok(user) = get_user_with_verify(&mut db, auth).await {
		for task in tasks.0 {
			if let Err(_) = MainDB::set_task(&mut db, Task { id: 0, user_id: user.id, user_task_id: task.user_task_id, text: task.task }).await {
				return Json(SuccessJson { success: false })
			}
		}

		return Json(SuccessJson { success: true })
	}

	return Json(SuccessJson { success: false })
}

#[post("/tasks/delete", format = "json", data = "<user_task_ids>")]
pub async fn delete_tasks(user_task_ids: Json<Vec<i32>>, mut db: Connection<MainDB>, auth: AuthHeader) -> Json<SuccessJson> {
	if let Ok(user) = get_user_with_verify(&mut db, auth).await {
		for user_task_id in user_task_ids.0 {
			if let Err(_) = MainDB::delete_task(&mut db, Task { id: 0, user_id: user.id, user_task_id, text: "".to_string() }).await {
				return Json(SuccessJson { success: false })
			}
		}

		return Json(SuccessJson { success: true })
	}

	return Json(SuccessJson { success: false })
}