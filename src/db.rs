pub mod reprs;

use rocket::{fairing::{AdHoc, self}, Rocket, Build, futures::future::try_join_all, error};
use rocket_db_pools::{Database, Connection};
use sqlx::Row;

use crate::Error;

use self::reprs::{User, Task};

const SQL_CREATE_TABLE: &'static str = include_str!("../sql/schema.sql");

#[derive(Database)]
#[database("main_db")]
pub struct MainDB(sqlx::MySqlPool);

impl MainDB {
	pub fn stage() -> AdHoc {
		AdHoc::on_ignite("Stage: Create database table", |rocket| async {
			rocket
				.attach(MainDB::init())
				.attach(AdHoc::try_on_ignite("Create database table", MainDB::create_table))
		})
	}

	async fn create_table(rocket: Rocket<Build>) -> fairing::Result {
		match MainDB::fetch(&rocket) {
			Some(db) => {
				match try_join_all(SQL_CREATE_TABLE.split(";").filter(|s| !s.is_empty()).map(|stmt| {
					sqlx::query(stmt).execute(&**db)
				}).collect::<Vec<_>>()).await {
					Ok(res) => {
						res.into_iter().for_each(|res| println!("Query success: affected {} rows", res.rows_affected()));
						Ok(rocket)
					}
					Err(e) => {
						error!("Failed to run query: {}", e);
						Err(rocket)
					}
				}
			}
			None => Err(rocket)
		}
	}

	pub async fn get_user(conn: &mut Connection<MainDB>, user_id: i32) -> Result<User, Error> {
		let res = sqlx::query("select * from main_db.auth where user_id = ? limit 1").bind(user_id).fetch_one(&mut ***conn).await.map_err(|e| Error::Sql(e))?;

		Ok(User::from_row(res).ok_or(Error::UserDoesNotExist(format!("id {}", user_id)))?)
	}

	pub async fn find_user(conn: &mut Connection<MainDB>, user_name: &str) -> Result<User, Error> {
		let res = sqlx::query("select * from main_db.auth where user_name = ? limit 1").bind(user_name).fetch_one(&mut ***conn).await.map_err(|e| Error::Sql(e))?;

		Ok(User::from_row(res).ok_or(Error::UserDoesNotExist(user_name.to_string()))?)
	}

	/// Requires: user.name, user.pass
	pub async fn create_user(conn: &mut Connection<MainDB>, user: User) -> Result<i32, Error> {
		let existing_user = sqlx::query("select user_name from main_db.auth where user_name = ? limit 1").bind(user.name.clone()).fetch_optional(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		if existing_user.is_some() {
			return Err(Error::UserExists(user.name.clone()))
		}

		sqlx::query("insert into main_db.auth(user_name, user_pass) values(?, ?)").bind(user.name.clone()).bind(user.pass).execute(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		let inserted = sqlx::query("select user_id, user_name from main_db.auth where user_name = ?").bind(user.name).fetch_one(&mut ***conn).await.map_err(|e| Error::Sql(e))?;

		Ok(inserted.try_get("user_id").map_err(|e| Error::Sql(e))?)
	}

	/// Requires: task.user_id, task.user_task_id, task.text
	pub async fn set_task(conn: &mut Connection<MainDB>, task: Task) -> Result<(), Error> {
		let row = sqlx::query("select * from main_db.tasks where user_id = ? and user_task_id = ?").bind(task.user_id).bind(task.user_task_id).fetch_optional(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		if row.is_some() {
			sqlx::query("update main_db.tasks set task_text = ? where user_id = ? and user_task_id = ?").bind(task.text).bind(task.user_id).bind(task.user_task_id).execute(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		} else {
			sqlx::query("insert into main_db.tasks(user_task_id, user_id, task_text) values(?, ?, ?)").bind(task.user_task_id).bind(task.user_id).bind(task.text).execute(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		}
		Ok(())
	}

	/// Requires: task.user_id, task.user_task_id
	pub async fn remove_task(conn: &mut Connection<MainDB>, task: Task) -> Result<(), Error> {
		sqlx::query("delete from main_db.tasks where user_id = ? and user_task_id = ?").bind(task.user_id).bind(task.user_task_id).execute(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		Ok(())
	}

	pub async fn get_user_tasks(conn: &mut Connection<MainDB>, user_id: i32) -> Result<Vec<Task>, Error> {
		Ok(sqlx::query("select * from main_db.tasks where user_id = ?").bind(user_id).fetch_all(&mut ***conn).await.map_err(|e| Error::Sql(e))?.into_iter().map(|row| Task::from_row(row).unwrap()).collect())
	}
}