pub mod reprs;

use rocket::{fairing::{AdHoc, self}, Rocket, Build, futures::future::try_join_all, error};
use rocket_db_pools::{Database, Connection};
use sqlx::Row;

use crate::Error;

use self::reprs::User;

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

	pub async fn create_user(conn: &mut Connection<MainDB>, user: User) -> Result<i32, Error> {
		let existing_user = sqlx::query("select user_name from main_db.auth where user_name = ? limit 1").bind(user.name.clone()).fetch_optional(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		if existing_user.is_some() {
			return Err(Error::UserExists(user.name.clone()))
		}

		sqlx::query("insert into main_db.auth(user_name, user_pass) values(?, ?)").bind(user.name.clone()).bind(user.pass).execute(&mut ***conn).await.map_err(|e| Error::Sql(e))?;
		let inserted = sqlx::query("select user_id, user_name from main_db.auth where user_name = ?").bind(user.name).fetch_one(&mut ***conn).await.map_err(|e| Error::Sql(e))?;

		Ok(inserted.try_get("user_id").map_err(|e| Error::Sql(e))?)
	}
}