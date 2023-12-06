pub mod reprs;

use rocket::{fairing::{AdHoc, self}, Rocket, Build, futures::future::try_join_all, error};
use rocket_db_pools::Database;
use sqlx::Error;

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

	async fn get_user(&self, user_name: &str) -> Result<User, Error> {
		let res = sqlx::query("select * from main_db.auth where user_name = ?").bind(user_name).fetch_one(&**self).await?;

		todo!()
	}

	async fn create_user(&self, user: User) {
		todo!()
	}
}